//! Groups of principals.

use std::time::Duration;

use crate::client::{request, Client};
use crate::error::Error;
use crate::list::{failed, paginate, ListOptions, ListStream};
use crate::raw::permissions::v1 as pb;

/// Prefix of the principal ID that grants to a group are made to.
pub const GROUP_PRINCIPAL_PREFIX: &str = "group:";

/// Most principals accepted by one add/remove members call.
const MAX_MEMBERS_PER_CALL: usize = 100;

fn validate_slug(slug: &str) -> Result<&str, Error> {
    let ok = (1..=64).contains(&slug.len())
        && slug
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    if ok {
        Ok(slug)
    } else {
        Err(Error::validation(format!(
            "permio: invalid group name {slug:?}: use 1-64 lower-case letters, digits and hyphens"
        )))
    }
}

/// The principal ID of a group, `"group:<slug>"`. Grant permissions and roles
/// to it, or check it, like any other principal; every member of the group
/// holds what the group is granted.
///
/// ```
/// assert_eq!(permio::group("editors").unwrap(), "group:editors");
/// ```
///
/// # Errors
///
/// [`Error::Validation`] unless `slug` is 1-64 lower-case letters, digits and
/// hyphens.
pub fn group(slug: &str) -> Result<String, Error> {
    Ok(format!("{GROUP_PRINCIPAL_PREFIX}{}", validate_slug(slug)?))
}

// Group methods accept a slug or a "group:<slug>" principal ID.
fn slug_of(name_or_principal: &str) -> Result<String, Error> {
    let name = name_or_principal
        .strip_prefix(GROUP_PRINCIPAL_PREFIX)
        .unwrap_or(name_or_principal);
    validate_slug(name).map(str::to_string)
}

fn principal_list<I, S>(principals: I) -> Result<Vec<String>, Error>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut values: Vec<String> = Vec::new();
    for p in principals {
        let p = p.into();
        if p.is_empty() {
            return Err(Error::validation(
                "permio: principal IDs must be non-empty strings",
            ));
        }
        if !values.contains(&p) {
            values.push(p);
        }
    }
    if values.is_empty() {
        return Err(Error::validation(
            "permio: at least one principal is required",
        ));
    }
    if values.len() > MAX_MEMBERS_PER_CALL {
        return Err(Error::validation(format!(
            "permio: at most {MAX_MEMBERS_PER_CALL} principals per call"
        )));
    }
    Ok(values)
}

/// Result of [`Groups::add_members`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct AddMembersResult {
    /// Principals that were added, in request order.
    pub added: Vec<String>,
    /// Principals that were already members, in request order.
    pub existing: Vec<String>,
}

/// Result of [`Groups::remove_members`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct RemoveMembersResult {
    /// Principals that were removed, in request order.
    pub removed: Vec<String>,
    /// Principals that were not members, in request order.
    pub not_members: Vec<String>,
}

/// Groups of principals, from [`Client::groups`]. A group is referenced as the
/// principal `group:<slug>` (see [`group`]): grant to it and every member
/// holds the grant. Groups cannot contain groups.
///
/// Every method that takes a group accepts its slug (`"editors"`) or its
/// principal ID (`"group:editors"`).
///
/// Calls use the client's default project and timeouts. `Groups` is a cheap
/// handle, so set them per call with [`project`](Self::project) and
/// [`timeout`](Self::timeout):
///
/// ```no_run
/// # async fn demo(client: permio::Client) -> Result<(), permio::Error> {
/// use std::time::Duration;
/// client
///     .groups()
///     .project("other-app")
///     .timeout(Duration::from_secs(3))
///     .delete("editors")
///     .await?;
/// # Ok(()) }
/// ```
///
/// For the list methods, [`ListOptions::project`] and
/// [`ListOptions::timeout`] take precedence over these.
#[derive(Debug, Clone)]
pub struct Groups {
    client: Client,
    project: Option<String>,
    timeout: Option<Duration>,
}

impl Groups {
    pub(crate) fn new(client: Client) -> Self {
        Groups {
            client,
            project: None,
            timeout: None,
        }
    }

