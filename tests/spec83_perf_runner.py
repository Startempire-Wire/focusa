#!/usr/bin/env python3
"""Portable, fail-closed Spec 83 child measurement and validation helpers."""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import platform
import re
import signal
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any

try:
    import resource
except ImportError:  # Windows has no resource module; unsupported metrics fail closed.
    resource = None

EXTENSION_COMMAND = "focusa-status"
REQUIRED_STATUS_COUNTERS = (
    "supervisor_ticks_total",
    "driver_start_attempts",
    "driver_stop_attempts",
    "dispatch_attempts",
    "dispatch_skipped_disallowed",
    "dispatch_recovery_restarts",
)
EXTENSION_LOAD_ERROR = re.compile(
    r"(?:failed|unable|error|cannot|could not).{0,80}(?:load|register).{0,40}extension|"
    r"extension.{0,80}(?:load failed|registration failed|failed to load|cannot load)",
    re.IGNORECASE | re.DOTALL,
)


def normalize_maxrss_kb(raw: int | float, platform_name: str) -> int | None:
    """Normalize getrusage's platform-specific ru_maxrss to KiB."""
    if platform_name == "darwin":
        return int(math.ceil(float(raw) / 1024))
    if platform_name.startswith("linux"):
        return int(raw)
    return None


def kill_process_tree(process: subprocess.Popen[bytes]) -> None:
    try:
        if os.name == "posix":
            os.killpg(process.pid, signal.SIGKILL)
        else:
            process.kill()
    except ProcessLookupError:
        pass


def resource_before() -> tuple[float, float] | None:
    if resource is None:
        return None
    usage = resource.getrusage(resource.RUSAGE_CHILDREN)
    return usage.ru_utime, usage.ru_stime


def resource_after(before: tuple[float, float] | None, elapsed_s: float) -> dict[str, Any]:
    if resource is None or before is None:
        return {"user_s": None, "sys_s": None, "cpu_pct": None, "maxrss_kb": None}
    usage = resource.getrusage(resource.RUSAGE_CHILDREN)
    user_s = max(0.0, usage.ru_utime - before[0])
    sys_s = max(0.0, usage.ru_stime - before[1])
    rss_kb = normalize_maxrss_kb(usage.ru_maxrss, sys.platform)
    if rss_kb is not None and rss_kb <= 0:
        rss_kb = None
    return {
        "user_s": round(user_s, 6),
        "sys_s": round(sys_s, 6),
        "cpu_pct": round((user_s + sys_s) / elapsed_s * 100, 2) if elapsed_s > 0 else 0.0,
        "maxrss_kb": rss_kb,
    }


