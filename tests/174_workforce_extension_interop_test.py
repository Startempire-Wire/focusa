#!/usr/bin/env python3
from pathlib import Path
import subprocess
R=Path(__file__).resolve().parents[1]
def text(path): return (R/path).read_text()
health=text('crates/focusa-api/src/routes/health.rs')
envelope=text('crates/focusa-api/src/routes/silent_sessions_contract.rs')
routes=text('crates/focusa-api/src/routes/silent_sessions.rs')
auth=text('crates/focusa-api/src/middleware/auth.rs')
client=text('apps/workforce-extension/src/lib/api-client.mjs')
client_test=text('apps/workforce-extension/tests/api-client.test.mjs')
menubar=text('apps/menubar/src/lib/stores/pairing.svelte.ts')
# New fields are additive: established legacy fields remain.
assert '"schema": HEALTH_SCHEMA' in health
for field in ['"ok": true','"status": "ok"','"version": version','"uptime_ms": uptime_ms','"persistence": persistence']:
    assert field in health, field
assert 'pub schema: String' in envelope
for field in ['pub ok: bool','pub status: String','pub canonical: bool','pub data: Option<T>']:
    assert field in envelope, field
envelope_decl=envelope.split('pub struct SilentSessionApiEnvelope',1)[0].rsplit('#[derive',1)[1]
assert '#[serde(deny_unknown_fields)]' not in envelope_decl
# Existing menubar pairing code remains on its established route/field contract.
for route in ['/device/pair/start','/device/pair/status','/device/pair/list','/device/pair/revoke']:
    assert route in menubar, route
subprocess.run(['git','diff','--quiet','v0.9.183-dev','--','apps/menubar/src/lib/stores/pairing.svelte.ts','crates/focusa-api/src/routes/device_pairing.rs','crates/focusa-api/src/routes/silent_sessions_lifecycle.rs'],cwd=R,check=True)
# Approval is additive and does not replace lifecycle routes.
assert '/v1/silent-sessions/{session_id}/approvals' in routes
for action in ['start','pause','resume','cancel']:
    assert f'/v1/silent-sessions/{{session_id}}/{action}' in routes
# Pair bootstrap stays narrowly public; administrative completion/list/revoke do not.
public=auth.split('fn is_pre_auth',1)[1].split('\n}',1)[0]
for route in ['/v1/device/pair/start','/v1/device/pair/status','/v1/device/pair/qr']:
    assert route in public
for route in ['/v1/device/pair/complete','/v1/device/pair/list','/v1/device/pair/revoke']:
    assert route not in public
# Old daemon behavior is explicit unsupported, never false success.
assert "body.schema !== contract.schema" in client
assert "ProjectionRequestError('unsupported'" in client
assert 'unknown or malformed schemas fail closed' in client_test
subprocess.run(['node','--test','apps/workforce-extension/tests/api-client.test.mjs','apps/workforce-extension/tests/pairing.test.mjs'],cwd=R,check=True)
print('PASS: Workforce cross-version pairing, approval, lifecycle, and old-daemon behavior')
