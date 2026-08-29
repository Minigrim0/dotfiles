#!/usr/bin/env bash
# Suspend on idle only when this machine is a laptop (desktop stays up for
# long-running work; its screens are already DPMS-off by this point).
[ "$(hostnamectl chassis 2>/dev/null)" = "laptop" ] && systemctl suspend
