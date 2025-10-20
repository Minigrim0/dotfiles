{ pkgs, ... }:

{
  home.packages = with pkgs; [
    # Programming languages & runtimes
    rustup # Rust
    nodejs_24 # Node
    python3
    jdk21 # Java
    
    # IDEs & editors
    vscode
    jetbrains.idea-ultimate
    
    # Creative/design tools
    gimp3
    freecad

    # Game development
    godot
    
    # AI/ML development
    lmstudio
    
    # Document preparation
    texliveFull # LaTeX
    
    # Image processing
    imagemagick
  ];
}
