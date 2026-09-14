#!/usr/bin/env python3
"""Validate GitHub workflow job dependency graphs.

This is intentionally lightweight and dependency-free enough for CI/static guards.
It catches workflow-file failures GitHub otherwise reports opaquely as
"No jobs were run" / "workflow file issue" before any job logs exist.
"""

from __future__ import annotations

import sys
from pathlib import Path
from typing import Any

try:
    import yaml  # type: ignore
except Exception as exc:  # pragma: no cover - CI image provides PyYAML
    print(f"validate-github-workflows: PyYAML unavailable: {exc}", file=sys.stderr)
    sys.exit(2)


def as_list(value: Any) -> list[str]:
    if value is None:
        return []
    if isinstance(value, str):
        return [value]
    if isinstance(value, list):
        return [str(v) for v in value]
    raise TypeError(f"unsupported needs value: {value!r}")


def validate(path: Path) -> list[str]:
    data = yaml.safe_load(path.read_text())
    errors: list[str] = []
    if not isinstance(data, dict):
        return [f"{path}: workflow is not a mapping"]
    jobs = data.get("jobs")
    if not isinstance(jobs, dict) or not jobs:
        return [f"{path}: workflow has no jobs"]
    job_ids = set(jobs)
    runnable_without_needs = []
    for job_id, job in jobs.items():
        if not isinstance(job, dict):
            errors.append(f"{path}: job {job_id} is not a mapping")
            continue
        needs = as_list(job.get("needs"))
        if not needs:
            runnable_without_needs.append(job_id)
        for need in needs:
            if need not in job_ids:
                errors.append(f"{path}: job {job_id} needs missing job {need}")
    if not runnable_without_needs:
        errors.append(f"{path}: workflow has no root job without needs")
    return errors


def main(argv: list[str]) -> int:
    paths = [Path(a) for a in argv[1:]] or sorted(
        Path(".github/workflows").glob("*.yml")
    )
    all_errors: list[str] = []
    for path in paths:
        all_errors.extend(validate(path))
    if all_errors:
        for err in all_errors:
            print(err, file=sys.stderr)
        return 1
    print("github_workflow_graph_validation=ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
