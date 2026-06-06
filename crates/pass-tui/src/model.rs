//! Application model — owns the tui-realm `Application` and all domain state.
//!
//! `Model::new` mounts the Phase-1 components; later phases add more.
//! `Model::view` draws the three-row layout (Header / content / StatusBar).
//! `Model::update` processes messages from the event loop.

use std::sync::Arc;
use std::time::Duration;

use tuirealm::application::Application;
use tuirealm::event::NoUserEvent;
use tuirealm::ratatui::layout::{Constraint, Direction, Layout};
use tuirealm::ratatui::Frame;
use tuirealm::terminal::TerminalAdapter;

use crate::components::{Detail, EntryTree, Header, StatusBar};
use crate::id::Id;
use crate::msg::Msg;
use crate::theme;

/// Central model for the Ichtaca TUI.
pub struct Model {
    /// The tui-realm application (view + subscriptions + event listener).
    pub app: Application<Id, Msg, NoUserEvent>,
    /// Set to `true` when the app should exit the main loop.
    pub quit: bool,
    /// Set to `true` when the terminal needs to be redrawn.
    pub redraw: bool,
    /// Password store backend.
    pub store: Box<dyn passcore::PasswordStore + Send>,
    /// User configuration.
    pub config: passcore::Config,

    // ── Phase 2 domain state ─────────────────────────────────────────────────
    /// Currently selected entry path (leaf store path or dir id).
    pub selected_path: Option<String>,
    /// Currently loaded entry detail (Some when a leaf is selected).
    pub detail_entry: Option<passcore::Entry>,
    /// Whether the password is currently revealed.
    pub reveal: bool,
    /// Status notice to show in the detail panel (e.g. "copied (clears in 45s)").
    pub notice: Option<String>,
}

impl Model {
    /// Mount Phase-1 components (Header + StatusBar) into the application.
    ///
    /// Call this once after `Application::init`.
    pub fn mount_phase1(&mut self) {
        self.app
            .mount(Id::Header, Box::new(Header::default()), vec![])
            .expect("mount Header");

        self.app
            .mount(Id::StatusBar, Box::new(StatusBar::default()), vec![])
            .expect("mount StatusBar");

        // Give StatusBar initial focus so it receives keyboard events via the
        // global subscriptions registered in main (q / Ctrl-C).
        self.app.active(&Id::StatusBar).expect("activate StatusBar");
    }

    /// Mount Phase-2 components (Tree + Detail) into the application.
    ///
    /// Builds the entry tree from the store and activates Tree.
    pub fn mount_phase2(&mut self) {
        // Build tree from store listing.
        let tree = build_store_tree(self.store.as_ref());

        // Find the initial node: first leaf if any.
        let initial_node = first_leaf_id(&tree);

        let tree_comp = EntryTree::new(tree, initial_node.clone());
        self.app
            .mount(Id::Tree, Box::new(tree_comp), vec![])
            .expect("mount Tree");

        let detail_comp = Detail::default();
        self.app
            .mount(Id::Detail, Box::new(detail_comp), vec![])
            .expect("mount Detail");

        // Activate the tree so keyboard events reach it.
        self.app.active(&Id::Tree).expect("activate Tree");

        // If there's a first leaf, load its detail immediately.
        if let Some(path) = initial_node {
            self.load_detail(&path);
        }
    }

    /// Draw the current frame.
    ///
    /// Layout (top → bottom):
    /// - Row 0 (1 line)  : Header
    /// - Row 1 (fill)    : Tree (left) + Detail (right)
    /// - Row 2 (1 line)  : StatusBar
    pub fn view<T: TerminalAdapter>(&mut self, terminal: &mut T) {
        let _ = terminal.draw(|f: &mut Frame| {
            Self::render_frame(&mut self.app, f);
        });
    }

