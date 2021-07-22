import socket
import time

import fontawesome as fa


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
    except socket.timeout:
        return False


hosts = [("cloud", "10.0.0.1"), ("laptop", "10.0.0.3")]

messages = []
for host in hosts:
    if is_up(host[1]):
        messages.append(
            f"{host[0]} {'%{F#66ee53}'}{fa.icons['check-circle']}{'%{F-}'}")
    else:
        messages.append(
            f"{host[0]} {'%{F#ee1117}'}{fa.icons['times-circle']}{'%{F-}'}")

print(" | ".join(messages))
