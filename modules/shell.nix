{ config, pkgs, ... }:

{
  programs.zsh = {
    enable = true;
    enableCompletion = true;
    autosuggestion.enable = true;
    syntaxHighlighting.enable = true;

    oh-my-zsh = {
      enable = true;
      plugins = [
        "git"
        "colorize"
        "rust"
        "sudo"
        "zoxide"
        "npm"
      ];
      theme = "agnoster";
    };

    shellAliases = {
      vim = "nvim";
      grep = "grep --color=auto";
      hms = "home-manager switch";
      nrs = "sudo nixos-rebuild switch";
    };

    history = {
      size = 10000;
      path = "${config.xdg.dataHome}/zsh/history";
    };

    initContent = ''
      export EDITOR=nvim
      export BROWSER=firefox
      export SSH_AUTH_SOCK=$XDG_RUNTIME_DIR/gcr/ssh
      eval $(zoxide init zsh)

      export NPM_CONFIG_PREFIX="$HOME/.npm-global"
      export PATH="$NPM_CONFIG_PREFIX/bin:$PATH"

      # Create npm global directory if it doesn't exist
      mkdir -p "$NPM_CONFIG_PREFIX"

      # Set npm config explicitly
      npm config set prefix "$HOME/.npm-global" 2>/dev/null || true

      # Source background script for swww functions
      if [ -f "$HOME/.config/background/swww-utils.sh" ]; then
        source "$HOME/.config/background/swww-utils.sh"
      fi
    '';
  };

  home.sessionVariables = {
    SHELL = "${pkgs.zsh}/bin/zsh";
    NPM_CONFIG_PREFIX = "$HOME/.npm-global";
  };

  home.sessionPath = [
    "$HOME/.npm-global/bin"
  ];
}
