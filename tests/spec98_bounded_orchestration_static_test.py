#!/usr/bin/env python3
"""Spec98 focusa-877z.13: bounded orchestration is separate from cognition authority."""

from pathlib import Path
import re
import sys
import yaml

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs/worksheets/focusa-877z.13-bounded-orchestration-contract.yaml"
TYPES = ROOT / "crates/focusa-core/src/types.rs"
WORK_LOOP = ROOT / "crates/focusa-api/src/routes/work_loop.rs"
CAP_EXTRA = ROOT / "crates/focusa-api/src/routes/capabilities_extra.rs"
TOOLS = ROOT / "apps/pi-extension/src/tools.ts"
SILENT_DOC = ROOT / "docs/focusa-tools/tools/focusa_silent_sessions.md"
SILENT_ROUTES = ROOT / "crates/focusa-api/src/routes"


def fail(message: str) -> None:
    print(f"✗ FAIL: {message}")
    sys.exit(1)


def fn_body(source: str, name: str) -> str:
    for marker in [f"fn {name}", f"async fn {name}", f"pub fn {name}"]:
        start = source.find(marker)
        if start != -1:
            break
    else:
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


def check_silent_sessions(tools: str, doc: str, owners: dict[str, str]) -> None:
    """Spec133 owns process/lifecycle authority; the Pi tool is only its client."""
    anchor = tools.find('name: "focusa_silent_sessions"')
    start = tools.rfind('pi.registerTool({', 0, anchor)
    end = tools.find('pi.registerTool({', anchor)
    if anchor < 0 or start < 0 or end < anchor:
        fail("Spec133 SilentSession bounded client registration missing")
    client = tools[start:end]
    for needle in ["focusaFetchDetailed", "const requireSession", "encodeURIComponent(sessionId)",
                   "const exactMutation", "approval_id", "idempotency_key", "run_id", "generation",
                   'kill: "cancel"', 'start: "start"', 'pause: "pause"', 'resume: "resume"',
                   "never grants authority", "daemon policy decides force", "never executed as a shell command"]:
        if needle not in client:
            fail(f"Spec133 SilentSession client missing exact transport/authority contract {needle}")
    mutation = client[client.index('const exactMutation'):client.index('let result:')]
    for field in ["run_id", "generation", "approval_id", "idempotency_key"]:
        if f"!p.{field}" not in mutation or f"{field}: p.{field}" not in mutation:
            fail(f"Spec133 SilentSession client missing exact mutation guard {field}")
    calls = list(re.finditer(r"exactMutation\(\)", client))
    if len(calls) != 2 or any(call.start() < client.index('action === "send"') for call in calls):
        fail("Spec133 SilentSession reads/previews must not require mutation approval")
    for retired in ["force !== true", "approved !== true", "tmux_", "execSync(", "spawn("]:
        if retired in client:
            fail(f"Spec133 SilentSession client resurrected local process/approval authority {retired}")
    for action in ["list", "capabilities", "preflight", "health", "tail", "watch", "receipt", "config"]:
        if f'"{action}"' not in client:
            fail(f"Spec133 SilentSession observation/recovery client action missing {action}")

    for name, function, needles in [
        ("silent_sessions_create.rs", "create", ["durable_request_principal", "resolve_silent_session_config",
            "authorize_config(", "idempotency_key", "require_silent_session_north_star_admission("]),
        ("silent_sessions_lifecycle.rs", "start", ["durable_request_principal", "guard_exact_target(",
            "load_durable_approval(", "authorize_start(", "reduce_lifecycle(",
            "require_silent_session_north_star_admission(", "append_reducer_event_and_project("]),
        ("silent_sessions_lifecycle.rs", "authorize_start", ["request_principal.principal",
            "controller || administrator", "project_root:", "continuity_id:", "owner_os_user:",
            "config_hash:", "model_binding:", "let workspace = config", "workspace,",
            "authorize_silent_session_action(", "approval_durably_verified: true", "legacy_approved: false"]),
        ("silent_sessions_control.rs", "control", ["durable_request_principal", "guard_exact_target(",
            "session.current_run_generation != body.generation", "load_durable_approval(",
            "authorize_mutation(", "ensure_silent_session_temporal_guard(", "reduce_lifecycle(",
            "idempotency_key_reused", "append_reducer_event_and_project("]),
        ("silent_sessions_authorize.rs", "authorize_mutation", ["request_principal.principal",
            "controller || administrator", "project_root:", "continuity_id:", "owner_os_user:",
            "config_hash:", "model_binding:", "workspace:", "authorize_silent_session_action("]),
    ]:
        body = fn_body(owners[name], function)
        for needle in needles:
            if needle not in body:
                fail(f"Spec133 SilentSession daemon owner {name}:{function} missing {needle}")
    control = fn_body(owners["silent_sessions_control.rs"], "control")
    if control.count("require_silent_session_north_star_admission(") != 1 or not re.search(
        r"if matches!\(kind, ControlKind::Resume\)\s*\{\s*if let Err\(response\) = require_silent_session_north_star_admission\(", control
    ):
        fail("Spec133 SilentSession recovery controls must remain available without new-work admission")
    for name, function in [("silent_sessions.rs", "list"), ("silent_sessions.rs", "status"),
                           ("silent_sessions_create.rs", "preflight")]:
        if "require_silent_session_north_star_admission(" in fn_body(owners[name], function):
            fail(f"Spec133 SilentSession read/preflight was incorrectly admission-gated {function}")
    routes = owners["silent_sessions.rs"]
    for handler in ["get(list)", "get(status)", "post(super::silent_sessions_create::preflight)",
                    "post(super::silent_sessions_lifecycle::start)", "post(super::silent_sessions_control::resume)",
                    "post(super::silent_sessions_control::pause)", "post(super::silent_sessions_control::interrupt)",
                    "post(super::silent_sessions_control::cancel)"]:
        if handler not in routes:
            fail(f"Spec133 SilentSession registered daemon route missing {handler}")
    for needle in ["Daemon-native Spec133", "exact daemon North Star admission",
                   "pause, interrupt, cancel, and observation remain available for recovery",
                   "process_control_failed", "receipt-backed recovery", "`session_id`", "`run_id`",
                   "`generation`", "`approval_id`", "`idempotency_key`", "never grants authority",
                   "never executed as a shell command", "daemon policy decides force",
                   "Source capability metadata is not installed support or action admission"]:
        if needle not in doc:
            fail(f"Spec133 SilentSession doc missing canonical authority/recovery contract {needle}")


