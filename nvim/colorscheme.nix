{ pkgs, ... }:

{
  programs.neovim = {
    plugins = with pkgs.vimPlugins; [
      gruvbox-material
    ];
    extraConfig = /* lua */ ''
      vim.o.termguicolors = true
      vim.cmd('colorscheme gruvbox-material')
      vim.g.gruvbox_material_background = 'hard'
    '';
  };
}
