//! Unit tests of the client against an in-process tonic server.

mod support;

use std::time::{Duration, Instant};

use futures_util::TryStreamExt;
use permio::raw::pagination::PaginationResponse;
use permio::raw::permissions::v1::*;
use permio::{BatchCheck, Check, Error, Grant, ListOptions, WhoHasAccess};
use support::{default_client, serve, start, Reply, KEY};
use tonic::{Code, Status};

fn page(cursor: &str) -> Option<PaginationResponse> {
    Some(PaginationResponse {
        next_cursor: cursor.to_string(),
    })
}

#[tokio::test]
async fn check_sends_auth_project_and_returns_the_answer() {
    let (fake, builder) = start(|method, req, _| {
        assert_eq!(method, "Check");
        let req = req.downcast_ref::<CheckPermissionRequest>().unwrap();
        Reply::ok(CheckPermissionResponse {
            passed: req.principal_id == "alice",
            missing_permissions: if req.principal_id == "alice" {
                vec![]
            } else {
                vec![MissingPermissions {
                    resource_uri: "/docs/1".into(),
                    missing_permissions: vec!["docs.read".into()],
                }]
            },
        })
    })
    .await;
    let client = builder.build().unwrap();
    assert!(client.check("alice", "docs.read", "/docs/1").await.unwrap());
    assert!(!client.check("bob", "docs.read", "/docs/1").await.unwrap());

    let calls = fake.calls();
    assert_eq!(
        calls[0].authorization.as_deref(),
        Some(format!("Bearer {KEY}").as_str())
    );
    let req = calls[0].request::<CheckPermissionRequest>();
    assert_eq!(req.project_name, "proj");
    assert_eq!(req.resource_uris, vec!["/docs/1"]);
    assert_eq!(req.permissions, vec!["docs.read"]);

    let err = client
        .assert("bob", "docs.read", "/docs/1")
        .await
        .unwrap_err();
    match err {
        Error::Denied {
            principal, missing, ..
        } => {
            assert_eq!(principal, "bob");
            assert_eq!(missing[0].permissions, vec!["docs.read"]);
        }
        other => panic!("unexpected {other:?}"),
    }
    client
        .assert("alice", "docs.read", "/docs/1")
        .await
        .unwrap();

    // Per-call project, deduplicated inputs.
    let res = client
        .check_detailed(
            Check::new("alice")
                .resources(["/a", "/a", "/b"])
                .permissions(["p", "p"])
                .project("other"),
        )
        .await
        .unwrap();
    assert!(res.allowed);
    let req = fake
        .calls()
        .last()
        .unwrap()
        .request::<CheckPermissionRequest>()
        .clone();
    assert_eq!(req.project_name, "other");
    assert_eq!(req.resource_uris, vec!["/a", "/b"]);
    assert_eq!(req.permissions, vec!["p"]);
}

#[tokio::test]
async fn check_any_uses_the_missing_list() {
    let (_fake, builder) = start(|_, _, _| {
        Reply::ok(CheckPermissionResponse {
            passed: false,
            missing_permissions: vec![MissingPermissions {
                resource_uri: "/docs/1".into(),
                missing_permissions: vec!["docs.write".into()],
            }],
        })
    })
    .await;
    let client = builder.build().unwrap();
    assert!(client
        .check_any("alice", ["docs.read", "docs.write"], "/docs/1")
        .await
        .unwrap());
    assert!(!client
        .check_any("alice", ["docs.write"], "/docs/1")
        .await
        .unwrap());
    assert!(!client
        .check_all("alice", ["docs.read", "docs.write"], "/docs/1")
        .await
        .unwrap());
}

#[tokio::test]
async fn validates_input_before_calling() {
    let (fake, client) = default_client().await;
    for err in [
        client.check("", "p", "/a").await.unwrap_err(),
        client.check("alice", "", "/a").await.unwrap_err(),
        client.check("alice", "p", "no-slash").await.unwrap_err(),
        client.check("alice", "p", "/a//b").await.unwrap_err(),
        client
            .check_detailed(Check::new("alice").resource("/a"))
            .await
            .unwrap_err(),
        client.grant(Grant::new("alice", "/a")).await.unwrap_err(),
        client
            .batch_check(BatchCheck::new("alice").permission("p"))
            .await
            .unwrap_err(),
        client
            .groups()
            .create("Not A Slug", None)
            .await
            .unwrap_err(),
        client
            .groups()
            .add_members("editors", ["group:other"])
            .await
            .unwrap_err(),
    ] {
        assert!(
            matches!(err, Error::Validation { status: None, .. }),
            "{err:?}"
        );
    }
    assert!(fake.calls().is_empty());
}

