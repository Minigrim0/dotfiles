use anyhow::{Context, Result};
use std::process::Command;

use crate::wallpaper::{self, load_state, save_state};

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

    // Re-apply current wallpaper so matugen regenerates the colour scheme
    if !state.current.is_empty() {
        wallpaper::set(&state.current)?;
    }

    println!("  ✓ Theme set to {}", if dark { "dark" } else { "light" });
    Ok(())
}
