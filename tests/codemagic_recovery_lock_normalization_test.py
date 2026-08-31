#!/usr/bin/env python3
"""Exercise the recovery-only Cargo.lock proof embedded in codemagic.yaml."""

from __future__ import annotations

import subprocess
import sys
import tempfile
import textwrap
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CODEMAGIC = (ROOT / "codemagic.yaml").read_text(encoding="utf-8")
START_MARKER = (
    '            python3 - "$lock_before" Cargo.lock '
    '"${FOCUSA_RELEASE_TAG#v}" <<\'PY\'\n'
)
END_MARKER = "\n          PY"

start = CODEMAGIC.index(START_MARKER) + len(START_MARKER)
end = CODEMAGIC.index(END_MARKER, start)
PROOF = textwrap.dedent(CODEMAGIC[start:end])

ALLOWED = [
    "agent-stateful-cognitive-runtime",
    "cognitive-state-projection",
    "letta-adapter",
    "pi-client-tool-gateway",
]


def package(name: str, version: str, *, checksum: str | None = None) -> str:
    lines = ["[[package]]", f'name = "{name}"', f'version = "{version}"']
    if checksum is not None:
        lines.extend(
            [
                'source = "registry+https://github.com/rust-lang/crates.io-index"',
                f'checksum = "{checksum}"',
            ]
        )
    lines.append("dependencies = []")
    return "\n".join(lines) + "\n"


def lock(version: str, *, external_checksum: str = "abc", local_dependency=False) -> str:
    records = ['version = 4\n']
    for name in ALLOWED:
        record = package(name, version)
        if local_dependency and name == ALLOWED[0]:
            record = record.replace("dependencies = []", 'dependencies = ["serde"]')
        records.append(record)
    records.append(package("serde", "1.0.228", checksum=external_checksum))
    return "\n".join(records)


def run(before_text: str, after_text: str) -> subprocess.CompletedProcess[str]:
    with tempfile.TemporaryDirectory() as tmp:
        before = Path(tmp) / "before.lock"
        after = Path(tmp) / "after.lock"
        before.write_text(before_text, encoding="utf-8")
        after.write_text(after_text, encoding="utf-8")
        return subprocess.run(
            [sys.executable, "-", str(before), str(after), "0.9.187"],
            input=PROOF,
            text=True,
            capture_output=True,
            check=False,
        )


valid = run(lock("0.9.186"), lock("0.9.187"))
assert valid.returncode == 0, valid.stderr
assert "codemagic_recovery_lock_normalization=passed" in valid.stdout

external_drift = run(
    lock("0.9.186", external_checksum="abc"),
    lock("0.9.187", external_checksum="def"),
)
assert external_drift.returncode != 0

non_version_drift = run(lock("0.9.186"), lock("0.9.187", local_dependency=True))
assert non_version_drift.returncode != 0

wrong_source_version = run(lock("0.9.185"), lock("0.9.187"))
assert wrong_source_version.returncode != 0

print("PASS: Codemagic recovery lock proof accepts only exact local version drift")
