#!/usr/bin/env python3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
appliance = (ROOT / "scripts/focusa-sms-appliance.py").read_text()
supervisor = (ROOT / "scripts/focusa-sms-supervisor.py").read_text()
broker = (ROOT / "scripts/focusa-google-messages-broker.py").read_text()
probe = (ROOT / "scripts/focusa-sms-ready-probe.mjs").read_text()
unit = (ROOT / "templates/systemd/focusa-sms-appliance.service").read_text()
policy = (ROOT / "templates/focusa-sms-provider-policy.example.json").read_text()
installer = (ROOT / "scripts/install-focusa-sms-appliance-service.sh").read_text()

for required in (
    "AESGCM", "os.replace", "os.fsync", "verified_standby", "restore_latest_profile",
    "rolled_back_corrupt_generation", "SIGSTOP", "SIGCONT", "source_preserved",
    'confirm != "REVOKE"', "archive path traversal rejected", "profile contains a symlink",
):
    assert required in appliance, required

assert supervisor.index("proofs = self.probe(self.standby_port, 2, standby_profile)") < supervisor.index("terminate(source)")
for required in (
    "restore_latest_profile", "boot restore failed semantic readiness", "automatic recovery failed semantic readiness",
    "SIGUSR1", "SIGUSR2", "WATCHDOG=1", "FOCUSA_SMS_REQUIRE_TMPFS", "checkpoint_requested",
):
    assert required in supervisor, required

for required in (
    "origin_ok", "conversations", "unable", "list_ready", "list_probe_ok", "Target.attachToTarget",
):
    assert required in probe, required

for required in (
    "grant-usage.json", "send-idempotency.json", "audit.jsonl", "challenge_ineligible", "active_challenge_exists", "otp_candidate_ambiguous", "inject_target",
    "Runtime.callFunctionOn", "use_count_used", "explicit_revoke_confirmation_required",
):
    assert required in broker, required

for required in (
    "Restart=always", "WatchdogSec=30s", "UMask=0077", "NoNewPrivileges=true",
    "PrivateTmp=true", "ProtectSystem=strict", "RuntimeDirectoryMode=0700", "StateDirectoryMode=0700",
    "LoadCredentialEncrypted=sms-checkpoint-key", "LoadCredentialEncrypted=sms-broker-token",
    "LoadCredentialEncrypted=sms-grants", "LoadCredentialEncrypted=sms-targets", "LoadCredentialEncrypted=sms-provider-policy",
):
    assert required in unit, required
assert "Restart=no" not in unit
assert "FOCUSA_SMS_PROVIDER_POLICY_FILE=%d/sms-provider-policy" in unit
assert "renewable_login_otp" in policy and "github.com" in policy
assert "/tmp/" not in unit
assert "focusa-sms-ready-probe.mjs" in installer
assert "provision-focusa-sms-appliance-credentials.py" in installer
assert "systemctl enable --now focusa-sms-appliance.service" in installer
print("sms pair-once lifecycle static: passed")
