import os
import glob
import socket
import time

import fontawesome as fa

TINC_NETNAME = os.environ.get("TINC_NETNAME", "teamname")
TINC_LOCALHOST = os.environ.get("TINC_LOCALHOST", None)


class Socket(object):
    def __init__(self, family, type_, timeout):
        s = socket.socket(family, type_)
        s.settimeout(timeout)
        self._s = s

    def connect(self, host, port=80):
        self._s.connect((host, int(port)))

    def shutdown(self):
        self._s.shutdown(socket.SHUT_RD)

    def close(self):
        self._s.close()


def is_up(host: str) -> bool:

    s = Socket(socket.AF_INET, socket.SOCK_STREAM, timeout=5)

    try:
        time.sleep(1)
        s.connect(host, 22)
        s.shutdown()
        return True
    except Exception:
        return False


hosts: list[tuple] = []

for host in glob.glob(f"/etc/tinc/{TINC_NETNAME}/hosts/*"):
    hostname = host.split("/")[-1]
    if hostname == TINC_LOCALHOST:
        continue
    with open(host) as config:
        for line in config.readlines():
            splitted = line.split()
            if splitted[0] == "Subnet":
                ip = splitted[-1].split("/")[0]
                hosts.append((hostname, ip))
                break

messages = []
for host in hosts:
    if is_up(host[1]):
        messages.append(
            f"{host[0]} {'%{F#66ee53}'}{fa.icons['check-circle']}{'%{F-}'}")
    else:
        messages.append(
            f"{host[0]} {'%{F#ee1117}'}{fa.icons['times-circle']}{'%{F-}'}")

print(" | ".join(messages))
