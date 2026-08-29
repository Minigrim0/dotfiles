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
    /// Keybind cheatsheet overlay (wofi)
    Keys,
    /// Toggle game mode (animations / blur / shadows off)
    Game,
    /// Run health checks
    Doctor,
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
}
