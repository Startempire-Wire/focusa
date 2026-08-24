#!/usr/bin/env python3
from pathlib import Path
R=Path(__file__).resolve().parents[1]
core=(R/'crates/focusa-core/src/capability_authorization.rs').read_text()
route=(R/'crates/focusa-api/src/middleware/route_scope.rs').read_text()
principal=(R/'crates/focusa-api/src/middleware/principal.rs').read_text()
permissions=(R/'crates/focusa-api/src/routes/permissions.rs').read_text()
persistence=(R/'crates/focusa-core/src/runtime/persistence_sqlite.rs').read_text()
server=(R/'crates/focusa-api/src/server.rs').read_text()
assert 'pub fn can(' in core
for field in ['workstream_key','workset_id','work_item_id','frame_id','risk','entitlement_satisfied']:
 assert f'pub {field}' in core
assert 'CLIENT_SCOPE_ELEVATION_DENIED' in core
assert 'grounded_catalog_is_exhaustively_covered_by_one_gate' in core
assert 'requested_scopes(req.headers())' in route
assert 'permission_context(req.headers()' not in route
assert 'request_principal(req.headers())' in route
assert 'EntitlementGateAccepted' in route
assert 'entitlement_satisfied: req.extensions()' in route
assert '.append_capability_authorization_audit(&decision)' in route
assert 'if !decision.allowed' in route
assert 'resolved.canonical_capability_principal()' in route
assert 'canonical_capability_principal' in principal
assert '"admin" | "admin:*" => {}' in principal
assert 'requested_scopes' in permissions and 'Non-authoritative legacy scope request metadata' in permissions
assert 'CREATE TABLE IF NOT EXISTS capability_authorization_audits' in persistence
assert 'capability authorization audit mismatch' in persistence
assert 'from_fn_with_state' in server and 'route_scope::route_scope_layer' in server
print('PASS: Workforce Full canonical can gate')
