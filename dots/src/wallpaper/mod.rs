use crate::{arrow, ok, warn};
use anyhow::{Context, Result};
use std::fs::copy;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::info;

mod dir;
mod helpers;
mod state;

use dir::wallpaper_dir;
use helpers::{extract_frame, reload_apps};
pub use state::{load_state, save_state};

use crate::wallpaper::helpers::to_still_path;

const ANIMATED_FPS: u32 = 10;

/// Convert/copy a file into ~/.config/wallpaper/<name>.[gif|ext].
/// For GIFs/Videos, a single frame is extracted and registered as <name>.still.jpg for static
/// wallpapers
pub fn register(path: &Path, name: Option<&str>) -> Result<()> {
    let wdir = wallpaper_dir();
    std::fs::create_dir_all(&wdir)?;

    let stem = name.map(|s| s.to_string()).unwrap_or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("wallpaper")
            .to_string()
    });

    let dest = if helpers::is_video(path) {
        let dest = wdir.join(format!("{}.gif", stem));
        arrow!("Converting video to gif at {}fps…", ANIMATED_FPS);
        let status = Command::new("ffmpeg")
            .args([
                "-y",
                "-i",
                &path.to_string_lossy(),
                "-vf",
                &format!("fps={},scale=trunc(iw/2)*2:trunc(ih/2)*2", ANIMATED_FPS),
                "-loop",
                "0",
                &dest.to_string_lossy(),
            ])
            .status()
            .context("running ffmpeg — is it installed?")?;
        if !status.success() {
            anyhow::bail!("ffmpeg conversion failed");
        }
        let still_path = extract_frame(path)?;
        let still_dst_path = helpers::to_still_path(&dest);
        copy(still_path, still_dst_path)?;
        dest
    } else {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("jpg");
        let dest = wdir.join(format!("{}.{}", stem, ext));
        std::fs::copy(path, &dest)
            .with_context(|| format!("copying {} to {}", path.display(), dest.display()))?;
        if helpers::is_gif(&dest) {
            arrow!("Extracting still frame from gif…");
            let still_path = extract_frame(&dest)?;
            copy(still_path, helpers::to_still_path(&dest))?;
        }
        dest
    };

    ok!("Registered as '{stem}' → {}", dest.display());
    Ok(())
}

/// Apply a registered wallpaper by name. Resolves to any file with that stem.
pub fn set(name: &str) -> Result<()> {
    let wdir = wallpaper_dir();
    let mut state = load_state();

    // Find any file whose stem matches
    let entry = std::fs::read_dir(&wdir)
        .with_context(|| format!("reading {}", wdir.display()))?
        .filter_map(|e| e.ok())
        .find(|e| e.path().file_stem().and_then(|s| s.to_str()) == Some(name))
        .ok_or_else(|| anyhow::anyhow!("No wallpaper named '{}' in {}", name, wdir.display()))?;

    let path = if helpers::is_animated(&entry.path()) && state.mode == "static" {
        let still = to_still_path(&entry.path());
        if still.exists() { still } else { entry.path() }
    } else {
        entry.path()
    };

    // awww handles both static images and animated gifs
    let status = Command::new("awww")
        .arg("img")
        .arg(&path)
        .status()
        .context("running awww")?;
    if !status.success() {
        anyhow::bail!("awww failed");
    }

    // matugen: extract a frame if gif, use file directly if static image
    let palette_path;
    let matugen_input: &Path = if helpers::is_gif(&path) {
        palette_path = extract_frame(&path)?;
        &palette_path
    } else {
        &path
    };

    let mode_flag = if state.dark_mode { "dark" } else { "light" };
    info!("Extracting palette from still");
    let matugen_status = Command::new("matugen")
        .args([
            "image",
            &matugen_input.to_string_lossy(),
            "-m",
            mode_flag,
            "--source-color-index",
            "0",
        ])
        .status()
        .context("running matugen")?;
    if !matugen_status.success() {
        warn!("matugen exited with error");
    }

    reload_apps();

    state.current = name.to_string();
    save_state(&state)?;

    ok!("Wallpaper set to '{}'", name);
    Ok(())
}

pub fn list() -> Result<()> {
    let wdir = wallpaper_dir();
    if !wdir.exists() {
        warn!("No wallpapers registered yet ({})", wdir.display());
        return Ok(());
    }
    let mut entries: Vec<_> = std::fs::read_dir(&wdir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter(|e| !e.file_name().to_string_lossy().ends_with(".still.jpg"))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    let state = load_state();
    for entry in entries {
        let p = entry.path();
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("");
        let active = if stem == state.current {
            " \x1b[32m(active)\x1b[0m"
        } else {
            ""
        };
        println!("  {}.{}{}", stem, ext, active);
    }
    Ok(())
}

pub fn set_mode(mode: &str) -> Result<()> {
    let socket_path = dirs::runtime_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("dots.sock");

    if socket_path.exists() {
        use std::os::unix::net::UnixStream;
        let mut stream =
            UnixStream::connect(&socket_path).context("connecting to daemon socket")?;
        let msg = format!("{{\"cmd\":\"set_mode\",\"mode\":\"{}\"}}\n", mode);
        stream
            .write_all(msg.as_bytes())
            .context("sending to daemon")?;
        arrow!("Mode sent to daemon: {}", mode);
    } else {
        let mut state = load_state();
        state.mode = mode.to_string();
        save_state(&state)?;
        arrow!("Mode saved: {} (daemon not running)", mode);
    }
    Ok(())
}

/// Called by the daemon when power state changes.
pub fn apply_for_power(_is_ac: bool) -> Result<()> {
    let state = load_state();
    if !state.current.is_empty() {
        info!("Changing state to: {}", state.current);
        set(&state.current)?;
    }
    Ok(())
}
