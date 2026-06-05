//! Pure state transitions. `update` mutates `AppState` and returns an optional
//! `SideEffect` for the runtime. No I/O happens here.

use crate::action::{Action, SideEffect};
use crate::state::{AppState, Mode};
use crate::tree::{flatten, move_selection, FlatRow, Nav};

/// Apply an action; return a side effect for the runtime to perform, if any.
// Called by `app.rs` (Task 13); allow until the runtime is wired up.
#[allow(dead_code)]
pub fn update(state: &mut AppState, action: Action) -> Option<SideEffect> {
    state.clear_notification();
    match state.mode.clone() {
        Mode::Browse => update_browse(state, action),
        // Search / EditForm / Confirm / Help transitions arrive in later tasks.
        _ => update_other(state, action),
    }
}

fn visible(state: &AppState) -> Vec<FlatRow> {
    flatten(&state.roots, &state.expanded)
}

fn selected_row(state: &AppState) -> Option<FlatRow> {
    visible(state).into_iter().nth(state.selected)
}

/// After a selection move, if the new row is a leaf, request its detail.
fn after_move(state: &mut AppState) -> Option<SideEffect> {
    match selected_row(state) {
        Some(row) if !row.is_dir => {
            let path = row.path.clone()?;
            if state.detail_path.as_deref() == Some(path.as_str()) {
                None
            } else {
                Some(SideEffect::LoadDetail(path))
            }
        }
        _ => None,
    }
}

fn update_browse(state: &mut AppState, action: Action) -> Option<SideEffect> {
    let len = visible(state).len();
    match action {
        Action::Quit => {
            state.should_quit = true;
            None
        }
        Action::MoveDown => {
            state.selected = move_selection(state.selected, len, Nav::Down);
            after_move(state)
        }
        Action::MoveUp => {
            state.selected = move_selection(state.selected, len, Nav::Up);
            after_move(state)
        }
        Action::MoveTop => {
            state.selected = move_selection(state.selected, len, Nav::Top);
            after_move(state)
        }
        Action::MoveBottom => {
            state.selected = move_selection(state.selected, len, Nav::Bottom);
            after_move(state)
        }
        Action::Expand => {
            if let Some(row) = selected_row(state) {
                if let Some(key) = row.dir_key {
                    state.expanded.insert(key);
                }
            }
            None
        }
        Action::Collapse => {
            if let Some(row) = selected_row(state) {
                if let Some(key) = row.dir_key {
                    state.expanded.remove(&key);
                }
            }
            None
        }
        Action::ToggleReveal => {
            state.reveal = !state.reveal;
            None
        }
        Action::Copy => selected_row(state)
            .and_then(|r| r.path)
            .map(SideEffect::CopyPassword),
        Action::EnterSearch => {
            state.mode = Mode::Search;
            None
        }
        _ => None,
    }
}

/// Placeholder for non-Browse modes; replaced/extended in Tasks 8–11.
fn update_other(state: &mut AppState, action: Action) -> Option<SideEffect> {
    if let Action::Cancel = action {
        state.mode = Mode::Browse;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{Action, SideEffect};
    use crate::state::{AppState, Mode};
    use passcore::EntryNode;

    fn state_with_web_expanded() -> AppState {
        let mut s = AppState::new();
        s.roots = EntryNode::from_paths(&[
            "email/work".to_string(),
            "web/github.com".to_string(),
            "web/gitlab.com".to_string(),
        ]);
        s.expanded.insert("web".to_string());
        s
    }

    #[test]
    fn move_down_advances_selection() {
        let mut s = state_with_web_expanded();
        let eff = update(&mut s, Action::MoveDown);
        assert_eq!(s.selected, 1);
        assert!(eff.is_none());
    }

    #[test]
    fn quit_sets_should_quit() {
        let mut s = AppState::new();
        update(&mut s, Action::Quit);
        assert!(s.should_quit);
    }

    #[test]
    fn expanding_a_dir_adds_to_expanded_set() {
        let mut s = AppState::new();
        s.roots = EntryNode::from_paths(&["web/github.com".to_string()]);
        s.selected = 0; // the `web` dir row
        update(&mut s, Action::Expand);
        assert!(s.expanded.contains("web"));
    }

    #[test]
    fn collapsing_an_expanded_dir_removes_it() {
        let mut s = state_with_web_expanded();
        // select the `web` row (index 1: email, web, ...)
        s.selected = 1;
        update(&mut s, Action::Collapse);
        assert!(!s.expanded.contains("web"));
    }

    #[test]
    fn moving_onto_a_leaf_requests_detail_load() {
        let mut s = state_with_web_expanded();
        // rows: email(0), web(1), github.com(2), gitlab.com(3)
        s.selected = 1;
        let eff = update(&mut s, Action::MoveDown); // now on github.com
        assert_eq!(s.selected, 2);
        assert_eq!(
            eff,
            Some(SideEffect::LoadDetail("web/github.com".to_string()))
        );
    }

    #[test]
    fn copy_on_a_leaf_emits_copy_effect() {
        let mut s = state_with_web_expanded();
        s.selected = 2; // github.com
        let eff = update(&mut s, Action::Copy);
        assert_eq!(
            eff,
            Some(SideEffect::CopyPassword("web/github.com".to_string()))
        );
    }

    #[test]
    fn toggle_reveal_flips_flag() {
        let mut s = AppState::new();
        assert!(!s.reveal);
        update(&mut s, Action::ToggleReveal);
        assert!(s.reveal);
        update(&mut s, Action::ToggleReveal);
        assert!(!s.reveal);
    }

    #[test]
    fn enter_search_switches_mode() {
        let mut s = AppState::new();
        update(&mut s, Action::EnterSearch);
        assert_eq!(s.mode, Mode::Search);
    }
}