def run_arm(
    *, label: str, program: str, arguments: list[str], timeout_seconds: float,
    extension_path: str | None = None, program_args: list[str] | None = None,
    env: dict[str, str] | None = None,
) -> dict[str, Any]:
    """Run one child, retaining exit/output/resource truth without shell tools."""
    errors: list[str] = []
    resolved_extension = str(Path(extension_path).resolve()) if extension_path else None
    if resolved_extension and not Path(resolved_extension).is_file():
        errors.append("extension_file_missing")
    before = resource_before()
    started = time.perf_counter()
    stdout = b""
    stderr = b""
    exit_code: int | None = None
    timed_out = False
    launch_error: str | None = None
    pid: int | None = None
    argv = [program, *(program_args or []), *arguments]
    popen_options: dict[str, Any] = {"stdout": subprocess.PIPE, "stderr": subprocess.PIPE}
    if os.name == "posix":
        popen_options["start_new_session"] = True
    elif hasattr(subprocess, "CREATE_NEW_PROCESS_GROUP"):
        popen_options["creationflags"] = subprocess.CREATE_NEW_PROCESS_GROUP
    try:
        child = subprocess.Popen(argv, env=env, **popen_options)
        pid = child.pid
        try:
            stdout, stderr = child.communicate(timeout=timeout_seconds)
            exit_code = child.returncode
        except subprocess.TimeoutExpired:
            timed_out = True
            kill_process_tree(child)
            stdout, stderr = child.communicate()
            exit_code = 124
    except OSError as exc:
        launch_error = f"launch_failed:{exc.__class__.__name__}"
        stderr = str(exc).encode("utf-8", errors="replace")
        exit_code = 127

    elapsed_s = time.perf_counter() - started
    resources = resource_after(before, elapsed_s)
    exact_output = stdout in (b"OK", b"OK\n")
    if launch_error:
        errors.append(launch_error)
    if timed_out:
        errors.append("timeout")
    if exit_code != 0:
        errors.append(f"exit_code:{exit_code}")
    if not exact_output:
        errors.append("output_mismatch")
    if resolved_extension and not Path(resolved_extension).is_file():
        errors.append("extension_file_missing")
    if resolved_extension and EXTENSION_LOAD_ERROR.search(stderr.decode("utf-8", errors="replace")):
        errors.append("extension_load_failure")
    if resources["cpu_pct"] is None:
        errors.append("cpu_pct_unavailable")
    if resources["maxrss_kb"] is None:
        errors.append("maxrss_unavailable")
    return {
        "label": label,
        "elapsed_s": round(elapsed_s, 6),
        **resources,
        "pid": pid,
        "exit_code": exit_code,
        "timed_out": timed_out,
        "output_ok": exact_output,
        "stdout_output": stdout.decode("utf-8", errors="replace") if exact_output else None,
        "extension_path": resolved_extension,
        "stdout_sha256": hashlib.sha256(stdout).hexdigest(),
        "stderr_sha256": hashlib.sha256(stderr).hexdigest(),
        "stdout_bytes": len(stdout),
        "stderr_bytes": len(stderr),
        "ok": not errors,
        "errors": errors,
        "environment": {"python": sys.version.split()[0], "platform": platform.platform(), "system": platform.system()},
    }


def probe_extension(
    program: str, extension_path: str, timeout_seconds: float = 8.0,
    program_args: list[str] | None = None,
) -> dict[str, Any]:
    """Use Pi RPC get_commands to prove the explicitly loaded extension registered."""
    path = str(Path(extension_path).resolve())
    if not Path(path).is_file():
        return {"status": "failed", "registered": False, "errors": ["extension_file_missing"]}
    argv = [program, *(program_args or []), "--mode", "rpc", "--no-session", "--no-context-files",
            "--no-prompt-templates", "--no-skills", "--no-extensions", "-e", path]
    options: dict[str, Any] = {"stdin": subprocess.PIPE, "stdout": subprocess.PIPE, "stderr": subprocess.PIPE}
    if os.name == "posix":
        options["start_new_session"] = True
    elif hasattr(subprocess, "CREATE_NEW_PROCESS_GROUP"):
        options["creationflags"] = subprocess.CREATE_NEW_PROCESS_GROUP
    try:
        child = subprocess.Popen(argv, **options)
        try:
            output, stderr = child.communicate(b'{"type":"get_commands"}\n', timeout=timeout_seconds)
            exit_code, timed_out = child.returncode, False
        except subprocess.TimeoutExpired:
            timed_out = True
            kill_process_tree(child)
            output, stderr = child.communicate()
            exit_code = None
    except OSError as exc:
        return {"status": "failed", "registered": False, "errors": [f"launch_failed:{exc.__class__.__name__}"]}

    response = None
    for line in output.decode("utf-8", errors="replace").splitlines():
        try:
            candidate = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(candidate, dict) and candidate.get("type") == "response" and candidate.get("command") == "get_commands":
            response = candidate
            break
    data = (response or {}).get("data", {})
    commands = data.get("commands", []) if isinstance(data, dict) else []
    if not isinstance(commands, list):
        commands = []
    registered = any(
        command.get("name") == EXTENSION_COMMAND and command.get("source") == "extension"
        for command in commands if isinstance(command, dict)
    )
    errors: list[str] = []
    if not response or not response.get("success"):
        errors.append("get_commands_failed")
    if not registered:
        errors.append("focusa_status_command_not_registered")
    if EXTENSION_LOAD_ERROR.search(stderr.decode("utf-8", errors="replace")):
        errors.append("extension_load_failure")
    if timed_out and not response:
        errors.append("extension_probe_timeout")
    return {
        "status": "healthy" if not errors else "failed", "registered": registered,
        "command": EXTENSION_COMMAND, "extension_path": path, "request": "get_commands",
        "provider_calls": 0, "errors": errors, "exit_code_before_cleanup": exit_code,
    }


