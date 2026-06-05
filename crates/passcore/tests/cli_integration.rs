//! Integration tests that drive the real `pass` CLI. Opt-in: they require
//! `pass` and `gpg` plus a throwaway GPG key, so they are `#[ignore]` by default.
//!
//! Run locally with: `cargo test -p passcore -- --ignored`
//!
//! Setup expected by these tests (documented for the runner; not automated):
//!   1. Generate a throwaway GPG key, note its ID.
//!   2. `PASSWORD_STORE_DIR=$tmp pass init <KEY_ID>`
//!   3. `printf 'pw\nuser: bob\n' | PASSWORD_STORE_DIR=$tmp pass insert -m web/example`
//!      Then point PASS_CLIENT_TEST_STORE at $tmp.

use passcore::store::{cli::PassCliStore, PasswordStore};

#[test]
#[ignore = "requires pass + gpg + a prepared store (see file header)"]
fn show_reads_and_parses_a_real_entry() {
    let store_dir = std::env::var("PASS_CLIENT_TEST_STORE")
        .expect("set PASS_CLIENT_TEST_STORE to a prepared store dir");
    let store = PassCliStore::with_store_dir(store_dir.into());
    let entry = store.show("web/example").expect("entry should decrypt");
    assert_eq!(entry.password(), "pw");
    assert_eq!(entry.field("user"), Some("bob"));
}
