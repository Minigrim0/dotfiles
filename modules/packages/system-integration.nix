{ pkgs, ... }:

{
  home.packages = with pkgs; [
    # Network management
    networkmanagerapplet
    impala # TUI wifi manager

    # Authentication & security
    polkit_gnome
    seahorse # GUI for gnome secrets
    clamav # Antivirus

    tailscale
    trayscale

    # Bluetooth management
    blueman
    bluetui # TUI bluetooth manager

    # Screenshots
    hyprshot # Wayland screenshot tool

    # System notifications
    libnotify # provides notify-send

    # Media thumbnails
    xfce.tumbler
    ffmpegthumbnailer

    # Keyboard layout
    qwerty-fr
  ];
}
