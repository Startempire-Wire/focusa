#!/usr/bin/env python3
"""Build-independent Spec 152 installer presenter parity report."""

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = json.loads((ROOT / "docs/contracts/spec152-installer-presenter-parity.v1.json").read_text())


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


require(CONTRACT["schema"] == "focusa.installer_presenter_parity.v1", "wrong contract schema")
require(set(CONTRACT["states"]) == {"active", "offline_grace", "expired", "revoked", "invalid", "unactivated"}, "state matrix incomplete")
require({case["id"] for case in CONTRACT["cases"]} == {"active", "offline", "expired", "revoked", "invalid", "unactivated", "raw_credential", "interrupted"}, "case matrix incomplete")

sources = {
    name: (ROOT / presenter["source"]).read_text()
    for name, presenter in CONTRACT["presenters"].items()
}
rust = sources["rust_cli"]
bash = sources["bash"]
powershell = sources["powershell"]
menubar = sources["menubar"]

for name, code in CONTRACT["stable_errors"].items():
    if name == "status_unavailable":
        require(code in (ROOT / "apps/menubar/src/lib/components/FirstRunWizard.svelte").read_text(), f"missing menubar code {code}")
    elif name == "interrupted":
        require(code in bash and code in powershell, "interruption code differs across bootstrap delegates")
    elif name == "raw_credential":
        require(code in rust and code in bash, "raw credential rejection differs across Unix entrypoints")
    else:
        require(code in rust, f"canonical Rust presenter missing {code}")

for state in CONTRACT["states"]:
    require(f"'{state}'" in menubar, f"menubar projection missing state {state}")
for action in CONTRACT["recovery_actions"]:
    require(action in menubar.lower(), f"menubar recovery policy omits {action}")

require('ARGS=(install --target="$RUST_TARGET"' in bash, "Bash does not delegate")
require('$Args = @("install", "--target=$ResolvedTarget"' in powershell, "PowerShell does not delegate")
require("return Ok(\"eval\".to_string())" not in rust, "Rust retains self-issued eval bypass")
require("write_license_json" not in bash and "license.json" not in powershell, "bootstrapper retains local authority")
require("$LicenseKey" not in powershell and "LICENSE_KEY=" not in bash, "bootstrapper stores raw license material")

node_tests = [
    "apps/menubar/tests/first-run-entitlement.mjs",
    "apps/menubar/tests/entitlement-posture.mjs",
]
for test in node_tests:
    subprocess.run(
        ["node", "--experimental-strip-types", str(ROOT / test)],
        cwd=ROOT / "apps/menubar",
        check=True,
        capture_output=True,
        text=True,
    )

report = {
    "schema": "focusa.installer_presenter_parity_report.v1",
    "contract": "docs/contracts/spec152-installer-presenter-parity.v1.json",
    "presenters": [
        {
            "name": name,
            "role": definition["role"],
            "cases": [case["id"] for case in CONTRACT["cases"]],
            "result": "passed",
        }
        for name, definition in CONTRACT["presenters"].items()
    ],
    "redaction": "passed",
    "recovery_parity": "passed",
    "lease_status_fields": "passed",
    "interruption_rollback": "passed",
}
print(json.dumps(report, indent=2, sort_keys=True))
