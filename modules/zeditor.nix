{ pkgs, lib, ... }:

{
  home.packages = with pkgs; [
    zed-editor
  ];

  programs.zed-editor = {
    enable = true;

    ## This populates the userSettings "auto_install_extensions"
    extensions = [ "nix" "toml" "tex" "make" ];

    userSettings = {
      assistant = {
        enabled = true;
        version = "2";

        default_model = {
          provider = "zed.dev";
          model = "claude-3-5-sonnet-latest";
        };
      };

      node = {
        path = lib.getExe pkgs.nodejs_24;
        npm_path = lib.getExe' pkgs.nodejs_24 "npm";
      };

      hour_format = "hour24";
      auto_update = false;
      terminal = {
        alternate_scroll = "off";
        blinking = "off";
        copy_on_select = false;
        dock = "bottom";
        detect_venv = {
          on = {
            directories = [ ".env" "env" ".venv" "venv" ];
            activate_script = "default";
          };
        };
        env = {
          TERM = "alacritty";
        };
        font_family = "AnonymicePro Nerd Font";
        font_features = null;
        font_size = null;
        line_height = "comfortable";
        option_as_meta = false;
        button = false;
        shell = "system";
        toolbar = {
          title = true;
        };
        working_directory = "current_project_directory";
      };

      lsp = {
        pylsp = {
          binary = {
            path = "${pkgs.python3Packages.python-lsp-server}/bin/pylsp";
          };
        };

        rust-analyzer = {
          binary = {
            path = "${pkgs.rustup}/bin/rust-analyzer";
          };
        };

        nixd = {
          binary = {
            path = "${pkgs.nixd}/bin/nixd";
          };
          settings = {
            nixd = {
              nixpkgs = {
                expr = "import <nixpkgs> { }";
              };
              formatting = {
                command = [ "${pkgs.nixpkgs-fmt}/bin/nixpkgs-fmt" ];
              };
            };
          };
        };

        nil = {
          binary = {
            path = "${pkgs.nil}/bin/nil";
          };
          settings = {
            nil = {
              formatting = {
                command = [ "nixpkgs-fmt" ];
              };
            };
          };
        };

        texlive = {
          binary = {
            path_lookup = true;
          };
        };
      };

      languages = {
        Nix = {
          language_servers = [ "nil" ];
          formatter = {
            external = {
              command = "nixpkgs-fmt";
            };
          };
        };
        Python = {
          language_servers = [ "pylsp" ];
        };
        Cpp = {
          format_on_save = "on";
          tab_size = 2;
        };
      };

      vim_mode = false;
      load_direnv = "shell_hook";
      base_keymap = "VSCode";
      theme = {
        mode = "dark";
        light = "Catppuccin Latte";
        dark = "Catppuccin Macchiato";
      };
      show_whitespaces = "all";
      ui_font_size = 16;
      buffer_font_size = 16;
    };
  };
}
