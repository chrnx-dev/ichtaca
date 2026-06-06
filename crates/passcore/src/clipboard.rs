//! Cross-platform clipboard copy with auto-clear and an ownership check.
//!
//! The OS interaction lives behind `ClipboardBackend` so the copy/clear logic is
//! testable without a real clipboard. The default backend shells out to
//! `pbcopy`/`pbpaste` (macOS) or `wl-copy`/`wl-paste` / `xclip` (Linux).

use std::time::Duration;

use zeroize::Zeroizing;

use crate::error::{PassError, Result};
use crate::secret::Secret;

/// An OS clipboard. Implementors provide set/get/clear.
pub trait ClipboardBackend {
    fn set(&self, value: &str) -> Result<()>;
    fn get(&self) -> Result<String>;
    fn clear(&self) -> Result<()>;
}

/// Copy a secret's contents to the clipboard via `backend`.
pub fn copy_with(backend: &dyn ClipboardBackend, secret: &Secret) -> Result<()> {
    backend.set(secret.expose_str())
}

/// Clear the clipboard only if it still holds `expected` (ownership check),
/// so we never wipe a value the user copied afterwards.
pub fn clear_if_owned(backend: &dyn ClipboardBackend, expected: &str) -> Result<()> {
    if backend.get()? == expected {
        backend.clear()?;
    }
    Ok(())
}

/// Copy a secret, then spawn a thread that clears it after `timeout`, but only
/// if the clipboard still holds our value. Returns immediately. Frontends that
/// manage their own runtime may prefer `copy_with` + their own timer instead.
pub fn copy_and_autoclear(
    backend: std::sync::Arc<dyn ClipboardBackend + Send + Sync>,
    secret: &Secret,
    timeout: Duration,
) -> Result<()> {
    let expected: Zeroizing<String> = Zeroizing::new(secret.expose_str().to_string());
    copy_with(backend.as_ref(), secret)?;
    std::thread::spawn(move || {
        std::thread::sleep(timeout);
        let _ = clear_if_owned(backend.as_ref(), &expected);
    });
    Ok(())
}

/// The platform default backend, selected at runtime.
pub fn default_backend() -> Result<Box<dyn ClipboardBackend + Send + Sync>> {
    #[cfg(target_os = "macos")]
    {
        Ok(Box::new(CommandClipboard {
            copy: ("pbcopy", vec![]),
            paste: ("pbpaste", vec![]),
        }))
    }
    #[cfg(target_os = "linux")]
    {
        if which::which("wl-copy").is_ok() {
            Ok(Box::new(CommandClipboard {
                copy: ("wl-copy", vec![]),
                paste: ("wl-paste", vec!["--no-newline".to_string()]),
            }))
        } else if which::which("xclip").is_ok() {
            Ok(Box::new(CommandClipboard {
                copy: (
                    "xclip",
                    vec!["-selection".to_string(), "clipboard".to_string()],
                ),
                paste: (
                    "xclip",
                    vec![
                        "-selection".to_string(),
                        "clipboard".to_string(),
                        "-o".to_string(),
                    ],
                ),
            }))
        } else {
            Err(PassError::Config(
                "no clipboard tool found (install wl-clipboard or xclip)".into(),
            ))
        }
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        Err(PassError::Config(
            "clipboard unsupported on this platform".into(),
        ))
    }
}

/// A backend that pipes through external commands (`pbcopy`/`wl-copy`/`xclip`).
pub struct CommandClipboard {
    copy: (&'static str, Vec<String>),
    paste: (&'static str, Vec<String>),
}

impl ClipboardBackend for CommandClipboard {
    fn set(&self, value: &str) -> Result<()> {
        use std::io::Write;
        use std::process::{Command, Stdio};
        let mut child = Command::new(self.copy.0)
            .args(&self.copy.1)
            .stdin(Stdio::piped())
            .spawn()?;
        child
            .stdin
            .take()
            .expect("stdin piped")
            .write_all(value.as_bytes())?;
        let status = child.wait()?;
        if status.success() {
            Ok(())
        } else {
            Err(PassError::Config(format!("{} failed", self.copy.0)))
        }
    }

    fn get(&self) -> Result<String> {
        use std::process::Command;
        let out = Command::new(self.paste.0).args(&self.paste.1).output()?;
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }

    fn clear(&self) -> Result<()> {
        self.set("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    /// A fake clipboard backend recording read/write history.
    #[derive(Default)]
    struct FakeClipboard {
        value: RefCell<Option<String>>,
    }

    impl ClipboardBackend for FakeClipboard {
        fn set(&self, value: &str) -> Result<()> {
            *self.value.borrow_mut() = Some(value.to_string());
            Ok(())
        }
        fn get(&self) -> Result<String> {
            Ok(self.value.borrow().clone().unwrap_or_default())
        }
        fn clear(&self) -> Result<()> {
            *self.value.borrow_mut() = Some(String::new());
            Ok(())
        }
    }

    #[test]
    fn copy_sets_the_value() {
        let fake = FakeClipboard::default();
        copy_with(&fake, &Secret::from("hunter2")).unwrap();
        assert_eq!(fake.get().unwrap(), "hunter2");
    }

    #[test]
    fn clear_if_owned_clears_when_value_unchanged() {
        let fake = FakeClipboard::default();
        copy_with(&fake, &Secret::from("hunter2")).unwrap();
        clear_if_owned(&fake, "hunter2").unwrap();
        assert_eq!(fake.get().unwrap(), "");
    }

    #[test]
    fn clear_if_owned_is_noop_when_value_changed() {
        let fake = FakeClipboard::default();
        copy_with(&fake, &Secret::from("hunter2")).unwrap();
        fake.set("something-else").unwrap();
        clear_if_owned(&fake, "hunter2").unwrap();
        // We do not clobber a value we no longer own.
        assert_eq!(fake.get().unwrap(), "something-else");
    }
}
