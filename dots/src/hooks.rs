use anyhow::{Context, Result};
use std::process::Command;

pub fn run(cmds: &[String], label: &str) -> Result<()> {
    for cmd in cmds {
        println!("  → [{}] {}", label, cmd);
        let status = Command::new("sh")
            .args(["-c", cmd])
            .status()
            .with_context(|| format!("failed to spawn: {}", cmd))?;
        if !status.success() {
            anyhow::bail!("hook failed ({}): {}", status, cmd);
        }
    }
    Ok(())
}
