use crate::menu::wofi_pick;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub struct Bind {
    pub keys: String,
    pub desc: String,
}

/// Parse hyprland config text into displayable keybinds.
/// A comment on the line directly above a bind becomes its description;
/// otherwise the dispatcher + args are shown.
pub fn parse_binds(content: &str) -> Vec<Bind> {
    let mut vars: HashMap<String, String> = HashMap::new();
    let mut binds = Vec::new();
    let mut last_comment: Option<(usize, String)> = None;

    let substitute = |vars: &HashMap<String, String>, s: &str| {
        let mut out = s.to_string();
        for (k, v) in vars {
            out = out.replace(k, v);
        }
        out
    };

    for (idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Variable definition: $name = value
        if trimmed.starts_with('$')
            && let Some((name, value)) = trimmed.split_once('=')
        {
            vars.insert(name.trim().to_string(), value.trim().to_string());
            continue;
        }

        // Comment — remember it if it's real text, not a ruler
        if let Some(text) = trimmed.strip_prefix('#') {
            let text = text.trim();
            if !text.is_empty() && !text.chars().all(|c| "-=~ ".contains(c)) {
                last_comment = Some((idx, text.to_string()));
            }
            continue;
        }

        // bind / bindel / bindl / bindm lines
        let Some(rest) = trimmed
            .strip_prefix("bind")
            .and_then(|r| r.split_once('='))
            .filter(|_| trimmed.starts_with("bind"))
            .map(|(_flags, rhs)| rhs)
        else {
            continue;
        };

        let parts: Vec<&str> = rest.splitn(4, ',').map(|p| p.trim()).collect();
        if parts.len() < 3 {
            continue;
        }
        let mods = substitute(&vars, parts[0]);
        let key = parts[1];
        let action = parts[2..].join(" ");

        let keys = if mods.is_empty() {
            key.to_string()
        } else {
            format!("{} + {}", mods.replace(' ', " + "), key)
        };

        let desc = match &last_comment {
            Some((comment_idx, text)) if idx == comment_idx + 1 => text.clone(),
            _ => substitute(&vars, &action),
        };
        last_comment = None;

        binds.push(Bind { keys, desc });
    }
    binds
}

fn config_paths() -> Vec<PathBuf> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/root"));
    vec![
        home.join(".config/hypr/hyprland.conf"),
        home.join(".config/hypr/machine.conf"),
    ]
}

pub fn show() -> Result<()> {
    let mut binds = Vec::new();
    for path in config_paths() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            binds.extend(parse_binds(&content));
        }
    }
    anyhow::ensure!(!binds.is_empty(), "no keybinds found in hypr configs");

    let width = binds.iter().map(|b| b.keys.len()).max().unwrap_or(0) + 4;
    let lines: Vec<String> = binds
        .iter()
        .map(|b| format!("{:<w$}{}", b.keys, b.desc, w = width))
        .collect();

    wofi_pick("keybinds", &lines).context("wofi closed")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_binds_with_variables_and_comments() {
        let conf = r#"
$mainMod = SUPER
$terminal = kitty

# Screenshot
bind = $mainMod SHIFT, F, exec, hyprshot -m region
bind = $mainMod, Return, exec, $terminal
bindel = , XF86AudioRaiseVolume, exec, wpctl set-volume 5%+
bindm = $mainMod, mouse:272, movewindow
"#;
        let binds = parse_binds(conf);
        assert_eq!(binds.len(), 4);

        assert_eq!(binds[0].keys, "SUPER + SHIFT + F");
        assert_eq!(binds[0].desc, "Screenshot"); // comment directly above

        assert_eq!(binds[1].keys, "SUPER + Return");
        assert_eq!(binds[1].desc, "exec kitty"); // variable substituted

        assert_eq!(binds[2].keys, "XF86AudioRaiseVolume"); // empty mods
        assert_eq!(binds[3].desc, "movewindow"); // bindm, no args
    }

    #[test]
    fn ignores_rulers_and_non_bind_lines() {
        let conf = r#"
# ---------------------------------------------------------------------------
# Core
# ---------------------------------------------------------------------------
general {
    border_size = 2
}
bind = SUPER, D, exec, wofi
"#;
        let binds = parse_binds(conf);
        assert_eq!(binds.len(), 1);
        // ruler comments skipped; "Core" is 4 lines above, not adjacent
        assert_eq!(binds[0].desc, "exec wofi");
    }
}
