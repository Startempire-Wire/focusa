#!/usr/bin/env python3
from pathlib import Path

workflow = (Path(__file__).resolve().parents[1] / ".github/workflows/deslop.yml").read_text()

required = [
    'DESLOP_VERSION: "0.32.0"',
    "DESLOP_ARCHIVE_SHA256:",
    "DESLOP_CONTAINER:",
    "sha256sum -c -",
    "runs-on: [self-hosted, Linux, X64, ovh-build]",
    "podman run --rm",
    "--memory 4g",
    "--memory-swap 4g",
    "--userns=keep-id",
    '"${GITHUB_WORKSPACE}:/workspace:ro,Z"',
    "deslop . --no-incremental --min-nodes 80 --output /reports/deslop-report",
    'test -s "${reports}/deslop-report.json"',
    'test -s "${reports}/deslop-report.txt"',
    'test -s "${reports}/deslop-report.html"',
    'exit "${scan_status}"',
    "deslop-exit-code.txt",
    "if: failure()",
    "if-no-files-found: error",
]
for marker in required:
    assert marker in workflow, f"missing fail-closed Deslop workflow marker: {marker}"

assert "if: always()" not in workflow, "successful scans must not consume artifact quota"
assert "continue-on-error" not in workflow, "Deslop failures must not be masked"
assert "if-no-files-found: ignore" not in workflow, "missing reports must fail closed"
assert "Nimblesite/Deslop@v0.30.0" not in workflow, "glibc-incompatible action remains active"

print("deslop workflow static test: PASS")
