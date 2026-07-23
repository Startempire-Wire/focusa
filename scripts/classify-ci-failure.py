#!/usr/bin/env python3
"""DRY release-path failure classifier for Focusa CI/self-heal.

Reads a GitHub Actions failed log and emits either JSON (default) or shell-safe
KEY=value lines (--format env).  Auto Heal, Watchdog, Audit, and agents should
consume this one taxonomy instead of duplicating regex decisions.
"""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Iterable

ANSI_RE = re.compile(r"(?:\x1b|\^\[)\[[0-9;]*[A-Za-z]")
TS_PREFIX_RE = re.compile(
    r"^(?P<job>[^\t]+)\t[^\t]+\t\d{4}-\d{2}-\d{2}T[^ ]+Z\s*(?P<body>.*)$"
)
RUST_REF_RE = re.compile(r"-->\s+([^\s:]+\.rs):(\d+):(\d+)")
ERROR_CODE_RE = re.compile(r"error\[(E\d+)\]")


HARD = "hard_failure_no_rerun"
RERUN = "rerun_once"


def clean_log(text: str) -> str:
    return ANSI_RE.sub("", text.replace("\ufeff", ""))


def compact_line(line: str) -> str:
    m = TS_PREFIX_RE.match(line)
    if m:
        return m.group("body")
    return line


def source_refs(text: str) -> list[str]:
    refs: list[str] = []
    seen: set[str] = set()
    error_context = 0
    for raw_line in text.splitlines():
        line = compact_line(raw_line)
        if re.search(r"error(\[E\d+\])?:", line):
            error_context = 30
        elif error_context > 0:
            error_context -= 1
        else:
            continue
        for match in RUST_REF_RE.finditer(line):
            ref = f"{match.group(1)}:{match.group(2)}:{match.group(3)}"
            if ref not in seen:
                refs.append(ref)
                seen.add(ref)
    if not refs:
        for match in RUST_REF_RE.finditer(text):
            ref = f"{match.group(1)}:{match.group(2)}:{match.group(3)}"
            if ref not in seen:
                refs.append(ref)
                seen.add(ref)
    return refs[:20]


def rust_error_codes(text: str) -> list[str]:
    codes: list[str] = []
    seen: set[str] = set()
    for code in ERROR_CODE_RE.findall(text):
        if code not in seen:
            codes.append(code)
            seen.add(code)
    return codes


