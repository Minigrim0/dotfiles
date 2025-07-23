{ pkgs, ... }:

{
  home.packages = with pkgs; [
    firefox # Browser
    alacritty # Terminal
    wofi # Application Launcher
    neovim # TUI Editor
    htop # System monitor
    nerd-fonts.anonymice # Font
    networkmanagerapplet # Network Applet
    nextcloud-client #  Cloud
    eww # Widget/bar
    dunst # Notifications
    swww # Animated wallpaper
    brightnessctl
    playerctl
    zoxide
    libnotify # provides notify-send
    thunderbird
    mate.caja
    vlc
    signal-desktop
    slack
    discord
    stacer
    clamav
    seahorse # GUI for gnome secrets
    polkit_gnome # GUI auth agent
    blueman # GUI bluetooth manager
    xfce.tumbler
    ffmpegthumbnailer
  ];
}
