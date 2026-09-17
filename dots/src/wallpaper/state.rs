use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

fn default_dark_mode() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct WallpaperState {
    pub current: String,
    pub mode: String,
    #[serde(default = "default_dark_mode")]
    pub dark_mode: bool,
    /// Name of a pinned theme preset; None = colors follow the wallpaper.
    #[serde(default)]
    pub pinned_theme: Option<String>,
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
        s
    } else {
        tracing::warn!("Unable to read wallpaper state, returning default");
        WallpaperState {
            current: String::new(),
            mode: "auto".to_string(),
            dark_mode: true,
            pinned_theme: None,
        }
    }
}

pub fn save_state(state: &WallpaperState) -> Result<()> {
    let path = state_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string(state).context("serializing wallpaper state")?;
    std::fs::write(&path, content).with_context(|| format!("writing {}", path.display()))
}

// ---------------------------------------------------------------------------
// Render progress
// ---------------------------------------------------------------------------

/// Marker for "a wallpaper render is in flight", read by `dots bar wallpaper`.
/// `dots wallpaper set` runs matugen and, for animated wallpapers, ffmpeg —
/// long enough to wonder whether it worked.
fn render_marker() -> PathBuf {
    dirs::state_dir()
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/root"))
                .join(".local/state")
        })
        .join("dots/wallpaper-rendering")
}

pub fn is_rendering() -> bool {
    render_marker().exists()
}

/// RAII: the marker is cleared however `set` returns, including on error.
pub struct RenderGuard;

impl RenderGuard {
    pub fn start() -> Self {
        let path = render_marker();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, "1");
        crate::bar::refresh(crate::bar::SIG_WALLPAPER);
        Self
    }
}

impl Drop for RenderGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(render_marker());
        crate::bar::refresh(crate::bar::SIG_WALLPAPER);
    }
}
