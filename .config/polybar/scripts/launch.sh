#!/usr/bin/env bash

source $HOME/.zshlocalrc
killall -q polybar

if [[ $MONITOR_AMOUNT -eq 1 ]]; then
    for m in $(polybar --list-monitors | cut -d":" -f1); do
        MONITOR=$m polybar --config=$HOME/.config/polybar/config.ini --reload top-DP >>/tmp/polybar1.log 2>&1 &
        MONITOR=$m polybar --config=$HOME/.config/polybar/config.ini --reload bottom-DP >>/tmp/polybar2.log 2>&1 &
    done
else
    MONITOR=DP-4 polybar --config=$HOME/.config/polybar/config.ini --reload top-DP >>/tmp/polybar1.log 2>&1 &
    MONITOR=DP-4 polybar --config=$HOME/.config/polybar/config.ini --reload bottom-DP >>/tmp/polybar2.log 2>&1 &
    MONITOR=HDMI-0 polybar --config=$HOME/.config/polybar/config.ini --reload top-HDMI >>/tmp/polybar3.log 2>&1 &
fi

echo "Bars launched..."
