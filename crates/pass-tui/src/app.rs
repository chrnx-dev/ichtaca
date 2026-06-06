//! The runtime: owns state + a store, performs side effects, rebuilds the tree.

use passcore::{Config, EntryNode, PasswordStore};

use crate::action::SideEffect;
use crate::state::{AppState, NoticeKind};

pub struct App {
    pub state: AppState,
    pub store: Box<dyn PasswordStore>,
    pub config: Config,
}

impl App {
    /// Build an app and load the entry tree from the store.
    pub fn new(store: Box<dyn PasswordStore>, config: Config) -> Self {
        let mut state = AppState::new();
        state.reveal = config.ui.reveal_default;
        let mut app = Self {
            state,
            store,
            config,
        };
        app.reload_tree();
        app
    }

    /// Rebuild the tree from the store's flat path list. Errors go to the bar.
    pub fn reload_tree(&mut self) {
        match self.store.list() {
            Ok(paths) => self.state.roots = EntryNode::from_paths(&paths),
            Err(e) => self.state.notify(e.to_string(), NoticeKind::Error),
        }
    }

    /// Perform a side effect produced by `update`. Never panics; errors → bar.
    pub fn perform(&mut self, effect: SideEffect) {
        match effect {
            SideEffect::LoadDetail(path) => match self.store.show(&path) {
                Ok(entry) => {
                    self.state.detail = Some(entry);
                    self.state.detail_path = Some(path);
                    self.state.reveal = self.config.ui.reveal_default;
                }
                Err(e) => self.state.notify(e.to_string(), NoticeKind::Error),
            },
            SideEffect::CopyPassword(path) => self.copy_password(&path),
            SideEffect::Save {
                path,
                contents,
                overwrite,
            } => {
                match self.store.insert(
                    &path,
                    &passcore::Secret::from(contents.as_str()),
                    overwrite,
                ) {
                    Ok(()) => {
                        self.reload_tree();
                        self.state.notify(format!("saved {path}"), NoticeKind::Info);
                    }
                    Err(e) => self.state.notify(e.to_string(), NoticeKind::Error),
                }
            }
            SideEffect::Remove(path) => match self.store.remove(&path) {
                Ok(()) => {
                    self.reload_tree();
                    self.state.detail = None;
                    self.state.detail_path = None;
                    self.state
                        .notify(format!("deleted {path}"), NoticeKind::Info);
                }
                Err(e) => self.state.notify(e.to_string(), NoticeKind::Error),
            },
            SideEffect::Generate { length } => self.generate_into_form(length),
            // RawEdit is handled by the terminal runtime (Task 14), not here.
            SideEffect::RawEdit(_) => {}
        }
    }

    fn copy_password(&mut self, path: &str) {
        match self.store.show_raw(path) {
            Ok(secret) => {
                let timeout = self.config.clipboard.clear_after;
                // Real core API: use `copy_and_autoclear(Arc<backend>, &Secret, Duration)`.
                let backend = match passcore::clipboard::default_backend() {
                    Ok(b) => std::sync::Arc::from(b),
                    Err(e) => {
                        self.state.notify(e.to_string(), NoticeKind::Error);
                        return;
                    }
                };
                match passcore::clipboard::copy_and_autoclear(
                    backend,
                    &secret,
                    std::time::Duration::from_secs(timeout),
                ) {
                    Ok(()) => self
                        .state
                        .notify(format!("copied (clears in {timeout}s)"), NoticeKind::Info),
                    Err(e) => self.state.notify(e.to_string(), NoticeKind::Error),
                }
            }
            Err(e) => self.state.notify(e.to_string(), NoticeKind::Error),
        }
    }

