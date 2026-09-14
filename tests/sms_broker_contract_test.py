#!/usr/bin/env python3
"""Regression checks for the private broker and all public consumers."""
from __future__ import annotations
from datetime import datetime, timedelta, timezone
import json
import os
import runpy
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BROKER = ROOT / "scripts/focusa-google-messages-broker.py"

with tempfile.TemporaryDirectory(prefix="focusa-sms-broker-test-") as raw:
    temp = Path(raw)
    files = {name: temp / name for name in ("token", "grants", "targets", "policy")}
    files["token"].write_text("t" * 64)
    grant = {
        "schema": "focusa.sms_grant.v1", "grant_id": "grant-1", "status": "active",
        "consumer_ref": "consumer-1", "capabilities": ["otp_challenge", "inject_otp", "checkpoint"],
        "scope": {"connector_id": "communications-1", "provider": "github.com", "target_handle": "target-1"},
        "granted_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "expires_at": (datetime.now(timezone.utc) + timedelta(minutes=5)).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "use_count_allowed": 1, "use_count_used": 0,
    }
    scoped_grant = {
        **grant, "grant_id": "grant-scoped", "capabilities": ["read_thread", "send"],
        "scope": {"connector_id": "communications-1", "thread_handles": ["thread-allowed"], "recipient_handles": ["recipient-allowed"]},
        "use_count_allowed": 10,
    }
    files["grants"].write_text(json.dumps({"grants": [grant, scoped_grant]}))
    files["targets"].write_text(json.dumps({"targets": {"target-1": {"origin": "https://github.com", "cdp_url": "http://127.0.0.1:9336", "input_selector": "input"}}}))
    files["policy"].write_text(json.dumps({"providers": {"github.com": {"thread_pattern": "GitHub", "otp_pattern": "(?<!\\d)(\\d{6})(?!\\d)"}}}))
    for path in files.values(): path.chmod(0o600)
    state = temp / "state"
    state.mkdir(mode=0o700)
    names = {"FOCUSA_SMS_BROKER_TOKEN_FILE": "token", "FOCUSA_SMS_GRANTS_FILE": "grants", "FOCUSA_SMS_TARGETS_FILE": "targets", "FOCUSA_SMS_PROVIDER_POLICY_FILE": "policy"}
    previous = {name: os.environ.get(name) for name in names}
    for name, key in names.items(): os.environ[name] = str(files[key])
    os.environ["FOCUSA_SMS_STATE_DIR"] = str(state)
    try:
        module = runpy.run_path(str(BROKER), run_name="focusa_sms_broker_test")
        first = module["opaque"]("thread", "provider-internal-value")
        second = module["opaque"]("thread", "provider-internal-value")
        assert first == second and first.startswith("thread-")
        assert "provider-internal-value" not in first
        module["authorize"]("grant-1", "consumer-1", "inject_otp", target_handle="target-1", provider="github.com")
        module["authorize"]("grant-scoped", "consumer-1", "read_thread", thread_handle="thread-allowed")
        module["authorize"]("grant-scoped", "consumer-1", "send", recipient_handle="recipient-allowed")
        for capability, scope in (("read_thread", {"thread_handle": "thread-wrong"}), ("send", {"recipient_handle": "recipient-wrong"})):
            try:
                module["authorize"]("grant-scoped", "consumer-1", capability, **scope)
            except PermissionError:
                pass
            else:
                raise AssertionError("scoped grant accepted an out-of-scope handle")
        try:
            module["authorize"]("grant-1", "consumer-1", "read_thread")
        except PermissionError:
            pass
        else:
            raise AssertionError("OTP grant widened into thread read")
        module["authorize"]("grant-1", "consumer-1", "inject_otp", target_handle="target-1", provider="github.com", consume=True)
        assert json.loads((state / "grant-usage.json").read_text())["uses"]["grant-1"] == 1
        assert json.loads(files["grants"].read_text())["grants"][0]["use_count_used"] == 0
        try:
            module["authorize"]("grant-1", "consumer-1", "inject_otp", target_handle="target-1", provider="github.com")
        except PermissionError:
            pass
        else:
            raise AssertionError("single-use grant replay accepted")
        receipt = module["envelope"](True, "ok", "bounded")
        assert receipt == {"schema": "focusa.tool_result_v1", "canonical": True, "ok": True, "status": "ok", "summary": "bounded"}
        module["audit"]("health", "ok")
        event = module["EVENTS"][-1]
        assert {"schema", "audit_id", "action", "consumer_ref", "grant_id", "connector_id", "target_handle", "status", "failure_class", "occurred_at"} == set(event)
        assert not any(key in event for key in ("body", "otp", "token", "cookie", "selector"))
        module["audit"]("checkpoint", "ok", consumer_ref="consumer-1", grant_id="grant-1")
        module["audit"]("checkpoint", "ok", consumer_ref="consumer-2", grant_id="grant-2")
        visible = module["audit_events"]("consumer-1", False, 100)
        assert visible and all(item["consumer_ref"] == "consumer-1" for item in visible)
        module["idempotent_send"].__globals__["send_message"] = lambda _recipient, _body: "send-test-receipt"
        sent, replayed = module["idempotent_send"]("grant-1", "consumer-1", "idem-1", "thread-1", "bounded body")
        assert sent == "send-test-receipt" and replayed is False
        sent, replayed = module["idempotent_send"]("grant-1", "consumer-1", "idem-1", "thread-1", "bounded body")
        assert sent == "send-test-receipt" and replayed is True
        try:
            module["idempotent_send"]("grant-1", "consumer-1", "idem-1", "thread-1", "different body")
        except PermissionError:
            pass
        else:
            raise AssertionError("idempotency key accepted a different payload")
    finally:
        for name, value in previous.items():
            if value is None: os.environ.pop(name, None)
            else: os.environ[name] = value
        os.environ.pop("FOCUSA_SMS_STATE_DIR", None)

api = (ROOT / "crates/focusa-api/src/routes/sms.rs").read_text()
assert "sms_broker_url_not_private" in api and "sms_broker_token_permissions_invalid" in api
assert ".content_length()" in api and api.count("1_048_576") >= 2
cli = (ROOT / "crates/focusa-cli/src/commands/sms.rs").read_text()
assert "io::stdin().read_to_string" in cli and "send requires --confirm" in cli
assert '"confirm":"REVOKE"' in cli.replace(" ", "")
pi = (ROOT / "apps/pi-extension/src/sms-tools.ts").read_text()
for name in ("focusa_sms_health", "focusa_sms_enrollment", "focusa_sms_threads", "focusa_sms_read_thread", "focusa_sms_search", "focusa_sms_send", "focusa_sms_otp_challenge", "focusa_sms_otp_inject", "focusa_sms_checkpoint", "focusa_sms_events", "focusa_sms_revoke"):
    assert name in pi, name
assert "The OTP value never enters model context" in pi
assert 'confirm: "REVOKE"' in pi
broker = BROKER.read_text()
for required in ("otp_candidate_ambiguous", "challenge_ineligible", "active_challenge_exists", "injecting", "inject_target", "use_count_used", "send-idempotency.json", "audit.jsonl", "supervisor_request", "checkpoint_generation", "participant_handles", "recipient_handles", "sent_at"):
    assert required in broker
core = (ROOT / "crates/focusa-core/src/sms.rs").read_text()
for required in ("OtpChallenge", "pub grant_id: String", "pub confirm: bool", "pub sent_at: Option<String>", "challenge grant mismatch"):
    assert required in core
print("sms broker contract: passed")
