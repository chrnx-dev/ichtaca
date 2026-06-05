//! User configuration: TOML with sane defaults. Loading never hard-fails on a
//! missing file; a malformed file is reported so the frontend can warn.

use std::path::PathBuf;

use serde::Deserialize;

use crate::error::{PassError, Result};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Override for `$PASSWORD_STORE_DIR`.
    pub store_dir: Option<PathBuf>,
    pub clipboard: ClipboardConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ClipboardConfig {
    /// Seconds before the clipboard is cleared. Default, not a constant.
    pub clear_after: u64,
}

impl Default for ClipboardConfig {
    fn default() -> Self {
        Self { clear_after: 45 }
    }
}

impl Config {
    /// Parse from a TOML string, filling unspecified fields with defaults.
    pub fn from_toml_str(s: &str) -> Result<Self> {
        toml::from_str(s).map_err(|e| PassError::Config(e.to_string()))
    }

    /// The default config file path: `$XDG_CONFIG_HOME/pass-client/config.toml`.
    pub fn default_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("pass-client").join("config.toml"))
    }

    /// Load from the default path. Returns `Config::default()` if the file is
    /// absent; returns an error only if the file exists but is malformed.
    pub fn load() -> Result<Self> {
        match Self::default_path() {
            Some(p) if p.exists() => {
                let raw = std::fs::read_to_string(&p)?;
                Self::from_toml_str(&raw)
            }
            _ => Ok(Self::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let c = Config::default();
        assert_eq!(c.clipboard.clear_after, 45);
        assert!(c.store_dir.is_none());
    }

    #[test]
    fn parses_partial_toml_and_fills_defaults() {
        let toml = r#"
            [clipboard]
            clear_after = 90
        "#;
        let c = Config::from_toml_str(toml).unwrap();
        assert_eq!(c.clipboard.clear_after, 90);
        // unspecified section keeps its default
        assert!(c.store_dir.is_none());
    }

    #[test]
    fn invalid_toml_is_an_error() {
        assert!(Config::from_toml_str("clipboard = [not valid").is_err());
    }
}
