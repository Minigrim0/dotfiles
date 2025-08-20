{ pkgs, lib, ... }:

{
  # Install neovim and essential tools
  programs.neovim = {
    enable = true;
    defaultEditor = true;
    viAlias = true;
    vimAlias = true;
    vimdiffAlias = true;
  };

  # Essential packages for LazyVim to work properly
  home.packages = with pkgs; [
    # Language servers
    lua-language-server
    # rust-analyzer (provided by rustup in development.nix)
    nodePackages.typescript-language-server
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
    
    # Git (required by Lazy)
    git
    
    # Clipboard support
    wl-clipboard
    xclip
    
    # Optional: ripgrep, fd for better search
    ripgrep
    fd
  ];

  # Symlink LazyVim configuration
  home.file = {
    ".config/nvim" = {
      source = ./config;
      recursive = true;
    };
  };
}
