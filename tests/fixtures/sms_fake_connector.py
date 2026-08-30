#!/usr/bin/env python3
import argparse
from pathlib import Path
import signal
import time

parser = argparse.ArgumentParser()
parser.add_argument("--profile", type=Path, required=True)
parser.add_argument("--port", required=True)
args = parser.parse_args()
(args.profile / "Default").mkdir(mode=0o700, parents=True, exist_ok=True)
(args.profile / "Default" / "Cookies").write_bytes(b"synthetic-connector-state")
(args.profile / "connector.ready").write_text(args.port)
for path in (args.profile / "Default" / "Cookies", args.profile / "connector.ready"):
    path.chmod(0o600)
running = True
signal.signal(signal.SIGTERM, lambda *_: globals().__setitem__("running", False))
while running:
    time.sleep(0.05)
