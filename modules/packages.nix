{ config, pkgs, ... }:

{
  home.packages = with pkgs; [
    firefox
    chezmoi
    alacritty
    wofi
    neovim
    htop
    rustup
    nerd-fonts.anonymice
    zed-editor
    nodejs_24
    networkmanagerapplet
    nextcloud-client
    eww
    dunst
    swww
    hyprlock
    brightnessctl
    playerctl
    zoxide
    libnotify  # provides notify-send
    thunderbird
    mate.caja
    gimp3
    vlc
    signal-desktop
    slack
    discord
    stacer
    clamav
    texliveFull
    godot
  ];
}
