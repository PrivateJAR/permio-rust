//! Typed errors.

use std::time::Duration;

use tonic::{Code, Status};

/// Permissions missing on one resource.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct MissingPermission {
    /// The resource URI the permissions are missing on.
    pub resource: String,
    /// The permission names the principal does not hold on `resource`.
    pub permissions: Vec<String>,
}

impl MissingPermission {
    /// `permissions` missing on `resource`.
    #[must_use]
    pub fn new<I, S>(resource: impl Into<String>, permissions: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        MissingPermission {
            resource: resource.into(),
            permissions: permissions.into_iter().map(Into::into).collect(),
        }
    }
}

/// Every error returned by the SDK.
///
/// API failures keep the original [`tonic::Status`] (see [`Error::status`]);
/// [`Error::code`] gives its gRPC code. A principal failing a check is *not*
/// an error: [`Client::check`](crate::Client::check) returns `false`, and only
/// [`Client::assert`](crate::Client::assert) turns a denial into
/// [`Error::Denied`].
///
/// The enum and each variant are `#[non_exhaustive]`: match variants with
/// `..` and keep a wildcard arm, so new kinds and fields can be added in minor
/// releases.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The client was configured incorrectly, e.g. a malformed API key, a
    /// missing project or an invalid endpoint.
    #[error("{message}")]
    #[non_exhaustive]
    Config {
        /// What is wrong.
        message: String,
    },

    /// The API key was missing, unknown, invalidated or otherwise rejected
    /// (gRPC `Unauthenticated`).
    #[error("permio: unauthenticated: {message}")]
    #[non_exhaustive]
    Auth {
        /// The server's message.
        message: String,
        /// The original status.
        status: Box<Status>,
    },

    /// The API key is valid but lacks the rights for this call (gRPC
    /// `PermissionDenied`).
    ///
    /// For `Check`, the key needs `permission.check` on the checked resources
    /// and on the principal. When the API lists what is missing, it is parsed
    /// into `missing`; those resource URIs are the fully qualified internal ones
    /// (`/organisation/{org}/project/{project}/...`), not the URIs you sent.
    #[error("permio: permission denied: {message}")]
    #[non_exhaustive]
    Permission {
        /// The server's message.
        message: String,
        /// What the API key is missing, when the API said.
        missing: Vec<MissingPermission>,
        /// The original status.
        status: Box<Status>,
    },

    /// The request was rejected as malformed (gRPC `InvalidArgument`), or
    /// failed the SDK's own checks (then `status` is `None`).
    #[error("{message}")]
    #[non_exhaustive]
    Validation {
        /// What is wrong.
        message: String,
        /// The original status, for errors from the API.
        status: Option<Box<Status>>,
    },

    /// The requested entity does not exist (gRPC `NotFound`).
    #[error("permio: not found: {message}")]
    #[non_exhaustive]
    NotFound {
        /// The server's message.
        message: String,
        /// The original status.
        status: Box<Status>,
    },

    /// The API could not be reached or is temporarily unavailable (gRPC
    /// `Unavailable`).
    #[error("permio: unavailable: {message}")]
    #[non_exhaustive]
    Unavailable {
        /// The server's or transport's message.
        message: String,
        /// The original status.
        status: Box<Status>,
    },

    /// The call did not finish in time (gRPC `DeadlineExceeded`, or the
    /// client's per-attempt timeout).
    #[error("permio: timed out: {message}")]
    #[non_exhaustive]
    Timeout {
        /// The server's or client's message.
        message: String,
        /// The original status.
        status: Box<Status>,
    },

    /// The organisation's rate limit is used up (gRPC `ResourceExhausted`).
    ///
    /// Idempotent reads are retried automatically after the server's requested
    /// delay when it is within [`RetryConfig::max_delay`](crate::RetryConfig);
    /// otherwise this is returned with `retry_after` set to that delay.
    #[error("permio: rate limited: {message}")]
    #[non_exhaustive]
    RateLimited {
        /// The server's message.
        message: String,
        /// How long the server asked the client to wait, if it said
        /// (`google.rpc.RetryInfo` detail, else the `retry-after` header).
        retry_after: Option<Duration>,
        /// The original status.
        status: Box<Status>,
    },

    /// Returned by [`Client::assert`](crate::Client::assert) when the
    /// principal does not hold the requested permissions. This is a normal
    /// authorization outcome, not an API failure, so it has no status.
    #[error("{message}")]
    #[non_exhaustive]
    Denied {
        /// Summary of what is missing.
        message: String,
        /// The principal that was checked.
        principal: String,
        /// What is missing, per resource.
        missing: Vec<MissingPermission>,
    },

    /// A collect-all call ([`Client::who_has_access`](crate::Client::who_has_access),
    /// [`Client::list_access`](crate::Client::list_access)) found more than
    /// its `max_results` cap (default 10,000). Nothing beyond `max_results + 1`
    /// items was fetched. Use the streaming form
    /// ([`Client::who_has_access_stream`](crate::Client::who_has_access_stream),
    /// [`Client::list_access_stream`](crate::Client::list_access_stream)) or
    /// raise `max_results`.
    #[error("{message}")]
    #[non_exhaustive]
    TooManyResults {
        /// What happened and what to do instead.
        message: String,
        /// The cap that was exceeded.
        max_results: usize,
    },

    /// Any other API failure (e.g. `AlreadyExists`, `Internal`), or a
    /// response the SDK could not trust (then `status` is `None`).
    #[error("{message}")]
    #[non_exhaustive]
    Other {
        /// The message.
        message: String,
        /// The original status, for errors from the API.
        status: Option<Box<Status>>,
    },
}

