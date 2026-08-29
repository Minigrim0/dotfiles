use crate::{gamemode, keys, monitor, theme, wallpaper};
use anyhow::{Context, Result};
use std::io::Write;
use std::process::{Command, Stdio};

/// Pipe items into `wofi --dmenu` and return the selection (None if dismissed).
pub fn wofi_pick(prompt: &str, items: &[String]) -> Option<String> {
    let mut child = Command::new("wofi")
        .args(["--dmenu", "--prompt", prompt])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .ok()?;
    child
        .stdin
        .take()?
        .write_all(items.join("\n").as_bytes())
        .ok()?;
    let out = child.wait_with_output().ok()?;
    let choice = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if choice.is_empty() {
        None
    } else {
        Some(choice)
    }
}

fn notify(body: &str) {
    let _ = Command::new("notify-send")
        .args(["-a", "dots", body])
        .status();
}

fn is_running(name: &str) -> bool {
    Command::new("pgrep")
        .args(["-x", name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn spawn_detached(cmd: &str, args: &[&str]) -> Result<()> {
    Command::new(cmd)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("spawning {}", cmd))?;
    Ok(())
}

fn toggle_night_light() -> Result<()> {
    if is_running("hyprsunset") {
        let _ = Command::new("pkill").args(["-x", "hyprsunset"]).status();
        notify("Night light OFF");
    } else {
        spawn_detached("hyprsunset", &["-t", "4000"])?;
        notify("Night light ON (4000K)");
    }
    Ok(())
}

fn toggle_idle() -> Result<()> {
    if is_running("hypridle") {
        let _ = Command::new("pkill").args(["-x", "hypridle"]).status();
        notify("Idle inhibited — screen stays on");
    } else {
        spawn_detached("hypridle", &[])?;
        notify("Idle management ON");
    }
    Ok(())
}

pub fn wallpaper_menu() -> Result<()> {
    let names = wallpaper::names()?;
    anyhow::ensure!(!names.is_empty(), "no wallpapers registered");
    if let Some(choice) = wofi_pick("wallpaper", &names) {
        wallpaper::set(&choice)?;
    }
    Ok(())
}

fn theme_menu() -> Result<()> {
    let items = vec![
        "󰖔  Dark".to_string(),
        "󰖨  Light".to_string(),
        "󰔎  Toggle".to_string(),
    ];
    match wofi_pick("theme", &items).as_deref() {
        Some(s) if s.contains("Dark") => theme::set(true),
        Some(s) if s.contains("Light") => theme::set(false),
        Some(s) if s.contains("Toggle") => theme::toggle(),
        _ => Ok(()),
    }
}

fn monitors_menu() -> Result<()> {
    let items = vec![
        "󰃚  25%".to_string(),
        "󰃝  50%".to_string(),
        "󰃟  75%".to_string(),
        "󰃠  100%".to_string(),
    ];
    if let Some(choice) = wofi_pick("brightness", &items) {
        let pct = choice
            .split_whitespace()
            .last()
            .unwrap_or("")
            .trim_end_matches('%');
        monitor::brightness(pct, None, true)?;
    }
    Ok(())
}

/// The settings hub: a small wofi-driven tree over dots + system tools.
pub fn show() -> Result<()> {
    let items = vec![
        "󰸉  Wallpaper".to_string(),
        "󰔎  Theme".to_string(),
        "󰍹  Monitors".to_string(),
        "󰌾  Idle & lock".to_string(),
        "󰖔  Night light".to_string(),
        "󰊴  Game mode".to_string(),
        "󰌌  Keybinds".to_string(),
        "⏻  Power".to_string(),
    ];

    match wofi_pick("dots", &items).as_deref() {
        Some(s) if s.contains("Wallpaper") => wallpaper_menu(),
        Some(s) if s.contains("Theme") => theme_menu(),
        Some(s) if s.contains("Monitors") => monitors_menu(),
        Some(s) if s.contains("Idle") => toggle_idle(),
        Some(s) if s.contains("Night") => toggle_night_light(),
        Some(s) if s.contains("Game") => gamemode::toggle(),
        Some(s) if s.contains("Keybinds") => keys::show(),
        Some(s) if s.contains("Power") => spawn_detached("wlogout", &[]),
        _ => Ok(()),
    }
}
