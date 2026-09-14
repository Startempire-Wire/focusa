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
