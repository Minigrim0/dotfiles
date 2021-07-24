#!/usr/bin/env bash

zscroll --before-text "%{T3}%{F#0AA641}%{F0} " --delay 0.3 \
		--match-command "python .config/polybar/py/spotify.py" \
		--update-check true "python .config/polybar/py/spotify.py" &

wait
