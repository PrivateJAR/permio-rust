//! The [`Client`] and its builder.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use futures_util::{StreamExt, TryStreamExt};
use http::HeaderValue;
use tonic::transport::{Channel, Endpoint};

use crate::access::{AccessEntry, AccessScope, AccessScopes, ScopeKind};
use crate::apikey::{validate_api_key, ApiKeyKind};
use crate::error::{Error, MissingPermission};
use crate::groups::Groups;
use crate::list::{collect_capped, failed, paginate, paginate_capped, ListOptions, ListStream};
use crate::raw::organisation::v1::organisation_service_client::OrganisationServiceClient;
use crate::raw::permissions::v1 as pb;
use crate::raw::permissions::v1::permissions_service_client::PermissionsServiceClient;
use crate::resource::validate_resource_uri;
use crate::transport::{CallTimeout, RetryConfig, Transport};

/// Production API.
pub const DEFAULT_ENDPOINT: &str = "https://api.perms.io";
/// Development API.
pub const DEV_ENDPOINT: &str = "https://dev-api.perms.io";
/// Environment variable read for the API key when none is given to the builder.
pub const API_KEY_ENV: &str = "PERMIO_API_KEY";
/// Default per-attempt timeout.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

/// Default cap on the results [`Client::who_has_access`] and
/// [`Client::list_access`] collect into memory; see
/// [`WhoHasAccess::max_results`] and [`ListAccess::max_results`].
pub const DEFAULT_MAX_RESULTS: usize = 10_000;

/// Default cap on the resources one [`Client::batch_check`] call accepts; see
/// [`BatchCheck::max_resources`].
pub const DEFAULT_MAX_BATCH_RESOURCES: usize = 10_000;

/// Most resources or permissions accepted by one `Check`/`BatchCheck` call.
const MAX_PER_CALL: usize = 100;

/// Most `BatchCheck` calls one [`Client::batch_check`] has in flight at once.
const BATCH_CHECK_CONCURRENCY: usize = 4;

/// Wraps a request message, attaching the call's overall timeout if it has one.
pub(crate) fn request<T>(message: T, timeout: Option<Duration>) -> tonic::Request<T> {
    let mut req = tonic::Request::new(message);
    if let Some(timeout) = timeout {
        req.extensions_mut().insert(CallTimeout::new(timeout));
    }
    req
}

/// Builder for [`Client`]; see [`Client::builder`].
#[derive(Default)]
pub struct ClientBuilder {
    api_key: Option<String>,
    endpoint: Option<String>,
    project: Option<String>,
    timeout: Option<Duration>,
    call_timeout: Option<Duration>,
    list_timeout: Option<Duration>,
    retry: Option<RetryConfig>,
    channel: Option<Channel>,
}

impl fmt::Debug for ClientBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientBuilder")
            .field("api_key", &self.api_key.as_ref().map(|_| "<redacted>"))
            .field("endpoint", &self.endpoint)
            .field("project", &self.project)
            .field("timeout", &self.timeout)
            .field("call_timeout", &self.call_timeout)
            .field("list_timeout", &self.list_timeout)
            .field("retry", &self.retry)
            .field("channel", &self.channel.is_some())
            .finish()
    }
}

impl ClientBuilder {
    /// The API key from the dashboard (`perms_...`), or a legacy JWT key.
    /// Defaults to the `PERMIO_API_KEY` environment variable.
    #[must_use]
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// API endpoint URL. Defaults to production, [`DEFAULT_ENDPOINT`]; use
    /// [`DEV_ENDPOINT`] for the development API.
    #[must_use]
    pub fn endpoint(mut self, url: impl Into<String>) -> Self {
        self.endpoint = Some(url.into());
        self
    }

    /// Default project (`project_name`) for every call. Without one, calls
    /// that need a project fail with [`Error::Config`]; use
    /// [`Client::with_project`] to switch per call.
    #[must_use]
    pub fn project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }

    /// Timeout per attempt. Default 10 s; [`Duration::ZERO`] disables it.
    /// See [`call_timeout`](Self::call_timeout) for a deadline across retries.
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Default overall deadline for each call, across every attempt and the
    /// backoff between retries (for list streams: for each page request).
    /// Without one (the default), a read can take up to `retries + 1`
    /// per-attempt timeouts plus backoff. A call's own `timeout` option
    /// overrides it; [`Duration::ZERO`] here means none.
    #[must_use]
    pub fn call_timeout(mut self, timeout: Duration) -> Self {
        self.call_timeout = Some(timeout);
        self
    }

    /// Timeout per attempt for list page requests (the paginated RPCs behind
    /// every [`ListStream`], [`Client::who_has_access`] and
    /// [`Client::list_access`], including raw clients). Defaults to
    /// [`timeout`](Self::timeout); [`Duration::ZERO`] disables it. Large
    /// pages can take longer than a check, so this can be set separately.
    #[must_use]
    pub fn list_timeout(mut self, timeout: Duration) -> Self {
        self.list_timeout = Some(timeout);
        self
    }

    /// Retries for idempotent reads (default 2). Writes are never retried.
    #[must_use]
    pub fn retries(mut self, max_retries: u32) -> Self {
        self.retry = Some(RetryConfig {
            max_retries,
            ..self.retry.unwrap_or_default()
        });
        self
    }

    /// Full retry settings; see [`RetryConfig`].
    #[must_use]
    pub fn retry_config(mut self, retry: RetryConfig) -> Self {
        self.retry = Some(retry);
        self
    }

    /// Advanced: a tonic [`Channel`] to use instead of connecting to
    /// `endpoint`, e.g. one connected to an in-process test server.
    /// Authentication, timeouts and retries still apply.
    #[must_use]
    pub fn channel(mut self, channel: Channel) -> Self {
        self.channel = Some(channel);
        self
    }

    /// Validates the configuration and builds the client. The connection is
    /// made lazily, on the first call.
    ///
    /// Must be called from within a Tokio runtime.
    ///
    /// # Errors
    ///
    /// [`Error::Config`] if the API key is missing or malformed, the endpoint
    /// is not an http(s) URL, an https endpoint is used without the `tls`
    /// feature, or there is no Tokio runtime.
    pub fn build(self) -> Result<Client, Error> {
        let api_key = match self.api_key {
            Some(key) => key,
            None => std::env::var(API_KEY_ENV).map_err(|_| {
                Error::config(format!(
                    "permio: no API key; pass .api_key() or set {API_KEY_ENV}"
                ))
            })?,
        };
        let api_key_kind = validate_api_key(&api_key)?;
        let mut authorization = HeaderValue::from_str(&format!("Bearer {api_key}"))
            .map_err(|_| Error::config("permio: API key contains invalid characters"))?;
        authorization.set_sensitive(true);

        let timeout = self.timeout.unwrap_or(DEFAULT_TIMEOUT);
        let timeout = (!timeout.is_zero()).then_some(timeout);
        let list_timeout = match self.list_timeout {
            Some(t) => (!t.is_zero()).then_some(t),
            None => timeout,
        };

        let endpoint = self
            .endpoint
            .unwrap_or_else(|| DEFAULT_ENDPOINT.to_string())
            .trim_end_matches('/')
            .to_string();
        let channel = match self.channel {
            Some(channel) => channel,
            None => connect(&endpoint, timeout)?,
        };
        let transport = Transport::new(
            channel,
            authorization,
            timeout,
            list_timeout,
            self.call_timeout.filter(|d| !d.is_zero()),
            self.retry.unwrap_or_default(),
        );
        Ok(Client {
            transport,
            project: self.project.map(Arc::from),
            endpoint: Arc::from(endpoint),
            api_key_kind,
        })
    }
}