def main() -> None:
    data = yaml.safe_load(CONTRACT.read_text())
    if data.get("schema_version") != "focusa.bounded_orchestration_contract.v1":
        fail("unexpected .13 contract schema")
    rule = data.get("normative_rule", "")
    for phrase in [
        "bounded orchestration surfaces",
        "writer ownership",
        "operator controls",
        "do not become Focus State authority",
    ]:
        if phrase not in rule:
            fail(f"contract normative rule missing {phrase}")

    types = TYPES.read_text()
    for needle in [
        "pub enum OrchestrationSurface",
        "WorkLoop",
        "AutonomyCalibration",
        "SilentSession",
        "pub const BOUNDED_ORCHESTRATION_CONTRACT",
    ]:
        if needle not in types:
            fail(f"types.rs missing orchestration contract needle {needle}")
    plane_text = types[
        types.find("pub const FOCUSA_STATE_PLANE_CONTRACT") : types.find(
            "/// The complete cognitive state"
        )
    ]
    for mapping in [
        '("work_loop", AuthorityPlane::BoundedOrchestration)',
        '("autonomy", AuthorityPlane::BoundedOrchestration)',
    ]:
        if mapping not in plane_text:
            fail(f"state plane contract missing {mapping}")
    for mapping in [
        '("work_loop", OrchestrationSurface::WorkLoop)',
        '("autonomy", OrchestrationSurface::AutonomyCalibration)',
        '("silent_session", OrchestrationSurface::SilentSession)',
    ]:
        if mapping not in types:
            fail(f"bounded orchestration contract missing {mapping}")

    work_loop = WORK_LOOP.read_text()
    helper = fn_body(work_loop, "bounded_orchestration_authority_payload")
    for needle in [
        '"authority_plane": "bounded_orchestration"',
        '"canonical": false',
        '"focus_state_authority": false',
        '"writer_ownership_required": true',
        '"operator_controls"',
        '"pause"',
        '"resume"',
        '"stop"',
        '"preflight"',
    ]:
        if needle not in helper:
            fail(f"work-loop authority helper missing {needle}")
    for needle in [
        "ensure_writer_claim",
        "recovery::control",
        "ensure_claimed_writer_matches_for_context",
        "x-focusa-writer-id",
        "x-focusa-approval",
    ]:
        if needle not in work_loop:
            fail(f"work-loop writer/operator guard missing {needle}")
    recovery = WORK_LOOP.with_name("work_loop_recovery.rs").read_text()
    for needle in [
        "require_authoritative_claim_key", "validate_writer_for_key",
        "validate_recovery_partition", "state.writer_claims.write().await",
    ]:
        if needle not in recovery:
            fail(f"work-loop recovery lost canonical writer guard {needle}")
    if recovery.index("dispatch.await?;") > recovery.index("claims.remove(key)"):
        fail("stop must not remove its claim before successful dispatch")
    if work_loop.count('"authority": bounded_orchestration_authority_payload()') < 3:
        fail(
            "work-loop health/summary/deep status payloads must expose authority metadata"
        )

    cap_extra = CAP_EXTRA.read_text()
    for name in ["autonomy_status", "autonomy_ledger", "autonomy_explain"]:
        body = fn_body(cap_extra, name)
        for needle in [
            '"authority_plane": "bounded_orchestration"',
            '"canonical": false',
            '"focus_state_authority": false',
        ]:
            if needle not in body:
                fail(f"{name} missing bounded orchestration marker {needle}")
        if "state.focusa.write().await" in body:
            fail(f"{name} must not mutate Focus State authority")

    check_silent_sessions(TOOLS.read_text(), SILENT_DOC.read_text(), {
        name: (SILENT_ROUTES / name).read_text()
        for name in ["silent_sessions.rs", "silent_sessions_create.rs", "silent_sessions_lifecycle.rs",
                     "silent_sessions_control.rs", "silent_sessions_authorize.rs"]
    })

    print(
        "✓ PASS: work-loop, autonomy, and SilentSession are bounded orchestration, not Focus State authority"
    )


if __name__ == "__main__":
    main()
