#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALL="$ROOT/crates/focusa-cli/src/commands/install.rs"
python3 - "$INSTALL" <<'PY'
import pathlib, sys
text = pathlib.Path(sys.argv[1]).read_text()
start = text.index('pub async fn run(args: InstallArgs)')
end = text.index('// ----- Phase 1:', start)
run = text[start:end]

def once(token):
    count = run.count(token)
    assert count == 1, f'{token} appears {count} times in install run'
    return run.index(token)

smoke = run.index('phase_smoke_test(&bin_dir).await')
cleanup = run.index('phase_atomic_cleanup(&stash_path)')
finished = once('InstallEvent::InstallFinished')
summary = run.index('summary.render_human()')
walkthrough = run.index('print_walkthrough_human(&result.walkthrough)')
assert smoke < finished, 'InstallFinished emitted before smoke-test gate'
assert cleanup < finished, 'InstallFinished emitted before stash cleanup'
assert finished < summary < walkthrough, 'durable output is not after completion event'
assert 'Installed assets to' not in run, 'superseded early success report remains'
assert '100%' not in run, 'installer run fabricates an overall 100% report'
# JSON has one success document and no decorative completion print branch.
assert run[finished:].count('serde_json::to_string_pretty(&report)') == 1
print('Spec 132 completion ordering and single-document JSON: PASS')
PY
