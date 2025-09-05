{ pkgs, ... }:

{
  home.packages = with pkgs; [
    # Image editing
    aseprite
    
    # 3D modeling & game development
    blockbench
  ];
}