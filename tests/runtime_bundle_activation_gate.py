#!/usr/bin/env python3
"""256 gate — Runtime Bundle activation fields present."""
import pathlib, sys
ROOT = pathlib.Path(__file__).resolve().parents[1]
text = (ROOT / "crates/focusa-core/src/runtime_bundle.rs").read_text()
need = ["RuntimeBundleManifest", "bundle_id", "activation", "rollback", "digest", "RUNTIME_BUNDLE_SCHEMA"]
failed = [n for n in need if n not in text]
if failed:
    print(f"Runtime bundle gate: FAIL missing {failed}")
    sys.exit(1)
print("Runtime bundle gate: PASS (256 — manifest, activation, rollback present)")
sys.exit(0)
