//! The boot splash: a plymouth `script` theme rendered from the matugen palette.
//!
//! Plymouth is the one themed surface that cannot follow the wallpaper on its
//! own. mkinitcpio bakes a *copy* of the theme into the initramfs, so a new
//! palette does not reach the boot screen until the theme is reinstalled to
//! /usr/share and the initramfs is rebuilt — a ~30s, root-owned operation that
//! has no business running on every `wallpaper set`. Hence an explicit
//! `dots splash apply`.
//!
//! Three kinds of file end up in ~/.config/plymouth/themes/dots:
//!   - dots.plymouth, logo.src.jpg   symlinked from the repo by `dots sync`
//!   - dots.script, palette.conf     rendered by matugen
//!   - logo/entry/bullet .png        drawn here, because plymouth's script
//!                                   module composites images and cannot draw
//!                                   a shape

use crate::{arrow, head, ok, warn};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

const THEME: &str = "dots";
const SYSTEM_DIR: &str = "/usr/share/plymouth/themes/dots";

/// Entry pill geometry. Fixed rather than resolution-derived: the logo is
/// composited 1:1, so scaling the furniture around it would only break the
/// proportion between the two.
const ENTRY_W: u32 = 420;
const ENTRY_H: u32 = 52;
const BULLET_D: u32 = 10;

fn theme_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/root"))
        .join(".config/plymouth/themes")
        .join(THEME)
}

/// ImageMagick 7 is `magick`; 6 is `convert`. Both are in the repositories and
/// the repo pins neither, so pick whichever answers.
fn magick() -> Result<&'static str> {
    for bin in ["magick", "convert"] {
        if Command::new(bin)
            .arg("-version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Ok(bin);
        }
    }
    anyhow::bail!("neither `magick` nor `convert` found — install imagemagick")
}

/// Parse the `key=#rrggbb` file matugen renders next to the script.
fn load_palette(dir: &Path) -> Result<HashMap<String, String>> {
    let path = dir.join("palette.conf");
    let text = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "reading {} — run `dots wallpaper set <name>` first, so matugen \
             renders the plymouth templates",
            path.display()
        )
    })?;

    let mut map = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    Ok(map)
}

fn color<'a>(palette: &'a HashMap<String, String>, key: &str) -> Result<&'a str> {
    palette
        .get(key)
        .map(String::as_str)
        .ok_or_else(|| anyhow::anyhow!("palette.conf has no '{}' key", key))
}

fn run(bin: &str, args: &[&str]) -> Result<()> {
    let out = Command::new(bin)
        .args(args)
        .output()
        .with_context(|| format!("running {}", bin))?;
    anyhow::ensure!(
        out.status.success(),
        "{} failed: {}",
        bin,
        String::from_utf8_lossy(&out.stderr).trim()
    );
    Ok(())
}

/// Draw the three PNGs the script composites.
///
/// The logo is a straight JPEG→PNG transcode: no scale, no mask. Its outer
/// ring is a flat near-black field, so on the theme's black ground the square
/// is already edgeless — feathering it would only clip the eyelid and leave a
/// halo.
fn render_assets(dir: &Path) -> Result<()> {
    let im = magick()?;
    let palette = load_palette(dir)?;

    let src = dir.join("logo.src.jpg");
    anyhow::ensure!(
        src.exists(),
        "{} is missing — `dots sync` links it from the repo",
        src.display()
    );

    let logo = dir.join("logo.png");
    run(
        im,
        &[
            &src.to_string_lossy(),
            "-strip",
            &format!("PNG24:{}", logo.display()),
        ],
    )?;

    let (w, h) = (ENTRY_W as f64, ENTRY_H as f64);
    let r = h / 2.0 - 1.0;
    run(
        im,
        &[
            "-size",
            &format!("{}x{}", ENTRY_W, ENTRY_H),
            "xc:none",
            "-fill",
            color(&palette, "entry_fill")?,
            "-stroke",
            color(&palette, "entry_stroke")?,
            "-strokewidth",
            "1",
            // Half-pixel inset so the 1px stroke lands inside the canvas
            // instead of being clipped in half by the edge.
            "-draw",
            &format!("roundrectangle 0.5,0.5 {},{} {},{}", w - 0.5, h - 0.5, r, r),
            "-strip",
            &format!("PNG32:{}", dir.join("entry.png").display()),
        ],
    )?;

    let c = BULLET_D as f64 / 2.0;
    run(
        im,
        &[
            "-size",
            &format!("{}x{}", BULLET_D, BULLET_D),
            "xc:none",
            "-fill",
            color(&palette, "bullet")?,
            "-stroke",
            "none",
            "-draw",
            &format!("circle {},{} {},0", c, c, c),
            "-strip",
            &format!("PNG32:{}", dir.join("bullet.png").display()),
        ],
    )?;

    arrow!("assets · logo.png, entry.png, bullet.png");
    Ok(())
}

