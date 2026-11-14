from __future__ import annotations

import json
import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def read_versions() -> dict[str, str]:
    cargo = tomllib.loads((ROOT / "Cargo.toml").read_text())["package"]["version"]
    python = tomllib.loads((ROOT / "pyproject.toml").read_text())["project"]["version"]
    plugin = json.loads((ROOT / "cwplugin.json").read_text())["version"]
    return {"Cargo.toml": cargo, "pyproject.toml": python, "cwplugin.json": plugin}


def main() -> int:
    versions = read_versions()
    unique_versions = set(versions.values())
    if len(unique_versions) != 1:
        details = ", ".join(f"{path}={version}" for path, version in versions.items())
        print(f"version mismatch: {details}", file=sys.stderr)
        return 1

    version = next(iter(unique_versions))
    expected = sys.argv[1] if len(sys.argv) > 1 else None
    if expected and version != expected:
        print(f"version mismatch: files={version}, expected={expected}", file=sys.stderr)
        return 1

    print(version)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