#[tokio::test]
async fn config_errors() {
    let (_fake, channel) = serve(|_, _, _| Reply::Default).await;
    let no_project = permio::Client::builder()
        .api_key(KEY)
        .channel(channel)
        .build()
        .unwrap();
    assert!(matches!(
        no_project.check("a", "p", "/a").await.unwrap_err(),
        Error::Config { .. }
    ));
    // with_project fixes it.
    no_project
        .with_project("x")
        .check("a", "p", "/a")
        .await
        .unwrap();

    let bad_key = permio::Client::builder()
        .api_key("perms_short")
        .build()
        .unwrap_err();
    assert!(matches!(bad_key, Error::Config { .. }), "{bad_key}");
    let bad_url = permio::Client::builder()
        .api_key(KEY)
        .endpoint("ftp://example.com")
        .build()
        .unwrap_err();
    assert!(matches!(bad_url, Error::Config { .. }));
    // The default endpoint is production over TLS.
    let prod = permio::Client::builder().api_key(KEY).build().unwrap();
    assert_eq!(prod.endpoint(), permio::DEFAULT_ENDPOINT);
}

#[test]
fn build_outside_a_runtime_is_a_config_error() {
    let err = permio::Client::builder().api_key(KEY).build().unwrap_err();
    assert!(matches!(err, Error::Config { .. }), "{err}");
}

#[tokio::test]
async fn maps_api_errors() {
    let (_fake, builder) = start(|_, req, _| {
        let req = req.downcast_ref::<CheckPermissionRequest>().unwrap();
        Reply::Err(match req.principal_id.as_str() {
            "auth" => Status::unauthenticated("invalid api key"),
            "perm" => Status::permission_denied(
                "missing permissions: [{ResourceURI:/organisation/o/project/p/docs MissingPermissions:[permission.check]}]",
            ),
            "invalid" => Status::invalid_argument("bad uri"),
            "notfound" => Status::not_found("no such project"),
            _ => Status::internal("boom"),
        })
    })
    .await;
    let client = builder.build().unwrap();
    let err = client.check("auth", "p", "/a").await.unwrap_err();
    assert!(matches!(err, Error::Auth { .. }));
    assert_eq!(err.code(), Some(Code::Unauthenticated));
    assert_eq!(err.message(), "invalid api key");
    match client.check("perm", "p", "/a").await.unwrap_err() {
        Error::Permission { missing, .. } => {
            assert_eq!(missing[0].resource, "/organisation/o/project/p/docs");
            assert_eq!(missing[0].permissions, vec!["permission.check"]);
        }
        other => panic!("unexpected {other:?}"),
    }
    assert!(matches!(
        client.check("invalid", "p", "/a").await.unwrap_err(),
        Error::Validation {
            status: Some(_),
            ..
        }
    ));
    assert!(matches!(
        client.check("notfound", "p", "/a").await.unwrap_err(),
        Error::NotFound { .. }
    ));
    let other = client.check("x", "p", "/a").await.unwrap_err();
    assert!(matches!(other, Error::Other { .. }));
    assert_eq!(other.code(), Some(Code::Internal));
}

#[tokio::test]
async fn retries_idempotent_reads_on_unavailable() {
    let (fake, builder) = start(|_, _, attempt| {
        if attempt < 2 {
            Reply::Err(Status::unavailable("try again"))
        } else {
            Reply::ok(CheckPermissionResponse {
                passed: true,
                missing_permissions: vec![],
            })
        }
    })
    .await;
    let client = builder.build().unwrap();
    assert!(client.check("a", "p", "/a").await.unwrap());
    assert_eq!(fake.methods(), vec!["Check"; 3]);
}