fn connect(url: &str, timeout: Option<Duration>) -> Result<Channel, Error> {
    let https = url.starts_with("https://");
    if !https && !url.starts_with("http://") {
        return Err(Error::config(format!(
            "permio: endpoint must be an http(s) URL, got {url:?}"
        )));
    }
    let mut endpoint = Endpoint::from_shared(url.to_string())
        .map_err(|e| Error::config(format!("permio: invalid endpoint {url:?}: {e}")))?
        .user_agent(concat!("permio-rust/", env!("CARGO_PKG_VERSION")))
        .map_err(|e| Error::config(format!("permio: {e}")))?;
    if let Some(timeout) = timeout {
        endpoint = endpoint.connect_timeout(timeout);
    }
    if https {
        #[cfg(feature = "tls")]
        {
            endpoint = endpoint
                .tls_config(tonic::transport::ClientTlsConfig::new().with_enabled_roots())
                .map_err(|e| Error::config(format!("permio: TLS setup failed: {e}")))?;
        }
        #[cfg(not(feature = "tls"))]
        {
            return Err(Error::config(
                "permio: https endpoints need the `tls` feature of the permio crate",
            ));
        }
    }
    if tokio::runtime::Handle::try_current().is_err() {
        return Err(Error::config(
            "permio: Client::builder().build() must be called within a Tokio runtime",
        ));
    }
    Ok(endpoint.connect_lazy())
}

/// Client for the perms.io API.
///
/// Cheap to clone: clones share the connection.
///
/// ```no_run
/// # async fn demo() -> Result<(), permio::Error> {
/// let client = permio::Client::builder().project("my-app").build()?; // key from PERMIO_API_KEY
/// let doc = permio::resource!("docs", 42)?;
/// if client.check("user-1", "docs.read", &doc).await? {
///     // ...
/// }
/// # Ok(()) }
/// ```
#[derive(Clone)]
pub struct Client {
    transport: Transport,
    project: Option<Arc<str>>,
    endpoint: Arc<str>,
    api_key_kind: ApiKeyKind,
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("endpoint", &self.endpoint)
            .field("project", &self.project)
            .field("api_key_kind", &self.api_key_kind)
            .finish_non_exhaustive()
    }
}

/// What to check; see [`Client::check_detailed`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Check {
    principal: String,
    resources: Vec<String>,
    permissions: Vec<String>,
    project: Option<String>,
    timeout: Option<Duration>,
}

impl Check {
    /// Checks `principal` (one of your user/service IDs, or a `group:<slug>`).
    #[must_use]
    pub fn new(principal: impl Into<String>) -> Self {
        Check {
            principal: principal.into(),
            ..Check::default()
        }
    }

    /// Adds a resource URI, e.g. from [`resource!`](crate::resource!).
    #[must_use]
    pub fn resource(mut self, resource: impl Into<String>) -> Self {
        self.resources.push(resource.into());
        self
    }

    /// Adds several resource URIs; every one is checked.
    #[must_use]
    pub fn resources<I, S>(mut self, resources: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.resources.extend(resources.into_iter().map(Into::into));
        self
    }

    /// Adds a permission name.
    #[must_use]
    pub fn permission(mut self, permission: impl Into<String>) -> Self {
        self.permissions.push(permission.into());
        self
    }

    /// Adds several permission names.
    #[must_use]
    pub fn permissions<I, S>(mut self, permissions: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.permissions
            .extend(permissions.into_iter().map(Into::into));
        self
    }

    /// Checks in this project instead of the client's.
    #[must_use]
    pub fn project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }

    /// Overall deadline for this call, across retries. Overrides the client's
    /// [`call_timeout`](ClientBuilder::call_timeout); [`Duration::ZERO`]
    /// means none.
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

/// Result of [`Client::check_detailed`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CheckResult {
    /// True when the principal holds every permission on every resource.
    pub allowed: bool,
    /// What is missing, per resource, in request order. Empty when allowed.
    pub missing: Vec<MissingPermission>,
}

