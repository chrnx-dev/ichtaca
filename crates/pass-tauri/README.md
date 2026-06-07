# pass-tauri (Ichtaca desktop)

Tauri 2 desktop client for the pass-client workspace. Embeds the Svelte UI (`ui/`) at compile time and exposes password-store operations via Tauri IPC commands backed by `passcore`.

The produced binary is named **`ichtaca-desktop`** (`cargo run -p pass-tauri` still works). The macOS app bundle and installer display name is **Ichtaca** with bundle identifier `dev.chrnx.ichtaca`.

---

## Prerequisites

- Rust (stable, 1.77+)
- Node via nvm (v22 recommended). If `npm`/`node`/`npx` are shell functions wrapping something else, unset them first:

  ```sh
  unset -f npm node npx 2>/dev/null
  export PATH="$HOME/.nvm/versions/node/v22.22.2/bin:$PATH"
  ```

- Optional: `@tauri-apps/cli` / `cargo-tauri` for `cargo tauri dev` and `cargo tauri build`.

---

## Build order

**The Rust crate embeds `ui/dist` at compile time.** You must build the UI before building or bundling the Rust crate:

```sh
# 1. Build the frontend
npm --prefix crates/pass-tauri/ui run build

# 2. Build the Rust crate (debug or release) → produces target/debug/ichtaca-desktop
cargo build -p pass-tauri
cargo build --release -p pass-tauri
```

If `ui/dist` is absent when `cargo build` runs, the build will fail because `tauri.conf.json` points `frontendDist` at `ui/dist`.

> `cargo run -p pass-tauri` continues to work (Cargo resolves by package). The installed binary name is `ichtaca-desktop`.

---

## Development

### Without Tauri CLI (cargo tauri not installed)

```sh
# Build the UI in watch mode (separate terminal)
npm --prefix crates/pass-tauri/ui run dev   # Vite dev server on localhost:1420 (or similar)

# Build and run the native binary (produces ichtaca-desktop)
cargo run -p pass-tauri
```

> Note: without `cargo tauri dev`, hot-reload is not available. Re-run `npm --prefix … run build` + `cargo run` after UI changes.

### With Tauri CLI installed

```sh
# Install the CLI once
cargo install tauri-cli --version "^2"
# or
npm install -g @tauri-apps/cli

# Then from the crate directory:
cd crates/pass-tauri
cargo tauri dev
```

`cargo tauri dev` builds the UI automatically via `beforeBuildCommand` in `tauri.conf.json`.

---

## Producing installers

```sh
# Build the UI first, then:
cargo tauri build        # requires tauri-cli / @tauri-apps/cli
```

### Per-OS caveats

The `bundle.targets` in `tauri.conf.json` is set to `["app", "dmg", "deb", "appimage"]`. Tauri only builds targets for the **current host OS**:

| Host OS | Targets produced          |
|---------|---------------------------|
| macOS   | `app`, `dmg`              |
| Linux   | `deb`, `appimage`         |
| Windows | `msi`, `nsis` (not listed)|

Cross-OS bundling is not supported by Tauri. Run the build on the target platform.

### Icon set

The repository currently contains only `icons/icon.png`. Before producing real installers, generate the full icon set (ICO, ICNS, and various PNG sizes):

```sh
cargo tauri icon crates/pass-tauri/icons/icon.png
```

This overwrites `icons/` with all required sizes. Commit the generated icons before cutting a release build.

### macOS signing and notarization

For local/development use, the app bundle is unsigned (ad-hoc). For distribution:

1. Set `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY` in the environment.
2. Set `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID` for notarization.
3. Run `cargo tauri build` — Tauri will sign and submit for notarization automatically.

Ad-hoc unsigned builds work on the local machine but will be rejected by Gatekeeper on other machines.

---

## Security model

Secrets stay in the Rust backend at all times:

- `list` / `search` / `show_meta` — return only metadata (paths, field names, OTP presence flag). No secrets cross the IPC boundary.
- `copy_password` / `otp_code` — the backend performs the clipboard write or TOTP derivation itself; the plaintext never appears in the frontend.
- `reveal_password` / `reveal_otp_uri` — explicitly return plaintext to the UI on user request. These are the only commands that cross the IPC boundary with secret material.

### Hardened CSP

`tauri.conf.json` enforces:

```
default-src 'self';
script-src 'self';
style-src 'self' 'unsafe-inline';
img-src 'self' data:;
connect-src 'self' ipc: http://ipc.localhost;
object-src 'none';
base-uri 'self';
frame-ancestors 'none'
```

- `'unsafe-inline'` in `style-src` only — required because Svelte injects scoped styles at runtime.
- `ipc:` and `http://ipc.localhost` in `connect-src` are required for Tauri 2's IPC/invoke mechanism.
- No remote scripts, no object/embed, no framing.

> **Note:** `dangerousRemoteUrlLoadingAllowed` is not a valid Tauri 2 `SecurityConfig` field (as of tauri-utils 2.9.x). Remote URL loading is disabled by default in Tauri 2 — no explicit config key is needed.

### Security smoke test (desktop only)

The hardened CSP cannot be verified headlessly. On a desktop session:

1. Run the app (`cargo tauri dev` or `cargo run -p pass-tauri`).
2. Confirm that `list`/`search` loads entries in the sidebar.
3. Select an entry — metadata appears in the detail panel.
4. Click Copy / OTP — confirm the action completes without a console CSP error.
5. Click Reveal — confirm the password or OTP URI appears.

If IPC breaks under the CSP (no entries load, invoke calls hang), relax `connect-src` first by adding `http://localhost` or checking the Tauri devtools console for blocked requests.

---

## Running tests

```sh
# Rust (all crates)
cargo test

# Clippy
cargo clippy --all-targets -- -D warnings

# UI type-check
npm --prefix crates/pass-tauri/ui run check

# UI unit tests
npm --prefix crates/pass-tauri/ui run test
```
