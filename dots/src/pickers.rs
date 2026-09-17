//! Network, audio and Bluetooth pickers, drawn with wofi.
//!
//! These replace nm-connection-editor, pavucontrol and blueman-manager as the
//! *everyday* path — they are wofi surfaces, so matugen themes them along with
//! everything else and they match the launcher. The heavyweight GUIs stay
//! installed for the rare deep configuration.
//!
//! Bound to the bar: clicking the network / audio / bluetooth icon opens the
//! matching picker.

use crate::menu::{wofi_input, wofi_pick};
use crate::{ok, warn};
use anyhow::{Context, Result};
use std::process::Command;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Split one `nmcli -t` record, honouring its `\:` escaping.
fn split_escaped(line: &str) -> Vec<String> {
    let mut fields = vec![String::new()];
    let mut escaped = false;
    for c in line.chars() {
        match c {
            '\\' if !escaped => escaped = true,
            ':' if !escaped => fields.push(String::new()),
            _ => {
                escaped = false;
                fields.last_mut().expect("always non-empty").push(c);
            }
        }
    }
    fields
}

fn stdout(cmd: &str, args: &[&str]) -> Result<String> {
    let out = Command::new(cmd)
        .args(args)
        .output()
        .with_context(|| format!("running {}", cmd))?;
    anyhow::ensure!(
        out.status.success(),
        "{} failed: {}",
        cmd,
        String::from_utf8_lossy(&out.stderr).trim()
    );
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn notify(body: &str) {
    let _ = Command::new("notify-send")
        .args(["-a", "dots", body])
        .status();
}

// ---------------------------------------------------------------------------
// Wi-Fi
// ---------------------------------------------------------------------------

fn signal_icon(strength: u8) -> &'static str {
    match strength {
        75..=100 => "󰤨",
        50..=74 => "󰤥",
        25..=49 => "󰤢",
        _ => "󰤟",
    }
}

/// The first wifi device NetworkManager knows about.
fn wifi_device() -> Option<String> {
    let out = stdout("nmcli", &["-t", "-f", "DEVICE,TYPE", "device"]).ok()?;
    out.lines().find_map(|l| {
        let f = split_escaped(l);
        (f.len() >= 2 && f[1] == "wifi").then(|| f[0].clone())
    })
}

fn wifi_enabled() -> bool {
    stdout("nmcli", &["-t", "radio", "wifi"])
        .map(|s| s.trim() == "enabled")
        .unwrap_or(false)
}

/// Connect, prompting for a passphrase only if NetworkManager asks for one.
fn wifi_connect(ssid: &str) -> Result<()> {
    let attempt = Command::new("nmcli")
        .args(["device", "wifi", "connect", ssid])
        .output()
        .context("running nmcli")?;

    if attempt.status.success() {
        ok!("Connected to {}", ssid);
        notify(&format!("Connected to {}", ssid));
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&attempt.stderr).to_lowercase();
    let needs_secret = stderr.contains("secrets were required")
        || stderr.contains("no secrets")
        || stderr.contains("802-11-wireless-security");

    if !needs_secret {
        anyhow::bail!("{}", String::from_utf8_lossy(&attempt.stderr).trim());
    }

    let Some(password) = wofi_input(&format!("password for {}", ssid), true) else {
        return Ok(()); // dismissed
    };

    let retry = Command::new("nmcli")
        .args(["device", "wifi", "connect", ssid, "password", &password])
        .output()
        .context("running nmcli")?;

    anyhow::ensure!(
        retry.status.success(),
        "{}",
        String::from_utf8_lossy(&retry.stderr).trim()
    );
    ok!("Connected to {}", ssid);
    notify(&format!("Connected to {}", ssid));
    Ok(())
}

/// One access point as `nmcli -t` reports it.
#[derive(Debug, PartialEq, Eq)]
pub struct Network {
    pub active: bool,
    pub strength: u8,
    pub secured: bool,
    pub ssid: String,
}

/// Parse `nmcli -t -f IN-USE,SIGNAL,SECURITY,SSID device wifi list`.
///
/// Strongest first, one entry per SSID — a mesh reports the same network once
/// per radio, and picking any of them connects to the same thing.
pub fn parse_wifi_list(listing: &str) -> Vec<Network> {
    let mut networks: Vec<Network> = Vec::new();
    for line in listing.lines() {
        let f = split_escaped(line);
        if f.len() < 4 {
            continue;
        }
        // An SSID may itself contain a colon, so everything past the third
        // separator belongs to it.
        let ssid = f[3..].join(":");
        if ssid.is_empty() || networks.iter().any(|n| n.ssid == ssid) {
            continue;
        }
        let security = f[2].trim();
        networks.push(Network {
            active: f[0].trim() == "*",
            strength: f[1].trim().parse().unwrap_or(0),
            secured: !security.is_empty() && security != "--",
            ssid,
        });
    }
    networks.sort_by_key(|n| std::cmp::Reverse(n.strength));
    networks
}