impl Error {
    pub(crate) fn config(message: impl Into<String>) -> Self {
        Error::Config {
            message: message.into(),
        }
    }

    pub(crate) fn validation(message: impl Into<String>) -> Self {
        Error::Validation {
            message: message.into(),
            status: None,
        }
    }

    pub(crate) fn too_many_results(call: &str, stream: &str, max_results: usize) -> Self {
        Error::TooManyResults {
            message: format!(
                "permio: {call} found more than {max_results} results; use {stream} to stream them, or raise max_results"
            ),
            max_results,
        }
    }

    pub(crate) fn inconsistent(what: impl std::fmt::Display) -> Self {
        Error::Other {
            message: format!("permio: inconsistent response from the API: {what}"),
            status: None,
        }
    }

    pub(crate) fn denied(principal: &str, missing: Vec<MissingPermission>) -> Self {
        let summary = missing
            .iter()
            .map(|m| format!("{} on {}", m.permissions.join(", "), m.resource))
            .collect::<Vec<_>>()
            .join("; ");
        let summary = if summary.is_empty() {
            "the requested permissions".to_string()
        } else {
            summary
        };
        Error::Denied {
            message: format!("principal {principal:?} is missing {summary}"),
            principal: principal.to_string(),
            missing,
        }
    }

    /// The original gRPC status, when the error came from the API or the
    /// transport.
    #[must_use]
    pub fn status(&self) -> Option<&Status> {
        match self {
            Error::Auth { status, .. }
            | Error::Permission { status, .. }
            | Error::NotFound { status, .. }
            | Error::Unavailable { status, .. }
            | Error::Timeout { status, .. }
            | Error::RateLimited { status, .. } => Some(status),
            Error::Validation { status, .. } | Error::Other { status, .. } => status.as_deref(),
            Error::Config { .. } | Error::Denied { .. } | Error::TooManyResults { .. } => None,
        }
    }

    /// The gRPC status code, when the error came from (or maps to) an API status.
    #[must_use]
    pub fn code(&self) -> Option<Code> {
        self.status().map(Status::code)
    }

    /// The error message without the `permio: <kind>:` prefix.
    #[must_use]
    pub fn message(&self) -> &str {
        match self {
            Error::Config { message }
            | Error::Auth { message, .. }
            | Error::Permission { message, .. }
            | Error::Validation { message, .. }
            | Error::NotFound { message, .. }
            | Error::Unavailable { message, .. }
            | Error::Timeout { message, .. }
            | Error::RateLimited { message, .. }
            | Error::Denied { message, .. }
            | Error::TooManyResults { message, .. }
            | Error::Other { message, .. } => message,
        }
    }

    /// Response metadata (headers/trailers) sent with the error, if any.
    #[must_use]
    pub fn metadata(&self) -> Option<&tonic::metadata::MetadataMap> {
        self.status().map(Status::metadata)
    }
}

impl From<Status> for Error {
    fn from(status: Status) -> Self {
        let message = status.message().to_string();
        match status.code() {
            Code::Unauthenticated => Error::Auth {
                message,
                status: Box::new(status),
            },
            Code::PermissionDenied => Error::Permission {
                missing: parse_missing_permissions(&message),
                message,
                status: Box::new(status),
            },
            Code::InvalidArgument => Error::Validation {
                message,
                status: Some(Box::new(status)),
            },
            Code::NotFound => Error::NotFound {
                message,
                status: Box::new(status),
            },
            Code::Unavailable => Error::Unavailable {
                message,
                status: Box::new(status),
            },
            Code::DeadlineExceeded => Error::Timeout {
                message,
                status: Box::new(status),
            },
            Code::ResourceExhausted => Error::RateLimited {
                retry_after: server_retry_delay(&status),
                message,
                status: Box::new(status),
            },
            code => Error::Other {
                message: if message.is_empty() {
                    format!("permio: {code:?}")
                } else {
                    format!("permio: {code:?}: {message}")
                },
                status: Some(Box::new(status)),
            },
        }
    }
}