/// What to check in one [`Client::batch_check`] call.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct BatchCheck {
    principal: String,
    resources: Vec<String>,
    permissions: Vec<String>,
    project: Option<String>,
    timeout: Option<Duration>,
    max_resources: Option<usize>,
}

impl BatchCheck {
    /// Checks `principal` on each resource separately.
    #[must_use]
    pub fn new(principal: impl Into<String>) -> Self {
        BatchCheck {
            principal: principal.into(),
            ..BatchCheck::default()
        }
    }

    /// Adds resource URIs, each answered separately. More than 100 are split
    /// into several `BatchCheck` calls, at most four in flight at once.
    #[must_use]
    pub fn resources<I, S>(mut self, resources: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.resources.extend(resources.into_iter().map(Into::into));
        self
    }

    /// Adds a resource URI.
    #[must_use]
    pub fn resource(mut self, resource: impl Into<String>) -> Self {
        self.resources.push(resource.into());
        self
    }

    /// Adds a permission; a resource is allowed only with all of them.
    #[must_use]
    pub fn permission(mut self, permission: impl Into<String>) -> Self {
        self.permissions.push(permission.into());
        self
    }

    /// Adds several permissions (at most 100 in total).
    #[must_use]
    pub fn permissions<I, S>(mut self, permissions: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.permissions
            .extend(permissions.into_iter().map(Into::into));
        self
    }

    /// Checks in this project instead of the client's.
    #[must_use]
    pub fn project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }

    /// Overall deadline for each `BatchCheck` request, across its retries.
    /// Overrides the client's [`call_timeout`](ClientBuilder::call_timeout);
    /// [`Duration::ZERO`] means none.
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Most resources this call accepts, counted as given (before
    /// de-duplication). Default [`DEFAULT_MAX_BATCH_RESOURCES`] (10,000);
    /// more is an [`Error::Validation`] before anything is sent. Must be at
    /// least 1.
    #[must_use]
    pub fn max_resources(mut self, max: usize) -> Self {
        self.max_resources = Some(max);
        self
    }
}

/// One resource's result from [`Client::batch_check`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct BatchCheckEntry {
    /// The resource URI.
    pub resource: String,
    /// True when the principal holds every requested permission on `resource`.
    pub allowed: bool,
    /// The requested permissions the principal lacks on `resource`.
    pub missing: Vec<String>,
}

/// Options for [`Client::who_has_access`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct WhoHasAccess {
    resource: String,
    permission: Option<String>,
    expand_groups: bool,
    page_size: Option<u32>,
    project: Option<String>,
    timeout: Option<Duration>,
    max_results: Option<usize>,
}

impl WhoHasAccess {
    /// Looks at `resource`. A `_` segment here is literal.
    #[must_use]
    pub fn new(resource: impl Into<String>) -> Self {
        WhoHasAccess {
            resource: resource.into(),
            ..WhoHasAccess::default()
        }
    }

    /// Only counts grants that confer this permission, so a principal is
    /// listed exactly when a check passes for it through its own grants (or,
    /// for entries with `via_group`, through that group).
    #[must_use]
    pub fn permission(mut self, permission: impl Into<String>) -> Self {
        self.permission = Some(permission.into());
        self
    }

    /// Also lists every member of each listed group, once per group, with
    /// `via_group` set.
    #[must_use]
    pub fn expand_groups(mut self, expand: bool) -> Self {
        self.expand_groups = expand;
        self
    }

    /// Most entries [`Client::who_has_access`] collects; more is
    /// [`Error::TooManyResults`]. Default [`DEFAULT_MAX_RESULTS`] (10,000);
    /// must be at least 1. Ignored by [`Client::who_has_access_stream`].
    #[must_use]
    pub fn max_results(mut self, max: usize) -> Self {
        self.max_results = Some(max);
        self
    }

    /// Items requested per page (1..=100, default 100).
    #[must_use]
    pub fn page_size(mut self, size: u32) -> Self {
        self.page_size = Some(size);
        self
    }

    /// Uses this project instead of the client's.
    #[must_use]
    pub fn project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }

    /// Overall deadline for each page request, across its retries. Overrides the client's
    /// [`call_timeout`](ClientBuilder::call_timeout); [`Duration::ZERO`]
    /// means none.
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

/// What to ask in [`Client::list_access`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListAccess {
    principal: String,
    permission: String,
    under: String,
    page_size: Option<u32>,
    project: Option<String>,
    timeout: Option<Duration>,
    max_results: Option<usize>,
}

impl ListAccess {
    /// Where may `principal` use `permission` at or below the URI `under`?
    #[must_use]
    pub fn new(
        principal: impl Into<String>,
        permission: impl Into<String>,
        under: impl Into<String>,
    ) -> Self {
        ListAccess {
            principal: principal.into(),
            permission: permission.into(),
            under: under.into(),
            ..ListAccess::default()
        }
    }

    /// Scopes requested per page (1..=100, default 100).
    #[must_use]
    pub fn page_size(mut self, size: u32) -> Self {
        self.page_size = Some(size);
        self
    }

    /// Most scopes [`Client::list_access`] collects; more is
    /// [`Error::TooManyResults`]. Default [`DEFAULT_MAX_RESULTS`] (10,000);
    /// must be at least 1. Ignored by [`Client::list_access_stream`].
    #[must_use]
    pub fn max_results(mut self, max: usize) -> Self {
        self.max_results = Some(max);
        self
    }

    /// Uses this project instead of the client's.
    #[must_use]
    pub fn project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }

    /// Overall deadline for each page request, across its retries. Overrides the client's
    /// [`call_timeout`](ClientBuilder::call_timeout); [`Duration::ZERO`]
    /// means none.
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

/// What to grant or revoke; see [`Client::grant`] and [`Client::revoke`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Grant {
    principal: String,
    resource: String,
    permissions: Vec<String>,
    roles: Vec<String>,
    project: Option<String>,
    timeout: Option<Duration>,
}

