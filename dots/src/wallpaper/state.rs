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
        }
    }
}

pub fn save_state(state: &WallpaperState) -> Result<()> {
    let path = state_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = format!(
        "current   = {:?}\nmode      = {:?}\ndark_mode = {}\n",
        state.current, state.mode, state.dark_mode
    );
    std::fs::write(&path, content).with_context(|| format!("writing {}", path.display()))
}
