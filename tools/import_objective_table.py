#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Reproduce player objective fields from the hash-locked public client schema."""
import argparse
import json
from pathlib import Path
import re
import subprocess

from import_entity_baseline_tables import ROOT, locked_text

TARGET = ROOT / "crates/client-core/src/entities/objectives/table.rs"
WORDS = {"origin[0]": "OriginX", "origin[1]": "OriginY", "origin[2]": "OriginZ",
         "icon": "Icon", "entNum": "Entity", "teamNum": "Team"}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path, help="Read-only public client checkout")
    parser.add_argument("--write", action="store_true", help="Regenerate checked table")
    args = parser.parse_args()
    lock = json.loads((ROOT / "upstream.lock.json").read_text(encoding="utf-8"))
    source = next(item for item in lock["sources"] if item["name"] == "client-reference")
    header = locked_text(args.reference, "src/q_shared.h", source["files"])
    text = locked_text(args.reference, "src/msg.c", source["files"])
    if not re.search(r"objective_t\s+objective\[16\];", header):
        raise SystemExit("Expected native sixteen-objective player section")
    tables = re.findall(r"static netField_t objectiveFields\[\] =\s*\{(.*?)\n\};", text, re.S)
    if len(tables) != 1:
        raise SystemExit("Expected one reviewed objective schema")
    fields = re.findall(r"\{ OBJF\( (.*?) \), (-?\d+), (\d+)\}", tables[0])
    if len(fields) != 6 or {path for path, _, _ in fields} != set(WORDS):
        raise SystemExit("Expected six distinct reviewed objective words")
    lines = ["// SPDX-License-Identifier: AGPL-3.0-or-later",
             "// Generated from hash-locked msg.c by tools/import_objective_table.py.",
             "use super::super::fields::{ValueSpec, v};", "use super::ObjectiveWord as W;", "",
             "pub(super) struct FieldSpec {", "    pub word: W,",
             "    pub value: ValueSpec,", "}", "",
             "const fn f(word: W, bits: i16) -> FieldSpec {",
             "    FieldSpec { word, value: v(bits, false) }", "}", "",
             "pub(super) const FIELDS: [FieldSpec; 6] = ["]
    for path, bits, hint in fields:
        if int(bits) not in {0, 4, 10, 12} or hint != "0":
            raise SystemExit(f"Unreviewed objective field: {path}")
        lines.append(f"    f(W::{WORDS[path]}, {bits}), // {path}")
    lines.extend(["];", ""])
    result = subprocess.run(["rustfmt", "--edition", "2024", "--emit", "stdout"],
                            input="\n".join(lines), capture_output=True, text=True, check=True)
    if args.write:
        TARGET.write_text(result.stdout, encoding="utf-8", newline="\n")
    if not TARGET.is_file() or TARGET.read_text(encoding="utf-8") != result.stdout:
        raise SystemExit("Generated objective table mismatch")
    print("Verified six objective fields and sixteen-slot section against locked public sources")


if __name__ == "__main__":
    main()
