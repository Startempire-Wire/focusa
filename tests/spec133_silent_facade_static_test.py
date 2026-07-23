#!/usr/bin/env python3
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
TOOLS=(ROOT/'apps/pi-extension/src/tools.ts').read_text()
CONTRACTS=(ROOT/'apps/pi-extension/src/tool-contracts.ts').read_text()
DOC=(ROOT/'docs/focusa-tools/tools/focusa_silent_sessions.md').read_text()
start=TOOLS.index('name: "focusa_silent_sessions"')
end=TOOLS.index('pi.registerTool({', start)
facade=TOOLS[start:end]
for marker in ['/silent-sessions','run_id','generation','approval_id','idempotency_key','parity: "full"','authority: "daemon"','focusaFetchDetailed','URLSearchParams']:
    assert marker in facade, marker
for forbidden in ['silentSessionExec','listSilentSessions','tmux new-session','capture-pane','kill-session','/tmp/','execFileSync']:
    assert forbidden not in facade, forbidden
contract=CONTRACTS[CONTRACTS.index('name: "focusa_silent_sessions"'):]
contract=contract[:contract.index('name: "focusa_tool_doctor"')]
assert 'parity_status: "full"' in contract
assert 'api_routes: [' in contract
assert 'tmux' not in contract.lower()
assert 'Daemon-native Spec133' in DOC
assert 'does not:' in DOC
print('Spec133 Pi daemon facade static contract: PASS')
