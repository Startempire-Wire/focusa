#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALL="$ROOT/crates/focusa-cli/src/commands/install.rs"
python3 - "$INSTALL" <<'PY'
import pathlib, sys
text = pathlib.Path(sys.argv[1]).read_text()
assert 'InstallEvent::RollbackStarted' in text
assert 'InstallEvent::RollbackSucceeded' in text
assert 'InstallEvent::RollbackFailed' in text
assert 'cleanup_staged_downloads(&install_root)' in text
assert 'prior installation restored' in text
assert 'clean-state cleanup completed; no prior installation existed' in text
assert 'phase_atomic_recover' in text
assert 'remove failed fresh install' in text
assert 'pre-commit binary smoke test failed' in text
commit_start = text.index('async fn execute_real_install(')
commit_end = text.index('\nasync fn delegate_service_render(', commit_start)
commit = text[commit_start:commit_end]
assert commit.index('pre-commit binary smoke test failed') < commit.index(
    'place_symlinks(target, &bin_dir, install_root)'
)
assert 'eprint!("\\x1b[?25h\\x1b[?1049l")' not in text
# Failure paths preserve their original error after emitting presentation state.
for marker in ('recovering from installer phase failure', 'recovering from smoke-test failure'):
    assert marker in text
print('Spec 132 cancellation/failure truthful rollback contract: PASS')
PY
