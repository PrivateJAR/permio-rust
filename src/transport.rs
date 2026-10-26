//! Authentication, per-attempt timeouts and retries, as a tower service
//! wrapped around the tonic [`Channel`].
//!
//! Both the helper methods on [`Client`](crate::Client) and the raw generated
//! clients ([`Client::raw_permissions`](crate::Client::raw_permissions)) send
//! every call through [`Transport`], so they all behave the same way.

use std::collections::hash_map::RandomState;
use std::convert::Infallible;
use std::fmt;
use std::future::Future;
use std::hash::{BuildHasher, Hasher};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use bytes::Bytes;
use http::{HeaderMap, HeaderValue};
use http_body::Frame;
use http_body_util::{BodyExt, Full, StreamBody};
use tonic::body::Body;
use tonic::codegen::Service;
use tonic::metadata::MetadataMap;
use tonic::transport::Channel;
use tonic::{Code, Status};

use crate::error::{parse_retry_after, server_retry_delay};

type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// An overall deadline for one call, across every attempt and the backoff
/// between them.
///
/// The helper methods set it from their `timeout` options. For a raw client
/// call, put it in the request's extensions:
///
/// ```no_run
/// # async fn demo(client: permio::Client) -> Result<(), permio::Error> {
/// use std::time::Duration;
/// use permio::raw::permissions::v1::WhoAmIRequest;
/// let mut req = tonic::Request::new(WhoAmIRequest {});
/// req.extensions_mut().insert(permio::CallTimeout::new(Duration::from_secs(3)));
/// let me = client.raw_permissions().who_am_i(req).await?;
/// # Ok(()) }
/// ```
///
/// [`Duration::ZERO`] means no overall deadline for this call, even when the
/// client has a default one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CallTimeout(Duration);

impl CallTimeout {
    /// A deadline of `timeout` from when the call starts.
    #[must_use]
    pub fn new(timeout: Duration) -> Self {
        CallTimeout(timeout)
    }

    /// The configured duration.
    #[must_use]
    pub fn duration(self) -> Duration {
        self.0
    }
}

/// Retry settings for idempotent reads.
///
/// Reads on the allowlist (see [`is_idempotent_method`]) are retried on
/// `Unavailable` and `DeadlineExceeded` (including the client's own
/// per-attempt timeout) with jittered exponential backoff. A rate-limited read
/// (`ResourceExhausted`) is retried after the server's requested delay when
/// the server gives one and it is at most `max_delay`. Writes (create, update,
/// delete, grant, revoke, ...) are never retried.
///
/// ```
/// use std::time::Duration;
/// let retry = permio::RetryConfig::default()
///     .max_retries(3)
///     .base_delay(Duration::from_millis(50));
/// assert_eq!(retry.max_retries, 3);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct RetryConfig {
    /// Retries after the first attempt. Default 2.
    pub max_retries: u32,
    /// Delay before the first retry; doubles each time, with jitter. Default 100 ms.
    pub base_delay: Duration,
    /// Upper bound for a single delay. Default 2 s.
    pub max_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        RetryConfig {
            max_retries: 2,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(2),
        }
    }
}

impl RetryConfig {
    /// No retries at all.
    #[must_use]
    pub fn none() -> Self {
        RetryConfig {
            max_retries: 0,
            ..RetryConfig::default()
        }
    }

    /// Sets the number of retries after the first attempt.
    #[must_use]
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Sets the delay before the first retry.
    #[must_use]
    pub fn base_delay(mut self, delay: Duration) -> Self {
        self.base_delay = delay;
        self
    }

    /// Sets the upper bound for a single delay.
    #[must_use]
    pub fn max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }
}

