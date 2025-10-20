{ pkgs, ... }:

{
  home.packages = with pkgs; [
    # Core user applications
    obsidian
    firefox # Browser
    thunderbird
    nextcloud-client # Cloud
    
    # Media & communication
    vlc
    signal-desktop
    slack
    discord
    spotify
    
    # File management
    nautilus

    jabref

    # Office suite
    libreoffice-qt6
  ];
}
