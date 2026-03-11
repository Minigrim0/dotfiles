{ pkgs, lib, ... }:

{
  # Import the modular NixVim configuration
  imports = [
    ./nixvim.nix
  ];

  # Essential packages for NixVim to work properly
  home.packages = with pkgs; [
    # Language servers
    lua-language-server
    # rust-analyzer (provided by rustup in development.nix)
    pyright
    
    # Formatters
    stylua
    nodePackages.prettier
    black
    
    # Build tools
    gcc
    cmake
    pkg-config
    
    # Debugging tools  
    lldb # Required for Rust debugging
    
    # Clipboard support
    wl-clipboard
    xclip
    
    # Search tools
    ripgrep
    fd
    
    # Git tools
    lazygit
  ];
}
