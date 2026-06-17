# Security Policy

This document explains how to report security vulnerabilities in Focusa and how we handle them.

## Supported versions

The `main` branch of the Focusa repository receives security updates. Released
versions of Focusa Operator, Focusa CLI, Focusa Core, and the Focusa TUI receive
security updates for the duration of their commercial support window as documented
in the relevant license agreement.

Source builds from `main` outside the commercial support window are not eligible
for backported fixes. Customers with an active commercial license are covered per
their license terms.

## Reporting a vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Email security reports to:

`security@startempirewire.com`

Use the PGP key published at `https://startempirewire.com/.well-known/security.asc`
when sending sensitive material. The PGP fingerprint is also pinned at the same
URL.

Include the following in your report:

- A clear description of the vulnerability and its impact.
- The exact version, commit SHA, or build artifact affected.
- Reproduction steps, proof-of-concept code, or a recorded run that triggers the
  issue. Run the repro against a local evaluation build, not a production
  deployment.
- The environment you observed the issue in: OS, runtime version, deployment
  topology (loopback, remote, hosted).
- Whether you intend to disclose publicly and on what timeline.
- Your contact details and, if you want one, an acknowledgement preference.

We aim to acknowledge new reports within two business days. We do not commit to a
hard patch SLA during the public report phase.

## What we will do

1. Triage the report and assign an internal owner.
2. Reproduce the issue in a private build.
3. Decide on a fix and coordinate disclosure timing with the reporter.
4. Release a fix on `main` and, if the issue affects a commercially licensed
   release, ship a patched build to the license registry.
5. Publish a security advisory in `docs/SECURITY_ADVISORIES/` once the fix ships,
   or sooner if the issue is being actively exploited.

## Scope

This policy covers Focusa itself: the Focusa CLI, the Focusa Core runtime, the
Focusa TUI, the Focusa Workloop, the Focusa Trajectory, the Focusa Workpoint
format, and the related schemas and protocol files.

The policy does **not** cover:

- UIAI Engine — see `WPUIAI/uiai-engine/SECURITY.md`.
- Focusa Arena proof stage — see `arena.focusa.dev/security.txt`.
- Focusa install gateway — see `install.focusa.dev/security.txt`.
- Third-party dependencies. Report upstream first and let us know so we can
  track the patch.

## Out of scope

- Best-practice recommendations without a concrete exploit.
- Findings on forks, unofficial builds, or unmaintained branches.
- Findings that require the operator to disable the source-available license
  enforcement.
- Social engineering, phishing, or physical attacks.

## Recognition

We maintain a public acknowledgement list at
`docs/SECURITY_CONTRIBUTORS.md` for reporters who ask to be credited. We do not
operate a paid bug bounty program at this time.
