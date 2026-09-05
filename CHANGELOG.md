# Changelog

All notable changes to **Ichtaca** are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/), and the project uses CalVer
(`YY.MM.PATCH`).

## [26.9.0-beta.1] - 2026-09-04

Closes the last two feature items on the beta roadmap: git sync and structured TOTP
entry. Everything else on the beta checklist was already done in `26.6.0-beta.1`.

### Added

- **Git sync.** `pass` already commits every write when your store is a git repo;
  Ichtaca adds the half it leaves out — seeing that you have unpushed commits, and
  pushing them.
  - TUI: a right-aligned footer chip (` main ↑2`) and `Ctrl-g` to pull `--rebase`
    then push. Sync suspends the terminal, so SSH passphrase and credential prompts
    are visible and answerable, exactly as with plain `git`.
  - Desktop: the same chip in the status bar; click it to sync. Prompts are disabled
    (a webview has no terminal), so auth that needs one fails fast and tells you to
    run `ichtaca git push` from a shell.
  - CLI: `ichtaca git [status|pull|push]`. `status` is local-only.
  - `ichtaca doctor` reports the store's git state, or prints the two commands that
    enable sync when the store has no repo.
  - **When the store is not a git repo, none of this appears** — no chip, no key,
    no error. The `↓` behind count comes from local refs, so it is only as fresh as
    your last pull; nothing ever fetches on its own.
- **Structured TOTP entry.** Adding a one-time code no longer means hand-writing an
  `otpauth://` URI.
  - The OTP field in both apps accepts either the **secret key** a site prints next
    to its QR code — spaces, dashes, padding and lowercase all tolerated — or a full
    `otpauth://totp/…` URI, which is kept verbatim (unknown parameters survive an edit).
  - A bare secret is wrapped into a URI labelled with the entry's own name and its
    `user` field, so the account still identifies itself if imported elsewhere.
  - Invalid input is refused at save time with the reason. Previously a bad secret was
    stored happily and only failed later when you asked for a code.
  - The desktop form previews what the backend understood
    (`GitHub (alice) · 6 digits · 30s · SHA1`) as you type.
  - CLI: `ichtaca set <path> --otp <secret-or-uri>` and `--remove-otp`.
  - Defaults follow RFC 6238 (SHA-1, 6 digits, 30s); non-default values come through
    a pasted URI.

### Fixed

- **TUI repainted nothing after returning from a suspension.** Coming back from a raw
  `$EDITOR` edit (`E`) left the screen blank: ratatui diffs each frame against its own
  buffer, which still held the pre-suspend content, so the redraw was a no-op.
- **Creating an entry showed the previous entry's fields.** `save_create` moved the
  selection to the new entry but nothing reloaded the detail panel, so the new title
  appeared over stale data — it looked as though the fields had not been saved.
- OTP validation errors no longer read "could not parse entry: …". The entry parsed
  fine; the OTP did not.

### Beta stability notes

**Stable in this release:** everything listed under `26.6.0-beta.1`, plus git sync and
the OTP input contract (a bare base32 secret is accepted anywhere a URI is).

**Known rough edges:**
- macOS desktop builds are not yet code-signed or notarized; Gatekeeper blocks launch
  on first open (right-click → Open, or `xattr -dr com.apple.quarantine`).
- Git merge conflicts are not handled in-app by design: a failed pull stops before the
  push and leaves the store for you to resolve with `pass git`.

---

## [26.6.0-beta.1] - 2026-06-23

### Added

- **No-silent-fake-store safety (P0):** Launching the TUI or desktop without `pass`,
  `gpg`, or an initialised store no longer opens an empty fake vault.
  - `ichtaca` (TUI): prints actionable setup guidance to stderr and exits non-zero.
  - `ichtaca-desktop`: shows a first-run **Setup screen** with the same guidance
    (checklist of missing prerequisites + next steps).
- **Demo mode:** `ICHTACA_DEMO=1 ichtaca` / `ICHTACA_DEMO=1 ichtaca-desktop` launches
  an explicit, labeled demo session seeded with obviously-fake entries (`demo/example.com`,
  `demo/github.com`). The TUI shows a `· DEMO` marker; the desktop shows a `DEMO` badge.
  This is not your real data and is not connected to any password store.