// Read-only RPCs that are safe to repeat. Anything not listed (every Create,
// Update, Delete, Grant, Revoke, Invite, Set, Invalidate, Remove...) is sent
// exactly once.
//
// Grant and Revoke converge on the server (a grant is additive, a revoke
// ignores names the grant lacks), but they stay off this list: after a timeout
// the first attempt may still land, and repeating it could undo another
// caller's revoke or grant made in between. The caller decides instead.
const IDEMPOTENT_METHODS: &[&str] = &[
    "permio.permissions.v1.PermissionsService.Check",
    "permio.permissions.v1.PermissionsService.GetPermission",
    "permio.permissions.v1.PermissionsService.ListPermissions",
    "permio.permissions.v1.PermissionsService.GetRole",
    "permio.permissions.v1.PermissionsService.ListRoles",
    "permio.permissions.v1.PermissionsService.GetUserPermissionsAndRolesOnResource",
    "permio.permissions.v1.PermissionsService.ListPrincipals",
    "permio.permissions.v1.PermissionsService.GetAllAssignmentsForPrincipal",
    "permio.permissions.v1.PermissionsService.GetTotalRolesAndPermissionsForOrganisation",
    "permio.permissions.v1.PermissionsService.BatchCheck",
    "permio.permissions.v1.PermissionsService.ListAccess",
    "permio.permissions.v1.PermissionsService.ListPrincipalsWithAccess",
    "permio.permissions.v1.PermissionsService.GetGroup",
    "permio.permissions.v1.PermissionsService.ListGroups",
    "permio.permissions.v1.PermissionsService.ListGroupMembers",
    "permio.permissions.v1.PermissionsService.ListGroupsForPrincipal",
    "permio.permissions.v1.PermissionsService.WhoAmI",
    "permio.organisation.v1.OrganisationService.GetOrganisation",
    "permio.organisation.v1.OrganisationService.GetUsersOrganisation",
    "permio.organisation.v1.OrganisationService.ListAPIKeys",
    "permio.organisation.v1.OrganisationService.GetProject",
    "permio.organisation.v1.OrganisationService.GetProjectsForOrganisation",
    "permio.organisation.v1.OrganisationService.GetPrincipalUsageStats",
    "permio.organisation.v1.OrganisationService.GetUsageHistory",
    "permio.organisation.v1.OrganisationService.GetOrganisationDashboard",
];

/// Whether the SDK retries this RPC, by fully qualified
/// `package.Service.Method` name (e.g. `permio.permissions.v1.PermissionsService.Check`).
#[must_use]
pub fn is_idempotent_method(name: &str) -> bool {
    IDEMPOTENT_METHODS.contains(&name)
}

/// The SDK's tower service around a tonic [`Channel`]: adds the
/// `authorization: Bearer <key>` header, applies the per-attempt timeout and
/// the overall deadline ([`CallTimeout`]) and retries idempotent reads.
///
/// It buffers each request and response body, which is right for the
/// perms.io API (every public RPC is unary).
#[derive(Clone)]
pub struct Transport {
    inner: Channel,
    authorization: HeaderValue,
    timeout: Option<Duration>,
    call_timeout: Option<Duration>,
    retry: RetryConfig,
}

impl fmt::Debug for Transport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Transport")
            .field("authorization", &"Bearer <redacted>")
            .field("timeout", &self.timeout)
            .field("call_timeout", &self.call_timeout)
            .field("retry", &self.retry)
            .finish_non_exhaustive()
    }
}

impl Transport {
    pub(crate) fn new(
        inner: Channel,
        authorization: HeaderValue,
        timeout: Option<Duration>,
        call_timeout: Option<Duration>,
        retry: RetryConfig,
    ) -> Self {
        Transport {
            inner,
            authorization,
            timeout,
            call_timeout,
            retry,
        }
    }

    pub(crate) fn with_call_timeout(&self, call_timeout: Option<Duration>) -> Self {
        Transport {
            call_timeout,
            ..self.clone()
        }
    }

    async fn run(self, req: http::Request<Body>) -> Result<http::Response<Body>, BoxError> {
        let (mut parts, body) = req.into_parts();
        parts
            .headers
            .insert(http::header::AUTHORIZATION, self.authorization.clone());
        let method = parts.uri.path().trim_start_matches('/').replace('/', ".");
        let retries = if is_idempotent_method(&method) {
            self.retry.max_retries
        } else {
            0
        };
        let body = body
            .collect()
            .await
            .map_err(|e| {
                Box::new(Status::internal(format!(
                    "permio: reading request body: {e}"
                )))
            })?
            .to_bytes();

        // The overall deadline: the call's own CallTimeout, else the client's
        // default. Zero means none.
        let overall = parts
            .extensions
            .get::<CallTimeout>()
            .map(|t| t.0)
            .or(self.call_timeout)
            .filter(|d| !d.is_zero());
        let deadline = overall.map(|d| (d, Instant::now() + d));

        let mut attempt: u32 = 0;
        loop {
            let req = http::Request::from_parts(parts.clone(), Body::new(Full::new(body.clone())));
            let status = match self.attempt(req, deadline).await {
                Ok(Outcome::Ok(response)) => return Ok(response),
                Ok(Outcome::Failed(status)) | Err(status) => status,
            };
            let Some(delay) = self.retry_delay(&status, attempt, retries) else {
                return Err(Box::new(status));
            };
            // Don't start a retry that could not finish before the deadline:
            // report the failure we have instead of a bare timeout.
            if let Some((_, at)) = deadline {
                if Instant::now() + delay >= at {
                    return Err(Box::new(status));
                }
            }
            tokio::time::sleep(delay).await;
            attempt += 1;
        }
    }

