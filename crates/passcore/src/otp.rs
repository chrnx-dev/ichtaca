//! TOTP computation from an `otpauth://totp/...` URI, per RFC 6238.
//!
//! The core entry point `code_at(uri, unix_secs)` is pure: the timestamp is a
//! parameter, so RFC 6238 test vectors are deterministic. `current(uri)` is the
//! thin wall-clock wrapper used by frontends.

use hmac::{Hmac, Mac};
use sha1::Sha1;
use sha2::{Sha256, Sha512};

use crate::error::{PassError, Result};

/// The supported HMAC hash algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Algorithm {
    Sha1,
    Sha256,
    Sha512,
}

/// A computed one-time code plus how long until it rolls over.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Otp {
    pub code: String,
    pub seconds_remaining: u64,
}

/// Parsed TOTP parameters from an `otpauth://` URI.
#[derive(Debug, Clone)]
struct TotpParams {
    secret: Vec<u8>,
    algorithm: Algorithm,
    digits: u32,
    period: u64,
}

fn parse_uri(uri: &str) -> Result<TotpParams> {
    let rest = uri
        .strip_prefix("otpauth://totp/")
        .ok_or_else(|| PassError::Parse("not an otpauth://totp/ URI".into()))?;

    let query = rest.split_once('?').map(|(_, q)| q).unwrap_or("");

    let mut secret_b32: Option<String> = None;
    let mut algorithm = Algorithm::Sha1;
    let mut digits: u32 = 6;
    let mut period: u64 = 30;

    for pair in query.split('&').filter(|p| !p.is_empty()) {
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        match k {
            "secret" => secret_b32 = Some(v.to_string()),
            "algorithm" => {
                algorithm = match v.to_ascii_uppercase().as_str() {
                    "SHA1" => Algorithm::Sha1,
                    "SHA256" => Algorithm::Sha256,
                    "SHA512" => Algorithm::Sha512,
                    other => {
                        return Err(PassError::Parse(format!("unknown algorithm: {other}")));
                    }
                }
            }
            "digits" => {
                digits = v
                    .parse()
                    .map_err(|_| PassError::Parse(format!("bad digits: {v}")))?;
            }
            "period" => {
                period = v
                    .parse()
                    .map_err(|_| PassError::Parse(format!("bad period: {v}")))?;
            }
            _ => {}
        }
    }

    let secret_b32 = secret_b32.ok_or_else(|| PassError::Parse("missing secret".into()))?;
    let secret = base32::decode(
        base32::Alphabet::Rfc4648 { padding: false },
        secret_b32.trim_end_matches('='),
    )
    .ok_or_else(|| PassError::Parse("secret is not valid base32".into()))?;

    if period == 0 {
        return Err(PassError::Parse("period must be > 0".into()));
    }

    Ok(TotpParams {
        secret,
        algorithm,
        digits,
        period,
    })
}

fn hmac_digest(algo: Algorithm, key: &[u8], counter: u64) -> Vec<u8> {
    let msg = counter.to_be_bytes();
    match algo {
        Algorithm::Sha1 => {
            let mut mac = <Hmac<Sha1>>::new_from_slice(key).expect("HMAC accepts any key length");
            mac.update(&msg);
            mac.finalize().into_bytes().to_vec()
        }
        Algorithm::Sha256 => {
            let mut mac = <Hmac<Sha256>>::new_from_slice(key).expect("HMAC accepts any key length");
            mac.update(&msg);
            mac.finalize().into_bytes().to_vec()
        }
        Algorithm::Sha512 => {
            let mut mac = <Hmac<Sha512>>::new_from_slice(key).expect("HMAC accepts any key length");
            mac.update(&msg);
            mac.finalize().into_bytes().to_vec()
        }
    }
}

/// RFC 4226 dynamic truncation -> `digits`-digit code.
fn truncate(digest: &[u8], digits: u32) -> String {
    let offset = (digest[digest.len() - 1] & 0x0f) as usize;
    let bin = ((u32::from(digest[offset]) & 0x7f) << 24)
        | (u32::from(digest[offset + 1]) << 16)
        | (u32::from(digest[offset + 2]) << 8)
        | u32::from(digest[offset + 3]);
    let modulo = 10u32.pow(digits);
    format!("{:0width$}", bin % modulo, width = digits as usize)
}

