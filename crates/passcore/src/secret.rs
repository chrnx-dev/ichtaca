//! A container for sensitive bytes that is zeroized on drop and never logged.

use zeroize::Zeroize;

/// Sensitive data (a decrypted entry or password). Zeroized on drop.
///
/// `Debug` is deliberately redacted so secrets never reach logs.
pub struct Secret {
    bytes: Vec<u8>,
}

impl Secret {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    /// The raw decrypted bytes.
    pub fn expose_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// The contents as UTF-8 (lossy is avoided; callers expect valid text).
    pub fn expose_str(&self) -> &str {
        std::str::from_utf8(&self.bytes).unwrap_or("")
    }

    /// The first line — the password, per the `pass` format.
    pub fn first_line(&self) -> &str {
        self.expose_str().lines().next().unwrap_or("")
    }
}

impl From<&str> for Secret {
    fn from(s: &str) -> Self {
        Self::new(s.as_bytes().to_vec())
    }
}

impl From<String> for Secret {
    fn from(s: String) -> Self {
        Self::new(s.into_bytes())
    }
}

impl Drop for Secret {
    fn drop(&mut self) {
        self.bytes.zeroize();
    }
}

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Secret(<redacted>)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_str_and_bytes() {
        let s = Secret::from("hunter2");
        assert_eq!(s.expose_str(), "hunter2");
        assert_eq!(s.expose_bytes(), b"hunter2");
    }

    #[test]
    fn first_line_returns_password_line_only() {
        let s = Secret::from("pw\nuser: bob\n");
        assert_eq!(s.first_line(), "pw");
    }

    #[test]
    fn debug_does_not_leak_contents() {
        let s = Secret::from("topsecret");
        assert!(!format!("{s:?}").contains("topsecret"));
    }
}
