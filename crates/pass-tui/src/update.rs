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
        Action::BeginCreate => {
            // Default to the Login template; a UI template picker can swap it
            // before the first keystroke. Open the form immediately.
            state.form = Some(crate::form::Form::new_from_template(
                "",
                crate::form::Template::Login,
            ));
            state.mode = Mode::EditForm;
            None
        }
        Action::BeginEdit => {
            match (selected_row(state), state.detail.clone()) {
                (Some(row), Some(entry)) if !row.is_dir => {
                    if let Some(path) = row.path {
                        state.form = Some(crate::form::Form::from_entry(&path, &entry));
                        state.mode = Mode::EditForm;
                    }
                }
                _ => state.notify("select an entry to edit", crate::state::NoticeKind::Error),
            }
            None
        }
        Action::BeginRawEdit => selected_row(state)
            .filter(|r| !r.is_dir)
            .and_then(|r| r.path)
            .map(SideEffect::RawEdit),
        Action::BeginDelete => {
            if let Some(row) = selected_row(state).filter(|r| !r.is_dir) {
                if let Some(path) = row.path {
                    state.mode = Mode::Confirm(crate::state::Confirm {
                        prompt: format!("Delete {path}? (y/n)"),
                        target: path,
                        kind: crate::state::ConfirmKind::Delete,
                    });
                }
            }
            None
        }
        _ => None,
    }
}

fn update_other(state: &mut AppState, action: Action) -> Option<SideEffect> {
    match state.mode.clone() {
        Mode::Search => update_search(state, action),
        Mode::EditForm => update_form(state, action),
        Mode::Confirm(confirm) => update_confirm(state, action, confirm),
        _ => {
            if let Action::Cancel = action {
                state.mode = Mode::Browse;
            }
            None
        }
    }
}

fn update_form(state: &mut AppState, action: Action) -> Option<SideEffect> {
    match action {
        Action::Cancel => {
            state.form = None;
            state.mode = Mode::Browse;
            None
        }
        Action::Accept => {
            let form = state.form.take();
            state.mode = Mode::Browse;
            form.map(|f| SideEffect::Save {
                path: f.path.clone(),
                contents: f.to_contents(),
                overwrite: f.editing,
            })
        }
        Action::GenerateInField => Some(SideEffect::Generate { length: 20 }),
        Action::Input(c) => {
            if let Some(f) = state.form.as_mut() {
                if f.focus == 0 {
                    f.password.push(c);
                } else if let Some(field) = f.fields.get_mut(f.focus - 1) {
                    field.value.push(c);
                }
            }
            None
        }
        Action::Backspace => {
            if let Some(f) = state.form.as_mut() {
                if f.focus == 0 {
                    f.password.pop();
                } else if let Some(field) = f.fields.get_mut(f.focus - 1) {
                    field.value.pop();
                }
            }
            None
        }
        Action::MoveDown => {
            if let Some(f) = state.form.as_mut() {
                let last = f.fields.len();
                f.focus = (f.focus + 1).min(last);
            }
            None
        }
        Action::MoveUp => {
            if let Some(f) = state.form.as_mut() {
                f.focus = f.focus.saturating_sub(1);
            }
            None
        }
        _ => None,
    }
}

fn update_confirm(
    state: &mut AppState,
    action: Action,
    confirm: crate::state::Confirm,
) -> Option<SideEffect> {
    match action {
        Action::Accept => {
            state.mode = Mode::Browse;
            match confirm.kind {
                crate::state::ConfirmKind::Delete => Some(SideEffect::Remove(confirm.target)),
            }
        }
        Action::Cancel => {
            state.mode = Mode::Browse;
            None
        }
        _ => None,
    }
}

fn update_search(state: &mut AppState, action: Action) -> Option<SideEffect> {
    let all: Vec<String> = flatten_all_paths(state);
    match action {
        Action::Input(c) => {
            state.search.push(c);
            state.search.recompute(&all);
            None
        }
        Action::Backspace => {
            state.search.backspace();
            state.search.recompute(&all);
            None
        }
        Action::Cancel => {
            state.search.clear();
            state.mode = Mode::Browse;
            None
        }
        Action::Accept => {
            // Focus the chosen result: load its detail and return to Browse.
            let chosen = state.search.selected_path().map(str::to_string);
            state.mode = Mode::Browse;
            chosen.map(SideEffect::LoadDetail)
        }
        Action::MoveDown => {
            let len = state.search.results.len();
            state.search.cursor =
                crate::tree::move_selection(state.search.cursor, len, crate::tree::Nav::Down);
            None
        }
        Action::MoveUp => {
            let len = state.search.results.len();
            state.search.cursor =
                crate::tree::move_selection(state.search.cursor, len, crate::tree::Nav::Up);
            None
        }
        _ => None,
    }
}

/// All leaf paths in the tree, collected into a sorted Vec.
fn flatten_all_paths(state: &AppState) -> Vec<String> {
    let mut all = std::collections::BTreeSet::new();
    collect_paths(&state.roots, &mut all);
    all.into_iter().collect()
}

