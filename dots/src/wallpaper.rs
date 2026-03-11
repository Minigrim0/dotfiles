use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

const ANIMATED_FPS: u32 = 10;

// ---------------------------------------------------------------------------
// Wallpaper directory
// ---------------------------------------------------------------------------

fn wallpaper_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/root"))
        .join(".config/wallpaper")
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct WallpaperState {
    pub current: String,
    pub mode: String,
}

fn state_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/root"))
        .join(".local/share/dots/wallpaper.toml")
}

pub fn load_state() -> WallpaperState {
    let path = state_path();
    if let Ok(content) = std::fs::read_to_string(&path)
        && let Ok(s) = toml::from_str::<WallpaperState>(&content)
    {
        return s;
    }
    WallpaperState {
        current: String::new(),
        mode: "auto".to_string(),
    }
}

pub fn save_state(state: &WallpaperState) -> Result<()> {
    let path = state_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = format!(
        "current = {:?}\nmode    = {:?}\n",
        state.current, state.mode
    );
    std::fs::write(&path, content).with_context(|| format!("writing {}", path.display()))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn is_video(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("mp4" | "mkv" | "webm" | "avi" | "mov")
    )
}

fn reload_apps() {
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
fn extract_frame(path: &Path) -> Result<PathBuf> {
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

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Convert/copy a file into ~/.config/wallpaper/<name>.[gif|ext].
pub fn register(path: &Path, name: Option<&str>) -> Result<()> {
    let wdir = wallpaper_dir();
    std::fs::create_dir_all(&wdir)?;

    let stem = name.map(|s| s.to_string()).unwrap_or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("wallpaper")
            .to_string()
    });

    let dest = if is_video(path) {
        let dest = wdir.join(format!("{}.gif", stem));
        println!("  → Converting video to gif at {}fps…", ANIMATED_FPS);
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
        dest
    } else {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("jpg");
        let dest = wdir.join(format!("{}.{}", stem, ext));
        std::fs::copy(path, &dest)
            .with_context(|| format!("copying {} to {}", path.display(), dest.display()))?;
        dest
    };

    println!("  ✓ Registered as '{stem}' → {}", dest.display());
    Ok(())
}

/// Apply a registered wallpaper by name. Resolves to any file with that stem.
pub fn set(name: &str) -> Result<()> {
    let wdir = wallpaper_dir();

    // Find any file whose stem matches
    let entry = std::fs::read_dir(&wdir)
        .with_context(|| format!("reading {}", wdir.display()))?
        .filter_map(|e| e.ok())
        .find(|e| e.path().file_stem().and_then(|s| s.to_str()) == Some(name))
        .ok_or_else(|| anyhow::anyhow!("No wallpaper named '{}' in {}", name, wdir.display()))?;

    let path = entry.path();

    // swww handles both static images and animated gifs
    let status = Command::new("swww")
        .arg("img")
        .arg(&path)
        .status()
        .context("running swww")?;
    if !status.success() {
        anyhow::bail!("swww failed");
    }

    // matugen: extract a frame if gif, use file directly if static image
    let is_gif = path.extension().and_then(|e| e.to_str()) == Some("gif");
    let palette_path;
    let matugen_input: &Path = if is_gif {
        palette_path = extract_frame(&path)?;
        &palette_path
    } else {
        &path
    };

    let matugen_status = Command::new("matugen")
        .args(["image", &matugen_input.to_string_lossy()])
        .status()
        .context("running matugen")?;
    if !matugen_status.success() {
        eprintln!("  ~ matugen exited with error");
    }

    reload_apps();

    let mut state = load_state();
    state.current = name.to_string();
    save_state(&state)?;

    println!("  ✓ Wallpaper set to '{}'", name);
    Ok(())
}

pub fn list() -> Result<()> {
    let wdir = wallpaper_dir();
    if !wdir.exists() {
        println!("  ~ No wallpapers registered yet ({})", wdir.display());
        return Ok(());
    }
    let mut entries: Vec<_> = std::fs::read_dir(&wdir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
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
        println!("  → Mode sent to daemon: {}", mode);
    } else {
        let mut state = load_state();
        state.mode = mode.to_string();
        save_state(&state)?;
        println!("  → Mode saved: {} (daemon not running)", mode);
    }
    Ok(())
}

/// Called by the daemon when power state changes.
pub fn apply_for_power(_is_ac: bool) -> Result<()> {
    let state = load_state();
    if !state.current.is_empty() {
        set(&state.current)?;
    }
    Ok(())
}
