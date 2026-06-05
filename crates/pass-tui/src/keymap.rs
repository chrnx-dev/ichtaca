//! Pure mapping from a crossterm `KeyEvent` (plus current `Mode` and config) to
//! an `Action`. No terminal, no state mutation.

use crossterm::event::{KeyCode, KeyEvent};
use passcore::config::KeybindingsConfig;

use crate::action::Action;
use crate::state::Mode;

/// Resolve a key event into an action for the current mode.
#[allow(dead_code)]
pub fn map(ev: KeyEvent, mode: &Mode, kb: &KeybindingsConfig) -> Action {
    match mode {
        Mode::Browse => map_browse(ev, kb),
        Mode::Search | Mode::EditForm => map_text_input(ev),
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
        KeyCode::Char(c) => Action::Input(c),
        _ => Action::Noop,
    }
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
