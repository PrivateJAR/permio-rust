//! Builds an authenticated PermissionsService client.
//!
//! Run with: PERMIO_API_KEY=... cargo run --example client --features tls
use permio_rust::permissions::v1::permissions_service_client::PermissionsServiceClient;
use permio_rust::permissions::v1::CheckPermissionRequest;
use tonic::metadata::MetadataValue;
use tonic::transport::Channel;
use tonic::Request;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("PERMIO_API_KEY").unwrap_or_default();
    let bearer: MetadataValue<_> = format!("Bearer {api_key}").parse()?;

    // connect_lazy does not touch the network until the first request.
    let channel = Channel::from_static("https://api.perms.io").connect_lazy();
    let mut client = PermissionsServiceClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut().insert("authorization", bearer.clone());
        Ok(req)
    });

    if api_key.is_empty() {
        println!("client built; set PERMIO_API_KEY to send a request");
        return Ok(());
    }

    let res = client
        .check(CheckPermissionRequest {
            project_name: "my-project".into(),
            principal_id: "user-123".into(),
            resource_uris: vec!["/documents/42".into()],
            permissions: vec!["document.read".into()],
        })
        .await?;
    println!("passed: {}", res.into_inner().passed);
    Ok(())
}
