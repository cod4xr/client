#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Verify bounded immutable public IWI format references without cloning a repository."""
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
REPOSITORY = "https://github.com/Laupetin/OpenAssetTools"
MAXIMUM_BYTES = 256 * 1024


def main() -> None:
    record = json.loads((ROOT / "docs/iwi-format-sources.json").read_text(encoding="utf-8"))
    revision = record["revision"]
    if record["repository"] != REPOSITORY or not re.fullmatch(r"[0-9a-f]{40}", revision):
        raise SystemExit("Invalid immutable IWI reference identity")
    if record["normalization"] != "CRLF to LF":
        raise SystemExit("Unsupported IWI reference normalization")
    sources = record["sources"]
    if not sources or len(sources) > 16:
        raise SystemExit("Invalid IWI reference inventory")
    for name, expected in sources.items():
        relative = PurePosixPath(name)
        if (relative.is_absolute() or ".." in relative.parts
                or not name.startswith("src/ObjImage/Image/")
                or not re.fullmatch(r"[A-Za-z0-9_./]+", name)
                or not re.fullmatch(r"[0-9a-f]{64}", expected)):
            raise SystemExit("Invalid IWI reference path/hash")
        url = f"https://raw.githubusercontent.com/Laupetin/OpenAssetTools/{revision}/{name}"
        request = urllib.request.Request(url, headers={"User-Agent": "CoD4XR-source-audit"})
        with urllib.request.urlopen(request, timeout=20) as response:
            data = response.read(MAXIMUM_BYTES + 1)
        if len(data) > MAXIMUM_BYTES:
            raise SystemExit("IWI reference exceeds inspection budget")
        actual = hashlib.sha256(data.replace(b"\r\n", b"\n")).hexdigest()
        if actual != expected:
            raise SystemExit(f"IWI source hash differs: {name}")
    print(f"Verified {len(sources)} immutable public IWI format references")


if __name__ == "__main__":
    main()
