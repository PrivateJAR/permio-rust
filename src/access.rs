//! Results of [`Client::list_access`](crate::Client::list_access) and
//! [`Client::who_has_access`](crate::Client::who_has_access).

use crate::raw::permissions::v1 as pb;

/// The `_` segment: in a [`ScopeKind::Pattern`] scope it matches exactly one segment.
pub const WILDCARD_SEGMENT: &str = "_";

/// Which resources a scope's `uri` stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ScopeKind {
    /// The URI itself and everything below it.
    Subtree,
    /// Every URI whose leading segments match `uri`, where each `_` segment
    /// matches exactly one segment, and everything below those.
    Pattern,
    /// Exactly the URI. Reserved by the API; not returned today.
    Exact,
    /// A kind this SDK version does not know. It never matches.
    Unknown,
}

impl ScopeKind {
    pub(crate) fn from_raw(value: i32) -> Self {
        match pb::AccessScopeKind::try_from(value) {
            Ok(pb::AccessScopeKind::Subtree) => ScopeKind::Subtree,
            Ok(pb::AccessScopeKind::Pattern) => ScopeKind::Pattern,
            Ok(pb::AccessScopeKind::Exact) => ScopeKind::Exact,
            _ => ScopeKind::Unknown,
        }
    }
}

/// One place where a principal holds a permission.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct AccessScope {
    /// Resource URI (subtree, exact) or URI pattern with `_` segments (pattern).
    pub uri: String,
    /// How `uri` is matched.
    pub kind: ScopeKind,
}

impl AccessScope {
    /// A scope of `kind` at `uri`.
    #[must_use]
    pub fn new(uri: impl Into<String>, kind: ScopeKind) -> Self {
        AccessScope {
            uri: uri.into(),
            kind,
        }
    }
}

// Splits "/a/b" into ["a", "b"]. Returns None for anything that is not a
// well-formed URI (no leading slash, empty segments), so it can never match.
fn segments(uri: &str) -> Option<Vec<&str>> {
    let rest = uri.strip_prefix('/')?;
    if rest.is_empty() {
        return None;
    }
    let parts: Vec<&str> = rest.split('/').collect();
    if parts.iter().any(|p| p.is_empty()) {
        None
    } else {
        Some(parts)
    }
}

fn scope_matches(scope: &AccessScope, target: &[&str]) -> bool {
    let Some(own) = segments(&scope.uri) else {
        return false;
    };
    match scope.kind {
        ScopeKind::Exact => own == target,
        // Segment by segment, so /a/b covers /a/b/c but not /a/bc. A "_" here is literal.
        ScopeKind::Subtree => {
            own.len() <= target.len() && own.iter().zip(target).all(|(s, t)| s == t)
        }
        ScopeKind::Pattern => {
            own.len() <= target.len()
                && own
                    .iter()
                    .zip(target)
                    .all(|(s, t)| *s == WILDCARD_SEGMENT || s == t)
        }
        ScopeKind::Unknown => false,
    }
}

fn escape_regex(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if ".*+?^${}()|[]\\/".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

fn scope_regex_source(scope: &AccessScope) -> Option<String> {
    let own = segments(&scope.uri)?;
    let body: String = own
        .iter()
        .map(|s| {
            let part = if scope.kind == ScopeKind::Pattern && *s == WILDCARD_SEGMENT {
                "[^/]+".to_string()
            } else {
                escape_regex(s)
            };
            format!("\\/{part}")
        })
        .collect();
    match scope.kind {
        ScopeKind::Exact => Some(body),
        ScopeKind::Subtree | ScopeKind::Pattern => Some(format!("{body}(?:\\/.*)?")),
        ScopeKind::Unknown => None,
    }
}

/// The answer to "where may this principal use this permission under
/// `under`?", returned by [`Client::list_access`](crate::Client::list_access).
/// Every page has already been fetched.
///
/// The scopes describe resources at or below `under`. A URI outside `under`
/// was not asked about, so [`allows`](Self::allows) returning `false` for it
/// means "unknown", not "denied". A pattern scope may also match some URIs
/// outside `under` (its `_` segments can sit in the part that matches
/// `under`); those really are allowed.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct AccessScopes {
    /// The `under` URI that was asked about.
    pub under: String,
    /// True when the principal holds the permission on `under` itself, and so
    /// on everything below it. `scopes` is then a single subtree scope for
    /// `under`. Check this first: when it is true, a list needs no filter.
    pub covers_all: bool,
    /// Every scope, ordered by URI as the API returned them.
    pub scopes: Vec<AccessScope>,
}

