# Contributing to Focusa

Focusa is currently source-available and commercially licensed.

External contributions are not accepted unless Startempire Wire has explicitly approved the contribution path first. Approved contributors may be required to sign a Contributor License Agreement or assignment before code, docs, designs, tests, issues, or other materials are incorporated. See `legal/CONTRIBUTOR_LICENSE_AGREEMENT_TEMPLATE.md` for the current draft CLA structure.

Do not submit proprietary, confidential, third-party, or employer-owned material unless you have the legal right to contribute it.

Small bug reports and discussion are welcome, but code or documentation patches are accepted only under the approved contributor process.

## Release lanes

All changes land in a release lane per `docs/release-strategy.md`:
security/critical fixes ride the patch lane (0.9.x), features and non-critical
fixes batch into the minor lane (0.10.x), breaking changes require a planned
major bump. Commits must remain Conventional Commit subjects so the version
policy (`scripts/next-version.py`) can classify each change correctly.

## Python contract tests

On Linux/macOS, use Python 3.11 or newer and the committed hash-locked
dependencies instead of ambient user or system packages:

```bash
bash scripts/ci/setup-python-test-env.sh
source .focusa-python-test-venv/bin/activate
python scripts/ci/verify-python-test-env.py
```

Run Python contract gates from that activated environment. Override
`FOCUSA_PYTHON_TEST_VENV` only when an alternate isolated location is needed.
CI uses the same setup helper and exports its interpreter for following steps.
The default environment is ignored by Git; never commit it or remove package
hash checks to make dependency installation pass.
