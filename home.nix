{ config, pkgs, lib, ... }:

{
  home.username = "minigrim0";
  home.homeDirectory = "/home/minigrim0";

  nixpkgs.config.allowUnfreePredicate = pkg: builtins.elem (pkgs.lib.getName pkg) [
    "obsidian"
    "discord"
    "slack"
    "spotify"
    "vscode"
    "cursor"
    "aseprite"
    "lmstudio"
    "idea-ultimate"
    "tailscale" 
  ];

  imports = [
    ./modules/packages/cli-tools.nix
    ./modules/packages/creative.nix
    ./modules/packages/development.nix
    ./modules/dunst.nix
    ./modules/eww.nix
    ./modules/fonts.nix
    ./modules/hyprland.nix
    ./modules/hyprlock.nix
    ./modules/kitty.nix
    ./nvim/nixvim.nix
    ./modules/neovim-extras.nix
    ./modules/packages/packages.nix
    ./modules/shell.nix
    ./modules/swww.nix
    ./modules/packages/system-integration.nix
    ./modules/theme.nix
    ./modules/wofi.nix
    ./modules/zeditor.nix
  ];

  fonts.fontconfig.enable = true;

  # Home Manager is pretty good at managing dotfiles. The primary way to manage
  # plain files is through 'home.file'.
  home.file = {
    # # Building this configuration will create a copy of 'dotfiles/screenrc' in
    # # the Nix store. Activating the configuration will then make '~/.screenrc' a
    # # symlink to the Nix store copy.
    # ".screenrc".source = dotfiles/screenrc;

    # # You can also set the file content immediately.
    # ".gradle/gradle.properties".text = ''
    #   org.gradle.console=verbose
    #   org.gradle.daemon.idletimeout=3600000
    # '';
  };

  # Home Manager can also manage your environment variables through
  # 'home.sessionVariables'. These will be explicitly sourced when using a
  # shell provided by Home Manager. If you don't want to manage your shell
  # through Home Manager then you have to manually source 'hm-session-vars.sh'
  # located at either
  #
  #  ~/.nix-profile/etc/profile.d/hm-session-vars.sh
  #
  # or
  #
  #  ~/.local/state/nix/profiles/profile/etc/profile.d/hm-session-vars.sh
  #
  # or
  #
  #  /etc/profiles/per-user/minigrim0/etc/profile.d/hm-session-vars.sh
  #
  programs.zsh.enable = true;

  home.stateVersion = "25.05"; # Please read the comment before changing.
  programs.home-manager.enable = true;
}
