#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Verify immutable public PC fastfile references without cloning a repository."""
from public_format_sources import verify


def main() -> None:
    count = verify("fastfile-format-sources.json", (
        "src/ZoneLoading/", "src/ZoneCommon/", "src/ZoneWriting/", "src/Common/Zone/",
        "src/Common/Game/IW3/",
    ))
    print(f"Verified {count} immutable public fastfile format references")


if __name__ == "__main__":
    main()
