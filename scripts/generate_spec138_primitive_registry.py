#!/usr/bin/env python3
"""Generate the canonical Spec138 primitive registry from §5 text catalogs."""
from pathlib import Path
import hashlib
import re

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "docs/138-focusa-prediction-outcome-calibration-metacognitive-learning-transfer-and-epistemic-governance-spec.md"
OUT = ROOT / "crates/focusa-core/src/epistemic_primitives.txt"
lines = SOURCE.read_text().splitlines()
in_catalog = False
family = None
fence = False
rows = []
for line in lines:
    if line.startswith("## 5. "):
        in_catalog = True
    elif in_catalog and line.startswith("## 6."):
        break
    if not in_catalog:
        continue
    match = re.match(r"### 5\.(\d+) (.+)", line)
    if match:
        family = (int(match.group(1)), match.group(2).strip())
        continue
    if line.strip() == "```text":
        fence = True
        continue
    if fence and line.strip() == "```":
        fence = False
        continue
    primitive = line.strip()
    if fence and family and re.fullmatch(r"[A-Za-z][A-Za-z0-9]+", primitive):
        rows.append(f"{family[0]}|{family[1]}|{primitive}")
assert len(rows) == 629, len(rows)
assert len(set(rows)) == 629
assert len({row.rsplit("|", 1)[1] for row in rows}) == 625
OUT.write_text("\n".join(rows) + "\n")
print(f"generated {len(rows)} primitives; sha256={hashlib.sha256(OUT.read_bytes()).hexdigest()}")