impl AccessScopes {
    /// Builds an answer by hand, e.g. to stub
    /// [`Client::list_access`](crate::Client::list_access) in your tests.
    #[must_use]
    pub fn new(under: impl Into<String>, covers_all: bool, scopes: Vec<AccessScope>) -> Self {
        AccessScopes {
            under: under.into(),
            covers_all,
            scopes,
        }
    }

    /// True when the principal has no access anywhere under `under`.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        !self.covers_all && !self.scopes.iter().any(|s| s.kind != ScopeKind::Unknown)
    }

    /// Whether the principal holds the permission on `uri`, according to these
    /// scopes. Matching is per segment: a subtree scope `/a/b` allows `/a/b`
    /// and `/a/b/c` but not `/a/bc`; a pattern scope `/a/_/c` allows `/a/x/c`
    /// and `/a/x/c/d` but not `/a/x` or `/a/x/y`. Malformed URIs are never
    /// allowed.
    #[must_use]
    pub fn allows(&self, uri: &str) -> bool {
        let Some(target) = segments(uri) else {
            return false;
        };
        self.scopes.iter().any(|s| scope_matches(s, &target))
    }

    /// The subtree scopes' URIs, for filtering a database by path prefix. A
    /// URI `p` here stands for `p` itself and everything below it, so match
    /// `path = p OR path LIKE p || '/%'`; a bare prefix match would also match
    /// `/docs/12` for `/docs/1`. Prefixes already covered by a shorter one are
    /// dropped.
    ///
    /// Pattern scopes cannot be expressed as a prefix and are not included:
    /// see [`patterns`](Self::patterns), [`regex_pattern`](Self::regex_pattern)
    /// and [`has_patterns`](Self::has_patterns).
    #[must_use]
    pub fn to_prefixes(&self) -> Vec<String> {
        let mut subtrees: Vec<&str> = Vec::new();
        for s in &self.scopes {
            if s.kind == ScopeKind::Subtree && !subtrees.contains(&s.uri.as_str()) {
                subtrees.push(&s.uri);
            }
        }
        subtrees
            .iter()
            .filter(|p| !subtrees.iter().any(|q| p.starts_with(&format!("{q}/"))))
            .map(|p| (*p).to_string())
            .collect()
    }

    /// The pattern scopes' URIs (with `_` segments).
    #[must_use]
    pub fn patterns(&self) -> Vec<String> {
        self.of_kind(ScopeKind::Pattern)
    }

    /// The exact scopes' URIs. The API does not return exact scopes today.
    #[must_use]
    pub fn exact(&self) -> Vec<String> {
        self.of_kind(ScopeKind::Exact)
    }

    fn of_kind(&self, kind: ScopeKind) -> Vec<String> {
        self.scopes
            .iter()
            .filter(|s| s.kind == kind)
            .map(|s| s.uri.clone())
            .collect()
    }

    /// True when some scope is a pattern, so [`to_prefixes`](Self::to_prefixes)
    /// alone does not cover everything.
    #[must_use]
    pub fn has_patterns(&self) -> bool {
        self.scopes.iter().any(|s| s.kind == ScopeKind::Pattern)
    }

    /// One anchored regular expression that matches exactly the URIs
    /// [`allows`](Self::allows) accepts, for stores that filter by regex
    /// (PostgreSQL `~`, MongoDB `$regex`, the `regex` crate). With no scopes it
    /// is `$.^`, which matches nothing.
    #[must_use]
    pub fn regex_pattern(&self) -> String {
        let parts: Vec<String> = self.scopes.iter().filter_map(scope_regex_source).collect();
        if parts.is_empty() {
            "$.^".to_string()
        } else {
            format!("^(?:{})$", parts.join("|"))
        }
    }
}

/// How a grant reaches the resource asked about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum GrantMatch {
    /// The grant is on exactly the resource.
    Exact,
    /// The grant is on an ancestor of the resource.
    Ancestor,
    /// The grant is on a URI with `_` segments that matches the resource.
    Wildcard,
    /// A kind this SDK version does not know.
    Unknown,
}

