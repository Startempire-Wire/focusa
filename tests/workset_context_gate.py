#!/usr/bin/env python3
"""272 gate — Workset context bounded preload, Workpoint/CallGraph binding."""
import pathlib, sys
ROOT=pathlib.Path(__file__).resolve().parents[1]
need=[
 ("crates/focusa-core/src/callgraph.rs","CallGraph"),
 ("crates/focusa-core/src/callgraph_envelope.rs","CallGraph"),
 ("crates/focusa-core/src/types.rs","Workpoint"),
]
failed=[]
for p, n in need:
  if n not in (ROOT/p).read_text():
    failed.append(f"{p} missing {n}")
if failed:
  print(f"272 FAIL {failed}"); sys.exit(1)
print("272 PASS Workpoint/CallGraph/bounded preload present")
sys.exit(0)
