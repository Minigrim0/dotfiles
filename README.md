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

On a fresh machine:

```sh
cargo install --git https://github.com/Minigrim0/dotfiles dots
# or, once published: paru -S dots-bin   (PKGBUILDs in packaging/)

dots init git@github.com:Minigrim0/dotfiles.git   # clone to ~/.local/share/dots/repo,
                                                  # pick machine, symlink configs
dots install --machine <name>                     # packages + hooks + machine extras
```

An existing checkout is adopted with `dots migrate` — it moves the repo to
`~/.local/share/dots/repo` and rewrites every symlink. The repo location is
resolved from `$DOTFILES_DIR`, then `~/.config/dots/config.toml`, then by
walking up from the current directory (dev convenience).

The wallpaper daemon runs as a systemd user service (`dots.service`, in the
`daemon` module), started by Hyprland at login.

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
