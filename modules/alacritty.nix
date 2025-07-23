{ ... }:

{
  programs.alacritty = {
    enable = true;

    settings = {
      window = {
        opacity = 0.8;
      };

      font = {
        size = 12.0;
        normal = {
          family = "AnonymicePro Nerd Font Mono";
        };
      };

      bell = {
        animation = "Ease";
        duration = 0;
      };

      cursor = {
        style = {
          shape = "Underline";
        };
      };

      colors = {
        primary = {
          background = "#1F242D"; # Raisin Black (dark)
          foreground = "#ffffff"; # White text
        };

        cursor = {
          text = "#1F242D"; # Raisin Black
          cursor = "#C74D39"; # Jasper
        };

        selection = {
          text = "#ffffff"; # White
          background = "#645156"; # Wenge
        };

        normal = {
          black = "#191E28"; # Raisin Light
          red = "#DF502C"; # Cinnabar
          green = "#4caf50"; # Green (complementary)
          yellow = "#ffeb3b"; # Yellow (complementary)
          blue = "#2196f3"; # Blue (complementary)
          magenta = "#C74D39"; # Jasper
          cyan = "#00bcd4"; # Cyan (complementary)
          white = "#ffffff"; # White
        };

        bright = {
          black = "#645156"; # Wenge
          red = "#DF502C"; # Cinnabar (same as normal)
          green = "#66bb6a"; # Brighter green
          yellow = "#ffee58"; # Brighter yellow
          blue = "#42a5f5"; # Brighter blue
          magenta = "#C74D39"; # Jasper (same as normal)
          cyan = "#26c6da"; # Brighter cyan
          white = "#ffffff"; # White
        };
      };
    };
  };
}
