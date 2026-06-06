# pass-tui

The Ichtaca TUI frontend — a terminal user interface for the password store built with
[tui-realm](https://github.com/veeso/tui-realm) on top of ratatui.

## Requirements

### Nerd Font (required for icons)

The TUI uses [Nerd Font](https://www.nerdfonts.com/) glyphs throughout the interface
(folder icons in the tree, key/lock icons in the detail panel, field icons, search
glyph, etc.).  **A Nerd Font must be installed and configured as the terminal font**
for these glyphs to render correctly.  Without a Nerd Font the icon codepoints will
appear as boxes or question marks.

**Recommended font:** JetBrainsMono Nerd Font (or any other Nerd Font from
<https://www.nerdfonts.com/>).

To set the font, configure your terminal emulator (iTerm2, Alacritty, WezTerm, Ghostty,
etc.) to use a Nerd Font variant.  For example, in Alacritty:

```toml
[font.normal]
family = "JetBrainsMono Nerd Font"
```

### Other dependencies

- A working `pass` installation with a GPG-encrypted store (or the fake in-memory
  store is used automatically for development/testing).
- Rust 1.80+ (MSRV follows the workspace).

## Running

```sh
cargo run -p pass-tui
```

## Key bindings (browse mode)

| Key           | Action                        |
|---------------|-------------------------------|
| `↑` / `k`    | Move up in tree               |
| `↓` / `j`    | Move down in tree             |
| `←` / `h`    | Collapse directory            |
| `→` / `l`    | Expand directory              |
| `Enter`       | Select entry                  |
| `c`           | Copy password to clipboard    |
| `s`           | Toggle password reveal        |
| `/`           | Open fuzzy search             |
| `a`           | Add new entry                 |
| `e`           | Edit selected entry           |
| `E`           | Raw edit in `$EDITOR`         |
| `d`           | Delete selected entry         |
| `q` / `Esc`  | Quit                          |
