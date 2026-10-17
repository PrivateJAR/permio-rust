# Permio Rust

Generated Protocol Buffer and gRPC client code for the [perms.io](https://perms.io) API
(prost 0.14 / tonic 0.14).

This repository is generated from `PrivateJAR/proto` on every release. Do not edit it by hand.

## Installation

```toml
[dependencies]
permio-rust = { git = "https://github.com/PrivateJAR/permio-rust", tag = "v0.0.16", features = ["tls"] }
```

## Usage

```rust
use permio_rust::permissions::v1::permissions_service_client::PermissionsServiceClient;
use permio_rust::organisation::v1::organisation_service_client::OrganisationServiceClient;
```

See [`examples/client.rs`](examples/client.rs) for an authenticated client
(`authorization: Bearer <API key>`).

## Version

v0.0.16
