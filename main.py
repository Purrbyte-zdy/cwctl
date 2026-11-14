from pathlib import Path
import subprocess

from ClassWidgets.SDK import CW2Plugin, PluginAPI


class CwctlPlugin(CW2Plugin):
    def __init__(self, api: PluginAPI):
        super().__init__(api)
        self.binary = Path(__file__).with_name("cwctl")

    def on_load(self):
        super().on_load()

    def on_unload(self):
        pass

    def run(self, *arguments: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [str(self.binary), *arguments],
            check=False,
            text=True,
            capture_output=True,
        )
