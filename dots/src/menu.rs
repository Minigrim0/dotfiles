use crate::{bar, display, gamemode, inhibit, keys, monitor, power, theme, wallpaper};
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

/// Free-text prompt. `secret` masks the input, for Wi-Fi passphrases.
/// Returns None if dismissed or left empty.
pub fn wofi_input(prompt: &str, secret: bool) -> Option<String> {
    let mut cmd = Command::new("wofi");
    cmd.args(["--dmenu", "--prompt", prompt, "--lines", "1"]);
    if secret {
        cmd.arg("--password");
    }
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .ok()?;
    // wofi --dmenu reads stdin to EOF; an empty list gives a bare input field.
    drop(child.stdin.take());
    let out = child.wait_with_output().ok()?;
    let value = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!value.is_empty()).then_some(value)
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
    /// The tile labels are centred with space padding, so they need a grid.
    const M: &str = "JetBrainsMono Nerd Font";

    let home = dirs::home_dir()?;
    let lines: Vec<String> = items
        .iter()
        .map(|(icon, label)| {
            // Pango left-justifies the lines of a multi-line label, so pad the
            // icon line to sit over the label's midpoint. In a monospace font
            // a 26pt glyph is 2.6 label-sized (10pt) cells wide — which is why
            // the face is named here rather than inherited: the surrounding UI
            // is a proportional sans now, and the padding maths needs a grid.
            let pad = ((label.chars().count() as f32 - 2.6) / 2.0).round().max(0.0) as usize;
            format!(
                "<span font=\"{M} 10\">{}</span><span font=\"{M} 26\">{}</span>&#10;<span font=\"{M} 10\">{}</span>",
                " ".repeat(pad),
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

fn spawn_detached(cmd: &str, args: &[&str]) -> Result<()> {
    Command::new(cmd)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("spawning {}", cmd))?;
    Ok(())
}

/// Warm temperature applied when night light is switched on by hand. The
/// scheduled one lives in hyprsunset.conf.
pub const NIGHT_TEMP: u32 = 4000;

/// Neutral. hyprsunset reports 6000 when the identity matrix is in use.
pub const NEUTRAL_TEMP: u32 = 6000;

/// Current colour temperature, from the running hyprsunset daemon.
pub fn night_temperature() -> Option<u32> {
    let out = Command::new("hyprctl")
        .args(["hyprsunset", "temperature"])
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().parse().ok())
        .flatten()
}

/// `None` toggles; `Some(true)` / `Some(false)` set explicitly.
///
/// Talks to the daemon rather than killing it: hyprsunset also runs a schedule
/// (hyprsunset.conf), and pkill would silently take that down with it.
pub fn set_night_light(target: Option<bool>) -> Result<()> {
    let temp = night_temperature();
    let on = temp.is_some_and(|t| t < NEUTRAL_TEMP);
    let want = target.unwrap_or(!on);

    if temp.is_none() {
        anyhow::bail!("hyprsunset is not running");
    }

    if want != on {
        // `identity` applies a neutral matrix but leaves the daemon still
        // *reporting* the warm value, so the indicator would never clear.
        // Setting the temperature back is both effective and observable.
        let target_temp = if want { NIGHT_TEMP } else { NEUTRAL_TEMP };
        let args = ["hyprsunset", "temperature", &target_temp.to_string()].map(String::from);
        let status = Command::new("hyprctl")
            .args(&args)
            .status()
            .context("running hyprctl hyprsunset")?;
        anyhow::ensure!(status.success(), "hyprctl hyprsunset failed");

        if want {
            notify(&format!("Night light ON ({}K)", NIGHT_TEMP));
        } else {
            notify("Night light OFF");
        }
    }
    bar::refresh(bar::SIG_NIGHT);
    Ok(())
}

pub fn toggle_night_light() -> Result<()> {
    set_night_light(None)
}

/// Pause / resume dunst. `None` toggles. Until now this had no feedback at
/// all — the bar's DND indicator is the other half of this.
pub fn set_dnd(target: Option<bool>) -> Result<()> {
    let arg = match target {
        None => "toggle",
        Some(true) => "true",
        Some(false) => "false",
    };
    let status = Command::new("dunstctl")
        .args(["set-paused", arg])
        .status()
        .context("running dunstctl — is dunst running?")?;
    anyhow::ensure!(status.success(), "dunstctl set-paused {} failed", arg);

    let paused = Command::new("dunstctl")
        .arg("is-paused")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "true")
        .unwrap_or(false);

    if paused {
        notify("Notifications paused");
    } else {
        notify("Notifications resumed");
    }
    bar::refresh(bar::SIG_DND);
    Ok(())
}

pub fn toggle_dnd() -> Result<()> {
    set_dnd(None)
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
    let mut items = vec![
        ("󰖔", "Dark"),
        ("󰖨", "Light"),
        ("󰔎", "Toggle"),
        ("󰸉", "Auto"),
    ];
    for name in theme::preset_names() {
        items.push(("󰏘", name));
    }
    match wofi_grid("theme", &items).as_deref() {
        Some("Dark") => theme::set(true),
        Some("Light") => theme::set(false),
        Some("Toggle") => theme::toggle(),
        Some("Auto") => theme::auto(),
        Some(preset) => theme::set_preset(preset),
        None => Ok(()),
    }
}

/// DDC/CI brightness only. Geometry lives in `dots displays`, because a
/// brightness step and a resolution change have nothing to do with each other
/// beyond happening to the same panel.
fn brightness_menu() -> Result<()> {
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
        ("󰍹", "Displays"),
        ("󰃟", "Brightness"),
        ("󰾅", "Power profile"),
        ("󰅶", "Keep awake"),
        ("󰖔", "Night light"),
        ("󰂛", "Do not disturb"),
        ("󰊴", "Game mode"),
        ("󰌌", "Keybinds"),
        ("⏻", "Session"),
    ];

    match wofi_grid("dots", &items).as_deref() {
        Some("Wallpaper") => wallpaper_menu(),
        Some("Theme") => theme_menu(),
        Some("Displays") => display::menu(),
        Some("Brightness") => brightness_menu(),
        Some("Power profile") => power::menu(),
        Some("Keep awake") => inhibit::set(None),
        Some("Night light") => toggle_night_light(),
        Some("Do not disturb") => toggle_dnd(),
        Some("Game mode") => gamemode::toggle(),
        Some("Keybinds") => keys::show(),
        Some("Session") => spawn_detached("wlogout", &[]),
        _ => Ok(()),
    }
}
