#!/usr/bin/env python3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
path = ROOT / "tests/spec137a_138a_144_documentation_closure_gate.py"
text = path.read_text()
old = '''assert "137A" in (ROOT / "docs/139-distributed-presence-environment-awareness-execution-placement-and-multi-daemon-coordination-spec.md").read_text().splitlines()[9]
assert "138A" in (ROOT / "docs/140-project-agent-runtime-constitution-instruction-authority-system-prompt-and-cross-harness-compiler-spec.md").read_text().splitlines()[7]
print("literal source atom coverage and remaining owner integration: PASS")'''
new = '''spec139_lines = (ROOT / "docs/139-distributed-presence-environment-awareness-execution-placement-and-multi-daemon-coordination-spec.md").read_text().splitlines()
spec140_lines = (ROOT / "docs/140-project-agent-runtime-constitution-instruction-authority-system-prompt-and-cross-harness-compiler-spec.md").read_text().splitlines()
spec139_depends = next(line for line in spec139_lines if line.startswith("**Depends on:**"))
spec140_depends = next(line for line in spec140_lines if line.startswith("**Depends on:**"))
assert "137A" in spec139_depends and "138A" in spec139_depends
assert "137A" in spec140_depends and "138A" in spec140_depends
print("literal source atom coverage and remaining owner integration: PASS")'''
if old in text:
    text = text.replace(old, new, 1)
elif "spec139_depends = next(" not in text:
    raise SystemExit("closure gate did not contain known dependency assertion form")
path.write_text(text.rstrip() + "\n")
print("Spec closure gate finalization applied")