    /// The delay before the next attempt, or `None` to give up with `status`.
    fn retry_delay(&self, status: &Status, attempt: u32, retries: u32) -> Option<Duration> {
        // The server's RetryInfo detail / retry-after header, if any. A rate
        // limited (ResourceExhausted) read is only retried when the server
        // says when, and only if that is within max_delay.
        let server_delay = server_retry_delay(status);
        let retryable = matches!(status.code(), Code::Unavailable | Code::DeadlineExceeded)
            || (status.code() == Code::ResourceExhausted && server_delay.is_some());
        if attempt >= retries || !retryable {
            return None;
        }
        if server_delay.is_some_and(|d| d > self.retry.max_delay) {
            return None;
        }
        let exp = self
            .retry
            .base_delay
            .saturating_mul(2u32.saturating_pow(attempt.min(31)));
        let ceiling = exp.min(self.retry.max_delay);
        let backoff = ceiling / 2 + ceiling.mul_f64(random_unit() / 2.0);
        Some(match server_delay {
            None => backoff,
            // Wait at least as long as the server asked, plus a little jitter
            // so clients limited in the same second don't all retry at once.
            Some(d) => backoff.max(
                d + ceiling
                    .min(Duration::from_millis(100))
                    .mul_f64(random_unit()),
            ),
        })
    }

    async fn attempt(
        &self,
        req: http::Request<Body>,
        deadline: Option<(Duration, Instant)>,
    ) -> Result<Outcome, Status> {
        let mut svc = self.inner.clone();
        let call = async move {
            std::future::poll_fn(|cx| svc.poll_ready(cx))
                .await
                .map_err(transport_status)?;
            let response = svc.call(req).await.map_err(transport_status)?;
            let (parts, body) = response.into_parts();
            let collected = body.collect().await.map_err(transport_status)?;
            Ok::<_, Status>(finish(parts, collected))
        };
        let per_attempt = self.timeout.map(|limit| {
            (
                Instant::now() + limit,
                Status::deadline_exceeded(format!("request timed out after {limit:?}")),
            )
        });
        let overall = deadline.map(|(limit, at)| {
            (
                at,
                Status::deadline_exceeded(format!(
                    "call exceeded its overall timeout of {limit:?}"
                )),
            )
        });
        // Whichever deadline comes first applies to this attempt.
        let limit = match (per_attempt, overall) {
            (Some(a), Some(b)) => Some(if b.0 <= a.0 { b } else { a }),
            (a, b) => a.or(b),
        };
        match limit {
            Some((at, status)) => tokio::time::timeout_at(at.into(), call)
                .await
                .map_err(|_| status)?,
            None => call.await,
        }
    }
}

enum Outcome {
    Ok(http::Response<Body>),
    Failed(Status),
}

/// Turns a buffered response into either a replayable OK response or the
/// call's error status.
fn finish(parts: http::response::Parts, collected: http_body_util::Collected<Bytes>) -> Outcome {
    let trailers = collected.trailers().cloned();
    let headers = &parts.headers;
    let status = if headers.contains_key("grpc-status") {
        Status::from_header_map(headers)
    } else if let Some(t) = trailers.as_ref().filter(|t| t.contains_key("grpc-status")) {
        Status::from_header_map(t)
    } else if parts.status != http::StatusCode::OK {
        Some(Status::with_metadata(
            http_status_code(parts.status),
            format!("HTTP status {}", parts.status),
            MetadataMap::from_headers(headers.clone()),
        ))
    } else {
        None
    };
    match status {
        Some(mut status) if status.code() != Code::Ok => {
            // A retry-after sent as a response header (not a trailer) still counts.
            copy_retry_after(headers, status.metadata_mut());
            Outcome::Failed(status)
        }
        _ => {
            let mut frames: Vec<Result<Frame<Bytes>, Infallible>> =
                vec![Ok(Frame::data(collected.to_bytes()))];
            if let Some(t) = trailers {
                frames.push(Ok(Frame::trailers(t)));
            }
            let body = Body::new(StreamBody::new(futures_util::stream::iter(frames)));
            Outcome::Ok(http::Response::from_parts(parts, body))
        }
    }
}

fn copy_retry_after(headers: &HeaderMap, metadata: &mut MetadataMap) {
    if metadata.get("retry-after").is_some() {
        return;
    }
    if let Some(v) = headers
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .filter(|v| parse_retry_after(v).is_some())
        .and_then(|v| v.parse().ok())
    {
        metadata.insert("retry-after", v);
    }
}

