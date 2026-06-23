//! The single runtime store-initialization path for every frontend.
//!
//! Real apps must never silently fall back to `FakeStore`: a missing `pass`,
//! `gpg`, or store directory becomes a propagated error the caller surfaces.
//! Demo mode is explicit, opt-in via `ICHTACA_DEMO=1`, and flagged so the UI
//! can label it.

use crate::{Config, FakeStore, PassCliStore, PasswordStore, Result};

/// A constructed store plus whether it is the explicit demo fake.
pub struct StoreInit {
    pub store: Box<dyn PasswordStore + Send>,
    pub demo: bool,
}

/// Build the runtime store. Returns `Err` (never a silent fake) when the real
/// environment is not ready. `ICHTACA_DEMO=1` opts into a labeled demo fake.
pub fn init_store(config: &Config) -> Result<StoreInit> {
    if std::env::var("ICHTACA_DEMO").as_deref() == Ok("1") {
        return Ok(StoreInit {
            store: Box::new(FakeStore::demo()),
            demo: true,
        });
    }
    let store = PassCliStore::detect(config.store_dir.clone())?;
    Ok(StoreInit {
        store: Box::new(store),
        demo: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;
    use std::path::PathBuf;

    // NOTE: Both tests mutate the `ICHTACA_DEMO` process env var. They are
    // merged into a single `#[test]` fn so they run sequentially in the same
    // thread, avoiding the race that would occur if cargo ran them concurrently.
    #[test]
    fn runtime_init_store_behavior() {
        // --- Case 1: no demo env, bad store dir → must error (never fall back) ---
        std::env::remove_var("ICHTACA_DEMO");
        let cfg = Config {
            store_dir: Some(PathBuf::from("/nonexistent/ichtaca-test-store")),
            ..Config::default()
        };
        assert!(init_store(&cfg).is_err());

        // --- Case 2: ICHTACA_DEMO=1 → labeled fake store ---
        std::env::set_var("ICHTACA_DEMO", "1");
        let init = init_store(&Config::default()).expect("demo store");
        assert!(init.demo);
        std::env::remove_var("ICHTACA_DEMO");
    }
}
