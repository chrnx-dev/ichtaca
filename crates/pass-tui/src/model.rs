//! Application model — owns the tui-realm `Application` and all domain state.
//!
//! `Model::new` mounts the Phase-1 components; later phases add more.
//! `Model::view` draws the three-row layout (Header / content / StatusBar).
//! `Model::update` processes messages from the event loop.

use std::sync::Arc;
use std::time::Duration;

use tuirealm::application::Application;
use tuirealm::event::NoUserEvent;
use tuirealm::props::{AttrValue, Attribute};
use tuirealm::ratatui::layout::{Constraint, Direction, Layout, Rect};
use tuirealm::ratatui::Frame;
use tuirealm::terminal::TerminalAdapter;

use crate::components::{
    ConfirmModal, Detail, EntryTree, FormField, FormMode, Header, SearchInput, SearchResults,
    StatusBar, TemplateModal,
};
use crate::id::Id;
use crate::msg::Msg;
use crate::theme;

// ── Overlay state ─────────────────────────────────────────────────────────────

/// Which overlay (if any) is currently displayed on top of the browse layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Overlay {
    None,
    Search,
    /// Template-pick step shown before the Create form.
    TemplatePick,
    /// Create or Edit form.
    Form(FormMode),
    Confirm,
}

// ── FormState ─────────────────────────────────────────────────────────────────

/// All mutable data for the currently-open form.
#[derive(Default)]
pub struct FormState {
    /// Entry path (only editable in Create mode).
    pub path: String,
    pub password: String,
    /// List of (key, value) pairs.
    pub fields: Vec<(String, String)>,
    pub otp: String,
    pub tags: String,
    /// Index of focused field in the form focus chain.
    pub focus_idx: usize,
    /// True when the password input is currently showing the plaintext.
    #[allow(dead_code)]
    pub pw_revealed: bool,
    /// Last error message from a failed insert/save (shown in the form status).
    pub error: Option<String>,
    /// Which template was selected (for Create only).
    #[allow(dead_code)]
    pub template_idx: usize,
}

impl FormState {
    /// Total number of focusable inputs:
    /// path + password + per-field (key + value) × n + otp + tags
    /// For Create mode, an extra row for the template selector (simplified:
    /// we treat template selection as a pre-mount step, not a live field).
    pub fn field_count(&self) -> usize {
        2 + self.fields.len() * 2 + 2
    }
}

