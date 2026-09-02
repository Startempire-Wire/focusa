#!/usr/bin/env python3
from __future__ import annotations

import hashlib
import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "appveyor_intake", ROOT / "scripts/intake-appveyor-release-artifacts.py"
)
module = importlib.util.module_from_spec(SPEC)
assert SPEC.loader
SPEC.loader.exec_module(module)

TAG = "v9.9.9"
SHA = "a" * 40


def make_detail() -> dict:
    jobs = []
    index = 0
    for target in module.TARGETS:
        arch = "amd64" if target.startswith("x86_64-") else "amd64_arm64"
        for surface in module.SURFACES:
            index += 1
            jobs.append(
                {
                    "jobId": f"job{index:013d}",
                    "name": (
                        f"Environment: RUST_TARGET={target}, "
                        f"MSVC_ARCH={arch}, SURFACE={surface}"
                    ),
                    "status": "success",
                }
            )
    return {
        "project": {
            "repositoryName": module.DEFAULT_REPOSITORY,
            "isPrivate": False,
        },
        "build": {
            "buildNumber": 321,
            "buildId": 987654,
            "isTag": True,
            "tag": TAG,
            "commitId": SHA,
            "status": "success",
            "jobs": jobs,
        },
    }


class FakeClient:
    def __init__(self, detail: dict | None = None):
        self.detail = detail or make_detail()
        self.payloads: dict[str, dict[str, bytes]] = {}
        for job in self.detail["build"]["jobs"]:
            match = module.JOB_NAME.fullmatch(job["name"])
            assert match
            target, surface = match.groups()
            names = module.expected_names(TAG, target, surface)
            self.payloads[job["jobId"]] = {
                name: f"{job['jobId']}:{name}".encode() for name in names
            }

    def history(self, account: str, project: str) -> dict:
        build = self.detail["build"]
        return {
            "builds": [
                {
                    "buildNumber": build["buildNumber"],
                    "isTag": build["isTag"],
                    "tag": build["tag"],
                    "commitId": build["commitId"],
                }
            ]
        }

    def build(self, account: str, project: str, build_number: int) -> dict:
        assert build_number == self.detail["build"]["buildNumber"]
        return self.detail

    def artifacts(self, job_id: str) -> list[dict]:
        return [
            {
                "fileName": f"artifacts/{name}",
                "size": len(payload),
                "type": "File",
            }
            for name, payload in sorted(self.payloads[job_id].items())
        ]

    def log(self, job_id: str) -> str:
        return (
            f"appveyor_recovery_identity=passed tag={TAG} sha={SHA} "
            "route=branch\n"
        )

    def download(
        self,
        job_id: str,
        provider_name: str,
        destination: Path,
        expected_size: int,
    ) -> str:
        name = Path(provider_name).name
        payload = self.payloads[job_id][name]
        if len(payload) != expected_size:
            raise module.IntakeError("fake provider size mismatch")
        destination.write_bytes(payload)
        return hashlib.sha256(payload).hexdigest()


