use crate::wallpaper;
use anyhow::{Context, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::net::UnixListener;
use tracing::{error, info, warn};

fn is_on_ac() -> bool {
    if let Ok(entries) = std::fs::read_dir("/sys/class/power_supply") {
        for entry in entries.flatten() {
            let online = entry.path().join("online");
            if std::fs::read_to_string(&online)
                .map(|s| s.trim() == "1")
                .unwrap_or(false)
            {
                return true;
            }
        }
    }
    true // default: assume AC if unreadable
}

pub async fn run() -> Result<()> {
    let socket_path = dirs::runtime_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("dots.sock");

    let _ = std::fs::remove_file(&socket_path);

    let listener = UnixListener::bind(&socket_path)
        .with_context(|| format!("binding Unix socket at {}", socket_path.display()))?;

    info!("daemon started; socket: {}", socket_path.display());

    let mut last_ac = is_on_ac();
    info!(
        "initial power state: {}",
        if last_ac { "AC" } else { "battery" }
    );

    if let Err(e) = wallpaper::apply_for_power(last_ac) {
        warn!("initial wallpaper apply failed: {}", e);
    }

    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let ac = is_on_ac();
                if ac != last_ac {
                    last_ac = ac;
                    info!("power state changed: {}", if ac { "AC" } else { "battery" });
                    if let Err(e) = wallpaper::apply_for_power(ac) {
                        warn!("wallpaper apply failed: {}", e);
                    }
                }
            }
            conn = listener.accept() => {
                match conn {
                    Ok((stream, _)) => {
                        tokio::spawn(async move {
                            if let Err(e) = handle_connection(stream).await {
                                warn!("socket connection error: {}", e);
                            }
                        });
                    }
                    Err(e) => error!("accept error: {}", e),
                }
            }
        }
    }
}

async fn handle_connection(mut stream: tokio::net::UnixStream) -> Result<()> {
    let (reader, mut writer) = stream.split();
    let mut reader = tokio::io::BufReader::new(reader);
    let mut line = String::new();
    reader.read_line(&mut line).await?;

    let v: Value = serde_json::from_str(line.trim())?;
    let cmd = v.get("cmd").and_then(|c: &Value| c.as_str()).unwrap_or("");
    info!("socket command: {}", cmd);

    match cmd {
        "set_mode" => {
            let mode = v
                .get("mode")
                .and_then(|m: &Value| m.as_str())
                .unwrap_or("auto");
            let mut state = wallpaper::load_state();
            state.mode = mode.to_string();
            wallpaper::save_state(&state)?;
            if let Err(e) = wallpaper::apply_for_power(is_on_ac()) {
                warn!("wallpaper apply failed after mode change: {}", e);
            }
            writer.write_all(b"{\"ok\":true}\n").await?;
        }
        "set_wallpaper" => {
            let name = v.get("name").and_then(|m: &Value| m.as_str()).unwrap_or("");
            if !name.is_empty()
                && let Err(e) = wallpaper::set(name)
            {
                warn!("set_wallpaper failed: {}", e);
            }
            writer.write_all(b"{\"ok\":true}\n").await?;
        }
        "status" => {
            let state = wallpaper::load_state();
            let resp = format!(
                "{{\"ok\":true,\"data\":{{\"current\":{:?},\"mode\":{:?}}}}}\n",
                state.current, state.mode
            );
            writer.write_all(resp.as_bytes()).await?;
        }
        _ => {
            warn!("unknown socket command: {:?}", cmd);
            writer
                .write_all(b"{\"ok\":false,\"error\":\"unknown command\"}\n")
                .await?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Service setup
// ---------------------------------------------------------------------------

// ExecStart uses %h (systemd home dir expansion) so it works for any user.
const SERVICE: &str = r#"[Unit]
Description=dots wallpaper daemon
After=graphical-session.target
PartOf=graphical-session.target

[Service]
ExecStart=%h/.local/bin/dots daemon
Environment=RUST_LOG=dots=info
Restart=on-failure
RestartSec=5

[Install]
WantedBy=graphical-session.target
"#;

/// Install binary symlink, write service file, reload + enable the unit.
pub fn setup_service(dotfiles: &Path) -> Result<()> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/root"));

    // 1. Symlink release binary to ~/.local/bin/dots
    let bin_dir = home.join(".local/bin");
    std::fs::create_dir_all(&bin_dir)?;
    let link = bin_dir.join("dots");
    let src = dotfiles.join("dots/target/release/dots");

    if !src.exists() {
        anyhow::bail!(
            "release binary not found at {}; run `cargo build --release` first",
            src.display()
        );
    }

    if link.exists() || link.is_symlink() {
        std::fs::remove_file(&link)?;
    }
    std::os::unix::fs::symlink(&src, &link)
        .with_context(|| format!("symlinking {} → {}", src.display(), link.display()))?;
    println!("  \x1b[36m→\x1b[0m  {} → {}", link.display(), src.display());

    // 2. Write service file
    let svc_dir = home.join(".config/systemd/user");
    std::fs::create_dir_all(&svc_dir)?;
    let svc_path = svc_dir.join("dots.service");
    std::fs::write(&svc_path, SERVICE)?;
    println!("  \x1b[36m→\x1b[0m  wrote {}", svc_path.display());

    // 3. daemon-reload
    Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status()
        .context("systemctl daemon-reload")?;

    // 4. enable (don't start — needs graphical session)
    let status = Command::new("systemctl")
        .args(["--user", "enable", "dots"])
        .status()
        .context("systemctl enable dots")?;

    if status.success() {
        println!("  \x1b[32m✓\x1b[0m  dots.service enabled");
    } else {
        println!("  \x1b[33m~\x1b[0m  systemctl enable returned non-zero");
    }

    Ok(())
}
