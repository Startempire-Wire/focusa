#!/usr/bin/env python3
from pathlib import Path

root = Path(__file__).resolve().parents[1]
workflow = (root / ".github/workflows/deslop.yml").read_text()
config = (root / "config/deslop-tool.env").read_text()
runner = (root / "scripts/deslop").read_text()

for marker in [
    "DESLOP_CONFIG: config/deslop-tool.env",
    'source "${DESLOP_CONFIG}"',
    "DESLOP_REPORT_DIR=\"${reports}\" scripts/deslop .",
    'test -s "${reports}/deslop-report.json"',
    'test -s "${reports}/deslop-report.txt"',
    'test -s "${reports}/deslop-report.html"',
    'exit "${scan_status}"',
    "if: failure()",
    "if-no-files-found: error",
]:
    assert marker in workflow, f"missing fail-closed Deslop workflow marker: {marker}"

for marker in [
    'DESLOP_VERSION="0.32.0"',
    'DESLOP_ARCHIVE_SHA256="231fd9893609b4fea945b31a6d6e13b3c5fba78a4c94ce2369a8e47faecc588b"',
    'DESLOP_CONTAINER="docker.io/library/node@sha256:',
    'DESLOP_MIN_NODES="80"',
    'DESLOP_MAX_DUPLICATION_PERCENT="20"',
]:
    assert marker in config, f"missing canonical Deslop policy marker: {marker}"

for marker in [
    "sha256sum -c -",
    "--network=none",
    "--memory 4g",
    "--memory-swap 4g",
    "--userns=keep-id",
    '"$ROOT:/workspace:ro,Z"',
    "--no-incremental",
    '"$DESLOP_MIN_NODES"',
    "configured_max_duplication_percent",
    "deslop-report.txt",
]:
    assert marker in runner, f"missing local Deslop safety/report marker: {marker}"

assert "if: always()" not in workflow, "successful scans must not consume artifact quota"
assert "continue-on-error" not in workflow, "Deslop failures must not be masked"
assert "if-no-files-found: ignore" not in workflow, "missing reports must fail closed"
assert "Nimblesite/Deslop@v0.30.0" not in workflow, "glibc-incompatible action remains active"

print("deslop workflow static test: PASS")
