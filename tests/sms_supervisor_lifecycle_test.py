#!/usr/bin/env python3
import importlib.util
import json
import os
from pathlib import Path
import shutil
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
MODULE = ROOT / "scripts" / "focusa-sms-supervisor.py"
spec = importlib.util.spec_from_file_location("focusa_sms_supervisor", MODULE)
module = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(module)

with tempfile.TemporaryDirectory(prefix="focusa-sms-supervisor-test-") as raw:
    root = Path(raw); root.chmod(0o700)
    state = root / "state"; state.mkdir(mode=0o700)
    runtime = root / "runtime"; runtime.mkdir(mode=0o700)
    key = root / "key"; key.write_bytes(os.urandom(32)); key.chmod(0o600)
    connector = ROOT / "tests/fixtures/sms_fake_connector.py"
    probe = ROOT / "tests/fixtures/sms_fake_probe.py"
    environment = {
        "FOCUSA_SMS_STATE_DIR": str(state),
        "FOCUSA_SMS_RUNTIME_DIR": str(runtime),
        "FOCUSA_SMS_CHECKPOINT_KEY_FILE": str(key),
        "FOCUSA_SMS_REQUIRE_TMPFS": "0",
        "FOCUSA_SMS_ACTIVE_CDP_PORT": "19334",
        "FOCUSA_SMS_STANDBY_CDP_PORT": "19335",
        "FOCUSA_SMS_BROWSER_COMMAND_JSON": json.dumps([str(connector), "--profile", "{profile}", "--port", "{cdp_port}"]),
        "FOCUSA_SMS_READY_PROBE_COMMAND_JSON": json.dumps([str(probe), "--profile", "{profile}", "--port", "{cdp_port}"]),
        "FOCUSA_SMS_BROKER_COMMAND_JSON": json.dumps(["sleep", "60"]),
        "FOCUSA_SMS_CHECKPOINT_INTERVAL_SECONDS": "60",
        "FOCUSA_SMS_PROBE_WINDOW_SECONDS": "5",
    }
    previous = {name: os.environ.get(name) for name in environment}
    os.environ.update(environment)
    first = module.Supervisor()
    try:
        first.start()
        assert first.enrolling is True
        deadline = time.monotonic() + 5
        while not (first.profile / "connector.ready").exists() and time.monotonic() < deadline:
            time.sleep(0.05)
        source_pid = first.browser.pid
        assert first.commit_enrollment() is True
        assert first.enrolling is False and first.browser.pid != source_pid
        assert module.appliance.load_state(state)["checkpoint_status"] == "paired_persisted"
        assert not Path(f"/proc/{source_pid}").exists()
        first.checkpoint()
        generation = module.appliance.load_state(state)["current_generation"]
        assert generation >= 2
        first.stop()

        shutil.rmtree(runtime)
        runtime.mkdir(mode=0o700)
        second = module.Supervisor()
        second.start()
        restored = module.appliance.load_state(state)
        assert restored["status"] == "ready"
        assert restored["ready_proof_count"] >= 2
        assert second.profile.exists() and (second.profile / "Default" / "Cookies").exists()
        second.stop()
    finally:
        first.stop()
        if "second" in locals(): second.stop()
        for name, value in previous.items():
            if value is None: os.environ.pop(name, None)
            else: os.environ[name] = value

print("sms supervisor lifecycle: passed")
