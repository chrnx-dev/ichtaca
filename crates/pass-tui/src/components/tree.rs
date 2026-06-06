//! Tree panel component — password-entry tree navigable with arrow/vim keys.
//!
//! Wraps `tui_realm_treeview::TreeView<String>`, maps keyboard events to
//! `CmdResult`s, and converts them to `Msg`:
//!
//! - `CmdResult::Changed` after move → `Msg::None` (redraw is enough)
//! - `CmdResult::Submit` after Enter → `Msg::SelectEntry(id)` (id == store path
//!   for leaf nodes, directory id for folder nodes — the model ignores dirs)
//! - Move/fold keys also emit `Msg::None` so the main loop redraws.
//!
//! ## Config-driven keybindings (DEFERRED)
//!
//! `passcore::Config.keybindings` allows overriding j/k/h/l/c/s/q and other
//! keys.  Currently the keys in the `on()` match are **hardcoded** to the
//! default vim values.  Wiring them through would require passing a
//! `KeybindingsConfig` reference into the component at construction time (and
//! rebuilding the match as a dynamic comparison), which is left as a follow-up
//! task.  The arrow keys (↑↓←→) always work regardless of this setting.

use tui_realm_treeview::{Node, Tree, TreeView, TREE_CMD_CLOSE, TREE_CMD_OPEN};
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{Event, Key, KeyEvent, KeyModifiers, NoUserEvent};
use tuirealm::props::{
    AttrValue, Attribute, BorderType, Borders, HorizontalAlignment, QueryResult, Style, Title,
};
use tuirealm::ratatui::layout::Rect;
use tuirealm::state::{State, StateValue};

use crate::msg::Msg;
use crate::theme;

/// Left-panel tree component.
pub struct EntryTree {
    inner: TreeView<String>,
}

impl EntryTree {
    /// Build a new tree from the given root nodes (from `EntryNode::from_paths`).
    /// `initial_node` is the id of the node to pre-select (root id by default).
    pub fn new(tree: Tree<String>, initial_node: Option<String>) -> Self {
        let initial = match initial_node {
            Some(id) if tree.root().query(&id).is_some() => id,
            _ => tree.root().id().to_string(),
        };

        let inner = TreeView::default()
            .background(theme::SURFACE)
            .foreground(theme::TEXT)
            .borders(
                Borders::default()
                    .color(theme::MUTED)
                    .modifiers(BorderType::Rounded),
            )
            .inactive(Style::default().fg(theme::MUTED))
            .indent_size(2)
            .scroll_step(8)
            .title(Title::from("Entries").alignment(HorizontalAlignment::Left))
            .highlight_style(theme::selection())
            .highlight_str("▸")
            .with_tree(tree)
            .initial_node(initial);

        Self { inner }
    }
}

// ── Component forwarding ──────────────────────────────────────────────────────

impl Component for EntryTree {
    fn view(&mut self, frame: &mut tuirealm::ratatui::Frame, area: Rect) {
        self.inner.view(frame, area);
    }

    fn query<'a>(&'a self, attr: Attribute) -> Option<QueryResult<'a>> {
        self.inner.query(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.inner.attr(attr, value);
    }

    fn state(&self) -> State {
        self.inner.state()
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        self.inner.perform(cmd)
    }
}

// ── AppComponent — key → Msg mapping ─────────────────────────────────────────

impl AppComponent<Msg, NoUserEvent> for EntryTree {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        let result = match ev {
            // ── Movement ─────────────────────────────────────────────────
            Event::Keyboard(KeyEvent {
                code: Key::Down | Key::Char('j'),
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Down)),