impl Grant {
    /// Grants to (or revokes from) `principal` on `resource`. Use
    /// [`group`](crate::group) for a group principal.
    #[must_use]
    pub fn new(principal: impl Into<String>, resource: impl Into<String>) -> Self {
        Grant {
            principal: principal.into(),
            resource: resource.into(),
            ..Grant::default()
        }
    }

    /// Adds a permission name.
    #[must_use]
    pub fn permission(mut self, permission: impl Into<String>) -> Self {
        self.permissions.push(permission.into());
        self
    }

    /// Adds several permission names.
    #[must_use]
    pub fn permissions<I, S>(mut self, permissions: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.permissions
            .extend(permissions.into_iter().map(Into::into));
        self
    }

    /// Adds a role ID (see [`Client::list_roles`]).
    #[must_use]
    pub fn role(mut self, role: impl Into<String>) -> Self {
        self.roles.push(role.into());
        self
    }

    /// Adds several role IDs.
    #[must_use]
    pub fn roles<I, S>(mut self, roles: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.roles.extend(roles.into_iter().map(Into::into));
        self
    }

    /// Uses this project instead of the client's.
    #[must_use]
    pub fn project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }

    /// Overall deadline for this call, across retries (writes are never retried). Overrides the client's
    /// [`call_timeout`](ClientBuilder::call_timeout); [`Duration::ZERO`]
    /// means none.
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

/// The kind of credential the client is using, as reported by [`Client::who_am_i`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PrincipalKind {
    /// A dashboard user.
    User,
    /// An API key.
    ApiKey,
    /// A service account.
    ServiceAccount,
    /// Something this SDK version does not know.
    Unknown,
}

/// Access level of an API key on its project, as reported by [`Client::who_am_i`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ApiKeyAccess {
    /// Checks and reads.
    Check,
    /// Also writes, grant and revoke.
    Full,
    /// Its rights were changed by hand.
    Unknown,
}

/// The caller, as returned by [`Client::who_am_i`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct WhoAmI {
    /// The caller's principal ID; for an API key, the key's ID.
    pub principal_id: String,
    /// The caller's organisation. Empty for a user who has not joined one yet.
    pub organisation_id: String,
    /// What kind of credential the client is using.
    pub kind: PrincipalKind,
    /// The project an API key belongs to; it can only be used there.
    pub project: Option<String>,
    /// For an API key, its access level; `None` for other credentials.
    pub api_key_access: Option<ApiKeyAccess>,
    /// The user's email address, when the caller is a user and has one.
    pub email: Option<String>,
}

fn non_empty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

fn unique(values: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::with_capacity(values.len());
    let mut out: Vec<String> = Vec::with_capacity(values.len());
    for v in values {
        if seen.insert(v.clone()) {
            out.push(v);
        }
    }
    out
}

fn max_results(max: Option<usize>) -> Result<usize, Error> {
    match max.unwrap_or(DEFAULT_MAX_RESULTS) {
        0 => Err(Error::validation("permio: max_results must be at least 1")),
        n => Ok(n),
    }
}

// At least one and at most 100 distinct, non-empty values.
fn pick(values: Vec<String>, what: &str) -> Result<Vec<String>, Error> {
    let values = unique(values);
    if values.is_empty() {
        return Err(Error::validation(format!(
            "permio: at least one {what} is required"
        )));
    }
    if values.len() > MAX_PER_CALL {
        return Err(Error::validation(format!(
            "permio: at most {MAX_PER_CALL} {what}s per check"
        )));
    }
    if values.iter().any(String::is_empty) {
        return Err(Error::validation(format!(
            "permio: {what}s must be non-empty strings"
        )));
    }
    Ok(values)
}

fn names(values: Vec<String>, what: &str) -> Result<Vec<String>, Error> {
    let values = unique(values);
    if values.iter().any(String::is_empty) {
        return Err(Error::validation(format!(
            "permio: {what}s must be non-empty strings"
        )));
    }
    Ok(values)
}

fn require(value: &str, what: &str) -> Result<(), Error> {
    if value.is_empty() {
        Err(Error::validation(format!("permio: {what} is required")))
    } else {
        Ok(())
    }
}

impl Client {
    /// Starts building a client.
    ///
    /// ```no_run
    /// # fn demo() -> Result<(), permio::Error> {
    /// let client = permio::Client::builder()
    ///     .api_key("perms_...")
    ///     .endpoint(permio::DEV_ENDPOINT)
    ///     .project("my-app")
    ///     .timeout(std::time::Duration::from_secs(5))
    ///     .retries(3)
    ///     .build()?;
    /// # Ok(()) }
    /// ```
    #[must_use]
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// A client for production configured from the environment
    /// (`PERMIO_API_KEY`), with `project` as the default project.
    ///
    /// # Errors
    ///
    /// See [`ClientBuilder::build`].
    pub fn from_env(project: impl Into<String>) -> Result<Client, Error> {
        Client::builder().project(project).build()
    }

    /// A clone of this client whose default project is `project`.
    #[must_use]
    pub fn with_project(&self, project: impl Into<String>) -> Client {
        Client {
            project: Some(Arc::from(project.into())),
            ..self.clone()
        }
    }

    /// A clone of this client whose default overall deadline per call is
    /// `timeout`; see [`ClientBuilder::call_timeout`]. Also applies to the
    /// raw clients it hands out.
    #[must_use]
    pub fn with_call_timeout(&self, timeout: Duration) -> Client {
        Client {
            transport: self
                .transport
                .with_call_timeout(Some(timeout).filter(|d| !d.is_zero())),
            ..self.clone()
        }
    }

    /// The default project, if configured.
    #[must_use]
    pub fn project(&self) -> Option<&str> {
        self.project.as_deref()
    }

