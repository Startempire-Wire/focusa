#!/usr/bin/env python3
"""Deterministic local Pi stand-in for Spec 83 contract tests."""
from __future__ import annotations

import json
import os
import sys
import time

mode = os.environ.get("SPEC83_FIXTURE_MODE", "ok")
if "--mode" in sys.argv and "rpc" in sys.argv:
    request = sys.stdin.readline()
    if json.loads(request).get("type") != "get_commands":
        raise SystemExit(3)
    commands = [] if mode == "extension-fail" else [{"name": "focusa-status", "source": "extension"}]
    print(json.dumps({"type": "response", "command": "get_commands", "success": True, "data": {"commands": commands}}), flush=True)
    if mode == "extension-fail":
        print("Failed to load extension Focusa", file=sys.stderr, flush=True)
    while True:
        time.sleep(1)
elif mode == "timeout":
    time.sleep(2)
elif mode == "fail":
    print("OK")
    raise SystemExit(7)
elif mode == "wrong-output":
    print("NOT-OK")
    raise SystemExit(0)
elif mode == "extension-fail":
    print("OK")
    print("Failed to load extension Focusa", file=sys.stderr)
    raise SystemExit(0)
else:
    print("OK")
    raise SystemExit(0)