    fn render_frame(app: &mut Application<Id, Msg, NoUserEvent>, f: &mut Frame) {
        let area = f.area();

        // Fill background
        let bg_block = tuirealm::ratatui::widgets::Block::default()
            .style(tuirealm::ratatui::style::Style::default().bg(theme::BG));
        f.render_widget(bg_block, area);

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Header
                Constraint::Fill(1),   // Tree + Detail
                Constraint::Length(1), // StatusBar
            ])
            .split(area);

        app.view(&Id::Header, f, rows[0]);

        // Split the middle row into left (Tree) and right (Detail).
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(35), // Tree
                Constraint::Fill(1),        // Detail
            ])
            .split(rows[1]);

        // Only render Tree/Detail if they are mounted (Phase 2+).
        if app.mounted(&Id::Tree) {
            app.view(&Id::Tree, f, cols[0]);
        }
        if app.mounted(&Id::Detail) {
            app.view(&Id::Detail, f, cols[1]);
        }

        app.view(&Id::StatusBar, f, rows[2]);
    }

    /// Process one message from the event loop.
    ///
    /// Returns `Some(next_msg)` to chain handling; `None` when done.
    pub fn update(&mut self, msg: Option<Msg>) -> Option<Msg> {
        match msg {
            None => None,
            Some(Msg::None) => None,

            Some(Msg::Quit) => {
                self.quit = true;
                None
            }

            Some(Msg::Tick) => {
                self.redraw = true;
                // Refresh OTP countdown on each tick if an entry is loaded.
                self.refresh_detail();
                None
            }

            Some(Msg::SelectEntry(path)) => {
                // Only load the entry if path changed; ignore dir nodes (no '/' = top-level dir, single segment).
                if Some(&path) != self.selected_path.as_ref() {
                    self.load_detail(&path);
                    self.reveal = false;
                    self.notice = None;
                }
                self.redraw = true;
                None
            }

            Some(Msg::ToggleReveal) => {
                self.reveal = !self.reveal;
                self.refresh_detail();
                self.redraw = true;
                None
            }

            Some(Msg::Copy) => {
                self.copy_password();
                self.redraw = true;
                None
            }

            // Phase 3 messages — received but not acted on yet.
            Some(_) => None,
        }
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    /// Load the entry at `path` from the store and refresh the detail panel.
    fn load_detail(&mut self, path: &str) {
        match self.store.show(path) {
            Ok(entry) => {
                self.selected_path = Some(path.to_string());
                self.detail_entry = Some(entry);
                self.refresh_detail();
            }
            Err(_) => {
                // Path is a directory node or the entry failed to load — clear detail.
                self.selected_path = Some(path.to_string());
                self.detail_entry = None;
                self.push_detail_clear();
            }
        }
    }

    /// Recompute and push the current detail view to the Detail component.
    fn refresh_detail(&mut self) {
        let path = match self.selected_path.clone() {
            Some(p) => p,
            None => return,
        };
        let entry = match self.detail_entry.clone() {
            Some(e) => e,
            None => return,
        };
        let otp = entry
            .otp_uri()
            .and_then(|uri| passcore::otp::current(uri).ok());

        let notice_opt = self.notice.clone();

        // Build the text lines via the Detail builder function, then push to
        // the mounted component via app.attr.
        let lines = crate::components::detail::build_lines_pub(
            &path,
            &entry,
            self.reveal,
            otp.as_ref(),
            notice_opt.as_deref(),
        );
        let text_val = tuirealm::props::AttrValue::Text(tuirealm::props::TextStatic::from(lines));
        let _ = self
            .app
            .attr(&Id::Detail, tuirealm::props::Attribute::Text, text_val);
    }

    /// Clear the detail panel (e.g. when a directory is selected).
    fn push_detail_clear(&mut self) {
        use tuirealm::props::{AttrValue, Attribute, TextStatic};
        let hint = crate::components::detail::empty_hint_line_pub();
        let _ = self.app.attr(
            &Id::Detail,
            Attribute::Text,
            AttrValue::Text(TextStatic::from(vec![hint])),
        );
    }

    /// Copy the password to the clipboard.
    fn copy_password(&mut self) {
        let entry = match &self.detail_entry {
            Some(e) => e.clone(),
            None => return,
        };
        let secret = passcore::Secret::from(entry.password());
        let timeout = Duration::from_secs(self.config.clipboard.clear_after);
        let notice = format!("copied (clears in {}s)", self.config.clipboard.clear_after);

        match passcore::clipboard::default_backend() {
            Ok(backend) => {
                let arc_backend: Arc<dyn passcore::clipboard::ClipboardBackend + Send + Sync> =
                    Arc::from(backend);
                match passcore::clipboard::copy_and_autoclear(arc_backend, &secret, timeout) {
                    Ok(()) => {
                        self.notice = Some(notice);
                        self.refresh_detail();
                    }
                    Err(e) => {
                        self.notice = Some(format!("copy failed: {e}"));
                        self.refresh_detail();
                    }
                }
            }
            Err(e) => {
                // Headless / no clipboard tool — show error in notice.
                self.notice = Some(format!("clipboard unavailable: {e}"));
                self.refresh_detail();
            }
        }
    }
}

// ── Tree construction ─────────────────────────────────────────────────────────

fn build_store_tree(store: &dyn passcore::PasswordStore) -> tui_realm_treeview::Tree<String> {
    let paths = store.list().unwrap_or_default();
    let nodes = passcore::EntryNode::from_paths(&paths);
    crate::components::tree::build_tree(&nodes)
}

fn first_leaf_id(tree: &tui_realm_treeview::Tree<String>) -> Option<String> {
    // Walk root children depth-first to find the first leaf node.
    first_leaf_in_node(tree.root())
}

