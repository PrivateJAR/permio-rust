//! Integration test against a real deployment. Skipped unless PERMIO_API_KEY is
//! set. PERMIO_BASE_URL defaults to the development API (DEV_ENDPOINT), and the
//! test refuses to run against production because it writes. The key needs
//! full access to its project (PERMIO_PROJECT, default: the key's project from
//! whoAmI):
//!
//!   PERMIO_API_KEY=perms_... cargo test --test live -- --nocapture
//!
//! Everything it creates uses fixed IDs prefixed "sdk-it-rust" and is removed
//! before and after the run, so an interrupted run is cleaned up by the next
//! one. Runs sharing a project must not overlap (CI serialises them).

use std::panic::AssertUnwindSafe;

use futures_util::FutureExt;
use permio::raw::permissions::v1::{CreatePermissionRequest, DeletePermissionRequest};
use permio::{group, BatchCheck, Client, Error, Grant, DEV_ENDPOINT};

const ALICE: &str = "sdk-it-rust-alice";
const BOB: &str = "sdk-it-rust-bob";
const GROUP: &str = "sdk-it-rust-editors";
const READ: &str = "sdk-it-rust.read";
const WRITE: &str = "sdk-it-rust.write";
const ROOT: &str = "/sdk-it-rust";
const DOCS: &str = "/sdk-it-rust/docs";
const DOC1: &str = "/sdk-it-rust/docs/1";
const OTHER: &str = "/sdk-it-rust/other";

fn is_prod(url: &str) -> bool {
    let host = url
        .trim_end_matches('/')
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let host = host.split(':').next().unwrap_or(host);
    host == "api.perms.io" || host == "temp-api.perms.io"
}

// Deleting the permissions also removes every grant that uses them.
async fn clean_up(client: &Client) {
    client.groups().delete(GROUP).await.expect("delete group");
    for name in [READ, WRITE] {
        let res = client
            .raw_permissions()
            .delete_permission(DeletePermissionRequest {
                project_name: client.project().unwrap().to_string(),
                name: name.to_string(),
            })
            .await;
        match res.map_err(Error::from) {
            Ok(_) | Err(Error::NotFound { .. }) => {}
            Err(e) => panic!("delete permission {name}: {e}"),
        }
    }
}

#[tokio::test]
async fn live_integration() {
    let Ok(api_key) = std::env::var("PERMIO_API_KEY") else {
        eprintln!("PERMIO_API_KEY is not set; skipping the live integration test");
        return;
    };
    let base_url = std::env::var("PERMIO_BASE_URL")
        .ok()
        .filter(|u| !u.is_empty())
        .unwrap_or_else(|| DEV_ENDPOINT.to_string());
    if is_prod(&base_url) {
        eprintln!("refusing to run the live integration test against production ({base_url})");
        return;
    }

    let probe = Client::builder()
        .api_key(api_key.clone())
        .endpoint(base_url.clone())
        .build()
        .unwrap();
    let project = match std::env::var("PERMIO_PROJECT")
        .ok()
        .filter(|p| !p.is_empty())
    {
        Some(p) => p,
        None => probe
            .who_am_i()
            .await
            .expect("whoAmI")
            .project
            .unwrap_or_else(|| "default".into()),
    };
    let client = probe.with_project(project.clone());

    clean_up(&client).await;
    for name in [READ, WRITE] {
        client
            .raw_permissions()
            .create_permission(CreatePermissionRequest {
                project_name: project.clone(),
                name: name.into(),
                description: "permio-rust integration test".into(),
            })
            .await
            .expect("create permission");
    }

    let outcome = AssertUnwindSafe(scenarios(&client, &base_url, &project))
        .catch_unwind()
        .await;
    clean_up(&client).await;
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
    // The cleanup really removed the group.
    assert!(matches!(
        client.groups().get(GROUP).await.unwrap_err(),
        Error::NotFound { .. }
    ));
}

