# Focusa License FAQ

## Is Focusa open source?

No. Focusa is source-available under the Business Source License terms in `LICENSE.md`. See that file for the Change Date and future license terms for each version.

## Can I inspect, clone, or build the source?

Use of the repository source is governed by `LICENSE.md` and its Additional Use Grant. Source visibility, cloning, or a successful build does not itself create an official Focusa runtime entitlement, commercial rights, protected-component access, support, or official updates.

## Can I evaluate Focusa personally?

Yes, through an **authority-issued Evaluation license** under Spec 152.

The required official flow is:

```text
verified account/email and current terms
→ explicit Focusa Evaluation product/features/limits
→ node registration
→ authority-signed lease
→ runtime verification
→ bounded first project/Workpoint/Evidence proof
```

Evaluation is time-, node-, concurrency-, feature-, and usage-bounded. Missing or invalid license state is recovery-only, not Evaluation.

The current repository still contains legacy self-issued `--eval` behavior. That path is release-blocked and must not be used for new evaluators.

## Does personal or non-commercial source-use permission eliminate activation?

No. Legal source-use permission and official product entitlement are distinct boundaries.

The repository license may permit specified source uses, but an unmodified official runtime can still require authority-issued activation before mutable or execution-capable product features run. A public development/recovery shell may omit protected private components.

## Can I create a local Evaluation license or edit `license.json`?

No approved evaluator/customer flow uses a self-issued license, editable tier/feature fields, a local TOML override, a non-empty environment key, or a missing file as authority.

Local files may cache a signed lease or preserve legacy state for migration. Editing them cannot create a valid signature, product grant, node binding, lease sequence, feature set, limit policy, protected worker, or capsule key.

Test fixtures use a separate test trust root and must be rejected by production artifacts.

## Can I use Focusa inside my company or team?

Commercial, company, team, internal production, hosted-service, or client-delivery use requires an applicable commercial agreement and explicit signed product/features/nodes/seats grant.

## Can I use Focusa for paid client work?

Paid client work, managed agent operations, hosted services, redistribution, resale, and embedding require the corresponding commercial rights. A general Operator grant must not be assumed to include hosted, team, redistribution, or embedding rights unless explicitly stated.

## What happens when Evaluation expires or a license is revoked?

Focusa enters recovery-only posture. It preserves:

- health and version;
- license start/status/refresh/doctor;
- safe backup/export and user-data location;
- data-preserving repair;
- uninstall;
- explicit purge only after separate destructive confirmation.

It denies new project mutation, Workpoint/Evidence writes, agent execution, Silent Sessions, protected components, UIAI execution tokens, gated exports/releases, and update apply not granted by the signed lease.

License failure never deletes or encrypts operator data.

## Does pairing or a local token unlock Focusa?

No. Pairing, local API, Pi, extension, provider, webhook, or device tokens authenticate a caller or integration. They do not create Evaluation, commercial status, product grants, features, or limits.

Entitlement resolution must occur before pairing in the final first-run flow.

## Does a healthy UIAI Engine mean UIAI is licensed?

No. UIAI Engine is an independently enforceable product. A Focusa/bundle lease must explicitly grant `uiai-engine`; Focusa may then broker a narrower short-lived child token. UIAI verifies its own product, feature, node, parent sequence/digest, time, and limits.

## Can I fork Focusa and remove the checks?

Forking remains governed by `LICENSE.md`; it does not remove legal restrictions. Modified forks are unsupported and cannot obtain official protected workers/capsules, node-bound key envelopes, signed operation capabilities, authority services, or official protected updates merely by nulling a public check.

Anti-tamper controls raise the cost of bypass; they are not represented as mathematically unbreakable DRM.

## What are protected components?

Spec 152A permits selected commercially valuable Focusa/UIAI implementations to move into private, signed workers or encrypted feature capsules. The public repository retains stable contracts, verification, recovery, and operator-owned data formats. The authority delivers protected components only for a valid product/feature/node/release posture.

A gitignored directory inside a public repository is not the durable private boundary.

## What about developer access?

Production/private developer components require both private-repository authorization and an authority-issued developer license. Source access, a local environment variable, or a test fixture cannot create developer entitlement.

## Where are commercial and support terms?

See `COMMERCIAL.md`, `SUPPORT_TERMS.md`, `TRADEMARKS.md`, `CONTRIBUTING.md`, and the official purchase/license-management path.

Specific product grants, launch offers, team/enterprise rights, support, refunds, node limits, hosted/embedding/redistribution rights, and protected components are controlled by the applicable agreement and authority record.

## Canonical references

- `LICENSE.md`
- `docs/152-mandatory-authority-licensing-evaluation-entitlements-and-unified-onboarding-spec.md`
- `docs/150a-spec152-entitlement-overlay-and-lifecycle-integration.md`
- `docs/152a-protected-distribution-private-feature-capsules-and-anti-tamper-spec.md`
- `docs/contracts/spec152-supersession-and-integration-matrix.v1.yaml`
- `docs/current/INSTALLER_UPDATE_POLICY.md`
- `docs/current/FIRST_RUN_FLOW.md`
