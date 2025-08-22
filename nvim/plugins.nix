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
      settings = {
        defaults = {
          vimgrep_arguments = [
            "rg"
            "--color=never"
            "--no-heading"
            "--with-filename"
            "--line-number"
            "--column"
            "--smart-case"
          ];
          file_ignore_patterns = [ "%.git/" "node_modules/" ];
        };
        pickers = {
          find_files = {
            find_command = [ "fd" "--type" "f" "--strip-cwd-prefix" ];
          };
        };
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
    web-devicons.enable = true; # Explicitly enable to avoid deprecation warning
    
    # Text manipulation
    vim-surround.enable = true; # vim-surround functionality
    comment.enable = true; # Smart commenting with gcc/gbc
    
    # Additional useful plugins
    indent-blankline.enable = true; # Show indentation guides
    nvim-autopairs.enable = true; # Auto close brackets/quotes
    leap.enable = true; # Fast motion plugin (like easymotion)
    
    # Code folding
    nvim-ufo = {
      enable = true;
      settings = {
        provider_selector = ''
          function(bufnr, filetype, buftype)
            return {'treesitter', 'indent'}
          end
        '';
      };
    };
    
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

    # Snacks (LazyVim's utility plugin with file explorer)
    snacks = {
      enable = true;
      settings = {
        bigfile = { enabled = true; };
        notifier = { enabled = true; };
        quickfile = { enabled = true; };
        statuscolumn = { enabled = true; };
        words = { enabled = true; };
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
  # (rustaceanvim moved to rust.nix)
}