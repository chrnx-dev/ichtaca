# Security Policy

## Supported Versions

Ichtaca is currently in **alpha**. Only the latest published version receives security attention.

| Version | Supported |
|---------|-----------|
| `26.6.0-alpha.1` (latest) | Yes |
| Older pre-release builds | No |

> **Alpha software disclaimer:** Ichtaca is pre-release software. It has not undergone a formal security audit. Use it at your own risk, and always maintain a backup of your `~/.password-store` and GPG keys.

---

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Use one of the following channels:

1. **GitHub Security Advisories (preferred):** Navigate to the repository on GitHub and click **Security → Report a vulnerability**. This opens a private advisory that only you and the maintainers can see.

2. **Email:** If you cannot use GitHub, contact the maintainer at `diego.resendez@zero-oneit.com`. Use the subject line `[ichtaca] Security Vulnerability` and, if possible, encrypt the message with the maintainer's GPG key.

You can expect an acknowledgement within **5 business days** and a triage update within **10 business days**. When the issue is resolved, you will be credited in the changelog unless you prefer to remain anonymous.

---

## Security Posture

### What Ichtaca does and does not do

- Ichtaca is a **client** for [`pass`](https://www.passwordstore.org/). It does not implement its own encryption, key management, or secret storage.
- All encryption and decryption is performed by **`gpg`/`gpg-agent`** via the `pass` CLI. The security of your secrets ultimately depends on your GPG key and `pass` setup.
- The app **never writes decrypted secrets to disk**. Plaintext exists only in memory for the duration of an operation.

### In-memory secret handling

- Sensitive values are held in a `Secret` type that is **zeroized on drop** (using the [`zeroize`](https://crates.io/crates/zeroize) crate). Bytes are overwritten before the memory is freed.
- `Secret` values are **redacted in `Debug` output** — they appear as `Secret(<redacted>)` and cannot accidentally leak into logs or error messages.

### Clipboard

- Passwords are copied via `pass` (which calls `gpg`). The app never stores the plaintext to disk during a copy.
- After a clipboard copy, a background thread **auto-clears the clipboard** after the configured timeout (default: 45 seconds). The clear is conditional: if the clipboard value has since changed (the user copied something else), it is left untouched.
- On Linux, `wl-clipboard` (`wl-copy`/`wl-paste`) is preferred; `xclip` is the fallback.

### Desktop app (Tauri)

- IPC commands are split by sensitivity:
  - `list`, `search`, `show_meta` — return only metadata (paths, field names, OTP presence flag). No secret material crosses the IPC boundary.
  - `copy_password`, `otp_code` — the Rust backend performs the clipboard write or TOTP computation; the webview receives no plaintext.
  - `reveal_password`, `reveal_otp_uri` — explicitly return plaintext to the UI only when the user requests it.
- A strict **Content Security Policy** is enforced (`default-src 'self'`; no remote scripts; no object/embed; no framing). `unsafe-inline` is permitted only in `style-src` (required by Svelte's scoped styles).
- Remote URL loading is disabled by default in Tauri 2. No `dangerousRemoteUrlLoadingAllowed` override is used.

### Password masking

Passwords are **masked by default** in both apps. They are only revealed on an explicit user action (pressing `s` in the TUI; clicking Reveal in the desktop app).

### No telemetry, no network

Ichtaca makes no outbound network connections. The only network activity that can occur is `git push`/`git pull` if the user's store is configured as a git repository — this is handled entirely by `pass git` and the user's git configuration.

---

## Out of Scope

The following are outside Ichtaca's threat model and are not considered vulnerabilities in this project:

- Attacks against the user's GPG key or `gpg-agent` (report to GnuPG)
- Vulnerabilities in the `pass` CLI itself (report to the `pass` project)
- Attacks requiring local root / physical access to the machine
- OS-level clipboard snooping (a limitation of all clipboard-based password managers)

---

## Dependencies

Ichtaca uses third-party Rust crates and (for the desktop app) npm packages. If you discover a vulnerability in a dependency, please report it upstream and also notify us so we can update.