pub fn wifi() -> Result<()> {
    let Some(device) = wifi_device() else {
        anyhow::bail!("no Wi-Fi device found by NetworkManager");
    };

    if !wifi_enabled() {
        let items = vec!["󰖩  Turn Wi-Fi on".to_string(), "󰅖  Cancel".to_string()];
        if wofi_pick("wifi", &items).as_deref() == Some("󰖩  Turn Wi-Fi on") {
            stdout("nmcli", &["radio", "wifi", "on"])?;
            ok!("Wi-Fi on");
        }
        return Ok(());
    }

    let listing = stdout(
        "nmcli",
        &[
            "-t",
            "-f",
            "IN-USE,SIGNAL,SECURITY,SSID",
            "device",
            "wifi",
            "list",
        ],
    )?;

    let networks = parse_wifi_list(&listing);

    let mut labels: Vec<String> = networks
        .iter()
        .map(|n| {
            format!(
                "{} {}  {}  {}  {}%",
                if n.active { "●" } else { " " },
                signal_icon(n.strength),
                if n.secured { "󰌾" } else { " " },
                n.ssid,
                n.strength
            )
        })
        .collect();

    const RESCAN: &str = "  󰑓  Rescan";
    const DISCONNECT: &str = "  󰖪  Disconnect";
    const RADIO_OFF: &str = "  󰖪  Turn Wi-Fi off";
    labels.push(RESCAN.to_string());
    if networks.iter().any(|n| n.active) {
        labels.push(DISCONNECT.to_string());
    }
    labels.push(RADIO_OFF.to_string());

    let Some(choice) = wofi_pick("wifi", &labels) else {
        return Ok(());
    };

    match choice.as_str() {
        RESCAN => {
            stdout("nmcli", &["device", "wifi", "rescan"])?;
            return wifi();
        }
        DISCONNECT => {
            stdout("nmcli", &["device", "disconnect", &device])?;
            ok!("Disconnected {}", device);
            notify("Wi-Fi disconnected");
        }
        RADIO_OFF => {
            stdout("nmcli", &["radio", "wifi", "off"])?;
            ok!("Wi-Fi off");
            notify("Wi-Fi off");
        }
        _ => {
            let idx = labels.iter().position(|l| *l == choice);
            if let Some(i) = idx
                && let Some(net) = networks.get(i)
            {
                if net.active {
                    ok!("Already connected to {}", net.ssid);
                } else {
                    wifi_connect(&net.ssid)?;
                }
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Audio
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
struct PaDevice {
    name: String,
    description: String,
}

fn pa_devices(kind: &str) -> Result<Vec<PaDevice>> {
    let raw = stdout("pactl", &["-f", "json", "list", kind])?;
    serde_json::from_str(&raw).with_context(|| format!("parsing pactl {} json", kind))
}

/// Move every currently playing stream too — switching the default without
/// moving the streams is the thing that makes people reopen pavucontrol.
fn move_streams(kind_short: &str, move_cmd: &str, target: &str) {
    let Ok(list) = stdout("pactl", &["list", "short", kind_short]) else {
        return;
    };
    for line in list.lines() {
        if let Some(id) = line.split_whitespace().next() {
            let _ = Command::new("pactl").args([move_cmd, id, target]).status();
        }
    }
}

pub fn audio() -> Result<()> {
    let sinks = pa_devices("sinks")?;
    let sources: Vec<PaDevice> = pa_devices("sources")?
        .into_iter()
        // Monitor sources are loopbacks of outputs, never something you pick.
        .filter(|d| !d.name.ends_with(".monitor"))
        .collect();

    let default_sink = stdout("pactl", &["get-default-sink"])
        .unwrap_or_default()
        .trim()
        .to_string();
    let default_source = stdout("pactl", &["get-default-source"])
        .unwrap_or_default()
        .trim()
        .to_string();

    let mut labels: Vec<String> = Vec::new();
    let mut actions: Vec<(bool, String)> = Vec::new(); // (is_sink, name)

    for d in &sinks {
        let mark = if d.name == default_sink { "●" } else { " " };
        labels.push(format!("{} 󰓃  {}", mark, d.description));
        actions.push((true, d.name.clone()));
    }
    for d in &sources {
        let mark = if d.name == default_source { "●" } else { " " };
        labels.push(format!("{} 󰍬  {}", mark, d.description));
        actions.push((false, d.name.clone()));
    }

    anyhow::ensure!(!labels.is_empty(), "no audio devices found");

    let Some(choice) = wofi_pick("audio", &labels) else {
        return Ok(());
    };
    let Some(i) = labels.iter().position(|l| *l == choice) else {
        return Ok(());
    };
    let (is_sink, name) = &actions[i];

    if *is_sink {
        stdout("pactl", &["set-default-sink", name])?;
        move_streams("sink-inputs", "move-sink-input", name);
        ok!("Output → {}", sinks[i].description);
        notify(&format!("Output: {}", sinks[i].description));
    } else {
        stdout("pactl", &["set-default-source", name])?;
        move_streams("source-outputs", "move-source-output", name);
        let desc = &sources[i - sinks.len()].description;
        ok!("Input → {}", desc);
        notify(&format!("Input: {}", desc));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Bluetooth
// ---------------------------------------------------------------------------

fn bt_powered() -> bool {
    stdout("bluetoothctl", &["show"])
        .map(|s| s.contains("Powered: yes"))
        .unwrap_or(false)
}

fn bt_connected(mac: &str) -> bool {
    stdout("bluetoothctl", &["info", mac])
        .map(|s| s.contains("Connected: yes"))
        .unwrap_or(false)
}

pub fn bluetooth() -> Result<()> {
    if !bt_powered() {
        let items = vec!["󰂯  Turn Bluetooth on".to_string(), "󰅖  Cancel".to_string()];
        if wofi_pick("bluetooth", &items).as_deref() == Some("󰂯  Turn Bluetooth on") {
            stdout("bluetoothctl", &["power", "on"])?;
            ok!("Bluetooth on");
        }
        return Ok(());
    }

    let listing = stdout("bluetoothctl", &["devices"])?;
    let mut devices: Vec<(String, String, bool)> = Vec::new(); // (mac, name, connected)
    for line in listing.lines() {
        let mut parts = line.splitn(3, ' ');
        if parts.next() != Some("Device") {
            continue;
        }
        let Some(mac) = parts.next() else { continue };
        let name = parts.next().unwrap_or(mac).to_string();
        let connected = bt_connected(mac);
        devices.push((mac.to_string(), name, connected));
    }
    // Connected devices first — they are the ones you act on.
    devices.sort_by(|a, b| b.2.cmp(&a.2).then(a.1.cmp(&b.1)));

    let mut labels: Vec<String> = devices
        .iter()
        .map(|(_, name, connected)| {
            let mark = if *connected { "●" } else { " " };
            let icon = if *connected { "󰂱" } else { "󰂯" };
            format!("{} {}  {}", mark, icon, name)
        })
        .collect();

    const SCAN: &str = "  󰑓  Scan for devices";
    const POWER_OFF: &str = "  󰂲  Turn Bluetooth off";
    labels.push(SCAN.to_string());
    labels.push(POWER_OFF.to_string());

    let Some(choice) = wofi_pick("bluetooth", &labels) else {
        return Ok(());
    };

    match choice.as_str() {
        SCAN => {
            notify("Scanning for 10s…");
            let _ = Command::new("bluetoothctl")
                .args(["--timeout", "10", "scan", "on"])
                .status();
            return bluetooth();
        }
        POWER_OFF => {
            stdout("bluetoothctl", &["power", "off"])?;
            ok!("Bluetooth off");
            notify("Bluetooth off");
        }
        _ => {
            let Some(i) = labels.iter().position(|l| *l == choice) else {
                return Ok(());
            };
            let (mac, name, connected) = &devices[i];
            let verb = if *connected { "disconnect" } else { "connect" };
            match stdout("bluetoothctl", &[verb, mac]) {
                Ok(_) => {
                    ok!(
                        "{} {}",
                        if *connected {
                            "Disconnected"
                        } else {
                            "Connected"
                        },
                        name
                    );
                    notify(&format!(
                        "{} {}",
                        if *connected {
                            "Disconnected"
                        } else {
                            "Connected"
                        },
                        name
                    ));
                }
                Err(e) => {
                    warn!("{}: {:#}", name, e);
                    notify(&format!("Could not {} {}", verb, name));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_nmcli_records_with_escaped_colons() {
        assert_eq!(split_escaped("a:b:c"), vec!["a", "b", "c"]);
        // nmcli escapes a literal colon inside a field.
        assert_eq!(
            split_escaped(r"*:80:WPA2:cafe\:wifi"),
            vec!["*", "80", "WPA2", "cafe:wifi"]
        );
        assert_eq!(split_escaped(""), vec![""]);
    }

    #[test]
    fn parses_and_ranks_wifi_list() {
        let listing = "\
*:100:WPA2:Fastoche
 :84:WPA2:Bluu-Eyes
 :70:WPA2 WPA3:Hyperoptic Fibre 73D3
 :60:WPA2 WPA3:Hyperoptic Fibre 73D3
 :45::OpenGuest
 :30:WPA2:";

        let nets = parse_wifi_list(listing);

        // Empty SSID dropped, duplicate mesh radio collapsed to its strongest.
        assert_eq!(nets.len(), 4);
        // Sorted by signal, strongest first.
        assert_eq!(nets[0].ssid, "Fastoche");
        assert!(nets[0].active);
        assert_eq!(nets[1].strength, 84);
        assert_eq!(nets[2].ssid, "Hyperoptic Fibre 73D3");
        assert_eq!(nets[2].strength, 70);
        // Open network reports as unsecured.
        assert_eq!(nets[3].ssid, "OpenGuest");
        assert!(!nets[3].secured);
        assert!(!nets[3].active);
    }

    #[test]
    fn signal_icons_step_with_strength() {
        assert_eq!(signal_icon(100), signal_icon(80));
        assert_ne!(signal_icon(80), signal_icon(60));
        assert_ne!(signal_icon(60), signal_icon(30));
        assert_ne!(signal_icon(30), signal_icon(5));
    }
}
