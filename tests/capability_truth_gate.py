#!/usr/bin/env python3
"""279 gate — Capability Truth, claim enforcement, Honesty Manifest."""
import pathlib, sys
ROOT=pathlib.Path(__file__).resolve().parents[1]
text=(ROOT/"crates/focusa-core/src/capability_truth.rs").read_text()
need=["CapabilityTruthClaim","HonestyManifest","honesty_manifest","public_safe"]
failed=[n for n in need if n not in text]
if failed:
  print(f"279 FAIL missing {failed}"); sys.exit(1)
# also check distribution manifest we created for 260
dm=(ROOT/"docs/contracts/spec141/generated-capability-v2/distribution-manifest.json")
if not dm.exists():
  print("279 FAIL distribution-manifest missing"); sys.exit(1)
print("279 PASS CapabilityTruth + HonestyManifest + distribution-manifest present")
sys.exit(0)
