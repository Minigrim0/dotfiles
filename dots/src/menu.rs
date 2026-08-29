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

/// Escape text for use inside Pango markup.
fn pango_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Grid variant: square tiles with a large icon above a small label.
/// Uses the dedicated menu conf + matugen-rendered style; returns the label.
pub fn wofi_grid(prompt: &str, items: &[(&str, &str)]) -> Option<String> {
    let home = dirs::home_dir()?;
    let lines: Vec<String> = items
        .iter()
        .map(|(icon, label)| {
            format!(
                "<span font=\"26\">{}</span>&#10;<span font=\"10\">{}</span>",
                pango_escape(icon),
                pango_escape(label)
            )
        })
        .collect();

    let mut child = Command::new("wofi")
        .args(["--dmenu", "--prompt", prompt, "--conf"])
        .arg(home.join(".config/wofi/menu-conf"))
        .arg("--style")
        .arg(home.join(".config/wofi/menu.css"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .ok()?;
    child
        .stdin
        .take()?
        .write_all(lines.join("\n").as_bytes())
        .ok()?;
    let out = child.wait_with_output().ok()?;
    let choice = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if choice.is_empty() {
        return None;
    }
    items
        .iter()
        .find(|(_, label)| choice.contains(&pango_escape(label)))
        .map(|(_, label)| label.to_string())
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
    let items = [("󰖔", "Dark"), ("󰖨", "Light"), ("󰔎", "Toggle")];
    match wofi_grid("theme", &items).as_deref() {
        Some("Dark") => theme::set(true),
        Some("Light") => theme::set(false),
        Some("Toggle") => theme::toggle(),
        _ => Ok(()),
    }
}

fn monitors_menu() -> Result<()> {
    let items = [("󰃚", "25%"), ("󰃝", "50%"), ("󰃟", "75%"), ("󰃠", "100%")];
    if let Some(choice) = wofi_grid("brightness", &items) {
        monitor::brightness(choice.trim_end_matches('%'), None, true)?;
    }
    Ok(())
}

/// The settings hub: a launchpad-style grid of square tiles.
pub fn show() -> Result<()> {
    let items = [
        ("󰸉", "Wallpaper"),
        ("󰔎", "Theme"),
        ("󰍹", "Monitors"),
        ("󰌾", "Idle & lock"),
        ("󰖔", "Night light"),
        ("󰊴", "Game mode"),
        ("󰌌", "Keybinds"),
        ("⏻", "Power"),
    ];

    match wofi_grid("dots", &items).as_deref() {
        Some("Wallpaper") => wallpaper_menu(),
        Some("Theme") => theme_menu(),
        Some("Monitors") => monitors_menu(),
        Some("Idle & lock") => toggle_idle(),
        Some("Night light") => toggle_night_light(),
        Some("Game mode") => gamemode::toggle(),
        Some("Keybinds") => keys::show(),
        Some("Power") => spawn_detached("wlogout", &[]),
        _ => Ok(()),
    }
}
