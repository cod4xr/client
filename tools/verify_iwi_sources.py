#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Verify bounded immutable public IWI format references without cloning a repository."""
from public_format_sources import verify


def main() -> None:
    count = verify("iwi-format-sources.json", ("src/ObjImage/Image/",))
    print(f"Verified {count} immutable public IWI format references")


if __name__ == "__main__":
    main()
