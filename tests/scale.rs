//! Scale guardrails: streaming forms, collect-all caps, the batch_check bound
//! and list timeouts, against an in-process fake with large result sets.

mod support;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use futures_util::TryStreamExt;
use permio::raw::pagination::PaginationResponse;
use permio::raw::permissions::v1::*;
use permio::{
    BatchCheck, Error, ListAccess, ScopeKind, WhoHasAccess, DEFAULT_MAX_BATCH_RESOURCES,
    DEFAULT_MAX_RESULTS,
};
use support::{rate_limited, start, Reply};
use tonic::{Code, Status};

/// A cursor-paged listing of `total` items: the cursor is the offset, and each
/// page holds `pagination.limit` items.
fn window(
    req_pagination: Option<&permio::raw::pagination::PaginationRequest>,
    total: usize,
) -> (usize, usize, Option<PaginationResponse>) {
    let p = req_pagination.expect("pagination");
    let start: usize = p.cursor.as_deref().map_or(0, |c| c.parse().unwrap());
    let limit = usize::try_from(p.limit.unwrap()).unwrap();
    assert!((1..=100).contains(&limit), "limit {limit}");
    let end = (start + limit).min(total);
    let next = if end < total {
        end.to_string()
    } else {
        String::new()
    };
    (start, end, Some(PaginationResponse { next_cursor: next }))
}

fn principals_server(total: usize) -> impl Fn(&str, &dyn std::any::Any, usize) -> Reply {
    move |_, req, _| {
        let req = req
            .downcast_ref::<ListPrincipalsWithAccessRequest>()
            .unwrap();
        let (start, end, pagination) = window(req.pagination.as_ref(), total);
        Reply::ok(ListPrincipalsWithAccessResponse {
            principals: (start..end)
                .map(|i| PrincipalAccess {
                    principal_id: format!("user-{i:07}"),
                    ..Default::default()
                })
                .collect(),
            pagination,
        })
    }
}

fn scopes_server(total: usize) -> impl Fn(&str, &dyn std::any::Any, usize) -> Reply {
    move |_, req, _| {
        let req = req.downcast_ref::<ListAccessRequest>().unwrap();
        let (start, end, pagination) = window(req.pagination.as_ref(), total);
        Reply::ok(ListAccessResponse {
            covers_all: false,
            scopes: (start..end)
                .map(|i| AccessScope {
                    uri: format!("/docs/{i}"),
                    scope: AccessScopeKind::Subtree as i32,
                })
                .collect(),
            pagination,
        })
    }
}

fn requested(fake: &support::Fake) -> Vec<i32> {
    fake.calls()
        .iter()
        .map(|c| {
            if let Some(r) = c.request.downcast_ref::<ListPrincipalsWithAccessRequest>() {
                r.pagination.as_ref().unwrap().limit.unwrap()
            } else {
                c.request::<ListAccessRequest>()
                    .pagination
                    .as_ref()
                    .unwrap()
                    .limit
                    .unwrap()
            }
        })
        .collect()
}

#[tokio::test]
async fn who_has_access_stream_follows_many_pages_lazily() {
    let (fake, builder) = start(principals_server(25_000)).await;
    let client = builder.build().unwrap();
    let mut stream = client.who_has_access_stream(WhoHasAccess::new("/docs/1"));
    // Nothing is fetched until polled, then one page at a time.
    assert_eq!(fake.calls().len(), 0);
    let first = stream.try_next().await.unwrap().unwrap();
    assert_eq!(first.principal, "user-0000000");
    assert_eq!(fake.calls().len(), 1);
    let mut n = 1;
    let mut last = first.principal;
    while let Some(e) = stream.try_next().await.unwrap() {
        assert!(e.principal > last);
        last = e.principal;
        n += 1;
    }
    assert_eq!(n, 25_000);
    assert_eq!(fake.calls().len(), 250);
    assert!(requested(&fake).iter().all(|&l| l == 100));
}

#[tokio::test]
async fn list_access_stream_follows_many_pages() {
    let (fake, builder) = start(scopes_server(12_345)).await;
    let client = builder.build().unwrap();
    let scopes: Vec<_> = client
        .list_access_stream(ListAccess::new("alice", "docs.read", "/docs"))
        .try_collect()
        .await
        .unwrap();
    assert_eq!(scopes.len(), 12_345);
    assert_eq!(scopes[12_344].uri, "/docs/12344");
    assert_eq!(scopes[0].kind, ScopeKind::Subtree);
    assert_eq!(fake.calls().len(), 124);
}