#[tokio::test]
async fn gives_up_after_max_retries() {
    let (fake, builder) = start(|_, _, _| Reply::Err(Status::unavailable("down"))).await;
    let client = builder.build().unwrap();
    let err = client.who_am_i().await.unwrap_err();
    assert!(matches!(err, Error::Unavailable { .. }), "{err:?}");
    assert_eq!(fake.methods(), vec!["WhoAmI"; 3]);

    let no_retry = start(|_, _, _| Reply::Err(Status::unavailable("down"))).await;
    let client = no_retry.1.retries(0).build().unwrap();
    client.who_am_i().await.unwrap_err();
    assert_eq!(no_retry.0.methods(), vec!["WhoAmI"]);
}

#[tokio::test]
async fn never_retries_writes_or_other_codes() {
    let (fake, builder) = start(|method, _, _| match method {
        "Check" => Reply::Err(Status::internal("boom")),
        _ => Reply::Err(Status::unavailable("down")),
    })
    .await;
    let client = builder.build().unwrap();
    let err = client
        .grant(Grant::new("alice", "/docs").permission("docs.read"))
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Unavailable { .. }));
    client
        .revoke(Grant::new("alice", "/docs").permission("docs.read"))
        .await
        .unwrap_err();
    client.groups().create("editors", None).await.unwrap_err();
    client.check("a", "p", "/a").await.unwrap_err();
    assert_eq!(
        fake.methods(),
        vec!["Grant", "Revoke", "CreateGroup", "Check"]
    );
}

#[tokio::test]
async fn per_attempt_timeout_is_retried_for_reads() {
    let (fake, builder) = start(|_, _, attempt| {
        let answer = Reply::ok(CheckPermissionResponse {
            passed: true,
            missing_permissions: vec![],
        });
        if attempt == 0 {
            Reply::Sleep(Duration::from_millis(500), Box::new(answer))
        } else {
            answer
        }
    })
    .await;
    let client = builder.timeout(Duration::from_millis(100)).build().unwrap();
    let started = Instant::now();
    assert!(client.check("a", "p", "/a").await.unwrap());
    assert!(started.elapsed() < Duration::from_millis(450));
    assert_eq!(fake.methods().len(), 2);

    let (fake, builder) =
        start(|_, _, _| Reply::Sleep(Duration::from_millis(500), Box::new(Reply::Default))).await;
    let client = builder.timeout(Duration::from_millis(50)).build().unwrap();
    let err = client
        .grant(Grant::new("alice", "/a").permission("p"))
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Timeout { .. }), "{err:?}");
    assert_eq!(err.code(), Some(Code::DeadlineExceeded));
    assert_eq!(fake.methods(), vec!["Grant"]);
}

#[tokio::test]
async fn rate_limited_reads_wait_for_the_server_delay() {
    let (fake, builder) = start(|_, _, attempt| {
        if attempt == 0 {
            Reply::Err(permio::rate_limited_status(
                "rate limited",
                Duration::from_millis(150),
            ))
        } else {
            Reply::Default
        }
    })
    .await;
    let client = builder.build().unwrap();
    let started = Instant::now();
    client.who_am_i().await.unwrap();
    assert!(started.elapsed() >= Duration::from_millis(150));
    assert_eq!(fake.methods().len(), 2);
}

#[tokio::test]
async fn rate_limit_beyond_max_delay_fails_fast() {
    let (fake, builder) = start(|_, _, _| {
        Reply::Err(permio::rate_limited_status(
            "rate limited",
            Duration::from_secs(5),
        ))
    })
    .await;
    let client = builder.build().unwrap();
    match client.who_am_i().await.unwrap_err() {
        Error::RateLimited { retry_after, .. } => {
            assert_eq!(retry_after, Some(Duration::from_secs(5)));
        }
        other => panic!("unexpected {other:?}"),
    }
    assert_eq!(fake.methods().len(), 1);
}

