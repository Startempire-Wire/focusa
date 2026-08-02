# Install + Purchase Public Status

**Reviewed:** 2026-08-01  
**Status:** mandatory authority-issued licensing implementation is open and release-blocking for new evaluators/customers.

## Current truthful posture

Focusa has public installer, license-status, license-doctor, lifecycle, release, and purchase-explanation surfaces. The current installer also contains a legacy locally self-issued `--eval` path.

That path is no longer an approved evaluator onboarding mechanism.

Every official runtime—including Evaluation—must ultimately require:

```text
verified account/email and current terms
→ authority-created license and explicit product grants
→ registered node
→ authority-signed lease
→ product/feature/time/node/sequence/limit verification
→ entitled or recovery-only runtime
```

Missing or invalid license state is recovery-only, not Evaluation.

## Current public surfaces

- `install.focusa.dev/focusa` — existing installer bootstrap; new evaluator use is blocked until the device-code/signed-lease flow replaces self-issued Evaluation.
- `install.focusa.dev/license` — public license/purchase/manage explanation.
- `focusa license status` — current local status command; must migrate to the canonical signed-lease state.
- `focusa license doctor` — current diagnostics; must distinguish legacy local state from verified authority state.
- `focusa install --preflight --json` — approved non-mutating inspection; does not issue a license or activate Evaluation.
- public uninstall — approved data-preserving removal.

Target device-code commands and routes are specified in Spec 152 but must not be advertised as shipped until implementation and proof exist.

## Distribution block

Do not present any of these as approved current evaluator onboarding:

- `--eval` without authority issuance;
- missing license as Evaluation;
- source checkout as product entitlement;
- pairing/local/API/extension token as a product grant;
- UIAI health or loopback as UIAI entitlement;
- Spec 150 lifecycle success as complete customer readiness without Spec 150A entitlement binding.

## Private implementation boundary

Internal registry, signing, transaction, vendor, webhook, payment/refund, customer, evaluator-policy, email/consent, protected-capsule, and admin implementation details remain in private repositories/services.

The executable server-agent plan is maintained in the private authority repository. Public contracts are:

- Spec 152 — mandatory licensing and unified onboarding;
- Spec 150A — lifecycle entitlement overlay;
- Spec 152A — protected distribution and anti-tamper;
- `docs/contracts/spec152-supersession-and-integration-matrix.v1.yaml`.

## Readiness definition

Install/purchase is ready only when Evaluation and paid flows issue signed leases, installers and runtimes enforce the same product/feature posture, refund/revoke/expiry propagate, data-preserving recovery remains available, and the authority/Focusa/optional UIAI receipts reconcile.
