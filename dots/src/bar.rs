//! `dots bar <topic>` — one JSON line for a waybar `custom/*` module.
//!
//! Every module that uses this is configured `"interval": "once"` with a
//! `signal`, and the dots command that *changes* the state raises it (see
//! [`refresh`]). Nothing here is polled on a timer, which is what retired the
//! five-second DDC read the old brightness module did.
//!
//! An empty `text` means waybar hides the module entirely — that is how
//! "silent until it matters" is implemented.

use crate::config::{dotfiles_dir, load_manifest};
use crate::{audit, gamemode, inhibit, menu, monitor, power, wallpaper};
use anyhow::Result;
use serde::Serialize;
use std::process::Command;

// ---------------------------------------------------------------------------
// Signals — mirrored in configs/waybar/.config/waybar/config.jsonc
// ---------------------------------------------------------------------------

pub const SIG_GAME: u8 = 5;
pub const SIG_DND: u8 = 6;
pub const SIG_NIGHT: u8 = 7;
pub const SIG_BRIGHTNESS: u8 = 8;
pub const SIG_DRIFT: u8 = 9;
/// Raised from the module's own on-click after `paru -Syu` returns — nothing
/// inside dots knows when pacman finished.
#[allow(dead_code)]
pub const SIG_UPDATES: u8 = 10;
pub const SIG_WALLPAPER: u8 = 11;
pub const SIG_POWER: u8 = 12;
pub const SIG_INHIBIT: u8 = 13;

/// Tell waybar that one module's state changed.
///
/// Deliberately infallible: the bar not running is not a failure for the
/// command that called this.
pub fn refresh(signal: u8) {
    let _ = Command::new("pkill")
        .arg(format!("-RTMIN+{}", signal))
        .arg("waybar")
        .status();
}

// ---------------------------------------------------------------------------
// Output shape
// ---------------------------------------------------------------------------

#[derive(Serialize, Default)]
struct Status {
    text: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    tooltip: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    class: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    percentage: Option<u32>,
}

impl Status {
    /// The hidden state: waybar drops modules whose label is empty.
    fn hidden() -> Self {
        Self::default()
    }

    fn new(text: impl Into<String>, tooltip: impl Into<String>, class: &str) -> Self {
        Self {
            text: text.into(),
            tooltip: tooltip.into(),
            class: class.to_string(),
            percentage: None,
        }
    }
}

