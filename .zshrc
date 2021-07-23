export ZSH="/home/minigrim0/.oh-my-zsh"

ZSH_THEME="gallois"

TERM=xterm-256color
EDITOR=vim
export EDITOR=vim

ZSH_COLORIZE_TOOL=pygmentize
ZSH_COLORIZE_STYLZ="colorful"
COMPLETION_WAITING_DOTS="true"

plugins=(
  git
  django
  history
  colorize
)

source $ZSH/oh-my-zsh.sh

alias irc="TERM=xterm && ssh minigrim0@irc.urlab.be"
alias activate="source */bin/activate"

# Change directory and list its content
lcd(){
    cd $1;
    ls;
}

# Open a port (to play in lan)
openport(){
    sudo iptables -I INPUT -p tcp --dport $1 --syn -j ACCEPT
}

# List all installed package and their size
listall(){
    LC_ALL=C pacman -Qi | awk '/^Name/{name=$3} /^Installed Size/{print $4$5, name}' | sort -h
}
autoremove(){
    sudo pacman -Rcns $(pacman -Qdtq)
}
searchall(){
    listall | grep $1
}
findin(){
    cat $1 | grep $2
}
cat(){
    ccat $1
}

alias config='/usr/bin/git --git-dir=/home/minigrim0/.cfg/ --work-tree=/home/minigrim0'
alias activate="source */bin/activate"

source $HOME/.zshlocalrc
