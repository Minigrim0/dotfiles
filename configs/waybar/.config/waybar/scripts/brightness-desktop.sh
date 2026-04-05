#!/usr/bin/env bash
# Desktop brightness backend — uses ddcutil (DDC/CI)

get_brightness() {
    ddcutil getvcp 10 2>/dev/null | grep -oP 'current value =\s+\K\d+'
}

set_brightness_up()   { ddcutil setvcp 10 + 5; }
set_brightness_down() { ddcutil setvcp 10 - 5; }
