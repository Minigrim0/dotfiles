use crate::{bar, ok};
use anyhow::{Context, Result};
use std::process::Command;

fn notify(body: &str) {
    let _ = Command::new("notify-send")
        .args([
            "-a",
            "dots",
            "-h",
            "string:x-dunst-stack-tag:dots-game",
            body,
        ])
        .status();
}

/// Whether the compositor is currently stripped down for games.
///
/// Inferred from `animations:enabled` rather than stored, so an unrelated
/// `hyprctl reload` silently leaves game mode — which is exactly why the bar
/// shows the state instead of leaving you to guess.
pub fn is_on() -> Result<bool> {
    let out = Command::new("hyprctl")
        .args(["getoption", "animations:enabled", "-j"])
        .output()
        .context("running hyprctl — is Hyprland running?")?;
    let opt: serde_json::Value =
        serde_json::from_slice(&out.stdout).context("parsing hyprctl output")?;
    Ok(opt.get("int").and_then(|v| v.as_i64()).unwrap_or(1) == 0)
}

/// Toggle game mode: animations, blur, shadows and dim off for max FPS.
/// Restores everything with a config reload.
pub fn toggle() -> Result<()> {
    if is_on()? {
        let status = Command::new("hyprctl")
            .arg("reload")
            .status()
            .context("running hyprctl reload")?;
        anyhow::ensure!(status.success(), "hyprctl reload failed");
        notify("Game mode OFF");
        ok!("Game mode OFF (config reloaded)");
    } else {
        let status = Command::new("hyprctl")
            .args([
                "--batch",
                "keyword animations:enabled 0; \
                 keyword decoration:blur:enabled 0; \
                 keyword decoration:shadow:enabled 0; \
                 keyword decoration:dim_inactive 0",
            ])
            .status()
            .context("running hyprctl --batch")?;
        anyhow::ensure!(status.success(), "hyprctl --batch failed");
        notify("Game mode ON");
        ok!("Game mode ON (animations, blur, shadows off)");
    }

    bar::refresh(bar::SIG_GAME);
    Ok(())
}
