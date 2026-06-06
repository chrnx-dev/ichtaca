//! The vocabulary connecting input → state and state → runtime.
//!
//! `Action` is produced by `keymap` and consumed by `update`. `SideEffect` is
//! produced by `update` and performed by the runtime (`app.rs`).

/// A user-intent command. Pure: applying it never does I/O.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Quit,
    MoveUp,
    MoveDown,
    MoveTop,
    MoveBottom,
    Expand,
    Collapse,
    /// Toggle reveal of the password in the detail panel.
    ToggleReveal,
    /// Copy the selected entry's password to the clipboard.
    Copy,
    /// Enter fuzzy-search mode.
    EnterSearch,
    /// Enter command (`:`) mode.
    EnterCommand,
    /// Begin creating a new entry (opens the template picker → form).
    BeginCreate,
    /// Begin editing the selected entry as a form.
    BeginEdit,
    /// Begin a raw `$EDITOR` edit of the selected entry.
    BeginRawEdit,
    /// Begin deleting the selected entry (opens a confirm modal).
    BeginDelete,
    /// Generate a password into the focused form field.
    GenerateInField,
    /// Confirm the active modal / submit the active form.
    Accept,
    /// Cancel the active mode (back to Browse).
    Cancel,
    /// A character typed into a text input (search bar or form field).
    Input(char),
    /// Backspace in a text input.
    Backspace,
    /// No-op (unmapped key).
    Noop,
}

/// Something the runtime must do that pure state cannot. Returned by `update`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SideEffect {
    /// Load and parse the detail for this entry path.
    LoadDetail(String),
    /// Copy this entry path's password to the clipboard.
    CopyPassword(String),
    /// Persist a new or edited entry: full raw text for `insert`/`edit`.
    Save {
        path: String,
        contents: String,
        overwrite: bool,
    },
    /// Remove an entry.
    Remove(String),
    /// Suspend the TUI and run `pass edit <path>` (`$EDITOR`), then reload.
    RawEdit(String),
    /// Generate a password and feed it back into the form (length from config).
    Generate { length: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actions_are_comparable_and_cloneable() {
        assert_eq!(Action::Quit, Action::Quit.clone());
        assert_ne!(Action::MoveUp, Action::MoveDown);
        assert_eq!(Action::Input('a'), Action::Input('a'));
    }

    #[test]
    fn side_effects_carry_their_payloads() {
        let e = SideEffect::Save {
            path: "a".into(),
            contents: "pw\n".into(),
            overwrite: true,
        };
        assert_eq!(
            e,
            SideEffect::Save {
                path: "a".into(),
                contents: "pw\n".into(),
                overwrite: true
            }
        );
    }
}
