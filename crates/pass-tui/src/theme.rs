//! Ichtaca "Obsidiana & Oro" colour theme.
//!
//! All palette values are `tuirealm::props::Color::Rgb(r, g, b)` so they
//! render correctly on 24-bit terminals.  Convenience [`Style`] helpers are
//! provided for the most common semantic roles; import the module and call
//! e.g. `theme::title()` instead of constructing styles by hand.
//!
//! Hex values come from `specs/ichtaca-brand.md`.
//!
//! # Nerd Font icons
//!
//! This module also exports icon constants (`icons::*`) for Nerd Font glyphs
//! used across the UI.  A **Nerd Font** must be installed and configured as
//! the terminal font for these glyphs to render correctly.  Any font from
//! <https://www.nerdfonts.com/> works — the recommended choice is
//! **JetBrainsMono Nerd Font**.

// Phase 2-3 will use all of these; suppress premature dead-code lints.
#![allow(dead_code)]

use tuirealm::props::{Color, Style, TextModifiers};

// ─── Palette ────────────────────────────────────────────────────────────────

/// Deepest background — obsidian.
pub const BG: Color = Color::Rgb(21, 19, 26);

/// Panel / surface background.
pub const SURFACE: Color = Color::Rgb(30, 27, 38);

/// Background of the selected row in lists.
pub const SURFACE_SEL: Color = Color::Rgb(42, 37, 51);

/// Primary text — warm cream.
pub const TEXT: Color = Color::Rgb(232, 226, 208);

/// Muted tone for borders, hints, and field labels.
pub const MUTED: Color = Color::Rgb(107, 100, 120);

/// Brighter muted — readable hint labels.
pub const MUTED_BRIGHT: Color = Color::Rgb(154, 146, 168);

/// Gold — primary accent: selection highlight, titles, brand.
pub const GOLD: Color = Color::Rgb(224, 164, 54);

/// Bright gold — revealed / emphasised values.
pub const GOLD_BRIGHT: Color = Color::Rgb(242, 198, 109);

/// Turquoise — secondary accent: OTP, highlighted values.
pub const TURQUOISE: Color = Color::Rgb(47, 182, 168);

/// Bright turquoise — active OTP code.
pub const TURQUOISE_BRIGHT: Color = Color::Rgb(70, 208, 192);

/// Jade — success indicators, tags.
pub const JADE: Color = Color::Rgb(63, 166, 106);

/// Cochineal — errors, delete actions, danger prompts.
pub const COCHINEAL: Color = Color::Rgb(200, 68, 59);

// ─── Style helpers ──────────────────────────────────────────────────────────

/// Selected row: gold fg + surface_sel bg + bold.
pub fn selection() -> Style {
    Style::default()
        .fg(GOLD)
        .bg(SURFACE_SEL)
        .add_modifier(TextModifiers::BOLD)
}

/// Block/panel title: gold + bold.
pub fn title() -> Style {
    Style::default().fg(GOLD).add_modifier(TextModifiers::BOLD)
}

/// Muted labels, border decorations, field keys.
pub fn key() -> Style {
    Style::default().fg(MUTED)
}

/// Active OTP code: bright turquoise.
pub fn otp() -> Style {
    Style::default().fg(TURQUOISE_BRIGHT)
}

/// OTP countdown seconds: turquoise.
pub fn otp_countdown() -> Style {
    Style::default().fg(TURQUOISE)
}

/// Tag labels: jade.
pub fn tag() -> Style {
    Style::default().fg(JADE)
}

/// Error / danger text: cochineal + bold.
pub fn error() -> Style {
    Style::default()
        .fg(COCHINEAL)
        .add_modifier(TextModifiers::BOLD)
}

/// Inline hint text: muted.
pub fn hint() -> Style {
    Style::default().fg(MUTED)
}

/// Status-bar hint: key glyph — gold + bold so it pops.
pub fn hint_key() -> Style {
    Style::default().fg(GOLD).add_modifier(TextModifiers::BOLD)
}

/// Status-bar hint: action label — readable, not opaque.
pub fn hint_label() -> Style {
    Style::default().fg(MUTED_BRIGHT)
}

/// Success / info notices: jade.
pub fn success() -> Style {
    Style::default().fg(JADE)
}

