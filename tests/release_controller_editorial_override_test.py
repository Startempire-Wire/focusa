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
    def exercise(self, *, reason="Critical startup recovery", ci=True, version=True):
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
                if sys.argv[1:3] == ['run', 'list']:
                    print(json.dumps([{'headSha': os.environ['FIXTURE_SHA'], 'status': 'completed', 'conclusion': 'success'}] if os.environ['CI_VALID'] == '1' else []))
                elif sys.argv[1:3] == ['workflow', 'run']:
                    pathlib.Path('dispatch.json').write_text(json.dumps(sys.argv[1:]))
                else:
                    raise SystemExit('unexpected synthetic GitHub operation')
            """)
            script("scripts/next-version.py", """#!/usr/bin/env python3
                import json, os, pathlib, sys
                pathlib.Path('version.json').write_text(json.dumps(sys.argv[1:]))
                print(json.dumps({'violations': [] if os.environ['VERSION_VALID'] == '1' else ['synthetic version regression']}))
            """)
            script("scripts/release-gate.py", """#!/usr/bin/env python3
                from pathlib import Path
                Path('duplicate-editorial-gate').touch()
                raise SystemExit(97)
            """)
            script("scripts/create-dev-release-tag.sh", """#!/usr/bin/env bash
                set -eu
                printf '%s\n' "$@" > helper-args
            """)
            (root / "docs/current/.release-version-stamp").write_text("9.9.999\n")
            env = dict(os.environ, PATH=str(root / "bin") + os.pathsep + os.environ["PATH"],
                       APPROVED_TAG="v9.9.999", APPROVED_BASE="9.9", PUSH="true",
                       RELEASE_REASON=reason, GITHUB_REPOSITORY="synthetic/release-fixture",
                       FIXTURE_SHA="a" * 40, CI_VALID=str(int(ci)), VERSION_VALID=str(int(version)))
            result = subprocess.run(["bash", "-c", shell], cwd=root, env=env,
                                    capture_output=True, text=True, timeout=5)
            self.last_output = result.stdout + result.stderr
            return (result.returncode,
                    (root / "helper-args").read_text().splitlines() if (root / "helper-args").exists() else None,
                    json.loads((root / "version.json").read_text()) if (root / "version.json").exists() else None,
                    (root / "dispatch.json").exists(), (root / "duplicate-editorial-gate").exists())

    def test_reason_reaches_canonical_helper_without_duplicate_editorial_veto(self):
        code, args, version, dispatched, duplicate = self.exercise()
        self.assertEqual(code, 0, self.last_output)
        self.assertIn("--force-release", args)
        self.assertEqual(args[args.index("--release-reason") + 1], "Critical startup recovery")
        self.assertEqual(version, ["--tag", "v9.9.999", "--json"])
        self.assertTrue(dispatched)
        self.assertFalse(duplicate)

    def test_normal_release_does_not_acquire_an_override(self):
        code, args, version, dispatched, duplicate = self.exercise(reason="")
        self.assertEqual(code, 0, self.last_output)
        self.assertNotIn("--force-release", args)
        self.assertIsNone(version)
        self.assertTrue(dispatched)
        self.assertFalse(duplicate)

    def test_reason_cannot_override_exact_source_ci_failure(self):
        code, args, version, dispatched, duplicate = self.exercise(ci=False)
        self.assertNotEqual(code, 0)
        self.assertIsNone(args)
        self.assertIsNone(version)
        self.assertFalse(dispatched)
        self.assertFalse(duplicate)

    def test_reason_cannot_override_version_failure(self):
        code, args, version, dispatched, duplicate = self.exercise(version=False)
        self.assertNotEqual(code, 0)
        self.assertIsNone(args)
        self.assertIsNotNone(version)
        self.assertFalse(dispatched)
        self.assertFalse(duplicate)


if __name__ == "__main__":
    unittest.main()
