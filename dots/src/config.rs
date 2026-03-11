#![allow(dead_code)]
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub meta: Meta,
    pub modules: HashMap<String, Module>,
}

#[derive(Debug, Deserialize)]
pub struct Meta {
    pub version: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Module {
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// configs/ subdirectory name; defaults to the module key if omitted
    #[serde(default)]
    pub configs: Option<String>,
    #[serde(default)]
    pub packages: Vec<String>,
    #[serde(default)]
    pub aur_packages: Vec<String>,
    #[serde(default)]
    pub hooks: Hooks,
}

impl Module {
    pub fn configs_dir<'a>(&'a self, name: &'a str) -> &'a str {
        self.configs.as_deref().unwrap_or(name)
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Hooks {
    #[serde(default)]
    pub pre_install: Vec<String>,
    #[serde(default)]
    pub post_install: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct MachineConfig {
    pub meta: MachineMeta,
    #[serde(default)]
    pub packages: MachinePackages,
    #[serde(default)]
    pub wallpaper: WallpaperConfig,
    #[serde(default)]
    pub hyprland: HyprlandConfig,
    #[serde(default)]
    pub waybar: WaybarConfig,
}

#[derive(Debug, Deserialize)]
pub struct MachineMeta {
    pub name: String,
    pub hostname: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct MachinePackages {
    #[serde(default)]
    pub extra: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct WallpaperConfig {
    pub animated_on_ac: bool,
    pub static_on_battery: bool,
}

#[derive(Debug, Deserialize, Default)]
pub struct HyprlandConfig {
    pub brightness_up: String,
    pub brightness_down: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct WaybarConfig {
    #[serde(default)]
    pub extra_modules_right: Vec<String>,
}

/// Resolve the dotfiles root directory.
/// Order:
///   1. $DOTFILES_DIR env var
///   2. ~/.config/dots/config.toml → dotfiles_dir key
///   3. Walk ancestors of cwd looking for modules.toml
///   4. Fallback: ~/Documents/dotfiles
pub fn dotfiles_dir() -> PathBuf {
    // 1. env var
    if let Ok(dir) = std::env::var("DOTFILES_DIR") {
        return PathBuf::from(dir);
    }

    // 2. ~/.config/dots/config.toml
    if let Some(home) = dirs::home_dir() {
        let cfg = home.join(".config/dots/config.toml");
        if cfg.exists()
            && let Ok(content) = std::fs::read_to_string(&cfg)
            && let Ok(t) = toml::from_str::<toml::Value>(&content)
            && let Some(dir) = t.get("dotfiles_dir").and_then(|v| v.as_str())
        {
            return PathBuf::from(dir);
        }
    }

    // 3. Walk ancestors of cwd for modules.toml
    if let Ok(cwd) = std::env::current_dir() {
        let mut current = cwd.as_path();
        loop {
            if current.join("modules.toml").exists() {
                return current.to_path_buf();
            }
            match current.parent() {
                Some(parent) => current = parent,
                None => break,
            }
        }
    }

    // 4. Fallback
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/root"))
        .join("Documents/dotfiles")
}

pub fn load_manifest(dotfiles: &Path) -> Result<Manifest> {
    let path = dotfiles.join("modules.toml");
    let content =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    toml::from_str(&content).with_context(|| format!("parsing {}", path.display()))
}

pub fn load_machine(dotfiles: &Path, name: &str) -> Result<MachineConfig> {
    let path = dotfiles.join("machines").join(format!("{}.toml", name));
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("reading machine config {}", path.display()))?;
    toml::from_str(&content).with_context(|| format!("parsing {}", path.display()))
}
