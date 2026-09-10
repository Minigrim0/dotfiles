//! Display geometry: resolution, refresh, scale, position, rotation.
//!
//! Deliberately separate from [`crate::monitor`], which talks to the *panel*
//! over DDC/CI to set brightness and contrast. This one talks to *Hyprland*
//! about layout. Same hardware, two unrelated channels — and only one of them
//! is worth persisting to a config file.
//!
//! Everything is applied live with `hyprctl keyword monitor`, then written to
//! `~/.config/hypr/monitors.conf` by [`save`]. That file is a symlink into
//! `configs/hyprland/.config/hypr/monitors-<machine>.conf`, so saving a layout
//! commits it to the machine it belongs to.

use crate::menu::{wofi_grid, wofi_pick};
use crate::{arrow, ok, warn};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;
use std::process::Command;

/// One output as `hyprctl monitors all -j` reports it.
#[derive(Debug, Deserialize, Clone)]
pub struct Output {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub width: u32,
    pub height: u32,
    #[serde(rename = "refreshRate")]
    pub refresh_rate: f64,
    pub x: i32,
    pub y: i32,
    pub scale: f64,
    pub transform: u8,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub focused: bool,
    #[serde(rename = "availableModes", default)]
    pub available_modes: Vec<String>,
    #[serde(rename = "mirrorOf", default)]
    pub mirror_of: String,
}

impl Output {
    /// Logical (post-scale, post-rotation) size, which is what positions are
    /// expressed in — a 4K panel at scale 2 occupies a 1920x1080 slot.
    pub fn logical_size(&self) -> (i32, i32) {
        let w = (self.width as f64 / self.scale).round() as i32;
        let h = (self.height as f64 / self.scale).round() as i32;
        // Odd transforms (90 / 270) swap the axes.
        if self.transform % 2 == 1 { (h, w) } else { (w, h) }
    }

    /// The `monitor =` rule that reproduces this output's current state.
    pub fn rule(&self) -> String {
        if self.disabled {
            return format!("{}, disable", self.name);
        }
        if !self.mirror_of.is_empty() && self.mirror_of != "none" {
            return format!(
                "{}, {}x{}@{:.3}, auto, {}, mirror, {}",
                self.name, self.width, self.height, self.refresh_rate, self.scale, self.mirror_of
            );
        }
        let mut rule = format!(
            "{}, {}x{}@{:.3}, {}x{}, {}",
            self.name, self.width, self.height, self.refresh_rate, self.x, self.y, self.scale
        );
        if self.transform != 0 {
            rule.push_str(&format!(", transform, {}", self.transform));
        }
        rule
    }

    /// Short human label used in the picker and the table.
    pub fn label(&self) -> String {
        let model = self
            .description
            .split(" (")
            .next()
            .unwrap_or(&self.description)
            .trim();
        if model.is_empty() {
            self.name.clone()
        } else {
            format!("{} — {}", self.name, model)
        }
    }
}

// ---------------------------------------------------------------------------
// hyprctl
// ---------------------------------------------------------------------------

/// Every output Hyprland knows about, disabled ones included.
pub fn outputs() -> Result<Vec<Output>> {
    let out = Command::new("hyprctl")
        .args(["monitors", "all", "-j"])
        .output()
        .context("running hyprctl — is Hyprland running?")?;
    anyhow::ensure!(out.status.success(), "hyprctl monitors failed");
    serde_json::from_slice(&out.stdout).context("parsing hyprctl monitors output")
}

