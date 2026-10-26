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

### Changed
- Licensed under Apache-2.0 (LICENSE file added).
- Minimum Rust version is 1.88 (required by tonic 0.14).
- TLS (`tls` feature) is on by default.
