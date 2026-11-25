#!/usr/bin/env python3
"""Exercise the actual controller shell without tags, network or publication."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import textwrap
import unittest

ROOT = Path(__file__).resolve().parents[1]


class ControllerEditorialOverrideTest(unittest.TestCase):
    def exercise(self, *, reason="Critical startup recovery", helper_exit=0):
        workflow = (ROOT / ".github/workflows/dev-release-tag.yml").read_text()
        step = workflow.split("      - name: Create approved immutable tag\n", 1)[1]
        shell = textwrap.dedent(step.split("        run: |\n", 1)[1])
        with tempfile.TemporaryDirectory(prefix="release-controller-fixture-") as tmp:
            root = Path(tmp)
            for name in ["bin", "scripts", "docs/current"]:
                (root / name).mkdir(parents=True, exist_ok=True)

            def script(name, body):
                path = root / name
                shebang, _, content = body.partition("\n")
                path.write_text(shebang + "\n" + textwrap.dedent(content))
                path.chmod(0o700)

            script("bin/git", """#!/usr/bin/env bash
                set -eu
                test "$1" = rev-parse
                printf '%s\n' "$FIXTURE_SHA"
            """)
            script("bin/gh", """#!/usr/bin/env python3
                import json, os, pathlib, sys
                if sys.argv[1:3] != ['workflow', 'run']:
                    raise SystemExit('unexpected synthetic GitHub operation')
                pathlib.Path('dispatch.json').write_text(json.dumps(sys.argv[1:]))
            """)
            script("scripts/create-dev-release-tag.sh", """#!/usr/bin/env bash
                set -eu
                printf '%s\n' "$@" > helper-args
                exit "$HELPER_EXIT"
            """)
            (root / "docs/current/.release-version-stamp").write_text("9.9.999\n")
            env = dict(os.environ, PATH=str(root / "bin") + os.pathsep + os.environ["PATH"],
                       APPROVED_TAG="v9.9.999", APPROVED_BASE="9.9", PUSH="true",
                       RELEASE_REASON=reason, GITHUB_REPOSITORY="synthetic/release-fixture",
                       FIXTURE_SHA="a" * 40, HELPER_EXIT=str(helper_exit),
                       GITHUB_STEP_SUMMARY=str(root / "summary"))
            result = subprocess.run(["bash", "-c", shell], cwd=root, env=env,
                                    capture_output=True, text=True, timeout=5)
            self.last_output = result.stdout + result.stderr
            return (result.returncode,
                    (root / "helper-args").read_text().splitlines() if (root / "helper-args").exists() else None,
                    (root / "dispatch.json").exists(),
                    (root / "summary").read_text() if (root / "summary").exists() else "")

    def test_reason_reaches_canonical_helper_without_duplicate_editorial_veto(self):
        code, args, dispatched, summary = self.exercise()
        self.assertEqual(code, 0, self.last_output)
        self.assertIn("--force-release", args)
        self.assertEqual(args[args.index("--release-reason") + 1], "Critical startup recovery")
        self.assertTrue(dispatched)
        self.assertFalse(summary)

    def test_normal_release_does_not_acquire_an_override(self):
        code, args, dispatched, summary = self.exercise(reason="")
        self.assertEqual(code, 0, self.last_output)
        self.assertNotIn("--force-release", args)
        self.assertTrue(dispatched)
        self.assertFalse(summary)

    def test_pending_source_ci_yields_without_dispatch(self):
        code, _, dispatched, summary = self.exercise(helper_exit=78)
        self.assertEqual(code, 0, self.last_output)
        self.assertFalse(dispatched)
        self.assertIn("Source CI pending", summary)

    def test_reason_does_not_override_helper_failure(self):
        code, _, dispatched, summary = self.exercise(helper_exit=1)
        self.assertEqual(code, 1, self.last_output)
        self.assertFalse(dispatched)
        self.assertFalse(summary)


if __name__ == "__main__":
    unittest.main()
