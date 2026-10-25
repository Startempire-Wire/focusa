#!/usr/bin/env python3
"""Provider-gated orchestration for the Spec 83 Pi performance benchmark."""
from __future__ import annotations

import json
import math
import os
import platform
import re
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from typing import Any
from urllib.parse import urlsplit

import spec83_perf_runner as runtime

ROOT = Path(__file__).resolve().parents[1]
RUNNER = Path(runtime.__file__).resolve()
SAFE_REF = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._:/#-]{0,199}$")


def unavailable(reason: str, output_path: str) -> int:
    result = {
        "schema": "focusa.spec83_benchmark_result.v1",
        "benchmark": "pi_focusa_rpc_efficiency",
        "status": "unavailable",
        "performance_evidence": False,
        "reason": reason,
        "provider_calls": 0,
    }
    try:
        runtime.write_json(output_path, result)
    except FileExistsError:
        result["output_write"] = "refused_existing_file"
    print(json.dumps(result, sort_keys=True))
    return 2


def evidence_output_path(path: str, project_root: Path = ROOT) -> bool:
    docs = project_root / "docs"
    evidence = docs / "evidence"
    if docs.is_symlink() or evidence.is_symlink():
        return False
    candidate = Path(path)
    if not candidate.is_absolute():
        candidate = project_root / candidate
    candidate = Path(os.path.abspath(candidate))
    evidence = Path(os.path.abspath(evidence))
    return (
        candidate.parent == evidence
        and candidate.name.startswith("spec83-perf-")
        and candidate.suffix == ".json"
        and not candidate.exists()
        and not candidate.is_symlink()
    )


def positive_env(name: str, default: str) -> float:
    value = float(os.environ.get(name, default))
    if not math.isfinite(value) or value <= 0:
        raise ValueError(f"{name} must be finite and greater than zero")
    return value


def safe_endpoint_label(url: str) -> str | None:
    parsed = urlsplit(url)
    if parsed.scheme not in ("http", "https") or not parsed.hostname or parsed.path != "/v1/status" or parsed.username or parsed.password or parsed.query or parsed.fragment:
        return None
    port = f":{parsed.port}" if parsed.port else ""
    return f"{parsed.scheme}://{parsed.hostname}{port}{parsed.path}"


def run_pi_version(program: str) -> str:
    result = subprocess.run([program, "--version"], check=False, capture_output=True, text=True, timeout=8)
    version = (result.stdout or result.stderr).strip()
    if result.returncode != 0 or not version:
        raise RuntimeError("pi_runtime_version_unavailable")
    return version[:200]


