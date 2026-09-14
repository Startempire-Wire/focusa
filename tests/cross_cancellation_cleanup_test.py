#!/usr/bin/env python3
"""Cancellation ownership regression for every containerized cross build."""

from __future__ import annotations

import os
from pathlib import Path
import signal
import subprocess
import tempfile
import textwrap
import time

ROOT = Path(__file__).resolve().parents[1]
WRAPPER = ROOT / "scripts/ci/run-cancellation-safe-cross.sh"
WORKFLOWS = {
    ".github/workflows/nightly.yml": 1,
    ".github/workflows/warmup.yml": 2,
    ".github/workflows/release.yml": 2,
    ".github/workflows/locked-release-candidate-artifacts.yml": 1,
    ".github/workflows/spec132-terminal-matrix.yml": 3,
}
CANONICAL_CALL = "scripts/ci/run-cancellation-safe-cross.sh build --release --target"


def write_executable(path: Path, body: str) -> None:
    path.write_text(textwrap.dedent(body).lstrip(), encoding="utf-8")
    path.chmod(0o755)


def base_environment(fake_bin: Path, state: Path) -> dict[str, str]:
    return {
        **os.environ,
        "PATH": f"{fake_bin}:{os.environ['PATH']}",
        "CROSS_CONTAINER_ENGINE": "docker",
        "GITHUB_RUN_ID": "33818088969",
        "GITHUB_RUN_ATTEMPT": "1",
        "GITHUB_JOB": "nightly",
        "FAKE_CONTAINER_STATE": str(state),
        "FAKE_CROSS_READY": str(fake_bin / "cross.ready"),
        "FAKE_CROSS_OPTS": str(fake_bin / "cross.opts"),
        "FAKE_DOCKER_REMOVED": str(fake_bin / "docker.removed"),
    }


def main() -> None:
    assert WRAPPER.is_file() and os.access(WRAPPER, os.X_OK)
    total = 0
    for relative, expected in WORKFLOWS.items():
        text = (ROOT / relative).read_text(encoding="utf-8")
        assert "cross build --release" not in text, relative
        actual = text.count(CANONICAL_CALL)
        assert actual == expected, f"{relative}: expected {expected} canonical calls, got {actual}"
        total += actual
    assert total == 9

    with tempfile.TemporaryDirectory(prefix="focusa-cross-cleanup-") as raw:
        temp = Path(raw)
        fake_bin = temp / "bin"
        fake_bin.mkdir()
        state = temp / "container.state"

        write_executable(
            fake_bin / "cross",
            """
            #!/usr/bin/env bash
            set -euo pipefail
            printf '%s\n' "$CROSS_CONTAINER_OPTS" > "$FAKE_CROSS_OPTS"
            if [[ "${FAKE_CROSS_MODE:-success}" == block ]]; then
              : > "$FAKE_CROSS_READY"
              exec sleep 300
            fi
            """,
        )
        write_executable(
            fake_bin / "docker",
            """
            #!/usr/bin/env bash
            set -euo pipefail
            command_name="${1:-}"
            case "$command_name" in
              ps)
                if [[ -s "$FAKE_CONTAINER_STATE" ]]; then
                  cat "$FAKE_CONTAINER_STATE"
                fi
                ;;
              inspect)
                if [[ "${FAKE_INSPECT_MISMATCH:-0}" == 1 ]]; then
                  printf '%s\n' 'different|9|other-job|other-target'
                else
                  printf '%s|%s|%s|%s\n' "$GITHUB_RUN_ID" "$GITHUB_RUN_ATTEMPT" "$GITHUB_JOB" "x86_64-unknown-linux-musl"
                fi
                ;;
              rm)
                id="${@: -1}"
                printf '%s\n' "$id" >> "$FAKE_DOCKER_REMOVED"
                current="$(cat "$FAKE_CONTAINER_STATE")"
                [[ "$id" == "$current" ]]
                : > "$FAKE_CONTAINER_STATE"
                ;;
              *)
                echo "unexpected fake docker command: $*" >&2
                exit 90
                ;;
            esac
            """,
        )

        env = base_environment(fake_bin, state)
        env["FAKE_CROSS_MODE"] = "block"
        state.write_text("owned-container\n", encoding="utf-8")
        process = subprocess.Popen(
            [
                str(WRAPPER),
                "build",
                "--release",
                "--target",
                "x86_64-unknown-linux-musl",
            ],
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        deadline = time.monotonic() + 5
        while not Path(env["FAKE_CROSS_READY"]).exists() and time.monotonic() < deadline:
            time.sleep(0.02)
        assert Path(env["FAKE_CROSS_READY"]).exists(), "fake cross did not start"
        process.send_signal(signal.SIGTERM)
        _, stderr = process.communicate(timeout=5)
        assert process.returncode == 143, (process.returncode, stderr)
        assert Path(env["FAKE_DOCKER_REMOVED"]).read_text().splitlines() == [
            "owned-container"
        ]
        options = Path(env["FAKE_CROSS_OPTS"]).read_text()
        for expected in (
            "--label=focusa.github.run_id=33818088969",
            "--label=focusa.github.run_attempt=1",
            "--label=focusa.github.job=nightly",
            "--label=focusa.cross.target=x86_64-unknown-linux-musl",
        ):
            assert expected in options
        assert state.read_text() == ""

        Path(env["FAKE_DOCKER_REMOVED"]).unlink()
        state.write_text("unrelated-container\n", encoding="utf-8")
        mismatch_env = {**env, "FAKE_CROSS_MODE": "success", "FAKE_INSPECT_MISMATCH": "1"}
        mismatch = subprocess.run(
            [
                str(WRAPPER),
                "build",
                "--release",
                "--target=x86_64-unknown-linux-musl",
            ],
            env=mismatch_env,
            capture_output=True,
            text=True,
            timeout=5,
            check=False,
        )
        assert mismatch.returncode != 0
        assert "refusing container with mismatched exact identity" in mismatch.stderr
        assert not Path(env["FAKE_DOCKER_REMOVED"]).exists()
        assert state.read_text().strip() == "unrelated-container"

    print("cross cancellation cleanup ownership: PASS")


if __name__ == "__main__":
    main()
