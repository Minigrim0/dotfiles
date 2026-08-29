use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

use crate::ok;

use super::dir::wallpaper_dir;

pub fn is_video(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("mp4" | "mkv" | "webm" | "avi" | "mov")
    )
}

pub fn is_gif(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("gif")
}

pub fn is_animated(path: &Path) -> bool {
    is_video(path) || is_gif(path)
}

pub fn reload_apps() {
    for (prog, args) in &[
        ("pkill", vec!["-SIGUSR2", "waybar"]),
        ("hyprctl", vec!["reload"]),
        ("pkill", vec!["-SIGUSR1", "kitty"]),
        ("dunstctl", vec!["reload"]),
    ] {
        // .output() so hyprctl's "ok" doesn't leak into our own output
        let _ = Command::new(prog).args(args).output();
    }
    // swayosd only reads its stylesheet at startup — restart it
    let _ = Command::new("pkill")
        .args(["-x", "swayosd-server"])
        .status();
    let _ = Command::new("swayosd-server")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
    ok!("reloaded — waybar, hyprland, kitty, dunst · swayosd restarted");
}

/// Print the freshly generated palette as truecolor chips plus a summary of
/// the matugen templates that were rendered.
pub fn print_apply_summary() {
    let Some(home) = dirs::home_dir() else { return };

    if let Ok(content) = std::fs::read_to_string(home.join(".config/hypr/colors.conf")) {
        let chips: Vec<String> = parse_palette(&content)
            .iter()
            .filter_map(|(_, hex)| {
                let (r, g, b) = parse_hex(hex)?;
                Some(format!("\x1b[38;2;{r};{g};{b}m██\x1b[0m #{hex}"))
            })
            .collect();
        if !chips.is_empty() {
            println!("  {}", chips.join("  "));
        }
    }

    if let Ok(content) = std::fs::read_to_string(home.join(".config/matugen/config.toml")) {
        let names = parse_template_names(&content);
        if !names.is_empty() {
            ok!("{} templates rendered — {}", names.len(), names.join(", "));
        }
    }
}

/// Parse `$name = rgb(hex)` lines from the generated hyprland colors file.
fn parse_palette(content: &str) -> Vec<(String, String)> {
    content
        .lines()
        .filter_map(|l| {
            let (name, rest) = l.trim().strip_prefix('$')?.split_once('=')?;
            let hex = rest.trim().strip_prefix("rgb(")?.strip_suffix(')')?;
            Some((name.trim().to_string(), hex.to_string()))
        })
        .collect()
}

fn parse_hex(hex: &str) -> Option<(u8, u8, u8)> {
    if hex.len() != 6 {
        return None;
    }
    Some((
        u8::from_str_radix(&hex[0..2], 16).ok()?,
        u8::from_str_radix(&hex[2..4], 16).ok()?,
        u8::from_str_radix(&hex[4..6], 16).ok()?,
    ))
}

/// Names of the [templates.*] sections in the matugen config.
fn parse_template_names(content: &str) -> Vec<String> {
    let mut names: Vec<String> = content
        .lines()
        .filter_map(|l| {
            l.trim()
                .strip_prefix("[templates.")?
                .strip_suffix(']')
                .map(str::to_string)
        })
        .collect();
    names.sort();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_palette_and_templates() {
        let colors = "# comment\n$background = rgb(121318)\n$accent = rgb(afc6ff)\n";
        let palette = parse_palette(colors);
        assert_eq!(palette.len(), 2);
        assert_eq!(palette[1], ("accent".into(), "afc6ff".into()));
        assert_eq!(parse_hex("afc6ff"), Some((0xaf, 0xc6, 0xff)));
        assert_eq!(parse_hex("nope"), None);

        let cfg = "[config]\n[templates.waybar]\nx = 1\n[templates.kitty]\n";
        assert_eq!(parse_template_names(cfg), vec!["kitty", "waybar"]);
    }
}

/// Extract a single frame from a gif/video for matugen palette generation.
pub fn extract_frame(path: &Path) -> Result<PathBuf> {
    let out = std::env::temp_dir().join("dots-wallpaper-frame.jpg");
    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-loglevel",
            "error",
            "-i",
            &path.to_string_lossy(),
            "-vframes",
            "1",
            "-update",
            "1",
            "-q:v",
            "2",
            &out.to_string_lossy(),
        ])
        .status()
        .context("running ffmpeg — is it installed?")?;
    if !status.success() {
        anyhow::bail!("ffmpeg frame extraction failed");
    }
    Ok(out)
}

/// Converts a wallpaper name to the name of its registered still
pub fn to_still_path(path: &Path) -> PathBuf {
    let wdir = wallpaper_dir();

    let still_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("wallpaper")
        .to_string();
    wdir.join(format!("{}.still.jpg", still_stem))
}
