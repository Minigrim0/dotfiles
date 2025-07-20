{ config, pkgs, ... }:

{
  # Install fonts
  home.packages = with pkgs; [
    # Nerd Fonts
    nerd-fonts.anonymice
    nerd-fonts.fira-code
    nerd-fonts.jetbrains-mono
    
    # System fonts
    font-awesome
    liberation_ttf
    noto-fonts
    noto-fonts-emoji
    source-code-pro
    
    # Additional monospace fonts
    fira-code
    fira-code-symbols
    jetbrains-mono
  ];
  
  # Font configuration
  fonts.fontconfig = {
    enable = true;
    defaultFonts = {
      serif = [ "Liberation Serif" "Noto Serif" ];
      sansSerif = [ "Liberation Sans" "Noto Sans" ];
      monospace = [ "AnonymicePro Nerd Font Mono" "Source Code Pro" ];
      emoji = [ "Noto Color Emoji" ];
    };
  };
}