    /// The endpoint URL this client was configured with.
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Whether the configured key is an opaque `perms_` key or a legacy JWT.
    #[must_use]
    pub fn api_key_kind(&self) -> ApiKeyKind {
        self.api_key_kind
    }

    /// The raw generated `PermissionsService` client, sharing this client's
    /// connection, authentication, timeouts and retries. Errors are
    /// [`tonic::Status`]; convert them with `permio::Error::from`.
    #[must_use]
    pub fn raw_permissions(&self) -> PermissionsServiceClient<Transport> {
        PermissionsServiceClient::new(self.transport.clone())
    }

    /// The raw generated `OrganisationService` client; see
    /// [`raw_permissions`](Self::raw_permissions).
    #[must_use]
    pub fn raw_organisations(&self) -> OrganisationServiceClient<Transport> {
        OrganisationServiceClient::new(self.transport.clone())
    }

    /// Groups of principals: create them, manage members, list them.
    #[must_use]
    pub fn groups(&self) -> Groups {
        Groups::new(self.clone())
    }

    pub(crate) fn resolve_project(&self, project: Option<&str>) -> Result<String, Error> {
        project
            .or(self.project.as_deref())
            .filter(|p| !p.is_empty())
            .map(str::to_string)
            .ok_or_else(|| {
                Error::config(
                    "permio: no project given; set .project() on the builder or use with_project()",
                )
            })
    }

    pub(crate) fn rpc(&self) -> PermissionsServiceClient<Transport> {
        self.raw_permissions()
    }

    /// True when `principal` holds `permission` on `resource` (directly, via
    /// a role, a group, an ancestor URI or a `_` wildcard grant). A denial is
    /// a normal result, not an error.
    ///
    /// # Errors
    ///
    /// [`Error::Permission`] if the API key lacks `permission.check`;
    /// [`Error::Validation`] for malformed URIs or empty inputs; transport and
    /// auth errors.
    pub async fn check(
        &self,
        principal: &str,
        permission: &str,
        resource: &str,
    ) -> Result<bool, Error> {
        Ok(self
            .check_detailed(
                Check::new(principal)
                    .permission(permission)
                    .resource(resource),
            )
            .await?
            .allowed)
    }

