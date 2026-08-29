#!/usr/bin/env bash
# Desktop brightness backend — DDC/CI via `dots monitor` (cached i2c bus map)

get_brightness() {
    dots monitor get 2>/dev/null
}

set_brightness_up()   { dots monitor brightness +5; }
set_brightness_down() { dots monitor brightness -5; }
