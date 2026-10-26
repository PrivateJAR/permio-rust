//! Checks a permission, then lists the project's roles through the raw client.
//!
//! Run with: PERMIO_API_KEY=perms_... cargo run --example client -- my-project
//! (add PERMIO_ENDPOINT=https://dev-api.perms.io for the development API).
use futures_util::TryStreamExt;
use permio::raw::permissions::v1::ListRolesRequest;

#[tokio::main]
async fn main() -> Result<(), permio::Error> {
    let project = std::env::args().nth(1).unwrap_or_else(|| "default".into());
    let mut builder = permio::Client::builder().project(project.clone());
    if let Ok(endpoint) = std::env::var("PERMIO_ENDPOINT") {
        builder = builder.endpoint(endpoint);
    }
    if std::env::var(permio::API_KEY_ENV).is_err() {
        println!("set PERMIO_API_KEY to run this example");
        return Ok(());
    }
    let client = builder.build()?;

    let me = client.who_am_i().await?;
    println!("calling as {} ({:?})", me.principal_id, me.kind);

    let doc = permio::resource!("documents", 42)?;
    let allowed = client.check("user-123", "document.read", &doc).await?;
    println!("user-123 may read {doc}: {allowed}");

    // The helpers stream every page:
    let roles: Vec<_> = client.list_roles(Default::default()).try_collect().await?;
    println!("{} roles", roles.len());

    // The raw client covers every RPC, with the same auth, timeouts and retries:
    let first_page = client
        .raw_permissions()
        .list_roles(ListRolesRequest {
            project_name: project,
            ..Default::default()
        })
        .await?
        .into_inner();
    println!("first page: {} roles", first_page.roles.len());
    Ok(())
}
