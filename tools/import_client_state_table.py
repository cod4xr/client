#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Reproduce snapshot client-state fields from hash-locked public client source."""
import argparse
import json
from pathlib import Path
import re
import subprocess

from import_entity_baseline_tables import ROOT, locked_text

TARGET = ROOT / "crates/client-core/src/entities/clients/table.rs"
WORDS = {
    "modelindex": "Model", "team": "Team", "rank": "Rank",
    "prestige": "Prestige", "perks": "Perks",
    "maxSprintTimeMultiplier": "SprintTimeMultiplier",
    "attachedVehEntNum": "VehicleEntity", "attachedVehSeat": "VehicleSeat",
}
for index in range(4):
    WORDS[f"name[{index * 4}]"] = f"Name{index}"
for index in range(6):
    WORDS[f"attachModelIndex[{index}]"] = f"AttachmentModel{index}"
    WORDS[f"attachTagIndex[{index}]"] = f"AttachmentTag{index}"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path, help="Read-only public client checkout")
    parser.add_argument("--write", action="store_true", help="Regenerate checked table")
    args = parser.parse_args()
    lock = json.loads((ROOT / "upstream.lock.json").read_text(encoding="utf-8"))
    source = next(item for item in lock["sources"] if item["name"] == "client-reference")
    header = locked_text(args.reference, "src/q_shared.h", source["files"])
    text = locked_text(args.reference, "src/msg.c", source["files"])
    native = re.search(r"typedef struct clientState_s\s*\{(.*?)\}\s*clientState_t;", header, re.S)
    if native is None or not re.search(r"char\s+name\[16\];", native[1]):
        raise SystemExit("Expected native sixteen-byte client-state name")
    tables = re.findall(r"static netField_t clientStateFields\[\] =\s*\{(.*?)\n\};", text, re.S)
    if len(tables) != 1:
        raise SystemExit("Expected one reviewed client-state schema")
    fields = re.findall(r"\{ CSF\( (.*?) \), (-?\d+), (\d+)\}", tables[0])
    if len(fields) != 24 or len({path for path, _, _ in fields}) != 24:
        raise SystemExit("Expected twenty-four distinct client-state words")
    lines = ["// SPDX-License-Identifier: AGPL-3.0-or-later",
             "// Generated from hash-locked msg.c by tools/import_client_state_table.py.",
             "use super::super::fields::{ValueSpec, v};",
             "use super::ClientStateWord as W;", "",
             "pub(super) struct FieldSpec {", "    pub word: W,",
             "    pub value: ValueSpec,", "}", "",
             "const fn f(word: W, bits: i16) -> FieldSpec {",
             "    FieldSpec { word, value: v(bits, false) }", "}", "",
             "pub(super) const FIELDS: [FieldSpec; 24] = ["]
    for path, bits, hint in fields:
        if path not in WORDS or int(bits) not in {0, 2, 5, 8, 9, 10, 32} or hint != "0":
            raise SystemExit(f"Unreviewed client-state field: {path}")
        lines.append(f"    f(W::{WORDS[path]}, {bits}), // {path}")
    lines.extend(["];", ""])
    result = subprocess.run(["rustfmt", "--edition", "2024", "--emit", "stdout"],
                            input="\n".join(lines), capture_output=True, text=True, check=True)
    if args.write:
        TARGET.write_text(result.stdout, encoding="utf-8", newline="\n")
    if not TARGET.is_file() or TARGET.read_text(encoding="utf-8") != result.stdout:
        raise SystemExit("Generated client-state table mismatch")
    print("Verified twenty-four client-state fields against locked public client sources")


if __name__ == "__main__":
    main()