#[tokio::test]
async fn rate_limit_retry_after_header_and_no_delay() {
    let (fake, builder) = start(|_, _, attempt| match attempt {
        0 => {
            let mut s = Status::resource_exhausted("slow down");
            s.metadata_mut().insert("retry-after", "0".parse().unwrap());
            Reply::Err(s)
        }
        1 => Reply::Err(Status::resource_exhausted("no delay given")),
        _ => Reply::Default,
    })
    .await;
    let client = builder.build().unwrap();
    // Retried once after retry-after: 0, then the delay-less one is final.
    let err = client.who_am_i().await.unwrap_err();
    match err {
        Error::RateLimited { retry_after, .. } => assert_eq!(retry_after, None),
        other => panic!("unexpected {other:?}"),
    }
    assert_eq!(fake.methods().len(), 2);
}

#[tokio::test]
async fn transport_failures_are_unavailable() {
    let client = permio::Client::builder()
        .api_key(KEY)
        .project("p")
        .endpoint("http://127.0.0.1:1")
        .retry_config(permio::RetryConfig {
            max_retries: 1,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(5),
        })
        .build()
        .unwrap();
    let err = client.check("a", "p", "/a").await.unwrap_err();
    assert!(matches!(err, Error::Unavailable { .. }), "{err:?}");
}

#[tokio::test]
async fn paginates_until_an_empty_or_repeated_cursor() {
    let (fake, builder) = start(|method, req, _| match method {
        "ListPermissions" => {
            let req = req.downcast_ref::<ListPermissionsRequest>().unwrap();
            let cursor = req.pagination.as_ref().and_then(|p| p.cursor.clone());
            let (name, next) = match cursor.as_deref() {
                None => ("a", "c1"),
                Some("c1") => ("b", "c2"),
                _ => ("c", ""),
            };
            Reply::ok(ListPermissionsResponse {
                permissions: vec![Permission {
                    name: name.into(),
                    ..Default::default()
                }],
                pagination: page(next),
            })
        }
        "ListRoles" => Reply::ok(ListRolesResponse {
            roles: vec![Role::default()],
            pagination: page("same"),
        }),
        _ => Reply::Default,
    })
    .await;
    let client = builder.build().unwrap();
    let names: Vec<String> = client
        .list_permissions(ListOptions::default().page_size(1).search("x"))
        .map_ok(|p| p.name)
        .try_collect()
        .await
        .unwrap();
    assert_eq!(names, vec!["a", "b", "c"]);
    let reqs: Vec<ListPermissionsRequest> = fake
        .calls()
        .iter()
        .map(|c| c.request::<ListPermissionsRequest>().clone())
        .collect();
    assert_eq!(reqs[0].pagination.as_ref().unwrap().limit, Some(1));
    assert_eq!(reqs[0].search.as_deref(), Some("x"));
    assert_eq!(
        reqs[2].pagination.as_ref().unwrap().cursor.as_deref(),
        Some("c2")
    );

    // A cursor that repeats stops instead of looping forever.
    let roles: Vec<Role> = client
        .list_roles(ListOptions::default())
        .try_collect()
        .await
        .unwrap();
    assert_eq!(roles.len(), 2);

    // Empty listings and a missing project.
    let principals: Vec<String> = client
        .list_principals(ListOptions::default())
        .try_collect()
        .await
        .unwrap();
    assert!(principals.is_empty());
    let err = client
        .groups()
        .list_members("Bad Name", ListOptions::default())
        .try_collect::<Vec<_>>()
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Validation { .. }));
}

#[tokio::test]
async fn batch_check_splits_and_fails_closed() {
    let (fake, builder) = start(|_, req, _| {
        let req = req.downcast_ref::<BatchCheckRequest>().unwrap();
        Reply::ok(BatchCheckResponse {
            results: req
                .resource_uris
                .iter()
                .filter(|u| u.as_str() != "/missing")
                .map(|u| BatchCheckResult {
                    resource_uri: u.clone(),
                    allowed: u.ends_with('0'),
                    missing_permissions: if u.ends_with('0') {
                        vec![]
                    } else {
                        vec!["p".into()]
                    },
                })
                .collect(),
        })
    })
    .await;
    let client = builder.build().unwrap();
    let resources: Vec<String> = (0..250).map(|i| format!("/docs/{i}")).collect();
    let entries = client
        .batch_check(
            BatchCheck::new("alice")
                .permission("p")
                .resources(resources.clone()),
        )
        .await
        .unwrap();
    assert_eq!(entries.len(), 250);
    assert_eq!(
        entries
            .iter()
            .map(|e| e.resource.clone())
            .collect::<Vec<_>>(),
        resources
    );
    assert!(entries[10].allowed);
    assert!(!entries[11].allowed);
    assert_eq!(entries[11].missing, vec!["p"]);
    let sizes: Vec<usize> = fake
        .calls()
        .iter()
        .map(|c| c.request::<BatchCheckRequest>().resource_uris.len())
        .collect();
    assert_eq!(sizes.iter().sum::<usize>(), 250);
    assert_eq!(sizes.len(), 3);

    let err = client
        .batch_check(
            BatchCheck::new("alice")
                .permission("p")
                .resource("/missing"),
        )
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Other { status: None, .. }), "{err:?}");
}

