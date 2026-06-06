//! UI components for the Ichtaca TUI.
//!
//! Each component implements `Component` (rendering + props) and
//! `AppComponent<Msg, NoUserEvent>` (event → `Msg` mapping).

pub mod detail;
pub mod header;
pub mod status_bar;
pub mod tree;

pub use detail::Detail;
pub use header::Header;
pub use status_bar::StatusBar;
pub use tree::EntryTree;
