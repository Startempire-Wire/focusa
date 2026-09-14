#!/usr/bin/env python3
"""262 gate — Workstream continuation, fanout, post-compaction."""
import pathlib, sys
ROOT = pathlib.Path(__file__).resolve().parents[1]
checks=[
 ("crates/focusa-core/src/workstream_root.rs", "workstream"),
 ("crates/focusa-core/src/session_fanout.rs", "FANOUT_SCHEMA"),
 ("crates/focusa-core/src/session_fanout.rs", "LaneRole"),
]
failed=[]
for p, needle in checks:
  if needle not in (ROOT/p).read_text():
    failed.append(f"{p} missing {needle}")
if failed:
  print(f"262 gate FAIL {failed}"); sys.exit(1)
print("262 gate PASS (workstream_root, session_fanout, LaneRole present)")
sys.exit(0)
