#!/usr/bin/env python3
"""Restart-durable Focusa communications connector checkpoint authority.

The paired source is never disposed until a restored successor proves semantic
readiness twice. Durable state is authenticated ciphertext plus value-free
metadata; plaintext profiles belong in an owner-only runtime directory.
"""
from __future__ import annotations

import argparse
import contextlib
import fcntl
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import shutil
import signal
import subprocess
import tarfile
import tempfile
import time
from typing import Iterator

from cryptography.hazmat.primitives.ciphers.aead import AESGCM

MAGIC_V1 = b"FOCUSA_SMS_CHECKPOINT_V1\0"
MAGIC_V2 = b"FOCUSA_SMS_CHECKPOINT_V2\0"
MAGIC = MAGIC_V2
AAD = b"focusa.sms.connector.v1"
SCHEMA = "focusa.sms_connector_checkpoint.v1"
STATE_SCHEMA = "focusa.sms_connector_state.v1"
EXCLUDED_NAMES = {"Cache", "Code Cache", "GPUCache", "GrShaderCache", "Crash Reports"}


def _read_key(path: Path) -> bytes:
    _require_regular(path, 0o600)
    key = path.read_bytes()
    if len(key) != 32:
        raise ValueError("checkpoint key must contain exactly 32 bytes")
    return key


def _require_regular(path: Path, maximum_mode: int | None = None) -> None:
    info = path.lstat()
    if not path.is_file() or path.is_symlink():
        raise ValueError(f"unsafe regular file: {path}")
    if info.st_uid != os.geteuid():
        raise ValueError(f"foreign-owned file: {path}")
    if maximum_mode is not None and info.st_mode & 0o777 & ~maximum_mode:
        raise ValueError(f"unsafe file permissions: {path}")


def _secure_dir(path: Path, *, create: bool = False) -> None:
    if create:
        path.mkdir(mode=0o700, parents=True, exist_ok=True)
    info = path.lstat()
    if not path.is_dir() or path.is_symlink() or info.st_uid != os.geteuid():
        raise ValueError(f"unsafe directory: {path}")
    if info.st_mode & 0o077:
        raise ValueError(f"unsafe directory permissions: {path}")


def _atomic_write(path: Path, data: bytes, mode: int = 0o600) -> None:
    _secure_dir(path.parent, create=True)
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


@contextlib.contextmanager
def state_lock(state_dir: Path) -> Iterator[None]:
    _secure_dir(state_dir, create=True)
    lock_path = state_dir / ".checkpoint.lock"
    fd = os.open(lock_path, os.O_CREAT | os.O_RDWR | os.O_NOFOLLOW, 0o600)
    try:
        fcntl.flock(fd, fcntl.LOCK_EX)
        yield
    finally:
        fcntl.flock(fd, fcntl.LOCK_UN)
        os.close(fd)


def seal(plaintext: Path, checkpoint: Path, key_path: Path, metadata: Path, generation: int) -> dict:
    _require_regular(plaintext)
    data = plaintext.read_bytes()
    if generation < 0 or generation >= 2**64:
        raise ValueError("checkpoint generation out of range")
    encoded_generation = generation.to_bytes(8, "big")
    nonce = os.urandom(12)
    blob = MAGIC_V2 + encoded_generation + nonce + AESGCM(_read_key(key_path)).encrypt(nonce, data, AAD + encoded_generation)
    _atomic_write(checkpoint, blob)
    receipt = {
        "schema": SCHEMA,
        "connector_kind": "transport_adapter",
        "generation": generation,
        "status": "verified_pending_restore",
        "ciphertext_sha256": hashlib.sha256(blob).hexdigest(),
        "ciphertext_bytes": len(blob),
    }
    _atomic_write(metadata, (json.dumps(receipt, separators=(",", ":")) + "\n").encode())
    return receipt


def _decrypt_details(checkpoint: Path, key_path: Path) -> tuple[bytes, int | None]:
    _require_regular(checkpoint, 0o600)
    blob = checkpoint.read_bytes()
    if blob.startswith(MAGIC_V2) and len(blob) > len(MAGIC_V2) + 8 + 12 + 16:
        offset = len(MAGIC_V2)
        encoded_generation = blob[offset : offset + 8]
        nonce = blob[offset + 8 : offset + 20]
        plaintext = AESGCM(_read_key(key_path)).decrypt(nonce, blob[offset + 20 :], AAD + encoded_generation)
        return plaintext, int.from_bytes(encoded_generation, "big")
    if blob.startswith(MAGIC_V1) and len(blob) > len(MAGIC_V1) + 12 + 16:
        offset = len(MAGIC_V1)
        nonce = blob[offset : offset + 12]
        return AESGCM(_read_key(key_path)).decrypt(nonce, blob[offset + 12 :], AAD), None
    raise ValueError("invalid Focusa SMS checkpoint envelope")