/// Copy the theme to /usr/share, select it, and rebuild the initramfs.
fn install(dir: &Path) -> Result<()> {
    let script = dir.join("dots.script");
    anyhow::ensure!(
        script.exists(),
        "{} is missing — run `dots wallpaper set <name>` so matugen renders it",
        script.display()
    );

    // -T so the destination is the directory itself rather than a nested copy
    // on a second run.
    run("sudo", &["cp", "-rLT", &dir.to_string_lossy(), SYSTEM_DIR])?;
    arrow!("installed · {}", SYSTEM_DIR);

    run("sudo", &["plymouth-set-default-theme", THEME])?;
    arrow!("default theme · {}", THEME);

    arrow!("rebuilding initramfs (this is the slow part)…");
    let status = Command::new("sudo")
        .args(["mkinitcpio", "-P"])
        .status()
        .context("running mkinitcpio")?;
    anyhow::ensure!(
        status.success(),
        "mkinitcpio failed — boot config unchanged"
    );

    Ok(())
}

/// Render assets, install the theme, rebuild the initramfs.
pub fn apply() -> Result<()> {
    head!("Boot splash");
    let dir = theme_dir();
    anyhow::ensure!(
        dir.exists(),
        "{} does not exist — run `dots sync` first",
        dir.display()
    );

    render_assets(&dir)?;
    install(&dir)?;

    ok!("Splash applied — visible on the next boot");
    if !kernel_has_splash() {
        warn!("kernel cmdline has no `splash` — plymouth will stay silent");
        arrow!(
            "add it to GRUB_CMDLINE_LINUX_DEFAULT and run `sudo grub-mkconfig -o /boot/grub/grub.cfg`"
        );
    }
    Ok(())
}

/// True when the *running* kernel was booted with `splash`. On a fresh setup
/// this lags the grub config by one reboot, so `dots doctor` reads the grub
/// default instead; here it is only a hint.
pub fn kernel_has_splash() -> bool {
    std::fs::read_to_string("/proc/cmdline")
        .map(|s| s.split_whitespace().any(|w| w == "splash"))
        .unwrap_or(false)
}

/// Compose a PNG mock of the splash at the given size and print its path.
///
/// Worth having because the real thing is only observable by rebooting: this
/// reproduces the script's layout arithmetic without touching /usr/share, the
/// initramfs or the bootloader.
pub fn preview(width: u32, height: u32) -> Result<()> {
    head!("Splash preview");
    let dir = theme_dir();
    render_assets(&dir)?;

    let im = magick()?;
    let palette = load_palette(&dir)?;

    // Mirrors dots.script. Kept in step by hand — there is no way to ask
    // plymouth to render offscreen.
    let logo_dim = image_size(im, &dir.join("logo.png"))?;
    let logo_x = width as i64 / 2 - logo_dim.0 as i64 / 2;
    let logo_y = (height as f64 * 0.42) as i64 - logo_dim.1 as i64 / 2;
    let entry_y = logo_y + logo_dim.1 as i64 + 64;
    let entry_x = width as i64 / 2 - ENTRY_W as i64 / 2;
    let label_y = entry_y + ENTRY_H as i64 + 16;

    let bullets = 6i64;
    let gap = BULLET_D as i64 + 8;
    let start_x = width as i64 / 2 - (bullets * gap - 8) / 2;
    let mid_y = entry_y + ENTRY_H as i64 / 2 - BULLET_D as i64 / 2;

    let out = std::env::temp_dir().join("dots-splash-preview.png");
    let mut args: Vec<String> = vec![
        "-size".into(),
        format!("{}x{}", width, height),
        "xc:black".into(),
        dir.join("logo.png").to_string_lossy().into_owned(),
        "-geometry".into(),
        format!("+{}+{}", logo_x, logo_y),
        "-composite".into(),
        dir.join("entry.png").to_string_lossy().into_owned(),
        "-geometry".into(),
        format!("+{}+{}", entry_x, entry_y),
        "-composite".into(),
    ];
    for i in 0..bullets {
        args.extend([
            dir.join("bullet.png").to_string_lossy().into_owned(),
            "-geometry".into(),
            format!("+{}+{}", start_x + i * gap, mid_y),
            "-composite".into(),
        ]);
    }
    // Pango sizes the theme's font in points at 96dpi; ImageMagick's
    // -pointsize is 72dpi. The 4/3 keeps the mock's label the size the real
    // one will be.
    args.extend([
        "-font".into(),
        "Adwaita-Sans".into(),
        "-pointsize".into(),
        format!("{}", 12 * 4 / 3),
        "-fill".into(),
        color(&palette, "muted")?.to_string(),
        "-gravity".into(),
        "north".into(),
        "-annotate".into(),
        format!("+0+{}", label_y),
        "Enter passphrase".into(),
        "-strip".into(),
        out.to_string_lossy().into_owned(),
    ]);

    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    run(im, &refs)?;

    ok!("{} ({}x{})", out.display(), width, height);
    Ok(())
}

fn image_size(im: &str, path: &Path) -> Result<(u32, u32)> {
    let out = Command::new(im)
        .args([&path.to_string_lossy(), "-format", "%w %h", "info:"])
        .output()
        .context("reading image size")?;
    anyhow::ensure!(out.status.success(), "could not read {}", path.display());
    let s = String::from_utf8_lossy(&out.stdout);
    let mut it = s.split_whitespace();
    let w = it.next().and_then(|v| v.parse().ok());
    let h = it.next().and_then(|v| v.parse().ok());
    match (w, h) {
        (Some(w), Some(h)) => Ok((w, h)),
        _ => anyhow::bail!(
            "unexpected size output for {}: {}",
            path.display(),
            s.trim()
        ),
    }
}
