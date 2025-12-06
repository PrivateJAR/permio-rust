# Permio Rust

Generated Protocol Buffer and gRPC code for Permio.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
permio-rust = { git = "https://github.com/PrivateJAR/permio-rust", tag = "v0.0.12" }
```

## Usage

```rust
use permio_rust::permissions::v1::service::permissions_service_client::PermissionsServiceClient;
use permio_rust::organisation::v1::service::organisation_service_client::OrganisationServiceClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = PermissionsServiceClient::connect("http://[::1]:50051").await?;
    // Use the client...
    Ok(())
}
```

## Version

v0.0.12