def decrypt(checkpoint: Path, key_path: Path) -> bytes:
    return _decrypt_details(checkpoint, key_path)[0]


def restore(checkpoint: Path, key_path: Path, output: Path) -> dict:
    data = decrypt(checkpoint, key_path)
    _atomic_write(output, data)
    return {"schema": SCHEMA, "status": "restored", "plaintext_bytes": len(data)}


def verify(checkpoint: Path, key_path: Path) -> dict:
    data, generation = _decrypt_details(checkpoint, key_path)
    return {
        "schema": SCHEMA,
        "status": "verified",
        "generation": generation,
        "plaintext_bytes": len(data),
        "ciphertext_sha256": hashlib.sha256(checkpoint.read_bytes()).hexdigest(),
    }


def _profile_paths(profile: Path) -> list[Path]:
    _secure_dir(profile)
    paths: list[Path] = []
    for root, directories, files in os.walk(profile, topdown=True, followlinks=False):
        base = Path(root)
        directories[:] = sorted(name for name in directories if name not in EXCLUDED_NAMES)
        for name in directories + sorted(files):
            candidate = base / name
            if candidate.is_symlink():
                raise ValueError("profile contains a symlink")
            info = candidate.lstat()
            if info.st_uid != os.geteuid():
                raise ValueError("profile contains foreign-owned state")
            if not (candidate.is_dir() or candidate.is_file()):
                raise ValueError("profile contains unsupported filesystem entry")
            paths.append(candidate)
    return paths


def create_profile_archive(profile: Path, output: Path) -> None:
    paths = _profile_paths(profile)
    _secure_dir(output.parent, create=True)
    with tempfile.TemporaryDirectory(prefix="focusa-sms-archive-", dir=output.parent) as raw:
        temporary = Path(raw)
        tar_path = temporary / "profile.tar"
        compressed = temporary / "profile.tar.zst"
        with tarfile.open(tar_path, "w") as archive:
            for path in paths:
                relative = path.relative_to(profile)
                archive.add(path, arcname=str(relative), recursive=False)
        with compressed.open("wb") as sink:
            subprocess.run(
                ["zstd", "-q", "-T1", "-3", "-c", str(tar_path)],
                check=True,
                stdout=sink,
                close_fds=True,
            )
        _atomic_write(output, compressed.read_bytes())


def _validate_members(archive: tarfile.TarFile) -> None:
    for member in archive.getmembers():
        path = PurePosixPath(member.name)
        if path.is_absolute() or ".." in path.parts or not member.name:
            raise ValueError("archive path traversal rejected")
        if member.issym() or member.islnk() or member.isdev() or member.isfifo():
            raise ValueError("archive special entry rejected")
        if not (member.isfile() or member.isdir()):
            raise ValueError("archive entry type rejected")


def extract_profile_archive(archive_path: Path, output: Path) -> None:
    _require_regular(archive_path, 0o600)
    if output.exists():
        raise ValueError("restore output already exists")
    _secure_dir(output.parent, create=True)
    staging = Path(tempfile.mkdtemp(prefix=f".{output.name}.", dir=output.parent))
    os.chmod(staging, 0o700)
    try:
        with tempfile.NamedTemporaryFile(prefix="focusa-sms-", suffix=".tar") as plain:
            subprocess.run(
                ["zstd", "-q", "-d", "-c", str(archive_path)],
                check=True,
                stdout=plain,
                close_fds=True,
            )
            plain.flush()
            with tarfile.open(plain.name, "r:") as archive:
                _validate_members(archive)
                archive.extractall(staging, filter="data")
        os.replace(staging, output)
    finally:
        if staging.exists():
            shutil.rmtree(staging)


def _state_path(state_dir: Path) -> Path:
    return state_dir / "connector-state.json"


def load_state(state_dir: Path) -> dict:
    path = _state_path(state_dir)
    if not path.exists():
        return {
            "schema": STATE_SCHEMA,
            "status": "unconfigured",
            "current_generation": 0,
            "verified_generation": 0,
            "checkpoint_status": "absent",
            "source_preserved": True,
        }
    _require_regular(path, 0o600)
    value = json.loads(path.read_text(encoding="utf-8"))
    if value.get("schema") != STATE_SCHEMA:
        raise ValueError("unsupported connector state schema")
    return value


def write_state(state_dir: Path, value: dict) -> None:
    allowed = {
        "schema", "status", "current_generation", "verified_generation",
        "checkpoint_status", "ciphertext_sha256", "created_at", "checked_at",
        "restored_at", "failure_class", "source_preserved", "ready_proof_count",
    }
    if set(value) - allowed:
        raise ValueError("connector metadata contains forbidden fields")
    _atomic_write(_state_path(state_dir), (json.dumps(value, separators=(",", ":")) + "\n").encode())


