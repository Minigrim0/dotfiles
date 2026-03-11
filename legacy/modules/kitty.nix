{ ... }:

{
  programs.kitty = {
    enable = true;

    settings = {
      # Window settings
      background_opacity = "0.8";
      confirm_os_window_close = 0;
      dynamic_background_opacity = true;
      mouse_hide_wait = "3.0";
      window_padding_width = 2;
      background_blur = 5;
      
      # Font settings
      font_size = "12.0";
      font_family = "AnonymicePro Nerd Font Mono";
      
      # Bell settings
      enable_audio_bell = false;
      visual_bell_duration = "0.0";
      
      # Cursor settings
      cursor_shape = "underline";
      
      # Color scheme
      background = "#1F242D"; # Raisin Black (dark)
      foreground = "#ffffff"; # White text
      
      cursor = "#C74D39"; # Jasper
      cursor_text_color = "#1F242D"; # Raisin Black
      
      selection_background = "#645156"; # Wenge
      selection_foreground = "#ffffff"; # White
      
      # Normal colors
      color0 = "#191E28"; # black - Raisin Light
      color1 = "#DF502C"; # red - Cinnabar
      color2 = "#4caf50"; # green
      color3 = "#ffeb3b"; # yellow
      color4 = "#2196f3"; # blue
      color5 = "#C74D39"; # magenta - Jasper
      color6 = "#00bcd4"; # cyan
      color7 = "#ffffff"; # white
      
      # Bright colors
      color8 = "#645156"; # bright black - Wenge
      color9 = "#DF502C"; # bright red - Cinnabar
      color10 = "#66bb6a"; # bright green
      color11 = "#ffee58"; # bright yellow
      color12 = "#42a5f5"; # bright blue
      color13 = "#C74D39"; # bright magenta - Jasper
      color14 = "#26c6da"; # bright cyan
      color15 = "#ffffff"; # bright white
    };
  };
}
