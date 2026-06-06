//! Pure mapping from a crossterm `KeyEvent` (plus current `Mode` and config) to
//! an `Action`. No terminal, no state mutation.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use passcore::config::KeybindingsConfig;

use crate::action::Action;
use crate::state::Mode;

/// Resolve a key event into an action for the current mode.
pub fn map(ev: KeyEvent, mode: &Mode, kb: &KeybindingsConfig) -> Action {
    match mode {
        Mode::Browse => map_browse(ev, kb),
        Mode::Search => map_text_input(ev),
        Mode::EditForm => map_edit_form(ev),
        Mode::Confirm(_) => map_confirm(ev),
        Mode::Help => map_help(ev),
    }
}

fn single(s: &str) -> Option<char> {
    let mut it = s.chars();
    match (it.next(), it.next()) {
        (Some(c), None) => Some(c),
        _ => None,
    }
}

fn map_browse(ev: KeyEvent, kb: &KeybindingsConfig) -> Action {
    // Arrow keys always work alongside the configurable vim bindings:
    // ↑/↓ navigate, →/← expand/collapse.
    match ev.code {
        KeyCode::Down => return Action::MoveDown,
        KeyCode::Up => return Action::MoveUp,
        KeyCode::Right => return Action::Expand,
        KeyCode::Left => return Action::Collapse,
        _ => {}
    }
    let c = match ev.code {
        KeyCode::Char(c) => c,
        _ => return Action::Noop,
    };
    // Compare against configured single-char bindings.
    let m = |field: &str| single(field) == Some(c);
    if m(&kb.down) {
        Action::MoveDown
    } else if m(&kb.up) {
        Action::MoveUp
    } else if m(&kb.expand) {
        Action::Expand
    } else if m(&kb.collapse) {
        Action::Collapse
    } else if m(&kb.top) {
        Action::MoveTop
    } else if m(&kb.bottom) {
        Action::MoveBottom
    } else if m(&kb.search) {
        Action::EnterSearch
    } else if m(&kb.command) {
        Action::EnterCommand
    } else if m(&kb.copy) {
        Action::Copy
    } else if m(&kb.reveal) {
        Action::ToggleReveal
    } else if m(&kb.quit) {
        Action::Quit
    } else {
        // Fixed (non-configurable) CRUD verbs in Browse.
        match c {
            'a' => Action::BeginCreate,
            'e' => Action::BeginEdit,
            'E' => Action::BeginRawEdit,
            'd' => Action::BeginDelete,
            _ => Action::Noop,
        }
    }
}

fn map_text_input(ev: KeyEvent) -> Action {
    match ev.code {
        KeyCode::Esc => Action::Cancel,
        KeyCode::Enter => Action::Accept,
        KeyCode::Backspace => Action::Backspace,
        KeyCode::Down => Action::MoveDown,
        KeyCode::Up => Action::MoveUp,
        KeyCode::Char(c) => Action::Input(c),
        _ => Action::Noop,
    }
}

/// Keymap for the create/edit form overlay.
/// Extends `map_text_input` with:
/// - Ctrl-g → `GenerateInField`
/// - Tab → `MoveDown` (cycle fields forward)
/// - BackTab → `MoveUp` (cycle fields backward)
///
/// Note: bare `j`/`k` still produce `Input('j')`/`Input('k')` (text input),
/// NOT navigation. Only Tab/arrows navigate fields in form mode.
fn map_edit_form(ev: KeyEvent) -> Action {
    // Ctrl-g generates a password into the focused field.
    if ev.code == KeyCode::Char('g') && ev.modifiers == KeyModifiers::CONTROL {
        return Action::GenerateInField;
    }
    // Tab / BackTab cycle fields.
    match ev.code {
        KeyCode::Tab => return Action::MoveDown,
        KeyCode::BackTab => return Action::MoveUp,
        _ => {}
    }
    map_text_input(ev)
}

fn map_confirm(ev: KeyEvent) -> Action {
    match ev.code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => Action::Accept,
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Action::Cancel,
        _ => Action::Noop,
    }
}