def checkpoint_profile(profile: Path, state_dir: Path, key: Path, retain: int = 3) -> dict:
    with state_lock(state_dir):
        state = load_state(state_dir)
        generation = int(state.get("current_generation", 0)) + 1
        generations = state_dir / "generations"
        _secure_dir(generations, create=True)
        with tempfile.TemporaryDirectory(prefix="focusa-sms-snapshot-", dir=state_dir) as raw:
            archive = Path(raw) / "profile.tar.zst"
            metadata = Path(raw) / "seal.json"
            create_profile_archive(profile, archive)
            target = generations / f"{generation:020d}.tar.zst.aesgcm"
            receipt = seal(archive, target, key, metadata, generation)
            verify(target, key)
        now = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
        source_was_ready = state.get("status") == "ready"
        next_state = {
            "schema": STATE_SCHEMA,
            "status": "ready" if source_was_ready else "checkpointing",
            "current_generation": generation,
            "verified_generation": generation,
            "checkpoint_status": "verified_standby" if source_was_ready else "verified_pending_restore",
            "ciphertext_sha256": receipt["ciphertext_sha256"],
            "created_at": state.get("created_at", now),
            "checked_at": now,
            "source_preserved": True,
            "ready_proof_count": int(state.get("ready_proof_count", 0)) if source_was_ready else 0,
        }
        write_state(state_dir, next_state)
        all_generations = sorted(generations.glob("*.tar.zst.aesgcm"), reverse=True)
        for stale in all_generations[max(retain, 2):]:
            stale.unlink()
        return {**receipt, "source_preserved": True}


def restore_latest_profile(state_dir: Path, key: Path, output: Path) -> dict:
    with state_lock(state_dir):
        state = load_state(state_dir)
        generations = sorted((state_dir / "generations").glob("*.tar.zst.aesgcm"), reverse=True)
        if not generations:
            raise ValueError("no verified connector generation")
        failures = 0
        for checkpoint in generations:
            generation = int(checkpoint.name.split(".", 1)[0])
            try:
                with tempfile.TemporaryDirectory(prefix="focusa-sms-restore-", dir=state_dir) as raw:
                    archive = Path(raw) / "profile.tar.zst"
                    verified = verify(checkpoint, key)
                    if verified.get("generation") not in {None, generation}:
                        raise ValueError("authenticated checkpoint generation mismatch")
                    restore(checkpoint, key, archive)
                    extract_profile_archive(archive, output)
                now = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
                state.update({
                    "status": "restoring",
                    "verified_generation": generation,
                    "checkpoint_status": "restored_pending_ready",
                    "restored_at": now,
                    "checked_at": now,
                    "source_preserved": True,
                    "ready_proof_count": 0,
                })
                if failures:
                    state["failure_class"] = "rolled_back_corrupt_generation"
                else:
                    state.pop("failure_class", None)
                write_state(state_dir, state)
                return {"schema": SCHEMA, "status": "restored_pending_ready", "generation": generation, "rolled_back": failures > 0}
            except Exception:
                failures += 1
                if output.exists():
                    shutil.rmtree(output)
        raise ValueError("all connector generations failed authenticated restore")


def mark_ready(state_dir: Path, generation: int, proof_count: int) -> dict:
    if proof_count < 2:
        raise ValueError("paired_persisted requires two semantic readiness proofs")
    with state_lock(state_dir):
        state = load_state(state_dir)
        if int(state.get("verified_generation", 0)) != generation:
            raise ValueError("ready proof generation mismatch")
        now = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
        state.update({
            "status": "ready",
            "checkpoint_status": "paired_persisted",
            "checked_at": now,
            "source_preserved": True,
            "ready_proof_count": proof_count,
        })
        state.pop("failure_class", None)
        write_state(state_dir, state)
        return {"schema": STATE_SCHEMA, "status": "paired_persisted", "generation": generation, "ready_proof_count": proof_count}


def run_probe(command: list[str], attempts: int = 2, window_seconds: float = 30.0) -> int:
    successes = 0
    deadline = time.monotonic() + max(window_seconds, 0.25)
    while time.monotonic() < deadline and successes < attempts:
        result = subprocess.run(command, stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=min(15.0, max(window_seconds, 0.25)), close_fds=True)
        successes = successes + 1 if result.returncode == 0 else 0
        if successes < attempts:
            time.sleep(0.25)
    return successes


