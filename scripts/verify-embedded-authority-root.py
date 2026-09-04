#!/usr/bin/env python3
"""Fail unless release binaries contain the configured production trust roots."""

from __future__ import annotations

import argparse
import base64
import binascii
import hashlib
import json
import os
from pathlib import Path
import sys

FORBIDDEN_KEY_MARKERS = ("test", "local", "dev")


def configured_roots() -> dict[str, str]:
    raw = os.environ.get("FOCUSA_AUTHORITY_ROOT_KEYS_JSON", "")
    if not raw:
        raise ValueError("FOCUSA_AUTHORITY_ROOT_KEYS_JSON is required")
    try:
        value = json.loads(raw)
    except json.JSONDecodeError as error:
        raise ValueError(f"FOCUSA_AUTHORITY_ROOT_KEYS_JSON is invalid JSON: {error}") from error
    if not isinstance(value, dict) or not value:
        raise ValueError("FOCUSA_AUTHORITY_ROOT_KEYS_JSON must be a non-empty object")
    roots: dict[str, str] = {}
    for key_id, public_key in value.items():
        if not isinstance(key_id, str) or not key_id.strip():
            raise ValueError("authority root key IDs must be non-empty strings")
        lowered = key_id.lower()
        if any(marker in lowered for marker in FORBIDDEN_KEY_MARKERS):
            raise ValueError(f"forbidden non-production authority root key ID: {key_id}")
        if not isinstance(public_key, str) or not public_key.strip():
            raise ValueError(f"authority root {key_id} has no public key")
        try:
            decoded = base64.b64decode(public_key, validate=True)
        except (binascii.Error, ValueError) as error:
            raise ValueError(
                f"authority root {key_id} public key is not valid Base64"
            ) from error
        if len(decoded) != 32:
            raise ValueError(
                f"authority root {key_id} public key must decode to 32 bytes"
            )
        roots[key_id] = public_key
    return roots


def verify_binary(path: Path, roots: dict[str, str]) -> str:
    if not path.is_file() or path.stat().st_size == 0:
        raise ValueError(f"release binary is missing or empty: {path}")
    payload = path.read_bytes()
    for key_id, public_key in roots.items():
        if key_id.encode() not in payload:
            raise ValueError(f"release binary lacks authority root key ID {key_id}: {path}")
        if public_key.encode() not in payload:
            raise ValueError(f"release binary lacks configured authority public key for {key_id}: {path}")
    return hashlib.sha256(payload).hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("binaries", nargs="+", type=Path)
    args = parser.parse_args()
    try:
        roots = configured_roots()
        for binary in args.binaries:
            digest = verify_binary(binary, roots)
            print(f"authority_root_embedding=passed binary={binary} sha256={digest}")
    except ValueError as error:
        print(f"authority_root_embedding=failed error={error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
