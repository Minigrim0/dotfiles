mod cli;
mod config;
mod daemon;
mod hooks;
mod installer;
mod linker;
mod syncthing;
mod theme;
mod wallpaper;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command, SyncthingCmd, ThemeCmd, WallpaperCmd};
use config::{dotfiles_dir, load_machine, load_manifest};
use std::io;
use std::path::Path;
use tracing_subscriber::EnvFilter;

// ---------------------------------------------------------------------------
// User-facing ANSI output helpers (not tracing — these are the UI)
// ---------------------------------------------------------------------------
macro_rules! ok   { ($($a:tt)*) => { println!("\x1b[32m✓\x1b[0m  {}", format!($($a)*)) } }
macro_rules! arrow{ ($($a:tt)*) => { println!("\x1b[36m→\x1b[0m  {}", format!($($a)*)) } }
macro_rules! warn { ($($a:tt)*) => { println!("\x1b[33m~\x1b[0m  {}", format!($($a)*)) } }
macro_rules! err  { ($($a:tt)*) => { eprintln!("\x1b[31m✗\x1b[0m  {}", format!($($a)*)) } }
macro_rules! head { ($($a:tt)*) => { println!("\x1b[1m==>\x1b[0m {}", format!($($a)*)) } }

#[tokio::main]
async fn main() -> Result<()> {
    // Tracing is silent by default so it doesn't pollute user-facing output.
    // Set RUST_LOG=dots=debug (or info/warn) to enable diagnostic output.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();
    let dotfiles = dotfiles_dir();
    tracing::debug!("dotfiles dir: {}", dotfiles.display());

    match cli.command {
        Command::Sync(args) => {
            let manifest = load_manifest(&dotfiles)?;
            let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("no $HOME"))?;

            let modules_to_sync: Vec<String> = if args.modules.is_empty() {
                manifest.modules.keys().cloned().collect()
            } else {
                args.modules.clone()
            };

            for name in &modules_to_sync {
                match manifest.modules.get(name) {
                    Some(module) if module.enabled => {
                        linker::sync_module(name, module, &dotfiles, &home)?;
                    }
                    Some(_) => warn!("Module '{}' is disabled, skipping", name),
                    None => warn!("Unknown module '{}', skipping", name),
                }
            }

            if let Some(machine_name) = args.machine {
                let mc = load_machine(&dotfiles, &machine_name)?;
                head!("Applying machine symlinks for '{}'", machine_name);
                linker::apply_machine_symlinks(&dotfiles, &mc, &home)?;
            }
        }

        Command::Install(args) => {
            let machine_name = args.machine.unwrap_or_else(|| {
                // Ask the user which machine to install
                arrow!("No machine specified. Type the name of the machine or press Enter to use 'desktop':");
                let mut input = String::new();
                loop {
                    match io::stdin().read_line(&mut input) {
                        Ok(_) => {
                            let name = input.trim();
                            if name.is_empty() {
                                return "desktop".to_string();
                            }
                            return name.to_string();
                        }
                        Err(e) => {
                            err!("Failed to read input: {}", e);
                            continue;
                        }
                    }
                }
            });

            let manifest = load_manifest(&dotfiles)?;

            let modules_to_install: Vec<String> = if args.modules.is_empty() {
                manifest.modules.keys().cloned().collect()
            } else {
                args.modules.clone()
            };

            for name in &modules_to_install {
                match manifest.modules.get(name) {
                    Some(module) if module.enabled => {
                        head!("Module: {}", name);
                        if !module.hooks.pre_install.is_empty() {
                            arrow!("Running pre-install hooks");
                        }
                        hooks::run(&module.hooks.pre_install, "pre_install")?;
                        installer::install_packages(module).await?;
                        if !module.hooks.post_install.is_empty() {
                            arrow!("Running post-install hooks");
                        }
                        hooks::run(&module.hooks.post_install, "post_install")?;
                    }
                    Some(_) => warn!("Module '{}' is disabled, skipping", name),
                    None => warn!("Unknown module '{}', skipping", name),
                }
            }

            let mc = load_machine(&dotfiles, &machine_name)?;
            if !mc.packages.extra.is_empty() {
                head!("Extra packages for '{}'", machine_name);
                installer::install_extra(&mc.packages.extra).await?;
            }
            head!("Setting up dots service");
            if let Err(e) = daemon::setup_service(&dotfiles) {
                err!("Service setup failed: {}", e);
            }
        }

        Command::Status => {
            print_status(&dotfiles)?;
        }

        Command::Packages(args) => {
            print_packages(&dotfiles, args.check)?;
        }

        Command::Wallpaper { cmd } => match cmd {
            WallpaperCmd::Register { path, name } => wallpaper::register(&path, name.as_deref())?,
            WallpaperCmd::Set { name } => wallpaper::set(&name)?,
            WallpaperCmd::List => wallpaper::list()?,
            WallpaperCmd::Mode { mode } => wallpaper::set_mode(&mode)?,
        },

        Command::Theme { cmd } => match cmd {
            ThemeCmd::Dark => theme::set(true)?,
            ThemeCmd::Light => theme::set(false)?,
        },

        Command::Daemon => {
            daemon::run().await?;
        }

        Command::Syncthing { cmd } => match cmd {
            SyncthingCmd::Install => syncthing::install().await?,
            SyncthingCmd::Start => syncthing::start().await?,
            SyncthingCmd::Stop => syncthing::stop()?,
            SyncthingCmd::Status => syncthing::status()?,
        },
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// dots status
// ---------------------------------------------------------------------------

fn print_status(dotfiles: &Path) -> Result<()> {
    let manifest = load_manifest(dotfiles)?;
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("no $HOME"))?;

    println!(
        "\x1b[1m{:<16} {:<10} {:<12} {:<8} SYMLINKS\x1b[0m",
        "MODULE", "STATUS", "CONFIGS", "PKGS"
    );
    println!("{}", "─".repeat(64));

    let mut names: Vec<&String> = manifest.modules.keys().collect();
    names.sort();

    for name in names {
        let module = &manifest.modules[name];
        let enabled = if module.enabled {
            "\x1b[32menabled\x1b[0m"
        } else {
            "\x1b[33mdisabled\x1b[0m"
        };
        let cfg_dir = dotfiles.join("configs").join(module.configs_dir(name));
        let has_cfg = if cfg_dir.exists() {
            "\x1b[32myes\x1b[0m"
        } else {
            "\x1b[31mno\x1b[0m"
        };
        let pkg_cnt = module.packages.len() + module.aur_packages.len();

        let mut unlinked = 0usize;
        if cfg_dir.exists() {
            let cfg_canon = cfg_dir.canonicalize().unwrap_or_default();
            for entry in walkdir::WalkDir::new(&cfg_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
            {
                if let Ok(src) = entry.path().canonicalize()
                    && let Ok(rel) = src.strip_prefix(&cfg_canon)
                    && !home.join(rel).is_symlink()
                {
                    unlinked += 1;
                }
            }
        }

        let symlinks = if unlinked == 0 {
            "\x1b[32m✓\x1b[0m".to_string()
        } else {
            format!("\x1b[31m{} unlinked\x1b[0m", unlinked)
        };

        println!(
            "{:<16} {:<18} {:<20} {:<8} {}",
            name, enabled, has_cfg, pkg_cnt, symlinks
        );
    }

    // Daemon health check
    println!();
    println!("\x1b[1mDaemons\x1b[0m");
    println!("{}", "─".repeat(40));

    let daemons: &[(&str, &str)] = &[
        ("waybar", "waybar"),
        ("dunst", "dunst"),
        ("awww-daemon", "awww-daemon"),
        ("nm-applet", "nm-applet"),
        ("udiskie", "udiskie"),
        ("nextcloud", "nextcloud"),
        ("wl-paste", "wl-paste"),
        ("polkit", "polkitd"),
        ("dots", "dots daemon"),
    ];

    for (label, pattern) in daemons {
        let running = std::process::Command::new("pgrep")
            .args(["-f", pattern])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if running {
            println!("  \x1b[32m✓\x1b[0m  {}", label);
        } else {
            println!("  \x1b[31m✗\x1b[0m  {} \x1b[2m(not running)\x1b[0m", label);
        }
    }

    // Service status
    println!();
    println!("\x1b[1mService\x1b[0m");
    println!("{}", "─".repeat(40));
    let enabled = std::process::Command::new("systemctl")
        .args(["--user", "is-enabled", "dots"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".into());
    let active = std::process::Command::new("systemctl")
        .args(["--user", "is-active", "dots"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".into());

    let color = |s: &str| match s {
        "enabled" | "active" => format!("\x1b[32m{}\x1b[0m", s),
        "disabled" | "inactive" => format!("\x1b[33m{}\x1b[0m", s),
        _ => format!("\x1b[31m{}\x1b[0m", s),
    };
    println!("  enabled: {}  active: {}", color(&enabled), color(&active));

    Ok(())
}

// ---------------------------------------------------------------------------
// dots packages
// ---------------------------------------------------------------------------

fn print_packages(dotfiles: &Path, check: bool) -> Result<()> {
    let manifest = load_manifest(dotfiles)?;

    if !check {
        let mut pacman_pkgs: Vec<String> = Vec::new();
        let mut aur_pkgs: Vec<String> = Vec::new();

        let mut names: Vec<&String> = manifest.modules.keys().collect();
        names.sort();

        for name in names {
            let module = &manifest.modules[name];
            if !module.enabled {
                continue;
            }
            pacman_pkgs.extend(module.packages.iter().cloned());
            aur_pkgs.extend(module.aur_packages.iter().cloned());
        }

        pacman_pkgs.sort();
        pacman_pkgs.dedup();
        aur_pkgs.sort();
        aur_pkgs.dedup();

        println!("\x1b[1m{:<40} AUR\x1b[0m", "PACMAN");
        println!("{}", "─".repeat(70));

        let rows = pacman_pkgs.len().max(aur_pkgs.len());
        for i in 0..rows {
            println!(
                "{:<40} {}",
                pacman_pkgs.get(i).map(|s| s.as_str()).unwrap_or(""),
                aur_pkgs.get(i).map(|s| s.as_str()).unwrap_or(""),
            );
        }
    } else {
        let output = std::process::Command::new("pacman")
            .args(["-Qq"])
            .output()?;
        if !output.status.success() {
            anyhow::bail!("pacman -Qq failed");
        }

        let installed: std::collections::HashSet<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|l| l.to_string())
            .collect();

        let mut missing: Vec<String> = Vec::new();
        for module in manifest.modules.values() {
            if !module.enabled {
                continue;
            }
            for pkg in module.packages.iter().chain(module.aur_packages.iter()) {
                if !installed.contains(pkg) {
                    missing.push(pkg.clone());
                }
            }
        }

        missing.sort();
        missing.dedup();

        if missing.is_empty() {
            ok!("All packages installed");
        } else {
            println!("\x1b[1mMissing packages:\x1b[0m");
            for pkg in &missing {
                println!("  \x1b[31m✗\x1b[0m  {}", pkg);
            }
        }
    }

    Ok(())
}
