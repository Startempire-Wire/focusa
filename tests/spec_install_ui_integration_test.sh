#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALL="$ROOT/crates/focusa-cli/src/commands/install.rs"
python3 - "$INSTALL" <<'PY'
import pathlib, sys
text = pathlib.Path(sys.argv[1]).read_text()
assert 'detect_capabilities(args.no_animation, args.json, args.quiet)' in text
assert 'AnimatedRenderLoop::new(mode, seed).run(receiver, token)' in text
assert 'EnterAlternateScreen' not in text, 'CLI must not own alternate-screen escapes'
assert 'PlainPresenter::new(quiet)' in text
assert 'channel.fail(error.to_string())' in text
assert 'ui.finish();' in text
# Plain and silent modes use presenters without starting the render thread.
plain = text.index('} else if mode == InstallRendererMode::Plain')
silent = text.index('} else {', plain)
assert 'renderer: None' in text[plain:silent]
assert 'renderer: None' in text[silent:silent + 300]
print('Spec 132 UI integration lifecycle/fallback contract: PASS')
PY
