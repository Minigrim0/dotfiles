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

Qt follows the same palette by a second route. `qt6ct` reads a matugen-rendered
QPalette (`configs/matugen/templates/qt-colors.conf`) and renders it with Fusion
— Fusion rather than Kvantum because a Kvantum theme needs a hand-drawn SVG,
while Fusion draws straight from the palette, which is the only part a wallpaper
can generate. `QT_QPA_PLATFORMTHEME` holds one value and Qt5 and Qt6 disagree
about it, so the session sets `qt6ct` and the `qt` module's hook rewrites VLC's
desktop entry — VLC 3 being the last Qt5 GUI here.

KDE applications are the exception, and deliberately not chased: with Plasma
absent they read neither qt6ct's palette nor `kdeglobals`. That is measured, not
assumed — setting `kdeglobals`' window background to pure red leaves Dolphin
grey. It is why the file manager is Nautilus (GTK4/libadwaita, themed by the
same `gtk-colors.css` as everything else) rather than a KDE one.

Typography is centralised the same way. Every GTK-CSS surface — waybar, wofi,
swayosd, wlogout — starts with `@import "../dots/fonts.css"`, so the interface
face is defined once in `configs/fonts/`. Three roles: **Adwaita Sans** for
interface text, **JetBrainsMono Nerd Font** for the terminal and anything
numeric, and **Symbols Nerd Font** as a glyph-only fallback — which is what
lets the UI use a real sans and still render `󰤨`. The same three roles are
declared to fontconfig (`configs/fonts/.config/fontconfig`), which is how Qt and
XWayland apps get them — GTK CSS reaches neither.

## The session

SDDM greets; **uwsm** runs the session. Picking *Hyprland (uwsm-managed)* at the
greeter is what makes the session a real systemd unit — the environment in
`configs/uwsm/.config/uwsm/env` reaches systemd user units and D-Bus activation,
`graphical-session.target` becomes true, and `dots.service` starts from its own
`WantedBy` instead of an `exec-once`. The plain *Hyprland* entry still boots, but
silently skips all of that, so `dots doctor` reports which one you are in.

gnome-keyring is the Secret Service behind Nextcloud, Signal and VS Code;
`pam_gnome_keyring` unlocks it with the login password. Being a PAM concern it
lives outside this repo, so `dots doctor` checks it rather than managing it.

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

## Displays

`dots monitor` covers both channels the hardware answers on: geometry through
Hyprland, brightness and contrast through DDC/CI on the display cable. One table
shows both.

```sh
dots monitor list                          # geometry + brightness, every output
dots monitor modes DP-2                    # what the EDID advertises
dots monitor mode 2560x1440@144 -m DP-2
dots monitor scale 1.5 -m eDP-1
dots monitor position -m DP-2 --right-of HDMI-A-1
dots monitor rotate 90 -m DP-3
dots monitor save                          # → monitors-<machine>.conf
```

`save` writes through the `monitors.conf` symlink into the machine's own tracked
file, so a layout is committed to the machine it belongs to. `dots displays`
(Super + Shift + M) is the same thing as a picker, and hands off to nwg-displays
for drag-and-drop arrangement — the one job a list of menu items is bad at.

## The boot splash

Plymouth, themed from the same matugen palette as everything else: the logo
composited at native size on black, with a pill entry field for the LUKS
passphrase and a Caps Lock warning that only appears when it applies.

```
dots splash preview     # compose a PNG mock, no root, no reboot
dots splash apply       # install to /usr/share and rebuild the initramfs
```

Two things make it unlike every other themed surface here.

It cannot follow the wallpaper. mkinitcpio bakes a *copy* of the theme into the
initramfs, so a new palette does not reach the boot screen until the theme is
reinstalled and the initramfs rebuilt — root-owned, and as slow as mkinitcpio.
That is why `dots splash apply` is explicit rather than something
`wallpaper set` triggers, and why `preview` exists: the real thing is otherwise
only observable by rebooting.

The ground is black rather than the palette surface. The logo's outer ring is a
flat near-black field, so black is what makes the square edgeless with no
feathering at all — feathering it into a lighter surface tone leaves a dark
halo, because the artwork's black is darker than any surface color matugen
produces. The palette still drives the entry, the bullets and the text.

Two boot-side prerequisites, neither managed by `dots`:

- `splash` on the kernel cmdline. Without it plymouth is built into the
  initramfs and then told to stay quiet, which looks exactly like a theme that
  does not work. `dots doctor` reports it.
- `plymouth` in `HOOKS`, before `encrypt`. Note there is deliberately no
  `plymouth-encrypt`: current Arch folded plymouth support into the stock
  `encrypt` hook, which pings plymouthd and calls `plymouth ask-for-password`
  when it answers. The separate hook no longer ships, and adding it to `HOOKS`
  breaks the build.

If the splash ever comes up blank, Esc switches plymouth to its text view — the
passphrase prompt is still there and still accepts input.

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
| `Super + Shift + M` | Display settings (`dots displays`) |
| `Super + Shift + P` | Power profile (`dots power`) |
| `Super + Shift + I` | Keep the screen awake (`dots inhibit`) |

`dots status`, `dots doctor` and `dots packages --audit` report on symlinks,
daemons and package drift. `doctor` also checks the things that fail silently:
whether the session is uwsm-managed, whether the keyring is unlocked by PAM,
whether Qt found its palette, and whether every default application in
`mimeapps.list` names a desktop entry that exists.
