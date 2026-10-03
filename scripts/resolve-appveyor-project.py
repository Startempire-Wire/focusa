#!/usr/bin/env python3
"""Resolve the AppVeyor account/project that owns this repository.

The release workflow previously hardcoded ``accountName=verioussmith`` and
``projectSlug=focusa``. With a valid APPVEYOR_API_TOKEN that still answered
``HTTP 404 {"message":"Project not found or access denied."}`` on
2026-10-02, so the queue job failed before AppVeyor ever started a build and
the release stopped at the external receipt gates.

This script asks AppVeyor which project is bound to the repository and prints
``<accountName> <projectSlug>`` for the caller. It prints no credentials: only
the account/project identifiers the build queue needs, or a diagnostic on
stderr with a non-zero exit.
"""

from __future__ import annotations

import json
import sys
import urllib.error
import urllib.request

API = "https://ci.appveyor.com/api/projects"
REPOSITORY_NAME = "focusa"


def fetch(token: str) -> list[dict]:
    request = urllib.request.Request(
        API,
        headers={
            "Authorization": f"Bearer {token}",
            "Accept": "application/json",
        },
    )
    with urllib.request.urlopen(request, timeout=60) as response:
        payload = json.load(response)
    if isinstance(payload, dict):
        return [payload]
    return list(payload)


def aliases(project: dict) -> set[str]:
    """Every lowercase identifier AppVeyor may use for this project."""
    values = {
        project.get("projectName"),
        project.get("slug"),
        project.get("projectSlug"),
        project.get("name"),
    }
    return {str(value).lower() for value in values if value}


def score(project: dict, version_prefix: str) -> int:
    """Lower is better. Exact repository binding wins over name similarity."""
    repository = str(project.get("repositoryName") or "").lower()
    if repository == REPOSITORY_NAME:
        return 0
    names = aliases(project)
    if REPOSITORY_NAME in names or any(REPOSITORY_NAME in name for name in names):
        return 1
    if version_prefix and any(version_prefix in name for name in names):
        return 2
    return 3


def resolve(projects: list[dict], version_prefix: str) -> tuple[str, str]:
    ranked = sorted(projects, key=lambda p: score(p, version_prefix))
    if not ranked or score(ranked[0], version_prefix) >= 3:
        raise LookupError(
            f"No AppVeyor project is bound to repository {REPOSITORY_NAME!r}. "
            "Confirm the GitHub repository is connected in AppVeyor project settings."
        )
    project = ranked[0]
    account = project.get("account")
    account_name = account.get("name") if isinstance(account, dict) else account
    account_name = account_name or project.get("accountName")
    slug = project.get("slug") or project.get("projectSlug")
    if not account_name or not slug:
        raise LookupError("Matched AppVeyor project is missing its account name or slug.")
    return str(account_name), str(slug)


def main(argv: list[str]) -> int:
    token = argv[1] if len(argv) > 1 else ""
    if not token:
        sys.stderr.write("APPVEYOR_API_TOKEN is required.\n")
        return 1
    version_prefix = argv[2].lstrip("v").split(".")[0] if len(argv) > 2 else ""
    try:
        projects = fetch(token)
        account_name, slug = resolve(projects, version_prefix)
    except (urllib.error.URLError, urllib.error.HTTPError) as exc:
        sys.stderr.write(f"AppVeyor project listing failed: {exc}\n")
        return 1
    except LookupError as exc:
        sys.stderr.write(f"{exc}\n")
        return 1
    except json.JSONDecodeError:
        sys.stderr.write("AppVeyor project listing returned invalid JSON.\n")
        return 1
    print(f"{account_name} {slug}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