class AppVeyorReleaseArtifactIntakeTests(unittest.TestCase):
    def validated(self, client: FakeClient):
        return module.validate_build(
            client.detail, module.DEFAULT_REPOSITORY, TAG, SHA
        )

    def test_complete_exact_tag_matrix_produces_typed_receipt(self):
        client = FakeClient()
        build, jobs = self.validated(client)
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "artifacts"
            receipt_path = output / f"appveyor-release-receipt-{TAG}.json"
            receipt = module.collect_artifacts(
                client,
                build,
                jobs,
                TAG,
                SHA,
                module.DEFAULT_ACCOUNT,
                module.DEFAULT_PROJECT,
                module.DEFAULT_REPOSITORY,
                output,
                receipt_path,
            )
            self.assertEqual(
                receipt["schema"], "focusa.appveyor_release_artifact_receipt.v1"
            )
            self.assertEqual(receipt["artifact_count"], 16)
            self.assertEqual(len(list(output.iterdir())), 17)
            self.assertTrue(
                any(item["name"].startswith("focusa-session-runner-") for item in receipt["artifacts"])
            )
            self.assertEqual(json.loads(receipt_path.read_text()), receipt)

    def test_wrong_candidate_sha_is_rejected(self):
        client = FakeClient()
        client.detail["build"]["commitId"] = "b" * 40
        with self.assertRaisesRegex(module.IntakeError, "candidate SHA mismatch"):
            self.validated(client)

    def test_failed_provider_job_is_rejected(self):
        client = FakeClient()
        client.detail["build"]["jobs"][0]["status"] = "failed"
        with self.assertRaisesRegex(module.IntakeError, "is not successful"):
            self.validated(client)

    def test_reviewed_recovery_build_requires_candidate_marker_in_every_job(self):
        client = FakeClient()
        controller_sha = "c" * 40
        build = client.detail["build"]
        build.update(
            {
                "isTag": False,
                "tag": None,
                "commitId": controller_sha,
                "branch": "fix/recovery-controller",
            }
        )
        _, jobs = module.validate_build(
            client.detail,
            module.DEFAULT_REPOSITORY,
            TAG,
            SHA,
            controller_sha,
            "fix/recovery-controller",
        )
        module.validate_recovery_logs(client, jobs, TAG, SHA)
        client.log = lambda _job_id: (
            f"appveyor_recovery_identity=passed tag={TAG} sha={SHA} "
            "route=same_repo_pr\n"
        )
        with self.assertRaisesRegex(module.IntakeError, "candidate marker missing"):
            module.validate_recovery_logs(client, jobs, TAG, SHA)

    def test_test_jobs_must_retain_no_artifacts(self):
        client = FakeClient()
        build, jobs = self.validated(client)
        test_job = jobs[(module.TARGETS[0], "tests")]["jobId"]
        client.payloads[test_job]["unexpected-test-output.txt"] = b"unexpected"
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "artifacts"
            with self.assertRaisesRegex(module.IntakeError, "artifacts mismatch"):
                module.collect_artifacts(
                    client,
                    build,
                    jobs,
                    TAG,
                    SHA,
                    module.DEFAULT_ACCOUNT,
                    module.DEFAULT_PROJECT,
                    module.DEFAULT_REPOSITORY,
                    output,
                    output / "receipt.json",
                )

    def test_missing_updater_signature_is_rejected(self):
        client = FakeClient()
        build, jobs = self.validated(client)
        menubar_job = jobs[(module.TARGETS[0], "menubar")]["jobId"]
        signature = next(name for name in client.payloads[menubar_job] if name.endswith(".sig"))
        del client.payloads[menubar_job][signature]
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "artifacts"
            with self.assertRaisesRegex(module.IntakeError, "artifacts mismatch"):
                module.collect_artifacts(
                    client,
                    build,
                    jobs,
                    TAG,
                    SHA,
                    module.DEFAULT_ACCOUNT,
                    module.DEFAULT_PROJECT,
                    module.DEFAULT_REPOSITORY,
                    output,
                    output / "receipt.json",
                )

    def test_unsafe_provider_path_is_rejected(self):
        client = FakeClient()
        listing = client.artifacts(next(iter(client.payloads)))
        listing[0]["fileName"] = "artifacts/../escape.exe"
        expected = {Path(item["fileName"]).name for item in listing}
        with self.assertRaisesRegex(module.IntakeError, "unsafe AppVeyor artifact path"):
            module.validate_artifact_listing(listing, expected, "job")

    def test_canonical_prerelease_tag_grammar_is_preserved(self):
        self.assertEqual(module._require_tag("v9.9.9-dev"), "v9.9.9-dev")
        with self.assertRaisesRegex(module.IntakeError, "invalid release tag"):
            module._require_tag("release/9.9.9")

    def test_multiple_exact_tag_builds_are_rejected(self):
        client = FakeClient()
        history = client.history(module.DEFAULT_ACCOUNT, module.DEFAULT_PROJECT)
        history["builds"].append(dict(history["builds"][0], buildNumber=322))
        client.history = lambda _account, _project: history
        with self.assertRaisesRegex(module.IntakeError, "multiple exact"):
            module.discover_build_number(
                client,
                module.DEFAULT_ACCOUNT,
                module.DEFAULT_PROJECT,
                TAG,
                SHA,
            )


if __name__ == "__main__":
    unittest.main()
