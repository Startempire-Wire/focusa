#!/usr/bin/env python3
import importlib.util
import json
from pathlib import Path
import tempfile

ROOT = Path(__file__).resolve().parents[1]
MODULE = ROOT / "scripts" / "focusa-sms-appliance.py"
spec = importlib.util.spec_from_file_location("focusa_sms_appliance", MODULE)
module = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(module)

with tempfile.TemporaryDirectory() as directory:
    root = Path(directory)
    key = root / "key"
    plain = root / "profile.tar.zst"
    checkpoint = root / "profile.tar.zst.aesgcm"
    metadata = root / "connector-state.json"
    restored = root / "restored.tar.zst"
    key.write_bytes(bytes(range(32)))
    plain.write_bytes((b"bounded-profile-fixture\0" * 1000))

    receipt = module.seal(plain, checkpoint, key, metadata, 7)
    assert receipt["schema"] == module.SCHEMA
    assert receipt["generation"] == 7
    assert receipt["status"] == "verified_pending_restore"
    assert checkpoint.stat().st_mode & 0o777 == 0o600
    assert metadata.stat().st_mode & 0o777 == 0o600
    assert key.read_bytes() not in checkpoint.read_bytes()

    verified = module.verify(checkpoint, key)
    assert verified["status"] == "verified"
    restored_receipt = module.restore(checkpoint, key, restored)
    assert restored_receipt["status"] == "restored"
    assert restored.read_bytes() == plain.read_bytes()

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

print("sms appliance checkpoint: passed")