/// Capture a command's stdout, or None if it could not run or failed.
fn stdout(cmd: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(cmd).args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn emit(topic: &str) -> Result<()> {
    let status = match topic {
        "game" => game(),
        "dnd" => dnd(),
        "night" => night(),
        "brightness" => brightness(),
        "drift" => drift(),
        "updates" => updates(),
        "temp" => temp(),
        "wallpaper" => wallpaper_render(),
        "power" => power_profile(),
        "inhibit" => idle_inhibit(),
        other => anyhow::bail!(
            "unknown bar topic '{}' (game, dnd, night, brightness, drift, updates, temp, \
             wallpaper, power, inhibit)",
            other
        ),
    };
    println!("{}", serde_json::to_string(&status)?);
    Ok(())
}

// ---------------------------------------------------------------------------
// Topics
// ---------------------------------------------------------------------------

/// Visible only while `dots game` has the compositor stripped down. Without
/// this the mode is invisible and survives until an unrelated config reload.
fn game() -> Status {
    if gamemode::is_on().unwrap_or(false) {
        Status::new(
            "󰊴 PERF",
            "Game mode — animations, blur and shadows off\nClick to restore",
            "on",
        )
    } else {
        Status::hidden()
    }
}

/// Visible only while dunst is paused. Counts what you are missing.
fn dnd() -> Status {
    let paused = stdout("dunstctl", &["is-paused"])
        .map(|s| s == "true")
        .unwrap_or(false);
    if !paused {
        return Status::hidden();
    }
    let waiting = stdout("dunstctl", &["count", "waiting"])
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    let text = if waiting > 0 {
        format!("󰂛 {}", waiting)
    } else {
        "󰂛".to_string()
    };
    let tooltip = if waiting > 0 {
        format!(
            "Notifications paused — {} waiting\nClick to resume",
            waiting
        )
    } else {
        "Notifications paused\nClick to resume".to_string()
    };
    Status::new(text, tooltip, "paused")
}

/// Visible only while the screen is actually tinted — it changes every colour
/// you see, so it should be the one thing that says so. Reads the temperature
/// rather than the process, which also makes hyprsunset.conf's evening
/// schedule visible when it fires.
fn night() -> Status {
    match menu::night_temperature() {
        Some(temp) if temp < menu::NEUTRAL_TEMP => Status::new(
            format!("󰖔 {}K", temp),
            format!("Night light — {}K\nClick to turn off", temp),
            "on",
        ),
        _ => Status::hidden(),
    }
}

/// Desktop brightness over DDC/CI. Read once at start and then only when
/// `dots monitor brightness` says it changed.
fn brightness() -> Status {
    match monitor::current_percent() {
        Some(pct) => Status {
            text: format!("󰃞 {}", pct),
            tooltip: format!("Brightness {}%\nScroll to adjust", pct),
            class: String::new(),
            percentage: Some(pct),
        },
        None => Status::hidden(),
    }
}

/// Visible only when the repo and the machine have drifted apart: packages
/// declared but not installed, or configs that are no longer symlinked.
fn drift() -> Status {
    let Ok(dotfiles) = dotfiles_dir() else {
        return Status::hidden();
    };
    let Ok(manifest) = load_manifest(&dotfiles) else {
        return Status::hidden();
    };

    let missing: Vec<String> = audit::run(&manifest).map(|r| r.missing).unwrap_or_default();
    let unlinked = count_unlinked(&manifest, &dotfiles);

    let total = missing.len() + unlinked;
    if total == 0 {
        return Status::hidden();
    }

    let mut lines = Vec::new();
    if !missing.is_empty() {
        lines.push(format!(
            "{} package(s) declared but not installed:\n  {}",
            missing.len(),
            missing.join("\n  ")
        ));
    }
    if unlinked > 0 {
        lines.push(format!("{} config file(s) not symlinked", unlinked));
    }
    lines.push("Click to run dots doctor".to_string());

    Status::new(format!("󰓅 {}", total), lines.join("\n\n"), "drift")
}

/// Config files under a module's tree that are not symlinks in $HOME.
fn count_unlinked(manifest: &crate::config::Manifest, dotfiles: &std::path::Path) -> usize {
    let Some(home) = dirs::home_dir() else {
        return 0;
    };
    let mut unlinked = 0usize;

    for (name, module) in &manifest.modules {
        if !module.enabled {
            continue;
        }
        let cfg_dir = dotfiles.join("configs").join(module.configs_dir(name));
        let Ok(cfg_canon) = cfg_dir.canonicalize() else {
            continue;
        };
        for entry in walkdir::WalkDir::new(&cfg_canon)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            if let Ok(rel) = entry.path().strip_prefix(&cfg_canon)
                && !home.join(rel).is_symlink()
            {
                unlinked += 1;
            }
        }
    }
    unlinked
}

/// Pending updates, but only once there are enough of them to be worth an
/// interruption. A permanent counter is guilt, not information.
const UPDATE_THRESHOLD: usize = 25;

fn updates() -> Status {
    // checkupdates syncs its own temporary database, so it never disturbs
    // pacman's. Called hourly by the bar.
    let Some(out) = stdout("checkupdates", &[]) else {
        return Status::hidden();
    };
    let pending: Vec<&str> = out.lines().filter(|l| !l.trim().is_empty()).collect();
    if pending.len() < UPDATE_THRESHOLD {
        return Status::hidden();
    }

    let preview: Vec<&str> = pending.iter().take(12).copied().collect();
    let more = pending.len().saturating_sub(preview.len());
    let mut tooltip = format!(
        "{} updates pending\n\n{}",
        pending.len(),
        preview.join("\n")
    );
    if more > 0 {
        tooltip.push_str(&format!("\n… and {} more", more));
    }

    Status::new(format!("󰚰 {}", pending.len()), tooltip, "pending")
}

