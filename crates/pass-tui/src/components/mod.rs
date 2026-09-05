//! UI components for the Ichtaca TUI.
//!
//! Each component implements `Component` (rendering + props) and
//! `AppComponent<Msg, NoUserEvent>` (event → `Msg` mapping).

pub mod confirm_modal;
pub mod custom_field_modal;
pub mod detail;
pub mod form_modal;
pub mod header;
pub mod notes_field;
pub mod search_modal;
pub mod status_bar;
pub mod template_modal;
pub mod tree;

pub use confirm_modal::ConfirmModal;
pub use custom_field_modal::CustomFieldInput;
pub use detail::Detail;
pub use form_modal::{FormField, FormMode};
pub use header::Header;
pub use notes_field::NotesField;
pub use search_modal::{SearchInput, SearchResults};
pub use status_bar::{git_chip, git_chip_width, StatusBar};
pub use template_modal::TemplateModal;
pub use tree::EntryTree;
