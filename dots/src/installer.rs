use crate::config::Module;
use anyhow::Result;
use std::process::Command;

pub fn aur_helper() -> Option<&'static str> {
    ["paru", "yay"].into_iter().find(|h| {
        Command::new("which")
            .arg(h)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    })
}

pub async fn install_packages(module: &Module) -> Result<()> {
    if !module.packages.is_empty() {
        println!(
            "  → Installing pacman packages: {}",
            module.packages.join(" ")
        );
        let status = Command::new("sudo")
            .args(["pacman", "-S", "--needed", "--noconfirm"])
            .args(&module.packages)
            .status()?;
        if !status.success() {
            anyhow::bail!("pacman install failed");
        }
    }

    if !module.aur_packages.is_empty() {
        match aur_helper() {
            Some(helper) => {
                println!(
                    "  → Installing AUR packages via {}: {}",
                    helper,
                    module.aur_packages.join(" ")
                );
                let status = Command::new(helper)
                    .args(["-S", "--needed", "--noconfirm"])
                    .args(&module.aur_packages)
                    .status()?;
                if !status.success() {
                    anyhow::bail!("{} install failed", helper);
                }
            }
            None => {
                eprintln!(
                    "  ~ Warning: no AUR helper found; skipping: {}",
                    module.aur_packages.join(" ")
                );
            }
        }
    }

    Ok(())
}

pub async fn install_extra(pkgs: &[String]) -> Result<()> {
    if pkgs.is_empty() {
        return Ok(());
    }
    println!("  → Installing extra packages: {}", pkgs.join(" "));
    let status = Command::new("sudo")
        .args(["pacman", "-S", "--needed", "--noconfirm"])
        .args(pkgs)
        .status()?;
    if !status.success() {
        anyhow::bail!("pacman install failed for extra packages");
    }
    Ok(())
}
