use crate::arrow;
use crate::config::Module;
use anyhow::{Result, bail};

/// Use paru's library entrypoint for package operations.
/// `paru::run` returns an exit code (0 == success).

pub async fn install_packages(module: &Module) -> Result<()> {
    // Use paru for both official repo and AUR packages. paru understands
    // both and will forward to pacman/libalpm when appropriate.
    if !module.packages.is_empty() {
        arrow!(
            "Installing pacman packages via paru: {}",
            module.packages.join(" ")
        );
        let mut args = vec!["-S", "--needed", "--noconfirm"];
        args.extend(module.packages.iter().map(|s| s.as_str()));
        let code = paru::run(&args).await;
        if code != 0 {
            bail!("paru install failed (code {})", code);
        }
    }

    if !module.aur_packages.is_empty() {
        arrow!(
            "Installing AUR packages via paru: {}",
            module.aur_packages.join(" ")
        );
        let mut args = vec!["-S", "--needed", "--noconfirm"];
        args.extend(module.aur_packages.iter().map(|s| s.as_str()));
        let code = paru::run(&args).await;
        if code != 0 {
            bail!("paru AUR install failed (code {})", code);
        }
    }

    Ok(())
}

pub async fn install_extra(pkgs: &[String]) -> Result<()> {
    if pkgs.is_empty() {
        return Ok(());
    }
    arrow!("Installing extra packages via paru: {}", pkgs.join(" "));
    let mut args = vec!["-S", "--needed", "--noconfirm"];
    args.extend(pkgs.iter().map(|s| s.as_str()));
    let code = paru::run(&args).await;
    if code != 0 {
        bail!("paru install failed for extra packages (code {})", code);
    }
    Ok(())
}
