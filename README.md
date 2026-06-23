<p align="center">
  <img src="site/static/logo-400.png" alt="Ichtaca logo" width="160" height="160">
</p>

<h1 align="center">Ichtaca</h1>

<p align="center"><strong>lo oculto</strong> — a <code>pass</code> client for the terminal and desktop</p>

**Ichtaca** (Classical Nahuatl: *"the hidden / the secret"*; pronounced *ich-TA-ka*) is a front-end for [`pass`](https://www.passwordstore.org/), the standard Unix password manager. It does **not** replace `pass` — it wraps the `pass` CLI and `gpg`/`gpg-agent`, giving you a richer interface over your existing `~/.password-store`.

---

## Screenshots

**`ichtaca` — terminal UI**

![Ichtaca terminal TUI](site/static/screenshots/tui.png)

**`ichtaca-desktop` — desktop GUI**

![Ichtaca desktop GUI](site/static/screenshots/desktop.png)

---

## What is this?

Ichtaca provides two apps that share a single Rust core library (`passcore`):

| App | Binary | Description |
|-----|--------|-------------|
| **Ichtaca TUI** | `ichtaca` | Full-featured terminal UI (keyboard-driven) |
| **Ichtaca Desktop** | `ichtaca-desktop` | Native desktop GUI (Tauri + Svelte) |

Both apps read and write the same `~/.password-store` directory that `pass` manages — no data migration needed.

### Requirements

- [`pass`](https://www.passwordstore.org/) installed and on `$PATH`
- `gpg` / `gpg-agent` installed and a key configured
- An initialized password store (`pass init <gpg-key-id>`)
- **TUI only:** a [Nerd Font](https://www.nerdfonts.com/) configured in your terminal emulator (for folder/key/lock icons)
- Rust (stable) — see [Building](#building)

Supported platforms: **macOS** and **Linux**.

---

## Features

- **Tree browser** — navigate your store directory tree with keyboard or mouse
- **Entry detail panel** — view fields parsed from the `pass` format (password, username, URL, notes, custom fields)
- **Password masked by default** — reveal on explicit action (`s` / Reveal button)
- **Copy to clipboard** — copies the password via `pass`; clipboard is auto-cleared after 45 seconds (configurable) and only if the clipboard still holds the copied value
- **TOTP / OTP codes** — generate and copy one-time passwords from `otpauth://` URIs stored in entries
- **Create / edit / delete entries** — form-based with user-defined templates; or raw `$EDITOR` edit
- **Password generator** — CSPRNG-backed; configurable length and character set
- **Fuzzy search** — live search across all entry paths, plus an on-demand content search (`Ctrl-f`) that decrypts entries to match inside bodies and tags
- **`git` passthrough** — if your store is a git repo, `pass` manages commits as usual

---

## Installation

### macOS

```sh
# Homebrew prerequisites (if not already installed)
brew install pass gnupg
```

Download the latest release from [GitHub Releases](https://github.com/chrnx-dev/ichtaca/releases/latest):
- `ichtaca-macos-arm64.tar.gz` (Apple Silicon) or `ichtaca-macos-x86_64.tar.gz` (Intel) — TUI binary
- `Ichtaca.dmg` — desktop app

Or build from source (see [Building](#building) below).

### Linux

Install prerequisites via your distro's package manager, for example on Debian/Ubuntu:

```sh
sudo apt install pass gnupg2
```

Download the Linux release artifacts:
- `ichtaca-linux-x86_64.tar.gz` — TUI binary
- `ichtaca-desktop.deb` / `ichtaca-desktop.AppImage` — desktop app

Or build from source.

---

## First-time setup

If you do not have a password store yet, create one from scratch:

```sh
# 1. Generate a GPG key (choose RSA 4096 or Ed25519; remember the passphrase)
gpg --full-generate-key

# 2. Find your new key's ID
gpg --list-secret-keys --keyid-format=long

# 3. Initialise the store
pass init <your-gpg-key-id>

# 4. Verify everything is ready
ichtaca doctor
```

`ichtaca doctor` exits 0 and prints `ok` for every check when the environment is ready. It exits 2 and prints guidance when something is missing.

### Using an existing store

If your store lives somewhere other than `~/.password-store`, point Ichtaca at it with the standard `pass` environment variable:

```sh
export PASSWORD_STORE_DIR=/path/to/your/store
```

Ichtaca respects `PASSWORD_STORE_DIR` exactly as `pass` does.

---

## Trying it without a store

Set `ICHTACA_DEMO=1` to launch an explicit demo session seeded with obviously-fake entries — no `pass` or `gpg` required:

```sh
ICHTACA_DEMO=1 ichtaca          # TUI demo
ICHTACA_DEMO=1 ichtaca-desktop  # Desktop demo
```

The TUI shows a `· DEMO` marker in the header; the desktop shows a `DEMO` badge in the navbar. Demo mode is **not your real data** and is not connected to any password store.

---

## CLI reference

Running `ichtaca` with no arguments launches the interactive TUI. Running `ichtaca <subcommand>` runs non-interactively and exits.

### Commands

| Command | Description |
|---------|-------------|
| `ichtaca doctor` | Check pass / gpg / store; exits 0 when ready, 2 when not |
| `ichtaca list` | Print all entry paths, one per line |
| `ichtaca search <query>` | Fuzzy-search entry paths |
| `ichtaca get <path>` | Print the password to stdout (no trailing newline; pipe-safe) |
| `ichtaca show <path> [--json]` | Print metadata only: path, fields, tags, `has_otp`. **Does not print the password or OTP URI.** |
| `ichtaca otp <path>` | Print the current TOTP code |
| `ichtaca copy <path>` | Copy the password to the clipboard, then clear it after the configured `clear_after` timeout (blocks until cleared; Ctrl-C to keep). If `clear_after` is 0, copies and returns immediately without clearing. |
| `ichtaca generate <path> [--length N] [--no-symbols]` | Generate and store a password, then print it to stdout; **refuses if the entry already exists** |
| `ichtaca set <path> [--password-stdin] [--field key=value ...]` | Create or update an entry; preserves existing OTP, tags, and fields |

#### `ichtaca show` JSON shape

```json
{"path":"web/example.com","fields":[["user","alice"]],"tags":["work"],"has_otp":true}
```

The password and raw OTP URI are intentionally excluded.

#### `ichtaca set` — secrets via stdin, never argv

Passwords are always read from stdin using `--password-stdin`. This keeps secrets out of shell history and process listings. Repeat `--field` for multiple fields.

#### `ichtaca generate` defaults

Default length is `32` characters. Override with `--length N`. Use `--no-symbols` for alphanumeric-only.

### Exit codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | User / input error (bad argument, entry not found, no OTP configured, etc.) |
| `2` | Missing dependency or store (pass not installed, gpg not installed, store not initialised) |
| `3` | Store / decrypt failure (GPG error, I/O error, git error) |

### Examples

```sh
# Store a password (read from stdin)
printf 'hunter2\n' | ichtaca set web/example.com --password-stdin --field user=me

# Add fields to an existing entry without changing the password
ichtaca set web/example.com --field url=https://example.com

# Show metadata as JSON and pretty-print it
ichtaca show web/example.com --json | jq .

# Get the password and copy it yourself (macOS)
ichtaca get web/example.com | pbcopy

# Generate a 24-character alphanumeric password
ichtaca generate dev/api-key --length 24 --no-symbols

# Check the TOTP code for an entry
ichtaca otp email/work

# Copy to clipboard (auto-clears after the configured timeout)
ichtaca copy email/work
```

---

## Building

### TUI (`ichtaca`)

```sh
# Run from source
cargo run -p pass-tui

# Install to ~/.cargo/bin/
cargo install --path crates/pass-tui
```

The produced binary is named `ichtaca`.

### Desktop GUI (`ichtaca-desktop`)

The Rust crate embeds the compiled frontend. **Build the UI before the Rust crate.**

```sh
# 1. Build the Svelte/Vite frontend
npm --prefix crates/pass-tauri/ui run build

# 2. Build and run the native binary
cargo run -p pass-tauri

# — or — produce installers (requires tauri-cli)
cargo install tauri-cli --version "^2"
cargo tauri build
```

Tauri only bundles for the **host OS**: `app`/`dmg` on macOS, `deb`/`appimage` on Linux.

#### Opening the macOS app (alpha is unsigned)

The alpha `.dmg`/`.app` is **not code-signed or notarized** yet, so macOS Gatekeeper
will block it on first launch ("unidentified developer" / "damaged"). To open it:

- **Right-click** (or Control-click) the app → **Open** → **Open** (only needed once), **or**
- remove the quarantine flag from a terminal:
  ```sh
  xattr -dr com.apple.quarantine /Applications/Ichtaca.app
  ```

A properly signed + notarized build (no warning) requires an Apple Developer ID;
the release workflow signs automatically once the Apple secrets are configured.

#### Releases & artifacts

Pushing a tag like `v26.6.0-alpha` triggers `.github/workflows/release.yml`, which
builds the desktop bundles (macOS `.dmg`, Linux `.deb`/`.AppImage`) and the TUI
binary tarballs for macOS (arm64 + Intel) and Linux, and attaches them to a draft
GitHub Release.

#### Node version note

The frontend build requires Node (v22 recommended). If `node`/`npm` are shell functions wrapping another version manager, ensure the real binaries are on `PATH` before running `npm`.

#### Development mode

Build the frontend first (the Rust crate embeds `ui/dist` at compile time), then run:

```sh
npm --prefix crates/pass-tauri/ui run build
cargo run -p pass-tauri
```

> The Tauri `beforeBuildCommand` is intentionally empty so the frontend build is
> explicit and deterministic in CI — always build the UI before building/bundling
> the Rust crate.

---

## Usage & Key Bindings

| Key | Action |
|-----|--------|
| `↑` / `k` | Move up in tree |
| `↓` / `j` | Move down in tree |
| `←` / `h` | Collapse directory |
| `→` / `l` | Expand directory |
| `Enter` | Select entry |
| `/` | Open fuzzy search |
| `Ctrl-f` | In search: content search — decrypts entries to match body/tags (slower) |
| `c` | Copy password to clipboard |
| `s` | Toggle password reveal |
| `a` | Add new entry |
| `e` | Edit selected entry (form) |
| `E` | Raw edit in `$EDITOR` |
| `d` | Delete selected entry |
| `q` / `Esc` | Quit |

---

## Security Model

Ichtaca is designed to keep secrets off disk and out of logs.

- Decryption is done by `gpg`/`gpg-agent` via the `pass` CLI — the app never writes decrypted data to disk
- In-memory secrets are zeroized on drop ([`zeroize`](https://crates.io/crates/zeroize)); `Secret` values are redacted in `Debug` output
- Passwords are **masked by default** and only revealed on an explicit user action
- Clipboard copies are auto-cleared after 45 seconds (configurable), using an ownership check so a value the user has since replaced is not wiped
- **Desktop app:** plaintext stays in the Rust backend. IPC commands that handle secrets (`copy_password`, `otp_code`) do the work in the backend and return nothing sensitive to the webview. Only the explicit `reveal_password` / `reveal_otp_uri` commands cross the IPC boundary with plaintext, and only on user request. A strict Content Security Policy (no remote scripts, no remote URLs) is enforced
- No telemetry, no network connections (other than `git` if your store uses it)

See [SECURITY.md](SECURITY.md) for the full security policy and vulnerability reporting instructions.

---

## Configuration

Configuration file: `~/.config/pass-client/config.toml` (XDG: `$XDG_CONFIG_HOME/pass-client/config.toml`).

The file is optional — all settings have sensible defaults. Example:

```toml
[clipboard]
clear_after = 45   # seconds

[keybindings]
copy   = "c"
reveal = "s"
search = "/"
quit   = "q"

[ui]
reveal_default = false

[generator]
length  = 20      # number of characters in a generated password
symbols = true    # include punctuation (!@#$…); set false for alphanumeric-only
```

### Password generator

The built-in generator is a CSPRNG (OS-backed, rejection-sampled so there is no
modulo bias) shared by both frontends. Both the length and the character set are
configurable via the `[generator]` section above (read from
`~/.config/pass-client/config.toml`):

- `length` — password length in characters (default `20`).
- `symbols` — when `true` (default) the charset is alphanumeric plus
  `!@#$%^&*()-_=+`; when `false` it is alphanumeric only.

---

## Project Layout

```
crates/
  passcore/      — core library: store access, parsing, TOTP, clipboard, search, templates
  pass-tui/      — TUI frontend (tui-realm / ratatui)
  pass-tauri/    — desktop frontend (Tauri 2 + Svelte + Tailwind / DaisyUI)
```

---

## Troubleshooting

### GPG agent / pinentry issues

- **"No pinentry program" / GPG prompt does not appear** — install a pinentry program: `brew install pinentry-mac` (macOS) or `sudo apt install pinentry-curses` (Linux), then configure it in `~/.gnupg/gpg-agent.conf`:
  ```
  pinentry-program /usr/local/bin/pinentry-mac
  ```
  Restart the agent: `gpgconf --kill gpg-agent`.

- **"Inappropriate ioctl for device"** — this happens when GPG cannot access the terminal (e.g. inside a script or piped command). Export the current TTY before running:
  ```sh
  export GPG_TTY=$(tty)
  ```
  Add this to your shell profile (`~/.zshrc`, `~/.bashrc`) to make it permanent.

- **Agent not running** — start it with `gpg-agent --daemon` or let GPG start it automatically on the next operation.

### Missing clipboard backend

- **macOS** — `pbcopy` is built in; no action needed.
- **Linux (Wayland)** — install `wl-clipboard`: `sudo apt install wl-clipboard` (or distro equivalent). Ichtaca uses `wl-copy` / `wl-paste`.
- **Linux (X11)** — install `xclip`: `sudo apt install xclip`. Ichtaca falls back to `xclip` when `wl-copy` is not found.

If neither tool is available, `ichtaca copy` exits with an error asking you to install one.

### macOS Gatekeeper ("unidentified developer" / "damaged app")

The desktop release is **not yet code-signed or notarized**. macOS Gatekeeper will block it on first launch. To open it:

- **Right-click** (or Control-click) the app → **Open** → **Open** (needed only once), **or**
- remove the quarantine flag:
  ```sh
  xattr -dr com.apple.quarantine /Applications/Ichtaca.app
  ```

Signing and notarization are planned for a future release. The TUI binary (`ichtaca`) is not affected — it runs from the terminal without Gatekeeper restrictions.

### Using a non-default store location

Set `PASSWORD_STORE_DIR` to point to your store:

```sh
export PASSWORD_STORE_DIR=/path/to/store
ichtaca doctor   # confirm it is found
```

---

## Status

**Alpha** — `26.6.0-alpha.2` (CalVer YY.MM.PATCH). The core features work, but expect rough edges, missing documentation, and breaking changes before a stable release. Use on a real password store at your own risk; always keep a backup.

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

---

## License

MIT — see [LICENSE](LICENSE).
