//! TOTP computation from an `otpauth://totp/...` URI, per RFC 6238.
//!
//! The core entry point `code_at(uri, unix_secs)` is pure: the timestamp is a
//! parameter, so RFC 6238 test vectors are deterministic. `current(uri)` is the
//! thin wall-clock wrapper used by frontends.

use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use sha2::{Sha256, Sha512};

use crate::error::{PassError, Result};

/// The supported HMAC hash algorithms.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Algorithm {
    #[default]
    Sha1,
    Sha256,
    Sha512,
}

impl Algorithm {
    /// The spelling used in an `otpauth://` URI.
    pub fn as_str(self) -> &'static str {
        match self {
            Algorithm::Sha1 => "SHA1",
            Algorithm::Sha256 => "SHA256",
            Algorithm::Sha512 => "SHA512",
        }
    }

    /// Parse the `algorithm=` parameter, case-insensitively.
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_ascii_uppercase().as_str() {
            "SHA1" => Ok(Algorithm::Sha1),
            "SHA256" => Ok(Algorithm::Sha256),
            "SHA512" => Ok(Algorithm::Sha512),
            other => Err(PassError::Parse(format!("unknown algorithm: {other}"))),
        }
    }
}

/// A computed one-time code plus how long until it rolls over.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Otp {
    pub code: String,
    pub seconds_remaining: u64,
}

/// Default number of digits in a TOTP code.
pub const DEFAULT_DIGITS: u32 = 6;
/// Default TOTP time step, in seconds.
pub const DEFAULT_PERIOD: u64 = 30;

/// The editable parts of a TOTP configuration.
///
/// This is what a user actually fills in or pastes, as opposed to [`Otp`],
/// which is a computed code. `secret` stays base32 (the form the user sees and
/// types) and is decoded only when computing a code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OtpConfig {
    /// Service name, e.g. `GitHub`. May be empty.
    pub issuer: String,
    /// Account name, e.g. `alice@example.com`. May be empty.
    pub account: String,
    /// Shared secret, base32-encoded.
    pub secret: String,
    pub algorithm: Algorithm,
    pub digits: u32,
    pub period: u64,
}

impl Default for OtpConfig {
    fn default() -> Self {
        Self {
            issuer: String::new(),
            account: String::new(),
            secret: String::new(),
            algorithm: Algorithm::default(),
            digits: DEFAULT_DIGITS,
            period: DEFAULT_PERIOD,
        }
    }
}

impl OtpConfig {
    /// Parse a full `otpauth://totp/...` URI, label included.
    ///
    /// Unknown query parameters are ignored; the label supplies issuer and
    /// account, and an explicit `issuer=` parameter wins over the label.
    pub fn parse(uri: &str) -> Result<Self> {
        let rest = strip_totp_prefix(uri)
            .ok_or_else(|| PassError::Parse("not an otpauth://totp/ URI".into()))?;

        let (label, query) = match rest.split_once('?') {
            Some((l, q)) => (l, q),
            None => (rest, ""),
        };

        // Label is `Issuer:Account`, `Issuer%3AAccount`, or just `Account`.
        let label = percent_decode(label.trim_start_matches('/'));
        let (mut issuer, account) = match label.split_once(':') {
            Some((i, a)) => (i.trim().to_string(), a.trim().to_string()),
            None => (String::new(), label.trim().to_string()),
        };

        let mut cfg = OtpConfig {
            account,
            ..Default::default()
        };

        for pair in query.split('&').filter(|p| !p.is_empty()) {
            let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
            match k.to_ascii_lowercase().as_str() {
                "secret" => cfg.secret = percent_decode(v),
                "issuer" => issuer = percent_decode(v),
                "algorithm" => cfg.algorithm = Algorithm::parse(&percent_decode(v))?,
                "digits" => {
                    cfg.digits = v
                        .parse()
                        .map_err(|_| PassError::Parse(format!("bad digits: {v}")))?
                }
                "period" => {
                    cfg.period = v
                        .parse()
                        .map_err(|_| PassError::Parse(format!("bad period: {v}")))?
                }
                _ => {}
            }
        }
        cfg.issuer = issuer;
        cfg.validate()?;
        Ok(cfg)
    }

