{ pkgs, lib, ... }:

let
  fromGithub = import ../functions/fromGithub.nix;
in

{
  imports = [
    ./colorscheme.nix
  ];

  programs.neovim = {
    enable = true;
    defaultEditor = true;
    viAlias = true;
    vimAlias = true;
    vimdiffAlias = true;

    plugins = with pkgs.vimPlugins; [
      nvim-lspconfig
      (nvim-treesitter.withPlugins (p: [p.c p.rust p.python p.lua p.nix]))
      plenary-nvim
      mini-nvim
      (fromGithub {user = "elihunter173"; repo = "dirbuf.nvim";})
    ];
  };
}
