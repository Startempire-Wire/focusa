#!/usr/bin/env python3
"""SPEC80 runtime performance gate harnesses.

Implements executable gates from Spec80 §20.2:
- Reflection+metacog added latency <= 12% (p95)
- Snapshot restore p95 <= 400ms
- Compaction p95 <= 1.5x pre-branch baseline
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


SAMPLE_FLOOR = 200


def load_jsonl(path: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    with path.open("r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            rows.append(json.loads(line))
    return rows


def p95_nearest_rank(values: list[float]) -> float:
    if not values:
        return 0.0
    sorted_vals = sorted(values)
    rank = max(1, int((0.95 * len(sorted_vals)) + 0.999999))
    idx = min(rank - 1, len(sorted_vals) - 1)
    return float(sorted_vals[idx])


def run_reflection_gate(rows: list[dict[str, Any]]) -> dict[str, Any]:
    baseline = [float(r["latency_ms"]) for r in rows if r.get("mode") == "baseline"]
    treatment = [float(r["latency_ms"]) for r in rows if r.get("mode") == "with_metacog"]

    baseline_ok = len(baseline) >= SAMPLE_FLOOR
    treatment_ok = len(treatment) >= SAMPLE_FLOOR
    if not (baseline_ok and treatment_ok):
        return {
            "gate_id": "D3.1-latency",
            "decision": "insufficient_sample",
            "sample_floor": SAMPLE_FLOOR,
            "baseline_count": len(baseline),
            "treatment_count": len(treatment),
        }

    p95_baseline = p95_nearest_rank(baseline)
    p95_treatment = p95_nearest_rank(treatment)
    added_ratio = 0.0 if p95_baseline == 0 else (p95_treatment - p95_baseline) / p95_baseline
    return {
        "gate_id": "D3.1-latency",
        "decision": "pass" if added_ratio <= 0.12 else "fail",
        "p95_baseline_ms": p95_baseline,
        "p95_with_metacog_ms": p95_treatment,
        "added_latency_ratio": added_ratio,
        "threshold_ratio": 0.12,
        "sample_floor": SAMPLE_FLOOR,
    }


def run_restore_compaction_gate(rows: list[dict[str, Any]]) -> dict[str, Any]:
    restore = [float(r["latency_ms"]) for r in rows if r.get("operation") == "restore"]
    prebranch = [
        float(r["latency_ms"])
        for r in rows
        if r.get("operation") == "compaction" and r.get("profile") == "prebranch"
    ]
    branch = [
        float(r["latency_ms"])
        for r in rows
        if r.get("operation") == "compaction" and r.get("profile") == "branch"
    ]

    if len(restore) < SAMPLE_FLOOR or len(prebranch) < SAMPLE_FLOOR or len(branch) < SAMPLE_FLOOR:
        return {
            "gate_id": "D3.2-restore-compaction",
            "decision": "insufficient_sample",
            "sample_floor": SAMPLE_FLOOR,
            "restore_count": len(restore),
            "prebranch_count": len(prebranch),
            "branch_count": len(branch),
        }

    restore_p95 = p95_nearest_rank(restore)
    pre_p95 = p95_nearest_rank(prebranch)
    branch_p95 = p95_nearest_rank(branch)
    ratio = 0.0 if pre_p95 == 0 else (branch_p95 / pre_p95)

    restore_pass = restore_p95 <= 400.0
    compaction_pass = ratio <= 1.5
    return {
        "gate_id": "D3.2-restore-compaction",
        "decision": "pass" if (restore_pass and compaction_pass) else "fail",
        "restore": {
            "p95_ms": restore_p95,
            "threshold_ms": 400.0,
            "pass": restore_pass,
        },
        "compaction": {
            "prebranch_p95_ms": pre_p95,
            "branch_p95_ms": branch_p95,
            "ratio": ratio,
            "threshold_ratio": 1.5,
            "pass": compaction_pass,
        },
        "sample_floor": SAMPLE_FLOOR,
    }


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("gate", choices=["reflection-latency", "restore-compaction"])
    ap.add_argument("--input", required=True)
    ap.add_argument("--output", required=True)
    args = ap.parse_args()

    rows = load_jsonl(Path(args.input))
    if args.gate == "reflection-latency":
        result = run_reflection_gate(rows)
    else:
        result = run_restore_compaction_gate(rows)

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result))


if __name__ == "__main__":
    main()
