#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Verify the exact public research snapshots; never build or modify them."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("client_reference", type=Path)
    parser.add_argument("server_reference", type=Path)
    args = parser.parse_args()
    directories = {"client-reference": args.client_reference, "server-reference": args.server_reference}
    lock = json.loads((ROOT / "upstream.lock.json").read_text(encoding="utf-8"))
    checked = 0
    for source in lock["sources"]:
        directory = directories[source["name"]]
        revision = subprocess.check_output(["git", "-C", str(directory), "rev-parse", "HEAD"], text=True).strip()
        if revision != source["revision"]:
            raise SystemExit(f"Revision differs: {source['name']}")
        for name, expected in source["files"].items():
            relative = Path(name)
            if relative.is_absolute() or ".." in relative.parts:
                raise SystemExit("Invalid locked relative source path")
            data = (directory / relative).read_bytes().replace(b"\r\n", b"\n")
            if hashlib.sha256(data).hexdigest() != expected:
                raise SystemExit(f"Source hash differs: {source['name']}/{name}")
            checked += 1
    print(f"Verified {checked} files in {len(lock['sources'])} pinned research snapshots")


if __name__ == "__main__":
    main()
