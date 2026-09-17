use crate::audit;
use crate::config::Manifest;
use anyhow::Result;
use std::path::Path;
use std::process::Command;

/// (label, pgrep pattern) — shared with `dots status`.
pub const DAEMONS: &[(&str, &str)] = &[
    ("waybar", "waybar"),
    ("dunst", "dunst"),
    ("awww-daemon", "awww-daemon"),
    ("nm-applet", "nm-applet"),
    ("udiskie", "udiskie"),
    ("wl-paste", "wl-paste"),
    ("gnome-keyring", "gnome-keyring-d"),
    ("polkit", "polkitd"),
    ("dots", "dots daemon"),
];

fn pass(msg: &str) {
    println!("  \x1b[32m✓\x1b[0m  {}", msg);
}

fn fail(msg: &str) {
    println!("  \x1b[31m✗\x1b[0m  {}", msg);
}

fn hint(msg: &str) {
    println!("     \x1b[2m{}\x1b[0m", msg);
}

fn heading(title: &str) {
    println!("\n\x1b[1m{}\x1b[0m", title);
    println!("{}", "─".repeat(48));
}

fn cmd_stdout(cmd: &str, args: &[&str]) -> String {
    Command::new(cmd)
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default()
}

/// Symlinks in $HOME that exist but whose target is gone or outside the repo.
fn check_symlinks(manifest: &Manifest, dotfiles: &Path, home: &Path) {
    heading("Symlinks");
    let repo = dotfiles
        .canonicalize()
        .unwrap_or_else(|_| dotfiles.to_path_buf());
    let mut broken = 0usize;
    let mut foreign = 0usize;

    for (name, module) in &manifest.modules {
        if !module.enabled {
            continue;
        }
        let cfg_dir = dotfiles.join("configs").join(module.configs_dir(name));
        if !cfg_dir.exists() {
            continue;
        }
        let Ok(cfg_canon) = cfg_dir.canonicalize() else {
            continue;
        };
        for entry in walkdir::WalkDir::new(&cfg_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let Ok(src) = entry.path().canonicalize() else {
                continue;
            };
            let Ok(rel) = src.strip_prefix(&cfg_canon) else {
                continue;
            };
            let dst = home.join(rel);
            if !dst.is_symlink() {
                continue;
            }
            match std::fs::canonicalize(&dst) {
                Err(_) => {
                    fail(&format!("broken link: {}", dst.display()));
                    broken += 1;
                }
                Ok(target) if !target.starts_with(&repo) => {
                    fail(&format!(
                        "points outside repo: {} → {}",
                        dst.display(),
                        target.display()
                    ));
                    foreign += 1;
                }
                Ok(_) => {}
            }
        }
    }
    if broken == 0 && foreign == 0 {
        pass("all module symlinks resolve into the repo");
    } else {
        hint("fix: dots sync");
    }
}

fn check_failed_units() {
    heading("Systemd user units");
    let out = cmd_stdout("systemctl", &["--user", "--failed", "--no-legend"]);
    let failed: Vec<&str> = out.lines().filter(|l| !l.trim().is_empty()).collect();
    if failed.is_empty() {
        pass("no failed user units");
    } else {
        for line in failed {
            fail(line.trim());
        }
    }
}

fn check_daemons() {
    heading("Daemons");
    for (label, pattern) in DAEMONS {
        let running = Command::new("pgrep")
            .args(["-f", pattern])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if running {
            pass(label);
        } else {
            fail(&format!("{} (not running)", label));
        }
    }
}

fn check_gpu() {
    heading("GPU (amdgpu)");
    let out = cmd_stdout("journalctl", &["-k", "-b", "0", "-p", "err", "--no-pager"]);
    let errors = out
        .lines()
        .filter(|l| {
            l.contains("SMU: No response")
                || l.contains("DMUB")
                || (l.contains("ring") && l.contains("reset"))
        })
        .count();
    if errors > 10 {
        fail(&format!("{} SMU/DMUB/ring-reset errors this boot", errors));
        hint("known Navi 33 gfxoff hang — boot with amdgpu.gfxoff=0");
    } else if errors > 0 {
        pass(&format!("only {} GPU error lines this boot", errors));
    } else {
        pass("no GPU errors this boot");
    }
}

