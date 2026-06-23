+++
title = "Troubleshooting"
description = "Common issues and fixes for Ichtaca."
weight = 10
+++

## GPG agent / pinentry issues

### No pinentry program

GPG needs a pinentry program to prompt for your passphrase. If you see an error like
"No pinentry" or decryption silently fails, install one:

- **macOS:** `brew install pinentry-mac`, then add `pinentry-program /usr/local/bin/pinentry-mac`
  (or `/opt/homebrew/bin/pinentry-mac` on Apple Silicon) to `~/.gnupg/gpg-agent.conf`.
- **Linux:** `sudo apt install pinentry-curses` (or `pinentry-gtk2`), then add
  `pinentry-program /usr/bin/pinentry-curses` to `~/.gnupg/gpg-agent.conf`.

After editing `gpg-agent.conf`, restart the agent:

```sh
gpgconf --kill gpg-agent
```

### "Inappropriate ioctl for device"

GPG cannot access the terminal when run inside a script or a piped command. Export the
TTY before running Ichtaca:

```sh
export GPG_TTY=$(tty)
```

Add this to your shell profile (`~/.zshrc`, `~/.bashrc`) to set it permanently.

### Agent not running

Start the agent manually:

```sh
gpg-agent --daemon
```

GPG normally starts the agent automatically on the first operation; if it does not, this
command launches it.

---

## Missing clipboard backend

Ichtaca shells out to OS clipboard tools.

| Platform | Tool | Install |
|----------|------|---------|
| macOS | `pbcopy` / `pbpaste` | Built in — no action needed |
| Linux (Wayland) | `wl-copy` / `wl-paste` | `sudo apt install wl-clipboard` |
| Linux (X11) | `xclip` | `sudo apt install xclip` |

On Linux, Ichtaca tries `wl-copy` first, then falls back to `xclip`. If neither is
found, `ichtaca copy` exits with an error asking you to install one.

---

## macOS Gatekeeper ("unidentified developer" / "damaged app")

The `ichtaca-desktop` release build is **not yet code-signed or notarized**. macOS
Gatekeeper blocks it on first launch. To open it anyway:

**Option 1 — Right-click to open (needed only once):**
1. Right-click (or Control-click) `Ichtaca.app` in Finder.
2. Choose **Open**.
3. Click **Open** in the dialog that appears.

**Option 2 — Remove the quarantine flag from Terminal:**

```sh
xattr -dr com.apple.quarantine /Applications/Ichtaca.app
```

Signing and notarization are planned for a future release and will eliminate this step.
The TUI binary (`ichtaca`) runs from the terminal and is not affected by Gatekeeper.

---

## Non-default store location

If your store does not live at the default `~/.password-store`, set `PASSWORD_STORE_DIR`:

```sh
export PASSWORD_STORE_DIR=/path/to/your/store
ichtaca doctor   # confirm the store is found
```

You can also add this export to your shell profile to persist it across sessions.

---

## `ichtaca doctor` — reading the output

`ichtaca doctor` checks all three prerequisites:

```
pass:  ok
gpg:   ok
store: ok (/home/you/.password-store)
```

Any line showing `missing` means that item needs attention. The command also prints
step-by-step installation guidance when something is missing, and exits 2.

Exit code 0 means everything is ready.
