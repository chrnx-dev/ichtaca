# Contributing to Ichtaca

Thank you for taking the time to contribute. This document covers how to set up a development environment, build and test the project, and submit changes.

---

## Prerequisites

### Rust

Install the **stable** Rust toolchain via [rustup](https://rustup.rs/):

```sh
rustup update stable
```

The workspace MSRV is Rust 1.80 (pass-tui) / 1.77 (pass-tauri); a recent stable release is recommended.

### Node (for the desktop GUI only)

The Svelte/Vite frontend in `crates/pass-tauri/ui/` requires Node. Node v22 is recommended. Use whichever version manager you prefer (nvm, fnm, etc.), but make sure `node` and `npm` are real executables on your `PATH` before running any `npm` commands. If `node`/`npm`/`npx` are shell functions wrapping a version manager, unset or bypass them so that `cargo tauri build` can invoke the binaries directly.

### Optional: Tauri CLI

Required only for `cargo tauri dev` (hot-reload) and `cargo tauri build` (installers):

```sh
cargo install tauri-cli --version "^2"
# or
npm install -g @tauri-apps/cli
```

### System dependencies

- `pass` installed and on `$PATH`
- `gpg` / `gpg-agent`
- An initialized `~/.password-store` (for manual testing with a real store; unit tests use an in-memory `FakeStore`)
- On Linux: `wl-clipboard` (Wayland) or `xclip` (X11) for clipboard tests

---

## Building

### TUI

```sh
cargo build -p pass-tui
# or run directly:
cargo run -p pass-tui
```

### Desktop GUI

The Rust crate embeds the compiled frontend. Build the UI first:

```sh
npm --prefix crates/pass-tauri/ui run build
cargo build -p pass-tauri
# or run directly:
cargo run -p pass-tauri
```

### Full workspace

```sh
cargo build --workspace
```

---

## Testing

```sh
# Rust tests (all crates)
cargo test

# Lint — must pass with zero warnings
cargo clippy --all-targets -- -D warnings

# Formatting — must be clean before submitting a PR
cargo fmt --check      # check only
cargo fmt              # apply

# UI type-check
npm --prefix crates/pass-tauri/ui run check

# UI unit tests
npm --prefix crates/pass-tauri/ui run test
```

All of the above are expected to pass before a pull request is opened.

---

## Coding Conventions

### General

- **Test-driven development is encouraged.** New logic in `passcore` should have unit tests alongside it. Public API additions without tests will be asked to add them.
- Keep `cargo clippy --all-targets -- -D warnings` clean. Suppressions (`#[allow(...)]`) must be justified in a comment.
- Keep `cargo fmt` clean. The project follows the default `rustfmt` style.

### `passcore` (the core library)

- `passcore` has **no UI dependencies** — keep it that way. It must remain a pure logic library that both `pass-tui` and `pass-tauri` (and future frontends) can depend on without pulling in UI frameworks.
- Sensitive values should use the `Secret` type so they are zeroized on drop and redacted in debug output.

### `pass-tui`

- UI state changes flow through the `tui-realm` model/message pattern. Keep components focused and delegate logic to `passcore`.

### `pass-tauri`

- Tauri commands that touch secret material must follow the existing IPC split: operations that don't need to return plaintext (copy, OTP) should do the work in the backend and return nothing sensitive to the webview.

---

## Commit Style

Use conventional-commits style prefixes. Examples:

```
feat(core): add password generator strength estimator
fix(tui): handle empty store gracefully on startup
refactor(tauri): extract clipboard logic into passcore
docs: update CONTRIBUTING with Node version guidance
chore: bump zeroize to 1.9
```

Keep the subject line under 72 characters. Use the body to explain *why*, not just *what*.

---

## Pull Request Process

1. Fork the repository and create a branch from `main` (e.g. `feat/my-feature` or `fix/issue-42`).
2. Make your changes and ensure all checks pass (see [Testing](#testing)).
3. Open a pull request against `main` with a clear description of what you changed and why.
4. A maintainer will review the PR. Please be patient — this is a small project.
5. Address review feedback and push updates to the same branch; the PR will update automatically.

### What makes a good PR

- Focused scope: one logical change per PR.
- Tests for new or changed behaviour.
- No unrelated formatting or style changes mixed in.
- The PR description explains the motivation (link to an issue if one exists).

---

## Repository Layout Notes

- `specs/` — local planning/spec documents. This directory is **not part of the repository** and is excluded from version control. Do not reference it in issues or PRs.
- `target/` — Rust build artefacts. Not committed.
- `crates/pass-tauri/ui/node_modules/` and `crates/pass-tauri/ui/dist/` — not committed.

---

## Code of Conduct

All contributors are expected to abide by the [Code of Conduct](CODE_OF_CONDUCT.md).
