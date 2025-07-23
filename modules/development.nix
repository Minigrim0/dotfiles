{ pkgs, ... }:

{
  home.packages = with pkgs; [
    rustup # Rust
    nodejs_24 # Node
    gimp3
    godot
    python3
    python3Packages.python-lsp-server
    nixd # Add nixd
    nixpkgs-fmt # Nix formatter
    nil # Alternative Nix language server
    texliveFull # TexLive
  ];
}
