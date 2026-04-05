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
    /// List or check packages
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
}

#[derive(Subcommand)]
pub enum WallpaperCmd {
    /// Register a wallpaper (video → gif at 10fps, image → copy)
    Register {
        path: std::path::PathBuf,
        /// Override the stored name (default: filename stem)
        #[arg(long, short)]
        name: Option<String>,
    },
    /// Apply a registered wallpaper by name
    Set { name: String },
    /// List registered wallpapers
    List,
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
}
