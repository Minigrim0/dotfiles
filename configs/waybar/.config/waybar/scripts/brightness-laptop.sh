#!/usr/bin/env bash
# Laptop brightness backend — uses brightnessctl

get_brightness() {
    brightnessctl get | awk -v max="$(brightnessctl max)" '{printf "%d\n", ($1/max)*100}'
}

set_brightness_up()   { brightnessctl -e4 -n2 set 5%+; }
set_brightness_down() { brightnessctl -e4 -n2 set 5%-; }