def classify(text: str) -> dict:
    clean = clean_log(text)
    lowered = clean.lower()
    refs = source_refs(clean)
    codes = rust_error_codes(clean)
    signals: list[str] = []

    if "positional arguments in format string" in lowered:
        signals.append("rust_format_arg_mismatch")
    if (
        "this function takes" in lowered
        and "arguments but" in lowered
        and "were supplied" in lowered
    ):
        signals.append("rust_api_signature_mismatch")
    if codes:
        signals.extend(f"rust_error_{code.lower()}" for code in codes)

    if signals:
        if (
            "rust_format_arg_mismatch" in signals
            and "rust_api_signature_mismatch" in signals
        ):
            failure_class = "rust_compile_api_drift"
            remediation = "Sync changed API surfaces: fix format! placeholders/arguments and update stale callsites or tests."
        elif "rust_format_arg_mismatch" in signals:
            failure_class = "rust_compile_format_arg_drift"
            remediation = "Sync format! placeholders with argument list at the reported source refs."
        elif "rust_api_signature_mismatch" in signals:
            failure_class = "rust_compile_api_signature_drift"
            remediation = (
                "Update stale callsites/tests to match the changed function signature."
            )
        else:
            failure_class = "rust_compile_failure"
            remediation = (
                "Patch deterministic Rust compiler errors at reported source refs."
            )
        return result(
            failure_class=failure_class,
            retry_policy=HARD,
            deterministic=True,
            plain_language_error="Blocked: deterministic Rust compile/API drift. Patch code, then let GitHub CI run again.",
            likely_root_cause="; ".join(signals),
            remediation_template=remediation,
            source_refs=refs,
            signals=signals,
        )

    if re.search(
        r"cargo clippy|clippy::|needless_borrow|derivable_impls|private_interfaces",
        clean,
    ):
        return result(
            "ci_clippy_failure",
            HARD,
            True,
            "Blocked: clippy found a deterministic code issue. Patch code, then let GitHub CI run again.",
            "clippy lint failure",
            "Patch lint violations or narrow code changes; do not rerun unchanged CI.",
            refs,
            ["clippy"],
        )

    if re.search(
        r"test result: FAILED|panicked at|assertion `left == right` failed|thread '.*' panicked",
        clean,
    ):
        return result(
            "ci_test_failure",
            HARD,
            True,
            "Blocked: tests failed deterministically. Patch code, then let GitHub CI run again.",
            "test assertion or panic",
            "Patch failing test subject or test expectation; do not rerun unchanged CI.",
            refs,
            ["test_failure"],
        )

    if re.search(
        r"release deploy automation static test|static proof|workflow name missing|static guard",
        lowered,
    ):
        return result(
            "release_static_proof_failure",
            HARD,
            True,
            "Blocked: release static proof failed. Patch the workflow/spec guard, then let GitHub CI run again.",
            "release static guard failure",
            "Patch workflow/static guard drift before retry.",
            refs,
            ["static_proof"],
        )

    if re.search(r"deploy_health timeout|/v1/health|health check failed", lowered):
        return result(
            "deploy_health_failure",
            RERUN,
            False,
            "Deploy health failed; one bounded redeploy is allowed, then inspect service health.",
            "deploy health endpoint failed",
            "Retry once; if repeated, inspect service logs, version, port, and rollback evidence.",
            refs,
            ["deploy_health"],
        )

    if re.search(
        r"Killed|oom|out of memory|No space left on device|runner.*lost|The operation was canceled",
        clean,
        re.I,
    ):
        return result(
            "runner_resource_failure",
            RERUN,
            False,
            "Runner/resource failure detected; one bounded rerun is allowed.",
            "runner resource exhaustion or cancellation",
            "Rerun once; if repeated, clean disk/cache or move job capacity.",
            refs,
            ["runner_resource"],
        )

    if re.search(
        r"failed to determine base repo|not a git repository|gh run rerun|gh workflow run",
        lowered,
    ):
        return result(
            "auto_heal_process_error",
            HARD,
            True,
            "Blocked: self-heal process error. Patch the self-heal workflow before retrying.",
            "self-heal workflow process error",
            "Patch Auto Heal/Watchdog process before retrying the release path.",
            refs,
            ["self_heal_process"],
        )

    if re.search(
        r"HTTP 5\d\d|connection reset|timed out|TLS|rate limit|upload.*failed|artifact.*failed",
        clean,
        re.I,
    ):
        return result(
            "transient_github_or_network_failure",
            RERUN,
            False,
            "Transient GitHub/network/upload failure detected; one bounded rerun is allowed.",
            "transient GitHub/network/upload failure",
            "Rerun once; if repeated, inspect provider status, auth, rate limit, and artifact logs.",
            refs,
            ["transient_network"],
        )

    return result(
        "unknown_process_failure",
        RERUN,
        False,
        "Transient or unknown release-path failure; one bounded rerun is allowed.",
        "unknown release-path failure",
        "Rerun once; if repeated, classify with source logs and add a taxonomy case.",
        refs,
        ["unknown"],
    )


def result(
    failure_class: str,
    retry_policy: str,
    deterministic: bool,
    plain_language_error: str,
    likely_root_cause: str,
    remediation_template: str,
    source_refs: Iterable[str],
    signals: Iterable[str],
) -> dict:
    return {
        "schema": "focusa.release_failure_classification.v1",
        "failure_class": failure_class,
        "retry_policy": retry_policy,
        "deterministic": deterministic,
        "safe_to_rerun_unchanged": retry_policy != HARD,
        "plain_language_error": plain_language_error,
        "likely_root_cause": likely_root_cause,
        "remediation_template": remediation_template,
        "source_refs": list(source_refs),
        "signals": list(signals),
    }


def emit_env(payload: dict) -> None:
    for key in (
        "failure_class",
        "retry_policy",
        "deterministic",
        "safe_to_rerun_unchanged",
        "plain_language_error",
        "likely_root_cause",
        "remediation_template",
    ):
        value = payload[key]
        if isinstance(value, bool):
            value = "true" if value else "false"
        print(f"{key}={str(value).replace(chr(10), ' ')}")
    print(f"source_refs={','.join(payload['source_refs'])}")
    print(f"signals={','.join(payload['signals'])}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("log_file", nargs="?", default="-")
    parser.add_argument("--format", choices=("json", "env"), default="json")
    args = parser.parse_args()

    if args.log_file == "-":
        import sys

        text = sys.stdin.read()
    else:
        text = Path(args.log_file).read_text(errors="ignore")
    payload = classify(text)
    if args.format == "env":
        emit_env(payload)
    else:
        print(json.dumps(payload, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
