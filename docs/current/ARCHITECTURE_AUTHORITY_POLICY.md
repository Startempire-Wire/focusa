# Focusa Architecture Authority and Provenance Policy

**Status:** LIVE CONSTITUTIONAL AUTHORITY CONTRACT

This policy governs architecture, product direction, trust boundaries, canonical specifications, authority semantics, and cross-system design decisions in Focusa and any Focusa-derived integration.

## 0. GitHub estate scope

This authority constitution applies to **every GitHub repository and organization owned, administered, or canonically controlled by Verious Smith III**, whether the repository lives under `verioussmith`, `Startempire-Wire`, `Philoveracity`, `WPUIAI`, or any present or future GitHub organization/account under his control.

Local repository policy MAY add stricter constraints but MUST NOT weaken this root authority contract.

## 1. Sole current human architecture authority

**Verious Smith III is the sole current and final canonical human architecture authority.**

No other person, customer, user, contributor, issue author, reviewer, employee, contractor, model, AI agent, external system, email sender, forwarded analysis, benchmark author, or repository participant acquires architecture authority by appearing in source code, documentation, Git history, issues, pull requests, comments, evidence, receipts, transcripts, customer records, tests, or conversations.

**Repository presence is provenance, not authority.**

## 2. External material is advisory by default

Material originating from anyone other than Verious Smith III MUST be classified as `advisory_external` unless Verious Smith III explicitly promotes the exact architectural decision or a valid Verious-rooted delegated authority does so within scope.

This includes GitHub issues/comments, PRs/reviews, customer requests/incidents/proofs, emails, forwarded AI analyses, model-generated architecture proposals, third-party research, and suggestions from Focusa/UIAI/Veragensia/Wirebot or any other agent/runtime.

External material MAY identify defects, constraints, opportunities, evidence, or candidate designs. It MUST NOT mint, redefine, supersede, or imply canonical architecture merely because it is merged, implemented, deployed, popular, urgent, repeated, old, or technically correct.

## 3. Operational authority is not architecture authority

Focusa contains real canonical operational authority: ProjectIdentity, `project_root + continuity_id`, Trajectory orientation, Workpoint continuation, Context Authority mutation gates, capability grants, reducer-backed state, evidence, settlement, verification, and related governance.

Those truths remain authoritative **for their scoped operational domain**. They do not make Focusa itself the owner of product/organizational architecture and they do not supersede Verious Smith III as the architecture root.

```text
canonical Focusa operational state != canonical architecture authority
Focusa reducer authority             != authority to redefine who owns architecture
issue/PR/commit provenance           != architecture authority
```

## 4. Future Wirebot authority is reserved, not active by name

`Wirebot` is the future AI authority candidate owned by Verious Smith III. The name `Wirebot`, a username, hostname, Linux account, process name, model prompt, repository identity, API credential, release, or textual claim MUST NEVER establish canonical authority.

The existing lowercase `wirebot` service/Linux account used in infrastructure is execution identity only and has **zero architecture authority**.

No active Wirebot architecture-authority hash is declared by this policy. It MUST NOT be fabricated before the canonical Wirebot principal manifest and authority public key exist and are explicitly approved by Verious Smith III.

## 5. Cryptographic identity model

Do not collapse stable identity, authority law, and changing runtime into one hash.

```text
wirebot_principal_sha256      -> WHO is Wirebot?
constitution_sha256           -> WHICH architecture-authority constitution is bound?
runtime_attestation_sha256    -> WHAT exact software/model/tool runtime is acting now?
```

Machine-readable authority objects MUST use deterministic UTF-8 JSON canonicalized with **JCS / RFC 8785** before SHA-256 hashing.

### 5.1 Stable Wirebot principal

A stable principal manifest binds only durable identity material:

