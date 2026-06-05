//! pass-tui — a ratatui terminal frontend for `pass`, consuming `passcore`.
//!
//! This file is intentionally thin: terminal glue only. All logic lives in the
//! sibling modules (`state`, `tree`, `action`, `keymap`, `update`, `ui`, ...).

mod action;
mod keymap;
mod state;
mod tree;

fn main() {
    // Real entrypoint arrives in Task 13. For now, prove the crate builds.
    println!("pass-tui");
}
