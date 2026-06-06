//! The pure application state. No ratatui, no crossterm, no terminal here.

use std::collections::BTreeSet;

use passcore::{Entry, EntryNode};

/// Which interaction mode the app is in. Drives both keymap and render.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Mode {
    /// Default: navigate the tree, view the detail panel.
    #[default]
    Browse,
    /// The fuzzy search bar is focused; typing filters the tree.
    Search,
    /// A create/edit form is focused.
    EditForm,
    /// A blocking confirmation (e.g. delete) is shown.
    Confirm(Confirm),
    /// A full-screen help / startup-check screen is shown.
    Help,
}

/// A pending confirmation the user must accept or reject.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Confirm {
    pub prompt: String,
    /// The path the confirmation acts on (e.g. the entry to delete).
    pub target: String,
    pub kind: ConfirmKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmKind {
    Delete,
}

/// A transient status-bar message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    pub text: String,
    pub kind: NoticeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoticeKind {
    Info,
    Error,
}

/// The whole app state. Pure data; transitions live in `update.rs`.
#[derive(Debug, Default)]
pub struct AppState {
    /// The full entry tree (built from `EntryNode::from_paths`).
    pub roots: Vec<EntryNode>,
    /// Directory paths (joined with `/`) that are currently expanded.
    pub expanded: BTreeSet<String>,
    /// Index of the selected row within the *visible* flattened tree.
    pub selected: usize,
    /// The detail of the currently selected leaf, once loaded.
    pub detail: Option<Entry>,
    /// Path of the entry whose detail is loaded (to avoid redundant reloads).
    pub detail_path: Option<String>,
    /// Whether the password is currently revealed in the detail panel.
    pub reveal: bool,
    /// Current interaction mode.
    pub mode: Mode,
    /// Transient status-bar message.
    pub notification: Option<Notification>,
    /// Set when the app should exit the event loop.
    pub should_quit: bool,
    /// Fuzzy search bar state (query + filtered results).
    pub search: crate::search::SearchState,
    /// The active create/edit form, if any.
    pub form: Option<crate::form::Form>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            mode: Mode::Browse,
            ..Self::default()
        }
    }

    pub fn notify(&mut self, text: impl Into<String>, kind: NoticeKind) {
        self.notification = Some(Notification {
            text: text.into(),
            kind,
        });
    }

    pub fn clear_notification(&mut self) {
        self.notification = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_starts_in_browse_with_no_selection() {
        let s = AppState::new();
        assert_eq!(s.mode, Mode::Browse);
        assert_eq!(s.selected, 0);
        assert!(!s.reveal);
        assert!(s.notification.is_none());
    }

    #[test]
    fn set_notification_and_clear() {
        let mut s = AppState::new();
        s.notify("copied", NoticeKind::Info);
        let n = s.notification.as_ref().unwrap();
        assert_eq!(n.text, "copied");
        assert_eq!(n.kind, NoticeKind::Info);
        s.clear_notification();
        assert!(s.notification.is_none());
    }

    #[test]
    fn error_notification_is_distinguishable() {
        let mut s = AppState::new();
        s.notify("boom", NoticeKind::Error);
        assert_eq!(s.notification.as_ref().unwrap().kind, NoticeKind::Error);
    }
}
