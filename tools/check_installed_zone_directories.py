#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Compare Rust published zone catalogs with independent Python prefix parsing.

Reads operator-owned content. Reports counts and aggregate hashes only; never writes
strings, asset names, serialized data or child assets.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import subprocess
from check_installed_fastfiles import decode_reference


def reference(decoded: bytes, sizes: tuple) -> dict:
    strings, string_pointer, assets, asset_pointer = struct.unpack_from("<iIiI", decoded, 44)
    if not 0 <= strings <= 1_000_000 or not 0 <= assets <= 1_000_000:
        raise ValueError("Reference zone count budget exceeded")
    for count, pointer in [(strings, string_pointer), (assets, asset_pointer)]:
        if not (pointer == 0xffffffff or (pointer == 0 and count == 0)):
            raise ValueError("Reference zone root link unsupported")
    pointers = struct.unpack_from(f"<{strings}I", decoded, 60)
    cursor = 60 + strings * 4
    start = cursor
    nulls = 0
    for pointer in pointers:
        if pointer == 0:
            nulls += 1
        elif pointer == 0xffffffff:
            cursor = decoded.index(0, cursor, min(len(decoded), 64 * 1024 * 1024 + 44)) + 1
        else:
            raise ValueError("Reference script-string alias unsupported")
    byte_count = cursor - start
    if byte_count > 32 * 1024 * 1024:
        raise ValueError("Reference inline string byte budget exceeded")
    digest = list(hashlib.sha256(decoded[start:cursor]).digest())
    kinds = [0] * 33
    links = dict.fromkeys(["null", "following", "insert", "offset"], 0)
    end = cursor + assets * 8
    if end > min(len(decoded), 64 * 1024 * 1024 + 44):
        raise ValueError("Reference asset directory extent invalid")
    for index in range(assets):
        kind, pointer = struct.unpack_from("<II", decoded, cursor + index * 8)
        if kind >= 33:
            raise ValueError("Reference zone type namespace unsupported")
        kinds[kind] += 1
        links[{0: "null", 0xffffffff: "following", 0xfffffffe: "insert"}.get(pointer, "offset")] += 1
    virtual = strings * 4 + byte_count
    if asset_pointer:
        virtual = (virtual + 3) // 4 * 4
    virtual += assets * 8
    if virtual > sizes[6]:
        raise ValueError("Reference zone virtual prefix exceeds declaration")
    return {"script_strings": strings, "null_strings": nulls, "asset_records": assets,
            "inline_string_bytes": byte_count, "body_prefix_bytes": end - 44,
            "virtual_prefix_bytes": virtual, "asset_type_counts": kinds,
            "asset_link_counts": links, "script_bytes_sha256": digest}


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
        raise SystemExit("Zone reference inventory outside audit budget")
    fields = ["script_strings", "null_strings", "asset_records", "inline_string_bytes",
              "body_prefix_bytes", "virtual_prefix_bytes"]
    totals = dict.fromkeys(fields, 0)
    kinds = [0] * 33
    links = dict.fromkeys(["null", "following", "insert", "offset"], 0)
    hashes = hashlib.sha256()
    largest = 0
    for ordinal, path in enumerate(paths, 1):
        _, decoded, sizes = decode_reference(path)
        wanted = reference(decoded, sizes)
        whole_digest = list(hashlib.sha256(decoded).digest())
        del decoded
        process = subprocess.run([str(args.inspector.resolve()), "--inspect-zone-directory", str(path),
                                  "--log-format", "json"], capture_output=True, timeout=120)
        if process.returncode:
            raise ValueError(f"Rust zone inspection failed at ordinal {ordinal}")
        actual = json.loads(process.stdout)
        if (actual["container_integrity"] != "zlib_adler32_verified"
                or actual["decoded_sha256"] != whole_digest
                or actual["asset_interpretation"] != "root_directory_only"
                or actual["child_asset_interpretation"] != "unverified"
                or actual["zone_directory"] != wanted):
            raise ValueError(f"Zone published metadata/string hash differs at ordinal {ordinal}")
        for name in fields:
            totals[name] += wanted[name]
        kinds = [a + b for a, b in zip(kinds, wanted["asset_type_counts"])]
        for name in links:
            links[name] += wanted["asset_link_counts"][name]
        hashes.update(bytes(wanted["script_bytes_sha256"]))
        largest = max(largest, wanted["body_prefix_bytes"])
        print(f"Independent zone catalog comparison: {ordinal} of {len(paths)}", flush=True)
    record = {"files": len(paths), **totals, "asset_type_counts": kinds, "asset_link_counts": links,
              "largest_body_prefix_bytes": largest,
              "sha256_of_ordered_script_bytes_sha256": hashes.hexdigest(),
              "order": "POSIX lexical relative zone paths; names omitted",
              "independent_python_comparison": "every full decoded hash and published root summary/string hash matched",
              "child_asset_interpretation": "unverified", "gameplay_compatibility": "unverified"}
    modes = []
    if args.evidence.exists():
        previous = json.loads(args.evidence.read_text(encoding="utf-8"))
        modes = previous.pop("modes")
        if previous != record:
            raise ValueError("Aggregate zone evidence differs from the recorded corpus")
    elif not args.write:
        raise ValueError("Aggregate zone evidence does not exist")
    if args.write:
        record["modes"] = sorted(set([*modes, args.mode]))
        args.evidence.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    print(f"Matched {len(paths)} published zone root catalogs and cooked string SHA-256 digests")


if __name__ == "__main__":
    main()
