use crate::config::{MachineConfig, Module};
use crate::{arrow, head, ok, warn};
use anyhow::{Context, Result};
use std::path::Path;
use walkdir::WalkDir;

pub enum LinkStatus {
    Ok,
    Created,
    Updated,
    BackedUp,
}

fn link(src: &Path, dst: &Path) -> Result<LinkStatus> {
    // Already correct symlink
    if dst.is_symlink() {
        if std::fs::read_link(dst).map(|t| t == src).unwrap_or(false) {
            return Ok(LinkStatus::Ok);
        }
        // Wrong target — remove and relink
        std::fs::remove_file(dst).with_context(|| format!("removing {}", dst.display()))?;
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::os::unix::fs::symlink(src, dst)
            .with_context(|| format!("symlinking {} → {}", src.display(), dst.display()))?;
        return Ok(LinkStatus::Updated);
    }

    // Real file — back it up
    if dst.exists() {
        let backup = dst.with_extension(format!(
            "{}.bak",
            dst.extension().and_then(|e| e.to_str()).unwrap_or("")
        ));
        std::fs::rename(dst, &backup).with_context(|| format!("backing up {}", dst.display()))?;
        std::os::unix::fs::symlink(src, dst)
            .with_context(|| format!("symlinking {} → {}", src.display(), dst.display()))?;
        return Ok(LinkStatus::BackedUp);
    }

    // Doesn't exist — create parents and symlink
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating dir {}", parent.display()))?;
    }
    std::os::unix::fs::symlink(src, dst)
        .with_context(|| format!("symlinking {} → {}", src.display(), dst.display()))?;
    Ok(LinkStatus::Created)
}

fn print_link_status(status: LinkStatus, dst: &Path) {
    match status {
        LinkStatus::Ok => ok!("{}", dst.display()),
        LinkStatus::Created => arrow!("Created: {}", dst.display()),
        LinkStatus::Updated => arrow!("Updated: {}", dst.display()),
        LinkStatus::BackedUp => arrow!("Backed up and linked: {}", dst.display()),
    }
}

pub fn sync_module(name: &str, module: &Module, dotfiles: &Path, home: &Path) -> Result<()> {
    let module_dir = dotfiles.join("configs").join(module.configs_dir(name));
    if !module_dir.exists() {
        warn!("No configs dir for module '{}'", name);
        return Ok(());
    }

    head!("Syncing module: {}", name);

    for entry in WalkDir::new(&module_dir)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let src = entry
            .path()
            .canonicalize()
            .with_context(|| format!("canonicalizing {}", entry.path().display()))?;

        let rel = src
            .strip_prefix(module_dir.canonicalize()?)
            .with_context(|| "stripping prefix")?;
        let dst = home.join(rel);

        print_link_status(link(&src, &dst)?, &dst);
    }

    Ok(())
}

pub fn apply_machine_symlinks(dotfiles: &Path, mc: &MachineConfig, home: &Path) -> Result<()> {
    let name = &mc.meta.name;

    // ~/.config/hypr/machine.conf → configs/hyprland/.config/hypr/machine-<name>.conf
    let hypr_src = dotfiles
        .join("configs/hyprland/.config/hypr")
        .join(format!("machine-{}.conf", name));
    let hypr_dst = home.join(".config/hypr/machine.conf");

    if hypr_src.exists() {
        print_link_status(link(&hypr_src, &hypr_dst)?, &hypr_dst);
    } else {
        warn!("machine-{}.conf not found, skipping hyprland machine link", name);
    }

    // ~/.config/waybar/scripts/brightness-backend.sh → configs/waybar/.config/waybar/scripts/brightness-<name>.sh
    let waybar_brightness_src = dotfiles
        .join("configs/waybar/.config/waybar/scripts")
        .join(format!("brightness-{}.sh", name));
    let waybar_brightness_dst = home.join(".config/waybar/scripts/brightness-backend.sh");

    if waybar_brightness_src.exists() {
        print_link_status(link(&waybar_brightness_src, &waybar_brightness_dst)?, &waybar_brightness_dst);
    } else {
        warn!("brightness-{}.sh not found, skipping waybar brightness link", name);
    }

    // ~/.config/waybar/config.jsonc → configs/waybar/.config/waybar/config-<name>.jsonc (if exists)
    let waybar_cfg_src = dotfiles
        .join("configs/waybar/.config/waybar")
        .join(format!("config-{}.jsonc", name));
    let waybar_cfg_dst = home.join(".config/waybar/config.jsonc");

    if waybar_cfg_src.exists() {
        print_link_status(link(&waybar_cfg_src, &waybar_cfg_dst)?, &waybar_cfg_dst);
    }

    Ok(())
}
