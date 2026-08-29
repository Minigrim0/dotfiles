# Packaging

Two PKGBUILDs for the AUR:

- `dots-bin/` — repackages the release binary that CI attaches to each `v*`
  tag. The fast install path for users.
- `dots-git/` — builds from the latest `main` with cargo.

## Publishing to the AUR

One-time: create an AUR account and add an SSH key, then for each package:

```sh
git clone ssh://aur@aur.archlinux.org/dots-bin.git aur-dots-bin
cp dots-bin/PKGBUILD aur-dots-bin/
cd aur-dots-bin
# For dots-bin: replace sha256sums=('SKIP') with the real checksum:
#   sha256sum dots-linux-x86_64
makepkg --printsrcinfo > .SRCINFO
git add PKGBUILD .SRCINFO
git commit -m "dots-bin 0.1.0"
git push
```

Test locally first with `makepkg -si` in the package directory.

## Release flow

1. Bump the version in `dots/Cargo.toml`, commit.
2. `git tag v<version> && git push --tags` — CI builds and attaches the binary.
3. Update `pkgver` + `sha256sums` in `dots-bin/PKGBUILD`, push to the AUR.
   (`dots-git` needs no update; its `pkgver()` tracks HEAD.)

## Before first publish

The repo has no LICENSE file yet — the AUR packages declare `license=('custom')`.
Add one (MIT is typical for dotfiles) and switch both PKGBUILDs to it.
