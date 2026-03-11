{ pkgs, ... }:

{
  home.packages = with pkgs; [
    # Language servers
    lua-language-server
    pyright
    nodePackages.typescript-language-server
    python3Packages.python-lsp-server # Python LSP
    nixd # Nix language server
    nil # Alternative Nix language server
    clang-tools # Provides clangd for C/C++
    
    # Formatters
    stylua
    nodePackages.prettier
    black
    nixpkgs-fmt # Nix formatter
    
    # Build tools
    gcc
    cmake
    pkg-config
    
    # Debugging tools
    lldb
    
    # Search tools
    ripgrep
    fd
    
    # Document processing
    ghostscript
    mermaid-cli
  ];
}
