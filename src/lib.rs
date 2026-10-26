//! Rust SDK for [perms.io](https://perms.io), the permissions (RBAC) service.
//!
//! ```no_run
//! # #[tokio::main] async fn main() -> Result<(), permio::Error> {
//! let client = permio::Client::builder().project("my-app").build()?; // key from PERMIO_API_KEY
//! let doc = permio::resource!("docs", 42)?;
//! if client.check("user-1", "docs.read", &doc).await? {
//!     println!("allowed");
//! }
//! # Ok(()) }
//! ```
//!
//! - [`Client`]: checks ([`check`](Client::check), [`check_all`](Client::check_all),
//!   [`check_any`](Client::check_any), [`check_detailed`](Client::check_detailed),
//!   [`assert`](Client::assert), [`batch_check`](Client::batch_check)), access
//!   queries ([`list_access`](Client::list_access),
//!   [`who_has_access`](Client::who_has_access)), [`grant`](Client::grant) /
//!   [`revoke`](Client::revoke), paginated listings as [`ListStream`]s,
//!   [`groups`](Client::groups) and [`who_am_i`](Client::who_am_i).
//! - [`Error`]: one enum with a variant per failure kind.
//! - [`raw`]: the generated prost/tonic code, for everything the helpers do
//!   not wrap. [`Client::raw_permissions`] and [`Client::raw_organisations`]
//!   give raw clients that share the SDK's auth, timeouts and retries.
//!
//! Calls time out after 10 s per attempt. Idempotent reads are retried twice
//! with jittered exponential backoff on `Unavailable` / `DeadlineExceeded`
//! (and after the server's delay when rate limited); writes are never retried.
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

mod access;
mod apikey;
mod client;
mod error;
mod groups;
mod list;
pub mod raw;
mod resource;
mod transport;

pub use access::{
    AccessEntry, AccessGrant, AccessScope, AccessScopes, GrantMatch, GrantSource, ScopeKind,
    WILDCARD_SEGMENT,
};
pub use apikey::{is_valid_api_key, validate_api_key, ApiKeyKind, API_KEY_LENGTH};
pub use client::{
    ApiKeyAccess, BatchCheck, BatchCheckEntry, Check, CheckResult, Client, ClientBuilder, Grant,
    PrincipalKind, WhoAmI, WhoHasAccess, API_KEY_ENV, DEFAULT_ENDPOINT, DEFAULT_TIMEOUT,
    DEV_ENDPOINT,
};
pub use error::{
    parse_missing_permissions, rate_limited_status, server_retry_delay, Error, MissingPermission,
};
pub use groups::{group, AddMembersResult, Groups, RemoveMembersResult, GROUP_PRINCIPAL_PREFIX};
pub use list::{paginate, ListOptions, ListStream, MAX_PAGE_SIZE};
pub use resource::{encode_segment, resource, validate_resource_uri, MAX_RESOURCE_URI_LENGTH};
pub use transport::{is_idempotent_method, RetryConfig, Transport};

/// Generated message types returned by the helpers.
pub use raw::permissions::v1::{Assignment, Group, GroupMember, Permission, Role};

/// Re-exported so the types in this crate's API can be named without adding
/// `tonic` as a direct dependency.
pub use tonic;