    /// Render back to an `otpauth://` URI.
    ///
    /// Default algorithm/digits/period are omitted, so a config parsed from a
    /// minimal URI renders back to a minimal URI.
    pub fn to_uri(&self) -> String {
        let label = if self.issuer.is_empty() {
            encode_component(&self.account)
        } else {
            format!(
                "{}:{}",
                encode_component(&self.issuer),
                encode_component(&self.account)
            )
        };
        let mut uri = format!(
            "otpauth://totp/{label}?secret={}",
            encode_component(&self.secret)
        );
        if !self.issuer.is_empty() {
            uri.push_str(&format!("&issuer={}", encode_component(&self.issuer)));
        }
        if self.algorithm != Algorithm::default() {
            uri.push_str(&format!("&algorithm={}", self.algorithm.as_str()));
        }
        if self.digits != DEFAULT_DIGITS {
            uri.push_str(&format!("&digits={}", self.digits));
        }
        if self.period != DEFAULT_PERIOD {
            uri.push_str(&format!("&period={}", self.period));
        }
        uri
    }

    /// Reject anything that would produce a broken or uncomputable code.
    pub fn validate(&self) -> Result<()> {
        self.secret_bytes()?;
        if self.digits == 0 || self.digits > 9 {
            return Err(PassError::Parse(format!(
                "otpauth digits must be 1-9, got {}",
                self.digits
            )));
        }
        if self.period == 0 {
            return Err(PassError::Parse("period must be > 0".into()));
        }
        Ok(())
    }

    /// Decode the base32 secret, tolerating padding, spaces, and lowercase —
    /// all three appear in secrets copied from real websites.
    pub fn secret_bytes(&self) -> Result<Vec<u8>> {
        let cleaned = normalize_secret(&self.secret);
        if cleaned.is_empty() {
            return Err(PassError::Parse("missing secret".into()));
        }
        base32::decode(base32::Alphabet::Rfc4648 { padding: false }, &cleaned)
            .filter(|b| !b.is_empty())
            .ok_or_else(|| PassError::Parse("secret is not valid base32".into()))
    }

    /// One-line description for a UI to confirm what was understood, e.g.
    /// `GitHub (alice) · 6 digits · 30s · SHA1`.
    pub fn summary(&self) -> String {
        let who = match (self.issuer.as_str(), self.account.as_str()) {
            ("", "") => "TOTP".to_string(),
            ("", a) => a.to_string(),
            (i, "") => i.to_string(),
            (i, a) => format!("{i} ({a})"),
        };
        format!(
            "{who} · {} digits · {}s · {}",
            self.digits,
            self.period,
            self.algorithm.as_str()
        )
    }
}

/// Turn whatever the user typed into a valid `otpauth://` URI.
///
/// Accepts either a full URI (validated and returned unchanged, so unknown
/// parameters and the user's own formatting survive an edit) or a bare base32
/// secret, which is wrapped using `issuer`/`account` as the label. This is the
/// whole point of the structured editor: nobody should have to hand-write an
/// `otpauth://` URI, and nothing invalid should save silently.
///
/// Returns `Ok(None)` for blank input — an entry with no OTP is normal.
pub fn normalize_input(input: &str, issuer: &str, account: &str) -> Result<Option<String>> {
    let input = input.trim();
    if input.is_empty() {
        return Ok(None);
    }
    if strip_totp_prefix(input).is_some() {
        // Validate, then keep the user's own text.
        OtpConfig::parse(input)?;
        return Ok(Some(input.to_string()));
    }
    if input.to_ascii_lowercase().starts_with("otpauth://") {
        return Err(PassError::Parse(
            "only otpauth://totp/ URIs are supported (HOTP is not)".into(),
        ));
    }
    let cfg = OtpConfig {
        issuer: issuer.trim().to_string(),
        account: account.trim().to_string(),
        secret: normalize_secret(input),
        ..Default::default()
    };
    cfg.validate()?;
    Ok(Some(cfg.to_uri()))
}

