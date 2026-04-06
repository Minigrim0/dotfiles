use crate::wallpaper;
use anyhow::{Context, Result};
use serde_json::Value;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::net::UnixListener;
use tracing::{debug, error, info, warn};

fn is_on_ac() -> bool {
    let dir = "/sys/class/power_supply";
    match std::fs::read_dir(dir) {
        Err(e) => {
            warn!("could not read {}: {} — assuming AC", dir, e);
            true
        }
        Ok(entries) => {
            for entry in entries.flatten() {
                let online = entry.path().join("online");
                match std::fs::read_to_string(&online) {
                    Ok(val) => {
                        let is_online = val.trim() == "1";
                        debug!(
                            "{}: {}",
                            online.display(),
                            if is_online { "online" } else { "offline" }
                        );
                        if is_online {
                            return true;
                        }
                    }
                    Err(e) => debug!("could not read {}: {}", online.display(), e),
                }
            }
            false
        }
    }
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
    // consume the first (immediate) tick so the loop doesn't double-apply
    interval.tick().await;

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let ac = is_on_ac();
                debug!("power poll: {}", if ac { "AC" } else { "battery" });
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

    debug!("socket raw input: {:?}", line.trim());

    let v: Value = serde_json::from_str(line.trim())?;
    let cmd = v.get("cmd").and_then(|c: &Value| c.as_str()).unwrap_or("");
    info!("socket command: {}", cmd);

    match cmd {
        "set_mode" => {
            let mode = v
                .get("mode")
                .and_then(|m: &Value| m.as_str())
                .unwrap_or("auto");
            info!("setting mode to: {}", mode);
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
            info!("setting wallpaper to: {:?}", name);
            if !name.is_empty()
                && let Err(e) = wallpaper::set(name)
            {
                warn!("set_wallpaper failed: {}", e);
            }
            writer.write_all(b"{\"ok\":true}\n").await?;
        }
        "status" => {
            let state = wallpaper::load_state();
            debug!(
                "status request: current={:?} mode={:?}",
                state.current, state.mode
            );
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
