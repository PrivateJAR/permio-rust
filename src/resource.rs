//! Building and checking resource URIs.

use std::fmt::{Display, Write as _};

use crate::error::Error;

/// The API rejects resource URIs longer than this many bytes.
pub const MAX_RESOURCE_URI_LENGTH: usize = 2048;

/// Percent-encodes one resource URI segment.
///
/// Everything except ASCII letters, digits and `- _ . ~` is encoded (as
/// UTF-8), so IDs containing `/`, spaces or characters the API rejects in
/// segments (`( ) *`) stay a single valid segment. This matches permio-js
/// `encodeSegment`.
///
/// # Errors
///
/// [`Error::Validation`] if the segment is empty.
pub fn encode_segment(segment: impl Display) -> Result<String, Error> {
    let text = segment.to_string();
    if text.is_empty() {
        return Err(Error::validation(
            "permio: resource URI segments must not be empty",
        ));
    }
    let mut out = String::with_capacity(text.len());
    for b in text.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(char::from(b));
        } else {
            let _ = write!(out, "%{b:02X}");
        }
    }
    Ok(out)
}

/// Builds a resource URI from raw segments, encoding each one with
/// [`encode_segment`].
///
/// ```
/// let uri = permio::resource(["projects", "p1", "docs", "a/b"]).unwrap();
/// assert_eq!(uri, "/projects/p1/docs/a%2Fb");
/// ```
///
/// Build URIs the same way when granting and when checking, or they will not
/// match. The [`resource!`](crate::resource!) macro accepts mixed types.
///
/// # Errors
///
/// [`Error::Validation`] if there are no segments or one is empty.
pub fn resource<I>(segments: I) -> Result<String, Error>
where
    I: IntoIterator,
    I::Item: Display,
{
    let mut out = String::new();
    for segment in segments {
        out.push('/');
        out.push_str(&encode_segment(segment)?);
    }
    if out.is_empty() {
        return Err(Error::validation(
            "permio: resource() needs at least one segment",
        ));
    }
    Ok(out)
}

/// Builds a resource URI from segments of any [`Display`] types.
///
/// ```
/// let doc_id = 42;
/// let uri = permio::resource!("docs", doc_id).unwrap();
/// assert_eq!(uri, "/docs/42");
/// ```
#[macro_export]
macro_rules! resource {
    ($($segment:expr),+ $(,)?) => {
        $crate::resource([$(::std::string::ToString::to_string(&$segment)),+])
    };
}

/// Checks that `uri` has the basic shape the API accepts: a leading `/`, no
/// empty segments (so no `//` or trailing `/`), no whitespace or control
/// characters, at most 2048 bytes. The API applies further rules (for example
/// no regex metacharacters such as `( ) [ ] * + ?` in segments) and answers
/// `InvalidArgument` when they are broken.
///
/// # Errors
///
/// [`Error::Validation`] describing the problem.
pub fn validate_resource_uri(uri: &str) -> Result<(), Error> {
    let fail = |why: &str| {
        Err(Error::validation(format!(
            "permio: invalid resource URI {uri:?}: {why}"
        )))
    };
    if uri.is_empty() {
        return fail("it is empty");
    }
    if !uri.starts_with('/') {
        return fail("it must start with \"/\" (use resource() to build URIs)");
    }
    if uri.len() > MAX_RESOURCE_URI_LENGTH {
        return fail(&format!(
            "it is longer than {MAX_RESOURCE_URI_LENGTH} bytes"
        ));
    }
    if uri.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return fail("it contains whitespace or control characters");
    }
    if uri[1..].split('/').any(str::is_empty) {
        return fail("it contains an empty segment");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_like_permio_js() {
        assert_eq!(encode_segment("a b/c").unwrap(), "a%20b%2Fc");
        assert_eq!(encode_segment("!'()*").unwrap(), "%21%27%28%29%2A");
        assert_eq!(encode_segment("é").unwrap(), "%C3%A9");
        assert_eq!(encode_segment("A-z_0.9~").unwrap(), "A-z_0.9~");
        assert!(encode_segment("").is_err());
        assert_eq!(resource(["docs", "1"]).unwrap(), "/docs/1");
        assert_eq!(crate::resource!("docs", 7, "x y").unwrap(), "/docs/7/x%20y");
        assert!(resource(Vec::<String>::new()).is_err());
    }

    #[test]
    fn validates_uris() {
        assert!(validate_resource_uri("/docs/1").is_ok());
        assert!(validate_resource_uri("/").is_err());
        assert!(validate_resource_uri("docs").is_err());
        assert!(validate_resource_uri("/a//b").is_err());
        assert!(validate_resource_uri("/a/").is_err());
        assert!(validate_resource_uri("/a b").is_err());
        assert!(validate_resource_uri("/a\u{7}").is_err());
        assert!(validate_resource_uri(&format!("/{}", "a".repeat(2048))).is_err());
    }
}
