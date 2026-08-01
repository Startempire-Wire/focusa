#!/usr/bin/env python3
"""Deterministic Spec144 §28 fixture evaluator (standard library only)."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any

SIX_COHORTS = (
    "builder_only",
    "same_model_self_review",
    "same_model_separate_context",
    "cross_family_verification",
    "deterministic_model_verification",
    "multi_aspect_portfolio",
)
REQUIRED_HASHES = {
    "code", "model", "prompt", "policy", "registry", "pack", "shape", "data",
    "environment", "source_144", "source_135_137_138",
}
REQUIRED_OPERATIONAL_METRICS = {
    "task_success", "criterion_coverage", "defect_escape_rate", "reproduction_rate",
    "repair_rate", "mean_rounds", "oscillation_rate", "anchoring_rate",
    "evidence_linkage_rate", "unsupported_objection_rate", "overfitting_rate",
    "routing_eligibility_rate", "unnecessary_verification_rate", "temporal_error_rate",
    "unsupported_estimate_rejection_rate", "information_leakage_rate",
    "source_dependence_error_rate", "resolution_scoring_error_rate",
    "learning_promotion_reversal_rate", "negative_transfer_rate", "operator_interventions",
    "post_settlement_regressions",
}


def _ratio(numerator: int, denominator: int) -> float:
    return numerator / denominator if denominator else 0.0


def load_fixture(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def validate_hashes(document: dict[str, Any], fixture_dir: Path) -> None:
    hashes = document["artifact_hashes"]
    missing = REQUIRED_HASHES - hashes.keys()
    if missing:
        raise ValueError(f"missing exact artifact hashes: {sorted(missing)}")
    for name, record in hashes.items():
        target = (fixture_dir / record["path"]).resolve()
        if not target.is_file():
            raise ValueError(f"hashed artifact does not exist: {name}")
        actual = hashlib.sha256(target.read_bytes()).hexdigest()
        if actual != record["sha256"]:
            raise ValueError(f"artifact hash mismatch: {name}")


def evaluate(document: dict[str, Any]) -> dict[str, dict[str, Any]]:
    if tuple(document["cohorts"]) != SIX_COHORTS:
        raise ValueError("the six comparison cohorts must be present in canonical order")
    scenarios = document["scenarios"]
    if len(scenarios) != 25 or len({s["id"] for s in scenarios}) != 25:
        raise ValueError("exactly 25 uniquely named golden scenarios are required")
    mandatory = {s["id"] for s in scenarios if s["mandatory"]}
    observations = document["observations"]
    aggregates = {row["cohort"]: row for row in document["aggregates"]}
    results: dict[str, dict[str, Any]] = {}
    for cohort in SIX_COHORTS:
        row = aggregates[cohort]
        missing = REQUIRED_OPERATIONAL_METRICS - row.keys()
        if missing:
            raise ValueError(f"{cohort} lacks §28 metrics: {sorted(missing)}")
        tp, fp, fn, tn = (int(row[k]) for k in ("tp", "fp", "fn", "tn"))
        cohort_obs = [o for o in observations if o["cohort"] == cohort]
        observed_ids = {o["scenario"] for o in cohort_obs}
        if observed_ids != mandatory or len(cohort_obs) != 25:
            raise ValueError(f"{cohort} does not execute all golden scenarios")
        golden_passes = sum(o["expected"] == o["predicted"] for o in cohort_obs)
        results[cohort] = {
            "precision": _ratio(tp, tp + fp),
            "recall": _ratio(tp, tp + fn),
            "false_positive_rate": _ratio(fp, fp + tn),
            "false_negative_rate": _ratio(fn, fn + tp),
            "coverage": _ratio(row["requirements_covered"], row["requirements_total"]),
            "calibration_ece": float(row["calibration_ece"]),
            "p95_latency_ms": int(row["p95_latency_ms"]),
            "resource_units": int(row["resource_units"]),
            "replay_equivalence": bool(row["replay_equivalent"]) and all(
                o["replay_output_hash"] == o["replay_repeat_hash"] for o in cohort_obs
            ),
            "golden_pass_rate": golden_passes / len(cohort_obs),
            "blocking_failures": sum(
                o["scenario"] in document["blocking_scenarios"]
                and o["expected"] != o["predicted"] for o in cohort_obs
            ),
            "operational": {key: row[key] for key in sorted(REQUIRED_OPERATIONAL_METRICS)},
        }
    return results


def promotion_decision(document: dict[str, Any], result: dict[str, Any]) -> tuple[bool, list[str]]:
    thresholds = document["promotion"]["thresholds"]
    failures: list[str] = []
    checks = {
        "precision_min": result["precision"] >= thresholds["precision_min"],
        "recall_min": result["recall"] >= thresholds["recall_min"],
        "false_positive_rate_max": result["false_positive_rate"] <= thresholds["false_positive_rate_max"],
        "false_negative_rate_max": result["false_negative_rate"] <= thresholds["false_negative_rate_max"],
        "coverage_min": result["coverage"] >= thresholds["coverage_min"],
        "calibration_ece_max": result["calibration_ece"] <= thresholds["calibration_ece_max"],
        "p95_latency_ms_max": result["p95_latency_ms"] <= thresholds["p95_latency_ms_max"],
        "resource_units_max": result["resource_units"] <= thresholds["resource_units_max"],
        "replay_equivalence_required": result["replay_equivalence"] is thresholds["replay_equivalence_required"],
        "golden_pass_rate_min": result["golden_pass_rate"] >= thresholds["golden_pass_rate_min"],
        "blocking_failures_max": result["blocking_failures"] <= thresholds["blocking_failures_max"],
    }
    failures.extend(name for name, passed in checks.items() if not passed)
    return not failures, failures


def run(fixture: Path) -> dict[str, Any]:
    document = load_fixture(fixture)
    validate_hashes(document, fixture.parent)
    results = evaluate(document)
    candidate = document["promotion"]["candidate"]
    eligible, failures = promotion_decision(document, results[candidate])
    if eligible != document["promotion"]["expected_eligible"]:
        raise ValueError("promotion result differs from fixture expectation")
    return {"schema": document["schema"], "cohorts": results, "promotion": {"candidate": candidate, "eligible": eligible, "failures": failures}}


if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("fixture", type=Path)
    args = parser.parse_args()
    print(json.dumps(run(args.fixture), indent=2, sort_keys=True))
