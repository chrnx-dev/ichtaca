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
    /// User-defined entry templates, overriding built-ins by name.
    pub templates: Vec<TemplateConfig>,
    pub keybindings: KeybindingsConfig,
    pub ui: UiConfig,
    pub generator: GeneratorConfig,
}

/// A config-defined entry template (suggested keys for new entries).
#[derive(Debug, Clone, Deserialize)]
pub struct TemplateConfig {
    pub name: String,
    #[serde(default)]
    pub fields: Vec<String>,
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

/// Vim-style keybindings for the TUI. Each value is the key character.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct KeybindingsConfig {
    pub down: String,
    pub up: String,
    pub expand: String,
    pub collapse: String,
    pub top: String,
    pub bottom: String,
    pub search: String,
    pub command: String,
    pub copy: String,
    pub reveal: String,
    pub quit: String,
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        Self {
            down: "j".into(),
            up: "k".into(),
            expand: "l".into(),
            collapse: "h".into(),
            top: "g".into(),
            bottom: "G".into(),
            search: "/".into(),
            command: ":".into(),
            copy: "c".into(),
            reveal: "s".into(),
            quit: "q".into(),
        }
    }
}

/// UI preferences for the TUI.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    /// Whether the password is revealed by default in the detail panel.
    pub reveal_default: bool,
}

/// Password-generator preferences (length + character set).
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct GeneratorConfig {
    /// Number of characters in a generated password.
    pub length: usize,
    /// Whether to include punctuation/symbol characters.
    pub symbols: bool,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            length: 20,
            symbols: true,
        }
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

    #[test]
    fn keybindings_have_vim_defaults() {
        let c = Config::default();
        assert_eq!(c.keybindings.down, "j");
        assert_eq!(c.keybindings.up, "k");
        assert_eq!(c.keybindings.expand, "l");
        assert_eq!(c.keybindings.collapse, "h");
        assert_eq!(c.keybindings.search, "/");
        assert_eq!(c.keybindings.copy, "c");
        assert_eq!(c.keybindings.reveal, "s");
        assert_eq!(c.keybindings.quit, "q");
    }

    #[test]
    fn keybindings_are_overridable_from_toml() {
        let toml = r#"
            [keybindings]
            down = "n"
            up = "e"
        "#;
        let c = Config::from_toml_str(toml).unwrap();
        assert_eq!(c.keybindings.down, "n");
        assert_eq!(c.keybindings.up, "e");
        // unspecified key keeps its default
        assert_eq!(c.keybindings.quit, "q");
    }

    #[test]
    fn ui_defaults_hide_password() {
        let c = Config::default();
        assert!(!c.ui.reveal_default);
    }

    #[test]
    fn generator_defaults_length_20_symbols_true() {
        let c = Config::default();
        assert_eq!(c.generator.length, 20);
        assert!(c.generator.symbols);
    }

    #[test]
    fn generator_overridable_from_toml() {
        let toml = r#"
            [generator]
            length = 32
            symbols = false
        "#;
        let c = Config::from_toml_str(toml).unwrap();
        assert_eq!(c.generator.length, 32);
        assert!(!c.generator.symbols);
    }

    #[test]
    fn generator_partial_toml_keeps_other_default() {
        // Only length given → symbols keeps its default (true).
        let toml = r#"
            [generator]
            length = 64
        "#;
        let c = Config::from_toml_str(toml).unwrap();
        assert_eq!(c.generator.length, 64);
        assert!(c.generator.symbols);
    }
}
