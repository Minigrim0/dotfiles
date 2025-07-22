{pkgs, lib, ... }:

{
  programs.zed-editor = {
    enable = true;

    ## This populates the userSettings "auto_install_extensions"
    extensions = ["nix" "toml" "tex" "make"];

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
        path = lib.getExe pkgs.nodejs;
        npm_path = lib.getExe' pkgs.nodejs "npm";
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
                directories = [".env" "env" ".venv" "venv"];
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
        rust-analyzer = {
          binary = {
            path_lookup = true;
          };
        };
        nix = {
          binary = {
            path_lookup = true;
          };
        };

        texlive = {
          binary = {
            path_lookup = true;
          };
        };
      };

      vim_mode = false;
      load_direnv = "shell_hook";
      base_keymap = "VSCode";
      theme = {
          mode = "system";
          light = "Catppuccin Latte";
          dark = "Catppuccin Macchiato";
      };
      show_whitespaces = "all" ;
      ui_font_size = 16;
      buffer_font_size = 16;
    };
  };
}