fn check_journal_size() {
    heading("Journal");
    let out = cmd_stdout("journalctl", &["--disk-usage"]);
    // "Archived and active journals take up 890.7M in the file system."
    let size = out
        .split_whitespace()
        .find(|w| w.ends_with('M') || w.ends_with('G'))
        .unwrap_or("?")
        .to_string();
    let too_big = size.ends_with('G')
        || size
            .trim_end_matches('M')
            .parse::<f64>()
            .map(|m| m > 500.0)
            .unwrap_or(false);
    if too_big {
        fail(&format!("journal uses {}", size));
        hint("cap it: SystemMaxUse=200M in /etc/systemd/journald.conf.d/");
    } else {
        pass(&format!("journal uses {}", size));
    }
}

fn check_packages(manifest: &Manifest) {
    heading("Packages");
    match audit::run(manifest) {
        Ok(report) => {
            if report.missing.is_empty() {
                pass("all manifest packages installed");
            } else {
                fail(&format!(
                    "{} manifest packages missing",
                    report.missing.len()
                ));
                hint("fix: dots install");
            }
            if report.orphans.is_empty() {
                pass("no orphaned packages");
            } else {
                fail(&format!("{} orphaned packages", report.orphans.len()));
                hint("inspect: dots packages --audit");
            }
        }
        Err(e) => fail(&format!("audit failed: {}", e)),
    }
}

