//! Power profiles, over power-profiles-daemon.
//!
//! The profile is one of the few pieces of desktop state that is invisible and
//! consequential — a laptop left on `performance` is a laptop with two hours of
//! battery. So the bar shows it, but only when it is *not* balanced: the
//! default state is not news.

use crate::menu::wofi_grid;
use crate::{bar, ok};
use anyhow::{Context, Result};
use std::process::Command;

/// The three profiles power-profiles-daemon defines. `power-saver` and
/// `performance` are both optional — a desktop without a platform driver only
/// reports `balanced`, which is why the list is read rather than assumed.
pub const BALANCED: &str = "balanced";

fn ppctl(args: &[&str]) -> Result<String> {
    let out = Command::new("powerprofilesctl")
        .args(args)
        .output()
        .context("running powerprofilesctl — is power-profiles-daemon installed?")?;
    anyhow::ensure!(
        out.status.success(),
        "powerprofilesctl {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&out.stderr).trim()
    );
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

/// Parse `powerprofilesctl list`. Profiles are the unindented `name:` lines;
/// the active one is marked with a leading `*`.
///
/// ```text
/// * balanced:
///     CpuDriver:  amd_pstate_epp
///   performance:
/// ```
pub fn parse_list(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            let stripped = line.strip_prefix('*').unwrap_or(line);
            // Property lines are indented four spaces; profile lines one or two.
            if line.starts_with("    ") {
                return None;
            }
            let name = stripped.trim().strip_suffix(':')?;
            (!name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'))
                .then(|| name.to_string())
        })
        .collect()
}

pub fn available() -> Result<Vec<String>> {
    Ok(parse_list(&ppctl(&["list"])?))
}

/// The active profile, or None when the daemon is not answering. Never fails
/// hard — a missing profile should blank the bar, not raise a toast.
pub fn current() -> Option<String> {
    let out = Command::new("powerprofilesctl").arg("get").output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}

fn notify(body: &str) {
    let _ = Command::new("notify-send")
        .args([
            "-a",
            "dots",
            "-h",
            "string:x-dunst-stack-tag:dots-power",
            body,
        ])
        .status();
}

pub fn set(profile: &str) -> Result<()> {
    let choices = available()?;
    anyhow::ensure!(
        choices.iter().any(|p| p == profile),
        "unknown profile '{}' — this machine offers: {}",
        profile,
        choices.join(", ")
    );
    ppctl(&["set", profile])?;
    ok!("Power profile: {}", profile);
    notify(&format!("Power profile: {}", profile));
    bar::refresh(bar::SIG_POWER);
    Ok(())
}

pub fn get() -> Result<()> {
    match current() {
        Some(p) => println!("{}", p),
        None => anyhow::bail!("power-profiles-daemon is not answering"),
    }
    Ok(())
}

pub fn icon(profile: &str) -> &'static str {
    match profile {
        "performance" => "󰓅",
        "power-saver" => "󰌪",
        _ => "󰾅",
    }
}

/// `dots power` — the picker. Only offers what the machine actually has.
pub fn menu() -> Result<()> {
    let choices = available()?;
    let active = current();
    let items: Vec<(&str, &str)> = choices
        .iter()
        .map(|p| (icon(p), p.as_str()))
        .collect();

    let prompt = match &active {
        Some(a) => format!("power — now {}", a),
        None => "power".to_string(),
    };
    match wofi_grid(&prompt, &items) {
        Some(choice) => set(&choice),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_profile_listing() {
        let text = "\
* balanced:
    CpuDriver:  amd_pstate_epp
    PlatformDriver: placeholder
    Degraded:   no

  performance:
    CpuDriver:  amd_pstate_epp

  power-saver:
    CpuDriver:  amd_pstate_epp
";
        assert_eq!(
            parse_list(text),
            vec!["balanced", "performance", "power-saver"]
        );
    }

    #[test]
    fn handles_a_single_profile_machine() {
        assert_eq!(parse_list("* balanced:\n    CpuDriver: none\n"), vec!["balanced"]);
    }

    #[test]
    fn ignores_indented_properties_that_look_like_profiles() {
        // "Degraded:" is a property, not a profile — four-space indent.
        let text = "* balanced:\n    Degraded:   no\n";
        assert_eq!(parse_list(text), vec!["balanced"]);
    }

    #[test]
    fn picks_an_icon_per_profile() {
        assert_eq!(icon("performance"), "󰓅");
        assert_eq!(icon("power-saver"), "󰌪");
        assert_eq!(icon("balanced"), "󰾅");
        assert_eq!(icon("something-else"), "󰾅");
    }
}
