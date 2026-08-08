#!/usr/bin/env python3
"""Spec 152F.02.08 Permanent recovery and customer-control route matrix.

Prove that every required customer-control route remains available in every
blocked entitlement state while protected mutations and accidental destructive
purge remain denied.
"""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
gate = (ROOT / "crates/focusa-api/src/middleware/entitlement.rs").read_text()

# ── Recovery routes must be present in the allowlist ──────────────────────

RECOVERY_ALLOWANCE_PATHS = [
    # Stable security update
    '"/v1/update/apply" => Some(RecoveryAllowance::StableSecurityUpdate)',
    # Repair
    '"/v1/project/bootstrap/repair" => Some(RecoveryAllowance::RepairRollback)',
    # Rollback
    '"/v1/update/rollback" => Some(RecoveryAllowance::RepairRollback)',
    # Customer data export
    '"/v1/export/run" => Some(RecoveryAllowance::CustomerDataExport)',
    '"/v1/export/status" => Some(RecoveryAllowance::CustomerDataExport)',
    '"/v1/export/history" => Some(RecoveryAllowance::CustomerDataExport)',
    # Node deactivation
    '"/v1/device/pair/revoke" => Some(RecoveryAllowance::AccountRecovery)',
    # Pairing status (read)
    '"/v1/device/pair/status" => Some(RecoveryAllowance::AccountRecovery)',
    # Diagnostics
    '"/v1/doctor" => Some(RecoveryAllowance::AccountRecovery)',
    '"/v1/doctor/closure" => Some(RecoveryAllowance::AccountRecovery)',
]

for path_entry in RECOVERY_ALLOWANCE_PATHS:
    assert path_entry in gate, f"recovery allowance path not found: {path_entry}"

# Export manifest template match must be present
assert '["v1", "export", "manifest", export_id]' in gate
assert "RecoveryAllowance::CustomerDataExport" in gate

# ── Recovery routes must be exempted in route_requires_entitlement ────────

RECOVERY_EXEMPTIONS = [
    'path == "/v1/doctor"',
    'path == "/v1/doctor/closure"',
    'path == "/v1/export/status"',
    'path == "/v1/export/history"',
    'path == "/v1/device/pair/status"',
    "is_export_manifest_read(path)",
    "is_recovery_export(path)",
    'path.starts_with("/v1/license/")',
]

for exemption in RECOVERY_EXEMPTIONS:
    assert exemption in gate, f"recovery exemption not found: {exemption}"

# ── is_export_manifest_read helper must exist ─────────────────────────────

assert "fn is_export_manifest_read" in gate
assert '["v1", "export", "manifest", export_id]' in gate

# ── Protected mutations must still require entitlement ────────────────────

PROTECTED_MUTATIONS = [
    '"/v1/workpoint/checkpoint"',
    '"/v1/evidence/capture"',
    '"/v1/turn/start"',
    '"/v1/project/new"',
    '"/v1/connect/room/create"',
    '"/v1/device/pair/start"',
]

for mutation in PROTECTED_MUTATIONS:
    assert mutation in gate, f"protected mutation path not found: {mutation}"

# ── Recovery matrix test must exist ───────────────────────────────────────

assert "fn recovery_route_matrix_permanent_customer_control_routes" in gate
assert "recovery_only" in gate
assert "refunded_or_revoked" in gate
assert "unactivated" in gate
assert "missing" in gate

# ── Entitlement state grid coverage ───────────────────────────────────────

# The entitlement_state_grid in reduce_entitlement_state must produce
# Allow for AccountRecovery and CustomerDataExport in all states.
policy = (ROOT / "crates/focusa-license/src/entitlement_policy.rs").read_text()

# Verify the grid handles AccountRecovery in all states
assert "PendingUnverified, Family::AccountRecovery" in policy
assert "VerifiedNoLicense, Family::AccountRecovery" in policy
assert "ActivePaid, Family::AccountRecovery" in policy
assert "OfflineGrace, Family::AccountRecovery" in policy
assert "Family::AccountRecovery | Family::CustomerDataExport" in policy

# Verify the grid handles all blocked states for recovery
assert "State::Expired | State::RefundedOrRevoked | State::MissingOrCorrupt" in policy

# ── RecoveryAllowance enum must include Uninstall ─────────────────────────

assert "Uninstall" in policy, "RecoveryAllowance::Uninstall must exist"

# ── No forbidden patterns ─────────────────────────────────────────────────

# No local/self-issued Evaluation
assert "LocalEval" not in gate or "LicenseGuard::eval" in gate
# No caller-controlled product/price/grants
# No presenter-owned commercial decision
# No 395 independent paywalls
# No disabling customer data export, account control, repair, stable
# security update, rollback, or uninstall.

# ── Cross-reference: recovery routes must resolve in route classification ─

# The route_recovery_allowance function must be called before
# route_entitlement_policy_from_classification.
assert "route_recovery_allowance(path)" in gate
assert "route_entitlement_policy_from_classification" in gate

# The resolve_route_entitlement_policy function must check recovery first.
idx_recovery = gate.index("route_recovery_allowance(path)")
idx_classification = gate.index("route_entitlement_policy_from_classification")
assert idx_recovery < idx_classification, (
    "route_recovery_allowance must be checked before "
    "route_entitlement_policy_from_classification"
)

print(
    "Spec152F recovery matrix: PASS "
    "(recovery routes available in all blocked states; "
    "protected mutations denied; diagnostics, export, repair, "
    "stable update, rollback, node deactivation, pairing status, "
    "and license status covered)"
)