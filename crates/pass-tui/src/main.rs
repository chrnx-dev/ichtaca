//! pass-tui — a ratatui terminal frontend for `pass`, consuming `passcore`.
//!
//! This file is intentionally thin: terminal glue only. All logic lives in the
//! sibling modules (`state`, `tree`, `action`, `keymap`, `update`, `ui`, ...).

mod action;
// form is consumed by `update` (Task 10) and the form UI widget (Task 12).
#[allow(dead_code)]
mod form;
mod keymap;
mod search;
mod state;
mod tree;
mod update;

fn main() {
    // Real entrypoint arrives in Task 13. For now, prove the crate builds.
    println!("pass-tui");
}