    /// True when `principal` holds every one of `permissions` on `resource`.
    ///
    /// # Errors
    ///
    /// As [`check`](Self::check).
    pub async fn check_all<I, S>(
        &self,
        principal: &str,
        permissions: I,
        resource: &str,
    ) -> Result<bool, Error>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Ok(self
            .check_detailed(
                Check::new(principal)
                    .permissions(permissions)
                    .resource(resource),
            )
            .await?
            .allowed)
    }

    /// True when `principal` holds at least one of `permissions` on
    /// `resource`. Uses a single `Check` call.
    ///
    /// # Errors
    ///
    /// As [`check`](Self::check).
    pub async fn check_any<I, S>(
        &self,
        principal: &str,
        permissions: I,
        resource: &str,
    ) -> Result<bool, Error>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let check = Check::new(principal)
            .permissions(permissions)
            .resource(resource);
        let wanted = unique(check.permissions.clone());
        let result = self.check_detailed(check).await?;
        if result.allowed {
            return Ok(true);
        }
        // Denied without saying what is missing: nothing shows that any one
        // permission is held, so fail closed.
        if result.missing.is_empty() {
            return Ok(false);
        }
        Ok(result
            .missing
            .iter()
            .all(|m| wanted.iter().any(|p| !m.permissions.contains(p))))
    }

    /// Checks whether the principal holds every permission on every resource
    /// and returns what is missing.
    ///
    /// # Errors
    ///
    /// As [`check`](Self::check).
    pub async fn check_detailed(&self, check: Check) -> Result<CheckResult, Error> {
        require(&check.principal, "principal")?;
        let resource_uris = pick(check.resources, "resource")?;
        for uri in &resource_uris {
            validate_resource_uri(uri)?;
        }
        let permissions = pick(check.permissions, "permission")?;
        let project_name = self.resolve_project(check.project.as_deref())?;
        let res = self
            .rpc()
            .check(request(
                pb::CheckPermissionRequest {
                    project_name,
                    principal_id: check.principal,
                    resource_uris,
                    permissions,
                },
                check.timeout,
            ))
            .await?
            .into_inner();
        Ok(CheckResult {
            allowed: res.passed,
            missing: res
                .missing_permissions
                .into_iter()
                .map(|m| MissingPermission {
                    resource: m.resource_uri,
                    permissions: m.missing_permissions,
                })
                .collect(),
        })
    }

    /// Succeeds when `principal` holds `permission` on `resource`.
    ///
    /// # Errors
    ///
    /// [`Error::Denied`] when it does not, with `missing` filled in; otherwise
    /// as [`check`](Self::check).
    pub async fn assert(
        &self,
        principal: &str,
        permission: &str,
        resource: &str,
    ) -> Result<(), Error> {
        self.assert_detailed(
            Check::new(principal)
                .permission(permission)
                .resource(resource),
        )
        .await
    }

    /// Succeeds when the principal holds every permission on every resource.
    ///
    /// # Errors
    ///
    /// [`Error::Denied`] when it does not; otherwise as [`check`](Self::check).
    pub async fn assert_detailed(&self, check: Check) -> Result<(), Error> {
        let principal = check.principal.clone();
        let result = self.check_detailed(check).await?;
        if result.allowed {
            Ok(())
        } else {
            Err(Error::denied(&principal, result.missing))
        }
    }

    /// Checks each resource separately and returns, per distinct resource in
    /// request order, whether the principal holds every requested permission
    /// on it. Use it to filter a list you already have in memory.
    ///
    /// Resources are sent in `BatchCheck` requests of 100, built as they are
    /// sent, with at most four in flight at once. The first failing request
    /// (in request order) fails the call and no further requests are sent.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`] for more than [`BatchCheck::max_resources`]
    /// resources (default 10,000); [`Error::Permission`] if the API key lacks
    /// `permission.check`; [`Error::Other`] if the API's answer does not cover
    /// every resource.
    pub async fn batch_check(&self, batch: BatchCheck) -> Result<Vec<BatchCheckEntry>, Error> {
        require(&batch.principal, "principal")?;
        let max_resources = batch.max_resources.unwrap_or(DEFAULT_MAX_BATCH_RESOURCES);
        if max_resources == 0 {
            return Err(Error::validation(
                "permio: max_resources must be at least 1",
            ));
        }
        if batch.resources.len() > max_resources {
            return Err(Error::validation(format!(
                "permio: batch_check got {} resources, more than max_resources ({max_resources}); split the call or raise max_resources",
                batch.resources.len()
            )));
        }
        let resource_uris = unique(batch.resources);
        if resource_uris.is_empty() {
            return Err(Error::validation(
                "permio: at least one resource is required",
            ));
        }
        for uri in &resource_uris {
            validate_resource_uri(uri)?;
        }
        let permissions = pick(batch.permissions, "permission")?;
        let project_name = self.resolve_project(batch.project.as_deref())?;
        let calls = resource_uris.chunks(MAX_PER_CALL).map(|chunk| {
            let mut rpc = self.rpc();
            let req = request(
                pb::BatchCheckRequest {
                    project_name: project_name.clone(),
                    principal_id: batch.principal.clone(),
                    resource_uris: chunk.to_vec(),
                    permissions: permissions.clone(),
                },
                batch.timeout,
            );
            async move { rpc.batch_check(req).await.map(tonic::Response::into_inner) }
        });
        let responses: Vec<pb::BatchCheckResponse> = futures_util::stream::iter(calls)
            .buffered(BATCH_CHECK_CONCURRENCY)
            .try_collect()
            .await?;
        let mut by_uri: HashMap<String, BatchCheckEntry> = HashMap::new();
        for r in responses.into_iter().flat_map(|r| r.results) {
            by_uri.insert(
                r.resource_uri.clone(),
                BatchCheckEntry {
                    allowed: r.allowed && r.missing_permissions.is_empty(),
                    resource: r.resource_uri,
                    missing: r.missing_permissions,
                },
            );
        }
        // Fail closed: every requested resource must have exactly its own answer.
        resource_uris
            .into_iter()
            .map(|uri| {
                by_uri
                    .remove(&uri)
                    .ok_or_else(|| Error::inconsistent(format!("no BatchCheck result for {uri:?}")))
            })
            .collect()
    }

    /// Answers "where may the principal use the permission at or below
    /// `under`?", following every page. Use the result to filter lists in
    /// your database: `covers_all` means no filter is needed,
    /// [`AccessScopes::to_prefixes`] gives path prefixes,
    /// [`AccessScopes::regex_pattern`] a regex, and [`AccessScopes::allows`]
    /// tests a single URI.
    ///
    /// Collects at most [`ListAccess::max_results`] scopes (default 10,000);
    /// use [`list_access_stream`](Self::list_access_stream) for more.
    ///
    /// ```no_run
    /// # async fn demo(client: permio::Client) -> Result<(), permio::Error> {
    /// let scopes = client
    ///     .list_access(permio::ListAccess::new("alice", "docs.read", "/docs"))
    ///     .await?;
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    ///
    /// [`Error::TooManyResults`] if there are more than `max_results` scopes
    /// (at most `max_results + 1` are fetched); [`Error::Permission`] unless
    /// the API key has `permission.check` on `under` and
    /// `principal.assignment.get` on the principal.
    pub async fn list_access(&self, query: ListAccess) -> Result<AccessScopes, Error> {
        let max = max_results(query.max_results)?;
        let under = query.under.clone();
        let mut pages = self.list_access_pages(query, Some(max.saturating_add(1)))?;
        let mut covers_all = false;
        let mut scopes = Vec::new();
        while let Some(page) = pages.try_next().await? {
            covers_all |= page.covers_all;
            for s in page.scopes {
                if scopes.len() >= max {
                    return Err(Error::too_many_results(
                        "list_access",
                        "list_access_stream",
                        max,
                    ));
                }
                scopes.push(AccessScope {
                    kind: ScopeKind::from_raw(s.scope),
                    uri: s.uri,
                });
            }
        }
        Ok(AccessScopes {
            under,
            covers_all,
            scopes,
        })
    }

    /// Streams the scopes of [`list_access`](Self::list_access), fetching
    /// pages as the stream is polled, with no cap. When the principal holds
    /// the permission on `under` itself, the stream yields exactly one
    /// [`ScopeKind::Subtree`] scope whose URI is `under` (that is how to
    /// detect "covers all"). `max_results` is ignored.
    ///
    /// ```no_run
    /// # async fn demo(client: permio::Client) -> Result<(), permio::Error> {
    /// use futures_util::TryStreamExt;
    /// let mut scopes = client.list_access_stream(permio::ListAccess::new("alice", "docs.read", "/docs"));
    /// while let Some(scope) = scopes.try_next().await? {
    ///     println!("{} {:?}", scope.uri, scope.kind);
    /// }
    /// # Ok(()) }
    /// ```
    pub fn list_access_stream(&self, query: ListAccess) -> ListStream<AccessScope> {
        match self.list_access_pages(query, None) {
            Ok(pages) => pages
                .map_ok(|page| {
                    futures_util::stream::iter(page.scopes.into_iter().map(|s| {
                        Ok(AccessScope {
                            kind: ScopeKind::from_raw(s.scope),
                            uri: s.uri,
                        })
                    }))
                })
                .try_flatten()
                .boxed(),
            Err(e) => failed(e),
        }
    }

    fn list_access_pages(
        &self,
        query: ListAccess,
        budget: Option<usize>,
    ) -> Result<ListStream<pb::ListAccessResponse>, Error> {
        require(&query.principal, "principal")?;
        require(&query.permission, "permission")?;
        validate_resource_uri(&query.under)?;
        let project_name = self.resolve_project(query.project.as_deref())?;
        let rpc = self.rpc();
        let page_size = query.page_size.unwrap_or(100);
        let ListAccess {
            principal,
            permission,
            under,
            timeout,
            ..
        } = query;
        Ok(paginate_capped(
            page_size,
            budget,
            |page: &pb::ListAccessResponse| page.scopes.len(),
            move |pagination| {
                let mut rpc = rpc.clone();
                let req = request(
                    pb::ListAccessRequest {
                        project_name: project_name.clone(),
                        principal_id: principal.clone(),
                        permission: permission.clone(),
                        under_uri: under.clone(),
                        pagination: Some(pagination),
                    },
                    timeout,
                );
                async move {
                    let mut res = rpc.list_access(req).await?.into_inner();
                    let pagination = res.pagination.take();
                    Ok((res, pagination))
                }
            },
        ))
    }

    /// Answers "who has access to this resource?" (`ListPrincipalsWithAccess`),
    /// following every page. Lists the principals (groups included, as
    /// `group:<slug>`) with a grant on the resource, an ancestor of it, or a
    /// `_` wildcard URI matching it, ordered by principal ID, with the grants
    /// that give the access.
    ///
    /// Collects at most [`WhoHasAccess::max_results`] entries (default
    /// 10,000); a resource can have millions, so use
    /// [`who_has_access_stream`](Self::who_has_access_stream) when there may
    /// be more.
    ///
    /// # Errors
    ///
    /// [`Error::TooManyResults`] if there are more than `max_results` entries
    /// (at most `max_results + 1` are fetched); [`Error::Permission`] unless
    /// the API key has `principal.list` and `principal.assignment.list` on the
    /// project.
    pub async fn who_has_access(&self, query: WhoHasAccess) -> Result<Vec<AccessEntry>, Error> {
        let max = max_results(query.max_results)?;
        let stream = self.who_has_access_pages(query, Some(max.saturating_add(1)))?;
        collect_capped(stream, max, "who_has_access", "who_has_access_stream").await
    }

    /// Streams the entries of [`who_has_access`](Self::who_has_access),
    /// fetching pages as the stream is polled, with no cap; memory stays at
    /// one page however many principals have access. `max_results` is
    /// ignored.
    ///
    /// ```no_run
    /// # async fn demo(client: permio::Client) -> Result<(), permio::Error> {
    /// use futures_util::TryStreamExt;
    /// let mut entries = client.who_has_access_stream(permio::WhoHasAccess::new("/docs/42"));
    /// while let Some(entry) = entries.try_next().await? {
    ///     println!("{}", entry.principal);
    /// }
    /// # Ok(()) }
    /// ```
    pub fn who_has_access_stream(&self, query: WhoHasAccess) -> ListStream<AccessEntry> {
        self.who_has_access_pages(query, None)
            .unwrap_or_else(failed)
    }

    fn who_has_access_pages(
        &self,
        query: WhoHasAccess,
        budget: Option<usize>,
    ) -> Result<ListStream<AccessEntry>, Error> {
        validate_resource_uri(&query.resource)?;
        if query.permission.as_deref() == Some("") {
            return Err(Error::validation("permio: permission must not be empty"));
        }
        let project_name = self.resolve_project(query.project.as_deref())?;
        let rpc = self.rpc();
        let page_size = query.page_size.unwrap_or(100);
        Ok(
            paginate_capped(page_size, budget, Vec::len, move |pagination| {
                let mut rpc = rpc.clone();
                let req = request(
                    pb::ListPrincipalsWithAccessRequest {
                        project_name: project_name.clone(),
                        resource_uri: query.resource.clone(),
                        permission: query.permission.clone(),
                        expand_groups: query.expand_groups,
                        pagination: Some(pagination),
                    },
                    query.timeout,
                );
                async move {
                    let res = rpc.list_principals_with_access(req).await?.into_inner();
                    Ok((res.principals, res.pagination))
                }
            })
            .map_ok(|page| {
                futures_util::stream::iter(page.into_iter().map(|p| Ok(AccessEntry::from(p))))
            })
            .try_flatten()
            .boxed(),
        )
    }

    /// Streams every permission in the project (`options.project`, else the
    /// client's).
    pub fn list_permissions(&self, options: ListOptions) -> ListStream<pb::Permission> {
        let project_name = match self.resolve_project(options.project.as_deref()) {
            Ok(p) => p,
            Err(e) => return failed(e),
        };
        let rpc = self.rpc();
        let limit = options.limit().unsigned_abs();
        paginate(limit, move |pagination| {
            let mut rpc = rpc.clone();
            let req = request(
                pb::ListPermissionsRequest {
                    project_name: project_name.clone(),
                    pagination: Some(pagination),
                    search: options.search.clone(),
                },
                options.timeout,
            );
            async move {
                let res = rpc.list_permissions(req).await?.into_inner();
                Ok((res.permissions, res.pagination))
            }
        })
    }

    /// Streams every role in the project.
    pub fn list_roles(&self, options: ListOptions) -> ListStream<pb::Role> {
        let project_name = match self.resolve_project(options.project.as_deref()) {
            Ok(p) => p,
            Err(e) => return failed(e),
        };
        let rpc = self.rpc();
        let limit = options.limit().unsigned_abs();
        paginate(limit, move |pagination| {
            let mut rpc = rpc.clone();
            let req = request(
                pb::ListRolesRequest {
                    project_name: project_name.clone(),
                    pagination: Some(pagination),
                    search: options.search.clone(),
                },
                options.timeout,
            );
            async move {
                let res = rpc.list_roles(req).await?.into_inner();
                Ok((res.roles, res.pagination))
            }
        })
    }

    /// Streams the IDs of every principal with grants in the project.
    pub fn list_principals(&self, options: ListOptions) -> ListStream<String> {
        let project_name = match self.resolve_project(options.project.as_deref()) {
            Ok(p) => p,
            Err(e) => return failed(e),
        };
        let rpc = self.rpc();
        let limit = options.limit().unsigned_abs();
        paginate(limit, move |pagination| {
            let mut rpc = rpc.clone();
            let req = request(
                pb::ListPrincipalsRequest {
                    project_name: project_name.clone(),
                    pagination: Some(pagination),
                    search: options.search.clone(),
                },
                options.timeout,
            );
            async move {
                let res = rpc.list_principals(req).await?.into_inner();
                Ok((res.principals, res.pagination))
            }
        })
    }

    /// Streams every assignment (resource + permissions + roles) of
    /// `principal` in the project. `options.search` is ignored.
    pub fn list_assignments(
        &self,
        principal: &str,
        options: ListOptions,
    ) -> ListStream<pb::Assignment> {
        if let Err(e) = require(principal, "principal") {
            return failed(e);
        }
        let project_name = match self.resolve_project(options.project.as_deref()) {
            Ok(p) => p,
            Err(e) => return failed(e),
        };
        let rpc = self.rpc();
        let principal = principal.to_string();
        let limit = options.limit().unsigned_abs();
        paginate(limit, move |pagination| {
            let mut rpc = rpc.clone();
            let req = request(
                pb::GetAllAssignmentsForPrincipalRequest {
                    project_name: project_name.clone(),
                    principal_id: principal.clone(),
                    pagination: Some(pagination),
                },
                options.timeout,
            );
            async move {
                let res = rpc
                    .get_all_assignments_for_principal(req)
                    .await?
                    .into_inner();
                Ok((res.assignments, res.pagination))
            }
        })
    }

    /// Grants permissions and roles to the principal on the resource, and so
    /// on every resource below it. Additive: what the principal already holds
    /// there is kept. Grant to [`group`](crate::group)`(slug)` to grant to
    /// every member of a group.
    ///
    /// Like every write, a grant is sent once and never retried automatically.
    ///
    /// # Errors
    ///
    /// [`Error::Validation`] if a permission or role does not exist in the
    /// project, or the input is malformed; [`Error::Permission`] unless the
    /// API key has full access.
    pub async fn grant(&self, grant: Grant) -> Result<(), Error> {
        let timeout = grant.timeout;
        let req = self.grant_request(grant)?;
        self.rpc()
            .grant_user_permissions_and_roles_on_resource(request(req, timeout))
            .await?;
        Ok(())
    }

    /// Removes permissions and roles from the principal's grant on exactly
    /// the resource. Names the grant does not hold are ignored, so revoking
    /// twice is harmless. Grants on ancestor URIs are not touched, so the
    /// principal may still pass a check on the resource afterwards;
    /// [`who_has_access`](Self::who_has_access) shows where access comes from.
    ///
    /// Like every write, a revoke is sent once and never retried automatically.
    ///
    /// # Errors
    ///
    /// [`Error::Permission`] unless the API key has full access.
    pub async fn revoke(&self, revoke: Grant) -> Result<(), Error> {
        let timeout = revoke.timeout;
        let req = self.grant_request(revoke)?;
        self.rpc()
            .revoke_user_permissions_and_roles_on_resource(request(
                pb::RevokeUserPermissionsAndRolesOnResourceRequest {
                    project_name: req.project_name,
                    user_id: req.user_id,
                    resource_uri: req.resource_uri,
                    permissions: req.permissions,
                    roles: req.roles,
                },
                timeout,
            ))
            .await?;
        Ok(())
    }

    fn grant_request(
        &self,
        grant: Grant,
    ) -> Result<pb::GrantUserPermissionsAndRolesOnResourceRequest, Error> {
        require(&grant.principal, "principal")?;
        validate_resource_uri(&grant.resource)?;
        let permissions = names(grant.permissions, "permission")?;
        let roles = names(grant.roles, "role")?;
        if permissions.is_empty() && roles.is_empty() {
            return Err(Error::validation(
                "permio: at least one permission or role is required",
            ));
        }
        Ok(pb::GrantUserPermissionsAndRolesOnResourceRequest {
            project_name: self.resolve_project(grant.project.as_deref())?,
            user_id: grant.principal,
            resource_uri: grant.resource,
            permissions,
            roles,
        })
    }

    /// Describes the credential this client uses: for an API key, its ID, its
    /// organisation, the project it belongs to and its access level. Useful to
    /// verify a key at startup. Needs no project.
    ///
    /// # Errors
    ///
    /// [`Error::Auth`] if the key is unknown or invalidated.
    pub async fn who_am_i(&self) -> Result<WhoAmI, Error> {
        let res = self
            .rpc()
            .who_am_i(pb::WhoAmIRequest {})
            .await?
            .into_inner();
        let kind = match pb::PrincipalType::try_from(res.principal_type) {
            Ok(pb::PrincipalType::User) => PrincipalKind::User,
            Ok(pb::PrincipalType::ApiKey) => PrincipalKind::ApiKey,
            Ok(pb::PrincipalType::ServiceAccount) => PrincipalKind::ServiceAccount,
            _ => PrincipalKind::Unknown,
        };
        let api_key_access = (kind == PrincipalKind::ApiKey).then(|| {
            match pb::ApiKeyAccess::try_from(res.api_key_access) {
                Ok(pb::ApiKeyAccess::Check) => ApiKeyAccess::Check,
                Ok(pb::ApiKeyAccess::Full) => ApiKeyAccess::Full,
                _ => ApiKeyAccess::Unknown,
            }
        });
        Ok(WhoAmI {
            principal_id: res.principal_id,
            organisation_id: res.organisation_id,
            kind,
            project: non_empty(res.project_name),
            api_key_access,
            email: non_empty(res.email),
        })
    }
}
