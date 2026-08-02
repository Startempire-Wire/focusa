# Spec 152C — Authority/Focusa/UIAI Coordination Contract

**Status:** Cross-repository implementation contract  
**Created:** 2026-08-01

## Canonical ownership

| Concern | Authority | Focusa | UIAI Engine |
| --- | --- | --- | --- |
| account/email/consent | owns | displays safe state | no duplicate signup |
| license/product/feature/limit policy | owns/signs | verifies/enforces Focusa | verifies/enforces UIAI |
| node registration | owns canonical records | creates/proves node key | binds engine instance/child token |
| lease schema/signature | signs/canonical payload | Rust verifier | Go verifier |
| project/Workpoint authority | none | owns | receives scope metadata only |
| caller authentication | service/browser auth | pairing/local/Pi auth | API/extension/child auth |
| protected capsule keys | owns envelopes/policy | unwraps Focusa capsules | unwraps UIAI capsules |
| usage reservation | canonical policy/reconciliation | Focusa usage/receipts | UIAI session/job usage |
| refund/revoke/expire | owns sequence/state | refreshes/denies/recovery | refreshes/denies/recovery |

No repository may create a parallel source of truth for another column.

## Shared golden-vector package

Authority publishes synthetic test vectors containing:

- canonical payload JSON;
- RFC8785 canonical bytes;
- domain-separated signed bytes digest;
- Ed25519 public key/key id/signature;
- expected parsed claims/state;
- invalid variants: bit flip, wrong product, stale sequence, expired, revoked, unknown key/schema.

The vectors contain no production keys/accounts. PHP authority, Rust Focusa, Go UIAI, and Tauri/JavaScript status projection must pass the same fixtures before integration proceeds.

## Version negotiation

Every request/lease/token/manifest declares schema/version. Clients advertise supported ranges. Authority never silently emits an unsupported schema.

Compatibility response includes:

```text
supported
minimum_client_version
minimum_contract_version
upgrade_required
recovery_allowed
```

Unsupported clients enter recovery/upgrade posture, not local Evaluation.

## Idempotency and request correlation

All authority mutations and client lifecycle mutations carry:

- opaque request id;
- caller-generated idempotency key;
- operation type;
- node/license/product binding;
- safe retry status.

The same idempotency key and canonical request cannot create duplicate license, node, lease, usage, email, refund, or capsule operations.

## Time and sequence

- Authority timestamps use UTC RFC3339 with explicit precision contract.
- Lease sequence is monotonic per license/node authority state.
- Clients persist highest accepted sequence and reject rollback.
- Local clocks never extend Evaluation or offline validity.
- Rollback of software does not roll back revocation, sequence, or absolute expiry.

## Bundle flow

```text
Authority signs explicit Focusa + UIAI product grants
→ Focusa verifies parent lease
→ Focusa requests/mints narrow UIAI child token
→ UIAI verifies audience, parent id/sequence/digest, node/client, feature/limit, expiry, nonce
→ both products emit redacted receipts referencing same parent state
```

A generic `bundle` label does not imply products unless explicit grants are present.

## Failure ownership

| Failure | Owner/action |
| --- | --- |
| account/eligibility/payment/refund | authority stable error and manage action |
| signature/schema/key rotation | authority + both verifiers/golden vectors |
| Focusa product/feature | Focusa denial before side effect |
| UIAI product/feature | UIAI denial before allocation |
| pairing/auth | relevant product; cannot change entitlement |
| project scope | Focusa; cannot change license |
| protected worker/capsule | product + authority envelope; public gateway cannot fake success |
| authority outage | signed offline policy only; no new local license |

## Integration staging order

1. freeze lease v1 schema and golden vectors;
2. implement authority shadow issuance;
3. implement Focusa verifier/recovery without enforcement;
4. implement UIAI verifier/recovery without enforcement;
5. compare shadow decisions and receipts;
6. implement device-code/node/product flows;
7. enforce preview Evaluation;
8. implement child tokens and UIAI gates;
9. protected worker pilot;
10. stable enforcement and legacy retirement.

## Cross-repository completion receipt

A successful install/onboarding receipt includes compatible digests for:

- authority lease payload and sequence;
- Focusa verifier/feature ledger;
- UIAI verifier/feature ledger when granted;
- lifecycle transaction;
- public/protected component set;
- node id and product grants;
- first proof/Workpoint when selected;
- recovery and rollback policy.

Sensitive account/email/key/token/capsule values are excluded.
