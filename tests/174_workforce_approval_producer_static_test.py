#!/usr/bin/env python3
from pathlib import Path
R=Path(__file__).resolve().parents[1]
route=(R/'crates/focusa-api/src/routes/silent_sessions_approvals.rs').read_text()
contract=(R/'crates/focusa-api/src/routes/silent_sessions_contract.rs').read_text()
input_src=(R/'crates/focusa-api/src/routes/silent_sessions_input.rs').read_text()
payload=(R/'crates/focusa-api/src/routes/silent_sessions_approval_payload.rs').read_text()
auth=(R/'crates/focusa-core/src/silent_sessions/authorization.rs').read_text()
store=(R/'crates/focusa-core/src/silent_sessions/authorization_persistence.rs').read_text()
schema=(R/'crates/focusa-core/src/silent_sessions/persistence_sqlite.rs').read_text()
mod=(R/'crates/focusa-api/src/routes/mod.rs').read_text()
router=(R/'crates/focusa-api/src/routes/silent_sessions.rs').read_text()
for marker in ['authorize_silent_session_approval_issuance','action_digest(&issuance_request)','save_durable_approval','load_durable_approval_by_idempotency','APPROVAL_TTL_MINUTES','risk_acknowledged','issuance_request_hash']:
 assert marker in route, marker
assert 'pub mod silent_sessions_approvals;' in mod
assert 'pub mod silent_sessions_approval_payload;' in mod
assert 'silent_sessions_approvals::create' in router
assert 'SILENT_SESSION_DB_SCHEMA_VERSION: i64 = 5' in schema
assert 'issuance_idempotency_key' in schema and 'MIGRATION_V5_SQL' in schema
assert 'load_durable_approval_by_idempotency' in store
assert 'authorize_silent_session_approval_issuance' in auth
assert 'delivery_request_hash_for_approval' in payload
assert 'delivery_request_hash_for_approval' in input_src
assert 'action_digest' not in contract.split('pub struct ApprovalCreateRequest',1)[1].split('}',1)[0]
for path in [R/'crates/focusa-api/src/routes/silent_sessions_approvals.rs',R/'crates/focusa-api/src/routes/silent_sessions_input.rs']:
 assert len(path.read_text().splitlines()) < 500, path
print('PASS: payload-bound durable approval producer is registered and fail-closed')
