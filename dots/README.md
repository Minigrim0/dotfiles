# dots

A dotfile manager for Arch Linux. `dots` symlinks configuration files into
`$HOME`, installs packages via `pacman`/`paru`/`yay`, manages wallpapers
(including animated wallpapers via `swww`), runs a background AC-monitor
daemon, and wraps `syncthing` service management — all driven from a single
`modules.toml` manifest.

---

## Requirements

| Tool | Purpose |
|------|---------|
| Arch Linux | Only supported distro |
| `paru` or `yay` | AUR helper used by `install` |
| `swww` | Wayland wallpaper daemon |
| `matugen` | Material-You colour generation from wallpaper |
| `ffmpeg` | Video-to-GIF conversion for animated wallpapers |

---

## Installation

```sh
# 1. Clone the dotfiles repo
git clone https://github.com/<you>/dotfiles ~/dotfiles
cd ~/dotfiles/dots

# 2. Build the binary
cargo build --release

# 3. Make it available on $PATH
ln -sf "$PWD/target/release/dots" ~/.local/bin/dots
```

Pre-built binaries for `x86_64-unknown-linux-gnu` are attached to each
[GitHub release](../../releases).

---

## Usage

```
dots <COMMAND>
```

### sync

Symlink configuration files from `configs/<module>/` into `$HOME`.

```sh
# Sync all enabled modules
dots sync

# Sync specific modules only
dots sync hyprland kitty

# Sync and apply machine-specific symlinks
dots sync --machine laptop
```

### install

Install packages for enabled modules using `pacman` (official) and `paru`/`yay`
(AUR). Runs pre- and post-install hooks defined in `modules.toml`.

```sh
# Install all enabled modules
dots install

# Install specific modules
dots install shell fonts

# Install + apply machine extras and set up the dots systemd service
dots install --machine desktop
```

### status

Show a summary table of every module: enabled/disabled state, whether a
`configs/` directory exists, package count, and how many symlinks are missing.
Also prints daemon health and the `dots` systemd service state.

```sh
dots status
```

### packages

List or audit installed packages across all enabled modules.

```sh
# Print a two-column table (pacman | AUR)
dots packages

# Check which required packages are not yet installed
dots packages --check

# Full audit: reconcile modules.toml against pacman reality.
# Reports MISSING (in manifest, not installed), UNMANAGED (explicitly
# installed but untracked), ORPHANS and untracked FOREIGN/AUR packages.
dots packages --audit
```

### wallpaper

Manages wallpapers stored in `~/.local/share/dots/wallpapers/`.

```sh
# Register a new wallpaper (video is converted to GIF at 10 fps)
dots wallpaper register /path/to/image.jpg
dots wallpaper register /path/to/video.mp4 --name my-animation

# Apply a registered wallpaper
dots wallpaper set my-animation

# List all registered wallpapers
dots wallpaper list

# Pick one from a wofi menu
dots wallpaper menu

# Change the active mode
dots wallpaper mode auto      # pick animated when on AC, static on battery
dots wallpaper mode animated  # always animated
dots wallpaper mode static    # always static
```

#### Wallpaper workflow

```
register  →  set  →  mode
```

1. `register` imports the file (copies images; converts videos to GIF).
2. `set` tells `swww` to display it and runs `matugen` to regenerate colours.
3. `mode` controls whether the daemon will switch between animated/static based
   on AC state (useful on laptops).

### theme

Set or toggle the global dark / light mode. Applies gsettings, regenerates
the matugen palette from the current wallpaper, and saves the state.

```sh
dots theme dark
dots theme light
dots theme toggle
```

### monitor

Monitor control — DDC/CI via `ddcutil` on desktops, falling back to
`brightnessctl` when no DDC display is present (laptops). The i2c bus map is
cached at `~/.local/state/dots/monitors.json` so adjustments skip the slow
`ddcutil detect` (~500 ms → ~20 ms). By default commands target the monitor
that currently has focus (via `hyprctl monitors -j`).

