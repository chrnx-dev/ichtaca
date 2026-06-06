//! Ichtaca "Obsidiana & Oro" colour theme.
//!
//! All palette values are `ratatui::style::Color::Rgb(r, g, b)` so they render
//! correctly on 24-bit terminals.  Convenience [`Style`] helpers are provided
//! for the most common semantic roles; import the module and call e.g.
//! `theme::title()` instead of constructing styles by hand in render code.

use ratatui::style::{Color, Modifier, Style};

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

/// Gold — primary accent: selection highlight, titles, brand.
pub const GOLD: Color = Color::Rgb(224, 164, 54);

/// Bright gold — revealed / emphasised values.
pub const GOLD_BRIGHT: Color = Color::Rgb(242, 198, 109);

/// Turquoise — secondary accent: links, field values.
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
        .add_modifier(Modifier::BOLD)
}

/// Block/panel title: gold + bold.
pub fn title() -> Style {
    Style::default().fg(GOLD).add_modifier(Modifier::BOLD)
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
    Style::default().fg(COCHINEAL).add_modifier(Modifier::BOLD)
}

/// Inline hint text: muted.
pub fn hint() -> Style {
    Style::default().fg(MUTED)
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

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_has_gold_fg_and_surface_sel_bg() {
        let s = selection();
        assert_eq!(s.fg, Some(GOLD), "selection() fg should be GOLD");
        assert_eq!(
            s.bg,
            Some(SURFACE_SEL),
            "selection() bg should be SURFACE_SEL"
        );
        assert!(
            s.add_modifier.contains(Modifier::BOLD),
            "selection() should be BOLD"
        );
    }

    #[test]
    fn title_has_gold_fg_and_bold() {
        let s = title();
        assert_eq!(s.fg, Some(GOLD), "title() fg should be GOLD");
        assert!(
            s.add_modifier.contains(Modifier::BOLD),
            "title() should be BOLD"
        );
    }

    #[test]
    fn error_has_cochineal_fg() {
        let s = error();
        assert_eq!(s.fg, Some(COCHINEAL), "error() fg should be COCHINEAL");
    }

    #[test]
    fn otp_has_turquoise_bright_fg() {
        let s = otp();
        assert_eq!(
            s.fg,
            Some(TURQUOISE_BRIGHT),
            "otp() fg should be TURQUOISE_BRIGHT"
        );
    }

    #[test]
    fn success_has_jade_fg() {
        let s = success();
        assert_eq!(s.fg, Some(JADE), "success() fg should be JADE");
    }

    #[test]
    fn hint_has_muted_fg() {
        let s = hint();
        assert_eq!(s.fg, Some(MUTED), "hint() fg should be MUTED");
    }

    #[test]
    fn revealed_has_gold_bright_fg() {
        let s = revealed();
        assert_eq!(
            s.fg,
            Some(GOLD_BRIGHT),
            "revealed() fg should be GOLD_BRIGHT"
        );
    }
}
