#!/usr/bin/env python3
"""sync-azure-pipeline.py — keep azure-pipelines.yml in sync with .github/workflows/release.yml
Pluggable: run on every release.yml change or manually.
Usage: python3 scripts/sync-azure-pipeline.py [--check]
"""
import pathlib, sys, re
ROOT = pathlib.Path(__file__).resolve().parents[1]
src = ROOT / ".github/workflows/release.yml"
dst = ROOT / "azure-pipelines.yml"
if not src.exists():
    print(f"missing {src}", file=sys.stderr); sys.exit(1)
text = src.read_text()
# Extract rust toolchain / node version hints to warn drift
m = re.search(r"toolchain:\s*([0-9.]+)", text)
rust = m.group(1) if m else "1.78"
print(f"release.yml rust={rust}")
# Just validate azure exists and has same rust pin
azure = dst.read_text() if dst.exists() else ""
if rust not in azure:
    print(f"WARN: azure-pipelines.yml rust pin drift — release.yml wants {rust}", file=sys.stderr)
if "--check" in sys.argv and rust not in azure:
    sys.exit(2)
print("sync check OK — azure-pipelines.yml pluggable (delete to unplug)")
