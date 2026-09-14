#!/usr/bin/env python3
"""Run the canonical advertised-route audit against an isolated Focusa daemon."""

from __future__ import annotations

import os
import socket
import subprocess
import sys
import tempfile
import time
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DAEMON = Path(os.environ.get("FOCUSA_DAEMON_BIN", ROOT / "target/debug/focusa-daemon"))
AUDIT = ROOT / "scripts/audit-route-health.mjs"


def reserve_port() -> int:
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


def main() -> int:
    if not DAEMON.is_file():
        print(f"route-health: daemon binary not found: {DAEMON}", file=sys.stderr)
        return 2

    classifier = subprocess.run(
        ["node", "--test", str(ROOT / "tests/609-route-health-classifier.test.mjs")],
        cwd=ROOT,
        check=False,
    )
    if classifier.returncode:
        return classifier.returncode

    port = reserve_port()
    with tempfile.TemporaryDirectory(prefix="focusa-route-health-") as data_dir:
        log_path = Path(data_dir) / "daemon.log"
        env = os.environ.copy()
        env.update(
            FOCUSA_BIND=f"127.0.0.1:{port}",
            FOCUSA_DATA_DIR=data_dir,
            FOCUSA_TEST_MODE="1",
        )
        with log_path.open("wb") as log:
            daemon = subprocess.Popen([str(DAEMON)], cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT)
        try:
            health = f"http://127.0.0.1:{port}/v1/health"
            for _ in range(80):
                if daemon.poll() is not None:
                    break
                try:
                    with urllib.request.urlopen(health, timeout=1) as response:
                        if response.status == 200:
                            break
                except OSError:
                    time.sleep(0.25)
            else:
                print("route-health: isolated daemon did not become healthy", file=sys.stderr)
                return 3

            if daemon.poll() is not None:
                print(log_path.read_text(errors="replace"), file=sys.stderr)
                return 4

            audit_env = os.environ.copy()
            audit_env["FOCUSA_API_BASE"] = f"http://127.0.0.1:{port}/v1"
            result = subprocess.run(["node", str(AUDIT)], cwd=ROOT, env=audit_env, check=False)
            return result.returncode
        finally:
            daemon.terminate()
            try:
                daemon.wait(timeout=5)
            except subprocess.TimeoutExpired:
                daemon.kill()
                daemon.wait(timeout=5)
            if daemon.returncode not in (0, -15):
                print(log_path.read_text(errors="replace"), file=sys.stderr)


if __name__ == "__main__":
    raise SystemExit(main())