/// Extracts the missing-permission list from a `PermissionDenied` message.
///
/// The permissions service formats it as Go's `%+v` of its missing list:
/// `missing permissions: [{ResourceURI:/organisation/o/project/p/doc MissingPermissions:[permission.check]} ...]`
#[must_use]
pub fn parse_missing_permissions(message: &str) -> Vec<MissingPermission> {
    const START: &str = "{ResourceURI:";
    const MIDDLE: &str = " MissingPermissions:[";
    let mut out = Vec::new();
    let mut rest = message;
    while let Some(i) = rest.find(START) {
        rest = &rest[i + START.len()..];
        let uri_end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        let resource = &rest[..uri_end];
        let after = &rest[uri_end..];
        let Some(after) = after.strip_prefix(MIDDLE) else {
            continue;
        };
        let Some(close) = after.find(']') else {
            break;
        };
        if !after[close..].starts_with("]}") {
            rest = after;
            continue;
        }
        out.push(MissingPermission {
            resource: resource.to_string(),
            permissions: after[..close]
                .split(' ')
                .filter(|p| !p.is_empty())
                .map(str::to_string)
                .collect(),
        });
        rest = &after[close + 2..];
    }
    out
}

// google.rpc.Status and google.rpc.RetryInfo, declared here so no generated
// googleapis types are needed.
#[derive(Clone, PartialEq, prost::Message)]
struct RpcStatus {
    #[prost(int32, tag = "1")]
    code: i32,
    #[prost(string, tag = "2")]
    message: String,
    #[prost(message, repeated, tag = "3")]
    details: Vec<prost_types::Any>,
}

#[derive(Clone, PartialEq, prost::Message)]
struct RetryInfo {
    #[prost(message, optional, tag = "1")]
    retry_delay: Option<prost_types::Duration>,
}

fn retry_info_delay(details: &[u8]) -> Option<Duration> {
    use prost::Message as _;
    if details.is_empty() {
        return None;
    }
    let status = RpcStatus::decode(details).ok()?;
    status.details.iter().find_map(|any| {
        let name = any.type_url.rsplit('/').next().unwrap_or_default();
        if name != "google.rpc.RetryInfo" {
            return None;
        }
        let delay = RetryInfo::decode(any.value.as_slice()).ok()?.retry_delay?;
        let secs = u64::try_from(delay.seconds.max(0)).unwrap_or(0);
        let nanos = u32::try_from(delay.nanos.max(0)).unwrap_or(0);
        Some(Duration::new(secs, 0) + Duration::from_nanos(u64::from(nanos)))
    })
}

/// Parses a `retry-after` value: delay-seconds, or an HTTP-date (RFC 9110),
/// which is turned into the time left until then (zero if it has passed).
pub(crate) fn parse_retry_after(value: &str) -> Option<Duration> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if value.bytes().all(|b| b.is_ascii_digit()) {
        return value.parse::<u64>().ok().map(Duration::from_secs);
    }
    let at = httpdate::parse_http_date(value).ok()?;
    Some(
        at.duration_since(std::time::SystemTime::now())
            .unwrap_or(Duration::ZERO),
    )
}

/// How long the server asked the client to wait before retrying: a
/// `google.rpc.RetryInfo` error detail, else a `retry-after` header (seconds
/// or an HTTP-date).
#[must_use]
pub fn server_retry_delay(status: &Status) -> Option<Duration> {
    retry_info_delay(status.details()).or_else(|| {
        status
            .metadata()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(parse_retry_after)
    })
}

#[cfg(test)]
pub(crate) fn encode_retry_info(delay: Duration) -> Vec<u8> {
    use prost::Message as _;
    let info = RetryInfo {
        retry_delay: Some(prost_types::Duration {
            seconds: i64::try_from(delay.as_secs()).unwrap_or(i64::MAX),
            nanos: i32::try_from(delay.subsec_nanos()).unwrap_or(0),
        }),
    };
    RpcStatus {
        code: Code::ResourceExhausted as i32,
        message: String::new(),
        details: vec![prost_types::Any {
            type_url: "type.googleapis.com/google.rpc.RetryInfo".into(),
            value: info.encode_to_vec(),
        }],
    }
    .encode_to_vec()
}