def guarded_handoff(args: argparse.Namespace) -> dict:
    source_pid = int(args.source_pid)
    if source_pid <= 1:
        raise ValueError("invalid source pid")
    os.kill(source_pid, 0)
    standby: subprocess.Popen[bytes] | None = None
    os.kill(source_pid, signal.SIGSTOP)
    try:
        receipt = checkpoint_profile(args.source_profile, args.state_dir, args.key, args.retain)
        restore_latest_profile(args.state_dir, args.key, args.standby_profile)
        launch = [part.format(profile=str(args.standby_profile)) for part in json.loads(args.launch_command_json)]
        standby = subprocess.Popen(launch, stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, close_fds=True)
        proofs = run_probe(json.loads(args.ready_probe_command_json), 2, args.probe_window_seconds)
        if proofs < 2:
            raise RuntimeError("restored connector failed semantic readiness")
        ready = mark_ready(args.state_dir, int(receipt["generation"]), proofs)
        os.kill(source_pid, signal.SIGCONT)
        os.kill(source_pid, signal.SIGTERM)
        return {**ready, "source_disposed_after_successor_ready": True, "standby_pid": standby.pid}
    except Exception:
        if standby is not None and standby.poll() is None:
            standby.terminate()
            with contextlib.suppress(subprocess.TimeoutExpired):
                standby.wait(timeout=5)
            if standby.poll() is None:
                standby.kill()
        if args.standby_profile.exists():
            shutil.rmtree(args.standby_profile)
        os.kill(source_pid, signal.SIGCONT)
        state = load_state(args.state_dir)
        state.update({"status": "degraded", "checkpoint_status": "handoff_rolled_back", "failure_class": "restored_connector_unavailable", "source_preserved": True, "checked_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())})
        write_state(args.state_dir, state)
        raise


def revoke(state_dir: Path, key: Path, confirm: str) -> dict:
    if confirm != "REVOKE":
        raise ValueError("explicit revoke confirmation required")
    with state_lock(state_dir):
        generations = state_dir / "generations"
        if generations.exists():
            shutil.rmtree(generations)
        for path in (_state_path(state_dir), key):
            if path.exists():
                _require_regular(path, 0o600)
                path.unlink()
    return {"schema": STATE_SCHEMA, "status": "revoked", "cryptographic_erasure": True}


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
    snapshot = commands.add_parser("snapshot-profile")
    snapshot.add_argument("--profile", type=Path, required=True)
    snapshot.add_argument("--state-dir", type=Path, required=True)
    snapshot.add_argument("--key", type=Path, required=True)
    snapshot.add_argument("--retain", type=int, default=3)
    latest = commands.add_parser("restore-latest")
    latest.add_argument("--state-dir", type=Path, required=True)
    latest.add_argument("--key", type=Path, required=True)
    latest.add_argument("--output", type=Path, required=True)
    ready = commands.add_parser("mark-ready")
    ready.add_argument("--state-dir", type=Path, required=True)
    ready.add_argument("--generation", type=int, required=True)
    ready.add_argument("--proof-count", type=int, required=True)
    handoff = commands.add_parser("guarded-handoff")
    handoff.add_argument("--source-pid", type=int, required=True)
    handoff.add_argument("--source-profile", type=Path, required=True)
    handoff.add_argument("--standby-profile", type=Path, required=True)
    handoff.add_argument("--state-dir", type=Path, required=True)
    handoff.add_argument("--key", type=Path, required=True)
    handoff.add_argument("--launch-command-json", required=True)
    handoff.add_argument("--ready-probe-command-json", required=True)
    handoff.add_argument("--retain", type=int, default=3)
    handoff.add_argument("--probe-window-seconds", type=float, default=45.0)
    erase = commands.add_parser("revoke")
    erase.add_argument("--state-dir", type=Path, required=True)
    erase.add_argument("--key", type=Path, required=True)
    erase.add_argument("--confirm", required=True)
    return result


def main() -> None:
    args = parser().parse_args()
    if args.command == "seal":
        receipt = seal(args.plaintext, args.checkpoint, args.key, args.metadata, args.generation)
    elif args.command == "restore":
        receipt = restore(args.checkpoint, args.key, args.output)
    elif args.command == "verify":
        receipt = verify(args.checkpoint, args.key)
    elif args.command == "snapshot-profile":
        receipt = checkpoint_profile(args.profile, args.state_dir, args.key, args.retain)
    elif args.command == "restore-latest":
        receipt = restore_latest_profile(args.state_dir, args.key, args.output)
    elif args.command == "mark-ready":
        receipt = mark_ready(args.state_dir, args.generation, args.proof_count)
    elif args.command == "guarded-handoff":
        receipt = guarded_handoff(args)
    else:
        receipt = revoke(args.state_dir, args.key, args.confirm)
    print(json.dumps(receipt, separators=(",", ":")))


if __name__ == "__main__":
    main()
