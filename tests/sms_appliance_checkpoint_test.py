#!/usr/bin/env python3
import importlib.util
import json
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time
from cryptography.hazmat.primitives.ciphers.aead import AESGCM

ROOT = Path(__file__).resolve().parents[1]
MODULE = ROOT / "scripts" / "focusa-sms-appliance.py"
spec = importlib.util.spec_from_file_location("focusa_sms_appliance", MODULE)
module = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(module)

with tempfile.TemporaryDirectory() as directory:
    root = Path(directory)
    root.chmod(0o700)
    key = root / "key"
    plain = root / "profile.tar.zst"
    checkpoint = root / "profile.tar.zst.aesgcm"
    metadata = root / "seal-state.json"
    restored = root / "restored.tar.zst"
    key.write_bytes(bytes(range(32)))
    key.chmod(0o600)
    plain.write_bytes(b"bounded-profile-fixture\0" * 1000)
    plain.chmod(0o600)

    receipt = module.seal(plain, checkpoint, key, metadata, 7)
    assert receipt["schema"] == module.SCHEMA
    assert receipt["generation"] == 7
    assert receipt["status"] == "verified_pending_restore"
    assert checkpoint.stat().st_mode & 0o777 == 0o600
    assert metadata.stat().st_mode & 0o777 == 0o600
    assert key.read_bytes() not in checkpoint.read_bytes()
    assert module.verify(checkpoint, key)["status"] == "verified"
    assert module.restore(checkpoint, key, restored)["status"] == "restored"
    assert restored.read_bytes() == plain.read_bytes()
    assert module.verify(checkpoint, key)["generation"] == 7
    legacy = root / "legacy-v1.aesgcm"
    legacy_nonce = bytes(range(12))
    legacy.write_bytes(module.MAGIC_V1 + legacy_nonce + AESGCM(key.read_bytes()).encrypt(legacy_nonce, plain.read_bytes(), module.AAD))
    legacy.chmod(0o600)
    assert module.verify(legacy, key)["generation"] is None
    assert module.decrypt(legacy, key) == plain.read_bytes()

    public_metadata = json.loads(metadata.read_text())
    forbidden_fields = {"key", "cookie", "otp", "message", "phone", "account", "pairing"}
    assert forbidden_fields.isdisjoint(public_metadata)
    damaged = bytearray(checkpoint.read_bytes())
    damaged[-1] ^= 1
    checkpoint.write_bytes(damaged)
    try:
        module.verify(checkpoint, key)
    except Exception:
        pass
    else:
        raise AssertionError("corrupt checkpoint passed authenticated verification")

    state = root / "state"
    state.mkdir(mode=0o700)
    profile = root / "profile"
    profile.mkdir(mode=0o700)
    (profile / "Default").mkdir(mode=0o700)
    cookie = profile / "Default" / "Cookies"
    cookie.write_bytes(b"generation-one")
    cookie.chmod(0o600)
    first = module.checkpoint_profile(profile, state, key)
    cookie.write_bytes(b"generation-two")
    second = module.checkpoint_profile(profile, state, key)
    assert second["generation"] == first["generation"] + 1
    generations = sorted((state / "generations").glob("*.aesgcm"), reverse=True)
    latest = bytearray(generations[0].read_bytes())
    latest[-1] ^= 1
    generations[0].write_bytes(latest)
    output = root / "restored-profile"
    restored_receipt = module.restore_latest_profile(state, key, output)
    assert restored_receipt["rolled_back"] is True
    assert (output / "Default" / "Cookies").read_bytes() == b"generation-one"
    try:
        module.mark_ready(state, first["generation"], 1)
    except ValueError:
        pass
    else:
        raise AssertionError("one semantic proof accepted")
    assert module.mark_ready(state, first["generation"], 2)["status"] == "paired_persisted"

    unsafe = root / "unsafe-profile"
    unsafe.mkdir(mode=0o700)
    (unsafe / "escape").symlink_to("/tmp")
    try:
        module.checkpoint_profile(unsafe, state, key)
    except ValueError:
        pass
    else:
        raise AssertionError("profile symlink accepted")

    source = subprocess.Popen(["sleep", "30"])
    standby = root / "standby"
    args = type("Args", (), {
        "source_pid": source.pid,
        "source_profile": profile,
        "standby_profile": standby,
        "state_dir": state,
        "key": key,
        "retain": 3,
        "probe_window_seconds": 0.5,
        "launch_command_json": json.dumps(["sleep", "30"]),
        "ready_probe_command_json": json.dumps(["false"]),
    })()
    try:
        module.guarded_handoff(args)
    except RuntimeError:
        pass
    else:
        raise AssertionError("failed successor handoff accepted")
    time.sleep(0.1)
    assert source.poll() is None
    status = Path(f"/proc/{source.pid}/status").read_text()
    assert "State:\tT" not in status, "paired source remained paused after rollback"
    source.terminate()
    source.wait(timeout=5)

    try:
        module.revoke(state, key, "yes")
    except ValueError:
        pass
    else:
        raise AssertionError("implicit revoke accepted")
    assert module.revoke(state, key, "REVOKE")["cryptographic_erasure"] is True
    assert not key.exists() and not (state / "generations").exists()

print("sms appliance checkpoint: passed")
