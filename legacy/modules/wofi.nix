{ pkgs, ... }:

{
  programs.wofi = {
    enable = true;

    settings = {
      width = 600;
      height = 400;
      location = "center";
      show = "drun";
      prompt = "Search...";
      filter_rate = 100;
      allow_markup = true;
      no_actions = true;
      halign = "fill";
      orientation = "vertical";
      content_halign = "fill";
      insensitive = true;
      allow_images = true;
      image_size = 40;
      gtk_dark = true;
    };

    style = ''
      window {
        margin: 0px;
        border: 2px solid #645156;
        background-color: rgba(31, 36, 45, 0.9);
        border-radius: 8px;
      }

      #input {
        margin: 5px;
        border: 1px solid #645156;
        color: #ffffff;
        background-color: #1f242d;
        border-radius: 4px;
        padding: 8px;
        font-family: "AnonymicePro Nerd Font", monospace;
        font-size: 12px;
      }

      #inner-box {
        margin: 5px;
        border: none;
        background-color: transparent;
      }

      #outer-box {
        margin: 5px;
        border: none;
        background-color: transparent;
      }

      #scroll {
        margin: 0px;
        border: none;
      }

      #text {
        margin: 5px;
        border: none;
        color: #ffffff;
        font-family: "AnonymicePro Nerd Font", monospace;
        font-size: 11px;
      }

      #entry {
        background-color: transparent;
        border-radius: 4px;
        margin: 2px;
        padding: 5px;
      }

      #entry:selected {
        background-color: #c74d39;
        color: #ffffff;
      }

      #entry:hover {
        background-color: #645156;
        color: #ffffff;
      }

      #text:selected {
        color: #ffffff;
      }
    '';
  };

  # Add wofi-emoji package
  home.packages = with pkgs; [
    wofi-emoji
  ];

  # Create wofi-emoji script
  home.file.".local/bin/wofi-emoji" = {
    text = ''
      #!/usr/bin/env bash
      ${pkgs.wofi-emoji}/bin/wofi-emoji
    '';
    executable = true;
  };
}