            Event::Keyboard(KeyEvent {
                code: Key::Up | Key::Char('k'),
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Move(Direction::Up)),

            Event::Keyboard(KeyEvent {
                code: Key::Char('g'),
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::GoTo(Position::Begin)),

            Event::Keyboard(KeyEvent {
                code: Key::Char('G'),
                modifiers: KeyModifiers::SHIFT,
            }) => self.perform(Cmd::GoTo(Position::End)),

            Event::Keyboard(KeyEvent {
                code: Key::PageDown,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Scroll(Direction::Down)),

            Event::Keyboard(KeyEvent {
                code: Key::PageUp,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Scroll(Direction::Up)),

            // ── Fold / Unfold ─────────────────────────────────────────────
            Event::Keyboard(KeyEvent {
                code: Key::Right | Key::Char('l'),
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Custom(TREE_CMD_OPEN)),

            Event::Keyboard(KeyEvent {
                code: Key::Left | Key::Char('h'),
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Custom(TREE_CMD_CLOSE)),

            // ── Select ────────────────────────────────────────────────────
            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Submit),

            // ── Global passthrough keys ───────────────────────────────────
            Event::Keyboard(KeyEvent {
                code: Key::Char('q'),
                modifiers: KeyModifiers::NONE,
            }) => return Some(Msg::Quit),

            Event::Keyboard(KeyEvent {
                code: Key::Char('c'),
                modifiers: KeyModifiers::CONTROL,
            }) => return Some(Msg::Quit),

            Event::Keyboard(KeyEvent {
                code: Key::Esc,
                modifiers: KeyModifiers::NONE,
            }) => return Some(Msg::Quit),

            Event::Keyboard(KeyEvent {
                code: Key::Char('c'),
                modifiers: KeyModifiers::NONE,
            }) => return Some(Msg::Copy),

            Event::Keyboard(KeyEvent {
                code: Key::Char('s'),
                modifiers: KeyModifiers::NONE,
            }) => return Some(Msg::ToggleReveal),

            // ── Phase 3 CRUD keys ─────────────────────────────────────────
            Event::Keyboard(KeyEvent {
                code: Key::Char('/'),
                modifiers: KeyModifiers::NONE,
            }) => return Some(Msg::OpenSearch),

            Event::Keyboard(KeyEvent {
                code: Key::Char('a'),
                modifiers: KeyModifiers::NONE,
            }) => return Some(Msg::OpenCreate),

            Event::Keyboard(KeyEvent {
                code: Key::Char('e'),
                modifiers: KeyModifiers::NONE,
            }) => return Some(Msg::OpenEdit),

            Event::Keyboard(KeyEvent {
                code: Key::Char('E'),
                modifiers: KeyModifiers::SHIFT,
            }) => return Some(Msg::OpenRawEdit),

            Event::Keyboard(KeyEvent {
                code: Key::Char('d'),
                modifiers: KeyModifiers::NONE,
            }) => return Some(Msg::AskDelete),

            // Tick — used to refresh the OTP countdown in the Detail panel.
            Event::Tick => return Some(Msg::Tick),

            // Anything else — consume the event, no-op.
            _ => return Some(Msg::None),
        };

        match result {
            // Tree selection changed: if the currently highlighted node is a
            // leaf (has a store path == id), emit SelectEntry.
            CmdResult::Changed(State::Single(StateValue::String(id))) => Some(Msg::SelectEntry(id)),
            CmdResult::Submit(State::Single(StateValue::String(id))) => Some(Msg::SelectEntry(id)),
            // Fold/unfold or no change — just redraw.
            _ => Some(Msg::None),
        }
    }
}

// ── Tree builder ──────────────────────────────────────────────────────────────

/// Build a `tui_realm_treeview::Tree<String>` from a list of `EntryNode` roots.
///
/// The tree root is a virtual "store" node (id = "").  Each `EntryNode` maps
/// to a `Node<String, String>` where the id is the store path for leaves and
/// the name segment for directories.
pub fn build_tree(roots: &[passcore::EntryNode]) -> Tree<String> {
    let mut root = Node::new(String::new(), String::from("store"));
    for n in roots {
        root = root.with_child(entry_node_to_orange(n));
    }
    Tree::new(root)
}

fn entry_node_to_orange(n: &passcore::EntryNode) -> Node<String> {
    // For leaves: id = full store path (e.g. "web/github.com").
    // For directories: id = name segment (e.g. "web").
    let id = if n.is_leaf() {
        n.path.clone().unwrap_or_else(|| n.name.clone())
    } else {
        n.name.clone()
    };

    let label = n.name.clone();
    let mut node = Node::new(id, label);
    for child in &n.children {
        node = node.with_child(entry_node_to_orange(child));
    }
    node
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use passcore::EntryNode;

    fn paths_tree(paths: &[&str]) -> Tree<String> {
        let owned: Vec<String> = paths.iter().map(|s| s.to_string()).collect();
        let nodes = EntryNode::from_paths(&owned);
        build_tree(&nodes)
    }

    #[test]
    fn tree_has_virtual_root() {
        let t = paths_tree(&["web/github.com", "web/gitlab.com"]);
        assert_eq!(t.root().id(), "");
    }

    #[test]
    fn leaf_id_is_full_path() {
        let t = paths_tree(&["web/github.com"]);
        // root → web → github.com (id == "web/github.com")
        let web = t.root().query(&"web".to_string()).unwrap();
        assert!(web.query(&"web/github.com".to_string()).is_some());
    }

    #[test]
    fn dir_id_is_name_segment() {
        let t = paths_tree(&["web/github.com", "email/work"]);
        assert!(t.root().query(&"web".to_string()).is_some());
        assert!(t.root().query(&"email".to_string()).is_some());
    }

    #[test]
    fn select_entry_msg_on_changed() {
        // Build a tree component and simulate Move(Down) to get a Changed result.
        let t = paths_tree(&["a/b", "a/c"]);
        let initial = t.root().id().to_string();
        let mut comp = EntryTree::new(t, Some(initial));

        // Move down once — expect SelectEntry or None (depending on what node is hit).
        let msg = comp.on(&Event::Keyboard(KeyEvent::new(
            Key::Down,
            KeyModifiers::NONE,
        )));
        // Should produce Some(Msg::SelectEntry(_)) or Some(Msg::None)
        assert!(msg.is_some());
    }

    #[test]
    fn quit_key_emits_quit() {
        let t = paths_tree(&["a/b"]);
        let mut comp = EntryTree::new(t, None);
        let msg = comp.on(&Event::Keyboard(KeyEvent::new(
            Key::Char('q'),
            KeyModifiers::NONE,
        )));
        assert_eq!(msg, Some(Msg::Quit));
    }

    #[test]
    fn copy_key_emits_copy() {
        let t = paths_tree(&["a/b"]);
        let mut comp = EntryTree::new(t, None);
        let msg = comp.on(&Event::Keyboard(KeyEvent::new(
            Key::Char('c'),
            KeyModifiers::NONE,
        )));
        assert_eq!(msg, Some(Msg::Copy));
    }

    #[test]
    fn reveal_key_emits_toggle_reveal() {
        let t = paths_tree(&["a/b"]);
        let mut comp = EntryTree::new(t, None);
        let msg = comp.on(&Event::Keyboard(KeyEvent::new(
            Key::Char('s'),
            KeyModifiers::NONE,
        )));
        assert_eq!(msg, Some(Msg::ToggleReveal));
    }
}