fn map_help(ev: KeyEvent) -> Action {
    match ev.code {
        KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
        _ => Action::Cancel,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::Action;
    use crate::state::Mode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use passcore::config::KeybindingsConfig;

    fn key(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
    }

    #[test]
    fn browse_vim_keys_map_to_navigation() {
        let kb = KeybindingsConfig::default();
        assert_eq!(map(key('j'), &Mode::Browse, &kb), Action::MoveDown);
        assert_eq!(map(key('k'), &Mode::Browse, &kb), Action::MoveUp);
        assert_eq!(map(key('h'), &Mode::Browse, &kb), Action::Collapse);
        assert_eq!(map(key('l'), &Mode::Browse, &kb), Action::Expand);
        assert_eq!(map(key('g'), &Mode::Browse, &kb), Action::MoveTop);
        assert_eq!(map(key('G'), &Mode::Browse, &kb), Action::MoveBottom);
    }

    #[test]
    fn browse_arrow_keys_navigate_and_fold() {
        let kb = KeybindingsConfig::default();
        let arrow = |code| KeyEvent::new(code, KeyModifiers::NONE);
        assert_eq!(
            map(arrow(KeyCode::Down), &Mode::Browse, &kb),
            Action::MoveDown
        );
        assert_eq!(map(arrow(KeyCode::Up), &Mode::Browse, &kb), Action::MoveUp);
        assert_eq!(
            map(arrow(KeyCode::Right), &Mode::Browse, &kb),
            Action::Expand
        );
        assert_eq!(
            map(arrow(KeyCode::Left), &Mode::Browse, &kb),
            Action::Collapse
        );
    }

    #[test]
    fn browse_action_keys_map() {
        let kb = KeybindingsConfig::default();
        assert_eq!(map(key('/'), &Mode::Browse, &kb), Action::EnterSearch);
        assert_eq!(map(key(':'), &Mode::Browse, &kb), Action::EnterCommand);
        assert_eq!(map(key('c'), &Mode::Browse, &kb), Action::Copy);
        assert_eq!(map(key('s'), &Mode::Browse, &kb), Action::ToggleReveal);
        assert_eq!(map(key('q'), &Mode::Browse, &kb), Action::Quit);
    }

    #[test]
    fn browse_respects_configured_overrides() {
        let kb = KeybindingsConfig {
            down: "n".into(),
            ..KeybindingsConfig::default()
        };
        assert_eq!(map(key('n'), &Mode::Browse, &kb), Action::MoveDown);
        // the old default no longer moves down
        assert_eq!(map(key('j'), &Mode::Browse, &kb), Action::Noop);
    }

    #[test]
    fn search_mode_routes_text_to_input() {
        let kb = KeybindingsConfig::default();
        assert_eq!(map(key('a'), &Mode::Search, &kb), Action::Input('a'));
        assert_eq!(
            map(
                KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
                &Mode::Search,
                &kb
            ),
            Action::Backspace
        );
        assert_eq!(
            map(
                KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
                &Mode::Search,
                &kb
            ),
            Action::Cancel
        );
        assert_eq!(
            map(
                KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
                &Mode::Search,
                &kb
            ),
            Action::Accept
        );
    }

    #[test]
    fn edit_form_ctrl_g_maps_to_generate_in_field() {
        let kb = KeybindingsConfig::default();
        let ctrl_g = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL);
        assert_eq!(
            map(ctrl_g, &Mode::EditForm, &kb),
            Action::GenerateInField,
            "Ctrl-g in EditForm mode should produce GenerateInField"
        );
    }

    #[test]
    fn edit_form_regular_chars_still_route_to_input() {
        let kb = KeybindingsConfig::default();
        assert_eq!(map(key('a'), &Mode::EditForm, &kb), Action::Input('a'));
        assert_eq!(
            map(
                KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
                &Mode::EditForm,
                &kb
            ),
            Action::Cancel
        );
    }

    #[test]
    fn edit_form_tab_maps_to_move_down() {
        let kb = KeybindingsConfig::default();
        let tab = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        assert_eq!(
            map(tab, &Mode::EditForm, &kb),
            Action::MoveDown,
            "Tab in EditForm should produce MoveDown"
        );
    }

    #[test]
    fn edit_form_backtab_maps_to_move_up() {
        let kb = KeybindingsConfig::default();
        let backtab = KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT);
        assert_eq!(
            map(backtab, &Mode::EditForm, &kb),
            Action::MoveUp,
            "BackTab in EditForm should produce MoveUp"
        );
    }

    #[test]
    fn edit_form_arrow_down_maps_to_move_down() {
        let kb = KeybindingsConfig::default();
        let down = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        assert_eq!(
            map(down, &Mode::EditForm, &kb),
            Action::MoveDown,
            "Down arrow in EditForm should produce MoveDown"
        );
    }

    #[test]
    fn edit_form_arrow_up_maps_to_move_up() {
        let kb = KeybindingsConfig::default();
        let up = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        assert_eq!(
            map(up, &Mode::EditForm, &kb),
            Action::MoveUp,
            "Up arrow in EditForm should produce MoveUp"
        );
    }

    #[test]
    fn edit_form_j_still_produces_text_input() {
        let kb = KeybindingsConfig::default();
        assert_eq!(
            map(key('j'), &Mode::EditForm, &kb),
            Action::Input('j'),
            "bare 'j' in EditForm must still produce text Input, not navigation"
        );
    }

    #[test]
    fn edit_form_k_still_produces_text_input() {
        let kb = KeybindingsConfig::default();
        assert_eq!(
            map(key('k'), &Mode::EditForm, &kb),
            Action::Input('k'),
            "bare 'k' in EditForm must still produce text Input, not navigation"
        );
    }

    #[test]
    fn search_mode_arrow_down_maps_to_move_down() {
        let kb = KeybindingsConfig::default();
        let down = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        assert_eq!(
            map(down, &Mode::Search, &kb),
            Action::MoveDown,
            "Down arrow in Search should produce MoveDown"
        );
    }

    #[test]
    fn search_mode_arrow_up_maps_to_move_up() {
        let kb = KeybindingsConfig::default();
        let up = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        assert_eq!(
            map(up, &Mode::Search, &kb),
            Action::MoveUp,
            "Up arrow in Search should produce MoveUp"
        );
    }

    #[test]
    fn confirm_mode_maps_y_n() {
        let kb = KeybindingsConfig::default();
        let confirm = Mode::Confirm(crate::state::Confirm {
            prompt: "delete?".into(),
            target: "web/x".into(),
            kind: crate::state::ConfirmKind::Delete,
        });
        assert_eq!(map(key('y'), &confirm, &kb), Action::Accept);
        assert_eq!(map(key('n'), &confirm, &kb), Action::Cancel);
    }
}