fn collect_paths(nodes: &[passcore::EntryNode], out: &mut std::collections::BTreeSet<String>) {
    for n in nodes {
        if let Some(p) = &n.path {
            out.insert(p.clone());
        }
        collect_paths(&n.children, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{Action, SideEffect};
    use crate::form::Template;
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

    #[test]
    fn search_input_recomputes_results_and_accept_loads() {
        let mut s = state_with_web_expanded();
        update(&mut s, Action::EnterSearch);
        update(&mut s, Action::Input('g'));
        update(&mut s, Action::Input('i'));
        update(&mut s, Action::Input('t'));
        assert!(!s.search.results.is_empty());
        let eff = update(&mut s, Action::Accept);
        assert_eq!(s.mode, Mode::Browse);
        assert!(matches!(eff, Some(SideEffect::LoadDetail(_))));
    }

    // --- Task 10: CRUD transitions ---

    #[test]
    fn begin_create_opens_a_form_in_edit_form_mode() {
        let mut s = AppState::new();
        update(&mut s, Action::BeginCreate);
        assert_eq!(s.mode, Mode::EditForm);
        assert!(s.form.is_some());
    }

    #[test]
    fn submitting_a_form_emits_save_effect() {
        let mut s = AppState::new();
        s.mode = Mode::EditForm;
        let mut f = crate::form::Form::new_from_template("web/new", Template::Blank);
        f.password = "pw".to_string();
        s.form = Some(f);
        let eff = update(&mut s, Action::Accept);
        assert!(
            matches!(
                eff,
                Some(SideEffect::Save { ref path, ref contents, overwrite: false })
                    if path == "web/new" && contents == "pw\n"
            ),
            "expected Save{{path=web/new, contents=pw\\n, overwrite=false}}, got {eff:?}"
        );
        assert_eq!(s.mode, Mode::Browse);
    }

    #[test]
    fn begin_delete_opens_confirm_then_accept_removes() {
        let mut s = state_with_web_expanded();
        s.selected = 2; // web/github.com
        update(&mut s, Action::BeginDelete);
        assert!(matches!(s.mode, Mode::Confirm(_)));
        let eff = update(&mut s, Action::Accept);
        assert_eq!(eff, Some(SideEffect::Remove("web/github.com".to_string())));
        assert_eq!(s.mode, Mode::Browse);
    }

    #[test]
    fn cancel_delete_returns_to_browse_without_effect() {
        let mut s = state_with_web_expanded();
        s.selected = 2;
        update(&mut s, Action::BeginDelete);
        let eff = update(&mut s, Action::Cancel);
        assert!(eff.is_none());
        assert_eq!(s.mode, Mode::Browse);
    }

    #[test]
    fn begin_raw_edit_emits_raw_edit_effect() {
        let mut s = state_with_web_expanded();
        s.selected = 2;
        let eff = update(&mut s, Action::BeginRawEdit);
        assert_eq!(eff, Some(SideEffect::RawEdit("web/github.com".to_string())));
    }

    #[test]
    fn generate_in_field_emits_generate_effect() {
        let mut s = AppState::new();
        s.mode = Mode::EditForm;
        s.form = Some(crate::form::Form::new_from_template("x", Template::Blank));
        let eff = update(&mut s, Action::GenerateInField);
        assert!(matches!(eff, Some(SideEffect::Generate { length: _ })));
    }

    #[test]
    fn form_input_appends_to_password_field() {
        let mut s = AppState::new();
        s.mode = Mode::EditForm;
        s.form = Some(crate::form::Form::new_from_template("x", Template::Blank));
        update(&mut s, Action::Input('p'));
        update(&mut s, Action::Input('w'));
        let pw = s.form.as_ref().unwrap().password.clone();
        assert_eq!(pw, "pw");
    }

    #[test]
    fn form_backspace_removes_last_char_from_password() {
        let mut s = AppState::new();
        s.mode = Mode::EditForm;
        let mut f = crate::form::Form::new_from_template("x", Template::Blank);
        f.password = "pw".to_string();
        s.form = Some(f);
        update(&mut s, Action::Backspace);
        assert_eq!(s.form.as_ref().unwrap().password, "p");
    }

    #[test]
    fn form_move_down_advances_focus() {
        let mut s = AppState::new();
        s.mode = Mode::EditForm;
        s.form = Some(crate::form::Form::new_from_template("x", Template::Login));
        update(&mut s, Action::MoveDown);
        assert_eq!(s.form.as_ref().unwrap().focus, 1);
    }

    #[test]
    fn form_cancel_clears_form_and_returns_browse() {
        let mut s = AppState::new();
        s.mode = Mode::EditForm;
        s.form = Some(crate::form::Form::new_from_template("x", Template::Blank));
        let eff = update(&mut s, Action::Cancel);
        assert!(eff.is_none());
        assert!(s.form.is_none());
        assert_eq!(s.mode, Mode::Browse);
    }

    #[test]
    fn begin_edit_with_loaded_detail_opens_form() {
        let mut s = state_with_web_expanded();
        s.selected = 2; // web/github.com
        s.detail = Some(passcore::Entry::parse("pw\nuser: bob\n"));
        s.detail_path = Some("web/github.com".to_string());
        update(&mut s, Action::BeginEdit);
        assert_eq!(s.mode, Mode::EditForm);
        assert!(s.form.is_some());
        let f = s.form.as_ref().unwrap();
        assert_eq!(f.password, "pw");
        assert!(f.editing);
    }
}
