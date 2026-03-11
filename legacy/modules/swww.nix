{ config, pkgs, ... }:

{
  # Install swww package
  home.packages = with pkgs; [
    swww
    ffmpeg
    acpi
  ];

  # Create background script in background directory
  home.file.".config/background/swww-utils.sh" = {
    text = ''
      # SWWW Aliases and Functions

      # Convert video to optimized GIF for swww
      alias vid2gif='convert_video_to_gif'

      # Quick swww commands
      alias swww-start='swww init'
      alias swww-kill='swww kill'
      alias swww-img='swww img'
      alias swww-clear='swww clear'
      alias swww-ls='swww-list'

      # Function to convert video to optimized GIF and extract first frame
      convert_video_to_gif() {
          if [ $# -eq 0 ]; then
              echo "Usage: vid2gif <input_video> [output_name] [width] [fps]"
              echo "Example: vid2gif video.mp4 background 1920 15"
              return 1
          fi
          
          local input="$1"
          local output="''${2:-converted_bg}"
          local width="''${3:-1920}"
          local fps="''${4:-15}"
          
          if [ ! -f "$input" ]; then
              echo "Error: File '$input' not found"
              return 1
          fi
          
          # Create directories if they don't exist
          mkdir -p ~/.config/background/animated
          mkdir -p ~/.config/background/static
          
          local animated_path="$HOME/.config/background/animated/''${output}.gif"
          local static_path="$HOME/.config/background/static/''${output}.jpg"
          
          echo "Converting $input..."
          echo "Settings: ''${width}px width, ''${fps}fps"
          echo "Output paths:"
          echo "  Animated: $animated_path"
          echo "  Static:   $static_path"
          
          # Extract first frame as static image
          echo "📸 Extracting first frame..."
          ${pkgs.ffmpeg}/bin/ffmpeg -i "$input" \
              -vf "scale=''${width}:-1:flags=lanczos" \
              -vframes 1 \
              -q:v 2 \
              "$static_path" -y
          
          if [ $? -ne 0 ]; then
              echo "✗ Failed to extract first frame"
              return 1
          fi
          
          # Convert video to optimized GIF
          echo "🎞️  Converting to animated GIF..."
          ${pkgs.ffmpeg}/bin/ffmpeg -i "$input" \
              -vf "scale=''${width}:-1:flags=lanczos,fps=''${fps}" \
              -c:v gif \
              -f gif \
              "$animated_path" -y
          
          if [ $? -eq 0 ]; then
              echo "✓ Conversion complete!"
              echo "✓ Animated: $animated_path"
              echo "✓ Static:   $static_path"
              echo "✓ To use: swww-smart $animated_path $static_path"
          else
              echo "✗ GIF conversion failed"
              return 1
          fi
      }

      # Function to check if on battery power
      is_on_battery() {
          # Check multiple battery indicators
          if command -v ${pkgs.acpi}/bin/acpi >/dev/null 2>&1; then
              ${pkgs.acpi}/bin/acpi -a | grep -q "off-line"
          elif [ -f /sys/class/power_supply/ADP*/online ]; then
              [ "$(cat /sys/class/power_supply/ADP*/online)" = "0" ]
          elif [ -f /sys/class/power_supply/AC*/online ]; then
              [ "$(cat /sys/class/power_supply/AC*/online)" = "0" ]
          else
              # Fallback: assume plugged in if can't determine
              false
          fi
      }

      # Smart wallpaper switcher (battery-aware) - updated for new paths
      swww-smart() {
          local name="$1"
          
          if [ -z "$name" ]; then
              echo "Usage: swww-smart <background_name>"
              echo "Example: swww-smart my_background"
              echo ""
              echo "Available backgrounds:"
              if [ -d ~/.config/background/animated ]; then
                  ls ~/.config/background/animated/*.gif 2>/dev/null | sed 's/.*\/\([^.]*\).gif/  \1/' | head -10
              fi
              return 1
          fi
          
          local animated_bg="$HOME/.config/background/animated/''${name}.gif"
          local static_bg="$HOME/.config/background/static/''${name}.jpg"
          
          # Check if files exist
          if [ ! -f "$animated_bg" ]; then
              echo "Error: Animated background not found: $animated_bg"
              return 1
          fi
          
          if [ ! -f "$static_bg" ]; then
              echo "Error: Static background not found: $static_bg"
              return 1
          fi
          
          if is_on_battery; then
              echo "🔋 On battery - using static wallpaper"
              ${pkgs.swww}/bin/swww img "$static_bg" --transition-type fade --transition-duration 1
          else
              echo "🔌 Plugged in - using animated wallpaper"
              ${pkgs.swww}/bin/swww img "$animated_bg" --transition-type fade --transition-duration 1
          fi
      }

      # Auto-start swww and set smart wallpaper - updated for new paths
      swww-auto() {
          local name="$1"
          
          if [ -z "$name" ]; then
              echo "Usage: swww-auto <background_name>"
              return 1
          fi
          
          # Start swww daemon if not running
          if ! pgrep -x swww-daemon >/dev/null; then
              echo "Starting swww daemon..."
              ${pkgs.swww}/bin/swww init
              sleep 1
          fi
          
          # Set wallpaper based on power status
          swww-smart "$name"
      }

      # List available backgrounds
      swww-list() {
          echo "Available backgrounds:"
          if [ -d ~/.config/background/animated ]; then
              for gif in ~/.config/background/animated/*.gif; do
                  if [ -f "$gif" ]; then
                      local name=$(basename "$gif" .gif)
                      local static="$HOME/.config/background/static/''${name}.jpg"
                      if [ -f "$static" ]; then
                          echo "  ✓ $name (animated + static)"
                      else
                          echo "  ⚠ $name (animated only)"
                      fi
                  fi
              done
          else
              echo "  No backgrounds found in ~/.config/background/"
          fi
      }
    '';
    executable = false;
  };

  # Create background directories
  home.file.".config/background/animated/.gitkeep".text = "";
  home.file.".config/background/static/.gitkeep".text = "";
}