# Changelog

All notable changes to this crate are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the crate uses
[Semantic Versioning](https://semver.org/).

## [0.1.0] - Unreleased

The first release of the hand-written SDK. The crate is renamed from
`permio-rust` (generated stubs only, never published) to `permio`.

### Added
- `permio::Client` with a builder: API key (or `PERMIO_API_KEY`) with local
  format and checksum validation, endpoint (`DEFAULT_ENDPOINT` /
  `DEV_ENDPOINT`), default project, per-attempt timeout (10 s), retries and an
  injectable tonic `Channel`.
- Checks: `check`, `check_all`, `check_any`, `check_detailed`, `assert`,
  `assert_detailed`, `batch_check` (split into concurrent calls above 100
  resources, fails closed).
- `list_access` (`AccessScopes` with `allows`, `to_prefixes`, `patterns`,
  `regex_pattern`) and `who_has_access`.
- `grant` / `revoke`, `who_am_i`.
- Groups: create, get, update, delete, list, add/remove members, list members,
  list a principal's groups; `group()` principal helper.
- Paginated `ListStream`s for permissions, roles, principals, assignments and
  groups; `paginate` for raw list RPCs.
- `permio::Error` enum: `Config`, `Auth`, `Permission`, `Validation`,
  `NotFound`, `Unavailable`, `Timeout`, `RateLimited`, `Denied`, `Other`.
- Retries with jittered exponential backoff for idempotent reads on
  `Unavailable` / `DeadlineExceeded`; rate-limited reads wait for the server's
  `RetryInfo` / `retry-after` delay. Writes are never retried.
- `resource!` / `resource()` URI builders and `validate_resource_uri`.
- `permio::raw`: the generated prost/tonic code, with raw clients that share
  the SDK's auth, timeouts and retries.
- Unit tests against an in-process tonic server; live integration test
  (`tests/live.rs`); CI and crates.io release workflows on Blacksmith.

- Per-call project for every call: `.project()` on `Grant`, `ListAccess` and
  `WhoHasAccess` (as on `Check` / `BatchCheck`), `ListOptions::project` for
  every list stream, and `Groups::project` for group methods.
- Overall deadline across retries: `ClientBuilder::call_timeout` (client
  default), `Client::with_call_timeout`, per-call `.timeout()` on `Check`,
  `BatchCheck`, `Grant`, `ListAccess`, `WhoHasAccess`, `ListOptions` and
  `Groups`, and `permio::CallTimeout` for raw calls.
- `RetryConfig` setters (`max_retries`, `base_delay`, `max_delay`),
  `ListOptions::new`, `MissingPermission::new`.
- `permio::prost` and `permio::prost_types` re-exports (next to
  `permio::tonic`); the README documents their semver relationship.
- `Cargo.lock` is committed; the MSRV CI job builds with `--locked`.

### Changed
- `list_access` takes a `ListAccess` (`ListAccess::new(principal,
  permission, under)`) instead of three arguments.
- Public result and option structs (`CheckResult`, `BatchCheckEntry`,
  `WhoAmI`, `AccessScope(s)`, `AccessEntry`, `AccessGrant`,
  `MissingPermission`, `AddMembersResult`, `RemoveMembersResult`,
  `ListOptions`, `RetryConfig`, and the request builders) and every `Error`
  variant are `#[non_exhaustive]`; build options with their constructors and
  setters, and match errors with `..`.
- An HTTP 429 without a `grpc-status` maps to `Error::RateLimited` (was
  `Unavailable`), so rate-limited writes are never retried and reads only
  after the server's delay.
- `retry-after` accepts an HTTP-date as well as delay-seconds.
- A repeated pagination cursor ends a list stream with `Error::Other` instead
  of silently returning a truncated listing.
- `batch_check` keeps at most 4 `BatchCheck` calls in flight.
- The live integration test only runs against an allowlist of hosts
  (`dev-api.perms.io`, loopback, `*.localhost`, `*.test`) instead of
  refusing a blocklist, and fails rather than skips on any other target.
- Licensed under Apache-2.0 (LICENSE file added).
- Minimum Rust version is 1.88 (required by tonic 0.14).
- TLS (`tls` feature) is on by default.

### Removed
- `rate_limited_status` is no longer public (it was a test helper).

### Fixed
- `check_any` returned `true` when the API denied the check without listing
  what was missing; it now fails closed and returns `false`.