def probe_status(url: str, samples: int = 25, timeout_seconds: float = 5.0) -> dict[str, Any]:
    """Measure status latency and require stable supervisor counters."""
    from urllib.parse import urlsplit
    parsed = urlsplit(url)
    if parsed.scheme not in ("http", "https") or not parsed.hostname or parsed.path != "/v1/status" or parsed.username or parsed.password or parsed.query or parsed.fragment:
        return {"status": "unavailable", "errors": ["status_url_invalid_or_contains_credentials"]}
    durations: list[float] = []
    missing: set[str] = set()
    counter_snapshot: dict[str, int] = {}
    try:
        for _ in range(samples):
            started = time.perf_counter()
            with urllib.request.urlopen(url, timeout=timeout_seconds) as response:
                if response.status != 200:
                    return {"status": "failed", "errors": [f"http_status:{response.status}"]}
                body = response.read(1024 * 1024)
            durations.append((time.perf_counter() - started) * 1000)
            payload = json.loads(body)
            present = find_keys(payload)
            missing.update(key for key in REQUIRED_STATUS_COUNTERS if key not in present)
            values = find_values(payload)
            for key in REQUIRED_STATUS_COUNTERS:
                value = values.get(key)
                if isinstance(value, bool) or not isinstance(value, int) or value < 0:
                    missing.add(key)
                else:
                    counter_snapshot[key] = value
    except (OSError, urllib.error.URLError, TimeoutError, json.JSONDecodeError, ValueError) as exc:
        return {"status": "unavailable", "errors": [f"status_probe_failed:{exc.__class__.__name__}"]}
    ordered = sorted(durations)
    result: dict[str, Any] = {
        "status": "passed" if not missing else "failed", "samples": len(ordered),
        "p50_ms": round(ordered[len(ordered) // 2], 2),
        "p95_ms": round(ordered[min(len(ordered) - 1, math.ceil(len(ordered) * 0.95) - 1)], 2),
        "max_ms": round(max(ordered), 2), "required_counters": list(REQUIRED_STATUS_COUNTERS),
        "counter_snapshot": counter_snapshot, "missing_counters": sorted(missing),
    }
    if missing:
        result["errors"] = ["runtime_perf_schema_incomplete"]
    return result


def find_keys(value: Any) -> set[str]:
    if isinstance(value, dict):
        return set(value).union(*(find_keys(child) for child in value.values()))
    if isinstance(value, list):
        return set().union(*(find_keys(child) for child in value))
    return set()


def find_values(value: Any) -> dict[str, Any]:
    found: dict[str, Any] = {}
    if isinstance(value, dict):
        for key, child in value.items():
            if key in REQUIRED_STATUS_COUNTERS:
                found[key] = child
            found.update(find_values(child))
    elif isinstance(value, list):
        for child in value:
            found.update(find_values(child))
    return found


def finite_number(value: Any) -> float | None:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        return None
    numeric = float(value)
    return numeric if math.isfinite(numeric) else None


def aggregate_results(
    arms: dict[str, dict[str, Any]], status: dict[str, Any], extension: dict[str, Any],
    thresholds: dict[str, float], metadata: dict[str, Any],
) -> dict[str, Any]:
    failures: list[str] = []
    status = status if isinstance(status, dict) else {}
    extension = extension if isinstance(extension, dict) else {}
    if extension.get("status") != "healthy" or not extension.get("registered"):
        failures.append("extension_registration_or_health")
    if status.get("status") != "passed":
        failures.append("status_probe_or_schema")
    normalized: dict[str, dict[str, Any]] = {}
    for name in ("none", "focusa_event_driven", "focusa_polling"):
        arm = arms.get(name)
        if not isinstance(arm, dict):
            failures.append(f"{name}:measurement_missing")
            normalized[name] = {}
            continue
        normalized[name] = arm
        if not arm.get("ok"):
            raw_errors = arm.get("errors", [])
            error_text = ",".join(item for item in raw_errors if isinstance(item, str)) if isinstance(raw_errors, list) else "invalid_error_list"
            failures.append(f"{name}:invalid_measurement:{error_text}")
        for metric, threshold, suffix in (
            ("elapsed_s", thresholds["max_arm_elapsed_s"], "elapsed"),
            ("maxrss_kb", thresholds["max_arm_rss_kb"], "rss"),
            ("cpu_pct", thresholds["max_arm_cpu_pct"], "cpu"),
        ):
            value = finite_number(arm.get(metric))
            if value is None or value < 0 or value > threshold:
                failures.append(f"{name}:{suffix}_threshold_or_unavailable")
    status_p95 = finite_number(status.get("p95_ms"))
    if status_p95 is None or status_p95 < 0 or status_p95 > thresholds["max_status_p95_ms"]:
        failures.append("status_latency:p95_threshold_or_unavailable")
    none = normalized.get("none", {})
    event = normalized.get("focusa_event_driven", {})
    polling = normalized.get("focusa_polling", {})
    none_elapsed = finite_number(none.get("elapsed_s"))
    event_elapsed = finite_number(event.get("elapsed_s"))
    polling_elapsed = finite_number(polling.get("elapsed_s"))
    deltas: dict[str, float] = {}
    if none_elapsed is not None and event_elapsed is not None:
        delta = round(event_elapsed - none_elapsed, 3)
        deltas["event_driven_vs_none_s"] = delta
        if delta > thresholds["max_event_delta_s"]:
            failures.append("event_driven:regression_threshold")
    else:
        failures.append("event_driven:comparison_unavailable")
    if event_elapsed is not None and polling_elapsed is not None:
        delta = round(polling_elapsed - event_elapsed, 3)
        deltas["polling_vs_event_driven_s"] = delta
        if delta > thresholds["max_polling_delta_s"]:
            failures.append("polling:regression_threshold")
    else:
        failures.append("polling:comparison_unavailable")
    result = {
        "schema": "focusa.spec83_benchmark_result.v1", "benchmark": "pi_focusa_rpc_efficiency",
        "status": "failed" if failures else "passed", "performance_evidence": not failures,
        "thresholds": thresholds, "modes": arms, "status_latency": status,
        "extension_health": extension, "comparisons": deltas, "metadata": metadata,
    }
    if failures:
        result["failures"] = failures
    return result


def write_json(path: str | Path, value: dict[str, Any]) -> None:
    target = Path(path)
    target.parent.mkdir(parents=True, exist_ok=True)
    with target.open("x", encoding="utf-8") as output:
        output.write(json.dumps(value, indent=2, sort_keys=True) + "\n")


def main() -> int:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="action", required=True)
    arm_parser = subparsers.add_parser("arm")
    arm_parser.add_argument("--label", required=True)
    arm_parser.add_argument("--program", required=True)
    arm_parser.add_argument("--timing", required=True)
    arm_parser.add_argument("--timeout-seconds", type=float, required=True)
    arm_parser.add_argument("--extension-path")
    arm_parser.add_argument("--program-arg", action="append", default=[])
    arm_parser.add_argument("--arg", action="append", default=[])
    ext_parser = subparsers.add_parser("probe-extension")
    ext_parser.add_argument("--program", required=True)
    ext_parser.add_argument("--extension-path", required=True)
    ext_parser.add_argument("--output", required=True)
    ext_parser.add_argument("--program-arg", action="append", default=[])
    ext_parser.add_argument("--timeout-seconds", type=float, default=2.0)
    args = parser.parse_args()
    if args.action == "arm":
        result = run_arm(label=args.label, program=args.program, program_args=args.program_arg,
                         arguments=args.arg, timeout_seconds=args.timeout_seconds, extension_path=args.extension_path)
        write_json(args.timing, result)
        print(json.dumps(result, sort_keys=True))
        return 0  # Aggregation owns pass/fail so every arm is observed.
    result = probe_extension(args.program, args.extension_path, args.timeout_seconds, args.program_arg)
    write_json(args.output, result)
    print(json.dumps(result, sort_keys=True))
    return 0 if result["status"] == "healthy" else 1


if __name__ == "__main__":
    raise SystemExit(main())
