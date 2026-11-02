#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Enforce CoD4XR crate boundaries, immutable shared revisions and focused source."""
import json
import os
from pathlib import Path
import re
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[1]


def check_core_tests(packages: dict, failures: list) -> None:
    """Keep the consolidated harness complete as component test files are added."""
    package = packages.get("cod4xr-client-core")
    if package is None:
        return
    manifest = tomllib.loads(Path(package["manifest_path"]).read_text(encoding="utf-8"))
    directory = ROOT / "crates/client-core/tests"
    harness = directory / "core.rs"
    if not harness.is_file():
        failures.append("Core integration harness is missing")
        return
    text = harness.read_text(encoding="utf-8")
    modules = re.findall(r"^mod ([a-z][a-z0-9_]*);$", text, re.M)
    if any(line.strip() and not line.startswith("//")
           and re.fullmatch(r"mod [a-z][a-z0-9_]*;", line) is None
           for line in text.splitlines()):
        failures.append("Core test harness may contain only unconditional component modules")
    expected = {path.stem for path in directory.glob("*.rs") if path != harness}
    expected.update(path.parent.name for path in directory.glob("*_support/mod.rs"))
    if len(modules) != len(set(modules)) or set(modules) != expected:
        failures.append("Core harness must register every component and shared fixture exactly once")
    if list(directory.glob("*/main.rs")):
        failures.append("Core component tests must use registered top-level Rust modules")
    targets = [target for target in package["targets"] if "test" in target["kind"]]
    if (manifest["package"].get("autotests") is not False or len(targets) != 1
            or targets[0]["name"] != "core" or Path(targets[0]["src_path"]) != harness):
        failures.append("Core tests must share the single explicitly registered integration harness")


def main() -> None:
    failures = []
    branch = subprocess.run(["git", "symbolic-ref", "--quiet", "--short", "HEAD"],
                            cwd=ROOT, capture_output=True, text=True)
    if branch.returncode == 0 and branch.stdout.strip() != "main" and not branch.stdout.strip().startswith("feature/"):
        failures.append("Working branch must be main or feature/...")
    elif branch.returncode not in (0, 1):
        failures.append("Cannot verify local branch policy")
    largest = (0, "")
    count = 0
    for directory, children, files in os.walk(ROOT):
        children[:] = [name for name in children if name not in {".git", "target", "__pycache__"}]
        for name in files:
            path = Path(directory) / name
            if path.suffix not in {".rs", ".py", ".ps1"}:
                continue
            length = len(path.read_text(encoding="utf-8").splitlines())
            relative = path.relative_to(ROOT).as_posix()
            largest = max(largest, (length, relative))
            count += 1
            if length > 400:
                failures.append(f"{relative}: exceeds 400 lines")
    rules = json.loads((ROOT / "dependency-rules.json").read_text(encoding="utf-8"))
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--no-deps", "--format-version", "1", "--locked"], cwd=ROOT, text=True))
    packages = {package["name"]: package for package in metadata["packages"]}
    if set(packages) != set(rules):
        failures.append("Crate inventory differs from dependency policy")
    check_core_tests(packages, failures)
    for name, package in packages.items():
        manifest = tomllib.loads(Path(package["manifest_path"]).read_text(encoding="utf-8"))
        for field in ("version", "edition", "rust-version", "license", "publish"):
            if manifest["package"].get(field) != {"workspace": True}:
                failures.append(f"{name}: {field} must inherit workspace metadata")
        if manifest.get("lints", {}).get("workspace") is not True:
            failures.append(f"{name}: workspace lints required")
        for dependency in package["dependencies"]:
            if dependency["name"] not in rules.get(name, []):
                failures.append(f"{name}: unapproved {dependency['name']}")
    workspace = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
    shared_revisions = set()
    for name in ("cod4x-protocol", "cod4x-transport", "cod4x-observability"):
        dependency = workspace["workspace"]["dependencies"][name]
        shared_revisions.add(dependency.get("rev"))
        if dependency.get("git") != "https://github.com/cod4xr/server.git" or not re.fullmatch(r"[0-9a-f]{40}", dependency.get("rev", "")):
            failures.append(f"{name}: explicit immutable shared revision required")
    if len(shared_revisions) != 1:
        failures.append("Shared libraries must use the same reviewed immutable revision")
    if failures:
        raise SystemExit("\n".join(failures))
    print(f"Checked {count} source files and {len(packages)} crate policies; largest {largest[0]} lines: {largest[1]}")


if __name__ == "__main__":
    main()
