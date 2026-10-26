//! The generated Protocol Buffer messages and tonic clients/servers.
//!
//! This is the escape hatch for anything the [`Client`](crate::Client)
//! helpers do not wrap (permission and role CRUD, organisation and project
//! management, ...). Use [`Client::raw_permissions`](crate::Client::raw_permissions)
//! and [`Client::raw_organisations`](crate::Client::raw_organisations) to get
//! raw clients that share the SDK's channel, authentication, timeouts and
//! retries.
//!
//! The files under `src/permio` and `src/pagination` are generated from the
//! `PrivateJAR/proto` repository and must not be edited by hand. The module
//! tree mirrors the proto package names because the generated code refers to
//! other packages with relative paths (`super::super::super::pagination`).
#![allow(missing_docs, clippy::all, clippy::pedantic, rustdoc::all)]

/// `pagination` proto package: cursor pagination request/response messages.
pub mod pagination {
    include!("pagination/pagination.rs");
}

/// The `permio.*` proto packages.
pub mod permio {
    /// `permio.organisation.*` proto packages.
    pub mod organisation {
        /// `permio.organisation.v1`: organisations, projects and API keys.
        pub mod v1 {
            include!("permio/organisation/v1/permio.organisation.v1.rs");
        }
    }
    /// `permio.permissions.*` proto packages.
    pub mod permissions {
        /// `permio.permissions.v1`: checks, grants, permissions, roles and groups.
        pub mod v1 {
            include!("permio/permissions/v1/permio.permissions.v1.rs");
        }
    }
}

pub use self::permio::{organisation, permissions};
