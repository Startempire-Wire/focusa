#!/usr/bin/env python3
"""Select a monotonic Focusa release version across mixed release channels."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
from collections.abc import Iterable

SEMVER_TAG = re.compile(
    r"^v(?P<major>0|[1-9][0-9]*)\.(?P<minor>0|[1-9][0-9]*)\."
    r"(?P<patch>0|[1-9][0-9]*)(?:-(?P<suffix>[0-9A-Za-z][0-9A-Za-z.-]*))?$"
)
BASE = re.compile(r"^(?P<major>0|[1-9][0-9]*)\.(?P<minor>0|[1-9][0-9]*)$")


def channel_for(suffix: str | None) -> str:
    if suffix is None:
        return "stable"
    prefix = suffix.split(".", 1)[0]
    if prefix == "dev":
        return "dev"
    if prefix == "rc":
        return "rc"
    return "preview"


def discover_git_tags() -> list[str]:
    output = subprocess.check_output(
        ["git", "tag", "--list"], text=True, stderr=subprocess.DEVNULL
    )
    return [tag for tag in output.splitlines() if tag]


def select_version(base: str, exact_tag: str | None, tags: Iterable[str]) -> dict[str, object]:
    base_match = BASE.fullmatch(base)
    if base_match is None:
        raise ValueError(f"invalid base {base!r}; expected MAJOR.MINOR")

    lane = (int(base_match["major"]), int(base_match["minor"]))
    parsed: list[tuple[str, int, str]] = []
    ignored: list[str] = []
    for tag in tags:
        match = SEMVER_TAG.fullmatch(tag)
        if match is None:
            ignored.append(tag)
            continue
        tag_lane = (int(match["major"]), int(match["minor"]))
        if tag_lane != lane:
            continue
        parsed.append((tag, int(match["patch"]), channel_for(match["suffix"])))

    maxima = {
        channel: max((patch for _, patch, found in parsed if found == channel), default=None)
        for channel in ("stable", "dev", "rc", "preview")
    }
    highest_patch = max((patch for _, patch, _ in parsed), default=0)

    if exact_tag:
        selected = SEMVER_TAG.fullmatch(exact_tag)
        if selected is None:
            raise ValueError(f"invalid exact tag {exact_tag!r}; expected semantic vMAJOR.MINOR.PATCH[-SUFFIX]")
        selected_lane = (int(selected["major"]), int(selected["minor"]))
        if selected_lane != lane:
            raise ValueError(f"exact tag {exact_tag!r} is outside requested base {base!r}")
        selected_patch = int(selected["patch"])
        if selected_patch <= highest_patch:
            raise ValueError(
                f"release version regression: selected patch {selected_patch} must be greater "
                f"than existing {base} maximum {highest_patch}"
            )
        selected_tag = exact_tag
        mode = "explicit"
        selected_channel = channel_for(selected["suffix"])
    else:
        selected_patch = highest_patch + 1
        selected_tag = f"v{base}.{selected_patch}-dev"
        mode = "automatic"
        selected_channel = "dev"

    return {
        "schema": "focusa.release_version_selection.v1",
        "status": "completed",
        "base": base,
        "mode": mode,
        "selected_tag": selected_tag,
        "selected_version": selected_tag.removeprefix("v"),
        "selected_patch": selected_patch,
        "selected_channel": selected_channel,
        "highest_patch": highest_patch,
        "channel_maxima": maxima,
        "considered_tags": len(parsed),
        "ignored_malformed_tags": sorted(ignored),
        "monotonic": selected_patch > highest_patch,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--base", required=True)
    parser.add_argument("--tag")
    parser.add_argument("--existing-tag", action="append", default=[])
    parser.add_argument("--use-git-tags", action="store_true")
    args = parser.parse_args()

    tags = list(args.existing_tag)
    if args.use_git_tags:
        tags.extend(discover_git_tags())
    try:
        result = select_version(args.base, args.tag, tags)
    except ValueError as error:
        parser.error(str(error))
    print(json.dumps(result, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
