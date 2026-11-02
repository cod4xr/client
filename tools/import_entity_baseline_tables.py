#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Reproduce reviewed entity field order from the hash-locked public client source."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
TARGET = ROOT / "crates/client-core/src/gamestate/entity_baselines/tables"

# Semantic word aliases reviewed against the locked q_shared.h entity/union types.
# Number is separate; these word keys never depend on the host's C struct ABI.
WORDS = {
    "eType": "Kind", "lerp.eFlags": "Flags",
    "time2": "Time2", "otherEntityNum": "OtherEntity",
    "attackerEntityNum": "AttackerEntity", "groundEntityNum": "GroundEntity",
    "loopSound": "LoopSound", "surfType": "SurfaceType", "index": "Index",
    "clientNum": "Client", "iHeadIcon": "HeadIcon", "iHeadIconTeam": "HeadIconTeam",
    "solid": "Solid", "eventParm": "EventParameter", "eventSequence": "EventSequence",
    "weapon": "Weapon", "weaponModel": "WeaponModel", "legsAnim": "LegsAnimation",
    "torsoAnim": "TorsoAnimation", "un1": "Auxiliary1",
    "un1.helicopterStage": "Auxiliary1", "un2": "Auxiliary2",
    "fTorsoPitch": "TorsoPitch", "fWaistPitch": "WaistPitch",
}
for native, name in [("pos", "Position"), ("apos", "Angle")]:
    for field, suffix in [("trType", "Type"), ("trTime", "Time"), ("trDuration", "Duration")]:
        WORDS[f"lerp.{native}.{field}"] = name + suffix
    for index, axis in enumerate("XYZ"):
        WORDS[f"lerp.{native}.trBase[{index}]"] = name + axis
        WORDS[f"lerp.{native}.trDelta[{index}]"] = name + "Delta" + axis
for index in range(4):
    WORDS[f"events[{index}]"] = f"Event{index}"
    WORDS[f"eventParms[{index}]"] = f"EventParameter{index}"
    WORDS[f"partBits[{index}]"] = f"Part{index}"
for index in range(7):
    WORDS[f"lerp.u.anonymous.data[{index}]"] = f"Union{index}"
for native, fields in {
    "player": ["leanf", "movementDir"],
    "loopFx": ["cullDist", "period"],
    "missile": ["launchTime"],
    "soundBlend": ["lerp"],
    "vehicle": ["bodyPitch", "bodyRoll", "steerYaw", "materialTime", "gunPitch", "gunYaw", "team"],
}.items():
    for index, field in enumerate(fields):
        WORDS[f"lerp.u.{native}.{field}"] = f"Union{index}"


def locked_text(reference: Path, relative: str, hashes: dict) -> str:
    data = (reference / relative).read_bytes().replace(b"\r\n", b"\n")
    if hashlib.sha256(data).hexdigest() != hashes[relative]:
        raise SystemExit(f"Locked source mismatch: {relative}")
    return data.decode("utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reference", type=Path, help="Read-only public client checkout")
    parser.add_argument("--write", action="store_true", help="Regenerate checked client tables")
    args = parser.parse_args()
    lock = json.loads((ROOT / "upstream.lock.json").read_text(encoding="utf-8"))
    source = next(item for item in lock["sources"] if item["name"] == "client-reference")
    locked_text(args.reference, "src/q_shared.h", source["files"])
    text = locked_text(args.reference, "src/msg.c", source["files"])
    tables = re.findall(r"netField_t entityStateFields_(\d+)\[\] =\s*\{(.*?)\n\};", text, re.S)
    if [int(index) for index, _ in tables] != list(range(18)):
        raise SystemExit("Expected exactly eighteen reviewed entity field lists")
    expected = {}
    for start in range(0, 18, 3):
        lines = ["// SPDX-License-Identifier: AGPL-3.0-or-later",
                 "// Generated from hash-locked msg.c by tools/import_entity_baseline_tables.py.",
                 "use super::{FieldSpec, f};", "use crate::gamestate::entity_baselines::BaselineWord as W;", ""]
        for index, body in tables[start:start + 3]:
            fields = re.findall(r"\{ NETF\( (.*?) \), (-?\d+), (\d+)\}", body)
            expected_count = {12: 58, 13: 60}.get(int(index), 59)
            if len(fields) != expected_count or fields[0][:2] != ("eType", "8"):
                raise SystemExit(f"Unexpected entity list shape: {index}")
            lines.append(f"pub(super) const TABLE_{index}: [FieldSpec; {len(fields)}] = [")
            seen = set()
            for path, bits, hint in fields:
                rules = {-100, -98, -97, -96, -94, -93, -92, -91, -90, -8, 0}
                if path not in WORDS or hint not in {"0", "1", "2"} or not (int(bits) in rules or 1 <= int(bits) <= 32):
                    raise SystemExit(f"Unreviewed entity field: {path}")
                if WORDS[path] in seen:
                    raise SystemExit(f"Duplicate semantic entity word: {path}")
                seen.add(WORDS[path])
                lines.append(f"    f(W::{WORDS[path]}, {bits}, {'true' if hint == '2' else 'false'}), // {path}")
            lines.extend(["];", ""])
        # Use the pinned toolchain's formatter so --write/verification match cargo fmt.
        formatted = subprocess.run(["rustfmt", "--edition", "2024", "--emit", "stdout"],
                                   input="\n".join(lines), capture_output=True, text=True, check=True)
        expected[f"group_{start // 3}.rs"] = formatted.stdout
    if args.write:
        TARGET.mkdir(parents=True, exist_ok=True)
        for name, content in expected.items():
            (TARGET / name).write_text(content, encoding="utf-8", newline="\n")
    for name, content in expected.items():
        path = TARGET / name
        if not path.is_file() or path.read_text(encoding="utf-8") != content:
            raise SystemExit(f"Generated entity table mismatch: {path.relative_to(ROOT)}")
    print("Verified eighteen entity field lists against the locked public client sources")


if __name__ == "__main__":
    main()
