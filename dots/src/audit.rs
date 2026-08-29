use crate::config::Manifest;
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::process::Command;

pub struct AuditReport {
    /// In manifest but not installed
    pub missing: Vec<String>,
    /// Explicitly installed but not in the manifest (adoption candidates)
    pub unmanaged: Vec<String>,
    /// Installed as dependency, required by nothing
    pub orphans: Vec<String>,
    /// Foreign (AUR/manual) packages not tracked in any aur_packages
    pub foreign: Vec<String>,
}

/// Pure reconciliation logic, separated for testing.
pub fn compute(
    manifest_all: &HashSet<String>,
    manifest_aur: &HashSet<String>,
    installed: &HashSet<String>,
    explicit: &HashSet<String>,
    orphans: &[String],
    foreign: &HashSet<String>,
) -> AuditReport {
    let ignored: HashSet<&str> = HashSet::from(["base", "base-devel"]);

    let mut missing: Vec<String> = manifest_all.difference(installed).cloned().collect();
    missing.sort();

    let mut unmanaged: Vec<String> = explicit
        .iter()
        .filter(|p| !manifest_all.contains(*p) && !ignored.contains(p.as_str()))
        .cloned()
        .collect();
    unmanaged.sort();

    let mut orphans = orphans.to_vec();
    orphans.sort();

    let mut foreign: Vec<String> = foreign.difference(manifest_aur).cloned().collect();
    foreign.sort();

    AuditReport {
        missing,
        unmanaged,
        orphans,
        foreign,
    }
}

fn pacman_set(args: &[&str]) -> Result<HashSet<String>> {
    let out = Command::new("pacman")
        .args(args)
        .output()
        .with_context(|| format!("running pacman {}", args.join(" ")))?;
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| l.to_string())
        .collect())
}

/// Gather manifest package sets from enabled modules.
pub fn manifest_sets(manifest: &Manifest) -> (HashSet<String>, HashSet<String>) {
    let mut all = HashSet::new();
    let mut aur = HashSet::new();
    for module in manifest.modules.values() {
        if !module.enabled {
            continue;
        }
        all.extend(module.packages.iter().cloned());
        all.extend(module.aur_packages.iter().cloned());
        aur.extend(module.aur_packages.iter().cloned());
    }
    (all, aur)
}

pub fn run(manifest: &Manifest) -> Result<AuditReport> {
    let (manifest_all, manifest_aur) = manifest_sets(manifest);
    let installed = pacman_set(&["-Qq"])?;
    let explicit = pacman_set(&["-Qeq"])?;
    let foreign = pacman_set(&["-Qmq"])?;
    // -Qtdq exits 1 when there are no orphans; treat output as-is
    let orphans: Vec<String> = Command::new("pacman")
        .args(["-Qtdq"])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(|l| l.to_string())
                .collect()
        })
        .unwrap_or_default();

    Ok(compute(
        &manifest_all,
        &manifest_aur,
        &installed,
        &explicit,
        &orphans,
        &foreign,
    ))
}

pub fn print(report: &AuditReport) {
    let section = |title: &str, items: &[String], hint: &str| {
        println!("\n\x1b[1m{} \x1b[0m\x1b[2m({})\x1b[0m", title, items.len());
        println!("{}", "─".repeat(56));
        if items.is_empty() {
            println!("  \x1b[32m✓\x1b[0m  nothing");
        } else {
            for chunk in items.chunks(3) {
                println!(
                    "  {}",
                    chunk
                        .iter()
                        .map(|s| format!("{:<26}", s))
                        .collect::<String>()
                );
            }
            if !hint.is_empty() {
                println!("  \x1b[2m{}\x1b[0m", hint);
            }
        }
    };

    section(
        "MISSING — in modules.toml, not installed",
        &report.missing,
        "fix: dots install",
    );
    section(
        "UNMANAGED — explicitly installed, not in modules.toml",
        &report.unmanaged,
        "adopt into a module, or prune: sudo pacman -Rns <pkg>",
    );
    section(
        "ORPHANS — dependencies nothing requires",
        &report.orphans,
        "prune: pacman -Qtdq | sudo pacman -Rns -",
    );
    section(
        "FOREIGN — AUR/manual, not tracked in aur_packages",
        &report.foreign,
        "adopt into a module's aur_packages, or prune",
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(items: &[&str]) -> HashSet<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn reconciles_sets() {
        let manifest_all = set(&["kitty", "waybar", "notinstalled"]);
        let manifest_aur = set(&["obsidian"]);
        let installed = set(&["kitty", "waybar", "base", "stray", "obsidian", "paru"]);
        let explicit = set(&["kitty", "base", "stray"]);
        let orphans = vec!["oldlib".to_string()];
        let foreign = set(&["obsidian", "paru"]);

        let r = compute(
            &manifest_all,
            &manifest_aur,
            &installed,
            &explicit,
            &orphans,
            &foreign,
        );
        assert_eq!(r.missing, vec!["notinstalled"]);
        assert_eq!(r.unmanaged, vec!["stray"]); // base excluded, kitty managed
        assert_eq!(r.orphans, vec!["oldlib"]);
        assert_eq!(r.foreign, vec!["paru"]); // obsidian tracked
    }
}
