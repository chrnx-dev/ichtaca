# Changelog

All notable changes to **Ichtaca** are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/), and the project uses CalVer
(`YY.MM.PATCH`).

## [26.6.0-alpha] - 2026-06-07

First public alpha. Expect rough edges.

### Added

- **`passcore`** — UI-less Rust core: reads a `pass` store, parses entries with a
  byte-exact round-trip (loose schema), wraps the `pass` CLI for reads/writes
  (secrets passed over stdin, never argv), RFC 6238 TOTP, clipboard with
  ownership-checked auto-clear, entry templates, and fuzzy + deep search.
- **`ichtaca`** — terminal UI (Rust + tui-realm): two-pane browse with a tree,
  vim + arrow navigation, entry detail with masked password and on-demand reveal,
  live OTP countdown, copy-to-clipboard, fuzzy search modal, create/edit/delete
  via forms with templates, a CSPRNG password generator, multi-line notes, and
  raw `$EDITOR` editing. "Obsidiana & Oro" theme with Nerd Font icons.
- **`ichtaca-desktop`** — desktop GUI (Tauri 2 + Svelte + Tailwind/DaisyUI):
  the same feature set with the secrets kept in the Rust backend (the webview
  only receives plaintext on an explicit reveal), a hardened CSP, and the shared
  Obsidiana & Oro theme.

### Security

- Decrypted secrets are never written to disk and are zeroized in memory.
- Passwords are masked by default and revealed only on explicit action.
- Clipboard contents auto-clear (default 45s).
- All GPG/pinentry/key handling is delegated to `gpg`/`gpg-agent`.
- No telemetry and no network access (other than `git` if your store uses it).

[26.6.0-alpha]: https://github.com/chrnx-dev/ichtaca/releases/tag/v26.6.0-alpha
