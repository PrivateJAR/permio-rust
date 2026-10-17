//! Generated Protocol Buffer and gRPC client code for the perms.io API.
//!
//! Generated from the PrivateJAR/proto repository. Do not edit by hand.
#![allow(clippy::all)]

pub mod pagination {
    include!("pagination/pagination.rs");
}
pub mod permio {
    pub mod organisation {
        pub mod v1 {
            include!("permio/organisation/v1/permio.organisation.v1.rs");
        }
    }
    pub mod permissions {
        pub mod v1 {
            include!("permio/permissions/v1/permio.permissions.v1.rs");
        }
    }
}

/// Convenience re-exports: `permio_rust::permissions::v1`, `permio_rust::organisation::v1`.
pub use permio::{organisation, permissions};