/// Resolve a connector name, accepting a unique case-insensitive prefix so
/// `dots monitor scale dp-2 1.5` works without matching the exact casing.
fn find<'a>(outputs: &'a [Output], name: &str) -> Result<&'a Output> {
    let needle = name.to_lowercase();
    let exact = outputs.iter().find(|o| o.name.to_lowercase() == needle);
    if let Some(o) = exact {
        return Ok(o);
    }
    let matches: Vec<&Output> = outputs
        .iter()
        .filter(|o| o.name.to_lowercase().starts_with(&needle))
        .collect();
    match matches.len() {
        1 => Ok(matches[0]),
        0 => anyhow::bail!(
            "no output named '{}' — have: {}",
            name,
            outputs
                .iter()
                .map(|o| o.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        _ => anyhow::bail!(
            "'{}' is ambiguous — matches {}",
            name,
            matches
                .iter()
                .map(|o| o.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

/// The output to act on when none was named: the focused one.
fn target<'a>(outputs: &'a [Output], name: Option<&str>) -> Result<&'a Output> {
    match name {
        Some(n) => find(outputs, n),
        None => outputs
            .iter()
            .find(|o| o.focused)
            .or_else(|| outputs.iter().find(|o| !o.disabled))
            .ok_or_else(|| anyhow::anyhow!("no active output")),
    }
}

/// Apply a `monitor =` rule live.
fn keyword(rule: &str) -> Result<()> {
    let status = Command::new("hyprctl")
        .args(["keyword", "monitor", rule])
        .status()
        .context("running hyprctl keyword monitor")?;
    anyhow::ensure!(status.success(), "hyprctl keyword monitor '{}' failed", rule);
    Ok(())
}

fn notify(body: &str) {
    let _ = Command::new("notify-send")
        .args([
            "-a",
            "dots",
            "-h",
            "string:x-dunst-stack-tag:dots-display",
            body,
        ])
        .status();
}

// ---------------------------------------------------------------------------
// Reading
// ---------------------------------------------------------------------------

pub fn transform_label(t: u8) -> &'static str {
    match t {
        0 => "normal",
        1 => "90°",
        2 => "180°",
        3 => "270°",
        4 => "flipped",
        5 => "flipped-90°",
        6 => "flipped-180°",
        7 => "flipped-270°",
        _ => "?",
    }
}

/// Available modes for one output, deduplicated — EDIDs routinely advertise
/// the same resolution and refresh several times over.
pub fn modes(name: Option<&str>) -> Result<()> {
    let outputs = outputs()?;
    let o = target(&outputs, name)?;
    let mut seen: Vec<String> = Vec::new();
    for m in &o.available_modes {
        if !seen.contains(m) {
            seen.push(m.clone());
        }
    }
    println!("\x1b[1m{}\x1b[0m", o.label());
    println!("{}", "─".repeat(40));
    for m in &seen {
        let current = format!("{}x{}@{:.2}Hz", o.width, o.height, o.refresh_rate);
        if *m == current {
            println!("  \x1b[32m✓\x1b[0m  {}", m);
        } else {
            println!("     {}", m);
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Writing
// ---------------------------------------------------------------------------

/// Re-apply an output with one field replaced, so a scale change does not also
/// reset the mode and a rotation does not move the window.
fn reapply(o: &Output, f: impl FnOnce(&mut Output)) -> Result<()> {
    let mut next = o.clone();
    f(&mut next);
    keyword(&next.rule())
}

pub fn set_mode(name: Option<&str>, mode: &str) -> Result<()> {
    let outputs = outputs()?;
    let o = target(&outputs, name)?;
    // "preferred", "highres", "highrr" and "auto" are Hyprland's own keywords;
    // anything else has to look like a mode or the rule is silently ignored.
    let known = ["preferred", "highres", "highrr", "auto"];
    if !known.contains(&mode) && !mode.contains('x') {
        anyhow::bail!(
            "'{}' is not a mode — expected WIDTHxHEIGHT[@HZ] or one of: {}",
            mode,
            known.join(", ")
        );
    }
    let rule = format!("{}, {}, {}x{}, {}", o.name, mode, o.x, o.y, o.scale);
    keyword(&rule)?;
    ok!("{} → {}", o.name, mode);
    notify(&format!("{} → {}", o.name, mode));
    Ok(())
}

pub fn set_scale(name: Option<&str>, scale: f64) -> Result<()> {
    anyhow::ensure!(
        (0.5..=3.0).contains(&scale),
        "scale must be between 0.5 and 3.0 — got {}",
        scale
    );
    let outputs = outputs()?;
    let o = target(&outputs, name)?;
    reapply(o, |n| n.scale = scale)?;
    ok!("{} scale {:.2}", o.name, scale);
    notify(&format!("{} scale {:.2}", o.name, scale));
    Ok(())
}

/// Where to put an output. Either an absolute slot or a side of another one.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Placement {
    At(i32, i32),
    RightOf(String),
    LeftOf(String),
    Above(String),
    Below(String),
}

/// Parse "1920x0" or "1920,0" into a position.
pub fn parse_at(spec: &str) -> Result<(i32, i32)> {
    let (x, y) = spec
        .split_once(['x', ','])
        .ok_or_else(|| anyhow::anyhow!("expected XxY (e.g. 1920x0) — got '{}'", spec))?;
    Ok((
        x.trim().parse().context("invalid x")?,
        y.trim().parse().context("invalid y")?,
    ))
}

/// Resolve a placement to absolute coordinates against the current layout.
pub fn resolve(outputs: &[Output], moving: &Output, placement: &Placement) -> Result<(i32, i32)> {
    let anchor_of = |name: &str| -> Result<(i32, i32, i32, i32)> {
        let a = find(outputs, name)?;
        anyhow::ensure!(a.name != moving.name, "cannot place {} against itself", a.name);
        let (w, h) = a.logical_size();
        Ok((a.x, a.y, w, h))
    };
    let (mw, mh) = moving.logical_size();
    Ok(match placement {
        Placement::At(x, y) => (*x, *y),
        Placement::RightOf(n) => {
            let (x, y, w, _) = anchor_of(n)?;
            (x + w, y)
        }
        Placement::LeftOf(n) => {
            let (x, y, _, _) = anchor_of(n)?;
            (x - mw, y)
        }
        Placement::Above(n) => {
            let (x, y, _, _) = anchor_of(n)?;
            (x, y - mh)
        }
        Placement::Below(n) => {
            let (x, y, _, h) = anchor_of(n)?;
            (x, y + h)
        }
    })
}

pub fn set_position(name: Option<&str>, placement: &Placement) -> Result<()> {
    let outputs = outputs()?;
    let o = target(&outputs, name)?;
    let (x, y) = resolve(&outputs, o, placement)?;
    reapply(o, |n| {
        n.x = x;
        n.y = y;
    })?;
    ok!("{} at {}x{}", o.name, x, y);
    notify(&format!("{} at {}x{}", o.name, x, y));
    Ok(())
}

pub fn rotate(name: Option<&str>, degrees: u32) -> Result<()> {
    let transform = match degrees {
        0 => 0u8,
        90 => 1,
        180 => 2,
        270 => 3,
        other => anyhow::bail!("rotation must be 0, 90, 180 or 270 — got {}", other),
    };
    let outputs = outputs()?;
    let o = target(&outputs, name)?;
    reapply(o, |n| n.transform = transform)?;
    ok!("{} rotated {}°", o.name, degrees);
    notify(&format!("{} rotated {}°", o.name, degrees));
    Ok(())
}

pub fn set_enabled(name: &str, on: bool) -> Result<()> {
    let outputs = outputs()?;
    let o = find(&outputs, name)?;
    if on {
        // Re-enabling has to name a mode; `preferred` lets the EDID decide.
        keyword(&format!("{}, preferred, auto, {}", o.name, o.scale))?;
        ok!("{} enabled", o.name);
    } else {
        let active = outputs.iter().filter(|m| !m.disabled).count();
        anyhow::ensure!(
            active > 1 || o.disabled,
            "refusing to disable {} — it is the only active output",
            o.name
        );
        keyword(&format!("{}, disable", o.name))?;
        ok!("{} disabled", o.name);
    }
    Ok(())
}

pub fn mirror(name: &str, of: &str) -> Result<()> {
    let outputs = outputs()?;
    let o = find(&outputs, name)?;
    let src = find(&outputs, of)?;
    anyhow::ensure!(o.name != src.name, "cannot mirror {} onto itself", o.name);
    keyword(&format!(
        "{}, preferred, auto, {}, mirror, {}",
        o.name, o.scale, src.name
    ))?;
    ok!("{} mirrors {}", o.name, src.name);
    Ok(())
}

// ---------------------------------------------------------------------------
// Persisting
// ---------------------------------------------------------------------------

fn monitors_conf() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/root"))
        .join(".config/hypr/monitors.conf")
}

/// Render the current layout as the contents of monitors.conf.
pub fn render(outputs: &[Output]) -> String {
    let mut s = String::from(
        "# Display layout, written by `dots monitor save`.\n\
         #\n\
         # Symlinked from configs/hyprland/.config/hypr/monitors-<machine>.conf and\n\
         # sourced by hyprland.conf after its wildcard rule, so the lines below win\n\
         # for the outputs they name and every other output still gets `preferred`.\n\
         #\n\
         # monitor = NAME, WIDTHxHEIGHT@HZ, XxY, SCALE\n\n",
    );
    for o in outputs {
        s.push_str("monitor = ");
        s.push_str(&o.rule());
        s.push('\n');
    }
    s
}

pub fn save() -> Result<()> {
    let outputs = outputs()?;
    anyhow::ensure!(!outputs.is_empty(), "hyprctl reported no outputs");
    let path = monitors_conf();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // fs::write follows the symlink, which is the point: the layout lands in
    // the machine's tracked monitors-<machine>.conf, not on top of the link.
    std::fs::write(&path, render(&outputs))
        .with_context(|| format!("writing {}", path.display()))?;

    let target = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
    ok!("Saved {} output(s) to {}", outputs.len(), target.display());
    for o in &outputs {
        arrow!("{}", o.rule());
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Picker
// ---------------------------------------------------------------------------

/// `dots displays` — the settings-hub surface for everything above.
pub fn menu() -> Result<()> {
    let items = [
        ("󰍹", "Arrange"),
        ("󰊓", "Resolution"),
        ("󰩻", "Scale"),
        ("󰑦", "Rotate"),
        ("󰶐", "Mirror"),
        ("󰽉", "Turn off"),
        ("󰆓", "Save layout"),
    ];
    match wofi_grid("displays", &items).as_deref() {
        Some("Arrange") => arrange(),
        Some("Resolution") => pick_mode(),
        Some("Scale") => pick_scale(),
        Some("Rotate") => pick_rotation(),
        Some("Mirror") => pick_mirror(),
        Some("Turn off") => pick_disable(),
        Some("Save layout") => save(),
        _ => Ok(()),
    }
}

/// Drag-and-drop arrangement is the one thing a list of menu items is bad at,
/// so it is the one thing delegated to a GUI.
fn arrange() -> Result<()> {
    Command::new("nwg-displays")
        .spawn()
        .context("launching nwg-displays — is it installed?")?;
    Ok(())
}

/// Ask which output to act on, skipping the question when there is only one.
fn pick_output(prompt: &str) -> Result<Option<Output>> {
    let outputs = outputs()?;
    anyhow::ensure!(!outputs.is_empty(), "no outputs");
    if outputs.len() == 1 {
        return Ok(Some(outputs[0].clone()));
    }
    let labels: Vec<String> = outputs.iter().map(|o| o.label()).collect();
    let Some(choice) = wofi_pick(prompt, &labels) else {
        return Ok(None);
    };
    Ok(outputs.into_iter().find(|o| o.label() == choice))
}

fn pick_mode() -> Result<()> {
    let Some(o) = pick_output("display")? else {
        return Ok(());
    };
    let mut seen: Vec<String> = Vec::new();
    for m in &o.available_modes {
        if !seen.contains(m) {
            seen.push(m.clone());
        }
    }
    if seen.is_empty() {
        warn!("{} reports no modes", o.name);
        return Ok(());
    }
    let Some(choice) = wofi_pick(&o.name, &seen) else {
        return Ok(());
    };
    set_mode(Some(&o.name), choice.trim_end_matches("Hz"))
}

fn pick_scale() -> Result<()> {
    let Some(o) = pick_output("display")? else {
        return Ok(());
    };
    let items = [
        ("󰁭", "1.00"),
        ("󰁭", "1.25"),
        ("󰁭", "1.50"),
        ("󰁭", "1.75"),
        ("󰁭", "2.00"),
    ];
    let Some(choice) = wofi_grid(&o.name, &items) else {
        return Ok(());
    };
    set_scale(Some(&o.name), choice.parse()?)
}

fn pick_rotation() -> Result<()> {
    let Some(o) = pick_output("display")? else {
        return Ok(());
    };
    let items = [
        ("󰸱", "Normal"),
        ("󰑦", "90"),
        ("󰑨", "180"),
        ("󰑧", "270"),
    ];
    let Some(choice) = wofi_grid(&o.name, &items) else {
        return Ok(());
    };
    let degrees = if choice == "Normal" {
        0
    } else {
        choice.parse()?
    };
    rotate(Some(&o.name), degrees)
}

fn pick_mirror() -> Result<()> {
    let all = outputs()?;
    anyhow::ensure!(all.len() > 1, "mirroring needs at least two outputs");
    let Some(o) = pick_output("mirror which display")? else {
        return Ok(());
    };
    let others: Vec<String> = all
        .iter()
        .filter(|m| m.name != o.name)
        .map(|m| m.label())
        .collect();
    let Some(choice) = wofi_pick("onto", &others) else {
        return Ok(());
    };
    let Some(src) = all.iter().find(|m| m.label() == choice) else {
        return Ok(());
    };
    mirror(&o.name, &src.name)
}

fn pick_disable() -> Result<()> {
    let Some(o) = pick_output("turn off which display")? else {
        return Ok(());
    };
    set_enabled(&o.name, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn out(name: &str, w: u32, h: u32, x: i32, y: i32, scale: f64, transform: u8) -> Output {
        Output {
            name: name.into(),
            description: format!("Acme {} (DP-1)", name),
            width: w,
            height: h,
            refresh_rate: 60.0,
            x,
            y,
            scale,
            transform,
            disabled: false,
            focused: false,
            available_modes: vec![],
            mirror_of: String::new(),
        }
    }

    #[test]
    fn logical_size_accounts_for_scale_and_rotation() {
        assert_eq!(out("a", 3840, 2160, 0, 0, 2.0, 0).logical_size(), (1920, 1080));
        // 90° swaps the axes after scaling
        assert_eq!(out("a", 2560, 1440, 0, 0, 1.0, 1).logical_size(), (1440, 2560));
    }

    #[test]
    fn rule_round_trips_current_state() {
        let o = out("DP-2", 2560, 1440, 1920, 0, 1.0, 0);
        assert_eq!(o.rule(), "DP-2, 2560x1440@60.000, 1920x0, 1");

        let rotated = out("DP-3", 1080, 1920, 0, 0, 1.0, 1);
        assert!(rotated.rule().ends_with(", transform, 1"));

        let mut off = out("HDMI-A-1", 1920, 1080, 0, 0, 1.0, 0);
        off.disabled = true;
        assert_eq!(off.rule(), "HDMI-A-1, disable");
    }

    #[test]
    fn resolves_relative_placement() {
        let a = out("HDMI-A-1", 1920, 1080, 0, 0, 1.0, 0);
        let b = out("DP-2", 2560, 1440, 0, 0, 1.0, 0);
        let outs = vec![a.clone(), b.clone()];

        assert_eq!(
            resolve(&outs, &b, &Placement::RightOf("HDMI-A-1".into())).unwrap(),
            (1920, 0)
        );
        // left-of subtracts the *moving* display's own width
        assert_eq!(
            resolve(&outs, &b, &Placement::LeftOf("HDMI-A-1".into())).unwrap(),
            (-2560, 0)
        );
        assert_eq!(
            resolve(&outs, &b, &Placement::Below("HDMI-A-1".into())).unwrap(),
            (0, 1080)
        );
        assert_eq!(
            resolve(&outs, &b, &Placement::Above("HDMI-A-1".into())).unwrap(),
            (0, -1440)
        );
        assert_eq!(resolve(&outs, &b, &Placement::At(10, 20)).unwrap(), (10, 20));
    }

    #[test]
    fn rejects_placing_an_output_against_itself() {
        let a = out("HDMI-A-1", 1920, 1080, 0, 0, 1.0, 0);
        let outs = vec![a.clone()];
        assert!(resolve(&outs, &a, &Placement::RightOf("HDMI-A-1".into())).is_err());
    }

    #[test]
    fn parses_position_specs() {
        assert_eq!(parse_at("1920x0").unwrap(), (1920, 0));
        assert_eq!(parse_at("-2560,120").unwrap(), (-2560, 120));
        assert!(parse_at("nonsense").is_err());
    }

    #[test]
    fn finds_outputs_by_prefix() {
        let outs = vec![
            out("HDMI-A-1", 1920, 1080, 0, 0, 1.0, 0),
            out("DP-2", 2560, 1440, 0, 0, 1.0, 0),
        ];
        assert_eq!(find(&outs, "dp-2").unwrap().name, "DP-2");
        assert_eq!(find(&outs, "HDMI").unwrap().name, "HDMI-A-1");
        assert!(find(&outs, "VGA-1").is_err());
    }

    #[test]
    fn renders_a_config_file() {
        let outs = vec![out("HDMI-A-1", 1920, 1080, 0, 0, 1.0, 0)];
        let text = render(&outs);
        assert!(text.contains("monitor = HDMI-A-1, 1920x1080@60.000, 0x0, 1"));
        assert!(text.starts_with("# Display layout"));
    }
}
