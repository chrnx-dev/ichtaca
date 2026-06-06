//! UI components for the Ichtaca TUI.
//!
//! Each component implements `Component` (rendering + props) and
//! `AppComponent<Msg, NoUserEvent>` (event → `Msg` mapping).

pub mod confirm_modal;
pub mod detail;
pub mod form_modal;
pub mod header;
pub mod search_modal;
pub mod status_bar;
pub mod tree;

pub use confirm_modal::ConfirmModal;
pub use detail::Detail;
pub use form_modal::{FormField, FormMode};
pub use header::Header;
pub use search_modal::{SearchInput, SearchResults};
pub use status_bar::StatusBar;
pub use tree::EntryTree;