// The gRPC spec's mapping for responses without a grpc-status.
fn http_status_code(status: http::StatusCode) -> Code {
    match status.as_u16() {
        400 => Code::Internal,
        401 => Code::Unauthenticated,
        403 => Code::PermissionDenied,
        404 => Code::Unimplemented,
        // The gRPC spec maps 429 to Unavailable; the perms.io gateway uses it
        // for rate limiting, so it is reported as rate limited instead (and
        // only retried when the server says when).
        429 => Code::ResourceExhausted,
        502..=504 => Code::Unavailable,
        _ => Code::Unknown,
    }
}

/// A failure below gRPC (DNS, refused connection, TLS, reset stream) is
/// reported as `Unavailable` so reads are retried and callers get
/// [`Error::Unavailable`](crate::Error::Unavailable).
fn transport_status<E: Into<BoxError>>(err: E) -> Status {
    let err: BoxError = err.into();
    match err.downcast::<Status>() {
        Ok(status) => *status,
        Err(err) => {
            let status = Status::from_error(err);
            if status.code() == Code::Unknown {
                Status::unavailable(status.message().to_string())
            } else {
                status
            }
        }
    }
}

/// A number in `[0, 1)` for jitter. `RandomState` is seeded randomly per
/// instance, which is plenty for spreading retries.
fn random_unit() -> f64 {
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u64(0x5eed);
    #[allow(clippy::cast_precision_loss)]
    let value = (hasher.finish() >> 11) as f64;
    value / (1u64 << 53) as f64
}

impl Service<http::Request<Body>> for Transport {
    type Response = http::Response<Body>;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Readiness of the channel is awaited per attempt in `call`.
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<Body>) -> Self::Future {
        Box::pin(self.clone().run(req))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowlist() {
        assert!(is_idempotent_method(
            "permio.permissions.v1.PermissionsService.Check"
        ));
        assert!(is_idempotent_method(
            "permio.permissions.v1.PermissionsService.WhoAmI"
        ));
        assert!(!is_idempotent_method(
            "permio.permissions.v1.PermissionsService.GrantUserPermissionsAndRolesOnResource"
        ));
        assert!(!is_idempotent_method(
            "permio.permissions.v1.PermissionsService.CreateGroup"
        ));
    }

    #[test]
    fn jitter_in_range() {
        for _ in 0..1000 {
            let x = random_unit();
            assert!((0.0..1.0).contains(&x));
        }
    }

    async fn http_failure(status: u16, retry_after: Option<&str>) -> Status {
        let mut builder = http::Response::builder().status(status);
        if let Some(v) = retry_after {
            builder = builder.header("retry-after", v);
        }
        let (parts, body) = builder.body(Full::new(Bytes::new())).unwrap().into_parts();
        let collected = body.collect().await.unwrap();
        match finish(parts, collected) {
            Outcome::Failed(status) => status,
            Outcome::Ok(_) => panic!("HTTP {status} must fail"),
        }
    }

    #[tokio::test]
    async fn http_429_without_grpc_status_is_rate_limited() {
        let status = http_failure(429, Some("2")).await;
        assert_eq!(status.code(), Code::ResourceExhausted);
        match crate::Error::from(status) {
            crate::Error::RateLimited { retry_after, .. } => {
                assert_eq!(retry_after, Some(Duration::from_secs(2)));
            }
            other => panic!("unexpected {other:?}"),
        }

        let date = httpdate::fmt_http_date(std::time::SystemTime::now() + Duration::from_secs(30));
        let status = http_failure(429, Some(&date)).await;
        let delay = server_retry_delay(&status).expect("HTTP-date retry-after kept");
        assert!(delay > Duration::from_secs(27) && delay <= Duration::from_secs(30));

        let status = http_failure(429, None).await;
        assert!(matches!(
            crate::Error::from(status),
            crate::Error::RateLimited {
                retry_after: None,
                ..
            }
        ));
        assert!(matches!(
            crate::Error::from(http_failure(503, None).await),
            crate::Error::Unavailable { .. }
        ));
    }

    #[test]
    fn retry_config_builder() {
        let r = RetryConfig::default()
            .max_retries(5)
            .base_delay(Duration::from_millis(1))
            .max_delay(Duration::from_millis(9));
        assert_eq!(r.max_retries, 5);
        assert_eq!(r.base_delay, Duration::from_millis(1));
        assert_eq!(r.max_delay, Duration::from_millis(9));
        assert_eq!(RetryConfig::none().max_retries, 0);
    }

    #[test]
    fn http_status_mapping() {
        assert_eq!(
            http_status_code(http::StatusCode::TOO_MANY_REQUESTS),
            Code::ResourceExhausted
        );
        assert_eq!(
            http_status_code(http::StatusCode::SERVICE_UNAVAILABLE),
            Code::Unavailable
        );
        assert_eq!(
            http_status_code(http::StatusCode::UNAUTHORIZED),
            Code::Unauthenticated
        );
    }
}
