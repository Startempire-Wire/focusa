#!/usr/bin/env python3
"""Spec98 focusa-877z.10: telemetry/resource/export do not advance cognition version."""

from pathlib import Path
import re
import sys
import yaml

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = (
    ROOT
    / "docs/worksheets/focusa-877z.10-telemetry-resource-export-cognition-version-contract.yaml"
)
SERVER = ROOT / "crates/focusa-api/src/server.rs"
TELEMETRY = ROOT / "crates/focusa-api/src/routes/telemetry.rs"
CAP_EXTRA = ROOT / "crates/focusa-api/src/routes/capabilities_extra.rs"
TYPES = ROOT / "crates/focusa-core/src/types.rs"


def fail(message: str) -> None:
    print(f"✗ FAIL: {message}")
    sys.exit(1)


def fn_body(source: str, name: str) -> str:
    marker = f"fn {name}"
    start = source.find(marker)
    if start == -1:
        marker = f"async fn {name}"
        start = source.find(marker)
    if start == -1:
        fail(f"missing function {name}")
    brace = source.find("{", start)
    depth = 0
    for i in range(brace, len(source)):
        if source[i] == "{":
            depth += 1
        elif source[i] == "}":
            depth -= 1
            if depth == 0:
                return source[start : i + 1]
    fail(f"unterminated function {name}")


def main() -> None:
    contract = yaml.safe_load(CONTRACT.read_text())
    if (
        contract.get("schema_version")
        != "focusa.telemetry_resource_export_cognition_version_contract.v1"
    ):
        fail("unexpected .10 contract schema")
    if "must not advance canonical FocusaState.version" not in contract.get(
        "normative_rule", ""
    ):
        fail("contract must state no canonical version advancement")

    types = TYPES.read_text()
    for mapping in [
        '("telemetry", AuthorityPlane::TelemetryHistory)',
        '("contribution", AuthorityPlane::TelemetryHistory)',
        '("anticipated_context", AuthorityPlane::AdvisoryProjection)',
    ]:
        if mapping not in types:
            fail(f"state plane contract missing {mapping}")

    server = SERVER.read_text()
    prune = fn_body(server, "prune_pressure_sensitive_state")
    if re.search(
        r"focusa\.version\s*=|focusa\.version\.saturating_add|version\s*\+=", prune
    ):
        fail("pressure-sensitive pruning must not modify FocusaState.version")
    if "state.mark_external_mutation()" not in prune:
        fail(
            "pressure-sensitive pruning should still mark external mutation when pruned"
        )
    for field in [
        "focusa.telemetry.trace_events",
        "focusa.telemetry.tool_calls",
        "focusa.telemetry.secondary_loop_ledger",
        "focusa.telemetry.tokens_per_task",
        "focusa.anticipated_context",
    ]:
        if field not in prune:
            fail(f"pressure prune missing bounded buffer {field}")

    telemetry = TELEMETRY.read_text()
    for name in [
        "record_token_budget",
        "record_cache_metadata",
        "record_tool_usage",
        "record_activity_event",
        "record_operational_event",
        "record_trace_batch",
        "record_trace_event",
    ]:
        body = fn_body(telemetry, name)
        if "state.mark_external_mutation()" not in body:
            fail(f"{name} should mark external mutation for daemon/UI wakeup")
        if re.search(
            r"focusa\.version\s*=|focusa\.version\.saturating_add|version\s*\+=", body
        ):
            fail(f"{name} must not modify FocusaState.version")

    cap_extra = CAP_EXTRA.read_text()
    for name in [
        "contribute_status",
        "contribute_policy",
        "contribute_queue",
        "export_history",
        "export_manifest",
    ]:
        body = fn_body(cap_extra, name)
        if "state.focusa.read().await" in body or name.startswith("export_"):
            pass
        else:
            fail(f"{name} must be read-only over Focusa state")
        if "state.focusa.write().await" in body or "mark_external_mutation" in body:
            fail(f"{name} must not mutate state or mark cognition freshness")

    print(
        "✓ PASS: telemetry/resource pruning/export queues do not advance canonical cognition version"
    )


if __name__ == "__main__":
    main()
