from __future__ import annotations

from pathlib import Path
import os
import socket
import subprocess
import time
from typing import Optional

from ClassWidgets.SDK import CW2Plugin, PluginAPI


class Aria2Runtime:
    def __init__(self, plugin_dir: Path):
        self.binary = next(
            (
                plugin_dir / name
                for name in ("aria2-next", "aria2-next.exe")
                if (plugin_dir / name).is_file()
            ),
            None,
        )
        self.process: Optional[subprocess.Popen] = None

    def start(self) -> None:
        if os.environ.get("CWCTL_ARIA2_RPC"):
            return
        if self.binary is None:
            raise RuntimeError(
                "bundled aria2-next was not found; "
                "set CWCTL_ARIA2_RPC to use an external engine"
            )

        port = self._free_port()
        data_dir = self._data_dir()
        data_dir.mkdir(parents=True, exist_ok=True)
        session_path = data_dir / "aria2.session"
        session_path.touch(exist_ok=True)
        self.process = subprocess.Popen(
            [
                str(self.binary),
                "--enable-rpc=true",
                "--rpc-listen-all=false",
                f"--rpc-listen-port={port}",
                f"--dir={data_dir}",
                f"--save-session={session_path}",
                f"--input-file={session_path}",
                "--continue=true",
                "--auto-file-renaming=false",
            ],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        endpoint = f"http://127.0.0.1:{port}/jsonrpc"
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            if self.process.poll() is not None:
                raise RuntimeError("bundled aria2-next exited during startup")
            try:
                with socket.create_connection(("127.0.0.1", port), timeout=0.2):
                    os.environ["CWCTL_ARIA2_RPC"] = endpoint
                    return
            except OSError:
                time.sleep(0.05)
        self.stop()
        raise RuntimeError("bundled aria2-next did not open its RPC port")

    def stop(self) -> None:
        if self.process is None:
            return
        self.process.terminate()
        try:
            self.process.wait(timeout=2)
        except subprocess.TimeoutExpired:
            self.process.kill()
            self.process.wait()
        self.process = None

    @staticmethod
    def _free_port() -> int:
        with socket.socket() as sock:
            sock.bind(("127.0.0.1", 0))
            return sock.getsockname()[1]

    @staticmethod
    def _data_dir() -> Path:
        if home := os.environ.get("CWCTL_HOME"):
            return Path(home) / "aria2"
        if data_home := os.environ.get("XDG_DATA_HOME"):
            return Path(data_home) / "cwctl" / "aria2"
        return Path.home() / ".local" / "share" / "cwctl" / "aria2"


class CwctlPlugin(CW2Plugin):
    def __init__(self, api: PluginAPI):
        super().__init__(api)
        plugin_dir = Path(__file__).parent
        self.binary = next(
            (
                plugin_dir / name
                for name in ("cwctl", "cwctl.exe")
                if (plugin_dir / name).is_file()
            ),
            plugin_dir / "cwctl",
        )
        self.aria2 = Aria2Runtime(plugin_dir)

    def on_load(self):
        super().on_load()
        self.aria2.start()

    def on_unload(self):
        self.aria2.stop()

    def run(self, *arguments: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [str(self.binary), *arguments],
            check=False,
            text=True,
            capture_output=True,
        )
