#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

python3 - <<'PY'
from pathlib import Path
import re, sys

runtime_roots = [Path('crates/focusa-api/src'), Path('crates/focusa-core/src'), Path('crates/focusa-cli/src')]
unwrap_hits=[]
for root in runtime_roots:
    for p in root.rglob('*.rs'):
        if p.name.endswith('_test.rs'):
            continue
        text=p.read_text(errors='ignore')
        # Runtime code convention in this repo: tests are below #[cfg(test)].
        runtime=text.split('#[cfg(test)]',1)[0]
        for i,line in enumerate(runtime.splitlines(),1):
            if '.unwrap()' in line or '.expect(' in line:
                # Poison recovery and option defaults use explicit non-panicking alternatives; panic APIs are disallowed in runtime slices.
                unwrap_hits.append(f'{p}:{i}:{line.strip()}')
if unwrap_hits:
    print('runtime unwrap/expect hotspots found:', file=sys.stderr)
    print('\n'.join(unwrap_hits[:80]), file=sys.stderr)
    sys.exit(1)

shell_patterns = [
    ('apps/pi-extension/src/state.ts', 'S.pi!.exec("bash", ["-lc", cmd])'),
    ('apps/pi-extension/src/config.ts', 'systemctl start focusa-daemon || systemctl restart focusa-daemon'),
    ('crates/focusa-cli/src/commands/cleanup.rs', 'Command::new("bash").arg("-lc").arg(cmd).output()'),
    ('crates/focusa-cli/src/commands/release.rs', 'Command::new("bash").arg("-lc").arg(command).output()'),
    ('crates/focusa-core/src/runtime/daemon.rs', 'tokio::process::Command::new("bash")'),
]
missing=[]
for file, marker in shell_patterns:
    text=Path(file).read_text(errors='ignore')
    if marker not in text:
        missing.append(f'{file}: expected reviewed shell hotspot marker missing/changed: {marker}')
if missing:
    print('\n'.join(missing), file=sys.stderr)
    sys.exit(1)

# Reject new bash -lc/-c shell execution outside the reviewed allowlist above.
allowed_files={file for file,_ in shell_patterns}
new_shell=[]
for p in list(Path('crates').rglob('*.rs')) + list(Path('apps/pi-extension/src').rglob('*.ts')):
    text=p.read_text(errors='ignore')
    if str(p) in allowed_files:
        continue
    for i,line in enumerate(text.splitlines(),1):
        if re.search(r'Command::new\("bash"\)|exec\("bash"|bash", \["-lc"|\.arg\("-lc"\)|\.arg\("-c"\)', line):
            new_shell.append(f'{p}:{i}:{line.strip()}')
if new_shell:
    print('new unreviewed shell execution hotspots found:', file=sys.stderr)
    print('\n'.join(new_shell[:80]), file=sys.stderr)
    sys.exit(1)
PY

echo "✓ shell execution allowlist and runtime unwrap classification passed"