async fn scenarios(client: &Client, base_url: &str, project: &str) {
    // whoAmI describes the API key.
    let me = client.who_am_i().await.unwrap();
    assert_eq!(me.kind, permio::PrincipalKind::ApiKey);
    assert_eq!(me.project.as_deref(), Some(project));
    assert!(!me.principal_id.is_empty());
    assert!(!me.organisation_id.is_empty());
    assert_eq!(me.api_key_access, Some(permio::ApiKeyAccess::Full));
    eprintln!("whoAmI ok");

    // grant, then check is allowed below the granted URI.
    assert!(!client.check(ALICE, READ, DOC1).await.unwrap());
    client
        .grant(Grant::new(ALICE, DOCS).permission(READ))
        .await
        .unwrap();
    assert!(client.check(ALICE, READ, DOC1).await.unwrap());
    assert!(!client.check(ALICE, WRITE, DOC1).await.unwrap());
    client.assert(ALICE, READ, DOC1).await.unwrap();
    assert!(matches!(
        client.assert(ALICE, WRITE, DOC1).await.unwrap_err(),
        Error::Denied { .. }
    ));
    assert!(client.check_any(ALICE, [READ, WRITE], DOC1).await.unwrap());
    assert!(!client.check_all(ALICE, [READ, WRITE], DOC1).await.unwrap());
    eprintln!("grant + check ok");

    // batchCheck answers each resource.
    let result = client
        .batch_check(
            BatchCheck::new(ALICE)
                .permission(READ)
                .resources([DOC1, OTHER, DOCS]),
        )
        .await
        .unwrap();
    let pairs: Vec<(&str, bool)> = result
        .iter()
        .map(|e| (e.resource.as_str(), e.allowed))
        .collect();
    assert_eq!(pairs, vec![(DOC1, true), (OTHER, false), (DOCS, true)]);
    eprintln!("batchCheck ok");

    // listAccess returns the granted subtree.
    let access = client.list_access(ALICE, READ, ROOT).await.unwrap();
    assert!(!access.covers_all);
    assert_eq!(access.to_prefixes(), vec![DOCS.to_string()]);
    assert!(access.allows(DOC1));
    assert!(!access.allows(OTHER));
    eprintln!("listAccess ok");

    // whoHasAccess lists alice through her grant.
    let entries = client
        .who_has_access(permio::WhoHasAccess::new(DOC1).permission(READ))
        .await
        .unwrap();
    assert!(entries.iter().any(|e| e.principal == ALICE));
    eprintln!("whoHasAccess ok");

    // revoke, then check is denied.
    client
        .revoke(Grant::new(ALICE, DOCS).permission(READ))
        .await
        .unwrap();
    assert!(!client.check(ALICE, READ, DOC1).await.unwrap());
    // Revoking what is no longer held is harmless.
    client
        .revoke(Grant::new(ALICE, DOCS).permission(READ))
        .await
        .unwrap();
    eprintln!("revoke ok");

    // A group grant applies to its members until the group is deleted.
    let groups = client.groups();
    groups
        .create(GROUP, Some("permio-rust integration test"))
        .await
        .unwrap();
    let added = groups.add_members(GROUP, [BOB]).await.unwrap();
    assert_eq!(added.added, vec![BOB.to_string()]);
    assert!(added.existing.is_empty());
    client
        .grant(Grant::new(group(GROUP).unwrap(), DOCS).permission(WRITE))
        .await
        .unwrap();
    assert!(client.check(BOB, WRITE, DOC1).await.unwrap());
    assert!(!client.check(ALICE, WRITE, DOC1).await.unwrap());
    groups.delete(GROUP).await.unwrap();
    assert!(!client.check(BOB, WRITE, DOC1).await.unwrap());
    eprintln!("groups ok");

    // An unknown key is rejected with Error::Auth.
    let bad = Client::builder()
        .api_key("perms_0123456789ABCDEFGHIJKLMNOPQRSTUV1ggZdL")
        .endpoint(base_url)
        .project(project)
        .build()
        .unwrap();
    assert!(matches!(
        bad.check(ALICE, READ, DOC1).await.unwrap_err(),
        Error::Auth { .. }
    ));
    assert!(matches!(
        bad.who_am_i().await.unwrap_err(),
        Error::Auth { .. }
    ));
    eprintln!("bad key ok");
}
