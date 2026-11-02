#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Reproduce typed live player-scalar keys and fields from the locked public schema."""
import argparse
import json
from pathlib import Path
import re
import subprocess

from import_entity_baseline_tables import ROOT, locked_text

TARGET = ROOT / "crates/client-core/src/entities/player_scalars"
GROUPS = ("motion", "view", "weapons", "effects", "events", "session")


def formatted(lines: list[str]) -> str:
    return subprocess.run(["rustfmt", "--edition", "2024", "--emit", "stdout"],
                          input="\n".join(lines) + "\n", capture_output=True,
                          text=True, check=True).stdout


def expected(reference: Path) -> dict[str, str]:
    lock = json.loads((ROOT / "upstream.lock.json").read_text(encoding="utf-8"))
    source = next(item for item in lock["sources"] if item["name"] == "client-reference")
    locked_text(reference, "src/q_shared.h", source["files"])
    text = locked_text(reference, "src/msg.c", source["files"])
    tables = re.findall(r"static netField_t playerStateFields\[\] =\s*\{(.*?)\n\};", text, re.S)
    if len(tables) != 1:
        raise SystemExit("Expected one reviewed player-scalar catalog")
    fields = re.findall(r"\{ PSF\( (.*?) \), (-?\d+), (\d+)\}", tables[0])
    manifest = json.loads((ROOT / "schemas/player-scalar-words.json").read_text(encoding="utf-8"))
    if len(fields) != 141 or len(manifest) != 141:
        raise SystemExit("Expected 141 reviewed player-scalar words")
    if [name for name, _, _ in fields] != [name for name, _, _ in manifest]:
        raise SystemExit("Scalar manifest differs from reviewed field order")
    if len({name for name, _, _ in manifest}) != 141 or len({(g, n) for _, g, n in manifest}) != 141:
        raise SystemExit("Scalar keys must be distinct")
    for _, group, key in manifest:
        if group not in GROUPS or re.fullmatch(r"[A-Z][A-Z0-9_]*", key) is None:
            raise SystemExit("Unreviewed scalar namespace/key")
    header = ["// SPDX-License-Identifier: AGPL-3.0-or-later",
              "// Generated from hash-locked msg.c by tools/import_player_scalar_table.py."]
    result = {}
    for group in GROUPS:
        lines = header + [f"//! Native {group} keys; values remain inert until typed interpretation.",
                          "use super::PlayerWord;", ""]
        for index, (native, namespace, key) in enumerate(manifest):
            if namespace == group:
                lines.extend([f"/// Key for the native `{native}` word.",
                              f"pub const {key}: PlayerWord = PlayerWord({index});", ""])
        result[f"word/{group}.rs"] = formatted(lines)
    lines = header + ["//! Closed semantic keys; catalog IDs are not C offsets or engine IDs.", ""]
    for group in GROUPS:
        lines.extend([f"pub mod {group};"])
    lines.extend(["", "/// A reviewed native scalar field key with no arbitrary numeric constructor.",
                  "#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]", "pub struct PlayerWord(u8);", "",
                  "impl PlayerWord {", "    pub(super) fn index(self) -> usize { usize::from(self.0) }", "}", "",
                  "/// All reviewed keys in native field-catalog order; not engine memory order.",
                  "pub const ALL: [PlayerWord; 141] = ["])
    lines.extend(f"    {group}::{key}," for _, group, key in manifest)
    lines.append("];")
    result["word/mod.rs"] = formatted(lines)
    lines = header + ["use super::super::fields::{ValueSpec, v};",
                      "use super::word::{self, PlayerWord};", "",
                      "pub(super) struct FieldSpec {", "    pub word: PlayerWord,",
                      "    pub value: ValueSpec,", "    pub zero_when_origins_sent: bool,", "}", "",
                      "const fn f(word: PlayerWord, bits: i16, hint: u8) -> FieldSpec {",
                      "    FieldSpec { word, value: v(bits, hint == 2), zero_when_origins_sent: hint == 3 }",
                      "}", "", "pub(super) const FIELDS: [FieldSpec; 141] = ["]
    for (native, bits, hint), (_, group, key) in zip(fields, manifest, strict=True):
        if not (1 <= int(bits) <= 32 or int(bits) in {-100, -98, -97, -88, -87, -16, -8, 0}) or hint not in {"0", "1", "2", "3"}:
            raise SystemExit(f"Unreviewed scalar rule: {native}")
        lines.append(f"    f(word::{group}::{key}, {bits}, {hint}),")
    lines.append("];")
    result["table.rs"] = formatted(lines)
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path, help="Read-only public client checkout")
    parser.add_argument("--write", action="store_true", help="Regenerate reviewed scalar catalog/keys")
    args = parser.parse_args()
    files = expected(args.reference)
    if args.write:
        for name, content in files.items():
            path = TARGET / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8", newline="\n")
    for name, content in files.items():
        path = TARGET / name
        if not path.is_file() or path.read_text(encoding="utf-8") != content:
            raise SystemExit(f"Generated scalar catalog/key mismatch: {name}")
    print("Verified 141 live player-scalar fields and six typed key namespaces")


if __name__ == "__main__":
    main()