#[tokio::test]
async fn list_access_stream_signals_covers_all_as_one_subtree_scope() {
    let (_fake, builder) = start(|_, _, _| {
        Reply::ok(ListAccessResponse {
            covers_all: true,
            scopes: vec![AccessScope {
                uri: "/docs".into(),
                scope: AccessScopeKind::Subtree as i32,
            }],
            pagination: None,
        })
    })
    .await;
    let client = builder.build().unwrap();
    let scopes: Vec<_> = client
        .list_access_stream(ListAccess::new("alice", "docs.read", "/docs"))
        .try_collect()
        .await
        .unwrap();
    assert_eq!(scopes.len(), 1);
    assert_eq!(
        (scopes[0].uri.as_str(), scopes[0].kind),
        ("/docs", ScopeKind::Subtree)
    );
    let all = client
        .list_access(ListAccess::new("alice", "docs.read", "/docs"))
        .await
        .unwrap();
    assert!(all.covers_all);
}

#[tokio::test]
async fn who_has_access_default_cap_fetches_at_most_cap_plus_one() {
    assert_eq!(DEFAULT_MAX_RESULTS, 10_000);
    let (fake, builder) = start(principals_server(1_000_000)).await;
    let client = builder.build().unwrap();
    let err = client
        .who_has_access(WhoHasAccess::new("/docs/1"))
        .await
        .unwrap_err();
    match &err {
        Error::TooManyResults {
            max_results,
            message,
            ..
        } => {
            assert_eq!(*max_results, 10_000);
            assert!(message.contains("who_has_access_stream"), "{message}");
            assert!(message.contains("max_results"), "{message}");
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(err.code(), None);
    let limits = requested(&fake);
    assert_eq!(limits.iter().map(|&l| l as usize).sum::<usize>(), 10_001);
    assert_eq!(limits.len(), 101);
    assert_eq!(*limits.last().unwrap(), 1);
}

#[tokio::test]
async fn who_has_access_custom_cap() {
    let (fake, builder) = start(principals_server(1_000)).await;
    let client = builder.build().unwrap();
    let err = client
        .who_has_access(WhoHasAccess::new("/docs/1").max_results(250))
        .await
        .unwrap_err();
    assert!(
        matches!(
            err,
            Error::TooManyResults {
                max_results: 250,
                ..
            }
        ),
        "{err:?}"
    );
    assert_eq!(requested(&fake), vec![100, 100, 51]);

    // Exactly at the cap is fine: the cap+1 probe comes back short.
    let (fake, builder) = start(principals_server(250)).await;
    let client = builder.build().unwrap();
    let entries = client
        .who_has_access(WhoHasAccess::new("/docs/1").max_results(250))
        .await
        .unwrap();
    assert_eq!(entries.len(), 250);
    assert_eq!(requested(&fake), vec![100, 100, 51]);

    // Raising the cap returns everything.
    let (_fake, builder) = start(principals_server(1_000)).await;
    let client = builder.build().unwrap();
    let entries = client
        .who_has_access(WhoHasAccess::new("/docs/1").max_results(5_000))
        .await
        .unwrap();
    assert_eq!(entries.len(), 1_000);

    let err = client
        .who_has_access(WhoHasAccess::new("/docs/1").max_results(0))
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Validation { .. }), "{err:?}");
}

#[tokio::test]
async fn list_access_cap_fetches_at_most_cap_plus_one() {
    let (fake, builder) = start(scopes_server(50_000)).await;
    let client = builder.build().unwrap();
    let err = client
        .list_access(ListAccess::new("alice", "docs.read", "/docs"))
        .await
        .unwrap_err();
    assert!(
        matches!(&err, Error::TooManyResults { max_results: 10_000, message, .. } if message.contains("list_access_stream")),
        "{err:?}"
    );
    assert_eq!(
        requested(&fake).iter().map(|&l| l as usize).sum::<usize>(),
        10_001
    );

    let (fake, builder) = start(scopes_server(50_000)).await;
    let client = builder.build().unwrap();
    let err = client
        .list_access(
            ListAccess::new("alice", "docs.read", "/docs")
                .max_results(150)
                .page_size(60),
        )
        .await
        .unwrap_err();
    assert!(
        matches!(
            err,
            Error::TooManyResults {
                max_results: 150,
                ..
            }
        ),
        "{err:?}"
    );
    assert_eq!(requested(&fake), vec![60, 60, 31]);
}

