#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Compare Rust fastfile inspection with independent Python zlib/size/SHA-256 checks.

Requires an operator-owned zone directory and built inspector. Stores aggregate
metadata only when requested; never extracts assets or reports file/asset names.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import zlib

MAXIMUM_FILE_BYTES = 512 * 1024 * 1024
MAXIMUM_DECODED_BYTES = 512 * 1024 * 1024


def reference(path: Path) -> dict:
    with path.open("rb") as stream:
        encoded = stream.read(MAXIMUM_FILE_BYTES + 1)
    if len(encoded) > MAXIMUM_FILE_BYTES or encoded[:12] != b"IWffu100\x05\0\0\0":
        raise ValueError("Reference fastfile profile or byte budget mismatch")
    decoder = zlib.decompressobj()
    decoded = decoder.decompress(encoded[12:], MAXIMUM_DECODED_BYTES + 1)
    if (len(decoded) > MAXIMUM_DECODED_BYTES or not decoder.eof
            or decoder.unused_data or decoder.unconsumed_tail or len(decoded) < 44):
        raise ValueError("Reference fastfile zlib extent mismatch")
    sizes = struct.unpack_from("<11I", decoded)
    if len(decoded) != sizes[0] + 44:
        raise ValueError("Reference fastfile size declaration mismatch")
    return {"encoded_bytes": len(encoded), "decoded_bytes": len(decoded),
            "serialized_body_bytes": sizes[0], "declared_external_bytes": sizes[1],
            "declared_total_block_bytes": sum(sizes[2:]),
            "decoded_sha256": list(hashlib.sha256(decoded).digest())}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("zone_directory", type=Path)
    parser.add_argument("inspector", type=Path)
    parser.add_argument("evidence", type=Path)
    parser.add_argument("--mode", choices=["debug", "release"], required=True)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    if not args.zone_directory.is_dir() or not args.inspector.is_file():
        raise SystemExit("Caller-owned zone directory and built inspector are required")
    paths = sorted(args.zone_directory.glob("**/*.ff"),
                   key=lambda path: path.relative_to(args.zone_directory).as_posix())
    if not paths or len(paths) > 10000:
        raise SystemExit("Reference fastfile inventory outside audit budget")
    fields = ["encoded_bytes", "decoded_bytes", "serialized_body_bytes",
              "declared_external_bytes", "declared_total_block_bytes"]
    totals = dict.fromkeys(fields, 0)
    largest = 0
    aggregate = hashlib.sha256()
    for ordinal, path in enumerate(paths, 1):
        wanted = reference(path)
        process = subprocess.run([str(args.inspector.resolve()), "--inspect-fastfile", str(path),
                                  "--log-format", "json"], capture_output=True, timeout=120)
        if process.returncode:
            raise ValueError(f"Rust fastfile inspection failed at ordinal {ordinal}")
        actual = json.loads(process.stdout)
        if (actual["container_integrity"] != "zlib_adler32_verified"
                or actual["asset_interpretation"] != "unverified"
                or any(actual[name] != value for name, value in wanted.items())):
            raise ValueError(f"Fastfile metadata or decoded content hash differs at ordinal {ordinal}")
        for name in fields:
            totals[name] += wanted[name]
        aggregate.update(bytes(wanted["decoded_sha256"]))
        largest = max(largest, wanted["decoded_bytes"])
        print(f"Independent fastfile comparison: {ordinal} of {len(paths)}", flush=True)
    record = {"files": len(paths), **totals, "largest_decoded_file": largest,
              "sha256_of_ordered_decoded_sha256": aggregate.hexdigest(),
              "order": "POSIX lexical relative zone paths; names omitted",
              "independent_python_comparison": "every full decoded SHA-256 and size field matched",
              "asset_interpretation": "unverified", "gameplay_compatibility": "unverified"}
    modes = []
    if args.evidence.exists():
        previous = json.loads(args.evidence.read_text(encoding="utf-8"))
        modes = previous.pop("modes")
        if record != previous:
            raise ValueError("Aggregate fastfile evidence differs from the recorded corpus")
    elif not args.write:
        raise ValueError("Aggregate fastfile evidence does not exist")
    if args.write:
        record["modes"] = sorted(set([*modes, args.mode]))
        args.evidence.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    print(f"Matched {len(paths)} complete fastfiles and decoded SHA-256 digests")


if __name__ == "__main__":
    main()
