{ pkgs, ... }:

{
  home.packages = with pkgs; [
    # Language servers
    lua-language-server
    pyright
    nodePackages.typescript-language-server
    
    # Formatters
    stylua
    nodePackages.prettier
    black
    
    # Build tools
    gcc
    cmake
    pkg-config
    
    # Debugging tools
    lldb
    
    # Search tools
    ripgrep
    fd
    
    # Node.js/Electron development (nodejs_24 and npm already in development.nix)
  ];
}