/// Default label for a bare secret: issuer from the entry path's last segment,
/// account from whichever username-ish field the entry has.
///
/// Shared so the TUI and the desktop app agree on what a wrapped secret is
/// called — the label is what other TOTP apps display.
pub fn label_from_entry(path: &str, fields: &[(String, String)]) -> (String, String) {
    let issuer = path.trim().rsplit('/').next().unwrap_or("").to_string();
    let account = fields
        .iter()
        .find(|(k, _)| matches!(k.to_lowercase().as_str(), "user" | "username" | "login"))
        .map(|(_, v)| v.trim().to_string())
        .unwrap_or_default();
    (issuer, account)
}

/// The bare message from an OTP validation failure.
///
/// `PassError::Parse` renders as "could not parse entry: …", which reads wrong
/// next to an OTP input — the entry parsed fine, the OTP did not.
pub fn error_message(e: &PassError) -> String {
    match e {
        PassError::Parse(m) => m.clone(),
        other => other.to_string(),
    }
}

/// Case-insensitive `otpauth://totp/` prefix strip.
fn strip_totp_prefix(uri: &str) -> Option<&str> {
    const PREFIX: &str = "otpauth://totp/";
    let head = uri.get(..PREFIX.len())?;
    head.eq_ignore_ascii_case(PREFIX)
        .then(|| &uri[PREFIX.len()..])
}

/// Strip the separators people paste along with a secret, and upper-case it.
fn normalize_secret(raw: &str) -> String {
    raw.chars()
        .filter(|c| !c.is_whitespace() && *c != '-' && *c != '=')
        .collect::<String>()
        .to_ascii_uppercase()
}

/// Percent-encode one URI component. Hand-rolled: encoding a label and a
/// base32 secret does not justify a dependency.
fn encode_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Percent-decode, also turning `+` into a space. Invalid escapes are left
/// as-is rather than dropped — a mangled label should not lose characters.
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => match u8::from_str_radix(&s[i + 1..i + 3], 16) {
                Ok(b) => {
                    out.push(b);
                    i += 3;
                }
                Err(_) => {
                    out.push(bytes[i]);
                    i += 1;
                }
            },
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
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
    let p = OtpConfig::parse(uri)?;
    let secret = p.secret_bytes()?;
    let counter = unix_secs / p.period;
    let digest = hmac_digest(p.algorithm, &secret, counter);
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

    #[test]
    fn rejects_out_of_range_digits() {
        let uri = "otpauth://totp/x?secret=GEZDGNBVGY3TQOJQ&digits=10";
        assert!(code_at(uri, 59).is_err());
    }
}

// ── OtpConfig / structured editor ────────────────────────────────────────

#[cfg(test)]
mod config_tests {
    use super::*;

    const SECRET: &str = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";

    #[test]
    fn parses_label_issuer_and_account() {
        let cfg = OtpConfig::parse(&format!(
            "otpauth://totp/GitHub:alice%40example.com?secret={SECRET}&issuer=GitHub"
        ))
        .unwrap();
        assert_eq!(cfg.issuer, "GitHub");
        assert_eq!(cfg.account, "alice@example.com");
        assert_eq!(cfg.secret, SECRET);
        // Unspecified parameters fall back to the RFC defaults.
        assert_eq!(cfg.digits, DEFAULT_DIGITS);
        assert_eq!(cfg.period, DEFAULT_PERIOD);
        assert_eq!(cfg.algorithm, Algorithm::Sha1);
    }

    #[test]
    fn label_without_issuer_is_account_only() {
        let cfg = OtpConfig::parse(&format!("otpauth://totp/alice?secret={SECRET}")).unwrap();
        assert_eq!(cfg.issuer, "");
        assert_eq!(cfg.account, "alice");
    }

    #[test]
    fn round_trips_through_to_uri() {
        let cfg = OtpConfig {
            issuer: "Big Corp".into(),
            account: "alice@example.com".into(),
            secret: SECRET.into(),
            algorithm: Algorithm::Sha512,
            digits: 8,
            period: 60,
        };
        let back = OtpConfig::parse(&cfg.to_uri()).unwrap();
        assert_eq!(back, cfg, "to_uri -> parse must be lossless");
    }

