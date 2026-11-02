#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Reproduce native HUD fields from the hash-locked public client schema."""
import argparse
import json
from pathlib import Path
import re
import subprocess

from import_entity_baseline_tables import ROOT, locked_text

TARGET = ROOT / "crates/client-core/src/entities/hud/table.rs"
WORDS = {
    "type": "Kind", "x": "X", "y": "Y", "z": "Z", "targetEntNum": "TargetEntity",
    "fontScale": "FontScale", "font": "Font", "alignOrg": "AlignOrigin",
    "alignScreen": "AlignScreen", "color.rgba": "Color", "fromColor.rgba": "FromColor",
    "fadeStartTime": "FadeStartTime", "fadeTime": "FadeTime", "label": "Label",
    "width": "Width", "height": "Height", "materialIndex": "Material",
    "offscreenMaterialIdx": "OffscreenMaterial", "fromWidth": "FromWidth",
    "fromHeight": "FromHeight", "scaleStartTime": "ScaleStartTime", "scaleTime": "ScaleTime",
    "fromX": "FromX", "fromY": "FromY", "fromAlignOrg": "FromAlignOrigin",
    "fromAlignScreen": "FromAlignScreen", "moveStartTime": "MoveStartTime",
    "moveTime": "MoveTime", "time": "Time", "duration": "Duration", "value": "Value",
    "text": "Text", "sort": "Sort", "glowColor.rgba": "GlowColor",
    "fxBirthTime": "EffectBirthTime", "fxLetterTime": "EffectLetterTime",
    "fxDecayStartTime": "EffectDecayStartTime", "fxDecayDuration": "EffectDecayDuration",
    "soundID": "Sound", "flags": "Flags",
}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path, help="Read-only public client checkout")
    parser.add_argument("--write", action="store_true", help="Regenerate checked table")
    args = parser.parse_args()
    lock = json.loads((ROOT / "upstream.lock.json").read_text(encoding="utf-8"))
    source = next(item for item in lock["sources"] if item["name"] == "client-reference")
    header = locked_text(args.reference, "src/q_shared.h", source["files"])
    text = locked_text(args.reference, "src/msg.c", source["files"])
    if not re.search(r"typedef struct hudelem_s\s*\{(.*?)\}hudelem_t;", header, re.S):
        raise SystemExit("Expected reviewed native HUD type")
    tables = re.findall(r"static netField_t hudElemFields\[\] =\s*\{(.*?)\n\};", text, re.S)
    if len(tables) != 1:
        raise SystemExit("Expected one reviewed HUD schema")
    fields = re.findall(r"\{ HEF\( (.*?) \), (-?\d+), (\d+)\}", tables[0])
    if len(fields) != 40 or {path for path, _, _ in fields} != set(WORDS):
        raise SystemExit("Expected forty distinct reviewed HUD words")
    lines = ["// SPDX-License-Identifier: AGPL-3.0-or-later",
             "// Generated from hash-locked msg.c by tools/import_hud_table.py.",
             "use super::super::fields::{ValueSpec, v};", "use super::HudWord as W;", "",
             "pub(super) struct FieldSpec {", "    pub word: W,",
             "    pub value: ValueSpec,", "}", "",
             "const fn f(word: W, bits: i16) -> FieldSpec {",
             "    FieldSpec { word, value: v(bits, false) }", "}", "",
             "pub(super) const FIELDS: [FieldSpec; 40] = ["]
    for path, bits, hint in fields:
        if int(bits) not in {-99, -97, -92, -91, -90, -86, -85, 0, 3, 4, 5, 6, 8, 9, 10, 12, 16, 32} or hint != "0":
            raise SystemExit(f"Unreviewed HUD field: {path}")
        lines.append(f"    f(W::{WORDS[path]}, {bits}), // {path}")
    lines.extend(["];", ""])
    result = subprocess.run(["rustfmt", "--edition", "2024", "--emit", "stdout"],
                            input="\n".join(lines), capture_output=True, text=True, check=True)
    if args.write:
        TARGET.write_text(result.stdout, encoding="utf-8", newline="\n")
    if not TARGET.is_file() or TARGET.read_text(encoding="utf-8") != result.stdout:
        raise SystemExit("Generated HUD table mismatch")
    print("Verified forty HUD fields against locked public client sources")


if __name__ == "__main__":
    main()
