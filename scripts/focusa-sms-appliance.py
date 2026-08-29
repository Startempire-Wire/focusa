#!/usr/bin/env python3
"""Focusa SMS connector checkpoint envelope.

DRY authority for authenticated seal/verify/restore. It never prints key or
plaintext bytes and writes checkpoints atomically with owner-only permissions.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import tempfile

from cryptography.hazmat.primitives.ciphers.aead import AESGCM

MAGIC = b"FOCUSA_SMS_CHECKPOINT_V1\0"
AAD = b"focusa.sms.google_messages.v1"
SCHEMA = "focusa.sms_connector_checkpoint.v1"


def _read_key(path: Path) -> bytes:
    key = path.read_bytes()
    if len(key) != 32:
        raise ValueError("checkpoint key must contain exactly 32 bytes")
    return key


def _atomic_write(path: Path, data: bytes, mode: int = 0o600) -> None:
    path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    fd, name = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    tmp = Path(name)
    try:
        os.fchmod(fd, mode)
        with os.fdopen(fd, "wb") as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(tmp, path)
        directory = os.open(path.parent, os.O_RDONLY | os.O_DIRECTORY)
        try:
            os.fsync(directory)
        finally:
            os.close(directory)
    finally:
        if tmp.exists():
            tmp.unlink()


def seal(plaintext: Path, checkpoint: Path, key_path: Path, metadata: Path, generation: int) -> dict:
    data = plaintext.read_bytes()
    key = _read_key(key_path)
    nonce = os.urandom(12)
    blob = MAGIC + nonce + AESGCM(key).encrypt(nonce, data, AAD)
    _atomic_write(checkpoint, blob)
    receipt = {
        "schema": SCHEMA,
        "connector": "google_messages",
        "generation": generation,
        "status": "verified_pending_restore",
        "ciphertext_sha256": hashlib.sha256(blob).hexdigest(),
        "ciphertext_bytes": len(blob),
    }
    _atomic_write(metadata, (json.dumps(receipt, separators=(",", ":")) + "\n").encode())
    return receipt


def decrypt(checkpoint: Path, key_path: Path) -> bytes:
    blob = checkpoint.read_bytes()
    if not blob.startswith(MAGIC) or len(blob) <= len(MAGIC) + 12 + 16:
        raise ValueError("invalid Focusa SMS checkpoint envelope")
    nonce_offset = len(MAGIC)
    nonce = blob[nonce_offset : nonce_offset + 12]
    return AESGCM(_read_key(key_path)).decrypt(nonce, blob[nonce_offset + 12 :], AAD)


def restore(checkpoint: Path, key_path: Path, output: Path) -> dict:
    data = decrypt(checkpoint, key_path)
    _atomic_write(output, data)
    return {"schema": SCHEMA, "status": "restored", "plaintext_bytes": len(data)}


def verify(checkpoint: Path, key_path: Path) -> dict:
    data = decrypt(checkpoint, key_path)
    return {
        "schema": SCHEMA,
        "status": "verified",
        "plaintext_bytes": len(data),
        "ciphertext_sha256": hashlib.sha256(checkpoint.read_bytes()).hexdigest(),
    }


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser()
    commands = result.add_subparsers(dest="command", required=True)
    seal_cmd = commands.add_parser("seal")
    seal_cmd.add_argument("--plaintext", type=Path, required=True)
    seal_cmd.add_argument("--checkpoint", type=Path, required=True)
    seal_cmd.add_argument("--key", type=Path, required=True)
    seal_cmd.add_argument("--metadata", type=Path, required=True)
    seal_cmd.add_argument("--generation", type=int, required=True)
    for name in ("verify", "restore"):
        command = commands.add_parser(name)
        command.add_argument("--checkpoint", type=Path, required=True)
        command.add_argument("--key", type=Path, required=True)
        if name == "restore":
            command.add_argument("--output", type=Path, required=True)
    return result


def main() -> None:
    args = parser().parse_args()
    if args.command == "seal":
        receipt = seal(args.plaintext, args.checkpoint, args.key, args.metadata, args.generation)
    elif args.command == "restore":
        receipt = restore(args.checkpoint, args.key, args.output)
    else:
        receipt = verify(args.checkpoint, args.key)
    print(json.dumps(receipt, separators=(",", ":")))


if __name__ == "__main__":
    main()
