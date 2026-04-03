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
export SSH_AUTH_SOCK="${XDG_RUNTIME_DIR}/gcr/ssh"

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
alias ssh="TERM=xterm-256color ssh"

# ---------------------------------------------------------------------------
# Screen helper: `sc <app>` to attach/create sessions by app name
# ---------------------------------------------------------------------------

function sc() {
    # Pass through flags to real screen (e.g. sc -ls, sc -r foo)
    if [[ $# -eq 0 ]] || [[ "$1" == -* ]]; then
        command screen "$@"
        return
    fi

    local app="$1"
    shift
    local extra_args=("$@")

    # Collect session IDs matching the app name (format: PID.name)
    local sessions=()
    while IFS= read -r line; do
        [[ -n "$line" ]] && sessions+=("$line")
    done < <(command screen -ls 2>/dev/null | grep -E "^\s+[0-9]+\..*${app}" | awk '{print $1}')

    local count=${#sessions[@]}

    if [[ $count -eq 0 ]]; then
        echo "No screen session for '$app', creating one..."
        command screen -S "$app" "$app" "${extra_args[@]}"
    elif [[ $count -eq 1 ]]; then
        echo "Attaching to '${sessions[1]}'..."
        command screen -x "${sessions[1]}"
    else
        echo "Multiple screen sessions found for '$app':"
        local full_lines=()
        while IFS= read -r line; do
            [[ -n "$line" ]] && full_lines+=("$line")
        done < <(command screen -ls 2>/dev/null | grep -E "^\s+[0-9]+\..*${app}")

        for i in {1..$count}; do
            printf "  [%d] %s\n" "$i" "${full_lines[$i]}"
        done
        printf "Choose a session [1-%d]: " "$count"
        read -r choice
        if [[ "$choice" =~ ^[0-9]+$ ]] && (( choice >= 1 && choice <= count )); then
            command screen -x "${sessions[$choice]}"
        else
            echo "Invalid choice"
            return 1
        fi
    fi
}

# dots management shortcuts
alias dots='$HOME/Documents/dotfiles/dots/target/release/dots'
export PATH="$HOME/.local/bin:$PATH"

export UV_ENV_FILE=.env