fn allow_all(req: &dyn std::any::Any) -> Reply {
    let req = req.downcast_ref::<BatchCheckRequest>().unwrap();
    Reply::ok(BatchCheckResponse {
        results: req
            .resource_uris
            .iter()
            .map(|u| BatchCheckResult {
                resource_uri: u.clone(),
                allowed: true,
                missing_permissions: vec![],
            })
            .collect(),
    })
}

#[tokio::test]
async fn batch_check_bound() {
    assert_eq!(DEFAULT_MAX_BATCH_RESOURCES, 10_000);
    let (fake, builder) = start(|_, req, _| allow_all(req)).await;
    let client = builder.build().unwrap();
    let too_many: Vec<String> = (0..10_001).map(|i| format!("/docs/{i}")).collect();
    let err = client
        .batch_check(
            BatchCheck::new("alice")
                .permission("p")
                .resources(too_many.clone()),
        )
        .await
        .unwrap_err();
    assert!(
        matches!(err, Error::Validation { status: None, .. }),
        "{err:?}"
    );
    assert!(err.message().contains("max_resources"), "{err}");
    assert!(fake.calls().is_empty());

    // Counted as given, before de-duplication.
    let dupes = vec!["/docs/1".to_string(); 11];
    let err = client
        .batch_check(
            BatchCheck::new("alice")
                .permission("p")
                .resources(dupes)
                .max_resources(10),
        )
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Validation { .. }), "{err:?}");

    let err = client
        .batch_check(
            BatchCheck::new("alice")
                .permission("p")
                .resource("/a")
                .max_resources(0),
        )
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Validation { .. }), "{err:?}");
    assert!(fake.calls().is_empty());

    // Exactly the default bound is accepted; raising it allows more.
    let ok: Vec<String> = too_many[..10_000].to_vec();
    let entries = client
        .batch_check(BatchCheck::new("alice").permission("p").resources(ok))
        .await
        .unwrap();
    assert_eq!(entries.len(), 10_000);
    assert_eq!(fake.calls().len(), 100);
    let entries = client
        .batch_check(
            BatchCheck::new("alice")
                .permission("p")
                .resources(too_many)
                .max_resources(20_000),
        )
        .await
        .unwrap();
    assert_eq!(entries.len(), 10_001);
}

#[tokio::test]
async fn batch_check_keeps_order_with_out_of_order_completion() {
    let (_fake, builder) = start(|_, req, _| {
        let r = req.downcast_ref::<BatchCheckRequest>().unwrap();
        // Earlier chunks answer later.
        let first: usize = r.resource_uris[0]
            .trim_start_matches("/docs/")
            .parse()
            .unwrap();
        let delay = Duration::from_millis(80u64.saturating_sub((first / 100) as u64 * 10));
        Reply::Sleep(delay, Box::new(allow_all(req)))
    })
    .await;
    let client = builder.build().unwrap();
    let resources: Vec<String> = (0..800).map(|i| format!("/docs/{i}")).collect();
    let entries = client
        .batch_check(
            BatchCheck::new("alice")
                .permission("p")
                .resources(resources.clone()),
        )
        .await
        .unwrap();
    assert_eq!(
        entries
            .iter()
            .map(|e| e.resource.clone())
            .collect::<Vec<_>>(),
        resources
    );
}

#[tokio::test]
async fn batch_check_stops_sending_after_the_first_error() {
    // At most four chunks are in flight (see batch_check_has_at_most_four_calls_in_flight
    // in tests/client.rs); after the third chunk fails, no more are started.
    let (fake, builder) = start(move |_, req, _| {
        let r = req.downcast_ref::<BatchCheckRequest>().unwrap();
        let reply = if r.resource_uris[0] == "/docs/200" {
            Reply::Err(Status::permission_denied("no"))
        } else {
            allow_all(req)
        };
        Reply::Sleep(Duration::from_millis(40), Box::new(reply))
    })
    .await;
    let client = builder.build().unwrap();
    let resources: Vec<String> = (0..10_000).map(|i| format!("/docs/{i}")).collect();
    let started = Instant::now();
    let err = client
        .batch_check(
            BatchCheck::new("alice")
                .permission("p")
                .resources(resources),
        )
        .await
        .unwrap_err();
    assert!(matches!(err, Error::Permission { .. }), "{err:?}");
    assert!(started.elapsed() < Duration::from_secs(1));
    tokio::time::sleep(Duration::from_millis(100)).await;
    // Only the first window of chunks was sent, never all 100.
    let sent = fake.calls().len();
    assert!(sent <= 8, "sent {sent} chunks");
}

