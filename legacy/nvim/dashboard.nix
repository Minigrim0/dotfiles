# Enhanced dashboard configuration ported from your dashboard.lua
{
  programs.nixvim.extraConfigLua = ''
    -- Enhanced dashboard configuration to match your LazyVim setup
    require('dashboard').setup({
      theme = 'hyper',
      config = {
        week_header = {
          enable = true,
        },
        shortcut = {
          {
            desc = ' Find Files',
            group = 'Label', 
            action = 'Telescope find_files',
            key = 'f',
          },
          {
            desc = ' Recent Files',
            group = 'Number',
            action = 'Telescope oldfiles',
            key = 'r', 
          },
          {
            desc = ' Find Text',
            group = 'DiagnosticHint',
            action = 'Telescope live_grep',
            key = 'g',
          },
          {
            desc = ' Restore Session', 
            group = 'String',
            action = function()
              require('auto-session').RestoreSession()
            end,
            key = 's',
          },
          {
            desc = ' Terminal',
            group = 'Function', 
            action = 'ToggleTerm direction=float',
            key = 't',
          },
          {
            desc = ' Config',
            group = 'Constant',
            action = function()
              vim.cmd('edit ~/.config/home-manager')
            end,
            key = 'c',
          },
          {
            desc = ' Git Status',
            group = 'Special',
            action = 'LazyGit', 
            key = 'G',
          },
          {
            desc = ' Quit',
            group = 'Error',
            action = 'qa',
            key = 'q',
          },
        },
        footer = function()
          return {
            '⚡ NixVim loaded with Nix packages - No more Mason!',
            '🎨 gruvbox colorscheme', 
            '📁 ' .. vim.fn.getcwd(),
          }
        end,
      },
    })
  '';
}