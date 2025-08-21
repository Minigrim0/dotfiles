# Plugins configuration ported from your LazyVim setup
{ pkgs, lib, ... }:

{
  programs.nixvim.plugins = {
    # LSP configuration
    lsp = {
      enable = true;
      servers = {
        # Lua LSP (using system lua-language-server)
        lua_ls = {
          enable = true;
          settings = {
            Lua = {
              workspace = {
                checkThirdParty = false;
              };
              completion = {
                callSnippet = "Replace";
              };
              telemetry = { enable = false; };
            };
          };
        };
        
        # Python LSP
        pyright.enable = true;
        
        # Rust LSP (will be handled by rustaceanvim)
        rust_analyzer.enable = false;
      };
    };

    # Formatting with conform (from your formatting.lua)
    conform-nvim = {
      enable = true;
      settings = {
        formatters_by_ft = {
          lua = [ "stylua" ];
          python = [ "black" ];
        };
        formatters = {
          stylua = {
            command = "stylua";
          };
        };
      };
    };

    # Terminal (from your toggleterm.lua)
    toggleterm = {
      enable = true;
      settings = {
        hidden = true;
        start_in_insert = true;
        insert_mappings = true;
        terminal_mappings = true;
        direction = "float";
        on_open = ''
          function(term)
            vim.cmd("startinsert!")
          end
        '';
      };
    };

    # File explorer and navigation (LazyVim defaults)
    telescope = {
      enable = true;
      keymaps = {
        "<leader>ff" = "find_files";
        "<leader>fg" = "live_grep";
        "<leader>fb" = "buffers";
        "<leader>fh" = "help_tags";
        "<leader>fr" = "oldfiles";
      };
    };

    # Git integration
    lazygit.enable = true;
    gitsigns.enable = true;

    # Session management (from your auto-session.lua)
    auto-session = {
      enable = true;
      settings = {
        suppressed_dirs = [ "~/" "~/Projects" "~/Downloads" "/" ];
      };
    };

    # UI enhancements
    lualine.enable = true;
    bufferline.enable = true;
    which-key.enable = true;
    
    # Completion
    cmp = {
      enable = true;
      autoEnableSources = true;
    };

    # Syntax highlighting
    treesitter = {
      enable = true;
      nixGrammars = true;
    };

    # File explorer
    neo-tree = {
      enable = true;
      closeIfLastWindow = true;
      window = {
        width = 30;
        autoExpandWidth = false;
      };
    };

    # Dashboard (basic config, enhanced in extraConfigLua)
    dashboard = {
      enable = true;
      settings = {
        theme = "hyper";
        config = {
          week_header = {
            enable = true;
          };
        };
      };
    };
  };

  # Extra plugins not directly supported by NixVim
  programs.nixvim.extraPlugins = with pkgs.vimPlugins; [
    rustaceanvim
  ];
}