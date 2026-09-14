#!/usr/bin/env python3
"""Build-independent Spec 152C authority-client protocol matrix gate."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MATRIX = json.loads(
    (ROOT / "docs/contracts/spec152-authority-client-protocol-case-matrix.v1.json").read_text()
)
CLIENT = (ROOT / "crates/focusa-license/src/authority_client.rs").read_text()
HTTP = (ROOT / "crates/focusa-license/src/authority_http.rs").read_text()
CREDENTIALS = (ROOT / "crates/focusa-license/src/authority_credentials.rs").read_text()

expected_cases = {
    "evaluation",
    "paid",
    "bundle",
    "pending",
    "slow_down",
    "expiry",
    "wrong_product",
    "revoke",
    "refund",
    "node_limit",
    "outage",
}
cases = {case["case"] for case in MATRIX["cases"]}
assert cases == expected_cases, f"protocol case drift: {sorted(expected_cases ^ cases)}"
assert len(MATRIX["cases"]) == len(expected_cases), "duplicate protocol case"

for case in MATRIX["cases"]:
    transcript = json.dumps(
        {"request": case["request_transcript"], "response": case["response_transcript"]},
        sort_keys=True,
    )
    for forbidden in ("device-secret", "refresh-secret", "signed-lease-secret", "@example"):
        assert forbidden not in transcript, f"raw credential in {case['case']} transcript"
    for key, value in case["request_transcript"].items():
        if "credential" in key or "code" in key:
            assert value == "[REDACTED]", f"unredacted request field {case['case']}:{key}"
    for key, value in case["response_transcript"].items():
        if "credential" in key or "signed_lease" in key:
            assert value == "[REDACTED]", f"unredacted response field {case['case']}:{key}"

assert '.redirect(reqwest::redirect::Policy::none())' in HTTP
assert '.header("Idempotency-Key", request_id.to_string())' in HTTP
assert '.header("X-Request-Id", request_id.to_string())' in HTTP
assert '.header("X-Focusa-Operation", operation)' in HTTP
assert "read_bounded_response(response, self.policy.max_response_bytes)" in HTTP
bounded_reader = HTTP.split("async fn read_bounded_response", 1)[1].split("fn authority_rejection", 1)[0]
assert ".content_length()" in bounded_reader
assert ".chunk()" in bounded_reader
assert "checked_add(chunk.len())" in bounded_reader
assert ".bytes()" not in bounded_reader
assert "RequestCorrelationMismatch" in HTTP
assert "AUTHORITY_HTTP_STATUS_{status}" in HTTP
assert ".map(|value| value.min(60_000))" in HTTP
assert 'field("device_code", &"[REDACTED]")' in HTTP
for code in (
    "AUTHORIZATION_PENDING",
    "SLOW_DOWN",
    "AUTHORIZATION_EXPIRED",
    "WRONG_PRODUCT",
    "LEASE_REVOKED",
    "LICENSE_REFUNDED",
    "NODE_LIMIT_EXHAUSTED",
    "AUTHORITY_UNAVAILABLE",
):
    assert f'"{code}"' in HTTP, f"missing typed authority disposition: {code}"
assert "Ok(DeviceCodePollResponse::AuthorizationPending)" in HTTP
assert "Ok(DeviceCodePollResponse::SlowDown)" in HTTP
assert "Ok(DeviceCodePollResponse::Expired)" in HTTP
assert "Ok(DeviceCodePollResponse::Denied { reason_code: code })" in HTTP

assert "max_polls" in CLIENT and "PollBudgetExhausted" in CLIENT
assert "challenge.expires_at_unix_ms" in CLIENT
assert "DeviceCodePollResponse::AuthorizationPending" in CLIENT
assert "DeviceCodePollResponse::SlowDown" in CLIENT
assert "base_interval + 5_000" in CLIENT
assert 'formatter.write_str("SensitiveCredential([REDACTED])")' in CLIENT
assert 'formatter.write_str("[REDACTED]")' in CLIENT
assert "persist_eval_license" not in CLIENT
assert "persist_eval_license" not in HTTP
assert "refresh_credential" in CREDENTIALS and "expose_for_protected_store" in CREDENTIALS

print("Spec152 authority client protocol: PASS (11 cases; bounded transport; redacted transcripts)")