#[tokio::test]
async fn list_access_merges_pages() {
    let (_fake, builder) = start(|_, req, _| {
        let req = req.downcast_ref::<ListAccessRequest>().unwrap();
        assert_eq!(req.under_uri, "/docs");
        let first = req.pagination.as_ref().unwrap().cursor.is_none();
        Reply::ok(ListAccessResponse {
            covers_all: false,
            scopes: vec![AccessScope {
                uri: if first {
                    "/docs/1".into()
                } else {
                    "/docs/_/x".into()
                },
                scope: if first {
                    AccessScopeKind::Subtree as i32
                } else {
                    AccessScopeKind::Pattern as i32
                },
            }],
            pagination: page(if first { "next" } else { "" }),
        })
    })
    .await;
    let client = builder.build().unwrap();
    let access = client
        .list_access("alice", "docs.read", "/docs")
        .await
        .unwrap();
    assert_eq!(access.to_prefixes(), vec!["/docs/1"]);
    assert_eq!(access.patterns(), vec!["/docs/_/x"]);
    assert!(access.allows("/docs/1/a"));
    assert!(access.allows("/docs/9/x"));
    assert!(!access.allows("/docs/2"));
}

#[tokio::test]
async fn who_has_access_maps_entries() {
    let (fake, builder) = start(|_, _, _| {
        Reply::ok(ListPrincipalsWithAccessResponse {
            principals: vec![PrincipalAccess {
                principal_id: "bob".into(),
                is_group: false,
                via_group: "editors".into(),
                via: vec![AccessGrant {
                    resource_uri: "/docs".into(),
                    inherited: true,
                    r#match: AccessGrantMatch::Ancestor as i32,
                    source: Some(access_grant::Source::Role(AccessGrantRole {
                        id: "r1".into(),
                        name: "Editor".into(),
                    })),
                }],
            }],
            pagination: None,
        })
    })
    .await;
    let client = builder.build().unwrap();
    let entries = client
        .who_has_access(
            WhoHasAccess::new("/docs/1")
                .permission("docs.read")
                .expand_groups(true),
        )
        .await
        .unwrap();
    assert_eq!(entries[0].principal, "bob");
    assert_eq!(entries[0].via_group.as_deref(), Some("editors"));
    assert_eq!(entries[0].via[0].match_kind, permio::GrantMatch::Ancestor);
    assert_eq!(
        entries[0].via[0].source,
        permio::GrantSource::Role {
            id: "r1".into(),
            name: "Editor".into()
        }
    );
    let req = fake.calls()[0]
        .request::<ListPrincipalsWithAccessRequest>()
        .clone();
    assert!(req.expand_groups);
    assert_eq!(req.permission.as_deref(), Some("docs.read"));
}

