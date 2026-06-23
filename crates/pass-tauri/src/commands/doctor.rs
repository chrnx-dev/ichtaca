use serde::Serialize;
use tauri::State;

use crate::state::AppState;

/// Serializable environment-health report returned to the webview so it can
/// render a setup screen when the store is unavailable.
#[derive(Debug, Serialize)]
pub struct DoctorReport {
    pub pass: bool,
    pub gpg: bool,
    /// Whether the store directory exists on disk. Distinct from whether
    /// `AppState` holds a live store (see `init_error`): the directory can be
    /// present while the runtime store failed to initialize, and vice versa.
    pub store_dir_exists: bool,
    pub store_dir: String,
    pub ok: bool,
    pub guidance: String,
    pub demo: bool,
    pub init_error: Option<String>,
}

// ── impl helper (testable without a Tauri runtime) ───────────────────────────

pub fn doctor_impl(state: &AppState) -> DoctorReport {
    let report = passcore::doctor::run(state.config.store_dir.clone());
    let guidance = passcore::doctor::guidance(&report);
    DoctorReport {
        pass: report.pass,
        gpg: report.gpg,
        store_dir_exists: report.store,
        store_dir: report.store_dir.to_string_lossy().into_owned(),
        ok: report.ok(),
        guidance,
        demo: state.demo,
        init_error: state.init_error.clone(),
    }
}

// ── Tauri command wrapper ─────────────────────────────────────────────────────

#[tauri::command]
pub fn doctor(state: State<'_, AppState>) -> DoctorReport {
    doctor_impl(&state)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use passcore::{Config, FakeStore};

    #[test]
    fn doctor_impl_uninitialized_reports_init_error() {
        let config = Config::default();
        let state = AppState::uninitialized("pass not found".to_string(), config);
        let report = doctor_impl(&state);
        assert_eq!(
            report.init_error.as_deref(),
            Some("pass not found"),
            "init_error must be propagated; report: {:?}",
            report
        );
        // The store is absent, so the env check will say store == false.
        assert!(!report.demo, "demo must be false for uninitialized state");
    }

    #[test]
    fn doctor_impl_demo_state_reports_demo_flag() {
        let mut store = FakeStore::new();
        store.seed("demo/entry", "pw\n");
        let config = Config::default();
        let state = AppState::new_demo(Box::new(store), config);
        let report = doctor_impl(&state);
        assert!(report.demo, "demo flag must be true for demo state");
        assert!(
            report.init_error.is_none(),
            "init_error must be None for demo state; got: {:?}",
            report.init_error
        );
    }

    #[test]
    fn doctor_impl_initialized_has_no_init_error() {
        let store = FakeStore::new();
        let config = Config::default();
        let state = AppState::new(Box::new(store), config);
        let report = doctor_impl(&state);
        assert!(
            report.init_error.is_none(),
            "init_error must be None for a live store; got: {:?}",
            report.init_error
        );
        assert!(!report.demo);
    }
}
