#!/usr/bin/env bash

# Terminate already running bar instances
killall -q polybar
# If all your bars have ipc enabled, you can also use 
# polybar-msg cmd quit

# Launch bar1 and bar2
MONITOR=DP-4 polybar --config=$HOME/.config/polybar/config.ini --reload top-DP >>/tmp/polybar1.log # 2>&1 &
MONITOR=DP-4 polybar --config=$HOME/.config/polybar/config.ini --reload bottom-DP >>/tmp/polybar2.log # 2>&1 &
MONITOR=HDMI-0 polybar --config=$HOME/.config/polybar/config.ini --reload top-HDMI >>/tmp/polybar3.log # 2>&1 &

echo "Bars launched..."
