use crate::audit;
use crate::config::Manifest;
use anyhow::Result;
use std::path::Path;
use std::process::Command;

/// (label, pgrep pattern) — shared with `dots status`.
pub const DAEMONS: &[(&str, &str)] = &[
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

fn pass(msg: &str) {
    println!("  \x1b[32m✓\x1b[0m  {}", msg);
}

fn fail(msg: &str) {
    println!("  \x1b[31m✗\x1b[0m  {}", msg);
}

fn hint(msg: &str) {
    println!("     \x1b[2m{}\x1b[0m", msg);
}

fn heading(title: &str) {
    println!("\n\x1b[1m{}\x1b[0m", title);
    println!("{}", "─".repeat(48));
}

fn cmd_stdout(cmd: &str, args: &[&str]) -> String {
    Command::new(cmd)
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default()
}

/// Symlinks in $HOME that exist but whose target is gone or outside the repo.
fn check_symlinks(manifest: &Manifest, dotfiles: &Path, home: &Path) {
    heading("Symlinks");
    let repo = dotfiles
        .canonicalize()
        .unwrap_or_else(|_| dotfiles.to_path_buf());
    let mut broken = 0usize;
    let mut foreign = 0usize;

    for (name, module) in &manifest.modules {
        if !module.enabled {
            continue;
        }
        let cfg_dir = dotfiles.join("configs").join(module.configs_dir(name));
        if !cfg_dir.exists() {
            continue;
        }
        let Ok(cfg_canon) = cfg_dir.canonicalize() else {
            continue;
        };
        for entry in walkdir::WalkDir::new(&cfg_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let Ok(src) = entry.path().canonicalize() else {
                continue;
            };
            let Ok(rel) = src.strip_prefix(&cfg_canon) else {
                continue;
            };
            let dst = home.join(rel);
            if !dst.is_symlink() {
                continue;
            }
            match std::fs::canonicalize(&dst) {
                Err(_) => {
                    fail(&format!("broken link: {}", dst.display()));
                    broken += 1;
                }
                Ok(target) if !target.starts_with(&repo) => {
                    fail(&format!(
                        "points outside repo: {} → {}",
                        dst.display(),
                        target.display()
                    ));
                    foreign += 1;
                }
                Ok(_) => {}
            }
        }
    }
    if broken == 0 && foreign == 0 {
        pass("all module symlinks resolve into the repo");
    } else {
        hint("fix: dots sync");
    }
}

fn check_failed_units() {
    heading("Systemd user units");
    let out = cmd_stdout("systemctl", &["--user", "--failed", "--no-legend"]);
    let failed: Vec<&str> = out.lines().filter(|l| !l.trim().is_empty()).collect();
    if failed.is_empty() {
        pass("no failed user units");
    } else {
        for line in failed {
            fail(line.trim());
        }
    }
}

fn check_daemons() {
    heading("Daemons");
    for (label, pattern) in DAEMONS {
        let running = Command::new("pgrep")
            .args(["-f", pattern])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if running {
            pass(label);
        } else {
            fail(&format!("{} (not running)", label));
        }
    }
}

fn check_gpu() {
    heading("GPU (amdgpu)");
    let out = cmd_stdout("journalctl", &["-k", "-b", "0", "-p", "err", "--no-pager"]);
    let errors = out
        .lines()
        .filter(|l| {
            l.contains("SMU: No response")
                || l.contains("DMUB")
                || (l.contains("ring") && l.contains("reset"))
        })
        .count();
    if errors > 10 {
        fail(&format!("{} SMU/DMUB/ring-reset errors this boot", errors));
        hint("known Navi 33 gfxoff hang — boot with amdgpu.gfxoff=0");
    } else if errors > 0 {
        pass(&format!("only {} GPU error lines this boot", errors));
    } else {
        pass("no GPU errors this boot");
    }
}

fn check_journal_size() {
    heading("Journal");
    let out = cmd_stdout("journalctl", &["--disk-usage"]);
    // "Archived and active journals take up 890.7M in the file system."
    let size = out
        .split_whitespace()
        .find(|w| w.ends_with('M') || w.ends_with('G'))
        .unwrap_or("?")
        .to_string();
    let too_big = size.ends_with('G')
        || size
            .trim_end_matches('M')
            .parse::<f64>()
            .map(|m| m > 500.0)
            .unwrap_or(false);
    if too_big {
        fail(&format!("journal uses {}", size));
        hint("cap it: SystemMaxUse=200M in /etc/systemd/journald.conf.d/");
    } else {
        pass(&format!("journal uses {}", size));
    }
}

fn check_packages(manifest: &Manifest) {
    heading("Packages");
    match audit::run(manifest) {
        Ok(report) => {
            if report.missing.is_empty() {
                pass("all manifest packages installed");
            } else {
                fail(&format!(
                    "{} manifest packages missing",
                    report.missing.len()
                ));
                hint("fix: dots install");
            }
            if report.orphans.is_empty() {
                pass("no orphaned packages");
            } else {
                fail(&format!("{} orphaned packages", report.orphans.len()));
                hint("inspect: dots packages --audit");
            }
        }
        Err(e) => fail(&format!("audit failed: {}", e)),
    }
}

pub fn run(manifest: &Manifest, dotfiles: &Path, home: &Path) -> Result<()> {
    check_symlinks(manifest, dotfiles, home);
    check_daemons();
    check_failed_units();
    check_gpu();
    check_journal_size();
    check_packages(manifest);
    println!();
    Ok(())
}