    /// The same handle, sending calls to `project` instead of the client's
    /// default project.
    #[must_use]
    pub fn project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }

    /// The same handle, with an overall deadline per call across retries
    /// (for list streams: per page request). Overrides the client's
    /// [`call_timeout`](crate::ClientBuilder::call_timeout);
    /// [`Duration::ZERO`] means none.
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    fn resolve_project(&self, project: Option<&str>) -> Result<String, Error> {
        self.client
            .resolve_project(project.or(self.project.as_deref()))
    }

    fn request<T>(&self, message: T) -> tonic::Request<T> {
        request(message, self.timeout)
    }

    /// Creates a group.
    ///
    /// # Errors
    ///
    /// [`Error::Other`] with code `AlreadyExists` if the project already has a
    /// group with this name.
    pub async fn create(&self, name: &str, description: Option<&str>) -> Result<pb::Group, Error> {
        let req = pb::CreateGroupRequest {
            name: slug_of(name)?,
            project_name: self.resolve_project(None)?,
            description: description.unwrap_or_default().to_string(),
        };
        Ok(self
            .client
            .rpc()
            .create_group(self.request(req))
            .await?
            .into_inner())
    }

    /// Returns a group.
    ///
    /// # Errors
    ///
    /// [`Error::NotFound`] if it does not exist.
    pub async fn get(&self, name: &str) -> Result<pb::Group, Error> {
        let req = pb::GetGroupRequest {
            name: slug_of(name)?,
            project_name: self.resolve_project(None)?,
        };
        Ok(self
            .client
            .rpc()
            .get_group(self.request(req))
            .await?
            .into_inner())
    }

    /// Replaces a group's description. A group's name cannot be changed.
    ///
    /// # Errors
    ///
    /// [`Error::NotFound`] if it does not exist.
    pub async fn update(&self, name: &str, description: &str) -> Result<pb::Group, Error> {
        let req = pb::UpdateGroupRequest {
            name: slug_of(name)?,
            project_name: self.resolve_project(None)?,
            description: description.to_string(),
        };
        Ok(self
            .client
            .rpc()
            .update_group(self.request(req))
            .await?
            .into_inner())
    }

    /// Deletes a group, its memberships and every grant made to it. Succeeds
    /// whether or not the group existed.
    ///
    /// # Errors
    ///
    /// Transport, auth and permission errors.
    pub async fn delete(&self, name: &str) -> Result<(), Error> {
        let req = pb::DeleteGroupRequest {
            name: slug_of(name)?,
            project_name: self.resolve_project(None)?,
        };
        self.client.rpc().delete_group(self.request(req)).await?;
        Ok(())
    }

    /// Streams every group in the project, ordered by name.
    pub fn list(&self, options: ListOptions) -> ListStream<pb::Group> {
        let project_name = match self.resolve_project(options.project.as_deref()) {
            Ok(p) => p,
            Err(e) => return failed(e),
        };
        let rpc = self.client.rpc();
        let limit = options.limit().unsigned_abs();
        let timeout = options.timeout.or(self.timeout);
        paginate(limit, move |pagination| {
            let mut rpc = rpc.clone();
            let req = request(
                pb::ListGroupsRequest {
                    project_name: project_name.clone(),
                    pagination: Some(pagination),
                    search: options.search.clone(),
                },
                timeout,
            );
            async move {
                let res = rpc.list_groups(req).await?.into_inner();
                Ok((res.groups, res.pagination))
            }
        })
    }

    /// Adds up to 100 principals to a group. Principals that are already
    /// members are reported in `existing`.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`] for group principals (`group:...`), because
    /// groups cannot be nested, or for empty input.
    pub async fn add_members<I, S>(
        &self,
        name: &str,
        principals: I,
    ) -> Result<AddMembersResult, Error>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let name = slug_of(name)?;
        let principal_ids = principal_list(principals)?;
        if principal_ids
            .iter()
            .any(|p| p.starts_with(GROUP_PRINCIPAL_PREFIX))
        {
            return Err(Error::validation("permio: groups cannot contain groups"));
        }
        let req = pb::AddGroupMembersRequest {
            project_name: self.resolve_project(None)?,
            name,
            principal_ids,
        };
        let res = self
            .client
            .rpc()
            .add_group_members(self.request(req))
            .await?
            .into_inner();
        Ok(AddMembersResult {
            added: res.added_principal_ids,
            existing: res.existing_principal_ids,
        })
    }

    /// Removes up to 100 principals from a group. Principals that were not
    /// members are reported in `not_members`.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`] for empty input.
    pub async fn remove_members<I, S>(
        &self,
        name: &str,
        principals: I,
    ) -> Result<RemoveMembersResult, Error>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let req = pb::RemoveGroupMembersRequest {
            name: slug_of(name)?,
            principal_ids: principal_list(principals)?,
            project_name: self.resolve_project(None)?,
        };
        let res = self
            .client
            .rpc()
            .remove_group_members(self.request(req))
            .await?
            .into_inner();
        Ok(RemoveMembersResult {
            removed: res.removed_principal_ids,
            not_members: res.not_member_principal_ids,
        })
    }

    /// Streams a group's members, ordered by principal ID.
    pub fn list_members(&self, name: &str, options: ListOptions) -> ListStream<pb::GroupMember> {
        let (name, project_name) = match slug_of(name)
            .and_then(|n| Ok((n, self.resolve_project(options.project.as_deref())?)))
        {
            Ok(v) => v,
            Err(e) => return failed(e),
        };
        let rpc = self.client.rpc();
        let limit = options.limit().unsigned_abs();
        let timeout = options.timeout.or(self.timeout);
        paginate(limit, move |pagination| {
            let mut rpc = rpc.clone();
            let req = request(
                pb::ListGroupMembersRequest {
                    project_name: project_name.clone(),
                    name: name.clone(),
                    pagination: Some(pagination),
                },
                timeout,
            );
            async move {
                let res = rpc.list_group_members(req).await?.into_inner();
                Ok((res.members, res.pagination))
            }
        })
    }

    /// Streams the groups `principal` is a member of, ordered by name.
    pub fn list_for_principal(
        &self,
        principal: &str,
        options: ListOptions,
    ) -> ListStream<pb::Group> {
        if principal.is_empty() {
            return failed(Error::validation("permio: principal is required"));
        }
        let project_name = match self.resolve_project(options.project.as_deref()) {
            Ok(p) => p,
            Err(e) => return failed(e),
        };
        let rpc = self.client.rpc();
        let principal = principal.to_string();
        let limit = options.limit().unsigned_abs();
        let timeout = options.timeout.or(self.timeout);
        paginate(limit, move |pagination| {
            let mut rpc = rpc.clone();
            let req = request(
                pb::ListGroupsForPrincipalRequest {
                    project_name: project_name.clone(),
                    principal_id: principal.clone(),
                    pagination: Some(pagination),
                },
                timeout,
            );
            async move {
                let res = rpc.list_groups_for_principal(req).await?.into_inner();
                Ok((res.groups, res.pagination))
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_principal() {
        assert_eq!(group("editors").unwrap(), "group:editors");
        assert!(group("Editors").is_err());
        assert!(group("").is_err());
        assert!(group(&"a".repeat(65)).is_err());
        assert_eq!(slug_of("group:a-1").unwrap(), "a-1");
    }
}
