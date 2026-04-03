use std::path::PathBuf;

pub fn wallpaper_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/root"))
        .join(".config/wallpaper")
}
