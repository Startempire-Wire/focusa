# Phase 2 — Focusa Operator Preview (controlled cohort)

**Original date:** 2026-07-07  
**Reviewed:** 2026-08-01  
**Owner:** Verious Smith  
**Status:** planned and release-blocked by mandatory authority-issued licensing  
**Historical target tag:** `v0.9.74-dev`

## Supersession note

This cohort plan originally assumed a one-line installer and an open install/daemon/Workpoint path. That assumption is no longer valid.

Focusa is BSL source-available, not open source, and every official runtime—including Evaluation—must use an authority-issued signed entitlement. The current legacy `--eval`/no-key path is not approved for cohort use.

The preview may begin only after the combined Spec 150 + Spec 150A + Spec 152 gates pass for the selected preview release.

## Goal

Run a tight 5–10 person Operator Preview cohort before a broader public launch. The cohort should test whether verified evaluators can complete authority onboarding, install a coherent Focusa/UIAI set, reach a first Workpoint/Evidence proof, resume after handoff/compaction, understand limits/upgrade value, and recover safely without direct operator intervention.

## What the preview validates

| Question | Measure |
| --- | --- |
| Identity/license onboarding works | verified-email/device-code/signed-lease completion rate ≥ 9/10 |
| Install path works on real machines | entitled atomic install success ≥ 9/10 |
| Canonical state reconciles | authority, Focusa, and optional UIAI lease id/sequence/digests agree for every successful install |
| Daemon comes up truthfully | entitled or recovery-only state in under 60 seconds after install |
| First Workpoint is obvious | time to first entitled Workpoint under 5 minutes for ≥ 8/10 |
| First Evidence ref is obvious | time to first Evidence ref under 10 minutes for ≥ 8/10 |
| Resume works | successful resume after handoff/compaction ≥ 8/10 |
| Evaluation limits are understandable | users can identify remaining time/capacity and locked features without support |
| Paid activation is safe | synthetic/internal conversion or approved real conversion succeeds without reinstall/data loss |
| Value is understood | explanation-dependence score ≤ 2/5 |
| Expiry/recovery is safe | test fixture expires into recovery-only while export/backup/activation/uninstall remain usable |

## Cohort profile

Target mix:

- indie hackers using Cursor, Claude Code, Codex, or Pi;
- solo SaaS builders running long sessions;
- agency/consulting engineers carrying context across handoff;
- macOS/Tauri user for menubar preview;
- backend/VPS user for remote authentication and node registration;
- one security-minded tester for wrong-product, copied-state, and token-boundary observations.

Selection criteria:

- shipped an AI-assisted project recently;
- comfortable running a signed installer and reading bounded logs;
- willing to verify an email/account and accept Evaluation terms;
- willing to give a debrief;
- understands that Evaluation is non-commercial and bounded.

Do not enroll external cohort members with test fixtures, developer licenses, shared keys, or legacy self-issued Evaluation.

## Required preview install path

```text
official signed preview release
→ installer preflight
→ Evaluate / Activate
→ authority device code
→ verified account/email and terms
→ separate promotional-email choice
→ authority-issued Evaluation license
→ node registration and signed lease verification
→ atomic Focusa install
→ optional explicit UIAI product grant and child token
→ pairing
→ first project/Workpoint/Evidence walkthrough
```

Target command/route names are governed by Spec 152 and must not be documented here as shipped until implementation proof lands.

## Five-minute proof

Expected bounded proof after entitlement and install:

1. canonical license status reports `active_evaluation` or applicable paid state, with matching lease sequence and redacted product/features digest;
2. daemon health reports entitled readiness rather than merely process health;
3. project scope is explicit and verified;
4. Workpoint checkpoint returns an id;
5. Evidence linking returns a stable reference;
6. resume returns the canonical state and next action;
7. optional UIAI action succeeds only when the separate UIAI grant/feature/limit is present;
8. locked feature invocation fails before side effects with a safe manage-license action.

## Menubar preview

For selected macOS testers:

- verify signed/ad-hoc distribution mode truthfully;
- entitlement onboarding precedes pairing;
- device code/account flow uses the verified authority origin;
- pairing token is stored in Keychain but does not create entitlement;
- UIAI inclusion/lock state is explicit;
- restart preserves both pairing and canonical entitlement posture;
- revoke/expire fixture returns to recovery-only;
- app lifecycle, screenshots, and logs remain redacted.

Menubar issues may remain preview-specific, but any path that bypasses entitlement is P0.

## Out of scope

- broad public launch/Product Hunt;
- production scale claims;
- commercial hosted/multi-tenant use under Evaluation;
- testing with real customer database dumps or production signing secrets;
- claiming protected components are impossible to reverse engineer;
- private anti-abuse policy disclosure;
- using the preview to silently change purchased lifetime terms.

## Entry gates

All must pass before inviting cohort members:

- authority repository/live-server parity and rollback proof;
- staging device-code and verified-email flow;
- signed lease golden vectors across PHP/Rust/Go/Tauri where applicable;
- Focusa Spec 150A lifecycle entitlement binding;
- Bash/PowerShell installer parity and no production self-issued Evaluation;
- UIAI authentication/entitlement separation and route coverage;
- protected worker/capsule proof for included crown-jewel features;
- test-root/fixture exclusion in release artifacts;
- active-documentation consistency workflows green;
- data-preserving expiry/revoke/refund/uninstall proof;
- support, privacy, consent, and Evaluation terms published.

## Success criteria to advance

- ≥ 9/10 identity/license onboarding completion;
- ≥ 8/10 entitled install and first Workpoint under five minutes after authorization;
- ≥ 8/10 successful resume;
- zero unauthorized execution from missing, wrong-product, expired, revoked, copied, local-token, loopback, or pairing-only state;
- zero P0/P1 data-loss, secret-exposure, installer, authority, or entitlement defects;
- all successful installs produce reconciled lifecycle receipts;
- all failed/expired cases preserve recovery/data/uninstall;
- value-understanding target met.

If any criterion misses, run another controlled cohort rather than broadening launch.

## Risk register

| Risk | Mitigation |
| --- | --- |
| evaluator cannot authorize | device-code recovery, resend/verification support, redacted request id |
| authority outage | signed offline policy for existing lease; no fresh local Evaluation |
| install fails after authorization | atomic staging/rollback; preserve lease and user state |
| copied local files unlock product | signature/product/node/sequence validation; protected components absent/key-bound |
| UIAI health is mistaken for license | independent UIAI product/child-token proof |
| legacy docs lead to `--eval` | consistency workflows and supersession matrices |
| refund/revoke not propagated | bounded refresh, sequence, webhook/idempotency tests |
| Mac packaging warnings | truthful distribution mode and explicit operator instructions |
| agent bypasses public check | protected workers/capsules and independent operation-token checks |
| cohort data leaks | privacy-minimal authority records and redacted telemetry/evidence |

## Tracking

Use one private/appropriately scoped issue or Beads work item per cohort member. Record only:

- synthetic/opaque evaluator id;
- platform/version;
- milestone timestamps;
- success/failure classes;
- support interactions;
- consent-safe survey/debrief results;
- redacted receipt/evidence references.

Never place evaluator email, raw license/token/code, project contents, visited URLs, prompts, screenshots, or customer data in public issues.
