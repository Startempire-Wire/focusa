#!/usr/bin/env python3
"""Spec 152E §18/§20 cross-surface recovery-only contract validation.

Every entitlement denial (email/account/payment/license/node/lease) must map
to an exact safe recovery action, preserved recovery surfaces (export,
diagnostics, repair, update-for-recovery, uninstall, license status, account
verification) must stay available in every blocked posture, protected
mutations must be denied before side effects, and recovery never grants
entitlement. The daemon middleware embeds the contract and every presenter
(API/CLI/TUI/menubar/agent) carries the same recovery posture.

Exact verification: python3 tests/spec152e_recovery_only_surface_test.py
"""

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACT_PATH = ROOT / "docs/contracts/spec152e-recovery-only-surface.v1.json"
ERRORS_PATH = ROOT / "docs/contracts/spec152e-activation-errors.v1.json"
INTERNAL_PATH = ROOT / "docs/contracts/spec152e-activation-internal.v1.json"
MIDDLEWARE = ROOT / "crates/focusa-api/src/middleware/entitlement.rs"
CLI_LICENSE = ROOT / "crates/focusa-cli/src/commands/license.rs"
TUI_API = ROOT / "crates/focusa-tui/src/api.rs"
MENUBAR_POSTURE = ROOT / "apps/menubar/src/lib/entitlementPosture.ts"
AGENT_STATE = ROOT / "apps/pi-extension/src/state.ts"
ENTITLEMENT_POLICY = ROOT / "crates/focusa-license/src/entitlement_policy.rs"

contract = json.loads(CONTRACT_PATH.read_text(encoding="utf-8"))
errors = json.loads(ERRORS_PATH.read_text(encoding="utf-8"))
internal = json.loads(INTERNAL_PATH.read_text(encoding="utf-8"))
middleware_src = MIDDLEWARE.read_text(encoding="utf-8")
cli_src = CLI_LICENSE.read_text(encoding="utf-8")
tui_src = TUI_API.read_text(encoding="utf-8")
menubar_src = MENUBAR_POSTURE.read_text(encoding="utf-8")
agent_src = AGENT_STATE.read_text(encoding="utf-8")
policy_src = ENTITLEMENT_POLICY.read_text(encoding="utf-8")

assert contract["contract_version"] == 1
assert contract["schema"] == "focusa.spec152e.recovery_only_surface.v1"
assert contract["authority"]["canonical"] == "WPUIAI.com EDD"
assert contract["authority"]["spec158"] == "excluded"
for invariant, expected in contract["invariants"].items():
    assert expected is True, f"invariant {invariant} must hold"

# ── Denial bindings: every stable failure maps to exactly one class ──
bindings = contract["denial_bindings"]
classes = [binding["class"] for binding in bindings]
assert classes == ["email", "account", "payment", "license", "node", "lease", "activation_mechanics"]
bound_codes = {}
for binding in bindings:
    assert binding["posture"] == "recovery_only"
    assert binding["recovery_action"]
    assert len(binding["codes"]) == len(set(binding["codes"])), binding["class"]
    assert binding["safe_next_actions"]
    for code in binding["codes"]:
        assert code not in bound_codes, f"duplicate denial code {code}"
        bound_codes[code] = binding["class"]

# Every code in the activation-errors registry is bound to exactly one class.
registry = {row["code"]: row for row in errors["errors"]}
assert set(registry) == set(bound_codes), (
    f"registry/binding mismatch: only in registry={set(registry) - set(bound_codes)} "
    f"only in bindings={set(bound_codes) - set(registry)}"
)
# Each code's granular safe_next_action is consistent with its class action set.
for code, row in registry.items():
    binding = next(b for b in bindings if code in b["codes"])
    assert row["safe_next_action"] in binding["safe_next_actions"], (
        f"{code}: safe_next_action {row['safe_next_action']} not in class {binding['class']}"
    )
# Refund/revoke/unusable-license failures are recovery-only on every surface.
for code in ("REFUNDED", "REVOKED", "EDD_LICENSE_UNUSABLE"):
    assert registry[code]["safe_next_action"] == "recovery_only", code

# ── Runtime (daemon) denial bindings ──
runtime = contract["runtime_denial_bindings"]
expected_runtime_codes = {
    "ENTITLEMENT_BASE_REQUIRED", "ENTITLEMENT_REQUIRED", "ENTITLEMENT_FEATURE_REQUIRED",
    "ENTITLEMENT_LIMIT_EXHAUSTED", "ENTITLEMENT_IDEMPOTENCY_REQUIRED",
    "ENTITLEMENT_RESERVATION_FAILED", "ENTITLEMENT_ROUTE_UNCLASSIFIED",
}
assert set(runtime) == expected_runtime_codes
for code, binding in runtime.items():
    assert binding["class"] in classes
    assert binding["recovery_action"]
assert contract["default_runtime_denial"]["recovery_action"] == "recovery_only"
assert contract["default_runtime_denial"]["class"] == "lease"

