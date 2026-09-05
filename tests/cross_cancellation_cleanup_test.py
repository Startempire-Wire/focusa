#!/usr/bin/env python3
"""Cancellation ownership regression for every containerized cross build."""

from __future__ import annotations

import os
from pathlib import Path
import signal
import shutil
import subprocess
import tempfile
import textwrap
import time

from structured_contract_loader import load_contract_mapping

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
CI_WORKFLOW = ROOT / ".github/workflows/ci.yml"


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
        workflow = load_contract_mapping(ROOT / relative)
        for job in workflow["jobs"].values():
            steps = job.get("steps", [])
            for index, step in enumerate(steps):
                if CANONICAL_CALL not in step.get("run", ""):
                    continue
                finalizer = steps[index + 1]
                condition = finalizer.get("if", "")
                assert "always()" in condition, relative
                assert f"steps.{step['id']}.outcome != 'skipped'" in condition, relative
                assert f"steps.{step['id']}.outcome != ''" in condition, relative
                assert "cancelled()" not in condition, relative
                assert "run-cancellation-safe-cross.sh cleanup-owned --target" in finalizer["run"], relative
                assert not finalizer.get("continue-on-error", False), relative
    assert total == 9
    ci = CI_WORKFLOW.read_text(encoding="utf-8")
    assert "Cross-build exact-identity cancellation ownership (#543, blocking)" in ci
    assert "python3 tests/cross_cancellation_cleanup_test.py" in ci

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
                if [[ "${FAKE_DISAPPEAR_ON_INSPECT:-0}" == 1 ]]; then
                  : > "$FAKE_CONTAINER_STATE"
                  exit 1
                fi
                [[ -s "$FAKE_CONTAINER_STATE" ]] || exit 1
                if [[ "${FAKE_INSPECT_MISMATCH:-0}" == 1 ]]; then
                  printf '%s\n' 'different|9|other-job|other-target'
                else
                  printf '%s|%s|%s|%s\n' "$GITHUB_RUN_ID" "$GITHUB_RUN_ATTEMPT" "$GITHUB_JOB" "x86_64-unknown-linux-musl"
                fi
                ;;
              rm)
                id="${@: -1}"
                current="$(cat "$FAKE_CONTAINER_STATE")"
                [[ "$id" == "$current" ]]
                if [[ "${FAKE_DISAPPEAR_ON_RM:-0}" == 1 ]]; then
                  : > "$FAKE_CONTAINER_STATE"
                  exit 1
                fi
                printf '%s\n' "$id" >> "$FAKE_DOCKER_REMOVED"
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
        def cancellation_case(sig: int, expected_status: int) -> None:
            Path(env["FAKE_CROSS_READY"]).unlink(missing_ok=True)
            Path(env["FAKE_DOCKER_REMOVED"]).unlink(missing_ok=True)
            state.write_text("owned-container\n", encoding="utf-8")
            process = subprocess.Popen(
                [str(WRAPPER), "build", "--release", "--target",
                 "x86_64-unknown-linux-musl"],
                env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
            )
            deadline = time.monotonic() + 5
            while not Path(env["FAKE_CROSS_READY"]).exists() and time.monotonic() < deadline:
                time.sleep(0.02)
            assert Path(env["FAKE_CROSS_READY"]).exists(), "fake cross did not start"
            process.send_signal(sig)
            _, stderr = process.communicate(timeout=5)
            assert process.returncode == expected_status, (process.returncode, stderr)
            assert Path(env["FAKE_DOCKER_REMOVED"]).read_text().splitlines() == ["owned-container"]
            assert state.read_text() == ""

        cancellation_case(signal.SIGTERM, 143)
        cancellation_case(signal.SIGINT, 130)
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

        for race_variable in ("FAKE_DISAPPEAR_ON_INSPECT", "FAKE_DISAPPEAR_ON_RM"):
            state.write_text("already-cleaned-container\n", encoding="utf-8")
            race = subprocess.run(
                [
                    str(WRAPPER),
                    "build",
                    "--release",
                    "--target=x86_64-unknown-linux-musl",
                ],
                env={
                    **env,
                    "FAKE_CROSS_MODE": "success",
                    "FAKE_INSPECT_MISMATCH": "0",
                    race_variable: "1",
                },
                capture_output=True,
                text=True,
                timeout=5,
                check=False,
            )
            assert race.returncode == 0, (race_variable, race.returncode, race.stderr)
            assert state.read_text() == ""
            assert not Path(env["FAKE_DOCKER_REMOVED"]).exists()

        # Simulate residue after the original launching shell is no longer
        # available. The finalizer must use the same owner without any compiler.
        (fake_bin / "cross").rename(fake_bin / "cross.disabled")
        for tool in ("bash", "cat"):
            (fake_bin / tool).symlink_to(shutil.which(tool))
        final_env = {**env, "PATH": str(fake_bin)}
        finalize_args = [str(WRAPPER), "cleanup-owned", "--target=x86_64-unknown-linux-musl"]
        state.write_text("orphaned-owned-container\n", encoding="utf-8")
        for _ in range(2):
            result = subprocess.run(finalize_args, env=final_env, capture_output=True,
                                    text=True, timeout=5, check=False)
            assert result.returncode == 0, result.stderr
            assert "cross cleanup verified:" in result.stdout
            assert state.read_text() == ""
        assert Path(env["FAKE_DOCKER_REMOVED"]).read_text().splitlines() == ["orphaned-owned-container"]
        Path(env["FAKE_DOCKER_REMOVED"]).unlink()
        state.write_text("unrelated-container\n", encoding="utf-8")
        refused = subprocess.run(finalize_args, env={**final_env, "FAKE_INSPECT_MISMATCH": "1"},
                                 capture_output=True, text=True, timeout=5, check=False)
        assert refused.returncode != 0 and "mismatched exact identity" in refused.stderr
        assert state.read_text() == "unrelated-container\n"
        assert not Path(env["FAKE_DOCKER_REMOVED"]).exists()
        invalid = subprocess.run(finalize_args + ["--all"], env=final_env,
                                 capture_output=True, text=True, timeout=5, check=False)
        assert invalid.returncode == 64 and state.read_text() == "unrelated-container\n"

    print("cross cancellation cleanup ownership and workflow finalization: PASS")


if __name__ == "__main__":
    main()
