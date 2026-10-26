//! Local API key format checks.
//!
//! An opaque key is `"perms_"` followed by 32 random base62 characters and 6
//! base62 characters encoding the CRC32 (IEEE) of those 32, 44 characters in
//! total. Checking it locally catches a truncated or mistyped key before any
//! request is made. It does not tell you whether the key exists or is still
//! valid; only the API can.

use crate::error::Error;

const PREFIX: &str = "perms_";
const RANDOM_LENGTH: usize = 32;
const CHECKSUM_LENGTH: usize = 6;
/// Length of an opaque `perms_` API key.
pub const API_KEY_LENGTH: usize = PREFIX.len() + RANDOM_LENGTH + CHECKSUM_LENGTH;
const ALPHABET: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// The kind of API key, as returned by [`validate_api_key`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ApiKeyKind {
    /// An opaque `perms_...` key from the dashboard.
    Opaque,
    /// A legacy JWT key.
    Jwt,
}

const fn crc_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut n = 0;
    while n < 256 {
        let mut c = n as u32;
        let mut k = 0;
        while k < 8 {
            c = if c & 1 != 0 {
                0xedb8_8320 ^ (c >> 1)
            } else {
                c >> 1
            };
            k += 1;
        }
        table[n] = c;
        n += 1;
    }
    table
}

static CRC_TABLE: [u32; 256] = crc_table();

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &b in bytes {
        crc = CRC_TABLE[((crc ^ u32::from(b)) & 0xff) as usize] ^ (crc >> 8);
    }
    crc ^ 0xffff_ffff
}

fn checksum(body: &[u8]) -> [u8; CHECKSUM_LENGTH] {
    let mut value = crc32(body);
    let mut out = [0u8; CHECKSUM_LENGTH];
    for slot in out.iter_mut().rev() {
        *slot = ALPHABET[(value % 62) as usize];
        value /= 62;
    }
    out
}

fn is_jwt(key: &str) -> bool {
    let parts: Vec<&str> = key.split('.').collect();
    parts.len() == 3
        && parts.iter().all(|p| {
            !p.is_empty()
                && p.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        })
}

/// Checks that `key` looks like a perms.io API key and returns its kind.
///
/// Opaque `perms_` keys are checked for length, alphabet and CRC32 checksum.
/// Legacy JWT keys are only checked for the three-part shape.
///
/// # Errors
///
/// [`Error::Config`] if the key is empty or malformed.
pub fn validate_api_key(key: &str) -> Result<ApiKeyKind, Error> {
    if key.is_empty() {
        return Err(Error::config("permio: api key is required"));
    }
    if let Some(rest) = key.strip_prefix(PREFIX) {
        if key.len() != API_KEY_LENGTH {
            return Err(Error::config(format!(
                "permio: malformed API key (expected {API_KEY_LENGTH} characters starting with \"{PREFIX}\", got {})",
                key.chars().count()
            )));
        }
        let rest = rest.as_bytes();
        if !rest.iter().all(|b| b.is_ascii_alphanumeric()) {
            return Err(Error::config(
                "permio: malformed API key (characters after perms_ must be 0-9, A-Z or a-z)",
            ));
        }
        if checksum(&rest[..RANDOM_LENGTH]) != rest[RANDOM_LENGTH..] {
            return Err(Error::config(
                "permio: API key checksum mismatch; check it was copied correctly",
            ));
        }
        return Ok(ApiKeyKind::Opaque);
    }
    if is_jwt(key) {
        return Ok(ApiKeyKind::Jwt);
    }
    Err(Error::config(
        "permio: malformed API key (expected a \"perms_\" key from the dashboard, or a legacy JWT key)",
    ))
}

/// Returns true if `key` passes [`validate_api_key`].
#[must_use]
pub fn is_valid_api_key(key: &str) -> bool {
    validate_api_key(key).is_ok()
}

/// Builds a well-formed opaque key from 32 base62 characters (test helper).
#[cfg(test)]
pub(crate) fn make_key(body: &str) -> String {
    assert_eq!(body.len(), RANDOM_LENGTH);
    let sum = checksum(body.as_bytes());
    format!("{PREFIX}{body}{}", std::str::from_utf8(&sum).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Same fixture as permio-js test/apikey.test.ts.
    const VALID: &str = "perms_0123456789ABCDEFGHIJKLMNOPQRSTUV1ggZdL";

    #[test]
    fn accepts_a_valid_opaque_key() {
        assert_eq!(validate_api_key(VALID).unwrap(), ApiKeyKind::Opaque);
        assert_eq!(make_key("0123456789ABCDEFGHIJKLMNOPQRSTUV"), VALID);
        assert!(is_valid_api_key(VALID));
    }

    #[test]
    fn rejects_bad_checksum_length_and_alphabet() {
        let mut bad = VALID.to_string();
        bad.replace_range(43..44, "M");
        assert!(matches!(validate_api_key(&bad), Err(Error::Config { .. })));
        assert!(validate_api_key(&VALID[..43]).is_err());
        let mut bad = VALID.to_string();
        bad.replace_range(10..11, "-");
        let err = validate_api_key(&bad).unwrap_err();
        assert!(err.to_string().contains("0-9, A-Z or a-z"), "{err}");
        assert!(validate_api_key("").is_err());
        assert!(validate_api_key("not a key").is_err());
    }

    #[test]
    fn accepts_jwt_shape() {
        assert_eq!(validate_api_key("aaa.bbb.c-_c").unwrap(), ApiKeyKind::Jwt);
        assert!(validate_api_key("aaa..ccc").is_err());
        assert!(validate_api_key("aaa.bbb").is_err());
    }
}
