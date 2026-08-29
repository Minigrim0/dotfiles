use crate::{arrow, ok, warn};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

const VCP_BRIGHTNESS: &str = "10";
const VCP_CONTRAST: &str = "12";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Display {
    pub display: u32,
    pub bus: u32,
    pub connector: String,
    pub model: String,
}

/// Absolute value or relative step, parsed from "60", "+5" or "-5".
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Adjust {
    Abs(u32),
    Up(u32),
    Down(u32),
}

pub fn parse_adjust(value: &str) -> Result<Adjust> {
    let value = value.trim();
    if let Some(rest) = value.strip_prefix('+') {
        Ok(Adjust::Up(rest.parse().context("invalid relative value")?))
    } else if let Some(rest) = value.strip_prefix('-') {
        Ok(Adjust::Down(
            rest.parse().context("invalid relative value")?,
        ))
    } else {
        let abs: u32 = value.parse().context("invalid absolute value")?;
        anyhow::ensure!(abs <= 100, "absolute value must be 0-100");
        Ok(Adjust::Abs(abs))
    }
}

fn cache_path() -> PathBuf {
    dirs::state_dir()
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/root"))
                .join(".local/state")
        })
        .join("dots/monitors.json")
}

/// Parse `ddcutil detect --brief` output into displays.
pub fn parse_detect(output: &str) -> Vec<Display> {
    let mut displays = Vec::new();
    let mut current: Option<Display> = None;

    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(n) = trimmed.strip_prefix("Display ") {
            if let Some(d) = current.take() {
                displays.push(d);
            }
            if let Ok(display) = n.trim().parse() {
                current = Some(Display {
                    display,
                    bus: 0,
                    connector: String::new(),
                    model: String::new(),
                });
            }
        } else if let Some(d) = current.as_mut() {
            if let Some(bus) = trimmed.strip_prefix("I2C bus:") {
                if let Some(n) = bus.trim().strip_prefix("/dev/i2c-") {
                    d.bus = n.trim().parse().unwrap_or(0);
                }
            } else if let Some(conn) = trimmed.strip_prefix("DRM connector:") {
                d.connector = conn.trim().to_string();
            } else if let Some(model) = trimmed.strip_prefix("Monitor:") {
                d.model = model.trim().to_string();
            }
        }
    }
    if let Some(d) = current.take() {
        displays.push(d);
    }
    displays
}

fn detect() -> Result<Vec<Display>> {
    arrow!("Running ddcutil detect (slow, cached afterwards)…");
    let out = Command::new("ddcutil")
        .args(["detect", "--brief"])
        .output()
        .context("running ddcutil — is it installed?")?;
    Ok(parse_detect(&String::from_utf8_lossy(&out.stdout)))
}

fn load_cache(refresh: bool) -> Result<Vec<Display>> {
    let path = cache_path();
    if !refresh
        && let Ok(content) = std::fs::read_to_string(&path)
        && let Ok(displays) = serde_json::from_str::<Vec<Display>>(&content)
    {
        return Ok(displays);
    }
    let displays = detect()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, serde_json::to_string_pretty(&displays)?)
        .with_context(|| format!("writing {}", path.display()))?;
    Ok(displays)
}

/// Name of the focused monitor's connector (e.g. "HDMI-A-1"), via hyprctl.
fn focused_connector() -> Option<String> {
    let out = Command::new("hyprctl")
        .args(["monitors", "-j"])
        .output()
        .ok()?;
    let monitors: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
    monitors.as_array()?.iter().find_map(|m| {
        if m.get("focused")?.as_bool()? {
            Some(m.get("name")?.as_str()?.to_string())
        } else {
            None
        }
    })
}

/// Pick target displays: --monitor name > --all > focused > all.
fn targets(displays: &[Display], monitor: Option<&str>, all: bool) -> Vec<Display> {
    if let Some(name) = monitor {
        return displays
            .iter()
            .filter(|d| d.connector.ends_with(name))
            .cloned()
            .collect();
    }
    if !all
        && let Some(focused) = focused_connector()
        && let Some(d) = displays.iter().find(|d| d.connector.ends_with(&focused))
    {
        return vec![d.clone()];
    }
    displays.to_vec()
}

