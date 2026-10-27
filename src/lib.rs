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
//!   [`who_has_access`](Client::who_has_access), capped at 10,000 results,
//!   and their uncapped streaming forms
//!   [`list_access_stream`](Client::list_access_stream) /
//!   [`who_has_access_stream`](Client::who_has_access_stream)),
//!   [`grant`](Client::grant) /
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
//! An overall deadline across retries can be set per client
//! ([`ClientBuilder::call_timeout`]) and per call (the `timeout` option of
//! each request type, [`ListOptions::timeout`], [`Groups::timeout`]); every
//! call also takes a `project` the same way.
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
    ListAccess, PrincipalKind, WhoAmI, WhoHasAccess, API_KEY_ENV, DEFAULT_ENDPOINT,
    DEFAULT_MAX_BATCH_RESOURCES, DEFAULT_MAX_RESULTS, DEFAULT_TIMEOUT, DEV_ENDPOINT,
};
pub use error::{parse_missing_permissions, server_retry_delay, Error, MissingPermission};
pub use groups::{group, AddMembersResult, Groups, RemoveMembersResult, GROUP_PRINCIPAL_PREFIX};
pub use list::{paginate, ListOptions, ListStream, MAX_PAGE_SIZE};
pub use resource::{encode_segment, resource, validate_resource_uri, MAX_RESOURCE_URI_LENGTH};
pub use transport::{is_idempotent_method, CallTimeout, RetryConfig, Transport};

/// Generated message types returned by the helpers.
pub use raw::permissions::v1::{Assignment, Group, GroupMember, Permission, Role};

// Re-exported so the generated types and raw clients can be used without
// adding these crates as direct dependencies at matching versions. A
// semver-incompatible upgrade of any of them is a breaking release of permio.

/// The `tonic` version this crate is built on (raw clients, `Status`, `Channel`).
pub use tonic;

/// The `prost` version the generated messages implement `prost::Message` from.
pub use prost;

/// The `prost-types` version used by generated messages (e.g. `Timestamp`).
pub use prost_types;
