#!/usr/bin/env python3
"""Real Git consumers, retained fixtures; no compiler, provider, or release calls."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import tempfile
import unittest

SOURCE = Path(__file__).resolve().parents[1]
HELPER = SOURCE / "scripts/release-git-admin-preflight.py"
EVIDENCE = Path(os.environ.get("FOCUSA_GIT_ADMIN_EVIDENCE_DIR", tempfile.gettempdir()))


class GitAdministrationConsumers(unittest.TestCase):
    def setUp(self):
        self.assertNotEqual(os.geteuid(), 0, "Exercise the real unprivileged release user")
        EVIDENCE.mkdir(parents=True, exist_ok=True)
        self.base = Path(tempfile.mkdtemp(prefix="450-git-admin-", dir=EVIDENCE))
        self.repo = self.base / "source"
        self.repo.mkdir()
        self.env = {
            **os.environ, "GIT_OPTIONAL_LOCKS": "0",
            "GIT_AUTHOR_NAME": "Focusa Git fixture", "GIT_AUTHOR_EMAIL": "fixture@example.invalid",
            "GIT_COMMITTER_NAME": "Focusa Git fixture", "GIT_COMMITTER_EMAIL": "fixture@example.invalid",
            "PYTHONDONTWRITEBYTECODE": "1",
        }
        self.git(self.repo, "init", "--initial-branch=main")
        (self.repo / "scripts").mkdir()
        for name in ("release-git-admin-preflight.py", "create-dev-release-tag.sh", "local-release-preflight.sh"):
            shutil.copy2(SOURCE / "scripts" / name, self.repo / "scripts" / name)
        (self.repo / "sentinel.txt").write_text("initial\n")
        self.git(self.repo, "add", ".")
        self.git(self.repo, "commit", "-m", "test: initialize isolated Git fixture")
        self.linked = self.base / "linked"
        self.git(self.repo, "worktree", "add", "--detach", str(self.linked), "HEAD")

    def run_command(self, argv, cwd, success=True):
        result = subprocess.run(argv, cwd=cwd, env=self.env, text=True, capture_output=True, timeout=20)
        if success:
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        return result

    def git(self, repo, *args, success=True):
        return self.run_command(["git", "-C", str(repo), *args], repo, success)

    def path(self, name):
        value = self.git(self.linked, "rev-parse", "--git-path", name).stdout.strip()
        path = Path(value)
        return path if path.is_absolute() else self.linked / path

    def snapshot(self):
        message = self.path("COMMIT_EDITMSG")
        return {
            "head": self.git(self.linked, "rev-parse", "HEAD").stdout,
            "status": self.git(self.linked, "status", "--porcelain").stdout,
            "refs": self.git(self.linked, "for-each-ref", "--format=%(refname):%(objectname)").stdout,
            "index": hashlib.sha256(self.path("index").read_bytes()).hexdigest(),
            "message": message.read_bytes() if message.is_file() else None,
            "message_mode": stat.S_IMODE(message.lstat().st_mode) if os.path.lexists(message) else None,
        }

    def guard(self, expected):
        result = self.run_command([sys.executable, str(HELPER), "--repo", str(self.linked)], self.linked, False)
        self.assertEqual(result.returncode, expected, result.stdout + result.stderr)
        body = json.loads(result.stdout)
        self.assertIs(body["release_ready"], False)
        self.assertEqual(body["mutations"], [])
        return body

    def stage(self):
        (self.linked / "sentinel.txt").write_text("staged change\n")
        self.git(self.linked, "add", "sentinel.txt")

    def test_actual_permission_failure_and_both_entrypoints_refuse_before_gates(self):
        self.stage()
        message = self.path("COMMIT_EDITMSG")
        message.write_text("retained message\n")
        message.chmod(0o444)  # Deliberate fixture failure, never repaired or removed.
        failed = self.git(self.linked, "commit", "-m", "test: must not commit", success=False)
        self.assertNotEqual(failed.returncode, 0)
        self.assertIn("Permission denied", failed.stderr)
        (self.base / "actual-git-failure.json").write_text(json.dumps({
            "exit_code": failed.returncode, "stderr": failed.stderr,
        }, indent=2))
        before = self.snapshot()
        body = self.guard(1)
        self.assertTrue(any(x["path"] == str(message) and x["reason"] == "git_admin_file_not_writable" for x in body["failures"]))
        for name, args in (("create-dev-release-tag.sh", ["--tag", "v0.0.0-fixture"]), ("local-release-preflight.sh", ["--strict"])):
            result = self.run_command(["bash", str(self.linked / "scripts" / name), *args], self.linked, False)
            self.assertNotEqual(result.returncode, 0)
            result_body = json.loads(result.stdout)
            self.assertEqual(result_body["failure_class"], "release_git_admin_not_writable")
            self.assertNotIn("learning guards", result.stdout)
            self.assertNotIn("Windows path lint", result.stdout)
            self.assertEqual(self.snapshot(), before)
        self.assertEqual(self.snapshot(), before)

    def test_successful_guard_real_commit_and_local_transport_preserve_ownership(self):
        self.stage()
        before = self.snapshot()
        self.assertTrue(self.guard(0)["ok"])
        self.assertEqual(self.snapshot(), before)
        self.git(self.linked, "commit", "-m", "test: commit after real administrative preflight")
        remote = self.base / "local-transport.git"
        self.run_command(["git", "init", "--bare", str(remote)], self.base)
        self.git(self.linked, "push", str(remote), "HEAD:refs/heads/fixture")
        local_head = self.git(self.linked, "rev-parse", "HEAD").stdout.strip()
        remote_head = self.run_command(["git", "--git-dir", str(remote), "rev-parse", "refs/heads/fixture"], self.base).stdout.strip()
        self.assertEqual(local_head, remote_head)
        admin = self.path("COMMIT_EDITMSG").parent
        self.assertEqual({x.stat().st_uid for x in admin.iterdir() if not x.is_symlink()}, {os.geteuid()})
        self.assertEqual(self.git(self.linked, "tag", "--list").stdout, "")
        self.assertTrue(self.guard(0)["ok"])

    def test_existing_lock_is_reported_and_never_removed(self):
        lock = self.path("index.lock")
        lock.write_text("retained concurrent-transaction fixture\n")
        original = lock.read_bytes()
        body = self.guard(1)
        self.assertTrue(any(x["reason"] == "git_transaction_active" and x["path"] == str(lock) for x in body["failures"]))
        self.assertEqual(lock.read_bytes(), original)

    def test_symlink_is_refused_without_touching_its_target(self):
        target = self.base / "outside-message.txt"
        target.write_text("unchanged external sentinel\n")
        self.path("COMMIT_EDITMSG").symlink_to(target)
        body = self.guard(1)
        self.assertTrue(any(x["reason"] == "git_admin_file_not_regular" for x in body["failures"]))
        self.assertEqual(target.read_text(), "unchanged external sentinel\n")

    def test_recheck_catches_permission_drift_after_early_success(self):
        self.stage()
        self.assertTrue(self.guard(0)["ok"])
        message = self.path("COMMIT_EDITMSG")
        message.write_text("late drift fixture\n")
        message.chmod(0o444)
        before = self.snapshot()
        self.assertFalse(self.guard(1)["ok"])
        self.assertEqual(self.snapshot(), before)

    def test_missing_git_context_returns_typed_failure(self):
        empty = self.base / "not-a-repository"
        empty.mkdir()
        result = self.run_command([sys.executable, str(HELPER), "--repo", str(empty)], empty, False)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(json.loads(result.stdout)["failure_class"], "release_git_admin_not_writable")

    def test_each_canonical_commit_has_the_same_last_moment_guard(self):
        script = (SOURCE / "scripts/create-dev-release-tag.sh").read_text().splitlines()
        commits = [n for n, line in enumerate(script) if line.lstrip().startswith("git commit ")]
        self.assertEqual(len(commits), 2)
        for n in commits:
            self.assertEqual(script[n - 1].strip(), "python3 scripts/release-git-admin-preflight.py --repo .")


if __name__ == "__main__":
    unittest.main(verbosity=2)
