#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALL="$ROOT/crates/focusa-cli/src/commands/install.rs"
python3 - "$INSTALL" <<'PY'
import pathlib, sys
text = pathlib.Path(sys.argv[1]).read_text()
assert 'phase_pi_extension_download' in text
assert 'Pi extension download/integration unavailable' in text
assert 'InstallEvent::PhaseSkipped' in text and 'Pi extension not detected' in text
assert 'InstallEvent::PhaseSucceeded' in text and 'verified at' in text
assert 'InstallEvent::PhaseWarning' in text and 'Pi integration could not be completed' in text
assert '["install", "--omit=dev", "--ignore-scripts"]' in text
assert '.take(512)' in text
# Optional Pi failure is explicitly converted to a warning and does not return
# from execute_real_install before the core assets proceed.
start = text.index('let pi_extension = match phase_pi_extension_download')
end = text.index('let agent_context = phase_agent_context_download', start)
chunk = text[start:end]
assert 'None' in chunk and 'PhaseWarning' in chunk
print('Spec 132 Rust-owned Pi integration truth contract: PASS')
PY