/// Above this, the CPU temperature is worth interrupting you for.
const TEMP_CRITICAL: u32 = 82;

/// Every hwmon sensor, by driver name, in degrees Celsius.
///
/// Looked up by name rather than by index: `hwmon<N>` numbering is not stable
/// across boots, and on this desktop `thermal_zone0` is the *wifi card*, not
/// the CPU — which is exactly the trap a hard-coded index walks into.
fn hwmon_temps() -> Vec<(String, u32)> {
    let Ok(dir) = std::fs::read_dir("/sys/class/hwmon") else {
        return Vec::new();
    };
    let mut sensors = Vec::new();
    for entry in dir.filter_map(|e| e.ok()) {
        let path = entry.path();
        let Ok(name) = std::fs::read_to_string(path.join("name")) else {
            continue;
        };
        if let Ok(raw) = std::fs::read_to_string(path.join("temp1_input"))
            && let Ok(milli) = raw.trim().parse::<u32>()
        {
            sensors.push((name.trim().to_string(), milli / 1000));
        }
    }
    sensors
}

/// First sensor matching `names`, in the order given (i.e. by preference).
fn pick_sensor(sensors: &[(String, u32)], names: &[&str]) -> Option<u32> {
    names.iter().find_map(|wanted| {
        sensors
            .iter()
            .find(|(name, _)| name == wanted)
            .map(|(_, celsius)| *celsius)
    })
}

/// Silent below the threshold. A thermometer that is always on the bar is
/// decoration; one that appears at 82°C is a warning.
fn temp() -> Status {
    let sensors = hwmon_temps();
    let Some(cpu) = pick_sensor(
        &sensors,
        &["k10temp", "coretemp", "zenpower", "cpu_thermal"],
    ) else {
        return Status::hidden();
    };
    let gpu = pick_sensor(&sensors, &["amdgpu", "nvidia", "i915"]);

    let mut tooltip = format!("CPU {}°C", cpu);
    if let Some(gpu) = gpu {
        tooltip.push_str(&format!("\nGPU {}°C", gpu));
    }

    if cpu < TEMP_CRITICAL {
        return Status::hidden();
    }
    Status::new(format!("󰈸 {}°", cpu), tooltip, "critical")
}

/// Visible while `dots wallpaper set` is running matugen / ffmpeg, which takes
/// long enough to wonder whether it worked.
fn wallpaper_render() -> Status {
    if wallpaper::is_rendering() {
        Status::new(
            "󰸉 rendering…",
            "Building palette and reloading apps",
            "busy",
        )
    } else {
        Status::hidden()
    }
}

/// Visible only while the power profile is *not* balanced. Balanced is the
/// default on every machine that has the daemon at all, so showing it would be
/// a permanent icon that means "nothing to see" — and a laptop silently left on
/// performance is exactly the case worth an icon.
fn power_profile() -> Status {
    let Some(profile) = power::current() else {
        return Status::hidden();
    };
    if profile == power::BALANCED {
        return Status::hidden();
    }
    Status::new(
        format!("{} {}", power::icon(&profile), profile),
        format!("Power profile: {}\nClick to change", profile),
        &profile,
    )
}

/// Visible only while `dots inhibit` is holding a lock.
///
/// Distinct from waybar's own `idle_inhibitor` module, which holds a Wayland
/// inhibitor that dies with the bar and cannot be seen by hypridle's suspend
/// listener. This one reports the systemd lock, so what the bar shows and what
/// actually stops the machine sleeping are the same thing.
fn idle_inhibit() -> Status {
    if inhibit::is_held() {
        Status::new(
            "󰅶",
            "Screen stays on — idle, lock and suspend inhibited\nClick to release",
            "on",
        )
    } else {
        Status::hidden()
    }
}
