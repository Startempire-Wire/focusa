#!/usr/bin/env python3
"""Fail-closed installed-runtime probe for the canonical CallGraph validator."""

from __future__ import annotations

import argparse
import json
import os
import sys
import urllib.error
import urllib.request

GOLDEN_GRAPH = {
    "schema": "focusa.callgraph.v1",
    "graph_id": "install-probe",
    "revision": 1,
    "scope": {"project_root": "/runtime-probe", "continuity_id": "install-probe"},
    "mission_ref": "install-probe",
    "title": "Installed CallGraph validator probe",
    "description": "Proves the installed daemon exposes canonical validation.",
    "entry_frame_ids": ["probe"],
    "frames": [
        {
            "frame_id": "probe",
            "name": "probe",
            "purpose": "Validate one side-effect-free frame.",
            "kind": "agent",
            "input_schema": {},
            "return_schema": {},
            "preconditions": [],
            "postconditions": [],
            "side_effect_class": "none",
            "capability_refs": [],
            "acceptance": {"acceptance_atoms": ["probe-valid"], "verifier": None},
        }
    ],
    "edges": [],
    "policies": {},
    "required_evidence": [],
    "created_at": "install-probe",
    "created_by": {"authority_kind": "operator", "reference": "install-probe"},
}


class ProbeError(RuntimeError):
    pass


def probe(url: str, token: str = "", timeout: float = 5.0) -> dict:
    headers = {"Content-Type": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    request = urllib.request.Request(
        url,
        data=json.dumps(GOLDEN_GRAPH, separators=(",", ":")).encode("utf-8"),
        headers=headers,
        method="POST",
    )
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            status = response.status
            body = response.read()
    except urllib.error.HTTPError as error:
        raise ProbeError(f"validator returned HTTP {error.code}") from error
    except (urllib.error.URLError, TimeoutError, OSError) as error:
        raise ProbeError(f"validator transport failed: {type(error).__name__}") from error
    if status != 200:
        raise ProbeError(f"validator returned HTTP {status}")
    try:
        payload = json.loads(body)
    except (json.JSONDecodeError, UnicodeDecodeError) as error:
        raise ProbeError("validator returned non-JSON output") from error
    if (
        payload.get("canonical") is not True
        or payload.get("valid") is not True
        or payload.get("status") != "valid"
        or payload.get("issues") != []
    ):
        raise ProbeError("validator returned a non-canonical or invalid envelope")
    return payload


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--url", required=True)
    parser.add_argument("--timeout", type=float, default=5.0)
    args = parser.parse_args()
    token = os.environ.get("FOCUSA_AGENT_TOKEN", "")
    try:
        payload = probe(args.url, token=token, timeout=args.timeout)
    except ProbeError as error:
        print(f"CallGraph validator probe failed: {error}", file=sys.stderr)
        return 1
    print(json.dumps({"status": "valid", "canonical": True, "graph_id": payload.get("graph_id")}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