#[tokio::test]
async fn grant_revoke_and_groups_send_the_right_requests() {
    let (fake, builder) = start(|method, _, _| match method {
        "AddGroupMembers" => Reply::ok(AddGroupMembersResponse {
            added_principal_ids: vec!["bob".into()],
            existing_principal_ids: vec!["carol".into()],
        }),
        "RemoveGroupMembers" => Reply::ok(RemoveGroupMembersResponse {
            removed_principal_ids: vec!["bob".into()],
            not_member_principal_ids: vec![],
        }),
        _ => Reply::Default,
    })
    .await;
    let client = builder.build().unwrap();
    client
        .grant(
            Grant::new(permio::group("editors").unwrap(), "/docs")
                .permission("docs.read")
                .role("r1"),
        )
        .await
        .unwrap();
    client
        .revoke(Grant::new("alice", "/docs").permissions(["docs.read", "docs.read"]))
        .await
        .unwrap();
    let groups = client.groups();
    groups
        .create("group:editors", Some("Editors"))
        .await
        .unwrap();
    let added = groups
        .add_members("editors", ["bob", "carol", "bob"])
        .await
        .unwrap();
    assert_eq!(added.added, vec!["bob"]);
    assert_eq!(added.existing, vec!["carol"]);
    let removed = groups.remove_members("editors", ["bob"]).await.unwrap();
    assert_eq!(removed.removed, vec!["bob"]);
    groups.update("editors", "new").await.unwrap();
    groups.get("editors").await.unwrap();
    groups.delete("editors").await.unwrap();
    let _: Vec<Group> = groups
        .list(ListOptions::default())
        .try_collect()
        .await
        .unwrap();
    let _: Vec<Group> = groups
        .list_for_principal("bob", ListOptions::default())
        .try_collect()
        .await
        .unwrap();

    let calls = fake.calls();
    let grant = calls[0].request::<GrantUserPermissionsAndRolesOnResourceRequest>();
    assert_eq!(grant.user_id, "group:editors");
    assert_eq!(grant.permissions, vec!["docs.read"]);
    assert_eq!(grant.roles, vec!["r1"]);
    let revoke = calls[1].request::<RevokeUserPermissionsAndRolesOnResourceRequest>();
    assert_eq!(revoke.permissions, vec!["docs.read"]);
    let create = calls[2].request::<CreateGroupRequest>();
    assert_eq!(create.name, "editors");
    assert_eq!(create.description, "Editors");
    assert_eq!(create.project_name, "proj");
    let add = calls[3].request::<AddGroupMembersRequest>();
    assert_eq!(add.principal_ids, vec!["bob", "carol"]);
    assert_eq!(
        fake.methods(),
        vec![
            "Grant",
            "Revoke",
            "CreateGroup",
            "AddGroupMembers",
            "RemoveGroupMembers",
            "UpdateGroup",
            "GetGroup",
            "DeleteGroup",
            "ListGroups",
            "ListGroupsForPrincipal"
        ]
    );
}

#[tokio::test]
async fn who_am_i_maps_the_response() {
    let (_fake, builder) = start(|_, _, _| {
        Reply::ok(WhoAmIResponse {
            principal_id: "key-1".into(),
            organisation_id: "org-1".into(),
            principal_type: PrincipalType::ApiKey as i32,
            project_name: "default".into(),
            api_key_access: ApiKeyAccess::Full as i32,
            email: String::new(),
        })
    })
    .await;
    let me = builder.build().unwrap().who_am_i().await.unwrap();
    assert_eq!(me.kind, permio::PrincipalKind::ApiKey);
    assert_eq!(me.project.as_deref(), Some("default"));
    assert_eq!(me.api_key_access, Some(permio::ApiKeyAccess::Full));
    assert_eq!(me.email, None);
}

#[tokio::test]
async fn raw_clients_share_auth_and_retries() {
    let (fake, builder) = start(|_, _, attempt| {
        if attempt == 0 {
            Reply::Err(Status::unavailable("blip"))
        } else {
            Reply::Default
        }
    })
    .await;
    let client = builder.build().unwrap();
    let mut raw = client.raw_permissions();
    raw.get_role(GetRoleRequest {
        project_name: "proj".into(),
        id: "r1".into(),
    })
    .await
    .unwrap();
    let calls = fake.calls();
    assert_eq!(calls.len(), 2);
    assert!(calls
        .iter()
        .all(|c| c.authorization.as_deref() == Some(format!("Bearer {KEY}").as_str())));
    // Raw errors convert to typed errors.
    let err = raw
        .delete_role(DeleteRoleRequest::default())
        .await
        .unwrap_err();
    assert!(matches!(Error::from(err), Error::Unavailable { .. }));
}