/// Where a grant's access comes from.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum GrantSource {
    /// Through a role.
    Role {
        /// The role ID.
        id: String,
        /// The role name.
        name: String,
    },
    /// Directly granted permissions.
    Direct,
    /// A source this SDK version does not know.
    Unknown,
}

/// One grant (or one role within a grant) that gives a principal access.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct AccessGrant {
    /// The URI the grant was made on: the resource, an ancestor, or a URI with
    /// `_` segments matching it.
    pub resource: String,
    /// True unless the grant was made on exactly the resource.
    pub inherited: bool,
    /// How the grant reaches the resource.
    pub match_kind: GrantMatch,
    /// Through a role, or as directly granted permissions.
    pub source: GrantSource,
}

/// One principal with access to a resource, returned by
/// [`Client::who_has_access`](crate::Client::who_has_access).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct AccessEntry {
    /// The principal ID; `group:<slug>` for a group.
    pub principal: String,
    /// True when `principal` is a group.
    pub is_group: bool,
    /// The slug of the group through which `principal` has the grants in
    /// `via`, for entries added by `expand_groups`. `None` for grants made to
    /// `principal` itself.
    pub via_group: Option<String>,
    /// The grants that give the access, most specific first.
    pub via: Vec<AccessGrant>,
}

impl From<pb::PrincipalAccess> for AccessEntry {
    fn from(p: pb::PrincipalAccess) -> Self {
        AccessEntry {
            principal: p.principal_id,
            is_group: p.is_group,
            via_group: if p.via_group.is_empty() {
                None
            } else {
                Some(p.via_group)
            },
            via: p
                .via
                .into_iter()
                .map(|g| AccessGrant {
                    match_kind: match pb::AccessGrantMatch::try_from(g.r#match) {
                        Ok(pb::AccessGrantMatch::Exact) => GrantMatch::Exact,
                        Ok(pb::AccessGrantMatch::Ancestor) => GrantMatch::Ancestor,
                        Ok(pb::AccessGrantMatch::Wildcard) => GrantMatch::Wildcard,
                        _ => GrantMatch::Unknown,
                    },
                    source: match g.source {
                        Some(pb::access_grant::Source::Role(r)) => GrantSource::Role {
                            id: r.id,
                            name: r.name,
                        },
                        Some(pb::access_grant::Source::Direct(_)) => GrantSource::Direct,
                        None => GrantSource::Unknown,
                    },
                    resource: g.resource_uri,
                    inherited: g.inherited,
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scopes(list: &[(&str, ScopeKind)]) -> AccessScopes {
        AccessScopes {
            under: "/docs".into(),
            covers_all: false,
            scopes: list
                .iter()
                .map(|(u, k)| AccessScope {
                    uri: (*u).into(),
                    kind: *k,
                })
                .collect(),
        }
    }

    #[test]
    fn subtree_and_pattern_matching() {
        let s = scopes(&[
            ("/docs/a", ScopeKind::Subtree),
            ("/docs/a/b", ScopeKind::Subtree),
            ("/docs/_/c", ScopeKind::Pattern),
        ]);
        assert!(s.allows("/docs/a"));
        assert!(s.allows("/docs/a/x"));
        assert!(!s.allows("/docs/ab"));
        assert!(s.allows("/docs/x/c"));
        assert!(s.allows("/docs/x/c/d"));
        assert!(!s.allows("/docs/x"));
        assert!(!s.allows("/docs/x/y"));
        assert!(!s.allows("docs/a"));
        assert!(!s.allows("/docs//a"));
        assert_eq!(s.to_prefixes(), vec!["/docs/a".to_string()]);
        assert_eq!(s.patterns(), vec!["/docs/_/c".to_string()]);
        assert!(s.has_patterns());
        assert!(!s.is_empty());
        assert_eq!(
            s.regex_pattern(),
            "^(?:\\/docs\\/a(?:\\/.*)?|\\/docs\\/a\\/b(?:\\/.*)?|\\/docs\\/[^/]+\\/c(?:\\/.*)?)$"
        );
    }

    #[test]
    fn empty_and_unknown() {
        let s = scopes(&[("/docs/a", ScopeKind::Unknown)]);
        assert!(s.is_empty());
        assert!(!s.allows("/docs/a"));
        assert_eq!(s.regex_pattern(), "$.^");
        let exact = scopes(&[("/docs/a", ScopeKind::Exact)]);
        assert!(exact.allows("/docs/a"));
        assert!(!exact.allows("/docs/a/b"));
    }
}
