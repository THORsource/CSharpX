"""Single-controller AUTOSAR TcpIp lifecycle demonstration over Windows loopback."""

import argparse
from contextlib import ExitStack
from enum import Enum
from pathlib import Path
import socket
import subprocess


LOOPBACK = "127.0.0.1"
TIMEOUT = 2


class Mode(Enum):
    OFFLINE = "OFFLINE"
    STARTUP = "STARTUP"
    ONLINE = "ONLINE"
    ONHOLD = "ONHOLD"
    SHUTDOWN = "SHUTDOWN"


class CommunicationUnavailable(RuntimeError):
    pass


class TcpIpDemo:
    def __init__(self, resources):
        self.resources = resources
        self.mode = Mode.OFFLINE
        self.requested_mode = Mode.OFFLINE
        self.transport = resources.enter_context(ExitStack())

    def request_com_mode(self, mode):
        if mode in (Mode.STARTUP, Mode.SHUTDOWN):
            raise ValueError(f"{mode.value} is an internal state")
        if mode is Mode.ONHOLD and self.mode is not Mode.ONLINE:
            raise ValueError("ONHOLD requires ONLINE in this example")
        self.requested_mode = mode

    def transition(self, mode):
        self.mode = mode
        print(f"EthSM mode indication: {mode.value}")

    def main_function(self):
        if self.mode is Mode.OFFLINE and self.requested_mode is Mode.ONLINE:
            self.transition(Mode.STARTUP)
        elif self.mode is Mode.STARTUP:
            self.open_loopback()
            print(f"Upper layer address indication: {LOOPBACK} ASSIGNED")
            self.transition(Mode.ONLINE)
        elif self.mode is Mode.ONLINE and self.requested_mode is Mode.ONHOLD:
            print("Upper layer address indication: ONHOLD")
            self.transition(Mode.ONHOLD)
        elif self.mode is Mode.ONHOLD and self.requested_mode is Mode.ONLINE:
            print(f"Upper layer address indication: {LOOPBACK} ASSIGNED")
            self.transition(Mode.ONLINE)
        elif self.mode in (Mode.ONLINE, Mode.ONHOLD) and self.requested_mode is Mode.OFFLINE:
            print("Upper layer address indication: UNASSIGNED")
            self.transport.close()
            self.transition(Mode.SHUTDOWN)
        elif self.mode is Mode.SHUTDOWN:
            self.transition(Mode.OFFLINE)

    def open_socket(self, kind):
        sock = self.transport.enter_context(socket.socket(socket.AF_INET, kind))
        sock.settimeout(TIMEOUT)
        return sock

    def open_loopback(self):
        with self.open_socket(socket.SOCK_STREAM) as listener:
            listener.bind((LOOPBACK, 0))
            listener.listen(1)
            self.tcp_tx = self.open_socket(socket.SOCK_STREAM)
            self.tcp_tx.connect(listener.getsockname())
            self.tcp_rx, _ = listener.accept()
            self.transport.enter_context(self.tcp_rx)
            self.tcp_rx.settimeout(TIMEOUT)

        self.udp_rx = self.open_socket(socket.SOCK_DGRAM)
        self.udp_rx.bind((LOOPBACK, 0))
        self.udp_tx = self.open_socket(socket.SOCK_DGRAM)
        self.udp_tx.bind((LOOPBACK, 0))

    def transmit(self, protocol, payload):
        if self.mode is not Mode.ONLINE:
            raise CommunicationUnavailable(self.mode.value)

        data = payload.encode("utf-8")
        if not 1 <= len(data) <= 1024:
            raise ValueError("expected a payload of 1–1024 bytes")

        if protocol == "TCP":
            self.tcp_tx.sendall(data)
            received = bytearray()
            while len(received) < len(data):
                chunk = self.tcp_rx.recv(len(data) - len(received))
                if not chunk:
                    raise RuntimeError("TCP peer closed before the full payload arrived")
                received.extend(chunk)
        elif protocol == "UDP":
            self.udp_tx.sendto(data, self.udp_rx.getsockname())
            received, sender = self.udp_rx.recvfrom(1024)
            if sender != self.udp_tx.getsockname():
                raise RuntimeError(f"unexpected UDP sender: {sender}")
        else:
            raise ValueError(f"unsupported protocol: {protocol}")

        if received != data:
            raise RuntimeError(f"{protocol} payload verification failed")
        print(f"Upper layer RX {protocol}: {received.decode('utf-8')}")


def run_scenario(executable):
    output = subprocess.run(
        [str(executable.resolve())], check=True, capture_output=True,
        encoding="utf-8", timeout=TIMEOUT,
    ).stdout
    with ExitStack() as resources:
        demo = TcpIpDemo(resources)
        for line in output.splitlines():
            command, _, payload = line.partition(" ")
            if command in ("ONLINE", "ONHOLD", "OFFLINE") and not payload:
                demo.request_com_mode(Mode(command))
                demo.main_function()
                demo.main_function()
                if demo.mode is not Mode(command):
                    raise RuntimeError(f"requested state {command} was not reached")
            elif command in ("TCP", "UDP"):
                demo.transmit(command, payload)
            elif command == "EXPECT_BLOCKED" and not payload:
                try:
                    demo.transmit("TCP", "must not be sent")
                except CommunicationUnavailable:
                    print(f"Transmission rejected in {demo.mode.value}")
                else:
                    raise RuntimeError("transmission should have been rejected")
            else:
                raise ValueError(f"unknown scenario line: {line!r}")

        if demo.mode is not Mode.OFFLINE:
            raise RuntimeError("scenario ended before OFFLINE")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("executable", type=Path)
    run_scenario(parser.parse_args().executable)
