# =============================================================================
# Zsh configuration
# =============================================================================

# ---------------------------------------------------------------------------
# History
# ---------------------------------------------------------------------------

HISTSIZE=10000
SAVEHIST=10000
HISTFILE="${XDG_DATA_HOME:-$HOME/.local/share}/zsh/history"

setopt HIST_IGNORE_DUPS
setopt HIST_IGNORE_SPACE
setopt SHARE_HISTORY

# ---------------------------------------------------------------------------
# Completion
# ---------------------------------------------------------------------------

autoload -Uz compinit
compinit -d "${XDG_CACHE_HOME:-$HOME/.cache}/zsh/zcompdump"

zstyle ':completion:*' menu select
zstyle ':completion:*' matcher-list 'm:{a-z}={A-Z}'

# ---------------------------------------------------------------------------
# Key bindings
# ---------------------------------------------------------------------------

bindkey '\e[1;5C' forward-word    # Ctrl+Right
bindkey '\e[1;5D' backward-word   # Ctrl+Left
bindkey '\e[3;5~' kill-word       # Ctrl+Delete
bindkey '^H'      backward-kill-word  # Ctrl+Backspace
bindkey '^[[H'    beginning-of-line   # Home
bindkey '^[[F'    end-of-line         # End

# ---------------------------------------------------------------------------
# Plugins  (installed via pacman)
# ---------------------------------------------------------------------------

source /usr/share/zsh/plugins/zsh-autosuggestions/zsh-autosuggestions.zsh
source /usr/share/zsh/plugins/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh

eval "$(zoxide init zsh)"
eval "$(starship init zsh)"

# ---------------------------------------------------------------------------
# npm global (no sudo)
# ---------------------------------------------------------------------------

export NPM_CONFIG_PREFIX="$HOME/.npm-global"
export PATH="$NPM_CONFIG_PREFIX/bin:$PATH"
mkdir -p "$NPM_CONFIG_PREFIX"

# ---------------------------------------------------------------------------
# Env
# ---------------------------------------------------------------------------

export EDITOR=nvim
export BROWSER=firefox
export SSH_AUTH_SOCK="${XDG_RUNTIME_DIR}/ssh-agent"

# ---------------------------------------------------------------------------
# Aliases
# ---------------------------------------------------------------------------

alias vim='nvim'
alias grep='grep --color=auto'
alias cat='bat'
alias ls='lsd'
alias ll='lsd -lah'
alias la='lsd -a'
alias l='lsd -lh'

# dots management shortcuts
alias dots='$HOME/Documents/dotfiles/dots/target/release/dots'
