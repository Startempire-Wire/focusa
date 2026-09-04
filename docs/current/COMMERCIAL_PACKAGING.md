# Commercial Packaging

> **Spec 152 boundary:** Source visibility and a successful source build are not product entitlement. Every official Focusa runtime—including Evaluation—requires an authority-issued signed lease. Selected premium implementations may be absent from the public tree and delivered as protected workers/capsules under Spec 152A.

Focusa packaging is local-first by default: the product is not a cloud memory service, and commercial packaging must preserve operator-controlled data, explicit scope authority, redaction-first public surfaces, and data-preserving recovery after license denial or expiry.

## Distribution classes

| Class | Audience | Packaging | Entitlement/support boundary |
| --- | --- | --- | --- |
| Source reference / development shell | developers studying or contributing under BSL terms | public source checkout, contracts, tests, recovery/development surfaces | not an automatically entitled Evaluation; no protected commercial components; unsupported modified forks cannot obtain protected delivery by claiming success |
| Authority-issued Evaluation | verified prospective evaluator | signed public product set plus explicitly granted evaluation features/limits and optional protected capsules | time/node/concurrency bounded; recovery/export/uninstall preserved after expiry |
| Operator local | licensed professional operator | signed CLI/daemon/TUI/session-runner/menubar/Pi/agent-context set plus entitled protected components | install/update support, local data ownership, product-qualified features |
| Team self-hosted | licensed team/multi-agent workstreams | daemon/services/adapters/policy package plus team grants | seat/node/team scope, backup/migration, security review |
| Enterprise | regulated organization | self-hosted bundle, protected/air-gap artifacts where contracted, support agreement | deployment review, SSO/auth planning, audit artifacts, pinned policy |
| Developer | authorized core developer | private development channel and test trust boundary | authority-issued developer license; never inferred from source access or environment variables |

“Community source” must not be used as shorthand for a perpetual runnable free edition. The BSL and Additional Use Grant define source-use rights; Spec 152 defines official product activation and Evaluation issuance.

## Package artifacts

Depending on signed product/features, packaging may include:

- Focusa public/recovery daemon;
- Focusa CLI and TUI;
- Mac menubar app;
- Pi extension and generated capability contracts;
- UIAI Engine public gateway/recovery component;
- private protected Focusa/UIAI workers or feature capsules;
- signed entitlement trust roots and key-rotation metadata;
- generated current docs (`docs/current/*`);
- release proof bundle;
- security/trust docs;
- installer/update policy;
- migration/backup/recovery policy.

No package includes authority signing private keys, reusable capsule content keys, raw customer records, or production test bypasses.

## Commercial and evaluator readiness gates

- Version consistency and complete compatible component-set proof pass.
- Tool/route/client entitlement coverage is complete.
- Spec 150 lifecycle mechanics pass **and** Spec 150A entitlement binding passes.
- Spec 152 authority staging E2E passes for Evaluation, paid, bundle, wrong-product, expired, revoked, refunded, offline, and node-limit cases.
- Spec 152A protected-component proof passes for selected crown-jewel features.
- Installer Bash/PowerShell, daemon, CLI, TUI, menubar, Pi, UIAI, and authority report the same lease id/sequence and compatible grant digests.
- Missing/invalid entitlement starts recovery-only rather than Evaluation.
- Security/trust, installer/update, migration/backup, privacy/consent, support, refund/cancellation, and data-processing terms are explicit.
- Public proof artifacts are redacted and `publish_allowed=true` only after review.
- Documentation consistency gate passes; active guides do not recommend legacy self-issued `--eval`.
- Data export, backup, repair, activation, and uninstall remain usable after expiry/revocation.

## Current implementation status

Focusa v0.9.142 contains strong lifecycle and release machinery, but the mandatory signed-entitlement integration is still open. The legacy installer and runtime Evaluation behavior must not be used to claim evaluator/customer distribution readiness.

Current truthful claim:

```text
lifecycle and product mechanics are advanced;
mandatory authority-issued licensing and protected-distribution closure are release blockers
```

## License and billing requirements

Any paid or evaluation packaging must include or link the current:

- BSL/license text and applicable commercial terms;
- product grants and named license class;
- Evaluation duration/limits when applicable;
- support scope and SLA where sold;
- billing owner, refund/cancellation, replacement, and revocation policy;
- privacy/data-processing boundaries;
- transactional versus promotional-email consent rules;
- device/node/seat limits;
- offline-grace and update policy;
- protected-component and watermark disclosure where applicable.

## Public/private source boundary

Keep public:

- BSL and public commercial/evaluation terms;
- stable entitlement/IPC/error schemas;
- signature verification and trust roots;
- recovery/export/uninstall behavior;
- public-safe tests and integration contracts.

Keep private or server-side:

- issuance/signing keys and customer records;
- evaluator eligibility and anti-abuse policy;
- payment/refund synchronization;
- private commercial caps and experiments;
- proprietary workers, prompts, routing, algorithms, and premium assets selected for protection;
- capsule encryption/key-envelope implementation details;
- operational admin routes and raw audit evidence.

A gitignored folder inside a public repository is not an acceptable durable protection boundary. Use private repositories/packages, isolated signing systems, and server-side services with stable public interfaces.

## Non-goals

- No hosted cloud-memory claim by default.
- No automatic public publishing of private project state.
- No adapter-specific fork of cognitive authority.
- No installer replacing live production binaries without Context Authority and entitlement preflight.
- No destructive response to license failure.
- No claim that encryption makes administrator-controlled native code impossible to recover.

## Proof

- Existing static guard: `tests/commercial_packaging_static_test.sh`
- Entitlement documentation guard: `tests/spec152_documentation_consistency_gate.py`
- Supersession matrix: `docs/contracts/spec152-supersession-and-integration-matrix.v1.yaml`
- Related: Spec 150A, Spec 152, Spec 152A, `FIRST_RUN_FLOW.md`, `INSTALLER_UPDATE_POLICY.md`