    #[test]
    fn to_uri_omits_default_parameters() {
        let cfg = OtpConfig {
            issuer: "GitHub".into(),
            account: "alice".into(),
            secret: SECRET.into(),
            ..Default::default()
        };
        let uri = cfg.to_uri();
        assert!(
            !uri.contains("algorithm="),
            "default algorithm omitted: {uri}"
        );
        assert!(!uri.contains("digits="), "default digits omitted: {uri}");
        assert!(!uri.contains("period="), "default period omitted: {uri}");
        // Still round-trips.
        assert_eq!(OtpConfig::parse(&uri).unwrap(), cfg);
    }

    #[test]
    fn rejects_invalid_configs() {
        let bad_secret = OtpConfig {
            secret: "not base32!!".into(),
            ..Default::default()
        };
        assert!(
            bad_secret.validate().is_err(),
            "invalid base32 must not save"
        );

        let empty = OtpConfig::default();
        assert!(empty.validate().is_err(), "an empty secret must not save");

        let zero_period = OtpConfig {
            secret: SECRET.into(),
            period: 0,
            ..Default::default()
        };
        assert!(zero_period.validate().is_err());

        let too_many_digits = OtpConfig {
            secret: SECRET.into(),
            digits: 12,
            ..Default::default()
        };
        assert!(too_many_digits.validate().is_err());
    }

    #[test]
    fn bare_secret_becomes_a_uri_with_the_entry_label() {
        let uri = normalize_input("gezd gnbv gy3t qojq gezd gnbv gy3t qojq", "GitHub", "alice")
            .unwrap()
            .expect("a secret produces a URI");
        let cfg = OtpConfig::parse(&uri).unwrap();
        assert_eq!(cfg.secret, SECRET, "spaces stripped and upper-cased");
        assert_eq!(cfg.issuer, "GitHub");
        assert_eq!(cfg.account, "alice");
        // And it actually computes.
        assert_eq!(code_at(&uri, 59).unwrap().code.len(), 6);
    }

    #[test]
    fn pasted_uri_is_validated_but_kept_verbatim() {
        let uri = format!("otpauth://totp/GitHub:alice?secret={SECRET}&image=https://x/y.png");
        let out = normalize_input(&uri, "Other", "other").unwrap().unwrap();
        assert_eq!(out, uri, "unknown parameters must survive an edit");
    }

    #[test]
    fn blank_input_means_no_otp() {
        assert_eq!(normalize_input("   ", "GitHub", "alice").unwrap(), None);
    }

    #[test]
    fn invalid_input_is_an_error_not_a_silent_save() {
        assert!(normalize_input("nope!!!", "GitHub", "alice").is_err());
        assert!(
            normalize_input("otpauth://hotp/GitHub:alice?secret=X", "", "").is_err(),
            "HOTP is not TOTP and must be rejected, not stored"
        );
        assert!(normalize_input(
            &format!("otpauth://totp/x?secret={SECRET}&digits=99"),
            "",
            ""
        )
        .is_err());
    }

    #[test]
    fn entry_label_prefers_the_username_field() {
        let fields = vec![
            ("url".to_string(), "https://github.com".to_string()),
            ("Login".to_string(), "alice".to_string()),
        ];
        let (issuer, account) = label_from_entry("web/github.com", &fields);
        assert_eq!(issuer, "github.com");
        assert_eq!(account, "alice", "matched case-insensitively");

        let (issuer, account) = label_from_entry("solo", &[]);
        assert_eq!(issuer, "solo");
        assert_eq!(account, "", "no username field is fine");
    }

    #[test]
    fn summary_describes_the_config() {
        let cfg = OtpConfig::parse(&format!(
            "otpauth://totp/GitHub:alice?secret={SECRET}&digits=8"
        ))
        .unwrap();
        let s = cfg.summary();
        assert!(s.contains("GitHub"), "{s}");
        assert!(s.contains("alice"), "{s}");
        assert!(s.contains("8 digits"), "{s}");
        assert!(s.contains("30s"), "{s}");
    }
}