/// Compute the TOTP code for `uri` at the given Unix timestamp (seconds).
pub fn code_at(uri: &str, unix_secs: u64) -> Result<Otp> {
    let p = parse_uri(uri)?;
    let counter = unix_secs / p.period;
    let digest = hmac_digest(p.algorithm, &p.secret, counter);
    let code = truncate(&digest, p.digits);
    let seconds_remaining = p.period - (unix_secs % p.period);
    Ok(Otp {
        code,
        seconds_remaining,
    })
}

/// Compute the TOTP code for `uri` at the current wall-clock time.
pub fn current(uri: &str) -> Result<Otp> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| PassError::Parse(format!("system clock before epoch: {e}")))?
        .as_secs();
    code_at(uri, now)
}

#[cfg(test)]
mod tests {
    use super::*;

    // RFC 6238 Appendix B uses an ASCII seed repeated to key length. The SHA-1
    // seed is "12345678901234567890" (20 bytes). Base32 of those bytes:
    const SECRET_SHA1: &str = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";
    // SHA-256 seed is the 32-byte "12345678901234567890123456789012".
    const SECRET_SHA256: &str = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZA";
    // SHA-512 seed is the 64-byte "12345678901234567890" padded/repeated.
    // RFC 6238 Appendix B: the key is "1234567890" * 6 + "1234" = 64 bytes.
    // Base32 of those 64 bytes (no padding):
    const SECRET_SHA512: &str =
        "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZDGNA";

    fn uri(secret: &str, algo: &str) -> String {
        format!(
            "otpauth://totp/Example:alice?secret={secret}&issuer=Example&algorithm={algo}&digits=8&period=30"
        )
    }

    #[test]
    fn rfc6238_sha1_vectors() {
        let u = uri(SECRET_SHA1, "SHA1");
        assert_eq!(code_at(&u, 59).unwrap().code, "94287082");
        assert_eq!(code_at(&u, 1111111109).unwrap().code, "07081804");
        assert_eq!(code_at(&u, 1111111111).unwrap().code, "14050471");
        assert_eq!(code_at(&u, 1234567890).unwrap().code, "89005924");
        assert_eq!(code_at(&u, 2000000000).unwrap().code, "69279037");
        assert_eq!(code_at(&u, 20000000000).unwrap().code, "65353130");
    }

    #[test]
    fn rfc6238_sha256_vector() {
        let u = uri(SECRET_SHA256, "SHA256");
        assert_eq!(code_at(&u, 59).unwrap().code, "46119246");
    }

    #[test]
    fn rfc6238_sha512_vector() {
        let u = uri(SECRET_SHA512, "SHA512");
        assert_eq!(code_at(&u, 59).unwrap().code, "90693936");
    }

    #[test]
    fn seconds_remaining_within_period() {
        let u = uri(SECRET_SHA1, "SHA1");
        // period=30: at t=59 we are 29s into the 2nd window -> 1s remaining.
        assert_eq!(code_at(&u, 59).unwrap().seconds_remaining, 1);
        // at t=30 we are at the start of a window -> 30s remaining.
        assert_eq!(code_at(&u, 30).unwrap().seconds_remaining, 30);
    }

    #[test]
    fn defaults_apply_when_params_absent() {
        // No algorithm/digits/period -> SHA1, 6 digits, 30s.
        let u = format!("otpauth://totp/x?secret={SECRET_SHA1}");
        let otp = code_at(&u, 59).unwrap();
        assert_eq!(otp.code.len(), 6);
        // 6-digit truncation of the SHA1 t=59 value.
        assert_eq!(otp.code, "287082");
    }

    #[test]
    fn rejects_non_otpauth_uri() {
        assert!(code_at("https://example.com", 0).is_err());
    }

    #[test]
    fn rejects_missing_secret() {
        assert!(code_at("otpauth://totp/x?digits=6", 0).is_err());
    }
}
