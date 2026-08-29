use crate::config::{
    self, DotsConfig, canonical_repo_dir, dotfiles_dir, load_config, load_manifest, save_config,
};
use crate::linker;
use crate::{arrow, head, ok, warn};
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Machine names available in the repo (stems of machines/*.toml).
fn list_machines(dotfiles: &Path) -> Vec<String> {
    let dir = dotfiles.join("machines");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("toml"))
        .filter_map(|e| {
            e.path()
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        })
        .collect();
    names.sort();
    names
}

/// Match the current hostname against machines/*.toml `meta.hostname`.
fn detect_machine(dotfiles: &Path) -> Option<String> {
    let hostname = Command::new("hostnamectl")
        .arg("hostname")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())?;
    if hostname.is_empty() {
        return None;
    }

    for name in list_machines(dotfiles) {
        if let Ok(mc) = config::load_machine(dotfiles, &name)
            && mc.meta.hostname == hostname
        {
            arrow!("Machine '{}' matched hostname '{}'", name, hostname);
            return Some(name);
        }
    }
    None
}

/// Ask the user to pick a machine from the available profiles.
fn prompt_machine(available: &[String]) -> Result<String> {
    anyhow::ensure!(!available.is_empty(), "no machines/*.toml in the repo");
    arrow!("Which machine is this? Available: {}", available.join(", "));
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .context("reading machine name")?;
    let name = input.trim().to_string();
    anyhow::ensure!(
        available.contains(&name),
        "'{}' is not one of: {}",
        name,
        available.join(", ")
    );
    Ok(name)
}

/// arg > config.toml > hostname match > interactive prompt.
fn resolve_machine(dotfiles: &Path, arg: Option<&str>) -> Result<String> {
    if let Some(name) = arg {
        return Ok(name.to_string());
    }
    if let Some(cfg) = load_config()
        && let Some(name) = cfg.machine
        && !name.is_empty()
    {
        return Ok(name);
    }
    if let Some(name) = detect_machine(dotfiles) {
        return Ok(name);
    }
    prompt_machine(&list_machines(dotfiles))
}

/// Sync every enabled module and the machine-specific symlinks.
fn full_sync(dotfiles: &Path, machine: &str) -> Result<()> {
    let manifest = load_manifest(dotfiles)?;
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("no $HOME"))?;

    let mut names: Vec<&String> = manifest.modules.keys().collect();
    names.sort();
    for name in names {
        let module = &manifest.modules[name];
        if module.enabled {
            linker::sync_module(name, module, dotfiles, &home)?;
        }
    }

    match config::load_machine(dotfiles, machine) {
        Ok(mc) => {
            head!("Applying machine symlinks for '{}'", machine);
            linker::apply_machine_symlinks(dotfiles, &mc, &home)?;
        }
        Err(e) => warn!("machine config '{}': {:#}", machine, e),
    }
    Ok(())
}

/// Clone a dotfiles repo into the canonical location and set everything up.
pub fn init(url: &str, machine: Option<&str>) -> Result<()> {
    let repo = canonical_repo_dir();
    anyhow::ensure!(
        !repo.exists(),
        "{} already exists — already initialized? (`dots sync` to relink)",
        repo.display()
    );
    if let Some(parent) = repo.parent() {
        std::fs::create_dir_all(parent)?;
    }

    head!("Cloning {}", url);
    let status = Command::new("git")
        .args(["clone", url])
        .arg(&repo)
        .status()
        .context("running git clone")?;
    anyhow::ensure!(status.success(), "git clone failed");

    let machine = resolve_machine(&repo, machine)?;
    save_config(&DotsConfig {
        dotfiles_dir: repo.to_string_lossy().to_string(),
        machine: Some(machine.clone()),
    })?;
    ok!("Wrote ~/.config/dots/config.toml (machine: {})", machine);

    full_sync(&repo, &machine)?;

    ok!("Initialized. Next: dots install --machine {}", machine);
    Ok(())
}

/// Move an existing checkout to the canonical location and relink $HOME.
pub fn migrate(machine: Option<&str>) -> Result<()> {
    let current = dotfiles_dir()?;
    anyhow::ensure!(
        current.join("modules.toml").exists(),
        "{} does not look like a dotfiles repo (no modules.toml)",
        current.display()
    );

    let repo = canonical_repo_dir();
    let already_there = repo.exists() && current.canonicalize().ok() == repo.canonicalize().ok();
    if already_there {
        ok!("Repo already lives at {}", repo.display());
    } else {
        anyhow::ensure!(
            !repo.exists(),
            "{} already exists — remove it first",
            repo.display()
        );
        if let Some(parent) = repo.parent() {
            std::fs::create_dir_all(parent)?;
        }

        head!("Moving {} → {}", current.display(), repo.display());
        std::fs::rename(&current, &repo).map_err(|e| {
            if e.kind() == std::io::ErrorKind::CrossesDevices {
                anyhow::anyhow!(
                    "{} and {} are on different filesystems — move the repo manually, \
                     then run `dots migrate` again from inside it",
                    current.display(),
                    repo.display()
                )
            } else {
                anyhow::Error::from(e).context("moving the repo")
            }
        })?;
    }

    let machine = resolve_machine(&repo, machine)?;
    save_config(&DotsConfig {
        dotfiles_dir: repo.to_string_lossy().to_string(),
        machine: Some(machine.clone()),
    })?;
    ok!("Wrote ~/.config/dots/config.toml (machine: {})", machine);

    // Relink everything: linker::link rewrites symlinks that point at the old path.
    full_sync(&repo, &machine)?;

    ok!("Migration complete — repo now at {}", repo.display());
    if !already_there {
        warn!(
            "Shells still sitting in {} are looking at a path that no longer exists",
            current.display()
        );
    }
    Ok(())
}