    fn generate_into_form(&mut self, length: usize) {
        // The form must already have a path (create/edit target). The real core
        // `generate(path, len, symbols)` writes the entry via `pass generate` AND
        // returns the new secret; we mirror its first line into the form field.
        let Some(path) = self.state.form.as_ref().map(|f| f.path.clone()) else {
            self.state
                .notify("no form to generate into", NoticeKind::Error);
            return;
        };
        if path.is_empty() {
            self.state
                .notify("name the entry before generating", NoticeKind::Error);
            return;
        }
        match self.store.generate(&path, length, true) {
            Ok(secret) => {
                if let Some(form) = self.state.form.as_mut() {
                    form.password = secret.first_line().to_string();
                    // The entry now exists on disk, so a subsequent Save is an
                    // overwrite: mark the form as editing an existing entry.
                    form.editing = true;
                }
                self.reload_tree();
            }
            Err(e) => self.state.notify(e.to_string(), NoticeKind::Error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::SideEffect;
    use passcore::FakeStore;

    fn app_with_entries() -> App {
        let mut store = FakeStore::new();
        // `FakeStore::seed(path, &str)` is the test helper; the trait `insert`
        // takes `&Secret` + overwrite and is NOT what we want here.
        store.seed("web/github.com", "ghpw\nuser: octocat\n");
        store.seed("email/work", "mailpw\n");
        App::new(Box::new(store), passcore::Config::default())
    }

    #[test]
    fn new_app_loads_the_tree() {
        let app = app_with_entries();
        // two top-level dirs after building the tree
        assert_eq!(app.state.roots.len(), 2);
    }

    #[test]
    fn load_detail_effect_populates_detail() {
        let mut app = app_with_entries();
        app.perform(SideEffect::LoadDetail("web/github.com".into()));
        let e = app.state.detail.as_ref().unwrap();
        assert_eq!(e.password(), "ghpw");
        assert_eq!(app.state.detail_path.as_deref(), Some("web/github.com"));
    }

    #[test]
    fn load_detail_missing_entry_sets_error_notification() {
        let mut app = app_with_entries();
        app.perform(SideEffect::LoadDetail("nope".into()));
        let n = app.state.notification.as_ref().unwrap();
        assert_eq!(n.kind, crate::state::NoticeKind::Error);
        assert!(app.state.detail.is_none());
    }

    #[test]
    fn save_effect_inserts_then_reload_lists_it() {
        let mut app = app_with_entries();
        app.perform(SideEffect::Save {
            path: "web/new.com".into(),
            contents: "newpw\nuser: me\n".into(),
            overwrite: false,
        });
        // After saving, the store lists the new path.
        assert!(app.store.list().unwrap().iter().any(|p| p == "web/new.com"));
    }

    #[test]
    fn remove_effect_deletes_entry() {
        let mut app = app_with_entries();
        app.perform(SideEffect::Remove("email/work".into()));
        assert!(!app.store.list().unwrap().iter().any(|p| p == "email/work"));
    }

    #[test]
    fn generate_effect_fills_form_password() {
        let mut app = app_with_entries();
        // Set up a form with a path so generate has a target.
        app.state.form = Some(crate::form::Form::new_from_template(
            "web/generated",
            crate::form::Template::Login,
        ));
        app.perform(SideEffect::Generate { length: 16 });
        let pw = app.state.form.as_ref().unwrap().password.clone();
        assert_eq!(
            pw.chars().count(),
            16,
            "generated password should be 16 chars"
        );
    }

    #[test]
    fn generate_with_empty_path_emits_error_notification_and_does_not_call_store() {
        let mut app = app_with_entries();
        // Form with an empty path.
        app.state.form = Some(crate::form::Form::new_from_template(
            "",
            crate::form::Template::Blank,
        ));
        app.perform(SideEffect::Generate { length: 16 });
        // Should have an error notification, NOT a generated password.
        let n = app
            .state
            .notification
            .as_ref()
            .expect("expected a notification");
        assert_eq!(n.kind, crate::state::NoticeKind::Error);
        assert!(
            n.text.contains("name the entry"),
            "expected 'name the entry' in error message, got: {}",
            n.text
        );
        // Password field should still be empty (store was not called).
        assert_eq!(app.state.form.as_ref().unwrap().password, "");
    }

    #[test]
    fn load_detail_resets_reveal_to_config_default() {
        let mut app = app_with_entries();
        // Manually reveal, then load a detail — should reset to config default (false).
        app.state.reveal = true;
        app.perform(SideEffect::LoadDetail("web/github.com".into()));
        assert!(
            !app.state.reveal,
            "reveal should reset to config default on load"
        );
    }

    #[test]
    fn remove_clears_detail_state() {
        let mut app = app_with_entries();
        // Load the detail first.
        app.perform(SideEffect::LoadDetail("email/work".into()));
        assert!(app.state.detail.is_some());
        // Now remove it.
        app.perform(SideEffect::Remove("email/work".into()));
        assert!(app.state.detail.is_none());
        assert!(app.state.detail_path.is_none());
    }
}
