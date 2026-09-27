#!/usr/bin/env python3
"""Filesystem regressions; Guardian and HTTP are fixtures, never live services."""

import concurrent.futures
import json
import os
from pathlib import Path
import pwd
import shutil
import stat
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
GUARD = ROOT / "scripts/run-guardian-release-cleanup.sh"
CHANNEL = ROOT / "tests/channel_separation_test.sh"


class ReleaseTempArtifactsTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        # Never execute an old producer that could clobber an ambient /tmp file.
        assert 'artifact="/tmp/focusa-guardian-release-cleanup-' not in GUARD.read_text()
        assert "/tmp/focusa-channel-body.json" not in CHANNEL.read_text()

    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="focusa-release-io-")
        self.addCleanup(self.temporary.cleanup)
        self.base = Path(self.temporary.name)
        self.base.chmod(0o755)
        self.repo = self.base / "repo"
        self.bin = self.base / "bin"
        self.shared = self.base / "shared"
        for path in (self.repo, self.bin, self.shared):
            path.mkdir(mode=0o755)
            path.chmod(0o755)
        self.shared.chmod(0o1777)
        for relative, source in (("scripts/run-guardian-release-cleanup.sh", GUARD),
                                 ("tests/channel_separation_test.sh", CHANNEL)):
            path = self.repo / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.parent.chmod(0o755)
            shutil.copyfile(source, path)
            path.chmod(0o644)
        self.stub("guardian", "#!/bin/sh\necho 'fixture disk checked'\n")
        self.stub("systemctl", '#!/bin/sh\ntest "${GUARDIAN_INACTIVE:-0}" != 1\n')
        self.stub("df", '#!/bin/sh\nprintf "Filesystem Blocks Used Available Capacity Mounted\\n/dev/fixture 100 10 90 10%% /\\n"\n')
        self.stub("cargo", "#!/bin/sh\necho unexpected-clean >&2\nexit 97\n")
        self.env = {
            "PATH": str(self.bin) + ":/usr/bin:/bin",
            "TMPDIR": str(self.shared),
            "FOCUSA_BASE_URL": "http://fixture.invalid",
            "RUN_LOG": str(self.base / "http.jsonl"),
            "SCOPE_LOG": str(self.base / "scope.txt"),
        }
        helper = self.repo / "tests/fixtures/admitted-project-scope.sh"
        helper.parent.mkdir()
        helper.write_text('''focusa_test_scope_create() {
  FOCUSA_FIXTURE_ROOT="$(mktemp -d "${TMPDIR}/channel-scope.XXXXXXXX")"
  printf '%s' "$FOCUSA_FIXTURE_ROOT" > "$SCOPE_LOG"
}
focusa_test_scope_cleanup() {
  rm -rf -- "$FOCUSA_FIXTURE_ROOT"
}
''')
        turns = self.repo / "apps/pi-extension/src/turns.ts"
        turns.parent.mkdir(parents=True)
        turns.write_text("// Focusa Minimal Applicable Slice\n")
        self.stub("curl", '''#!/usr/bin/python3
import json, os, pathlib, stat, sys
args = sys.argv[1:]
if "-sSI" in args:
    print("HTTP/1.1 200 OK\\r\\ncontent-type: text/event-stream\\r\\n")
    sys.exit(0)
p = pathlib.Path(args[args.index("-o") + 1])
p.write_text(json.dumps({"ok": True, "version": "fixture", "status": "accepted",
    "events": [{}], "context_stats": {"total_tokens": 1}, "candidates": [],
    "active_frame": {}, "stack": [], "semantic": [], "procedural": []}))
with open(os.environ["RUN_LOG"], "a") as log:
    log.write(json.dumps({"path": str(p), "mode": stat.S_IMODE(p.stat().st_mode)}) + "\\n")
print("500" if os.environ.get("FAIL_HEALTH") == "1" else "200", end="")
''')

    def stub(self, name, source):
        path = self.bin / name
        path.write_text(source)
        path.chmod(0o755)

    def guard(self, mode="pre", env=None, prefix=()):
        return subprocess.run(
            [*prefix, "/usr/bin/env", *[f"{k}={v}" for k, v in (env or self.env).items()],
             "/bin/bash", str(self.repo / "scripts/run-guardian-release-cleanup.sh"), mode],
            capture_output=True, text=True, timeout=15,
        )

    def receipt(self, result, uid=None):
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        receipt = json.loads(result.stdout)
        path = Path(receipt["artifact_path"])
        self.assertEqual(path.parent.parent, self.shared)
        self.assertEqual(json.loads(path.read_text()), receipt)
        self.assertEqual(stat.S_IMODE(path.parent.stat().st_mode), 0o700)
        self.assertEqual(stat.S_IMODE(path.stat().st_mode), 0o600)
        self.assertEqual(path.stat().st_uid, os.getuid() if uid is None else uid)
        self.assertEqual(receipt["cleaned"], [])
        self.assertEqual(receipt["scope"], "regenerable_release_artifacts_only")
        return path

    def test_concurrent_receipts_are_private_and_unique(self):
        with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
            results = list(pool.map(lambda _: self.guard(), range(2)))
        paths = [self.receipt(result) for result in results]
        self.assertNotEqual(*paths)

    def test_legacy_regular_file_and_symlink_are_untouched(self):
        old = self.shared / "focusa-guardian-release-cleanup-pre.json"
        old.write_text("prior evidence")
        before = old.stat()
        self.receipt(self.guard())
        self.assertEqual(old.read_text(), "prior evidence")
        self.assertEqual(old.stat().st_uid, before.st_uid)
        old.unlink()  # Only this test's own synthetic fixture.
        target = self.shared / "unrelated"
        target.write_text("preserve")
        old.symlink_to(target)
        self.receipt(self.guard())
        self.assertTrue(old.is_symlink())
        self.assertEqual(target.read_text(), "preserve")

    def test_unavailable_guardian_does_not_allocate_or_clean(self):
        result = self.guard(env=dict(self.env, GUARDIAN_INACTIVE="1"))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("refusing automated cleanup", result.stderr)
        self.assertEqual(list(self.shared.iterdir()), [])

    def test_receipt_allocation_failure_precedes_cleanup(self):
        target = self.repo / "target"
        target.mkdir()
        sentinel = target / "keep"
        sentinel.write_text("preserve")
        self.stub("mktemp", "#!/bin/sh\necho fixture-allocation-failed >&2\nexit 1\n")
        result = self.guard("post")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("fixture-allocation-failed", result.stderr)
        self.assertNotIn("unexpected-clean", result.stderr)
        self.assertEqual(sentinel.read_text(), "preserve")

    def test_different_users_in_protected_sticky_directory(self):
        if not shutil.which("sudo") or subprocess.run(
                ["sudo", "-n", "true"], capture_output=True).returncode:
            self.skipTest("cross-user proof requires noninteractive sudo")
        if int(Path("/proc/sys/fs/protected_regular").read_text()) != 2:
            self.skipTest("cross-user proof requires fs.protected_regular=2")
        first = self.receipt(self.guard())
        original = first.read_bytes()
        uid = pwd.getpwnam("nobody").pw_uid
        result = self.guard(prefix=("sudo", "-n", "-u", "nobody"))
        self.assertEqual(result.returncode, 0, result.stderr)
        second = Path(json.loads(result.stdout)["artifact_path"])
        self.assertEqual(second.parent.parent, self.shared)
        # Read/cleanup only the exact nobody-owned fixture, not ambient evidence.
        try:
            raw = subprocess.check_output(["sudo", "-n", "-u", "nobody", "/usr/bin/cat", str(second)], timeout=5)
            self.assertEqual(json.loads(raw), json.loads(result.stdout))
            self.assertEqual(second.parent.stat().st_uid, uid)
            self.assertEqual(stat.S_IMODE(second.parent.stat().st_mode), 0o700)
            metadata = subprocess.check_output(["sudo", "-n", "-u", "nobody", "/usr/bin/stat", "--format=%u:%a", str(second)], text=True, timeout=5)
            self.assertEqual(metadata.strip(), f"{uid}:600")
            self.assertEqual(first.read_bytes(), original)
            self.assertNotEqual(first, second)
        finally:
            subprocess.run(["sudo", "-n", "-u", "nobody", "/usr/bin/rm", "--", str(second)], check=True, timeout=5)
            subprocess.run(["sudo", "-n", "-u", "nobody", "/usr/bin/rmdir", "--", str(second.parent)], check=True, timeout=5)

    def test_channel_response_is_scoped_and_cleaned_on_success_and_failure(self):
        for fails in (False, True):
            with self.subTest(fails=fails):
                log = self.base / "http.jsonl"
                log.unlink(missing_ok=True)
                result = subprocess.run(
                    ["/bin/bash", str(self.repo / "tests/channel_separation_test.sh")],
                    env=dict(self.env, FAIL_HEALTH="1" if fails else "0"),
                    capture_output=True, text=True, timeout=15,
                )
                self.assertEqual(result.returncode, 1 if fails else 0, result.stdout + result.stderr)
                scope = Path((self.base / "scope.txt").read_text())
                records = [json.loads(line) for line in log.read_text().splitlines()]
                self.assertGreater(len(records), 0)
                self.assertEqual(len({r["path"] for r in records}), 1)
                for record in records:
                    self.assertEqual(Path(record["path"]).parent, scope)
                    self.assertEqual(record["mode"], 0o600)
                self.assertFalse(scope.exists(), "existing scope cleanup must remove the response")


if __name__ == "__main__":
    unittest.main(verbosity=2)