# ── Recovery surfaces: preserved exactly per Spec 152E §18 ──
surfaces = contract["recovery_surfaces"]
surface_names = {surface["surface"] for surface in surfaces}
assert surface_names == {
    "account_verification", "license_status_management", "export", "diagnostics",
    "repair", "update_for_recovery", "uninstall",
}
all_daemon_paths = []
for surface in surfaces:
    assert surface["action"]
    assert surface["allowance"]
    for path in surface["daemon_paths"]:
        assert path not in all_daemon_paths, f"duplicate recovery path {path}"
        all_daemon_paths.append(path)
uninstall = next(s for s in surfaces if s["surface"] == "uninstall")
assert uninstall["daemon_paths"] == []
assert "cli" in uninstall["presenters"] and "agent" in uninstall["presenters"]

# Recovery surfaces never overlap forbidden protected mutations.
forbidden = contract["forbidden_protected_mutations"]
assert len(forbidden) == len(set(forbidden))
assert set(forbidden).isdisjoint(all_daemon_paths)
assert contract["invariants"]["recovery_never_grants_entitlement"] is True

# ── Consistency block ──
consistency = contract["consistency"]
assert set(consistency["surfaces"]) == {"api", "cli", "tui", "menubar", "agent"}
assert consistency["status_path"] == "/v1/license/status"
assert consistency["recovery_message"] == "recovery, export, repair, and uninstall remain available"
allowed = consistency["envelope_allowed"]
assert len(allowed) == len(set(allowed))
for label in ("health", "version", "license_status", "export", "diagnostics",
              "repair", "update_for_recovery", "uninstall", "safe_read"):
    assert label in allowed, label

# ── Daemon middleware implements the contract ──
assert "spec152e-recovery-only-surface.v1.json" in middleware_src, (
    "middleware must embed the recovery-only surface contract"
)
assert '"action": guidance.action' in middleware_src, (
    "denial envelope must carry the bound recovery action"
)
assert '"allowed": guidance.allowed' in middleware_src, (
    "denial envelope must carry the preserved recovery surface list"
)
for code in expected_runtime_codes:
    assert code in middleware_src, f"middleware must emit runtime denial code {code}"
# Every declared recovery path resolves in the middleware (allowance or exempt read).
for surface in surfaces:
    for path in surface["daemon_paths"]:
        assert path in middleware_src, (
            f"recovery surface {surface['surface']} path {path} missing from middleware"
        )
# The route_recovery_allowance table must name every non-exempt recovery path.
allowance_block = middleware_src.split("fn route_recovery_allowance")[1]
exempt_paths = {"/v1/license/status", "/v1/update/check", "/v1/update/plan"}
for surface in surfaces:
    for path in surface["daemon_paths"]:
        if path in exempt_paths:
            continue
        assert path in allowance_block, (
            f"{path} must be declared in route_recovery_allowance"
        )
# Forbidden protected mutations are denied by the middleware's own matrix.
protected_block = middleware_src.split("fn blocked_leases_permit_only_declared_recovery_surfaces_with_zero_mutation_sentinels")[1]
for path in forbidden:
    assert path in protected_block, f"protected mutation {path} missing from middleware sentinel matrix"

# ── Presenters carry the same recovery posture ──
message = consistency["recovery_message"]
assert message in cli_src, "CLI presenter must surface the recovery message"
assert message in tui_src, "TUI presenter must surface the recovery message"
assert message.lower() in menubar_src.lower(), "menubar presenter must surface the recovery message"
assert '"recovery_only"' in menubar_src or "recovery_only" in menubar_src, (
    "menubar presenter must know the recovery_only posture"
)
assert consistency["status_path"] in agent_src, "agent presenter must carry the recovery status path"
for label in ("export", "diagnostics", "repair", "update_for_recovery", "uninstall"):
    assert f'"{label}"' in agent_src, f"agent presenter must allow recovery surface {label}"
assert "recovery_only" in agent_src
# Uninstall is a declared local lifecycle recovery allowance in focusa-license.
assert "Uninstall," in policy_src and "implied_family" in policy_src

# ── State machine: every terminal denial lands in recovery_only ──
machine = internal["registration_states"]
for denied in ("denied", "refunded", "revoked", "superseded", "expired"):
    assert denied in machine["transitions"]
    assert machine["transitions"][denied] == ["recovery_only"], denied
assert machine["transitions"]["recovery_only"] == []
assert "recovery_only" in machine["terminal"]

# ── Hygiene: no raw email, secret, or unmasked evidence ──
raw = CONTRACT_PATH.read_text(encoding="utf-8")
assert not re.search(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}", raw)
assert not re.search(r"(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+", raw)
assert "full_license_key" not in raw
assert "FOCUSA-" not in raw

print(json.dumps({
    "schema": "focusa.spec152e.recovery_only_surface_validation.v1",
    "denial_classes": len(bindings),
    "bound_codes": len(bound_codes),
    "recovery_surfaces": len(surfaces),
    "forbidden_protected_mutations": len(forbidden),
    "runtime_denial_bindings": len(runtime),
    "consistency_surfaces": sorted(consistency["surfaces"]),
    "result": "passed_fail_closed",
}, sort_keys=True))
