# permio

Rust SDK for [perms.io](https://perms.io), permissions as a service with
hierarchical resource-URI RBAC.

- One call to ask "may this principal do X on this resource?"
- Batch checks, groups, grants and "where can they do X?" queries that turn into database filters
- One typed `permio::Error` enum, per-attempt timeouts, retries for idempotent reads, `Stream`s over list endpoints
- Async on Tokio + tonic, TLS via rustls
- The full generated API (every RPC of `PermissionsService` and `OrganisationService`) is available as an escape hatch in `permio::raw`

## Install

The crate is not on crates.io yet. Until it is, depend on the repository:

```toml
[dependencies]
permio = { git = "https://github.com/PrivateJAR/permio-rust", branch = "main" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Once it is published:

```sh
cargo add permio
cargo add tokio --features macros,rt-multi-thread
```

Rust 1.88 or newer (tonic 0.14's minimum).

## Quick start

Create an API key in the perms.io dashboard (project, **API Keys**, **Create
New Key**), grant `docs.read` to `alice` on `/docs/42`, then:

```rust
#[tokio::main]
async fn main() -> Result<(), permio::Error> {
    let client = permio::Client::builder().project("my-app").build()?; // key from PERMIO_API_KEY
    let allowed = client.check("alice", "docs.read", &permio::resource!("docs", 42)?).await?;
    println!("{allowed}"); // true
    Ok(())
}
```

A principal lacking the permission is a normal answer: `check` returns
`Ok(false)`. It only returns `Err` when the call itself fails (bad key,
malformed input, network trouble). A runnable version is in
[`examples/client.rs`](examples/client.rs).

## Configuration

```rust
use std::time::Duration;

let client = permio::Client::builder()
    .api_key("perms_...")               // default: the PERMIO_API_KEY env var; format + checksum checked locally
    .endpoint(permio::DEV_ENDPOINT)     // default permio::DEFAULT_ENDPOINT (https://api.perms.io, production)
    .project("my-app")                  // default project for every call
    .timeout(Duration::from_secs(5))    // per attempt; default 10 s, Duration::ZERO disables
    .retries(3)                         // idempotent reads only; default 2 (see RetryConfig for delays)
    .build()?;                          // must run inside a Tokio runtime; connects lazily

let other = client.with_project("other-app"); // cheap clone with another default project
```

`Client` is cheap to clone and safe to share; clones share one HTTP/2
connection. `.channel(tonic::transport::Channel)` injects your own channel
(e.g. an in-process test server); auth, timeouts and retries still apply.

Features: `tls` (default) enables rustls with the platform's roots plus the
bundled webpki roots. Without it only `http://` endpoints work.

### Dev vs prod

| Constant | URL |
| --- | --- |
| `permio::DEFAULT_ENDPOINT` | `https://api.perms.io` (production, the default) |
| `permio::DEV_ENDPOINT` | `https://dev-api.perms.io` (development) |

An API key only works on the environment and project it was created in.
`client.who_am_i()` tells you which project a key belongs to and whether it
has check-only or full access.

## Checking permissions

```rust
use permio::{BatchCheck, Check};

client.check("alice", "docs.read", "/docs/42").await?;                      // bool
client.check_all("alice", ["docs.read", "docs.write"], "/docs/42").await?;  // every permission
client.check_any("alice", ["docs.read", "docs.write"], "/docs/42").await?;  // at least one
client.assert("alice", "docs.read", "/docs/42").await?;                     // Err(Error::Denied { missing, .. }) when not

// Several resources, another project, and what is missing:
let result = client
    .check_detailed(Check::new("alice").permission("docs.read").resources(["/docs/1", "/docs/2"]).project("other"))
    .await?;
println!("{} {:?}", result.allowed, result.missing);

// One answer per resource (more than 100 are split into concurrent calls):
let entries = client
    .batch_check(BatchCheck::new("alice").permission("docs.read").resources(doc_uris))
    .await?;
let visible: Vec<_> = entries.iter().filter(|e| e.allowed).map(|e| &e.resource).collect();
```

Grants apply to the URI and everything below it; a `_` segment in a grant
matches exactly one segment.

### Resource URIs

`permio::resource!("projects", project_id, "docs", doc_id)?` (or
`permio::resource([...])`) builds `/projects/p1/docs/a%2Fb`, percent-encoding
each segment so IDs containing `/`, spaces or `( ) *` stay one segment. Build
URIs the same way when granting and when checking. `validate_resource_uri`
checks a URI's shape locally; every helper calls it before sending.

## Granting and revoking

```rust
use permio::Grant;

client.grant(Grant::new("alice", "/docs").permission("docs.read")).await?;
client.grant(Grant::new("alice", "/docs").role(role_id)).await?;   // role IDs: client.list_roles()
client.revoke(Grant::new("alice", "/docs").permission("docs.read")).await?;
```

Grants are additive. A revoke removes names from the grant on exactly that
URI (grants on ancestors stay) and ignores names the grant lacks. Both need a
full-access key and are never retried automatically.

## Groups

```rust
let groups = client.groups();
groups.create("editors", Some("Docs editors")).await?;
groups.add_members("editors", ["alice", "bob"]).await?;          // AddMembersResult { added, existing }
client.grant(Grant::new(permio::group("editors")?, "/docs").permission("docs.write")).await?;
client.check("bob", "docs.write", "/docs/1").await?;              // true, through the group
groups.remove_members("editors", ["bob"]).await?;
groups.delete("editors").await?;                                  // also removes its grants
```

Also `get`, `update` (description), `list`, `list_members` and
`list_for_principal`. Group methods accept `"editors"` or `"group:editors"`.
Groups cannot contain groups.

## Filtering lists: `list_access` and `who_has_access`

```rust
let access = client.list_access("alice", "docs.read", "/docs").await?;
if access.covers_all {
    // everything under /docs
} else {
    let prefixes = access.to_prefixes();   // WHERE path = p OR path LIKE p || '/%'
    let regex = access.regex_pattern();    // or one anchored regex (PostgreSQL ~, MongoDB $regex)
    let ok = access.allows("/docs/42");    // or test URIs in memory
}

let who = client
    .who_has_access(permio::WhoHasAccess::new("/docs/42").permission("docs.read").expand_groups(true))
    .await?;                               // Vec<AccessEntry { principal, is_group, via_group, via }>
```

## Listing

List endpoints return a `permio::ListStream<T>` (a boxed `futures::Stream`)
that fetches pages lazily:

```rust
use futures_util::TryStreamExt;

let mut roles = client.list_roles(permio::ListOptions::default().search("doc"));
while let Some(role) = roles.try_next().await? {
    println!("{} {}", role.id, role.name);
}
let principals: Vec<String> = client.list_principals(Default::default()).try_collect().await?;
```

`list_permissions`, `list_roles`, `list_principals`, `list_assignments`,
`groups().list`, `groups().list_members`, `groups().list_for_principal`.
`permio::paginate` wraps any raw list RPC the same way.

## Errors

Every fallible call returns `Result<_, permio::Error>`:

| Variant | When |
| --- | --- |
| `Config` | Missing/malformed API key, no project, bad endpoint, `tls` feature off, no Tokio runtime |
| `Auth` | Key unknown, invalidated or missing (gRPC `Unauthenticated`) |
| `Permission { missing, .. }` | Key lacks the rights for the call (`PermissionDenied`); `missing` parsed when the API lists it |
| `Validation` | Malformed input, caught locally (`status: None`) or by the API (`InvalidArgument`) |
| `NotFound` | Entity does not exist (`NotFound`) |
| `Unavailable` | API unreachable or temporarily down (`Unavailable`, connection/TLS failures) |
| `Timeout` | Per-attempt timeout or `DeadlineExceeded` |
| `RateLimited { retry_after, .. }` | Rate limit used up (`ResourceExhausted`), with the server's requested delay |
| `Denied { principal, missing, .. }` | Only from `assert*`: the principal lacks the permission |
| `Other` | Any other gRPC code (e.g. `AlreadyExists`), or an inconsistent API response |

`err.code()` gives the gRPC code, `err.message()` the server's message and
`err.status()` the original `tonic::Status`. Errors from raw clients are
`tonic::Status`; `permio::Error::from(status)` maps them.

### Retries and timeouts

Each attempt times out after 10 s. Idempotent reads (Check, BatchCheck,
ListAccess, WhoAmI, every Get/List; see `permio::is_idempotent_method`) are
retried up to 2 times on `Unavailable` / `DeadlineExceeded`, with exponential
backoff from 100 ms (each delay capped at 2 s and jittered between half and all of it). A
rate-limited read is retried after the server's `RetryInfo` / `retry-after`
delay when that is at most `max_delay`; otherwise `Error::RateLimited` is
returned with `retry_after` set. Writes (create, update, delete, grant,
revoke, ...) are sent exactly once. For an overall deadline across retries,
wrap the call in `tokio::time::timeout`.

## Raw clients (full API)

`permio::raw` holds the generated prost messages and tonic clients for
`permio.permissions.v1`, `permio.organisation.v1` and `pagination`. The
clients from `client.raw_permissions()` / `client.raw_organisations()` share
the SDK's connection, auth, timeouts and retries:

```rust
use permio::raw::permissions::v1::CreatePermissionRequest;

let permission = client
    .raw_permissions()
    .create_permission(CreatePermissionRequest {
        project_name: "my-app".into(),
        name: "docs.read".into(),
        description: "Read documents".into(),
    })
    .await
    .map_err(permio::Error::from)?
    .into_inner();
```

Permission and role CRUD, organisation, project and API key management are
available this way.

## Integration tests

`tests/live.rs` runs the same scenarios as permio-js against a real
deployment (whoAmI, grant/check, batch check, list access, who has access,
revoke, groups, bad key). It is skipped unless `PERMIO_API_KEY` is set, uses
`PERMIO_BASE_URL` (default the dev API), refuses to run against production,
and uses `PERMIO_PROJECT` or the key's own project. It needs a full-access
key and creates fixed `sdk-it-rust-*` IDs, removed before and after the run.

```sh
PERMIO_API_KEY=perms_... cargo test --test live -- --nocapture
```

CI runs it when the `PERMIO_DEV_API_KEY` secret is set.

## Development

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test            # unit tests run against an in-process tonic server
cargo doc --no-deps
```

The files under `src/permio/` and `src/pagination/` are generated from
[PrivateJAR/proto](https://github.com/PrivateJAR/proto) (prost 0.14 / tonic
0.14); don't edit them by hand. The hand-written SDK is everything else.

## Releasing

Publishing is done by `.github/workflows/release.yml` on a `v*` tag that
matches the version in `Cargo.toml`:

1. Bump `version` in `Cargo.toml` and update `CHANGELOG.md`.
2. Tag and push: `git tag v0.1.0 && git push origin v0.1.0`.

The workflow needs a crates.io API token (scope `publish-new` /
`publish-update` for `permio`) in the repository secret
`CARGO_REGISTRY_TOKEN`.

## License

Apache-2.0. See [LICENSE](LICENSE).