/// Builds a `ResourceExhausted` status carrying a `google.rpc.RetryInfo`
/// detail, the way the perms.io API reports rate limiting.
#[cfg(test)]
pub(crate) fn rate_limited_status(message: impl Into<String>, retry_after: Duration) -> Status {
    Status::with_details(
        Code::ResourceExhausted,
        message,
        encode_retry_info(retry_after).into(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_codes_to_variants() {
        assert!(matches!(
            Error::from(Status::unauthenticated("bad key")),
            Error::Auth { .. }
        ));
        assert!(matches!(
            Error::from(Status::invalid_argument("x")),
            Error::Validation {
                status: Some(_),
                ..
            }
        ));
        assert!(matches!(
            Error::from(Status::not_found("x")),
            Error::NotFound { .. }
        ));
        assert!(matches!(
            Error::from(Status::unavailable("x")),
            Error::Unavailable { .. }
        ));
        assert!(matches!(
            Error::from(Status::deadline_exceeded("x")),
            Error::Timeout { .. }
        ));
        let other = Error::from(Status::already_exists("dup"));
        assert!(matches!(other, Error::Other { .. }));
        assert_eq!(other.code(), Some(Code::AlreadyExists));
        let auth = Error::from(Status::unauthenticated("bad key"));
        assert_eq!(auth.message(), "bad key");
        assert_eq!(auth.to_string(), "permio: unauthenticated: bad key");
    }

    #[test]
    fn parses_missing_permissions() {
        let msg = "missing permissions: [{ResourceURI:/organisation/o/project/p/doc MissingPermissions:[permission.check other.x]} {ResourceURI:/b MissingPermissions:[]}]";
        let err = Error::from(Status::permission_denied(msg));
        let Error::Permission { missing, .. } = err else {
            panic!("wrong variant")
        };
        assert_eq!(
            missing,
            vec![
                MissingPermission {
                    resource: "/organisation/o/project/p/doc".into(),
                    permissions: vec!["permission.check".into(), "other.x".into()],
                },
                MissingPermission {
                    resource: "/b".into(),
                    permissions: vec![],
                },
            ]
        );
        assert!(parse_missing_permissions("nothing here").is_empty());
    }

    #[test]
    fn reads_retry_info_and_retry_after() {
        let s = rate_limited_status("slow down", Duration::from_millis(250));
        let Error::RateLimited { retry_after, .. } = Error::from(s) else {
            panic!("wrong variant")
        };
        assert_eq!(retry_after, Some(Duration::from_millis(250)));

        let mut s = Status::resource_exhausted("slow down");
        s.metadata_mut().insert("retry-after", "3".parse().unwrap());
        assert_eq!(server_retry_delay(&s), Some(Duration::from_secs(3)));
        assert_eq!(server_retry_delay(&Status::resource_exhausted("x")), None);
        assert_eq!(parse_retry_after("1.5"), None);
        assert_eq!(parse_retry_after(""), None);
        assert_eq!(parse_retry_after("soon"), None);
    }

    #[test]
    fn retry_after_http_date() {
        let in_a_minute = std::time::SystemTime::now() + Duration::from_secs(60);
        let header = httpdate::fmt_http_date(in_a_minute);
        let delay = parse_retry_after(&header).expect("HTTP-date parsed");
        // One-second resolution.
        assert!(delay > Duration::from_secs(57) && delay <= Duration::from_secs(60));
        assert_eq!(
            parse_retry_after("Sun, 06 Nov 1994 08:49:37 GMT"),
            Some(Duration::ZERO)
        );
        // The obsolete RFC 850 and asctime forms are accepted too.
        assert_eq!(
            parse_retry_after("Sunday, 06-Nov-94 08:49:37 GMT"),
            Some(Duration::ZERO)
        );

        let mut s = Status::resource_exhausted("slow down");
        s.metadata_mut()
            .insert("retry-after", header.parse().unwrap());
        let Error::RateLimited { retry_after, .. } = Error::from(s) else {
            panic!("wrong variant")
        };
        assert!(retry_after.is_some_and(|d| d > Duration::from_secs(57)));
    }

    #[test]
    fn denied_message() {
        let e = Error::denied(
            "alice",
            vec![MissingPermission {
                resource: "/docs/1".into(),
                permissions: vec!["docs.read".into()],
            }],
        );
        assert_eq!(
            e.to_string(),
            "principal \"alice\" is missing docs.read on /docs/1"
        );
        assert_eq!(e.code(), None);
    }
}