def run_arm_isolated(
    label: str,
    program: str,
    arguments: list[str],
    extension_path: str | None,
    timeout_seconds: float,
    directory: Path,
    env: dict[str, str],
) -> dict[str, Any]:
    timing = directory / f"{label}.json"
    command = [sys.executable, str(RUNNER), "arm", "--label", label, "--program", program,
               "--timing", str(timing), "--timeout-seconds", str(timeout_seconds)]
    if extension_path:
        command.extend(["--extension-path", extension_path])
    command.extend(f"--arg={argument}" for argument in arguments)
    try:
        completed = subprocess.run(command, capture_output=True, text=True, env=env, timeout=timeout_seconds + 15)
    except subprocess.TimeoutExpired:
        return {"label": label, "ok": False, "errors": ["runner_timeout"], "elapsed_s": None,
                "cpu_pct": None, "maxrss_kb": None}
    if not timing.exists():
        return {"label": label, "ok": False, "errors": [f"runner_exit:{completed.returncode}"],
                "elapsed_s": None, "cpu_pct": None, "maxrss_kb": None,
                "runner_error_bytes": len(completed.stderr.encode())}
    try:
        result = json.loads(timing.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return {"label": label, "ok": False, "errors": ["runner_result_invalid"],
                "elapsed_s": None, "cpu_pct": None, "maxrss_kb": None}
    return result if isinstance(result, dict) else {
        "label": label, "ok": False, "errors": ["runner_result_invalid"],
        "elapsed_s": None, "cpu_pct": None, "maxrss_kb": None,
    }


def benchmark(output_path: str) -> int:
    if os.environ.get("SPEC83_LIVE_PROVIDER") != "1":
        return unavailable("live_provider_evaluation_not_explicitly_authorized", output_path)
    if not evidence_output_path(output_path):
        return unavailable("live_evidence_output_must_be_a_new_spec83_json_file_in_docs_evidence", output_path)
    program = os.environ.get("FOCUSA_PI_BIN", "").strip()
    model_id = os.environ.get("SPEC83_MODEL_ID", "").strip()
    authority_ref = os.environ.get("SPEC83_PROVIDER_AUTH_REF", "").strip()
    status_url = os.environ.get("FOCUSA_STATUS_URL", "").strip()
    budget_raw = os.environ.get("SPEC83_PROVIDER_BUDGET_USD", "").strip()
    if not all((program, model_id, authority_ref, status_url, budget_raw)):
        return unavailable("live_provider_requires_pi_model_authority_budget_and_status_url", output_path)
    if not SAFE_REF.fullmatch(model_id) or not SAFE_REF.fullmatch(authority_ref):
        return unavailable("model_id_or_provider_authority_reference_invalid", output_path)
    safe_url = safe_endpoint_label(status_url)
    if not safe_url:
        return unavailable("status_url_invalid_or_contains_credentials", output_path)
    try:
        budget = positive_env("SPEC83_PROVIDER_BUDGET_USD", "0")
        thresholds = {
            "max_arm_elapsed_s": positive_env("SPEC83_MAX_ARM_ELAPSED_S", "35"),
            "max_arm_rss_kb": positive_env("SPEC83_MAX_ARM_RSS_KB", "1048576"),
            "max_arm_cpu_pct": positive_env("SPEC83_MAX_ARM_CPU_PCT", "200"),
            "max_status_p95_ms": positive_env("SPEC83_MAX_STATUS_P95_MS", "1000"),
            "max_event_delta_s": positive_env("SPEC83_MAX_EVENT_DELTA_S", "5"),
            "max_polling_delta_s": positive_env("SPEC83_MAX_POLLING_DELTA_S", "5"),
        }
    except (ValueError, OverflowError):
        return unavailable("invalid_budget_or_threshold_configuration", output_path)
    extension_path = str((ROOT / "apps/pi-extension/src/index.ts").resolve())
    if not Path(extension_path).is_file():
        return unavailable("focusa_extension_entrypoint_missing", output_path)
    try:
        pi_version = run_pi_version(program)
    except (OSError, subprocess.TimeoutExpired, RuntimeError):
        return unavailable("pi_runtime_version_unavailable", output_path)

    extension = runtime.probe_extension(program, extension_path)
    status = runtime.probe_status(status_url)
    metadata = {
        "mode": "explicit_live_provider",
        "model_id": model_id,
        "pi_runtime_version": pi_version,
        "provider_authority_ref": authority_ref,
        "declared_provider_budget_usd": budget,
        "planned_provider_calls": 3,
        "measured_at_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "python": sys.version.split()[0],
        "platform": platform.platform(),
        "status_endpoint": safe_url,
    }
    preflight_failures = []
    if extension.get("status") != "healthy":
        preflight_failures.append("extension_registration_or_health")
    if status.get("status") != "passed":
        preflight_failures.append("status_probe_or_schema")
    elif float(status["p95_ms"]) > thresholds["max_status_p95_ms"]:
        preflight_failures.append("status_latency:p95_threshold")
    if preflight_failures:
        result = {
            "schema": "focusa.spec83_benchmark_result.v1", "benchmark": "pi_focusa_rpc_efficiency",
            "status": "failed", "performance_evidence": False, "provider_calls": 0,
            "extension_health": extension, "status_latency": status, "metadata": metadata,
            "failures": preflight_failures,
        }
        runtime.write_json(output_path, result)
        print(json.dumps(result, indent=2, sort_keys=True))
        return 1

    with tempfile.TemporaryDirectory(prefix="spec83-") as temporary:
        temp_path = Path(temporary)
        common = ["--no-session", "--print", "--no-context-files", "--no-prompt-templates",
                  "--no-skills", "--no-extensions", "--model", model_id]
        prompt = ["-p", "Reply with exactly: OK"]
        arms = {
            "none": run_arm_isolated("none", program, [*common, *prompt], None,
                thresholds["max_arm_elapsed_s"], temp_path, os.environ.copy()),
            "focusa_event_driven": run_arm_isolated("focusa_event_driven", program,
                [*common, "-e", extension_path, *prompt], extension_path,
                thresholds["max_arm_elapsed_s"], temp_path, os.environ.copy()),
        }
        polling_env = os.environ.copy()
        polling_env["FOCUSA_PI_BRIDGE_SYNC_MODE"] = "polling"
        polling_env["FOCUSA_PI_BRIDGE_POLL_MS"] = "5000"
        arms["focusa_polling"] = run_arm_isolated("focusa_polling", program,
            [*common, "-e", extension_path, *prompt], extension_path,
            thresholds["max_arm_elapsed_s"], temp_path, polling_env)
    result = runtime.aggregate_results(arms, status, extension, thresholds, metadata)
    result["provider_calls"] = 3
    runtime.write_json(output_path, result)
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if result["status"] == "passed" else 1


def main() -> int:
    import argparse

    parser = argparse.ArgumentParser()
    parser.add_argument("--output", default=os.environ.get("SPEC83_OUT_JSON", "/tmp/spec83_perf_metrics.json"))
    args = parser.parse_args()
    return benchmark(args.output)


if __name__ == "__main__":
    raise SystemExit(main())