#[tokio::test]
async fn list_pages_use_the_list_timeout() {
    // Pages take 150 ms: the 50 ms per-attempt timeout would fail them, a
    // 500 ms list timeout lets them through, and checks keep the short one.
    let (_fake, builder) = start(|method, req, _| match method {
        "ListPrincipalsWithAccess" => Reply::Sleep(
            Duration::from_millis(150),
            Box::new(principals_server(3)("", req, 0)),
        ),
        _ => Reply::Sleep(
            Duration::from_millis(150),
            Box::new(Reply::ok(CheckPermissionResponse {
                passed: true,
                missing_permissions: vec![],
            })),
        ),
    })
    .await;
    let client = builder
        .timeout(Duration::from_millis(50))
        .list_timeout(Duration::from_millis(500))
        .retries(0)
        .build()
        .unwrap();
    let entries = client
        .who_has_access(WhoHasAccess::new("/docs/1"))
        .await
        .unwrap();
    assert_eq!(entries.len(), 3);
    let err = client.check("a", "p", "/a").await.unwrap_err();
    assert!(matches!(err, Error::Timeout { .. }), "{err:?}");

    // Without list_timeout, list pages use the per-attempt timeout.
    let (_fake, builder) = start(|_, req, _| {
        Reply::Sleep(
            Duration::from_millis(150),
            Box::new(principals_server(3)("", req, 0)),
        )
    })
    .await;
    let client = builder
        .timeout(Duration::from_millis(50))
        .retries(0)
        .build()
        .unwrap();
    let err = client
        .who_has_access(WhoHasAccess::new("/docs/1"))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Some(Code::DeadlineExceeded));
}

#[tokio::test]
async fn list_page_deadline_exceeded_backs_off_before_retrying() {
    let times: Arc<Mutex<Vec<Instant>>> = Arc::default();
    let rec = times.clone();
    let (fake, builder) = start(move |_, req, attempt| {
        rec.lock().unwrap().push(Instant::now());
        if attempt < 2 {
            Reply::Err(Status::deadline_exceeded("slow"))
        } else {
            principals_server(5)("", req, 0)
        }
    })
    .await;
    let client = builder
        .retry_config(
            permio::RetryConfig::default()
                .max_retries(2)
                .base_delay(Duration::from_millis(40))
                .max_delay(Duration::from_millis(500)),
        )
        .build()
        .unwrap();
    let entries: Vec<_> = client
        .who_has_access_stream(WhoHasAccess::new("/docs/1"))
        .try_collect()
        .await
        .unwrap();
    assert_eq!(entries.len(), 5);
    assert_eq!(fake.calls().len(), 3);
    let t = times.lock().unwrap().clone();
    // Jittered backoff is at least half the exponential ceiling: 20 ms, then 40 ms.
    assert!(
        t[1] - t[0] >= Duration::from_millis(20),
        "{:?}",
        t[1] - t[0]
    );
    assert!(
        t[2] - t[1] >= Duration::from_millis(40),
        "{:?}",
        t[2] - t[1]
    );
}

#[tokio::test]
async fn list_page_rate_limit_waits_for_retry_after() {
    let (fake, builder) = start(|_, req, attempt| {
        if attempt == 0 {
            Reply::Err(rate_limited("slow down", Duration::from_millis(120)))
        } else {
            principals_server(5)("", req, 0)
        }
    })
    .await;
    let client = builder.build().unwrap();
    let started = Instant::now();
    let entries = client
        .who_has_access(WhoHasAccess::new("/docs/1"))
        .await
        .unwrap();
    assert_eq!(entries.len(), 5);
    assert!(started.elapsed() >= Duration::from_millis(120));
    assert_eq!(fake.calls().len(), 2);
}