```sh
dots monitor list             # table of displays + brightness (--refresh to re-detect)
dots monitor brightness +5    # focused monitor, relative
dots monitor brightness 60    # focused monitor, absolute
dots monitor brightness 80 --all
dots monitor contrast 70 --monitor DP-2
dots monitor get              # bare brightness number, for waybar
```

Every change sends a dunst progress notification (stacked, replaceable).

### menu

Settings hub rendered with wofi: wallpaper picker, theme switcher, brightness
presets, idle toggle, night light (hyprsunset), game mode, keybind cheatsheet,
and power menu (wlogout).

```sh
dots menu
```

### keys

Parse the keybinds from `~/.config/hypr/hyprland.conf` (+ `machine.conf`) and
show a searchable cheatsheet in wofi. Comments directly above a bind become
its description — the config is the documentation.

```sh
dots keys
```

### game

Toggle game mode: switches off animations, blur, shadows and inactive-dim via
`hyprctl --batch`; toggling back restores everything with `hyprctl reload`.

```sh
dots game
```

### doctor

Health checks: broken or foreign symlinks, daemon processes, failed systemd
user units, amdgpu error floods in the journal, journal disk usage, and
missing/orphaned packages.

```sh
dots doctor
```

### daemon

Run the background daemon. It watches AC power state and switches the wallpaper
mode automatically. It also listens on a Unix socket for IPC from other `dots`
subcommands.

```sh
dots daemon
```

The `install --machine` command registers `dots daemon` as a `systemd --user`
service so it starts on login automatically.

### syncthing

Thin wrapper around the `syncthing` systemd user service.

```sh
dots syncthing install   # install syncthing via AUR helper
dots syncthing start     # systemctl --user start syncthing
dots syncthing stop      # systemctl --user stop syncthing
dots syncthing status    # show service status
```

---

## Configuration

### `modules.toml`

Lives at the repo root. Declares every module — its packages, optional AUR
packages, the name of its `configs/` subdirectory (defaults to the module
name), and lifecycle hooks.

```toml
[meta]
version = "1"

[modules.terminal]
enabled = true
configs = "kitty"           # maps to configs/kitty/ in the repo
packages = ["kitty"]

[modules.shell]
enabled = true
packages = ["zsh", "starship", "zoxide"]
hooks.post_install = ["chsh -s /bin/zsh"]

[modules.wallpaper]
enabled = true
configs = "matugen"
packages = ["swww", "matugen", "ffmpeg"]
aur_packages = ["mpvpaper"]
```

Fields:

| Field | Type | Description |
|-------|------|-------------|
| `enabled` | bool | Whether `sync`/`install` act on this module |
| `configs` | string | Subdirectory under `configs/` (default: module name) |
| `packages` | list | Official pacman packages |
| `aur_packages` | list | AUR packages installed via `paru`/`yay` |
| `hooks.pre_install` | list | Shell commands run before package install |
| `hooks.post_install` | list | Shell commands run after package install |

### `machines/*.toml`

Per-machine overrides. Pass `--machine <name>` to `sync` or `install` to apply
them. The file name (without `.toml`) is the machine identifier.

```toml
[meta]
name = "laptop"
hostname = ""   # fill with: hostnamectl hostname

[packages]
extra = ["brightnessctl", "acpi"]   # extra packages installed on this machine

[wallpaper]
animated_on_ac = true       # switch to animated wallpaper when on AC power
static_on_battery = true    # switch to static wallpaper when on battery

[hyprland]
brightness_up   = "brightnessctl -e4 -n2 set 5%+"
brightness_down = "brightnessctl -e4 -n2 set 5%-"

[waybar]
extra_modules_right = ["battery"]
```

---

## Development

```sh
# Format
cargo fmt

# Lint (all warnings are errors)
cargo lint          # alias defined in .cargo/config.toml

# Test
cargo test
```
