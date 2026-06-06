//! Component identifiers for the tui-realm `Application`.
//!
//! All components that can be mounted in the view are listed here.
//! Phase 1 uses only `Header` and `StatusBar`; later phases add the rest.

/// Uniquely identifies a component mounted in the tui-realm view.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(dead_code)] // Phase 2-3 variants are declared ahead of their use
pub enum Id {
    /// Top brand bar — "ICHTACA · lo oculto".
    Header,
    /// Password-entry tree (Phase 2).
    Tree,
    /// Detail pane (Phase 2).
    Detail,
    /// Bottom hint / status line.
    StatusBar,
    /// Search-modal input (Phase 3).
    SearchInput,
    /// Search-modal results list (Phase 3).
    SearchResults,
    /// Generic form field indexed by position (Phase 3).
    FormField(usize),
    /// Template selector inside the create/edit form (Phase 3).
    FormTemplate,
    /// Notes textarea in the create/edit form (Enhancement 2).
    FormNotes,
    /// Delete-confirm dialog (Phase 3).
    ConfirmDialog,
}
