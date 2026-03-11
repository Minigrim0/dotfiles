use crate::installer::aur_helper;
use anyhow::{Context, Result};
use std::process::Command;

pub async fn install() -> Result<()> {
    // Check if syncthing is already installed
    let installed = Command::new("which")
        .arg("syncthing")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if installed {
        println!("  ✓ syncthing already installed");
        return Ok(());
    }

    match aur_helper() {
        Some(helper) => {
            println!("  → Installing syncthing via {}", helper);
            let status = Command::new(helper)
                .args(["-S", "--needed", "--noconfirm", "syncthing"])
                .status()?;
            if !status.success() {
                anyhow::bail!("failed to install syncthing");
            }
        }
        None => {
            println!("  → Installing syncthing via pacman");
            let status = Command::new("sudo")
                .args(["pacman", "-S", "--needed", "--noconfirm", "syncthing"])
                .status()?;
            if !status.success() {
                anyhow::bail!("failed to install syncthing");
            }
        }
    }
    Ok(())
}

pub async fn start() -> Result<()> {
    let status = Command::new("systemctl")
        .args(["--user", "start", "syncthing"])
        .status()
        .context("systemctl start syncthing")?;
    if !status.success() {
        anyhow::bail!("systemctl start syncthing failed");
    }
    println!("  ✓ syncthing started");
    Ok(())
}

pub fn stop() -> Result<()> {
    let status = Command::new("systemctl")
        .args(["--user", "stop", "syncthing"])
        .status()
        .context("systemctl stop syncthing")?;
    if !status.success() {
        anyhow::bail!("systemctl stop syncthing failed");
    }
    println!("  ✓ syncthing stopped");
    Ok(())
}

pub fn status() -> Result<()> {
    Command::new("systemctl")
        .args(["--user", "status", "syncthing"])
        .status()
        .context("systemctl status syncthing")?;
    Ok(())
}