fn first_leaf_in_node(node: &tui_realm_treeview::Node<String>) -> Option<String> {
    if node.is_leaf() && !node.id().is_empty() {
        return Some(node.id().to_string());
    }
    for child in node.children() {
        if let Some(id) = first_leaf_in_node(child) {
            return Some(id);
        }
    }
    None
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use passcore::FakeStore;
    use tuirealm::application::Application;
    use tuirealm::event::NoUserEvent;
    use tuirealm::listener::EventListenerCfg;

    /// Build a minimal `Model` backed by `FakeStore` for testing.
    fn test_model(store: FakeStore) -> Model {
        let listener_cfg = EventListenerCfg::<NoUserEvent>::default();
        let app: Application<Id, Msg, NoUserEvent> = Application::init(listener_cfg);
        Model {
            app,
            quit: false,
            redraw: false,
            store: Box::new(store),
            config: passcore::Config::default(),
            selected_path: None,
            detail_entry: None,
            reveal: false,
            notice: None,
        }
    }

    #[test]
    fn select_entry_loads_detail() {
        let mut store = FakeStore::new();
        store.seed("web/github.com", "s3cr3t\nuser: alice\n");
        let mut model = test_model(store);

        model.update(Some(Msg::SelectEntry("web/github.com".to_string())));

        assert_eq!(
            model.selected_path.as_deref(),
            Some("web/github.com"),
            "selected path should be set"
        );
        assert!(
            model.detail_entry.is_some(),
            "detail entry should be loaded"
        );
        assert!(!model.reveal, "reveal should start false");
    }

    #[test]
    fn select_entry_clears_reveal() {
        let mut store = FakeStore::new();
        store.seed("web/a", "pw_a\n");
        store.seed("web/b", "pw_b\n");
        let mut model = test_model(store);

        model.update(Some(Msg::SelectEntry("web/a".to_string())));
        model.reveal = true; // simulate user having toggled reveal

        model.update(Some(Msg::SelectEntry("web/b".to_string())));
        assert!(!model.reveal, "reveal must clear on new selection");
    }

    #[test]
    fn toggle_reveal_flips_flag() {
        let mut store = FakeStore::new();
        store.seed("e", "secret\n");
        let mut model = test_model(store);
        model.update(Some(Msg::SelectEntry("e".to_string())));

        assert!(!model.reveal);
        model.update(Some(Msg::ToggleReveal));
        assert!(model.reveal, "reveal should be true after first toggle");
        model.update(Some(Msg::ToggleReveal));
        assert!(!model.reveal, "reveal should be false after second toggle");
    }

    #[test]
    fn quit_message_sets_quit_flag() {
        let model = &mut test_model(FakeStore::new());
        model.update(Some(Msg::Quit));
        assert!(model.quit);
    }

    #[test]
    fn tick_sets_redraw() {
        let model = &mut test_model(FakeStore::new());
        model.redraw = false;
        model.update(Some(Msg::Tick));
        assert!(model.redraw);
    }

    #[test]
    fn otp_present_on_tick_when_entry_has_uri() {
        let mut store = FakeStore::new();
        // Use RFC 6238 SHA-1 test vector secret — any valid otpauth URI.
        let uri = "otpauth://totp/test?secret=GEZDGNBVGY3TQOJQ";
        store.seed("e/otp", &format!("pw\n{uri}\n"));
        let mut model = test_model(store);

        model.update(Some(Msg::SelectEntry("e/otp".to_string())));
        let entry = model.detail_entry.as_ref().expect("entry should be loaded");
        assert!(entry.otp_uri().is_some(), "entry should have OTP URI");
        // Verify we can compute an OTP code.
        let otp = passcore::otp::current(entry.otp_uri().unwrap());
        assert!(otp.is_ok(), "OTP computation should succeed");
        let otp = otp.unwrap();
        assert_eq!(otp.code.len(), 6, "code should be 6 digits");
    }

    #[test]
    fn copy_without_clipboard_sets_notice() {
        // On a headless CI machine clipboard may not be available.
        // The model should set a notice rather than panic.
        let mut store = FakeStore::new();
        store.seed("e", "secret\n");
        let mut model = test_model(store);
        model.update(Some(Msg::SelectEntry("e".to_string())));
        model.update(Some(Msg::Copy));
        // Either "copied" or an error notice should be set.
        assert!(
            model.notice.is_some(),
            "notice should be set after copy attempt"
        );
    }

    #[test]
    fn password_not_in_notice_on_copy() {
        // The notice must never contain the plaintext password.
        let mut store = FakeStore::new();
        store.seed("e", "v3ryS3cr3t!\n");
        let mut model = test_model(store);
        model.update(Some(Msg::SelectEntry("e".to_string())));
        model.update(Some(Msg::Copy));
        if let Some(notice) = &model.notice {
            assert!(
                !notice.contains("v3ryS3cr3t!"),
                "notice must not contain the plaintext password"
            );
        }
    }

    #[test]
    fn detail_entry_is_none_for_missing_path() {
        let model = &mut test_model(FakeStore::new());
        model.update(Some(Msg::SelectEntry("nonexistent/entry".to_string())));
        assert!(
            model.detail_entry.is_none(),
            "detail_entry should be None for nonexistent path"
        );
    }
}