// ── Model ─────────────────────────────────────────────────────────────────────

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

    // ── Phase 3 overlay state ────────────────────────────────────────────────
    /// Active overlay kind.
    pub overlay: Overlay,
    /// Form state (only valid while `overlay == Form(_)`).
    pub form: FormState,
    /// Current search results paths (parallel to the list widget).
    pub search_results: Vec<String>,
    /// Current search query.
    pub search_query: String,

    // ── Phase 4: raw-edit suspension ─────────────────────────────────────────
    /// When set, the main loop suspends the TUI, calls `store.edit` on this
    /// path, then restores the TUI.  Set by `Msg::OpenRawEdit`; cleared by the
    /// main loop after the editor returns.
    pub pending_raw_edit: Option<String>,
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
    ///
    /// When an overlay is active it is drawn centred on top of the browse
    /// layout.
    pub fn view<T: TerminalAdapter>(&mut self, terminal: &mut T) {
        let _ = terminal.draw(|f: &mut Frame| {
            Self::render_frame(&mut self.app, f, &self.overlay, &self.form);
        });
    }

    fn render_frame(
        app: &mut Application<Id, Msg, NoUserEvent>,
        f: &mut Frame,
        overlay: &Overlay,
        form: &FormState,
    ) {
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

        // ── Overlay rendering ─────────────────────────────────────────────────
        match overlay {
            Overlay::None => {}

            Overlay::TemplatePick => {
                // Template picker: 55% wide, auto height (uses list component border)
                let popup = centered_rect(area, 55, 55);
                if app.mounted(&Id::FormTemplate) {
                    app.view(&Id::FormTemplate, f, popup);
                }
            }

            Overlay::Search => {
                // Search popup: 60% wide, 50% tall, centred.
                let popup = centered_rect(area, 60, 50);

                // Gold-bordered panel so the modal looks intentional.
                let search_block = tuirealm::ratatui::widgets::Block::default()
                    .style(
                        tuirealm::ratatui::style::Style::default()
                            .bg(theme::SURFACE)
                            .fg(theme::TEXT),
                    )
                    .borders(tuirealm::ratatui::widgets::Borders::ALL)
                    .border_style(tuirealm::ratatui::style::Style::default().fg(theme::GOLD))
                    .border_type(tuirealm::ratatui::widgets::BorderType::Rounded)
                    .title(tuirealm::ratatui::text::Line::from(
                        tuirealm::ratatui::text::Span::styled(
                            " Search  [↑↓ navigate · Enter pick · Esc close] ",
                            tuirealm::ratatui::style::Style::default()
                                .fg(theme::GOLD)
                                .add_modifier(tuirealm::ratatui::style::Modifier::BOLD),
                        ),
                    ));
                let inner = search_block.inner(popup);
                f.render_widget(search_block, popup);

                let parts = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3), // Input field (with its own border)
                        Constraint::Fill(1),   // Results list
                    ])
                    .split(inner);

                if app.mounted(&Id::SearchInput) {
                    app.view(&Id::SearchInput, f, parts[0]);
                }
                if app.mounted(&Id::SearchResults) {
                    app.view(&Id::SearchResults, f, parts[1]);
                }
            }

            Overlay::Form(mode) => {
                // Form popup: 70% wide, 80% tall
                let popup = centered_rect(area, 70, 80);

                // Gold-bordered surface panel for the form.
                let form_title = match mode {
                    FormMode::Create => " New Entry  [Enter save · Esc cancel · Ctrl-g generate] ",
                    FormMode::Edit => " Edit Entry  [Enter save · Esc cancel · Ctrl-g generate] ",
                };
                let popup_block = tuirealm::ratatui::widgets::Block::default()
                    .style(
                        tuirealm::ratatui::style::Style::default()
                            .bg(theme::SURFACE)
                            .fg(theme::TEXT),
                    )
                    .borders(tuirealm::ratatui::widgets::Borders::ALL)
                    .border_style(tuirealm::ratatui::style::Style::default().fg(theme::GOLD))
                    .border_type(tuirealm::ratatui::widgets::BorderType::Rounded)
                    .title(tuirealm::ratatui::text::Line::from(
                        tuirealm::ratatui::text::Span::styled(
                            form_title,
                            tuirealm::ratatui::style::Style::default()
                                .fg(theme::GOLD)
                                .add_modifier(tuirealm::ratatui::style::Modifier::BOLD),
                        ),
                    ));
                let inner_area = popup_block.inner(popup);
                f.render_widget(popup_block, popup);

                // Error banner (1 line) + field rows (3 lines each).
                let num_fields = form.field_count();
                let error_height = if form.error.is_some() { 1u16 } else { 0 };
                let mut constraints: Vec<Constraint> = Vec::new();
                if error_height > 0 {
                    constraints.push(Constraint::Length(error_height));
                }
                constraints.extend((0..num_fields).map(|_| Constraint::Length(3)));
                // Add a spacer so fields don't stretch to fill the panel.
                constraints.push(Constraint::Fill(1));

                let parts = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(constraints)
                    .split(inner_area);

                let field_offset = if error_height > 0 { 1usize } else { 0 };

                // Error banner in cochineal.
                if let Some(err) = &form.error {
                    use tuirealm::ratatui::text::{Line, Span};
                    use tuirealm::ratatui::widgets::Paragraph;
                    let err_widget = Paragraph::new(Line::from(Span::styled(
                        format!(" ⚠  {err}"),
                        tuirealm::ratatui::style::Style::default()
                            .fg(theme::COCHINEAL)
                            .add_modifier(tuirealm::ratatui::style::Modifier::BOLD),
                    )));
                    f.render_widget(err_widget, parts[0]);
                }

                // Render path field (always index 0)
                if app.mounted(&Id::FormField(0)) {
                    app.view(&Id::FormField(0), f, parts[field_offset]);
                }
                // Render password field (always index 1)
                if app.mounted(&Id::FormField(1)) {
                    app.view(&Id::FormField(1), f, parts[field_offset + 1]);
                }
                // Render key/value pairs (fields 2..2+n*2)
                let base = 2usize;
                for i in 0..form.fields.len() {
                    let key_idx = base + i * 2;
                    let val_idx = base + i * 2 + 1;
                    let key_part = field_offset + key_idx;
                    let val_part = field_offset + val_idx;
                    if app.mounted(&Id::FormField(key_idx)) && key_part < parts.len() {
                        app.view(&Id::FormField(key_idx), f, parts[key_part]);
                    }
                    if app.mounted(&Id::FormField(val_idx)) && val_part < parts.len() {
                        app.view(&Id::FormField(val_idx), f, parts[val_part]);
                    }
                }
                // OTP and tags
                let otp_idx = base + form.fields.len() * 2;
                let tags_idx = otp_idx + 1;
                let otp_part = field_offset + otp_idx;
                let tags_part = field_offset + tags_idx;
                if app.mounted(&Id::FormField(otp_idx)) && otp_part < parts.len() {
                    app.view(&Id::FormField(otp_idx), f, parts[otp_part]);
                }
                if app.mounted(&Id::FormField(tags_idx)) && tags_part < parts.len() {
                    app.view(&Id::FormField(tags_idx), f, parts[tags_part]);
                }
            }

            Overlay::Confirm => {
                // Confirm dialog: 50% wide, 6 rows
                let popup = centered_rect_fixed(area, 55, 7);
                if app.mounted(&Id::ConfirmDialog) {
                    app.view(&Id::ConfirmDialog, f, popup);
                }
            }
        }
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

            // ── Phase 3: Search ───────────────────────────────────────────────
            Some(Msg::OpenSearch) => {
                self.open_search();
                self.redraw = true;
                None
            }

            Some(Msg::SearchChanged(q)) => {
                self.search_query = q.clone();
                let paths = self.store.list().unwrap_or_default();
                let hits = passcore::fuzzy_paths(&q, &paths);
                let result_paths: Vec<String> = hits.into_iter().map(|h| h.path).collect();
                self.search_results = result_paths.clone();
                // Remount SearchResults with the current filtered paths so that
                // the internal `paths` vec (used by `selected_path()`) stays in
                // sync with what is displayed.  A plain `app.attr` call only
                // updates the displayed text lines — it does NOT update the
                // component's `paths` field — so index-based path lookup would
                // silently return the wrong entry after any filtering step.
                if self.app.mounted(&Id::SearchResults) {
                    let mut results = SearchResults::default();
                    results.set_paths(result_paths);
                    let _ = self
                        .app
                        .remount(Id::SearchResults, Box::new(results), vec![]);
                }
                self.redraw = true;
                None
            }

            Some(Msg::SearchPick(path)) => {
                self.close_overlay();
                self.load_detail(&path);
                self.reveal = false;
                self.notice = None;
                // Activate tree so normal browse keys work again
                let _ = self.app.active(&Id::Tree);
                self.redraw = true;
                None
            }

            Some(Msg::CloseOverlay) => {
                self.close_overlay();
                let _ = self.app.active(&Id::Tree);
                self.redraw = true;
                None
            }

            // ── Phase 3: Form focus ───────────────────────────────────────────
            Some(Msg::FormFocusNext) => {
                if self.overlay == Overlay::Search {
                    // Move focus to the results list (input → results)
                    if self.app.mounted(&Id::SearchResults) {
                        let _ = self.app.active(&Id::SearchResults);
                    }
                } else {
                    self.advance_form_focus(1);
                }
                self.redraw = true;
                None
            }

            Some(Msg::FormFocusPrev) => {
                if self.overlay == Overlay::Search {
                    // Switch back to SearchInput
                    if self.app.mounted(&Id::SearchInput) {
                        let _ = self.app.active(&Id::SearchInput);
                    }
                } else {
                    self.advance_form_focus(-1);
                }
                self.redraw = true;
                None
            }

            // ── Phase 4: Create — open template picker first ─────────────────
            Some(Msg::OpenCreate) => {
                self.open_template_pick();
                self.redraw = true;
                None
            }

            // Template selected from the picker — open the create form.
            Some(Msg::SelectTemplate(idx)) => {
                self.close_overlay(); // close template picker
                self.open_create_form_with_template(idx);
                self.redraw = true;
                None
            }

            // ── Phase 3: Edit ─────────────────────────────────────────────────
            Some(Msg::OpenEdit) => {
                if let Some(path) = self.selected_path.clone() {
                    self.open_edit_form(&path);
                }
                self.redraw = true;
                None
            }

            // ── Phase 4: Raw edit — set pending flag so the main loop can
            // suspend the TUI before handing control to $EDITOR.
            Some(Msg::OpenRawEdit) => {
                if let Some(path) = self.selected_path.clone() {
                    // Signal the main loop; actual store.edit happens there
                    // after terminal raw-mode is disabled.
                    self.pending_raw_edit = Some(path);
                    // Don't set redraw here — main loop will force a full
                    // redraw after the editor returns.
                }
                None
            }

            // ── Phase 3: Form submit ──────────────────────────────────────────
            Some(Msg::SubmitForm) => {
                self.collect_form_values();
                match &self.overlay {
                    Overlay::Form(FormMode::Create) => {
                        let result = self.save_create();
                        if let Err(e) = result {
                            self.form.error = Some(e);
                            // Do NOT close overlay — let user fix the error.
                        } else {
                            self.close_overlay();
                            self.reload_tree();
                            let _ = self.app.active(&Id::Tree);
                        }
                    }
                    Overlay::Form(FormMode::Edit) => {
                        // Capture the original path before save_edit (which
                        // reads self.selected_path) and before close_overlay
                        // (which resets form state).
                        let original_path = self.selected_path.clone();
                        let result = self.save_edit();
                        if let Err(e) = result {
                            self.form.error = Some(e);
                        } else {
                            self.close_overlay();
                            self.reload_tree();
                            if let Some(path) = original_path {
                                self.load_detail(&path);
                            }
                            let _ = self.app.active(&Id::Tree);
                        }
                    }
                    _ => {}
                }
                self.redraw = true;
                None
            }

            // ── Phase 3: Generate ─────────────────────────────────────────────
            Some(Msg::Generate) => {
                let pw = crate::domain::generate_password(20, true);
                // Set the password in the form state and update the mounted widget.
                self.form.password = pw.clone();
                // Update the mounted password field (index 1)
                if self.app.mounted(&Id::FormField(1)) {
                    let _ =
                        self.app
                            .attr(&Id::FormField(1), Attribute::Value, AttrValue::String(pw));
                }
                self.redraw = true;
                None
            }

            // ── Phase 3: Delete ───────────────────────────────────────────────
            Some(Msg::AskDelete) => {
                if let Some(path) = self.selected_path.clone() {
                    self.open_confirm_delete(&path);
                    self.redraw = true;
                }
                None
            }

            Some(Msg::ConfirmDelete(yes)) => {
                if yes {
                    if let Some(path) = self.selected_path.clone() {
                        match self.store.remove(&path) {
                            Ok(()) => {
                                self.notice = Some(format!("deleted {path}"));
                                self.selected_path = None;
                                self.detail_entry = None;
                                self.push_detail_clear();
                                self.reload_tree();
                            }
                            Err(e) => {
                                self.notice = Some(format!("delete failed: {e}"));
                            }
                        }
                    }
                }
                self.close_overlay();
                let _ = self.app.active(&Id::Tree);
                self.redraw = true;
                None
            }

            // Tree-navigation messages — handled entirely by the EntryTree
            // component; the model just redraws.
            Some(Msg::MoveUp) | Some(Msg::MoveDown) | Some(Msg::Fold) | Some(Msg::Unfold) => {
                self.redraw = true;
                None
            }
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
        let text_val = AttrValue::Text(tuirealm::props::TextStatic::from(lines));
        let _ = self.app.attr(&Id::Detail, Attribute::Text, text_val);
    }

    /// Clear the detail panel (e.g. when a directory is selected).
    fn push_detail_clear(&mut self) {
        use tuirealm::props::{AttrValue, TextStatic};
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

    // ── Overlay management ────────────────────────────────────────────────────

    fn open_search(&mut self) {
        // Lock global subscriptions so Esc/'q' no longer fire while the modal
        // is open.  The active modal component still receives its keys directly.
        self.app.lock_subs();

        // Populate initial results (all paths)
        let paths = self.store.list().unwrap_or_default();
        let hits = passcore::fuzzy_paths("", &paths);
        let result_paths: Vec<String> = hits.into_iter().map(|h| h.path).collect();
        self.search_results = result_paths.clone();
        self.search_query = String::new();
        self.overlay = Overlay::Search;

        // Mount components if not already mounted
        if !self.app.mounted(&Id::SearchInput) {
            self.app
                .mount(Id::SearchInput, Box::new(SearchInput::default()), vec![])
                .expect("mount SearchInput");
        }
        if !self.app.mounted(&Id::SearchResults) {
            let mut results = SearchResults::default();
            results.set_paths(result_paths);
            self.app
                .mount(Id::SearchResults, Box::new(results), vec![])
                .expect("mount SearchResults");
        } else {
            // Refresh results in the existing widget (use owned String → Line<'static>)
            use tuirealm::props::{PropPayload, PropValue};
            let rows: Vec<tuirealm::props::LineStatic> = self
                .search_results
                .iter()
                .map(|p| tuirealm::props::LineStatic::from(p.clone()))
                .collect();
            let _ = self.app.attr(
                &Id::SearchResults,
                Attribute::Text,
                AttrValue::Payload(PropPayload::Vec(
                    rows.into_iter().map(PropValue::TextLine).collect(),
                )),
            );
        }
        let _ = self.app.active(&Id::SearchInput);
    }

    fn close_overlay(&mut self) {
        match &self.overlay {
            Overlay::Search => {
                let _ = self.app.umount(&Id::SearchInput);
                let _ = self.app.umount(&Id::SearchResults);
            }
            Overlay::TemplatePick => {
                let _ = self.app.umount(&Id::FormTemplate);
            }
            Overlay::Form(_) => {
                // Unmount all form fields
                let count = self.form.field_count();
                for i in 0..count {
                    let _ = self.app.umount(&Id::FormField(i));
                }
            }
            Overlay::Confirm => {
                let _ = self.app.umount(&Id::ConfirmDialog);
            }
            Overlay::None => {}
        }
        self.overlay = Overlay::None;
        self.form = FormState::default();
        // Restore global subscriptions now that no modal is active.
        self.app.unlock_subs();
    }

    /// Open the template-pick modal (step 1 of create).
    fn open_template_pick(&mut self) {
        // Lock global subscriptions while the modal is visible.
        self.app.lock_subs();

        let templates = passcore::Template::resolve(&self.config);
        if !self.app.mounted(&Id::FormTemplate) {
            self.app
                .mount(
                    Id::FormTemplate,
                    Box::new(TemplateModal::new(&templates)),
                    vec![],
                )
                .expect("mount TemplateModal");
        }
        self.overlay = Overlay::TemplatePick;
        let _ = self.app.active(&Id::FormTemplate);
    }

    /// Open the create form pre-filled with the template at `tpl_idx`.
    ///
    /// Note: `open_template_pick` already locked subs; `close_overlay` unlocks
    /// them before `SelectTemplate` calls this.  We therefore lock again here
    /// for the create-form phase.
    fn open_create_form_with_template(&mut self, tpl_idx: usize) {
        // Lock subscriptions for the create-form overlay.
        self.app.lock_subs();
        let templates = passcore::Template::resolve(&self.config);
        let tpl = templates
            .get(tpl_idx)
            .cloned()
            .unwrap_or_else(passcore::Template::default_template);
        // Build initial fields from the template
        let fields: Vec<(String, String)> = tpl
            .fields
            .iter()
            .map(|k| (k.clone(), String::new()))
            .collect();
        self.form = FormState {
            path: String::new(),
            password: String::new(),
            fields,
            otp: String::new(),
            tags: String::new(),
            focus_idx: 0,
            pw_revealed: false,
            error: None,
            template_idx: tpl_idx,
        };
        self.overlay = Overlay::Form(FormMode::Create);
        self.mount_form_fields(FormMode::Create);
    }

    fn open_edit_form(&mut self, path: &str) {
        // Lock subscriptions while the edit form is open.
        self.app.lock_subs();

        match self.store.show(path) {
            Ok(entry) => {
                let fields = entry.fields();
                let otp = entry.otp_uri().unwrap_or("").to_string();
                let tags = entry.tags().join(" ");
                self.form = FormState {
                    path: path.to_string(),
                    password: entry.password().to_string(),
                    fields,
                    otp,
                    tags,
                    focus_idx: 0,
                    pw_revealed: false,
                    error: None,
                    template_idx: 0,
                };
                self.overlay = Overlay::Form(FormMode::Edit);
                self.mount_form_fields(FormMode::Edit);
            }
            Err(e) => {
                // Failed to open — no overlay will be shown, so unlock subs.
                self.app.unlock_subs();
                self.notice = Some(format!("cannot open edit form: {e}"));
                self.refresh_detail();
            }
        }
    }

    fn mount_form_fields(&mut self, mode: FormMode) {
        let path_label = if mode == FormMode::Create {
            "Path (e.g. web/github.com)"
        } else {
            "Path (read-only)"
        };

        // 0: Path
        let path_field = FormField::new(path_label, &self.form.path.clone(), false);
        self.app
            .mount(Id::FormField(0), Box::new(path_field), vec![])
            .expect("mount FormField(0)");

        // 1: Password
        let pw_field = FormField::new(
            "Password  [Ctrl-g generate]",
            &self.form.password.clone(),
            true,
        );
        self.app
            .mount(Id::FormField(1), Box::new(pw_field), vec![])
            .expect("mount FormField(1)");

        // 2..: Key/Value pairs
        let base = 2usize;
        for (i, (k, v)) in self.form.fields.clone().iter().enumerate() {
            let key_idx = base + i * 2;
            let val_idx = key_idx + 1;
            let key_field = FormField::new(&format!("Key {}", i + 1), k, false);
            self.app
                .mount(Id::FormField(key_idx), Box::new(key_field), vec![])
                .expect("mount key field");
            let val_field = FormField::new(&format!("Value {}", i + 1), v, false);
            self.app
                .mount(Id::FormField(val_idx), Box::new(val_field), vec![])
                .expect("mount value field");
        }

        // OTP
        let otp_idx = base + self.form.fields.len() * 2;
        let otp_field = FormField::new("OTP URI (otpauth://...)", &self.form.otp.clone(), false);
        self.app
            .mount(Id::FormField(otp_idx), Box::new(otp_field), vec![])
            .expect("mount OTP field");

        // Tags
        let tags_idx = otp_idx + 1;
        let tags_field = FormField::new("Tags (space-separated)", &self.form.tags.clone(), false);
        self.app
            .mount(Id::FormField(tags_idx), Box::new(tags_field), vec![])
            .expect("mount Tags field");

        // Activate first field
        let _ = self.app.active(&Id::FormField(0));
    }

    fn open_confirm_delete(&mut self, path: &str) {
        // Lock subscriptions while the confirm dialog is open.
        self.app.lock_subs();

        self.overlay = Overlay::Confirm;
        if !self.app.mounted(&Id::ConfirmDialog) {
            self.app
                .mount(Id::ConfirmDialog, Box::new(ConfirmModal::new(path)), vec![])
                .expect("mount ConfirmDialog");
        }
        let _ = self.app.active(&Id::ConfirmDialog);
    }

    fn advance_form_focus(&mut self, delta: i32) {
        let count = self.form.field_count();
        if count == 0 {
            return;
        }
        let new_idx = if delta >= 0 {
            (self.form.focus_idx + delta as usize) % count
        } else {
            (self.form.focus_idx + count - ((-delta) as usize % count)) % count
        };
        self.form.focus_idx = new_idx;
        let _ = self.app.active(&Id::FormField(new_idx));
    }

    /// Collect field values from mounted form widgets into `self.form`.
    fn collect_form_values(&mut self) {
        // path
        if let tuirealm::state::State::Single(tuirealm::state::StateValue::String(v)) = self
            .app
            .state(&Id::FormField(0))
            .unwrap_or(tuirealm::state::State::None)
        {
            self.form.path = v;
        }
        // password
        if let tuirealm::state::State::Single(tuirealm::state::StateValue::String(v)) = self
            .app
            .state(&Id::FormField(1))
            .unwrap_or(tuirealm::state::State::None)
        {
            self.form.password = v;
        }
        // key/value pairs
        let base = 2usize;
        for i in 0..self.form.fields.len() {
            let key_idx = base + i * 2;
            let val_idx = key_idx + 1;
            let key = match self.app.state(&Id::FormField(key_idx)) {
                Ok(tuirealm::state::State::Single(tuirealm::state::StateValue::String(s))) => s,
                _ => self.form.fields[i].0.clone(),
            };
            let val = match self.app.state(&Id::FormField(val_idx)) {
                Ok(tuirealm::state::State::Single(tuirealm::state::StateValue::String(s))) => s,
                _ => self.form.fields[i].1.clone(),
            };
            self.form.fields[i] = (key, val);
        }
        // OTP
        let otp_idx = base + self.form.fields.len() * 2;
        if let Ok(tuirealm::state::State::Single(tuirealm::state::StateValue::String(v))) =
            self.app.state(&Id::FormField(otp_idx))
        {
            self.form.otp = v;
        }
        // Tags
        let tags_idx = otp_idx + 1;
        if let Ok(tuirealm::state::State::Single(tuirealm::state::StateValue::String(v))) =
            self.app.state(&Id::FormField(tags_idx))
        {
            self.form.tags = v;
        }
    }

    /// Build a `Secret` and call `store.insert` for Create.
    fn save_create(&mut self) -> Result<(), String> {
        let path = self.form.path.trim().to_string();
        if path.is_empty() {
            return Err("entry path cannot be empty".to_string());
        }
        let secret = build_secret(&self.form);
        match self.store.insert(&path, &secret, false) {
            Ok(()) => {
                self.selected_path = Some(path.clone());
                self.notice = Some(format!("created {path}"));
                Ok(())
            }
            Err(passcore::PassError::AlreadyExists(_)) => {
                Err(format!("entry '{path}' already exists — use 'e' to edit"))
            }
            Err(e) => Err(format!("insert failed: {e}")),
        }
    }

    /// Load the existing entry, apply form edits, and call `store.insert(..., true)`.
    ///
    /// The target path is always taken from `self.selected_path` (the path that
    /// was active when the edit form was opened), **not** from the form's path
    /// widget.  This prevents a user who edits the path field in the UI from
    /// accidentally overwriting a different entry.
    fn save_edit(&mut self) -> Result<(), String> {
        // Use the original path that was active when the edit form was opened.
        let path = self
            .selected_path
            .clone()
            .ok_or_else(|| "no entry selected for edit".to_string())?;
        let mut entry = self
            .store
            .show(&path)
            .map_err(|e| format!("load failed: {e}"))?;

        // Apply changes
        entry.set_password(&self.form.password);
        // Gather current field keys from the entry (before edits) so we can
        // detect deletions.
        let old_keys: Vec<String> = entry.field_keys();
        let new_keys: std::collections::HashSet<String> =
            self.form.fields.iter().map(|(k, _)| k.clone()).collect();
        for k in &old_keys {
            if !new_keys.contains(k) {
                entry.remove_field(k);
            }
        }
        for (k, v) in &self.form.fields {
            if !k.is_empty() {
                entry.set_field(k, v);
            }
        }
        let otp = if self.form.otp.is_empty() {
            None
        } else {
            Some(self.form.otp.as_str())
        };
        entry.set_otp(otp);
        let tags: Vec<String> = self
            .form
            .tags
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        entry.set_tags(&tags);

        let serialized = entry.serialize();
        let secret = passcore::Secret::from(serialized.as_str());
        self.store
            .insert(&path, &secret, true)
            .map_err(|e| format!("save failed: {e}"))?;

        self.notice = Some(format!("saved {path}"));
        Ok(())
    }

    /// Call `store.edit` after the main loop has suspended the terminal.
    ///
    /// This must only be called from the main loop, **after** raw mode and
    /// alternate screen have been disabled.  The main loop re-enters them and
    /// calls `force_redraw` afterwards.
    pub fn finish_raw_edit(&mut self, path: &str) {
        match self.store.edit(path) {
            Ok(()) => {
                self.notice = Some(format!("editor closed for {path}"));
                self.load_detail(path);
                self.reload_tree();
            }
            Err(e) => {
                self.notice = Some(format!("editor failed: {e}"));
            }
        }
    }

    /// Reload the tree widget from the store listing.
    fn reload_tree(&mut self) {
        let tree = build_store_tree(self.store.as_ref());
        let initial = self.selected_path.clone().or_else(|| first_leaf_id(&tree));
        let new_comp = EntryTree::new(tree, initial);
        // Remount the tree component with the updated data
        let _ = self.app.umount(&Id::Tree);
        self.app
            .mount(Id::Tree, Box::new(new_comp), vec![])
            .expect("remount Tree");
        let _ = self.app.active(&Id::Tree);
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

// ── Secret building from FormState ────────────────────────────────────────────

/// Build a `Secret` from the form state.
///
/// Format (pass-compatible):
/// ```text
/// <password>
/// <key>: <value>
/// [otpauth://...]
/// [@tag1 @tag2]
/// ```
fn build_secret(form: &FormState) -> passcore::Secret {
    let mut lines = vec![form.password.clone()];
    for (k, v) in &form.fields {
        if !k.is_empty() {
            lines.push(format!("{k}: {v}"));
        }
    }
    if !form.otp.is_empty() {
        lines.push(form.otp.clone());
    }
    if !form.tags.is_empty() {
        let tag_line = form
            .tags
            .split_whitespace()
            .map(|t| format!("@{}", t.trim_start_matches('@')))
            .collect::<Vec<_>>()
            .join(" ");
        lines.push(tag_line);
    }
    let mut text = lines.join("\n");
    text.push('\n');
    passcore::Secret::from(text.as_str())
}

// ── Popup geometry ────────────────────────────────────────────────────────────

/// Return a centred `Rect` that is `pct_x`% of `area.width` and `pct_y`%
/// of `area.height`.
fn centered_rect(area: Rect, pct_x: u16, pct_y: u16) -> Rect {
    let w = area.width * pct_x / 100;
    let h = area.height * pct_y / 100;
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect::new(x, y, w.max(1), h.max(1))
}

/// Return a centred `Rect` of a fixed width × height (in terminal cells).
fn centered_rect_fixed(area: Rect, w: u16, h: u16) -> Rect {
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect::new(x, y, w.min(area.width), h.min(area.height))
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
            overlay: Overlay::None,
            form: FormState::default(),
            search_results: Vec::new(),
            search_query: String::new(),
            pending_raw_edit: None,
        }
    }

    // ── Phase-2 tests (preserved) ─────────────────────────────────────────────

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

    // ── Phase-3: search ───────────────────────────────────────────────────────

    #[test]
    fn search_changed_filters_results() {
        let mut store = FakeStore::new();
        store.seed("web/github.com", "pw\n");
        store.seed("web/gitlab.com", "pw\n");
        store.seed("email/work", "pw\n");
        let mut model = test_model(store);

        model.update(Some(Msg::SearchChanged("github".to_string())));
        // Only github should match
        assert_eq!(
            model.search_results,
            vec!["web/github.com".to_string()],
            "search filter should return only matching paths"
        );
    }

    #[test]
    fn search_empty_query_returns_all() {
        let mut store = FakeStore::new();
        store.seed("web/github.com", "pw\n");
        store.seed("email/work", "pw\n");
        let mut model = test_model(store);

        model.update(Some(Msg::SearchChanged(String::new())));
        assert_eq!(
            model.search_results.len(),
            2,
            "empty query must return all paths"
        );
    }

    #[test]
    fn search_pick_loads_entry_and_closes_overlay() {
        let mut store = FakeStore::new();
        store.seed("web/github.com", "s3cr3t\nuser: alice\n");
        let mut model = test_model(store);

        // Simulate search being open
        model.overlay = Overlay::Search;

        model.update(Some(Msg::SearchPick("web/github.com".to_string())));

        assert_eq!(
            model.selected_path.as_deref(),
            Some("web/github.com"),
            "search pick must load the selected entry"
        );
        assert_eq!(
            model.overlay,
            Overlay::None,
            "overlay must close after pick"
        );
        assert!(
            model.detail_entry.is_some(),
            "entry must be loaded after search pick"
        );
    }

    // ── Phase-3: create ───────────────────────────────────────────────────────

    #[test]
    fn build_secret_creates_correct_entry_text() {
        let form = FormState {
            path: "web/test".to_string(),
            password: "hunter2".to_string(),
            fields: vec![
                ("user".to_string(), "alice".to_string()),
                ("url".to_string(), "example.com".to_string()),
            ],
            otp: String::new(),
            tags: "work personal".to_string(),
            focus_idx: 0,
            pw_revealed: false,
            error: None,
            template_idx: 0,
        };
        let secret = build_secret(&form);
        let text = secret.expose_str().to_string();
        assert!(text.starts_with("hunter2\n"), "password must be first line");
        assert!(text.contains("user: alice"), "user field must be present");
        assert!(
            text.contains("url: example.com"),
            "url field must be present"
        );
        assert!(text.contains("@work"), "@work tag must appear");
        assert!(text.contains("@personal"), "@personal tag must appear");
    }

    #[test]
    fn save_create_inserts_entry_into_store() {
        let store = FakeStore::new();
        let mut model = test_model(store);
        model.form = FormState {
            path: "new/entry".to_string(),
            password: "s3cr3t".to_string(),
            fields: vec![("user".to_string(), "bob".to_string())],
            otp: String::new(),
            tags: String::new(),
            focus_idx: 0,
            pw_revealed: false,
            error: None,
            template_idx: 0,
        };
        let result = model.save_create();
        assert!(result.is_ok(), "create must succeed: {:?}", result);
        let entry = model.store.show("new/entry").unwrap();
        assert_eq!(entry.password(), "s3cr3t", "password must match");
        assert_eq!(entry.field("user"), Some("bob"), "user field must match");
    }

    #[test]
    fn save_create_refuses_empty_path() {
        let store = FakeStore::new();
        let mut model = test_model(store);
        model.form = FormState {
            path: String::new(),
            ..FormState::default()
        };
        let result = model.save_create();
        assert!(result.is_err(), "empty path must fail");
    }

    #[test]
    fn save_create_refuses_duplicate_without_overwrite() {
        let mut store = FakeStore::new();
        store.seed("web/x", "old\n");
        let mut model = test_model(store);
        model.form = FormState {
            path: "web/x".to_string(),
            password: "new".to_string(),
            ..FormState::default()
        };
        let result = model.save_create();
        assert!(result.is_err(), "duplicate create must fail");
        // Original entry must still be intact
        let entry = model.store.show("web/x").unwrap();
        assert_eq!(entry.password(), "old", "original must be unchanged");
    }

    #[test]
    fn template_login_suggests_user_and_url() {
        let templates = passcore::Template::resolve(&passcore::Config::default());
        let login = templates.iter().find(|t| t.name == "Login").unwrap();
        assert!(login.fields.contains(&"user".to_string()));
        assert!(login.fields.contains(&"url".to_string()));
    }

    #[test]
    fn template_blank_has_no_fields() {
        let templates = passcore::Template::resolve(&passcore::Config::default());
        let blank = templates.iter().find(|t| t.name == "Blank").unwrap();
        assert!(blank.fields.is_empty());
    }

    #[test]
    fn open_create_opens_template_picker_first() {
        let model = &mut test_model(FakeStore::new());
        model.update(Some(Msg::OpenCreate));
        assert_eq!(
            model.overlay,
            Overlay::TemplatePick,
            "OpenCreate must show the template-picker overlay"
        );
    }

    #[test]
    fn select_template_login_opens_form_with_user_url_fields() {
        let model = &mut test_model(FakeStore::new());
        // Skip the template picker step: send SelectTemplate(0) = Login directly.
        model.update(Some(Msg::SelectTemplate(0)));
        assert_eq!(
            model.overlay,
            Overlay::Form(FormMode::Create),
            "SelectTemplate must open the create form"
        );
        // Login template fields: user + url
        let keys: Vec<&str> = model.form.fields.iter().map(|(k, _)| k.as_str()).collect();
        assert!(keys.contains(&"user"), "Login template must pre-fill user");
        assert!(keys.contains(&"url"), "Login template must pre-fill url");
    }

    #[test]
    fn select_template_blank_opens_form_with_no_fields() {
        let model = &mut test_model(FakeStore::new());
        // Blank is template index 4 in the default set.
        let templates = passcore::Template::resolve(&passcore::Config::default());
        let blank_idx = templates.iter().position(|t| t.name == "Blank").unwrap();
        model.update(Some(Msg::SelectTemplate(blank_idx)));
        assert_eq!(
            model.overlay,
            Overlay::Form(FormMode::Create),
            "SelectTemplate(Blank) must open the create form"
        );
        assert!(
            model.form.fields.is_empty(),
            "Blank template must produce no pre-filled key fields"
        );
    }

    // ── Phase-3: edit ─────────────────────────────────────────────────────────

    #[test]
    fn save_edit_round_trip_preserves_unknown_line() {
        let mut store = FakeStore::new();
        // Seed an entry with an unknown free-text line.
        store.seed("web/x", "oldpw\nuser: alice\nsome unknown note\n@work\n");
        let mut model = test_model(store);
        // selected_path must be set to the target entry (save_edit reads it).
        model.selected_path = Some("web/x".to_string());
        model.form = FormState {
            path: "web/x".to_string(),
            password: "newpw".to_string(),
            fields: vec![("user".to_string(), "bob".to_string())],
            otp: String::new(),
            tags: "home".to_string(),
            focus_idx: 0,
            pw_revealed: false,
            error: None,
            template_idx: 0,
        };

        let result = model.save_edit();
        assert!(result.is_ok(), "save edit must succeed: {:?}", result);

        let entry = model.store.show("web/x").unwrap();
        assert_eq!(entry.password(), "newpw", "password must be updated");
        assert_eq!(
            entry.field("user"),
            Some("bob"),
            "user field must be updated"
        );
        // The unknown note line must survive the round-trip.
        assert!(
            entry.serialize().contains("some unknown note"),
            "unknown line must be preserved in round-trip"
        );
        // @home tag must be set.
        assert!(
            entry.tags().contains(&"home".to_string()),
            "tag must be updated"
        );
        // @work tag must be gone (replaced by @home).
        assert!(
            !entry.tags().contains(&"work".to_string()),
            "old tag must be replaced"
        );
    }

    #[test]
    fn save_edit_removes_deleted_field() {
        let mut store = FakeStore::new();
        store.seed("web/x", "pw\nuser: alice\nurl: a.com\n");
        let mut model = test_model(store);
        // selected_path must be set to the target entry.
        model.selected_path = Some("web/x".to_string());
        // Edit: keep user, drop url
        model.form = FormState {
            path: "web/x".to_string(),
            password: "pw".to_string(),
            fields: vec![("user".to_string(), "alice".to_string())],
            otp: String::new(),
            tags: String::new(),
            ..FormState::default()
        };
        let result = model.save_edit();
        assert!(result.is_ok());
        let entry = model.store.show("web/x").unwrap();
        assert_eq!(entry.field("url"), None, "deleted field must be gone");
        assert_eq!(entry.field("user"), Some("alice"), "kept field must remain");
    }

    #[test]
    fn save_edit_sets_and_clears_otp() {
        let mut store = FakeStore::new();
        store.seed("web/x", "pw\n");
        let mut model = test_model(store);

        // selected_path must be set to the target entry.
        model.selected_path = Some("web/x".to_string());

        // Set OTP
        model.form = FormState {
            path: "web/x".to_string(),
            password: "pw".to_string(),
            otp: "otpauth://totp/test?secret=GEZDGNBVGY3TQOJQ".to_string(),
            ..FormState::default()
        };
        model.save_edit().unwrap();
        assert!(
            model.store.show("web/x").unwrap().otp_uri().is_some(),
            "OTP URI must be set"
        );

        // Clear OTP (selected_path is still "web/x" from above)
        model.form = FormState {
            path: "web/x".to_string(),
            password: "pw".to_string(),
            otp: String::new(),
            ..FormState::default()
        };
        model.save_edit().unwrap();
        assert!(
            model.store.show("web/x").unwrap().otp_uri().is_none(),
            "OTP URI must be cleared"
        );
    }

    // ── Phase-3: delete ───────────────────────────────────────────────────────

    #[test]
    fn confirm_delete_true_removes_entry() {
        let mut store = FakeStore::new();
        store.seed("web/x", "pw\n");
        let mut model = test_model(store);
        model.selected_path = Some("web/x".to_string());
        model.overlay = Overlay::Confirm;

        model.update(Some(Msg::ConfirmDelete(true)));

        assert!(
            model.store.show("web/x").is_err(),
            "entry must be removed after confirmed delete"
        );
        assert_eq!(
            model.overlay,
            Overlay::None,
            "overlay must close after delete"
        );
        assert_eq!(
            model.selected_path, None,
            "selected path must be cleared after delete"
        );
    }

    #[test]
    fn confirm_delete_false_preserves_entry() {
        let mut store = FakeStore::new();
        store.seed("web/x", "pw\n");
        let mut model = test_model(store);
        model.selected_path = Some("web/x".to_string());
        model.overlay = Overlay::Confirm;

        model.update(Some(Msg::ConfirmDelete(false)));

        assert!(
            model.store.show("web/x").is_ok(),
            "entry must survive a cancelled delete"
        );
        assert_eq!(model.overlay, Overlay::None, "overlay must still close");
    }

    // ── Phase-4: raw-edit suspension ─────────────────────────────────────

    #[test]
    fn open_raw_edit_sets_pending_flag_not_store_mutation() {
        let mut store = FakeStore::new();
        store.seed("web/x", "original\n");
        let mut model = test_model(store);
        model.selected_path = Some("web/x".to_string());

        // OpenRawEdit must set the pending flag, NOT call store.edit.
        model.update(Some(Msg::OpenRawEdit));

        assert_eq!(
            model.pending_raw_edit.as_deref(),
            Some("web/x"),
            "OpenRawEdit must set pending_raw_edit to the selected path"
        );
        // Store must be unchanged (editor has not been called yet).
        let entry = model.store.show("web/x").unwrap();
        assert_eq!(
            entry.password(),
            "original",
            "store must not be mutated when pending flag is set"
        );
    }

    #[test]
    fn open_raw_edit_without_selection_sets_no_flag() {
        let model = &mut test_model(FakeStore::new());
        model.update(Some(Msg::OpenRawEdit));
        assert!(
            model.pending_raw_edit.is_none(),
            "pending_raw_edit must remain None when no entry is selected"
        );
    }

    // ── Phase-3: generate ─────────────────────────────────────────────────────

    #[test]
    fn generate_msg_sets_non_empty_password() {
        let store = FakeStore::new();
        let mut model = test_model(store);
        model.overlay = Overlay::Form(FormMode::Create);
        // No mounted component in unit test; just check form state is set.
        model.update(Some(Msg::Generate));
        assert!(
            !model.form.password.is_empty(),
            "generated password must be non-empty"
        );
        assert_eq!(
            model.form.password.len(),
            20,
            "generated password must have the expected length"
        );
    }

    // ── Security invariants ───────────────────────────────────────────────────

    #[test]
    fn build_secret_password_not_leaked_into_fields() {
        let form = FormState {
            path: "p".to_string(),
            password: "topSecretPassword!".to_string(),
            fields: vec![("user".to_string(), "alice".to_string())],
            ..FormState::default()
        };
        let secret = build_secret(&form);
        let text = secret.expose_str();
        // The password must appear only on line 0
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines[0], "topSecretPassword!", "password on line 0");
        for l in lines.iter().skip(1) {
            assert!(
                !l.contains("topSecretPassword!"),
                "password must not appear in other lines"
            );
        }
    }

    // ── Centered rect helpers ─────────────────────────────────────────────────

    #[test]
    fn centered_rect_pct_is_centered() {
        let area = Rect::new(0, 0, 100, 50);
        let r = centered_rect(area, 60, 80);
        assert_eq!(r.width, 60);
        assert_eq!(r.height, 40);
        assert_eq!(r.x, 20); // (100-60)/2
        assert_eq!(r.y, 5); // (50-40)/2
    }

    #[test]
    fn centered_rect_fixed_is_centered() {
        let area = Rect::new(0, 0, 80, 24);
        let r = centered_rect_fixed(area, 50, 7);
        assert_eq!(r.width, 50);
        assert_eq!(r.height, 7);
        assert_eq!(r.x, 15); // (80-50)/2
        assert_eq!(r.y, 8); // (24-7)/2
    }

    // ── Fix 1: subscription locking while a modal is open ─────────────────────
    //
    // The Application::sub_lock field is private (lives in the tuirealm crate),
    // so we cannot read it directly in tests.  We instead test the observable
    // model behaviour that the locking enables:
    //
    // • While a modal is open, `model.overlay != Overlay::None`.  A Quit
    //   message sent through `update` must not propagate (overlay suppresses
    //   global Esc/'q'; the message is only produced by those subs when the
    //   active component is NOT the modal — which, with locking, never happens).
    //
    // • After CloseOverlay the overlay is None and Quit is processed normally.
    //
    // We test the two simpler invariants: overlay is set on open and cleared on
    // close.  The subs-locked/unlocked invariant is documented here and verified
    // by the fact that `lock_subs`/`unlock_subs` are called alongside every
    // overlay open/close (auditable in the source of open_* / close_overlay).

    /// Opening the search overlay sets `overlay = Search`.
    #[test]
    fn open_search_sets_overlay() {
        let mut store = FakeStore::new();
        store.seed("web/a", "pw\n");
        let mut model = test_model(store);
        // Mount Phase-1 and Phase-2 so open_search can mount search components.
        model.mount_phase1();
        model.mount_phase2();

        model.update(Some(Msg::OpenSearch));

        assert_eq!(
            model.overlay,
            Overlay::Search,
            "overlay must be Search after OpenSearch"
        );
        // Subscriptions are locked alongside the overlay being set.
        // (lock_subs() is called unconditionally inside open_search)
    }

    /// Closing the search overlay resets `overlay` to `None`.
    /// (unlock_subs() is called by close_overlay alongside the overlay reset.)
    #[test]
    fn close_overlay_resets_overlay() {
        let mut store = FakeStore::new();
        store.seed("web/a", "pw\n");
        let mut model = test_model(store);
        model.mount_phase1();
        model.mount_phase2();

        model.update(Some(Msg::OpenSearch));
        assert_eq!(model.overlay, Overlay::Search);

        model.update(Some(Msg::CloseOverlay));
        assert_eq!(
            model.overlay,
            Overlay::None,
            "overlay must be None after CloseOverlay"
        );
        // unlock_subs() is called by close_overlay — global subs resume.
    }

    /// `Msg::Quit` is processed directly (not via global subs) and must
    /// always set the quit flag regardless of overlay state.
    #[test]
    fn quit_not_suppressed_when_no_overlay() {
        let mut model = test_model(FakeStore::new());
        assert_eq!(model.overlay, Overlay::None);
        model.update(Some(Msg::Quit));
        assert!(model.quit, "Quit must work when no overlay is open");
    }

    /// Template-pick overlay also locks subs.
    #[test]
    fn open_template_pick_sets_overlay() {
        let mut model = test_model(FakeStore::new());
        model.mount_phase1();
        model.update(Some(Msg::OpenCreate));
        assert_eq!(
            model.overlay,
            Overlay::TemplatePick,
            "OpenCreate must show template-pick overlay"
        );
        // lock_subs() is called by open_template_pick alongside setting overlay.
    }

    /// Confirm-delete overlay also locks subs.
    #[test]
    fn open_confirm_delete_sets_overlay() {
        let mut store = FakeStore::new();
        store.seed("web/a", "pw\n");
        let mut model = test_model(store);
        model.mount_phase1();
        model.mount_phase2();
        model.selected_path = Some("web/a".to_string());

        model.update(Some(Msg::AskDelete));
        assert_eq!(
            model.overlay,
            Overlay::Confirm,
            "AskDelete must show the confirm overlay"
        );
        // lock_subs() is called by open_confirm_delete alongside setting overlay.
    }

    // ── Fix 2: search result paths stay in sync with filter ───────────────────

    /// After filtering, `search_results` contains only the filtered paths and
    /// the SearchResults component is remounted with the same filtered set so
    /// that index-based `selected_path()` lookups are correct.
    #[test]
    fn search_changed_syncs_result_paths() {
        let mut store = FakeStore::new();
        store.seed("web/github.com", "pw\n");
        store.seed("web/gitlab.com", "pw\n");
        store.seed("email/work", "pw\n");
        let mut model = test_model(store);
        model.mount_phase1();
        model.mount_phase2();

        // Open search then filter to only github
        model.update(Some(Msg::OpenSearch));
        model.update(Some(Msg::SearchChanged("github".to_string())));

        assert_eq!(
            model.search_results,
            vec!["web/github.com".to_string()],
            "model.search_results must contain only the filtered path"
        );
    }

    /// Filtering to 2 results then selecting index 1 must pick the correct
    /// (filtered) path, not an entry from the original unfiltered list.
    #[test]
    fn search_filter_selection_maps_to_correct_filtered_entry() {
        let mut store = FakeStore::new();
        store.seed("web/github.com", "gh_pw\n");
        store.seed("web/gitlab.com", "gl_pw\n");
        store.seed("email/work", "ew_pw\n");
        let mut model = test_model(store);
        model.mount_phase1();
        model.mount_phase2();

        // Open search and filter to the two "web/git*" entries.
        model.update(Some(Msg::OpenSearch));
        model.update(Some(Msg::SearchChanged("git".to_string())));

        // After filtering the model must hold exactly two paths.
        assert_eq!(
            model.search_results.len(),
            2,
            "filter 'git' should produce 2 results"
        );

        // Simulate picking index 1 (the second filtered entry) via SearchPick.
        let second_path = model.search_results[1].clone();
        model.update(Some(Msg::SearchPick(second_path.clone())));

        // The loaded entry must match the second filtered path.
        assert_eq!(
            model.selected_path.as_deref(),
            Some(second_path.as_str()),
            "SearchPick must load the correct filtered entry"
        );
        assert!(
            model.detail_entry.is_some(),
            "detail must be loaded for the picked entry"
        );
    }

    // ── Fix 3: edit save always targets the original entry path ───────────────

    /// Opening an edit form then saving with a modified path field in `form`
    /// must update the *original* entry (A) and leave entry B untouched.
    #[test]
    fn save_edit_ignores_form_path_uses_selected_path() {
        let mut store = FakeStore::new();
        store.seed("web/entry-a", "pw_a\nuser: alice\n");
        store.seed("web/entry-b", "pw_b\nuser: bob\n");
        let mut model = test_model(store);

        // Select entry-a as the active entry (as if the user navigated to it).
        model.selected_path = Some("web/entry-a".to_string());

        // Set up the form as if edit was opened on entry-a, but the user
        // has modified the form's path field to point at entry-b.
        model.form = FormState {
            path: "web/entry-b".to_string(), // user typed this in the path widget
            password: "new_pw_a".to_string(),
            fields: vec![("user".to_string(), "alice_updated".to_string())],
            otp: String::new(),
            tags: String::new(),
            focus_idx: 0,
            pw_revealed: false,
            error: None,
            template_idx: 0,
        };

        let result = model.save_edit();
        assert!(result.is_ok(), "save_edit must succeed: {:?}", result);

        // Entry A must be updated.
        let entry_a = model.store.show("web/entry-a").unwrap();
        assert_eq!(
            entry_a.password(),
            "new_pw_a",
            "entry-a password must be updated"
        );
        assert_eq!(
            entry_a.field("user"),
            Some("alice_updated"),
            "entry-a user field must be updated"
        );

        // Entry B must be completely untouched.
        let entry_b = model.store.show("web/entry-b").unwrap();
        assert_eq!(
            entry_b.password(),
            "pw_b",
            "entry-b must not be overwritten"
        );
        assert_eq!(
            entry_b.field("user"),
            Some("bob"),
            "entry-b user field must be unchanged"
        );
    }
}
