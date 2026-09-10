use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "dots", about = "Dotfile manager for Arch Linux", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Symlink config files into $HOME
    Sync(SyncArgs),
    /// Install packages for modules
    Install(InstallArgs),
    /// Show module status
    Status,
    /// List, check or audit packages
    Packages(PackagesArgs),
    /// Wallpaper management
    Wallpaper {
        #[command(subcommand)]
        cmd: WallpaperCmd,
    },
    /// Set global dark / light mode
    Theme {
        #[command(subcommand)]
        cmd: ThemeCmd,
    },
    /// Monitor control (DDC/CI via ddcutil, brightnessctl fallback)
    Monitor {
        #[command(subcommand)]
        cmd: MonitorCmd,
    },
    /// Settings hub (wofi menu)
    Menu,
    /// Wi-Fi picker — connect, rescan, toggle the radio (wofi)
    Wifi,
    /// Audio output / input picker, moving live streams too (wofi)
    Audio,
    /// Bluetooth device picker — connect, disconnect, scan (wofi)
    Bluetooth,
    /// Display picker — resolution, scale, rotation, arrangement (wofi)
    Displays,
    /// Power profile. No argument opens the picker; `get` prints the current
    /// one; anything else is a profile name (performance, balanced, power-saver).
    Power { profile: Option<String> },
    /// Hold or release an idle inhibitor: toggle | on | off
    Inhibit {
        #[arg(default_value = "toggle")]
        action: String,
    },
    /// Pause or resume notifications: toggle | on | off
    Dnd {
        #[arg(default_value = "toggle")]
        action: String,
    },
    /// Night light (hyprsunset): toggle | on | off
    Night {
        #[arg(default_value = "toggle")]
        action: String,
    },
    /// Print one waybar custom module's JSON and exit.
    ///
    /// Topics: game, dnd, night, brightness, drift, updates, wallpaper.
    /// Modules run "interval": "once" with a signal, so nothing polls — the
    /// dots command that changes the state raises it.
    Bar { topic: String },
    /// Keybind cheatsheet overlay (wofi)
    Keys,
    /// Toggle game mode (animations / blur / shadows off)
    Game,
    /// Run health checks
    Doctor,
    /// Clone a dotfiles repo to ~/.local/share/dots/repo and set it up
    Init {
        /// Git URL of the dotfiles repository
        url: String,
        /// Machine profile (default: match hostname, else prompt)
        #[arg(long, short)]
        machine: Option<String>,
    },
    /// Move the repo to ~/.local/share/dots/repo and rewrite all symlinks
    Migrate {
        /// Machine profile (default: config.toml, hostname match, else prompt)
        #[arg(long, short)]
        machine: Option<String>,
    },
    /// Run background daemon (AC monitor + socket server)
    Daemon,
}

#[derive(Args)]
pub struct SyncArgs {
    /// Machine profile (laptop, desktop)
    #[arg(long, short)]
    pub machine: Option<String>,
    /// Only sync specific modules
    pub modules: Vec<String>,
}

#[derive(Args)]
pub struct InstallArgs {
    /// Machine profile (laptop, desktop)
    #[arg(long, short)]
    pub machine: Option<String>,
    /// Only install specific modules
    pub modules: Vec<String>,
}

#[derive(Args)]
pub struct PackagesArgs {
    /// Check installed vs required
    #[arg(long, short)]
    pub check: bool,
    /// Audit: reconcile modules.toml against pacman reality
    #[arg(long, short)]
    pub audit: bool,
}

#[derive(Subcommand)]
pub enum WallpaperCmd {
    /// Register a wallpaper (video → gif, image → copy)
    Register {
        path: std::path::PathBuf,
        /// Override the stored name (default: filename stem)
        #[arg(long, short)]
        name: Option<String>,
        /// Frames per second for video → gif conversion
        #[arg(long, default_value_t = 6)]
        fps: u32,
    },
    /// Apply a registered wallpaper by name
    Set { name: String },
    /// List registered wallpapers
    List,
    /// Pick a wallpaper from a wofi menu
    Menu,
    /// Set wallpaper mode
    Mode {
        /// auto | animated | static
        mode: String,
    },
}

#[derive(Subcommand)]
pub enum ThemeCmd {
    /// Enable dark mode (gsettings + matugen dark palette)
    Dark,
    /// Enable light mode (gsettings + matugen light palette)
    Light,
    /// Toggle between dark and light
    Toggle,
    /// Pin a preset palette (tokyo-night, catppuccin, nord, gruvbox)
    Set {
        /// Preset name
        preset: String,
    },
    /// Unpin: derive colors from the current wallpaper again
    Auto,
}

#[derive(Subcommand)]
pub enum MonitorCmd {
    /// List detected displays and their brightness
    List {
        /// Re-run ddcutil detect and rebuild the cache
        #[arg(long, short)]
        refresh: bool,
    },
    /// Set brightness: absolute (60) or relative (+5 / -5)
    Brightness {
        value: String,
        /// Target a specific monitor by connector name (e.g. HDMI-A-1)
        #[arg(long, short)]
        monitor: Option<String>,
        /// Target all monitors
        #[arg(long, short)]
        all: bool,
    },
    /// Set contrast: absolute (60) or relative (+5 / -5)
    Contrast {
        value: String,
        /// Target a specific monitor by connector name (e.g. HDMI-A-1)
        #[arg(long, short)]
        monitor: Option<String>,
        /// Target all monitors
        #[arg(long, short)]
        all: bool,
    },
    /// Print the focused monitor's brightness (for waybar)
    Get,

    // --- geometry, over hyprctl rather than DDC ---------------------------
    /// List the modes an output advertises
    Modes {
        /// Connector name (default: the focused output)
        monitor: Option<String>,
    },
    /// Set the mode: WIDTHxHEIGHT[@HZ], or preferred | highres | highrr
    Mode {
        mode: String,
        /// Connector name (default: the focused output)
        #[arg(long, short)]
        monitor: Option<String>,
    },
    /// Set the fractional scale (0.5 - 3.0)
    Scale {
        scale: f64,
        /// Connector name (default: the focused output)
        #[arg(long, short)]
        monitor: Option<String>,
    },
    /// Move an output. Exactly one placement flag is required.
    Position {
        /// Connector name (default: the focused output)
        #[arg(long, short)]
        monitor: Option<String>,
        /// Absolute slot, as XxY (e.g. 1920x0)
        #[arg(long, group = "placement")]
        at: Option<String>,
        #[arg(long, group = "placement", value_name = "OTHER")]
        right_of: Option<String>,
        #[arg(long, group = "placement", value_name = "OTHER")]
        left_of: Option<String>,
        #[arg(long, group = "placement", value_name = "OTHER")]
        above: Option<String>,
        #[arg(long, group = "placement", value_name = "OTHER")]
        below: Option<String>,
    },
    /// Rotate an output: 0 | 90 | 180 | 270
    Rotate {
        degrees: u32,
        /// Connector name (default: the focused output)
        #[arg(long, short)]
        monitor: Option<String>,
    },
    /// Turn an output on
    Enable { monitor: String },
    /// Turn an output off
    Disable { monitor: String },
    /// Mirror one output onto another
    Mirror {
        monitor: String,
        /// The output to mirror *onto*
        #[arg(long, short)]
        onto: String,
    },
    /// Write the live layout to ~/.config/hypr/monitors.conf
    Save,
}