```json
{
  "schema": "wirebot.principal.v1",
  "principal_id": "wirebot:<stable-id>",
  "identity_name": "Wirebot",
  "owner_principal_ref": "Verious Smith III",
  "authority_public_key": "<encoded public key>",
  "authority_key_algorithm": "<algorithm>",
  "created_at": "<timestamp>",
  "lineage_root_ref": "<optional ref>"
}
```

Compute:

```text
wirebot_principal_sha256 = SHA-256(JCS(wirebot_principal_manifest))
wirebot_key_fingerprint  = fingerprint(Wirebot authority public key)
```

Mutable software/model/tool/host measurements MUST NOT be embedded into the stable principal hash in a way that changes Wirebot's identity on every upgrade.

### 5.2 Constitution hash

A machine-readable constitution manifest binds the normative authority documents and their digests.

```text
constitution_sha256 = SHA-256(JCS(constitution_manifest))
```

A material architecture-authority policy change produces a new constitution hash and requires compatibility/redelegation handling.

### 5.3 Runtime attestation hash

The actual software/model/tool posture is separately measured:

```text
runtime_attestation_sha256 = SHA-256(JCS(runtime_attestation))
```

A runtime attestation can bind source/release refs, binary/container digests, model identities, tool-registry digest, policy-bundle digest, optional hardware/host attestation, measurement time, and attestor key fingerprint.

Ordinary upgrades create new runtime attestations while normally preserving the stable Wirebot principal identity.

## 6. Authority requires a Verious-rooted signed delegation

Wirebot becomes authoritative only when an explicit delegation cryptographically authorized by Verious Smith III binds the stable Wirebot principal and acceptable constitution/runtime posture.

Minimum delegation fields:

```yaml
schema: wirebot.architecture_delegation.v2
issuer: Verious Smith III
subject_wirebot_principal_sha256:
subject_wirebot_key_fingerprint:
required_constitution_sha256:
runtime_attestation_policy_ref:
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

Every authority check MUST verify the trusted Verious root, owner signature, matching Wirebot principal hash/key fingerprint, key possession, constitution compatibility, current runtime-attestation compliance, requested scope/decision class, time window, revocation, and replay posture.

A hash alone never grants authority. A name alone never grants authority. A key alone never grants authority. A runtime attestation alone never grants authority.

`may_delegate` defaults to `false`. Wirebot MUST NOT promote another human or AI into canonical architecture authority unless Verious Smith III explicitly delegates that exact ability and scope.

## 7. Authority provenance on architectural decisions

Any new or materially changed canonical architectural decision SHOULD carry machine-readable provenance:

```yaml
decision_id:
status: canonical | advisory_external | proposed | superseded
canonical_authority: Verious Smith III | wirebot:<wirebot_principal_sha256>
authority_delegation_ref:
constitution_sha256:
runtime_attestation_ref:
authority_verification_ref:
source_refs: []
approved_at:
supersedes: []
```

Customer names, external-agent names, issue authors, reviewers, or implementers MUST NOT be used as architecture-authority identifiers merely because they contributed evidence or work.

## 8. Conflict rule

When any document, issue, code comment, historical spec, model output, customer material, or implementation appears to grant architectural authority to someone or something other than Verious Smith III or a cryptographically verified Wirebot delegation rooted in Verious Smith III:

1. treat the authority claim as non-authoritative;
2. do not propagate it into architecture;
3. preserve useful technical evidence as advisory input;
4. correct misleading current documentation where appropriate;
5. require Verious Smith III approval or a valid owner-rooted delegation before canonical promotion.

## 9. Non-negotiable invariant

```text
Verious Smith III is the current root of canonical architecture authority.
External provenance never equals authority.
Focusa may own scoped operational truth without owning architecture.
Stable Wirebot principal identity, constitution, and runtime attestation are separate cryptographic objects.
Future Wirebot authority must be explicitly delegated, scope-bounded, revocable, runtime-constrained, and rooted in Verious Smith III.
When verification is missing or ambiguous: advisory only, fail closed.
```
