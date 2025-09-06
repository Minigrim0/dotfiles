{ pkgs, ... }:

{
  home.packages = with pkgs; [
    # System monitoring & management
    htop
    stacer
    
    # Media controls
    brightnessctl
    playerctl
    
    # Navigation & file management  
    zoxide
    swayimg
    bat
    lsd

    # Task management
    taskwarrior3
    taskwarrior-tui
    
    # Version control
    lazygit
    
    # Clipboard management
    wl-clipboard
    xclip
  ];
}
