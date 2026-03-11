# Plugins configuration ported from your LazyVim setup
{ pkgs, lib, ... }:

{
  programs.nixvim = {
    
    plugins = {
      # LSP configuration with enhanced diagnostics
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

          # Clangd for C/C++
          clangd = {
            enable = true;
            settings = {
              clangd = {
                fallbackFlags = [ "-std=c++20" ];
              };
            };
          };

          # Python LSP
          ruff.enable = true;
          
          # Rust LSP (will be handled by rustaceanvim)
          rust_analyzer.enable = false;
        };
        
        # Enhanced diagnostic configuration for inline errors
        # inlayHints.enable = true;  # Handled by rustaceanvim
        keymaps = {
          diagnostic = {
            "[d" = "goto_prev";
            "]d" = "goto_next";
            "<leader>d" = "open_float";  # Changed from <leader>e to avoid conflict
            "<leader>q" = "setloclist";
          };
          lspBuf = {
            "gd" = "definition";
            "gr" = "references"; 
            "gi" = "implementation";
            "gt" = "type_definition";
            "K" = "hover";
            "<leader>ca" = "code_action";
            "<leader>rn" = "rename";
          };
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
          on_open.__raw = ''
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
      
      # React & JavaScript/TypeScript development  
      ts-autotag.enable = true; # Auto-close JSX tags
      ts-context-commentstring.enable = true; # Smart JSX commenting
      trouble.enable = true; # Better diagnostics panel
      
      # GitHub Copilot
      copilot-lua = {
        enable = false; # AI code completion
        settings = {
          suggestion = {
            enabled = false;
          };
        };
      };

      # Code folding
      nvim-ufo = {
        enable = true;
        settings = {
          provider_selector.__raw = ''
            function(bufnr, filetype, buftype)
              return {'treesitter', 'indent'}
            end
          '';
          open_fold_hl_timeout = 150;
          close_fold_kinds_for_ft = {
            default = {};  # Don't auto-close any folds by default
          };
        };
      };
      
      # Enhanced completion system with nvim-cmp
      cmp = {
        enable = true;
        settings = {
          snippet = {
            expand = ''
              function(args)
                require('luasnip').lsp_expand(args.body)
              end
            '';
          };
          mapping = {
            "<Tab>".__raw = "cmp.mapping(cmp.mapping.select_next_item({ behavior = cmp.SelectBehavior.Insert }), {'i', 's'})";
            "<S-Tab>".__raw = "cmp.mapping(cmp.mapping.select_prev_item({ behavior = cmp.SelectBehavior.Insert }), {'i', 's'})";
            "<CR>".__raw = "cmp.mapping.confirm({ select = false })";
            "<C-Space>".__raw = "cmp.mapping.complete()";
            "<C-e>".__raw = "cmp.mapping.abort()";
            "<C-d>".__raw = "cmp.mapping.scroll_docs(4)";
            "<C-u>".__raw = "cmp.mapping.scroll_docs(-4)";
          };
          sources = [
            { name = "nvim_lsp"; priority = 1000; }
            { name = "luasnip"; priority = 750; keyword_length = 2; }
            { name = "copilot"; priority = 700; }
            { name = "buffer"; priority = 500; keyword_length = 3; }
            { name = "path"; priority = 300; }
            { name = "crates"; priority = 400; } # For Rust crates
          ];
          window = {
            completion.__raw = "cmp.config.window.bordered()";
            documentation.__raw = "cmp.config.window.bordered()";
          };
          formatting = {
            fields = [ "kind" "abbr" "menu" ];
            format = ''
              function(entry, vim_item)
                local kind_icons = {
                  Text = "󰉿",
                  Method = "󰆧",
                  Function = "󰊕",
                  Constructor = "",
                  Field = "󰜢",
                  Variable = "󰀫",
                  Class = "󰠱",
                  Interface = "",
                  Module = "",
                  Property = "󰜢",
                  Unit = "󰑭",
                  Value = "󰎠",
                  Enum = "",
                  Keyword = "󰌋",
                  Snippet = "",
                  Color = "󰏘",
                  File = "󰈙",
                  Reference = "󰈇",
                  Folder = "󰉋",
                  EnumMember = "",
                  Constant = "󰏿",
                  Struct = "󰙅",
                  Event = "",
                  Operator = "󰆕",
                  TypeParameter = ""
                }
                vim_item.kind = string.format('%s %s', kind_icons[vim_item.kind], vim_item.kind)
                vim_item.menu = ({
                  nvim_lsp = "[LSP]",
                  luasnip = "[Snippet]",
                  copilot = "[Copilot]",
                  buffer = "[Buffer]",
                  path = "[Path]",
                  crates = "[Crates]",
                })[entry.source.name]
                return vim_item
              end
            '';
          };
        };
      };

      # Completion sources
      cmp-nvim-lsp.enable = true;
      cmp_luasnip.enable = true;
      cmp-buffer.enable = true;
      cmp-path.enable = true;

      # Snippet engine
      luasnip = {
        enable = true;
        settings = {
          enable_autosnippets = true;
          store_selection_keys = "<Tab>";
        };
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
          explorer = { enabled = true; };  # Enable file explorer
          image = { enabled = true; };
          input = { enabled = true; };
          picker = { enabled = true; };
          scroll = { enabled = true; };
          dim = {
            scope = {
              min_size = 5;
              max_size = 20;
              siblings = true;
            };
            animate = {
              enabled.__raw = "vim.fn.has('nvim-0.10') == 1";
              easing = "outQuad";
              duration = {
                step = 20;
                total = 300;
              };
            };
            filter.__raw = ''
              function(buf)
                return vim.g.snacks_dim ~= false and vim.b[buf].snacks_dim ~= false and vim.bo[buf].buftype == ""
              end
            '';
          };
          indent = {
            animate = {
              enabled.__raw = "vim.fn.has(\"nvim-0.10\") == 1";
              style = "out";
              easing = "linear"; duration = {
                step = 20;
                total = 500;
              };
            };
            scope = {
              enabled = true;
              priority = 200;
              char = "|";
              underline = false;
              only_current = false;
            };
          };
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

      typescript-tools = {
        enable = true;
        settings = {
          tsx_close_tag = {
            enable = true;
            filetypes = [ "typescriptreact" "javascriptreact" ];
          };
        };
      };
    };

    # Extra plugins for React/Electron development and completion
    extraPlugins = with pkgs.vimPlugins; [
      # Package.json management
      package-info-nvim
      # VSCode-style snippets
      friendly-snippets
      (pkgs.vimUtils.buildVimPlugin {
        name = "pdf-preview";
        src = pkgs.fetchFromGitHub {
          owner = "franco-ruggeri";
          repo = "pdf-preview.nvim";
          rev = "v0.3.3";
          hash = "sha256-t8zJL9MYjUTsNdKlGcPrweR6l/hBKZ7We9NGYEmevL8=";
        };
       })
    ];

    extraConfigLua = /* lua */ '' 
      -- Enhanced diagnostic configuration
      vim.diagnostic.config({
        virtual_text = {
          spacing = 4,
          prefix = '●',
          source = 'if_many',
        },
        float = {
          focusable = false,
          close_events = { "BufLeave", "CursorMoved", "InsertEnter", "FocusLost" },
          border = 'rounded',
          source = 'always',
          prefix = "",
          scope = 'cursor',
        },
        signs = true,
        underline = true,
        update_in_insert = false,
        severity_sort = true,
      })
      
      -- Auto-show diagnostic on cursor hold
      vim.api.nvim_create_autocmd({ "CursorHold" }, {
        pattern = "*",
        callback = function()
          vim.diagnostic.open_float(nil, { focus = false })
        end,
      })
      
      -- Configure React/JS plugins
      
      -- TypeScript Tools (enhanced TypeScript support)
      local has_typescript_tools, typescript_tools = pcall(require, 'typescript-tools')
      if has_typescript_tools then
        typescript_tools.setup({
          on_attach = function(client, bufnr)
            -- Disable tsserver formatting (use prettier instead)
            client.server_capabilities.documentFormattingProvider = false
            client.server_capabilities.documentRangeFormattingProvider = false
          end,
          settings = {
            tsx_close_tag = {
              enable = true,
              filetypes = { "typescriptreact", "javascriptreact" },
            },
          },
        })
      end
      
      -- Package Info (package.json management)
      local has_package_info, package_info = pcall(require, 'package-info')
      if has_package_info then
        package_info.setup({
          colors = {
            up_to_date = "#3C4048",
            outdated = "#fc7b7b", 
          },
          icons = {
            enable = true,
            style = {
              up_to_date = "|  ",
              outdated = "|  ",
            },
          },
          autostart = true,
          hide_up_to_date = false,
          hide_unstable_versions = false,
        })
      end

      -- PDF Preview
      require('pdf-preview').setup({})

      -- Configure LuaSnip for enhanced completion
      local luasnip = require('luasnip')

      -- Load VSCode-style snippets from friendly-snippets
      require('luasnip.loaders.from_vscode').lazy_load()

      -- Enhanced snippet expansion and jumping
      luasnip.config.setup({
        history = true,
        updateevents = "TextChanged,TextChangedI",
        enable_autosnippets = true,
        ext_opts = {
          [require("luasnip.util.types").choiceNode] = {
            active = {
              virt_text = { { "●", "GruvboxOrange" } },
            },
          },
        },
      })

      -- Set completeopt for better completion experience
      vim.opt.completeopt = {'menu', 'menuone', 'noselect'}

      -- Auto-pairs integration with nvim-cmp
      local cmp_autopairs = require('nvim-autopairs.completion.cmp')
      local cmp = require('cmp')
      cmp.event:on('confirm_done', cmp_autopairs.on_confirm_done())
    '';
  };
}