/// Run ddcutil serialized behind a file lock — concurrent i2c transactions
/// (waybar polling while a brightness key fires) make DDC replies fail.
fn ddcutil(args: &[&str]) -> Result<std::process::Output> {
    let lock = dirs::runtime_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("dots-ddc.lock");
    Command::new("flock")
        .args(["-w", "3", &lock.to_string_lossy(), "ddcutil"])
        .args(args)
        .output()
        .context("running ddcutil (via flock)")
}

/// Read a VCP value on a bus. Returns (current, max).
/// DDC replies are flaky on some monitors — retry a few times.
fn get_vcp(bus: u32, code: &str) -> Result<(u32, u32)> {
    let bus_s = bus.to_string();
    let mut last_err = String::new();
    for attempt in 0..3 {
        if attempt > 0 {
            std::thread::sleep(std::time::Duration::from_millis(150));
        }
        let out = ddcutil(&["--bus", &bus_s, "getvcp", code, "--brief"])?;
        // Format: "VCP 10 C 50 100"
        let text = String::from_utf8_lossy(&out.stdout);
        let fields: Vec<&str> = text.split_whitespace().collect();
        if let (Some(cur), Some(max)) = (fields.get(3), fields.get(4))
            && let (Ok(cur), Ok(max)) = (cur.parse(), max.parse())
        {
            return Ok((cur, max));
        }
        last_err = format!(
            "{} {}",
            text.trim(),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    anyhow::bail!("getvcp on bus {} kept failing: {}", bus, last_err.trim())
}

fn set_vcp(bus: u32, code: &str, adjust: Adjust) -> Result<()> {
    let bus_s = bus.to_string();
    let mut args: Vec<&str> = vec!["--bus", &bus_s, "setvcp", code];
    let val;
    match adjust {
        Adjust::Abs(v) => {
            val = v.to_string();
            args.push(&val);
        }
        Adjust::Up(v) => {
            val = v.to_string();
            args.push("+");
            args.push(&val);
        }
        Adjust::Down(v) => {
            val = v.to_string();
            args.push("-");
            args.push(&val);
        }
    }
    for attempt in 0..2 {
        if attempt > 0 {
            std::thread::sleep(std::time::Duration::from_millis(150));
        }
        if ddcutil(&args)?.status.success() {
            return Ok(());
        }
    }
    anyhow::bail!("ddcutil setvcp failed on bus {}", bus)
}

fn notify(label: &str, pct: u32) {
    let _ = Command::new("notify-send")
        .args([
            "-a",
            "dots",
            "-h",
            "string:x-dunst-stack-tag:dots-brightness",
            "-h",
            &format!("int:value:{}", pct),
            &format!("{} {}%", label, pct),
        ])
        .status();
}

/// brightnessctl fallback for machines without DDC displays (laptops).
fn brightnessctl(adjust: Adjust) -> Result<()> {
    let arg = match adjust {
        Adjust::Abs(v) => format!("{}%", v),
        Adjust::Up(v) => format!("{}%+", v),
        Adjust::Down(v) => format!("{}%-", v),
    };
    let status = Command::new("brightnessctl")
        .args(["-e4", "-n2", "set", &arg])
        .status()
        .context("running brightnessctl — no DDC display and no backlight?")?;
    anyhow::ensure!(status.success(), "brightnessctl failed");
    if let Some(pct) = brightnessctl_get() {
        notify("Brightness", pct);
        ok!("Brightness {}% (backlight)", pct);
    }
    Ok(())
}

fn brightnessctl_get() -> Option<u32> {
    let out = Command::new("brightnessctl").arg("-m").output().ok()?;
    // machine-readable CSV: device,class,current,percent%,max
    String::from_utf8_lossy(&out.stdout)
        .split(',')
        .nth(3)?
        .trim_end_matches('%')
        .parse()
        .ok()
}

pub fn list(refresh: bool) -> Result<()> {
    let displays = load_cache(refresh)?;
    if displays.is_empty() {
        warn!("No DDC displays found (try --refresh, or check i2c-dev / permissions)");
        return Ok(());
    }
    println!(
        "\x1b[1m{:<4} {:<6} {:<18} {:<28} BRIGHTNESS\x1b[0m",
        "N", "BUS", "CONNECTOR", "MODEL"
    );
    println!("{}", "─".repeat(72));
    for d in &displays {
        let brightness = get_vcp(d.bus, VCP_BRIGHTNESS)
            .map(|(cur, max)| format!("{}/{}", cur, max))
            .unwrap_or_else(|_| "?".into());
        println!(
            "{:<4} {:<6} {:<18} {:<28} {}",
            d.display, d.bus, d.connector, d.model, brightness
        );
    }
    Ok(())
}

fn apply(code: &str, label: &str, value: &str, monitor: Option<&str>, all: bool) -> Result<()> {
    let adjust = parse_adjust(value)?;
    let displays = load_cache(false)?;

    if displays.is_empty() {
        if code == VCP_BRIGHTNESS {
            return brightnessctl(adjust);
        }
        anyhow::bail!("no DDC displays found");
    }

    let targets = targets(&displays, monitor, all);
    anyhow::ensure!(!targets.is_empty(), "no monitor matched");

    for d in &targets {
        set_vcp(d.bus, code, adjust)?;
        // Read-back is best-effort: the set already succeeded.
        match get_vcp(d.bus, code) {
            Ok((cur, _)) => {
                notify(label, cur);
                ok!("{} {}% on {}", label, cur, d.connector);
            }
            Err(_) => ok!("{} adjusted on {}", label, d.connector),
        }
    }
    Ok(())
}

pub fn brightness(value: &str, monitor: Option<&str>, all: bool) -> Result<()> {
    apply(VCP_BRIGHTNESS, "Brightness", value, monitor, all)
}

pub fn contrast(value: &str, monitor: Option<&str>, all: bool) -> Result<()> {
    apply(VCP_CONTRAST, "Contrast", value, monitor, all)
}

/// Print focused monitor's brightness as a bare number (waybar-friendly).
/// Polled on an interval — never fails hard, prints nothing when DDC is
/// momentarily unreadable so the bar shows a blank instead of an error toast.
pub fn get() -> Result<()> {
    let Ok(displays) = load_cache(false) else {
        return Ok(());
    };
    if displays.is_empty() {
        if let Some(pct) = brightnessctl_get() {
            println!("{}", pct);
        }
        return Ok(());
    }
    if let Some(d) = targets(&displays, None, false).first()
        && let Ok((cur, _)) = get_vcp(d.bus, VCP_BRIGHTNESS)
    {
        println!("{}", cur);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_adjust_values() {
        assert_eq!(parse_adjust("60").unwrap(), Adjust::Abs(60));
        assert_eq!(parse_adjust("+5").unwrap(), Adjust::Up(5));
        assert_eq!(parse_adjust("-5").unwrap(), Adjust::Down(5));
        assert!(parse_adjust("101").is_err());
        assert!(parse_adjust("abc").is_err());
    }

    #[test]
    fn parses_ddcutil_detect_brief() {
        let out = "\
Display 1
   I2C bus:          /dev/i2c-2
   DRM connector:    card1-HDMI-A-1
   drm_connector_id: 399
   Monitor:          SAM:S24R65x:H4TRA03746

Display 2
   I2C bus:          /dev/i2c-6
   DRM connector:    card1-DP-2
   drm_connector_id: 392
   Monitor:          SAM:C27JG5x:H4ZN700459
";
        let displays = parse_detect(out);
        assert_eq!(displays.len(), 2);
        assert_eq!(displays[0].bus, 2);
        assert_eq!(displays[0].connector, "card1-HDMI-A-1");
        assert_eq!(displays[1].display, 2);
        assert_eq!(displays[1].bus, 6);
        assert_eq!(displays[1].model, "SAM:C27JG5x:H4ZN700459");
    }
}
