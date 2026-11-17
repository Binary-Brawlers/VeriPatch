#!/usr/bin/env python3

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CARGO_TOML = ROOT / "Cargo.toml"
TAURI_CONF = ROOT / "crates/veripatch-app/tauri.conf.json"
FRONTEND_PACKAGE = ROOT / "crates/veripatch-app/frontend/package.json"

SEMVER_RE = re.compile(r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$")


def read_current_version() -> str:
    cargo_text = CARGO_TOML.read_text()
    match = re.search(
        r"(?ms)^\[workspace\.package\]\s+.*?^version = \"([^\"]+)\"$",
        cargo_text,
    )
    if match is None:
        raise SystemExit("Could not find workspace version in Cargo.toml")
    version = match.group(1)
    if SEMVER_RE.fullmatch(version) is None:
        raise SystemExit(f"Workspace version is not a valid semver: {version}")
    return version


def replace_or_die(path: Path, pattern: str, replacement: str) -> None:
    text = path.read_text()
    updated, count = re.subn(pattern, replacement, text, count=1, flags=re.MULTILINE)
    if count != 1:
        raise SystemExit(f"Could not update version in {path}")
    path.write_text(updated)


def write_version(version: str) -> None:
    if SEMVER_RE.fullmatch(version) is None:
        raise SystemExit(f"Invalid semver: {version}")

    replace_or_die(
        CARGO_TOML,
        r"(?ms)(^\[workspace\.package\]\s+.*?^version = \")([^\"]+)(\"$)",
        rf"\g<1>{version}\g<3>",
    )

    replace_or_die(
        TAURI_CONF,
        r'(^\s*"version": ")([^"]+)(",?$)',
        rf'\g<1>{version}\g<3>',
    )

    replace_or_die(
        FRONTEND_PACKAGE,
        r'(^\s*"version": ")([^"]+)(",?$)',
        rf'\g<1>{version}\g<3>',
    )


def bump_version(version: str, bump: str) -> str:
    major, minor, patch = (int(part) for part in version.split("."))
    if bump == "major":
        return f"{major + 1}.0.0"
    if bump == "minor":
        return f"{major}.{minor + 1}.0"
    if bump == "patch":
        return f"{major}.{minor}.{patch + 1}"
    raise SystemExit(f"Unsupported bump type: {bump}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Manage the VeriPatch app version.")
    subparsers = parser.add_subparsers(dest="command", required=True)

    subparsers.add_parser("get", help="Print the current version")

    bump_parser = subparsers.add_parser("bump", help="Bump and persist the version")
    bump_parser.add_argument("bump", choices=("major", "minor", "patch"))

    set_parser = subparsers.add_parser("set", help="Persist an explicit version")
    set_parser.add_argument("version")

    return parser.parse_args()


def main() -> int:
    args = parse_args()

    if args.command == "get":
        print(read_current_version())
        return 0

    if args.command == "set":
        write_version(args.version)
        print(args.version)
        return 0

    if args.command == "bump":
        next_version = bump_version(read_current_version(), args.bump)
        write_version(next_version)
        print(next_version)
        return 0

    raise SystemExit(f"Unknown command: {args.command}")


if __name__ == "__main__":
    sys.exit(main())
