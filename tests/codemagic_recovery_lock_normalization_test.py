#!/usr/bin/env python3
"""Exercise the recovery-only Cargo.lock proof embedded in codemagic.yaml."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import textwrap
from pathlib import Path

import yaml

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


def exact_provider_identity():
    workflows = yaml.safe_load(CODEMAGIC)['workflows']
    script = workflows['menubar-macos-package-proof']['scripts'][0]['script']
    assert workflows['rust-macos-release-binaries']['scripts'][0]['script'] == script
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        origin, consumer = root / 'origin', root / 'consumer'
        subprocess.run(['git', 'init', '-q', str(origin)], check=True)
        git = ['git', '-C', str(origin), '-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid']
        subprocess.run([*git, 'commit', '-q', '--allow-empty', '-m', 'version fixture'], check=True)
        sha = subprocess.check_output([*git, 'rev-parse', 'HEAD'], text=True).strip()
        for tag in ('v0.9.212', 'v0.9.214-dev', 'v0.9.214-nightly'):
            subprocess.run([*git, 'tag', tag, sha], check=True)
        subprocess.run(['git', 'clone', '-q', '--no-tags', str(origin), str(consumer)], check=True)
        (consumer / 'config').mkdir()
        recovery = consumer / 'config/codemagic-release-recovery.json'
        journal = root / 'cm.env'
        for tag, commit, use_recovery, passes in (
            ('v0.9.212', sha, False, True),
            ('v0.9.214-dev', sha, False, True),
            ('v0.9.214-dev', sha, True, True),
            ('v0.9.214-dev', '0' * 40, False, False),
            ('v0.9.214-dev', '0' * 40, True, False),
            ('v0.9.214-nightly', sha, False, False),
            ('v0.9.214-rc', sha, False, False),
            ('v0.9.215-dev', sha, False, False),
            ('', sha, False, False),
        ):
            journal.write_text('')
            recovery.write_text(json.dumps({'enabled': True, 'tag': tag, 'sha': commit}))
            env = {key: os.environ[key] for key in ('PATH', 'HOME') if key in os.environ}
            env.update(CM_TAG='' if use_recovery else tag, CM_COMMIT='' if use_recovery else commit,
                       FOCUSA_CODEMAGIC_RECOVERY='enabled' if use_recovery else 'disabled', CM_ENV=str(journal))
            result = subprocess.run(['bash', '-c', script], cwd=consumer, env=env,
                                    capture_output=True, text=True, check=False, timeout=10)
            assert (result.returncode == 0) == passes, (tag, use_recovery, result.stderr)
            expected = f'FOCUSA_RELEASE_TAG={tag}\nFOCUSA_RELEASE_SHA={sha}\n' if passes else ''
            assert journal.read_text() == expected


exact_provider_identity()
print('PASS: both Codemagic producers bind stable/dev and recovery to identical immutable ref/SHA checks')
