use crate::{arrow, ok};
use anyhow::{Context, Result};
use std::process::Command;

use crate::wallpaper::{self, load_state, reload_apps, save_state};

/// Named preset palettes: seed color fed to matugen instead of the wallpaper.
const PRESETS: &[(&str, &str)] = &[
    ("tokyo-night", "#7aa2f7"),
    ("catppuccin", "#cba6f7"),
    ("nord", "#88c0d0"),
    ("gruvbox", "#fe8019"),
];

pub fn preset_names() -> Vec<&'static str> {
    PRESETS.iter().map(|(name, _)| *name).collect()
}

fn preset_seed(name: &str) -> Option<&'static str> {
    PRESETS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, seed)| *seed)
}

/// Render a matugen palette from a seed color instead of a wallpaper.
fn render_seed(seed: &str, dark: bool) -> Result<()> {
    let mode = if dark { "dark" } else { "light" };
    let status = Command::new("matugen")
        .args(["color", "hex", seed, "-m", mode])
        .status()
        .context("running matugen")?;
    anyhow::ensure!(status.success(), "matugen exited with error");
    Ok(())
}

/// Pin a preset: colors stop following the wallpaper until `theme auto`.
pub fn set_preset(name: &str) -> Result<()> {
    let seed = preset_seed(name).ok_or_else(|| {
        anyhow::anyhow!(
            "unknown preset '{}' — available: {}",
            name,
            preset_names().join(", ")
        )
    })?;

    let mut state = load_state();
    render_seed(seed, state.dark_mode)?;
    reload_apps();
    state.pinned_theme = Some(name.to_string());
    save_state(&state)?;

    ok!("Theme pinned to '{}'", name);
    Ok(())
}

/// Unpin: re-derive the palette from the current wallpaper.
pub fn auto() -> Result<()> {
    let mut state = load_state();
    state.pinned_theme = None;
    save_state(&state)?;

    if state.current.is_empty() {
        arrow!("No wallpaper set yet — colors will follow the next `wallpaper set`");
    } else {
        wallpaper::set(&state.current)?;
    }
    ok!("Theme follows the wallpaper again");
    Ok(())
}

/// Flip between dark and light based on the saved state.
pub fn toggle() -> Result<()> {
    let state = load_state();
    set(!state.dark_mode)
}

pub fn set(dark: bool) -> Result<()> {
    let scheme = if dark { "prefer-dark" } else { "default" };
    let gtk_theme = if dark { "Adwaita-dark" } else { "Adwaita" };

    // GTK4 / libadwaita apps honour color-scheme
    Command::new("gsettings")
        .args(["set", "org.gnome.desktop.interface", "color-scheme", scheme])
        .status()
        .context("running gsettings")?;

    // GTK3 apps still use the theme name
    Command::new("gsettings")
        .args(["set", "org.gnome.desktop.interface", "gtk-theme", gtk_theme])
        .status()
        .context("running gsettings")?;

    let mut state = load_state();
    state.dark_mode = dark;
    save_state(&state)?;

    if let Some(pinned) = state.pinned_theme.clone() {
        // Re-render the pinned seed in the new mode; `wallpaper::set` would
        // skip matugen entirely while a preset is pinned.
        if let Some(seed) = preset_seed(&pinned) {
            render_seed(seed, dark)?;
            reload_apps();
        }
    } else if !state.current.is_empty() {
        // Re-apply current wallpaper so matugen regenerates the colour scheme
        wallpaper::set(&state.current)?;
    }

    ok!("Theme set to {}", if dark { "dark" } else { "light" });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_preset_seeds() {
        assert_eq!(preset_seed("tokyo-night"), Some("#7aa2f7"));
        assert_eq!(preset_seed("gruvbox"), Some("#fe8019"));
        assert_eq!(preset_seed("nope"), None);
        assert!(preset_names().contains(&"catppuccin"));
    }
}
