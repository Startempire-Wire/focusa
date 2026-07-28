#!/usr/bin/env python3
"""Block canonical release work when KH or OVH authority resources are unsafe."""

import argparse
import json
import os
import sys
import urllib.request
from pathlib import Path

DEFAULT_LOCAL = "http://127.0.0.1:8791"
DEFAULT_MASTER = "http://[fd7a:115c:a1e0::e539:8453]:8791"


def health(url: str) -> dict:
    with urllib.request.urlopen(url.rstrip("/") + "/v1/health", timeout=10) as response:
        return json.loads(response.read())


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--max-disk-percent", type=float, default=90.0)
    parser.add_argument("--min-free-gb", type=float, default=15.0)
    parser.add_argument("--output")
    args = parser.parse_args()

    hosts = {
        "kh": health(os.environ.get("AGENT_KB_API_URL", DEFAULT_LOCAL)),
        "ovh": health(os.environ.get("AGENT_KB_MASTER_URL", DEFAULT_MASTER)),
    }
    checks = []
    for host, payload in hosts.items():
        disk = payload.get("disk", {})
        used = float(disk.get("used_percent", 100.0))
        free_gb = float(disk.get("available_bytes", 0)) / (1024**3)
        pending = int(payload.get("release_journal", {}).get("replication_pending", 1))
        reasons = []
        if used >= args.max_disk_percent:
            reasons.append(f"disk_used_percent={used:.2f}>={args.max_disk_percent:.2f}")
        if free_gb < args.min_free_gb:
            reasons.append(f"free_gb={free_gb:.2f}<{args.min_free_gb:.2f}")
        if pending != 0:
            reasons.append(f"replication_pending={pending}")
        checks.append(
            {
                "host": host,
                "status": "passed" if not reasons else "blocked",
                "disk_used_percent": round(used, 3),
                "free_gb": round(free_gb, 3),
                "replication_pending": pending,
                "reasons": reasons,
            }
        )
    result = {
        "schema": "focusa.release_resource_gate.v1",
        "status": "passed" if all(row["status"] == "passed" for row in checks) else "blocked",
        "thresholds": {"max_disk_percent": args.max_disk_percent, "min_free_gb": args.min_free_gb},
        "hosts": checks,
    }
    if args.output:
        Path(args.output).write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if result["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
