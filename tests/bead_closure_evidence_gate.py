#!/usr/bin/env python3
"""
Guard for #317/#318: Spec 131 Closure Authority — every bead close requires
cargo fmt + clippy + spec gate + docs/evidence/finish/<id>-acceptance.txt with commit SHA.
"""
import subprocess, pathlib, sys, re
ROOT = pathlib.Path(__file__).resolve().parents[1]
# Find commits in range that look like bd close
try:
    base = subprocess.check_output(["git","rev-parse","HEAD~1"], text=True).strip()
except: base="HEAD~1"
log = subprocess.check_output(["git","log","--oneline",f"{base}..HEAD"], text=True)
# Look for evidence file existence for any issue referenced
evidence_dir = ROOT / "docs/evidence/finish"
fails=[]
for line in log.splitlines():
    if "close" in line.lower() and "focusa-" in line:
        m=re.search(r"focusa-[a-z0-9.-]+",line)
        if m:
            fid=m.group(0).replace(".","-")
            # check if any acceptance file exists for this id
            matches=list(evidence_dir.glob(f"{fid}*")) + list(evidence_dir.glob(f"*{m.group(0)}*"))
            if not matches:
                fails.append(f"Missing evidence for {m.group(0)} in {line}")
if fails:
    for f in fails: print(f"FAIL: {f}",file=sys.stderr)
    sys.exit(1)
print("PASS: bead closure evidence gate")