/// Revealed / emphasised value: bright gold.
pub fn revealed() -> Style {
    Style::default().fg(GOLD_BRIGHT)
}

/// Normal body text: cream.
pub fn text() -> Style {
    Style::default().fg(TEXT)
}

/// Border style: muted.
pub fn border() -> Style {
    Style::default().fg(MUTED)
}

// ─── Nerd Font icons ────────────────────────────────────────────────────────

/// Centralised Nerd Font glyph constants.
///
/// **Requires a Nerd Font** installed and set as the terminal font.
/// Recommended: JetBrainsMono Nerd Font (<https://www.nerdfonts.com/>).
pub mod icons {
    /// Directory (folder) — collapsed state.  U+F07B
    pub const DIR_CLOSED: &str = "\u{f07b}";
    /// Directory (folder) — expanded state.  U+F07C
    pub const DIR_OPEN: &str = "\u{f07c}";
    /// Entry / leaf — key glyph.  U+F084
    pub const ENTRY: &str = "\u{f084}";
    /// Lock — used for the brand header and password field.  U+F023
    pub const LOCK: &str = "\u{f023}";

    // ── Detail-field icons ────────────────────────────────────────────────
    /// Person / user field.  U+F007
    pub const USER: &str = "\u{f007}";
    /// Globe / URL or website field.  U+F0AC
    pub const URL: &str = "\u{f0ac}";
    /// Envelope / email field.  U+F0E0
    pub const EMAIL: &str = "\u{f0e0}";
    /// Clock / OTP countdown.  U+F017
    pub const OTP: &str = "\u{f017}";
    /// Tag / tags field.  U+F02B
    pub const TAG: &str = "\u{f02b}";
    /// Generic field / unknown key.  U+F15C
    pub const FIELD: &str = "\u{f15c}";

    // ── Misc UI icons ─────────────────────────────────────────────────────
    /// Search / magnifying glass.  U+F002
    pub const SEARCH: &str = "\u{f002}";

    /// Return the best icon for a known field key, falling back to `FIELD`.
    pub fn for_key(key: &str) -> &'static str {
        match key.to_lowercase().as_str() {
            "user" | "username" | "login" => USER,
            "url" | "website" | "site" | "link" | "homepage" => URL,
            "email" | "mail" => EMAIL,
            "otp" | "totp" | "hotp" => OTP,
            "tag" | "tags" => TAG,
            "password" | "pass" | "pw" => LOCK,
            _ => FIELD,
        }
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gold_rgb_matches_spec() {
        // spec: #E0A436 → (224, 164, 54)
        assert_eq!(GOLD, Color::Rgb(224, 164, 54));
    }

    #[test]
    fn bg_rgb_matches_spec() {
        // spec: #15131A → (21, 19, 26)
        assert_eq!(BG, Color::Rgb(21, 19, 26));
    }

    #[test]
    fn cochineal_rgb_matches_spec() {
        // spec: #C8443B → (200, 68, 59)
        assert_eq!(COCHINEAL, Color::Rgb(200, 68, 59));
    }

    #[test]
    fn selection_has_gold_fg_and_surface_sel_bg() {
        let s = selection();
        assert_eq!(s.fg, Some(GOLD));
        assert_eq!(s.bg, Some(SURFACE_SEL));
        assert!(s.add_modifier.contains(TextModifiers::BOLD));
    }

    #[test]
    fn title_has_gold_fg_and_bold() {
        let s = title();
        assert_eq!(s.fg, Some(GOLD));
        assert!(s.add_modifier.contains(TextModifiers::BOLD));
    }

    #[test]
    fn error_has_cochineal_fg() {
        assert_eq!(error().fg, Some(COCHINEAL));
    }

    #[test]
    fn otp_has_turquoise_bright_fg() {
        assert_eq!(otp().fg, Some(TURQUOISE_BRIGHT));
    }

    #[test]
    fn success_has_jade_fg() {
        assert_eq!(success().fg, Some(JADE));
    }

    #[test]
    fn hint_has_muted_fg() {
        assert_eq!(hint().fg, Some(MUTED));
    }

    #[test]
    fn revealed_has_gold_bright_fg() {
        assert_eq!(revealed().fg, Some(GOLD_BRIGHT));
    }
}
