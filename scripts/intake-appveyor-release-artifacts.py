#!/usr/bin/env python3
"""Verify and download exact Windows release artifacts from AppVeyor.

AppVeyor is a build/signing producer only. This adapter reads its public API,
verifies the immutable release identity and complete Windows matrix, downloads
provider-retained artifacts, and emits a typed SHA-256 receipt. GitHub upload
authority remains in the calling canonical Release workflow.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path, PurePosixPath
from typing import Any

DEFAULT_API_BASE = "https://ci.appveyor.com/api"
DEFAULT_ACCOUNT = "verioussmith"
DEFAULT_PROJECT = "focusa"
DEFAULT_REPOSITORY = "Startempire-Wire/focusa"
TARGETS = ("x86_64-pc-windows-msvc", "aarch64-pc-windows-msvc")
SURFACES = ("binaries", "tests", "menubar")
RUST_BINARIES = ("focusa", "focusa-daemon", "focusa-session-runner", "focusa-tui")
JOB_NAME = re.compile(
    r"^Environment: RUST_TARGET=([^,]+), MSVC_ARCH=[^,]+, SURFACE=([^,]+)$"
)
TERMINAL_FAILURES = {"failed", "canceled", "cancelled"}


class IntakeError(RuntimeError):
    """Fail-closed provider identity, contract, or download error."""


class AppVeyorClient:
    def __init__(self, api_base: str = DEFAULT_API_BASE, timeout_seconds: int = 30):
        self.api_base = api_base.rstrip("/")
        self.timeout_seconds = timeout_seconds

    def _url(self, path: str) -> str:
        return f"{self.api_base}/{path.lstrip('/')}"

    def _request(self, path: str) -> urllib.request.Request:
        return urllib.request.Request(
            self._url(path),
            headers={"Accept": "application/json", "User-Agent": "focusa-release-intake/1"},
        )

    def _json(self, path: str) -> Any:
        with urllib.request.urlopen(
            self._request(path), timeout=self.timeout_seconds
        ) as response:
            return json.load(response)

    def history(self, account: str, project: str) -> dict[str, Any]:
        account_q = urllib.parse.quote(account, safe="")
        project_q = urllib.parse.quote(project, safe="")
        return self._json(
            f"projects/{account_q}/{project_q}/history?recordsNumber=100"
        )

    def build(self, account: str, project: str, build_number: int) -> dict[str, Any]:
        account_q = urllib.parse.quote(account, safe="")
        project_q = urllib.parse.quote(project, safe="")
        return self._json(f"projects/{account_q}/{project_q}/build/{build_number}")

    def artifacts(self, job_id: str) -> list[dict[str, Any]]:
        job_q = urllib.parse.quote(job_id, safe="")
        result = self._json(f"buildjobs/{job_q}/artifacts")
        if not isinstance(result, list):
            raise IntakeError(f"AppVeyor artifact response is not a list for job {job_id}")
        return result

    def log(self, job_id: str) -> str:
        job_q = urllib.parse.quote(job_id, safe="")
        request = urllib.request.Request(
            self._url(f"buildjobs/{job_q}/log"),
            headers={"Accept": "text/plain", "User-Agent": "focusa-release-intake/1"},
        )
        with urllib.request.urlopen(request, timeout=self.timeout_seconds) as response:
            return response.read().decode("utf-8", errors="replace")

    def download(
        self,
        job_id: str,
        provider_name: str,
        destination: Path,
        expected_size: int,
    ) -> str:
        job_q = urllib.parse.quote(job_id, safe="")
        name_q = urllib.parse.quote(provider_name, safe="/")
        request = self._request(f"buildjobs/{job_q}/artifacts/{name_q}")
        temporary = destination.with_name(destination.name + ".part")
        digest = hashlib.sha256()
        received = 0
        try:
            with urllib.request.urlopen(
                request, timeout=self.timeout_seconds
            ) as response, temporary.open("xb") as output:
                declared = response.headers.get("Content-Length")
                if declared is not None and int(declared) != expected_size:
                    raise IntakeError(
                        f"provider Content-Length mismatch for {destination.name}: "
                        f"declared={declared} expected={expected_size}"
                    )
                while chunk := response.read(1024 * 1024):
                    received += len(chunk)
                    if received > expected_size:
                        raise IntakeError(
                            f"provider sent excess bytes for {destination.name}"
                        )
                    output.write(chunk)
                    digest.update(chunk)
            if received != expected_size:
                raise IntakeError(
                    f"provider size mismatch for {destination.name}: "
                    f"received={received} expected={expected_size}"
                )
            os.replace(temporary, destination)
            return digest.hexdigest()
        except Exception:
            temporary.unlink(missing_ok=True)
            raise


def _require_string(value: Any, label: str) -> str:
    if not isinstance(value, str) or not value:
        raise IntakeError(f"missing or invalid {label}")
    return value


def _require_sha(value: str) -> str:
    if not re.fullmatch(r"[0-9a-f]{40}", value):
        raise IntakeError(f"invalid release SHA: {value!r}")
    return value


def _require_tag(value: str) -> str:
    if not re.fullmatch(
        r"v[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z][0-9A-Za-z.-]*)?", value
    ):
        raise IntakeError(f"invalid release tag: {value!r}")
    return value


def expected_binary_names(tag: str, target: str) -> set[str]:
    return {f"{binary}-{tag}-{target}.exe" for binary in RUST_BINARIES}


def expected_menubar_names(tag: str, target: str) -> set[str]:
    version = tag.removeprefix("v")
    architecture = "x64" if target.startswith("x86_64-") else "arm64"
    setup = f"Focusa_{version}_{architecture}-setup.exe"
    msi = f"Focusa_{version}_{architecture}_en-US.msi"
    return {setup, setup + ".sig", msi, msi + ".sig"}


def expected_names(tag: str, target: str, surface: str) -> set[str]:
    if surface == "binaries":
        return expected_binary_names(tag, target)
    if surface == "menubar":
        return expected_menubar_names(tag, target)
    return set()


def discover_build_number(
    client: AppVeyorClient,
    account: str,
    project: str,
    tag: str,
    sha: str,
    explicit_build_number: int | None = None,
) -> int | None:
    if explicit_build_number is not None:
        return explicit_build_number
    history = client.history(account, project)
    builds = history.get("builds")
    if not isinstance(builds, list):
        raise IntakeError("AppVeyor history response has no builds list")
    matches = [
        build
        for build in builds
        if build.get("isTag") is True
        and build.get("tag") == tag
        and build.get("commitId") == sha
    ]
    if len(matches) > 1:
        numbers = [build.get("buildNumber") for build in matches]
        raise IntakeError(f"multiple exact AppVeyor tag builds found: {numbers}")
    if not matches:
        return None
    number = matches[0].get("buildNumber")
    if not isinstance(number, int) or number <= 0:
        raise IntakeError("exact AppVeyor build has invalid buildNumber")
    return number


def validate_build(
    detail: dict[str, Any],
    repository: str,
    tag: str,
    sha: str,
    recovery_controller_sha: str | None = None,
    recovery_controller_branch: str | None = None,
) -> tuple[dict[str, Any], dict[tuple[str, str], dict[str, Any]]]:
    project = detail.get("project")
    build = detail.get("build")
    if not isinstance(project, dict) or not isinstance(build, dict):
        raise IntakeError("AppVeyor build detail is missing project/build objects")
    if project.get("repositoryName") != repository:
        raise IntakeError(
            f"AppVeyor repository mismatch: {project.get('repositoryName')!r}"
        )
    if project.get("isPrivate") is not False:
        raise IntakeError("AppVeyor release intake requires a public project")
    if recovery_controller_sha is None:
        if build.get("isTag") is not True or build.get("tag") != tag:
            raise IntakeError("AppVeyor build is not the exact release tag")
        if build.get("commitId") != sha:
            raise IntakeError(
                f"AppVeyor candidate SHA mismatch: {build.get('commitId')!r} != {sha}"
            )
    else:
        if build.get("isTag") is True:
            raise IntakeError("AppVeyor recovery route cannot consume a tag build")
        if build.get("commitId") != recovery_controller_sha:
            raise IntakeError("AppVeyor recovery controller SHA mismatch")
        if build.get("branch") != recovery_controller_branch:
            raise IntakeError("AppVeyor recovery controller branch mismatch")
    if build.get("status") != "success":
        raise IntakeError(f"AppVeyor build is not successful: {build.get('status')!r}")

    jobs = build.get("jobs")
    if not isinstance(jobs, list):
        raise IntakeError("AppVeyor build has no jobs list")
    indexed: dict[tuple[str, str], dict[str, Any]] = {}
    for job in jobs:
        if not isinstance(job, dict):
            raise IntakeError("AppVeyor job is not an object")
        name = _require_string(job.get("name"), "AppVeyor job name")
        match = JOB_NAME.fullmatch(name)
        if match is None:
            raise IntakeError(f"unexpected AppVeyor release job: {name}")
        key = (match.group(1), match.group(2))
        if key in indexed:
            raise IntakeError(f"duplicate AppVeyor release job: {key}")
        indexed[key] = job

    required = {(target, surface) for target in TARGETS for surface in SURFACES}
    if set(indexed) != required:
        missing = sorted(required - set(indexed))
        extra = sorted(set(indexed) - required)
        raise IntakeError(f"AppVeyor matrix mismatch: missing={missing} extra={extra}")
    for key, job in indexed.items():
        if job.get("status") != "success":
            raise IntakeError(f"AppVeyor job {key} is not successful: {job.get('status')}")
        _require_string(job.get("jobId"), f"AppVeyor job id for {key}")
    return build, indexed


def validate_recovery_logs(
    client: AppVeyorClient,
    jobs: dict[tuple[str, str], dict[str, Any]],
    tag: str,
    sha: str,
) -> None:
    marker = f"appveyor_recovery_identity=passed tag={tag} sha={sha} route=branch"
    for key, job in jobs.items():
        job_id = _require_string(job.get("jobId"), f"AppVeyor job id for {key}")
        if marker not in client.log(job_id):
            raise IntakeError(f"AppVeyor recovery candidate marker missing from job {key}")


def validate_artifact_listing(
    listing: list[dict[str, Any]], expected: set[str], job_id: str
) -> dict[str, tuple[str, int]]:
    found: dict[str, tuple[str, int]] = {}
    for artifact in listing:
        if not isinstance(artifact, dict) or artifact.get("type") != "File":
            raise IntakeError(f"non-file artifact in AppVeyor job {job_id}")
        provider_name = _require_string(artifact.get("fileName"), "artifact fileName")
        provider_path = PurePosixPath(provider_name)
        if (
            provider_path.is_absolute()
            or ".." in provider_path.parts
            or len(provider_path.parts) != 2
            or provider_path.parts[0] != "artifacts"
        ):
            raise IntakeError(f"unsafe AppVeyor artifact path: {provider_name!r}")
        name = provider_path.name
        size = artifact.get("size")
        if not isinstance(size, int) or size <= 0:
            raise IntakeError(f"invalid AppVeyor artifact size for {name}: {size!r}")
        if name in found:
            raise IntakeError(f"duplicate AppVeyor artifact basename: {name}")
        found[name] = (provider_name, size)
    if set(found) != expected:
        missing = sorted(expected - set(found))
        extra = sorted(set(found) - expected)
        raise IntakeError(
            f"AppVeyor artifacts mismatch for job {job_id}: missing={missing} extra={extra}"
        )
    return found


def collect_artifacts(
    client: AppVeyorClient,
    build: dict[str, Any],
    jobs: dict[tuple[str, str], dict[str, Any]],
    tag: str,
    sha: str,
    account: str,
    project: str,
    repository: str,
    output_dir: Path,
    receipt_path: Path,
    route: str = "exact_tag",
) -> dict[str, Any]:
    output_dir.mkdir(parents=True, exist_ok=True)
    if any(output_dir.iterdir()):
        raise IntakeError(f"output directory must be empty: {output_dir}")
    artifacts: list[dict[str, Any]] = []
    for target in TARGETS:
        test_job = jobs[(target, "tests")]
        test_job_id = _require_string(test_job.get("jobId"), "AppVeyor test job id")
        validate_artifact_listing(client.artifacts(test_job_id), set(), test_job_id)
        for surface in ("binaries", "menubar"):
            job = jobs[(target, surface)]
            job_id = _require_string(job.get("jobId"), "AppVeyor artifact job id")
            expected = expected_names(tag, target, surface)
            listing = validate_artifact_listing(client.artifacts(job_id), expected, job_id)
            for name in sorted(listing):
                provider_name, size = listing[name]
                destination = output_dir / name
                digest = client.download(job_id, provider_name, destination, size)
                if not re.fullmatch(r"[0-9a-f]{64}", digest):
                    raise IntakeError(f"invalid SHA-256 returned for {name}")
                artifacts.append(
                    {
                        "name": name,
                        "size": size,
                        "sha256": digest,
                        "job_id": job_id,
                        "target": target,
                        "surface": surface,
                    }
                )

    if len(artifacts) != 16:
        raise IntakeError(f"expected 16 AppVeyor artifacts, received {len(artifacts)}")
    receipt = {
        "schema": "focusa.appveyor_release_artifact_receipt.v1",
        "provider": "appveyor",
        "account": account,
        "project": project,
        "repository": repository,
        "build_number": build.get("buildNumber"),
        "build_id": build.get("buildId"),
        "tag": tag,
        "candidate_sha": sha,
        "provider_commit_sha": build.get("commitId"),
        "route": route,
        "provider_finished_at": build.get("finished"),
        "artifact_count": len(artifacts),
        "artifacts": sorted(artifacts, key=lambda item: item["name"]),
    }
    receipt_path.parent.mkdir(parents=True, exist_ok=True)
    with receipt_path.open("x", encoding="utf-8") as output:
        json.dump(receipt, output, indent=2, sort_keys=True)
        output.write("\n")
    return receipt


def wait_for_successful_build(
    client: AppVeyorClient,
    account: str,
    project: str,
    repository: str,
    tag: str,
    sha: str,
    explicit_build_number: int | None,
    timeout_minutes: int,
    poll_seconds: int,
    recovery_controller_sha: str | None = None,
    recovery_controller_branch: str | None = None,
) -> tuple[dict[str, Any], dict[tuple[str, str], dict[str, Any]]]:
    deadline = time.monotonic() + timeout_minutes * 60
    last_state = "not_found"
    while True:
        try:
            build_number = discover_build_number(
                client, account, project, tag, sha, explicit_build_number
            )
            if build_number is not None:
                detail = client.build(account, project, build_number)
                build = detail.get("build")
                state = build.get("status") if isinstance(build, dict) else None
                last_state = f"build={build_number} status={state}"
                if state == "success":
                    validated = validate_build(
                        detail,
                        repository,
                        tag,
                        sha,
                        recovery_controller_sha,
                        recovery_controller_branch,
                    )
                    if recovery_controller_sha is not None:
                        validate_recovery_logs(client, validated[1], tag, sha)
                    return validated
                if state in TERMINAL_FAILURES:
                    raise IntakeError(
                        f"AppVeyor selected build terminated without success: {last_state}"
                    )
        except (urllib.error.URLError, TimeoutError) as error:
            last_state = f"provider_error={error}"
            print(f"appveyor_intake_retry {last_state}", file=sys.stderr)
        if time.monotonic() >= deadline:
            raise IntakeError(f"AppVeyor intake timed out: {last_state}")
        print(f"appveyor_intake_wait {last_state}", file=sys.stderr)
        time.sleep(poll_seconds)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--tag", required=True)
    parser.add_argument("--sha", required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--receipt", type=Path, required=True)
    parser.add_argument("--account", default=DEFAULT_ACCOUNT)
    parser.add_argument("--project", default=DEFAULT_PROJECT)
    parser.add_argument("--repository", default=DEFAULT_REPOSITORY)
    parser.add_argument("--build-number", type=int)
    parser.add_argument("--recovery-controller-sha")
    parser.add_argument("--recovery-controller-branch")
    parser.add_argument("--timeout-minutes", type=int, default=385)
    parser.add_argument("--poll-seconds", type=int, default=30)
    args = parser.parse_args()

    try:
        tag = _require_tag(args.tag)
        sha = _require_sha(args.sha)
        if args.timeout_minutes <= 0 or args.poll_seconds <= 0:
            raise IntakeError("timeouts must be positive")
        recovery_values = (args.recovery_controller_sha, args.recovery_controller_branch)
        if any(recovery_values) and not all(recovery_values):
            raise IntakeError("recovery controller SHA and branch must be supplied together")
        if args.recovery_controller_sha is not None:
            _require_sha(args.recovery_controller_sha)
            if args.build_number is None:
                raise IntakeError("recovery intake requires an explicit build number")
        client = AppVeyorClient()
        build, jobs = wait_for_successful_build(
            client,
            args.account,
            args.project,
            args.repository,
            tag,
            sha,
            args.build_number,
            args.timeout_minutes,
            args.poll_seconds,
            args.recovery_controller_sha,
            args.recovery_controller_branch,
        )
        receipt = collect_artifacts(
            client,
            build,
            jobs,
            tag,
            sha,
            args.account,
            args.project,
            args.repository,
            args.output_dir,
            args.receipt,
            "recovery" if args.recovery_controller_sha else "exact_tag",
        )
    except (IntakeError, OSError, urllib.error.URLError, json.JSONDecodeError) as error:
        print(f"appveyor_intake=FAIL error={error}", file=sys.stderr)
        return 1

    print(
        "appveyor_intake=PASS "
        f"tag={tag} sha={sha} build={receipt['build_number']} "
        f"artifacts={receipt['artifact_count']} receipt={args.receipt}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