- **`ichtaca` CLI subcommands:** bare `ichtaca` still launches the TUI; a subcommand
  runs non-interactively and exits.
  - `ichtaca doctor` — environment check (pass / gpg / store); exits 0 when ready, 2 when not.
  - `ichtaca list` — print all entry paths.
  - `ichtaca search <query>` — fuzzy path search.
  - `ichtaca get <path>` — print the password to stdout (no trailing newline; pipe-safe).
  - `ichtaca show <path> [--json]` — metadata only (path, fields, tags, `has_otp`); never prints the password or OTP URI.
  - `ichtaca otp <path>` — print the current TOTP code.
  - `ichtaca copy <path>` — copy the password to clipboard, then clear it after the configured `clear_after` timeout (blocks until cleared; Ctrl-C to keep). If `clear_after` is 0, copies and returns immediately without clearing.
  - `ichtaca generate <path> [--length N] [--no-symbols]` — generate and store a password; refuses if the entry already exists (default length 32).
  - `ichtaca set <path> [--password-stdin] [--field key=value ...] [--tag T ...] [--remove-field K ...] [--remove-tag T ...]` — create/update an entry; reads password from stdin only; read-modify-write preserves existing OTP, tags, and fields. Add tags with `--tag`; remove fields or tags with `--remove-field` / `--remove-tag`.
- **Stable exit codes:** `0` success · `1` user/input error · `2` missing dependency or
  store · `3` store/decrypt failure.

### Beta stability notes

**Stable in this release:**
- TUI and desktop read + write (browse, copy, OTP, create, edit, delete, search).
- Full CLI subcommand surface (`doctor`, `list`, `search`, `get`, `show`, `otp`, `copy`, `generate`, `set`).
- Exit-code contract (will not change without a version note).

**Known rough edges:**
- macOS desktop builds are not yet code-signed or notarized; Gatekeeper blocks launch on first open (right-click → Open, or `xattr -dr com.apple.quarantine`).
- No structured TOTP editor in either UI — OTP URIs are stored as raw text fields.

---

## [26.6.0-alpha.2] - 2026-06-22

### Added

- TUI entry forms can add custom fields with `Ctrl-a`.
- TUI entry forms can edit custom field labels and values with `Ctrl-e`.

### Changed

- TUI entry serialization keeps custom fields before notes.

### Fixed

- TUI entry forms can remove custom fields with `Ctrl-d` while keeping fixed
  fields such as password and OTP protected.
- Desktop edit-form regression coverage now verifies custom fields are
  preserved.

## [26.6.0-alpha.1] - 2026-06-09

### Added

- **Configurable password generator** — `[generator]` section in
  `~/.config/pass-client/config.toml` (`length`, `symbols`). The generator now
  lives in `passcore` (shared CSPRNG); the desktop app generates in the Rust
  backend instead of the webview.
- **Content / metadata search** — search inside entries (fields, notes, tags),
  not just paths. TUI: `Ctrl-f` in the search modal. Desktop: a "search inside
  entries" toggle. It decrypts (GPG), so it's an explicit, on-demand mode.
- **Project logo and app icons** — an obsidian-mirror-and-keyhole mark across the
  desktop app icon (incl. `.icns`/`.ico`), the website, and the README.

### Fixed

- Desktop **copy** now copies only the password (first line), not the whole entry.
- **Empty entry paths are rejected** (`insert`/`generate`) and stray empty-stem
  `.gpg` files are skipped — prevents a corrupt entry that broke `pass ls`.
- TUI: picking a **search result now selects and reveals** that entry in the tree.
- Desktop: selecting an entry inside a **collapsed folder auto-reveals** it.
- Desktop: tree hover no longer shows a highlight box (text-colour change only).

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

[26.9.0-beta.1]: https://github.com/chrnx-dev/ichtaca/releases/tag/v26.9.0-beta.1
[26.6.0-beta.1]: https://github.com/chrnx-dev/ichtaca/releases/tag/v26.6.0-beta.1
[26.6.0-alpha.2]: https://github.com/chrnx-dev/ichtaca/releases/tag/v26.6.0-alpha.2
[26.6.0-alpha.1]: https://github.com/chrnx-dev/ichtaca/releases/tag/v26.6.0-alpha.1
[26.6.0-alpha]: https://github.com/chrnx-dev/ichtaca/releases/tag/v26.6.0-alpha
