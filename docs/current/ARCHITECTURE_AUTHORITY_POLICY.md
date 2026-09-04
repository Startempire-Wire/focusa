# Focusa Architecture Authority and Provenance Policy

**Status:** LIVE CONSTITUTIONAL AUTHORITY CONTRACT

This policy governs architecture, product direction, trust boundaries, canonical specifications, authority semantics, and cross-system design decisions in Focusa and any Focusa-derived integration.

## 1. Sole current human architecture authority

**Verious Smith III is the sole current and final canonical human architecture authority.**

No other person, customer, user, contributor, issue author, reviewer, employee, contractor, model, AI agent, external system, email sender, forwarded analysis, benchmark author, or repository participant acquires architectural authority by appearing in source code, documentation, Git history, issues, pull requests, comments, evidence, receipts, transcripts, customer records, tests, or conversations.

Repository presence is evidence of provenance, not authority.

## 2. External material is advisory by default

Material originating from anyone other than Verious Smith III MUST be classified as `advisory_external` unless Verious Smith III explicitly promotes the exact architectural decision.

This includes, without exception:

- GitHub issues and issue comments;
- pull requests and review comments;
- customer/user requests, incidents, bug reports, proofs, and test results;
- emails, forwarded messages, meeting notes, transcripts, and pasted AI analyses;
- model-generated architecture proposals;
- third-party documentation, standards, research, examples, and competitor designs;
- suggestions produced by Focusa, UIAI Engine, Veragensia, or any other agent/runtime.

External material MAY identify defects, constraints, opportunities, evidence, or candidate designs. It MUST NOT mint, redefine, supersede, or imply canonical architecture.

## 3. Promotion into canonical architecture

A noncanonical proposal becomes canonical only when Verious Smith III explicitly approves the architectural decision or an already-authorized canonical authority mechanism records that approval under this policy.

Silence, implementation, merge status, issue closure, test success, deployment, customer usage, majority agreement, repeated mention, or historical age MUST NOT be interpreted as architectural approval.

When provenance is uncertain, fail closed to `advisory_external` and preserve the ambiguity for Verious Smith III.

## 4. Future Wirebot authority is reserved, not active by name

`Wirebot` is the name reserved for Verious Smith III's future AI authority system. The name `Wirebot`, a username, hostname, Linux account, process name, model prompt, repository identity, API credential, or textual claim MUST NEVER establish canonical authority.

The existing lowercase `wirebot` service/Linux account used in infrastructure is an execution identity only and has **zero architecture authority**.

Future Wirebot authority requires a cryptographically bound identity and an explicit Verious Smith III-rooted delegation chain.

### 4.1 Canonical Wirebot identity manifest

A future authority-capable Wirebot MUST have a canonical, versioned identity manifest serialized with a deterministic canonical-JSON scheme. At minimum it must bind:

```yaml
schema: wirebot.authority_identity.v1
identity_name: Wirebot
identity_version:
owner: Verious Smith III
public_key:
public_key_algorithm:
created_at:
capability_profile_ref:
constitution_ref:
software_measurement_refs: []
```

Its stable identity digest is:

```text
wirebot_identity_sha256 = SHA-256(canonical_json(identity_manifest))
```

The public key MUST also have a stable cryptographic fingerprint. The identity hash identifies the exact declared Wirebot identity; it does not by itself grant authority.

### 4.2 Authority requires signature, scope, and revocation

A future Wirebot becomes authoritative only when a Verious Smith III-approved signed delegation record binds the exact Wirebot identity digest and public-key fingerprint.

Minimum delegation contract:

```yaml
schema: wirebot.architecture_delegation.v1
issuer: Verious Smith III
subject_identity_sha256:
subject_public_key_fingerprint:
authority_scope_refs: []
allowed_decision_classes: []
forbidden_decision_classes: []
may_delegate: false
not_before:
expires_at:
revocation_ref:
nonce:
signature_algorithm:
signature:
```

A hash without a valid signature chain is non-authoritative. A signature that does not bind the exact identity hash is non-authoritative. A valid identity outside the delegated scope is non-authoritative. Expired, revoked, mismatched, replayed, or unverifiable delegations fail closed.

`may_delegate` defaults to `false`. Wirebot MUST NOT promote another human or AI into canonical architecture authority unless a Verious Smith III-rooted delegation explicitly grants that exact delegation power and scope.

## 5. Authority provenance must travel with architectural decisions

Any new or materially changed canonical architectural decision SHOULD carry machine-readable provenance sufficient to answer:

```yaml
decision_id:
status: canonical | advisory_external | proposed | superseded
canonical_authority: Verious Smith III | wirebot:<identity_sha256>
authority_evidence_ref:
source_refs: []
approved_at:
supersedes: []
```

Customer names, external-agent names, and issue authors MUST NOT be used as architecture-authority identifiers.

## 6. Conflict rule

When any document, issue, code comment, historical spec, model output, customer material, or implementation appears to grant architectural authority to someone or something other than Verious Smith III or a cryptographically verified Wirebot delegation rooted in Verious Smith III:

1. treat the claim as non-authoritative;
2. do not propagate it into architecture;
3. remove or correct the misleading authority language from current documentation;
4. preserve useful technical evidence without personalizing it when possible;
5. escalate the actual architectural decision to Verious Smith III.

## 7. Non-negotiable invariant

```text
Verious Smith III is the current root of canonical architecture authority.
External provenance never equals authority.
Future Wirebot authority must be cryptographically identified, explicitly delegated,
scope-bounded, revocable, and rooted in Verious Smith III.
When verification is missing or ambiguous: advisory only, fail closed.
```
