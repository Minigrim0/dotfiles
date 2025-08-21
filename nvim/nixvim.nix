{ pkgs, lib, ... }:

{
  imports = [
    ./keymaps.nix
    ./options.nix
    ./colorscheme.nix
    ./plugins.nix
    ./dashboard.nix
    ./rust.nix
  ];

  programs.nixvim = {
    enable = true;
    defaultEditor = true;
    viAlias = true;
    vimAlias = true;
    vimdiffAlias = true;
  };
}