/// Which of the two Hyprland session entries you actually logged into.
///
/// SDDM offers both "Hyprland" and "Hyprland (uwsm-managed)", and they are not
/// interchangeable: only the uwsm one populates the systemd user manager and
/// the D-Bus activation environment. Picking the wrong one is silent — the
/// desktop comes up, and then Qt apps are grey and dots.service never starts.
fn check_session() {
    heading("Session");

    let uwsm_active = Command::new("systemctl")
        .args([
            "--user",
            "is-active",
            "--quiet",
            "wayland-wm@hyprland.service",
        ])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if uwsm_active {
        pass("uwsm-managed session (wayland-wm@hyprland.service)");
    } else {
        fail("not a uwsm session");
        hint("log out and pick \"Hyprland (uwsm-managed)\" at the greeter");
    }

    let graphical = Command::new("systemctl")
        .args(["--user", "is-active", "--quiet", "graphical-session.target"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if graphical {
        pass("graphical-session.target is active");
    } else {
        fail("graphical-session.target inactive — user units will not start");
    }

    // Proves configs/uwsm/.config/uwsm/env was actually sourced.
    match std::env::var("QT_QPA_PLATFORMTHEME") {
        Ok(v) if v == "qt6ct" => pass("session environment loaded (QT_QPA_PLATFORMTHEME=qt6ct)"),
        Ok(v) => fail(&format!(
            "QT_QPA_PLATFORMTHEME is '{}', expected 'qt6ct'",
            v
        )),
        Err(_) => {
            fail("QT_QPA_PLATFORMTHEME unset — uwsm env not loaded");
            hint("it lives in configs/uwsm/.config/uwsm/env; a non-uwsm session skips it");
        }
    }
}

/// The Secret Service. Without it Nextcloud, Signal and VS Code each fall back
/// to prompting or to plaintext, one app at a time, with no single symptom.
fn check_keyring(home: &Path) {
    heading("Keyring");

    let running = Command::new("pgrep")
        .args(["-x", "gnome-keyring-d"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if running {
        pass("gnome-keyring-daemon running");
    } else {
        fail("gnome-keyring-daemon not running");
    }

    // Auto-unlock is a PAM concern, and PAM lives outside this repo — so the
    // most it can do is say whether someone wired it up.
    let wired = ["/etc/pam.d/sddm", "/etc/pam.d/login"]
        .iter()
        .filter(|f| {
            std::fs::read_to_string(f)
                .map(|c| c.contains("pam_gnome_keyring.so"))
                .unwrap_or(false)
        })
        .count();
    if wired > 0 {
        pass(&format!("pam_gnome_keyring wired in {} PAM file(s)", wired));
    } else {
        fail("pam_gnome_keyring in neither /etc/pam.d/sddm nor /etc/pam.d/login");
        hint("without it the keyring asks for a password once per boot");
    }

    let control = home.join(".local/share/keyrings");
    if control.exists() {
        pass("keyring store present");
    } else {
        hint("no keyring created yet — the first app to store a secret makes one");
    }
}

/// Qt theming, which is invisible until you open a Qt app and it is grey.
fn check_qt(home: &Path) {
    heading("Qt");

    let conf = home.join(".config/qt6ct/qt6ct.conf");
    if !conf.exists() {
        fail("~/.config/qt6ct/qt6ct.conf missing");
        hint("fix: dots install qt   (renders it from qt6ct.conf.in)");
        return;
    }
    pass("qt6ct.conf present");

    let text = std::fs::read_to_string(&conf).unwrap_or_default();
    if text.contains("@HOME@") {
        fail("qt6ct.conf still contains the @HOME@ placeholder");
        hint("fix: dots install qt");
        return;
    }

    match text
        .lines()
        .find_map(|l| l.strip_prefix("color_scheme_path="))
    {
        Some(path) if Path::new(path.trim()).exists() => pass("matugen palette linked and present"),
        Some(path) => {
            fail(&format!("palette missing: {}", path.trim()));
            hint("fix: dots wallpaper set <name>   (re-renders every template)");
        }
        None => fail("qt6ct.conf has no color_scheme_path"),
    }
}

/// Portals decide where a screenshare prompt and a file dialog come from.
fn check_portals() {
    heading("Portals");
    for (label, unit) in [
        ("xdg-desktop-portal", "xdg-desktop-portal.service"),
        ("hyprland backend", "xdg-desktop-portal-hyprland.service"),
        ("gtk backend", "xdg-desktop-portal-gtk.service"),
    ] {
        let active = Command::new("systemctl")
            .args(["--user", "is-active", "--quiet", unit])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if active {
            pass(label);
        } else {
            // Portals are D-Bus activated, so "inactive" only means nothing has
            // asked yet. Worth reporting, not worth calling broken.
            hint(&format!(
                "{} inactive (D-Bus activated on first use)",
                label
            ));
        }
    }
}

/// Default applications. `xdg-open` silently picking the wrong thing is the
/// single most common "my desktop is not a desktop" symptom.
fn check_defaults(home: &Path) {
    heading("Default applications");

    let list = home.join(".config/mimeapps.list");
    let Ok(text) = std::fs::read_to_string(&list) else {
        fail("~/.config/mimeapps.list missing");
        hint("fix: dots sync xdg");
        return;
    };
    pass("mimeapps.list present");

    // What the repo *declares*, so it can be compared against what xdg-mime
    // actually resolves. The two diverge silently whenever a declared app is
    // not installed: xdg-mime skips the missing entry and falls through to
    // whatever else claims the type, which is how you end up opening folders
    // in a terminal without ever choosing to.
    let declared = parse_default_applications(&text);

    for (label, mime) in [
        ("web link", "x-scheme-handler/https"),
        ("pdf", "application/pdf"),
        ("directory", "inode/directory"),
        ("image", "image/png"),
        ("video", "video/mp4"),
    ] {
        let resolved = cmd_stdout("xdg-mime", &["query", "default", mime]);
        let resolved = resolved.trim();
        let want = declared.get(mime).map(|s| s.as_str());

        if resolved.is_empty() {
            fail(&format!("{:<10} no handler for {}", label, mime));
            continue;
        }
        match want {
            Some(want) if want != resolved => {
                fail(&format!("{:<10} → {} (declared {})", label, resolved, want));
                hint(&format!(
                    "{} is probably not installed — fix: dots install",
                    want
                ));
            }
            _ => pass(&format!("{:<10} → {}", label, resolved)),
        }
    }
}

/// The `[Default Applications]` section of a mimeapps.list, as mime → desktop
/// id. Only the first value of a semicolon-separated list matters — that is the
/// one xdg-mime prefers.
fn parse_default_applications(text: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let mut in_section = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_section = line == "[Default Applications]";
            continue;
        }
        if !in_section || line.starts_with('#') || line.is_empty() {
            continue;
        }
        if let Some((mime, apps)) = line.split_once('=')
            && let Some(first) = apps.split(';').next()
            && !first.trim().is_empty()
        {
            map.insert(mime.trim().to_string(), first.trim().to_string());
        }
    }
    map
}

/// The boot splash. Every failure mode here looks identical from the desktop —
/// you only find out by rebooting — so all four are worth naming separately.
fn check_splash(home: &Path) {
    heading("Boot splash");

    // 1. Is plymouth even asked to draw? This is the one that silently costs
    //    people an afternoon: plymouth installed, hooked and themed, and then
    //    left out of the cmdline.
    let cmdline = std::fs::read_to_string("/proc/cmdline").unwrap_or_default();
    if cmdline.split_whitespace().any(|w| w == "splash") {
        pass("kernel cmdline has `splash`");
    } else {
        fail("kernel cmdline has no `splash` — plymouth stays silent");
        hint(
            "add it to GRUB_CMDLINE_LINUX_DEFAULT, then `sudo grub-mkconfig -o /boot/grub/grub.cfg`",
        );
    }

    // 2. The hook, and the trap next to it.
    let hooks = std::fs::read_to_string("/etc/mkinitcpio.conf").unwrap_or_default();
    let hooks_line = hooks
        .lines()
        .find(|l| l.trim_start().starts_with("HOOKS="))
        .unwrap_or("");
    if hooks_line.contains("plymouth-encrypt") {
        fail("HOOKS uses plymouth-encrypt, which no longer ships");
        hint("use the stock `encrypt` hook — it calls `plymouth ask-for-password` itself");
    } else if hooks_line
        .split(|c: char| !c.is_alphanumeric() && c != '-')
        .any(|h| h == "plymouth")
    {
        pass("mkinitcpio HOOKS includes plymouth");
    } else {
        fail("mkinitcpio HOOKS has no plymouth hook");
    }

    // 3. Is our theme the selected one?
    let theme = cmd_stdout("plymouth-set-default-theme", &[])
        .trim()
        .to_string();
    if theme == "dots" {
        pass("default theme is `dots`");
    } else if theme.is_empty() {
        fail("could not read the default plymouth theme");
    } else {
        fail(&format!("default theme is `{}`, not `dots`", theme));
        hint("fix: dots splash apply");
    }

    // 4. Drift. The initramfs holds a copy, so the rendered theme and the
    //    installed one part ways on the next `wallpaper set` — and nothing
    //    else in the system will ever mention it.
    let rendered = home.join(".config/plymouth/themes/dots/dots.script");
    let installed = Path::new("/usr/share/plymouth/themes/dots/dots.script");
    match (
        std::fs::read_to_string(&rendered),
        std::fs::read_to_string(installed),
    ) {
        (Ok(a), Ok(b)) if a == b => pass("installed theme matches the rendered one"),
        (Ok(_), Ok(_)) => {
            fail("installed theme has drifted from the rendered one");
            hint("fix: dots splash apply   (reinstalls and rebuilds the initramfs)");
        }
        (Ok(_), Err(_)) => {
            fail("theme rendered but never installed");
            hint("fix: dots splash apply");
        }
        (Err(_), _) => {
            fail("theme not rendered");
            hint("fix: dots wallpaper set <name>, then dots splash apply");
        }
    }
}

pub fn run(manifest: &Manifest, dotfiles: &Path, home: &Path) -> Result<()> {
    check_symlinks(manifest, dotfiles, home);
    check_session();
    check_daemons();
    check_failed_units();
    check_keyring(home);
    check_qt(home);
    check_portals();
    check_defaults(home);
    check_splash(home);
    check_gpu();
    check_journal_size();
    check_packages(manifest);
    println!();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_applications_section() {
        let text = "\
# a comment
[Added Associations]
text/html=chromium.desktop

[Default Applications]
text/html=firefox.desktop
video/mp4=vlc-8.desktop;vlc-7.desktop
inode/directory=org.gnome.Nautilus.desktop

[Removed Associations]
image/png=gimp.desktop
";
        let map = parse_default_applications(text);
        assert_eq!(map.get("text/html").unwrap(), "firefox.desktop");
        // only the first of a semicolon list
        assert_eq!(map.get("video/mp4").unwrap(), "vlc-8.desktop");
        assert_eq!(
            map.get("inode/directory").unwrap(),
            "org.gnome.Nautilus.desktop"
        );
        // other sections are not defaults
        assert!(!map.contains_key("image/png"));
        assert_eq!(map.len(), 3);
    }
}
