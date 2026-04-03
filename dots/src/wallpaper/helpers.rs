use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

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
        let _ = Command::new(prog).args(args).status();
    }
}

/// Extract a single frame from a gif/video for matugen palette generation.
pub fn extract_frame(path: &Path) -> Result<PathBuf> {
    let out = std::env::temp_dir().join("dots-wallpaper-frame.jpg");
    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            &path.to_string_lossy(),
            "-vframes",
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
