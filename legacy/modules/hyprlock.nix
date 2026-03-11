{ ... }:

{
  programs.hyprlock = {
    enable = true;

    settings = {
      background = {
        monitor = "";
        path = "screenshot";
        color = "rgba(25, 20, 20, 1.0)";
        blur_passes = 2;
      };

      label = [
        {
          monitor = "";
          text = "cmd[update:1000] echo \"$(date +%H:%M)\"";
          color = "rgba(200, 200, 200, 1.0)";
          font_size = 80;
          font_family = "sans-serif Bold";
          halign = "center";
          valign = "center";
          position = "0, 200";
        }
        {
          monitor = "";
          text = "cmd[update:43200000] echo \"$(date +\"%A, %B %d\")\"";
          color = "rgba(200, 200, 200, 0.6)";
          font_size = 18;
          font_family = "sans-serif";
          halign = "center";
          valign = "center";
          position = "0, -200";
        }
      ];

      input-field = {
        halign = "center";
        valign = "center";
      };
    };
  };
}
