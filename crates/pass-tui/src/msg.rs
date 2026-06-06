//! Application messages (Elm-style) for the tui-realm `Application`.
//!
//! The `update` function consumes a `Msg` and produces the next model state.
//! Phase 1 needs only `Quit`, `Tick`, and `None`; the rest are pre-declared
//! for Phases 2–3 so the compiler can guide us.

/// All messages that can flow through the application.
#[derive(Debug, PartialEq)]
#[allow(dead_code)] // Phase 2-3 variants are declared ahead of their use
pub enum Msg {
    // ── Phase 1 ──────────────────────────────────────────────────────────
    /// Exit the application cleanly.
    Quit,
    /// Periodic tick (used to refresh OTP countdown in Phase 2).
    Tick,
    /// No-op: component handled an event but has nothing to report.
    None,

    // ── Phase 2 (browse) ─────────────────────────────────────────────────
    /// Move focus up in the tree.
    MoveUp,
    /// Move focus down in the tree.
    MoveDown,
    /// Fold the selected node.
    Fold,
    /// Unfold the selected node.
    Unfold,
    /// Select the entry at the given store path.
    SelectEntry(String),
    /// Copy the password to the clipboard.
    Copy,
    /// Toggle password reveal in the detail panel.
    ToggleReveal,

    // ── Phase 3 (modals + CRUD) ───────────────────────────────────────────
    /// Open the search modal.
    OpenSearch,
    // ── Phase 4: template picker ──────────────────────────────────────────
    /// User confirmed a template index in the template-pick modal.
    SelectTemplate(usize),
    /// Search query changed.
    SearchChanged(String),
    /// User picked a search result.
    SearchPick(String),
    /// Open the create-entry form.
    OpenCreate,
    /// Open the edit-entry form.
    OpenEdit,
    /// Open the raw `$EDITOR` edit flow.
    OpenRawEdit,
    /// Move focus to the next form field.
    FormFocusNext,
    /// Move focus to the previous form field.
    FormFocusPrev,
    /// Tab pressed while the path field is focused in Create mode.
    /// The model will compute the longest common-prefix folder completion
    /// and update the path field, or fall through to FormFocusNext when
    /// there is nothing to complete.
    PathTabComplete,
    /// Generate a password inside the form.
    Generate,
    /// Submit the current form.
    SubmitForm,
    /// Ask for delete confirmation.
    AskDelete,
    /// User answered the delete confirm dialog.
    ConfirmDelete(bool),
    /// Close any overlay / modal.
    CloseOverlay,
}
