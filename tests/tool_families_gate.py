#!/usr/bin/env python3
"""258 gate — Tool families, deduplication, working sets."""
import pathlib, sys, re
ROOT=pathlib.Path(__file__).resolve().parents[1]
text=(ROOT/"apps/pi-extension/src/tools.ts").read_text()
families=re.findall(r'family:\s*"([^"]+)"', text)
if len(families)<5:
  print(f"258 FAIL only {len(families)} families"); sys.exit(1)
print(f"258 PASS {len(set(families))} families, {len(families)} tools with family")
sys.exit(0)
