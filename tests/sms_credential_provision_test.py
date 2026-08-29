#!/usr/bin/env python3
import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
source = ROOT / "scripts/provision-focusa-sms-appliance-credentials.py"
spec = importlib.util.spec_from_file_location("focusa_sms_credentials", source)
module = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(module)

grant = {
    "schema": "focusa.sms_grant.v1", "grant_id": "grant-test", "status": "active",
    "consumer_ref": "consumer-test", "capabilities": ["otp_challenge", "inject_otp"],
    "scope": {"connector_id": "communications-1", "provider": "github.com", "target_handle": "target-test"},
    "granted_at": "2026-08-29T00:00:00Z", "expires_at": "2099-01-01T00:00:00Z", "use_count_allowed": 1, "use_count_used": 0,
}
target = {"origin": "https://github.com", "cdp_url": "http://127.0.0.1:9336", "input_selector": "input[name=otp]"}
module.validate_grants({"grants": [grant]})
module.validate_targets({"targets": {"target-test": target}})
module.validate_policy({"schema": "focusa.sms_provider_policy.v1", "providers": {"github.com": {"thread_pattern": "^GitHub$", "otp_pattern": "(?<!\\d)(\\d{6})(?!\\d)", "message_class": "renewable_login_otp"}}})
assert len(module.NAMES) == 5
try:
    module.validate_targets({"targets": {"target-test": {**target, "cdp_url": "http://192.0.2.1:9336"}}})
except ValueError:
    pass
else:
    raise AssertionError("non-loopback target CDP accepted")
try:
    module.validate_grants({"grants": [{**grant, "use_count_used": 1}]})
except ValueError:
    pass
else:
    raise AssertionError("exhausted grant accepted")
text = source.read_text()
for marker in ("implicit rotation is forbidden", "systemd-creds", "credential_count", "os.urandom(32)"):
    assert marker in text
for forbidden in ("print(payloads", "print(grants", "print(targets"):
    assert forbidden not in text
print("sms credential provision: passed")
