# Rust configuration with proper NixVim rustaceanvim
{ pkgs, lib, ... }:

{
  programs.nixvim = {
    plugins = {
      # Main Rust plugin - replaces rust-tools.nvim
      rustaceanvim = {
        enable = true;
        settings = {
          server = {
            # Use system rust-analyzer from rustup
            standalone = false;
            default_settings = {
              rust-analyzer = {
                check = {
                  command = "clippy";
                };
                inlayHints = {
                  lifetimeElisionHints = {
                    enable = "always";
                  };
                };
                cargo = {
                  allFeatures = true;
                };
              };
            };
          };
          tools = {
            hover_actions = {
              replace_builtin_hover = true;
            };
          };
        };
      };
    };

    # Extra Rust-specific plugins
    extraPlugins = with pkgs.vimPlugins; [
      # Cargo.toml support
      crates-nvim
      
      # TODO: Add tree_climber_rust.nvim when hash is available
      # (pkgs.vimUtils.buildVimPlugin {
      #   name = "tree_climber_rust.nvim";
      #   src = pkgs.fetchFromGitHub {
      #     owner = "adaszko";
      #     repo = "tree_climber_rust.nvim";
      #     rev = "HEAD";
      #     sha256 = lib.fakeHash; # Replace with actual hash
      #   };
      # })
    ];

    extraConfigLua = ''
      -- Configure crates.nvim
      require('crates').setup({
        completion = {
          cmp = { enabled = true }
        },
        lsp = {
          enabled = true,
          actions = true,
          completion = true,
          hover = true,
        },
      })
      
      -- Configure tree_climber_rust.nvim
      -- Note: Check if plugin loaded before setup
      local has_tree_climber, tree_climber = pcall(require, 'tree_climber_rust')
      if has_tree_climber then
        tree_climber.setup()
        
        -- Key mappings for tree climbing
        vim.keymap.set('n', '<C-n>', tree_climber.goto_next, { desc = 'Tree Climber: Goto Next' })
        vim.keymap.set('n', '<C-p>', tree_climber.goto_prev, { desc = 'Tree Climber: Goto Previous' })
        vim.keymap.set('v', '<C-n>', tree_climber.select_next, { desc = 'Tree Climber: Select Next' })
        vim.keymap.set('v', '<C-p>', tree_climber.select_prev, { desc = 'Tree Climber: Select Previous' })
      end
    '';
  };
}
