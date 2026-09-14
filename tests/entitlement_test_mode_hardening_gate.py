#!/usr/bin/env python3
"""Entitlement bypass hardening gate for 271 — ensures FOCUSA_TEST_MODE == '1' exact, not is_ok(), and bounded limit 1000 not MAX."""
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]

checks = [
    ("crates/focusa-api/src/main.rs", r'map\(\|value\| value == "1"\)\.unwrap_or\(false\)'),
    ("crates/focusa-api/src/middleware/entitlement.rs", r'map\(\|value\| value == "1"\)\.unwrap_or\(false\)'),
]

failed = []
for path, pattern in checks:
    text = (ROOT / path).read_text()
    if not re.search(pattern, text):
        failed.append(f"{path} missing exact '1' check")
    if re.search(r'FOCUSA_TEST_MODE.*is_ok\(\)', text):
        failed.append(f"{path} still uses is_ok() for FOCUSA_TEST_MODE")
    if "u64::MAX" in text and "FOCUSA_TEST_MODE" in text:
        # only fail if u64::MAX is in the test-mode block
        if re.search(r'FOCUSA_TEST_MODE', text):
            # check ent file separately for MAX outside test-mode is ok, but test-mode should not have MAX
            pass

# Check bounded limit
ent = (ROOT / "crates/focusa-api/src/middleware/entitlement.rs").read_text()
if "1000" not in ent:
    failed.append("middleware/entitlement.rs missing bounded 1000 limit")
if "u64::MAX" in ent:
    failed.append("middleware/entitlement.rs still contains u64::MAX")

if failed:
    print("Entitlement hardening gate: FAIL")
    for f in failed:
        print(f"  - {f}")
    sys.exit(1)

print("Entitlement hardening gate: PASS (exact 1, bounded 1000, no is_ok()/MAX)")
sys.exit(0)
