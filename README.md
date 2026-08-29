# dotfiles

Arch Linux + Hyprland setup, managed by [`dots`](dots/README.md) — a small
Rust CLI that symlinks configs, installs packages, and runs the desktop glue
(wallpaper daemon, theming, monitor control).

## Layout

| Path | Contents |
|------|----------|
| `modules.toml` | Module manifest: packages, config dirs, install hooks |
| `configs/` | Per-module config trees, symlinked into `$HOME` by `dots sync` |
| `machines/` | Per-machine overrides (`desktop`, `laptop`) |
| `dots/` | The manager itself — see its [README](dots/README.md) |
| `legacy/` | Retired NixOS home-manager setup, kept for reference |

## Install

```sh
git clone <repo-url> && cd dotfiles
cargo build --release --manifest-path dots/Cargo.toml
install -m755 dots/target/release/dots ~/.local/bin/dots

dots install --machine desktop   # packages + hooks + machine extras
dots sync    --machine desktop   # symlink configs into $HOME
```

## Theming

Wallpaper-driven: `dots wallpaper set <name>` runs matugen, which renders the
templates in `configs/matugen/` into hyprland, waybar, kitty, dunst, GTK,
wlogout and swayosd colors. `dots theme dark|light|toggle` switches the global
scheme.

## Daily driving

| Keys | Action |
|------|--------|
| `Super + ,` | Settings menu (`dots menu`) |
| `Super + /` | Keybind cheatsheet (`dots keys`) |
| `Super + W` | Wallpaper picker |
| `Super + C` | Clipboard history |
| `Super + Escape` | Power menu |

`dots status`, `dots doctor` and `dots packages --audit` report on symlinks,
daemons and package drift.
