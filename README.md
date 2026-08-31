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

Typography is centralised the same way. Every GTK-CSS surface — waybar, wofi,
swayosd, wlogout — starts with `@import "../dots/fonts.css"`, so the interface
face is defined once in `configs/fonts/`. Three roles: **Adwaita Sans** for
interface text, **JetBrainsMono Nerd Font** for the terminal and anything
numeric, and **Symbols Nerd Font** as a glyph-only fallback — which is what
lets the UI use a real sans and still render `󰤨`.

## The bar

Waybar, laid out as three floating islands built to Hyprland's own geometry
(radius 12, 14px margins, 2px borders). No layer blur: Hyprland 0.56 dropped
`ignorezero`, so blurring the bar smears a band across the full width of the
screen — gaps included — which is the one thing islands exist to avoid. Most
modules are invisible at
rest and appear only when they have something to say: mic/screen capture, a
failed systemd unit, paused notifications, game mode, night light, a held idle
inhibitor, disk pressure, CPU temperature, repo drift, pending updates. Volume
and brightness live in a hover drawer with real sliders, because swayosd
already reports them at the moment they change.

`config.jsonc` is shared; `machine-<name>.jsonc` is symlinked to
`machine.jsonc` and `include`d, mirroring how `hyprland.conf` sources
`machine.conf`.

## Daily driving

| Keys | Action |
|------|--------|
| `Super + ,` | Settings menu (`dots menu`) |
| `Super + /` | Keybind cheatsheet (`dots keys`) |
| `Super + W` | Wallpaper picker |
| `Super + C` | Clipboard history |
| `Super + Escape` | Power menu |
| `Super + Shift + N` | Wi-Fi picker (`dots wifi`) |
| `Super + Shift + B` | Bluetooth picker (`dots bluetooth`) |
| `Super + Shift + V` | Audio output / input picker (`dots audio`) |
| `Super + Shift + D` | Pause / resume notifications (`dots dnd`) |

`dots status`, `dots doctor` and `dots packages --audit` report on symlinks,
daemons and package drift.
