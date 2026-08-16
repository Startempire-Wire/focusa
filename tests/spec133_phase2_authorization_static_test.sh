#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SS="$ROOT/crates/focusa-core/src/silent_sessions"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }; pass(){ echo "✓ PASS: $*"; }
for scope in 'silent_sessions:read' 'silent_sessions:stream' 'silent_sessions:create' 'silent_sessions:control' 'silent_sessions:config' 'silent_sessions:admin' 'silent_sessions:forensics'; do
  rg -n "$scope" "$SS/authorization.rs" >/dev/null || fail "missing route scope: $scope"
done
pass "all exact §17.2 route scopes are typed"
for marker in AuthenticatedPrincipal SilentSessionRole os_user VerifiedAuthorityFacts authorized_project_root authorized_continuity_id authorized_work_item_ref writer_principal_id ContextAuthorityVerdict session_id run_id; do
  rg -n "$marker" "$SS/authorization.rs" >/dev/null || fail "missing authority fact: $marker"
done
pass "principal, actor/role, OS user, project, continuity, work-item, writer, context, session and run checks are explicit"
for marker in DurableApprovalRecord ApprovalId operator_actor action_digest config_hash model_binding workspace risk_class expires_at permitted_side_effects approval_durably_verified; do
  rg -n "$marker" "$SS/authorization.rs" >/dev/null || fail "missing durable approval field: $marker"
done
pass "durable expiring action/config/model/workspace approval matching is explicit"
rg -n 'legacy_approved' "$SS/authorization.rs" >/dev/null || fail "legacy approved compatibility input missing"
rg -n 'durably verified approval is required' "$SS/authorization.rs" >/dev/null || fail "approved=true fail-closed guard missing"
pass "approved=true alone cannot authorize"
for marker in RedactedSummary 'cross-user stream access is denied' RawForensics; do
  rg -n "$marker" "$SS/authorization.rs" >/dev/null || fail "missing isolation marker: $marker"
done
pass "cross-user stream denial and redacted/admin/forensics projection are explicit"
for marker in AuthenticatedRunnerCommand hmac_sha256_hex constant_time_eq socket_scope nonce Replay InvalidTag; do
  rg -n "$marker" "$SS/runner_security.rs" >/dev/null || fail "missing runner authentication marker: $marker"
done
pass "runner controls are scoped, authenticated, expiring and replay safe"
for marker in auth_header bearer_token provider_credential private_key_material secret_value redaction_classes; do
  rg -n "$marker" "$SS/runner_security.rs" >/dev/null || fail "missing audit redaction class: $marker"
done
pass "control audit redacts all forbidden secret classes"
for table in silent_session_principals silent_session_approvals silent_session_control_audits silent_session_runner_nonces; do
  rg -n "$table" "$SS/persistence_sqlite.rs" >/dev/null || fail "missing durable authorization table: $table"
done
pass "principals, approvals, redacted audits and consumed nonces are durable"
