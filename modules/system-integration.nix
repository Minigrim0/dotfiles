{ pkgs, ... }:

{
  home.packages = with pkgs; [
    # Network management
    networkmanagerapplet
    
    # Authentication & security
    polkit_gnome
    seahorse # GUI for gnome secrets
    clamav # Antivirus
    
    # Bluetooth management
    blueman
    
    # System notifications
    libnotify # provides notify-send
    
    # Media thumbnails
    xfce.tumbler
    ffmpegthumbnailer
    
    # Keyboard layout
    qwerty-fr
  ];
}