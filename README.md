# Home Manager Configuration

Personal NixOS Home Manager setup.

## Installation

1. Install Home Manager:
```bash
nix-channel --add https://github.com/nix-community/home-manager/archive/release-25.05.tar.gz home-manager
nix-channel --update
nix-shell '<home-manager>' -A install
```

2. Clone and setup:
```bash
git clone <repo-url> ~/.config/home-manager
cd ~/.config/home-manager
```

3. Create personal git config:
```bash
cp .git-config.nix.example .git-config.nix
```

4. Edit `.git-config.nix`:
```nix
{ config, pkgs, ... }:

{
  programs.git = {
    enable = true;
    userName = "Your Name";
    userEmail = "your.email@domain.com";
    extraConfig = {
      push = { autoSetupRemote = true; };
      init = { defaultBranch = "main"; };
    };
  };
}
```

5. Update username in `home.nix`:
```nix
home.username = "your-username";
home.homeDirectory = "/home/your-username";
```

6. Apply:
```bash
home-manager switch
```