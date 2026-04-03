use crate::installer::install_extra;
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

    install_extra(&["syncthing".to_string()]).await?;
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
