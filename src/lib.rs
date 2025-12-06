//! Permio Rust gRPC Client
//!
//! Generated Protocol Buffer and gRPC code for Permio.

pub mod pagination {
    include!("pagination.rs");
}

pub mod organisation {
    pub mod v1 {
        include!("permio.organisation.v1.rs");
        pub mod service {
            include!("permio.organisation.v1.tonic.rs");
        }
    }
}

pub mod permissions {
    pub mod v1 {
        include!("permio.permissions.v1.rs");
        pub mod service {
            include!("permio.permissions.v1.tonic.rs");
        }
    }
}
