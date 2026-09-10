//! "Keep the screen awake" — a real idle inhibitor.
//!
//! This replaces the previous approach of killing hypridle outright. Killing it
//! worked, but it also took down the *whole* idle chain: lock, DPMS off and
//! suspend-if-laptop all live in hypridle.conf, so the screen stayed on and the
//! machine also stopped locking permanently, with nothing to say so.
//!
//! Instead a `systemd-inhibit` lock is held by a detached child. hypridle
//! honours it (`ignore_systemd_inhibit` defaults to false), logind honours it
//! for suspend, and releasing it restores the full chain untouched. The child's
//! pid is tracked in the runtime directory, which the kernel clears on reboot —
//! so a lock can never outlive the session that took it.

use crate::{bar, ok};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::{Command, Stdio};

const WHY: &str = "dots: keep the screen awake";

fn pidfile() -> PathBuf {
    dirs::runtime_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("dots-inhibit.pid")
}

/// Whether a pid is our inhibitor and not a recycled number.
///
/// Checking the command line matters: pids wrap, and a stale pidfile that
/// happens to name a live unrelated process would otherwise make `dots inhibit`
/// kill it.
fn is_our_inhibitor(pid: u32) -> bool {
    let Ok(cmdline) = std::fs::read_to_string(format!("/proc/{}/cmdline", pid)) else {
        return false;
    };
    cmdline.contains("systemd-inhibit") && cmdline.contains(WHY)
}

/// The pid of the running inhibitor, if there is one.
pub fn holder() -> Option<u32> {
    let pid: u32 = std::fs::read_to_string(pidfile()).ok()?.trim().parse().ok()?;
    is_our_inhibitor(pid).then_some(pid)
}

pub fn is_held() -> bool {
    holder().is_some()
}

fn notify(body: &str) {
    let _ = Command::new("notify-send")
        .args([
            "-a",
            "dots",
            "-h",
            "string:x-dunst-stack-tag:dots-inhibit",
            body,
        ])
        .status();
}

fn acquire() -> Result<()> {
    // `sleep infinity` is the held lock: systemd-inhibit owns the inhibition
    // for exactly as long as its child runs, so the child has to be something
    // that does nothing, forever, cheaply.
    let child = Command::new("systemd-inhibit")
        .args([
            "--what=idle:sleep",
            "--who=dots",
            &format!("--why={}", WHY),
            "--mode=block",
            "sleep",
            "infinity",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("spawning systemd-inhibit")?;

    let path = pidfile();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, child.id().to_string())
        .with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

fn release() -> Result<()> {
    if let Some(pid) = holder() {
        let _ = Command::new("kill").arg(pid.to_string()).status();
    }
    let _ = std::fs::remove_file(pidfile());
    Ok(())
}

/// `None` toggles; `Some(true)` / `Some(false)` set explicitly.
pub fn set(target: Option<bool>) -> Result<()> {
    let held = is_held();
    let want = target.unwrap_or(!held);

    if want == held {
        ok!(
            "Idle inhibitor already {}",
            if held { "held" } else { "released" }
        );
        return Ok(());
    }

    if want {
        acquire()?;
        ok!("Idle inhibited — the screen stays on");
        notify("Screen stays on");
    } else {
        release()?;
        ok!("Idle inhibitor released — lock and sleep resume");
        notify("Idle management resumed");
    }

    bar::refresh(bar::SIG_INHIBIT);
    Ok(())
}
