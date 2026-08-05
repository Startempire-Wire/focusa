#!/usr/bin/env python3
"""Static safety gate for the immutable v0.9.144-demo.1 prerelease."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
license_lib = (ROOT / "crates/focusa-license/src/lib.rs").read_text()
core_license = (ROOT / "crates/focusa-core/src/license.rs").read_text()
installer = (ROOT / "scripts/install-focusa.sh").read_text()
workspace = (ROOT / "Cargo.toml").read_text()

assert 'version = "0.9.144-demo.1"' in workspace
assert "pub const UNRESTRICTED_DEMO_BUILD: bool = true;" in license_lib
assert "if UNRESTRICTED_DEMO_BUILD" in license_lib
assert "CapabilityCheck::PermittedWithWarning" in license_lib
assert "licensing enforcement is disabled" in license_lib
assert "const UNRESTRICTED_DEMO_BUILD: bool = true;" in core_license
assert "if UNRESTRICTED_DEMO_BUILD" in core_license
assert "return true;" in core_license
assert 'warn "v0.9.144-demo.1: licensing enforcement is disabled for this demo build."' in installer
assert 'if [ -z "$LICENSE_KEY" ] && [ "$EVAL" != 1 ]; then' in installer
assert "No fake commercial license" in installer
print("v0.9.144-demo.1 unrestricted demo posture: PASS")
