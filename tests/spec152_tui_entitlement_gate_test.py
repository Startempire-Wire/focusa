#!/usr/bin/env python3
from pathlib import Path

source = Path("crates/focusa-tui/src/api.rs").read_text()
mutation = source[source.index("pub async fn resolve_daemon_routing"):source.index("pub async fn fetch_with_scope")]
for marker in [
    "StatusCode::FORBIDDEN",
    'code.starts_with("ENTITLEMENT_")',
    "required_feature",
    "limit_bucket",
    "blocked before execution",
    "recovery, export, repair, and uninstall",
]:
    assert marker in mutation, marker
assert mutation.index("StatusCode::FORBIDDEN") < mutation.index("error_for_status")
assert "eval" not in mutation.lower()
print("Spec152 TUI entitlement gate: PASS")
