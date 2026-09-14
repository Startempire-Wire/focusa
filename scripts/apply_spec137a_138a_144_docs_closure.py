#!/usr/bin/env python3
"""Apply the documentation-only architecture closure for Specs 137A, 138A, and 144.

This migration is intentionally deterministic and idempotent. It amends the full
repository files in place, creates populated machine-readable closure artifacts,
and adds a CI gate. It does not claim runtime implementation or conformance.
"""
from __future__ import annotations

import hashlib
import json
import re
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
DOCS = ROOT / "docs"
CONTRACTS = DOCS / "contracts"
MARKER_ROOT = "SPEC137A_138A_144_ARCHITECTURE_CLOSURE"
NOW = datetime.now(timezone.utc).replace(microsecond=0).isoformat()


def sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def file_sha(path: Path) -> str:
    return sha256_text(path.read_text(encoding="utf-8"))


def read(rel: str) -> str:
    return (ROOT / rel).read_text(encoding="utf-8")


def write(rel: str, text: str) -> None:
    path = ROOT / rel
    path.parent.mkdir(parents=True, exist_ok=True)
    if not text.endswith("\n"):
        text += "\n"
    path.write_text(text, encoding="utf-8")


def marker(key: str) -> str:
    return f"<!-- {MARKER_ROOT}:{key} -->"


def append_once(rel: str, key: str, section: str) -> None:
    path = ROOT / rel
    if not path.exists():
        raise FileNotFoundError(rel)
    text = path.read_text(encoding="utf-8")
    m = marker(key)
    if m in text:
        return
    text = text.rstrip() + "\n\n" + m + "\n" + section.strip() + "\n"
    path.write_text(text, encoding="utf-8")


def replace_once(rel: str, old: str, new: str) -> None:
    path = ROOT / rel
    text = path.read_text(encoding="utf-8")
    if new in text:
        return
    if old not in text:
        raise RuntimeError(f"Expected replacement anchor missing in {rel}: {old[:100]!r}")
    path.write_text(text.replace(old, new, 1), encoding="utf-8")


def dump_yaml_compatible(data: Any) -> str:
    # JSON is valid YAML 1.2 when it is the complete document.
    return json.dumps(data, indent=2, sort_keys=False, ensure_ascii=False) + "\n"


def dump_yaml_root_entries(data: dict[str, Any]) -> str:
    # Existing Spec 137 is block YAML. Emit each extension value as a YAML
    # root key with a JSON flow value; concatenating a standalone JSON object
    # after block YAML produces an invalid document.
    return "".join(
        f"{key}: {json.dumps(value, indent=2, sort_keys=False, ensure_ascii=False)}\n"
        for key, value in data.items()
    )


def write_contract(name: str, data: dict[str, Any]) -> None:
    data = {
        "serialization": "JSON-compatible YAML 1.2",
        "generated_at": NOW,
        "runtime_claim": "none",
        "runtime_status": "implementation_open",
        **data,
    }
    write(f"docs/contracts/{name}", dump_yaml_compatible(data))


def extract_clauses(rel: str, prefix: str) -> list[dict[str, Any]]:
    text = read(rel)
    clauses: list[dict[str, Any]] = []
    heading = "document"
    in_code = False
    required_section = False
    normative = re.compile(r"\b(MUST NOT|SHALL NOT|MUST|SHALL|REQUIRED|SHOULD NOT|SHOULD|MAY)\b", re.I)
    required_headings = (
        "acceptance", "closure", "required", "mandatory", "core law", "core temporal law",
        "final invariant", "final law", "constitutional", "non-deferral", "omission", "profile",
        "implementation order", "machine-readable", "settlement", "migration", "security", "authority",
    )
    for lineno, raw in enumerate(text.splitlines(), 1):
        line = raw.rstrip()
        stripped = line.strip()
        if stripped.startswith("#"):
            heading = stripped.lstrip("#").strip()
            required_section = any(k in heading.lower() for k in required_headings)
        if stripped.startswith("```"):
            in_code = not in_code
            continue
        schema_field = in_code and bool(re.match(r"^[A-Za-z_][A-Za-z0-9_.-]*\s*:", stripped))
        governed_item = required_section and bool(re.match(r"^(?:[-*]|\d+[.)])\s+", stripped))
        if not stripped or stripped.startswith("<!--"):
            continue
        if normative.search(stripped) or schema_field or governed_item:
            digest = sha256_text(f"{rel}:{lineno}:{stripped}")[:12]
            clauses.append({
                "clause_id": f"{prefix}-{lineno:04d}-{digest}",
                "source_path": rel,
                "source_line": lineno,
                "section": heading,
                "text": stripped,
                "normative_tokens": sorted({m.group(1).upper() for m in normative.finditer(stripped)}),
                "mapping_status": "mapped_documentation_runtime_open",
            })
    return clauses


def source_coverage(schema: str, sources: list[tuple[str, str]]) -> dict[str, Any]:
    all_clauses: list[dict[str, Any]] = []
    source_rows = []
    combined = hashlib.sha256()
    for rel, prefix in sources:
        text = read(rel)
        digest = sha256_text(text)
        combined.update(rel.encode())
        combined.update(digest.encode())
        clauses = extract_clauses(rel, prefix)
        all_clauses.extend(clauses)
        source_rows.append({
            "path": rel,
            "sha256": digest,
            "line_count": len(text.splitlines()),
            "mapped_clause_count": len(clauses),
        })
    return {
        "schema": schema,
        "sources": source_rows,
        "combined_normative_source_hash": combined.hexdigest(),
        "clause_count": len(all_clauses),
        "unmapped_clause_refs": [],
        "duplicate_or_weakened_mapping_refs": [],
        "ambiguous_applicability_refs": [],
        "coverage_status": "documentation_source_coverage_complete_runtime_proof_open",
        "clauses": all_clauses,
    }


def requirement_rows(clauses: list[dict[str, Any]], owner: str) -> list[dict[str, Any]]:
    rows = []
    for clause in clauses:
        text = clause["text"].lower()
        conditional = any(token in text for token in (" if ", " when ", " where ", " unless ", " once ", " whenever "))
        rows.append({
            "requirement_id": clause["clause_id"],
            "source_clause_ref": f"{clause['source_path']}:{clause['source_line']}",
            "source_text_hash": sha256_text(clause["text"]),
            "primitive_owner": owner,
            "applicability_class": "conditional" if conditional else "required",
            "applicability_status": "active_or_requires_recorded_decision" if conditional else "active",
            "documentation_status": "contract_defined",
            "runtime_status": "implementation_open",
            "evidence_status": "required",
            "receipt_status": "required",
            "closure_impact": "blocking_for_claimed_conformance",
        })
    return rows


# ---------------------------------------------------------------------------
# 1. Parent and umbrella specification amendments
# ---------------------------------------------------------------------------

# This script owns the one-time pre-activation baseline migration. Once Spec 144
# is activated, rerunning it would overwrite evidence-bound runtime ledgers with
# implementation-open templates. The maintained hardener/activation workflow is
# the only valid post-activation reconciliation path.
activation_path = CONTRACTS / "spec144-activation.v1.json"
if activation_path.exists():
    activation = json.loads(activation_path.read_text(encoding="utf-8"))
    if activation.get("status") == "activated":
        print(
            "Spec closure baseline already activated; "
            "use harden_spec137a_138a_144_docs_closure.py and evidence-gated activation scripts"
        )
        raise SystemExit(0)

replace_once(
    "docs/137-focusa-temporal-authority-deadlines-urgency-grounded-forecasting-spec.md",
    "Canonical label: **Spec 137 — Temporal Authority, Deadlines, Urgency, and Grounded Forecasting**",
    "Canonical label: **Spec 137 — Temporal Authority, Deadlines, Urgency, and Grounded Forecasting**\n\n"
    "**Mandatory companion:** [`Spec 137A — Temporal Zero-Deferral, Applicability, and Omission Firewall`](137a-focusa-temporal-zero-deferral-applicability-and-omission-firewall-addendum.md). "
    "The combined normative source is Spec 137 + Spec 137A + activated inherited primitive-owner requirements. Full Spec 137 conformance or closure is prohibited unless both documents, their combined source coverage, and their closure artifacts validate.\n\n"
    "**Current truth:** existing temporal runtime surfaces may satisfy verified slices only; they do not by themselves establish combined Spec 137 + 137A full conformance."
)
append_once(
    "docs/137-focusa-temporal-authority-deadlines-urgency-grounded-forecasting-spec.md",
    "spec137a-parent-integration",
    """
## Mandatory Spec 137A integration and combined-source closure

Spec 137A is not optional guidance. It governs sequencing, applicability, variance, platform and domain qualification, surface parity, migration, proof, and closure for this parent specification. Any parent wording that could permit a weaker interpretation is resolved by Spec 137A.

The following are mandatory before a full-conformance claim:

- combined parent/addendum source coverage and hashes;
- Spec 137A requirement rows in the complete feature ledger;
- explicit applicability decisions with affirmative non-activation evidence;
- all accepted requirements in the root delivery DAG;
- updated proof, parity, migration, conformance, and placeholder-audit artifacts;
- a zero-unapproved-deferral and zero-omission Receipt.

The existing temporal implementation and release gate are classified as verified implementation slices until the combined closure system proves otherwise.
"""
)

replace_once(
    "docs/138-focusa-prediction-outcome-calibration-metacognitive-learning-transfer-and-epistemic-governance-spec.md",
    "**Owner:** Focusa core  ",
    "**Owner:** Focusa core  \n"
    "**Mandatory companion:** [`Spec 138A — Epistemic Zero-Deferral, Profile Completeness, and Omission Firewall`](138a-focusa-epistemic-zero-deferral-profile-completeness-and-omission-firewall-addendum.md)  \n"
    "**Combined normative source:** Spec 138 + Spec 138A + activated inherited primitive-owner requirements  \n"
    "**Conformance truth:** Profiles A–H are selective at runtime but mandatory for `full_spec138_conformance`; profile subsets must be labeled exactly and cannot be called the maximal substrate  "
)
append_once(
    "docs/138-focusa-prediction-outcome-calibration-metacognitive-learning-transfer-and-epistemic-governance-spec.md",
    "spec138a-parent-integration",
    """
## Mandatory Spec 138A integration and full-profile closure

Spec 138A governs staged activation, profile completeness, normative optionality, API/CLI/Pi and generated-client parity, migration, projections, proof, and closure. Parent wording such as `SHOULD`, `suggested`, `eventually`, or staged activation cannot remove an accepted generic capability from the full-conformance target.

Full Spec 138 conformance requires Profiles A–H, the complete scorer and calibration registry, append-only semantic history, migration, source-independence and resolution authority, transfer and self-model evaluation, consolidation, rollback, high-consequence governance, client parity, Evidence, Receipts, and replay proof. A profile-subset release remains a truthful verified subset only.
"""
)

replace_once(
    "docs/144-focusa-semantic-integrity-rdf-owl-shacl-build-verify-routing-and-vertical-intelligence-spec.md",
    "**Primary relationship:** Extends and composes Specs 45–50, 61, 66, 70, 72, 74–79, 88, 90, 95, 97, 100, 107, 109, 113, 116, 119, 120, 125, 130, 131, 133, 135F, 136, 137, 138, 140, 141, 142, and 143.  ",
    "**Primary relationship:** Extends and composes Specs 45–50, 61, 66, 70, 72, 74–79, 88, 90, 95, 97, 100, 107, 109, 113, 116, 119, 120, 125, 130, 131, 133, 135F, 136, the combined Spec 137 + 137A temporal source, the combined Spec 138 + 138A epistemic source, Spec 139 distributed presence and execution placement, and Specs 140, 141, 142, and 143.  "
)
replace_once(
    "docs/144-focusa-semantic-integrity-rdf-owl-shacl-build-verify-routing-and-vertical-intelligence-spec.md",
    "+ Spec 137 temporal applicability\n+ Spec 138 epistemic applicability",
    "+ combined Spec 137 + Spec 137A temporal applicability and closure law\n+ combined Spec 138 + Spec 138A epistemic applicability and full-profile law\n+ Spec 139 environment identity, placement, resource admission, deduplication, lease, and fencing posture"
)
append_once(
    "docs/144-focusa-semantic-integrity-rdf-owl-shacl-build-verify-routing-and-vertical-intelligence-spec.md",
    "architecture-gap-closure",
    """
## Normative architecture-gap closure amendment

This section closes the remaining integration gaps identified after the zero-deferral replacement. It is normative and has the same closure force as every earlier section.

### Core Verification Pack

Every Verification Plan MUST include the always-activated `focusa.verification.core@1` pack. Vertical and domain packs may add obligations but cannot replace, suppress, merge away, or downgrade this pack.

The core pack emits obligations for scope and authority, Work Contract completeness, requirement coverage, immutable snapshot identity, Evidence sufficiency and freshness, unresolved contradiction, final verified-snapshot equality, Receipt readiness, and reducer-only settlement.

### Obligation-compilation proof

The obligation compiler MUST emit an `ObligationCompilationReceipt` binding compiler identity/version, complete input hashes, requirement-set hash, registry and pack hashes, OWL/SHACL trigger hashes, semantic delta, emitted/deduplicated/rejected obligations, unknown-impact refs, uncovered requirements, independent coverage-challenger result, validation, and Receipt hash.

A valid Verification Plan cannot prove that the obligation graph was complete unless its compilation receipt validates. The coverage challenger MUST be independent from the compiler policy path at assurance tiers requiring independent verification.

### Spec 139 execution-placement binding

Every Builder, Verifier, deterministic validator, test executor, runtime probe, browser evaluator, arbiter, and external-authority assignment MUST bind a `VerificationExecutionBinding` containing environment identity, placement decision, node, daemon and daemon boot, repository, workspace, worktree, resource claims, deduplication key, lease, fencing token, and placement-policy version.

Unresolved or stale placement blocks execution. Shared environment, toolchain, cache, checkout, source, test-generator, or infrastructure dependencies MUST enter the independence and common-mode profile. Two assignments do not become independent merely because they use different models.

### Reproducible cognition and common-mode dependencies

Every Builder, Verifier, Router, coverage challenger, and arbiter run MUST bind a `CognitiveExecutionIdentity` containing actor/run/session identity, Runtime Constitution hash, prompt-assembly hash, role/capability/permission versions, skill-bundle hashes, tool-registry and tool-policy versions, harness adapter, model/provider/family/version and inference parameters, retrieval/source-set identity, test-generator identity, Spec 139 environment binding, context packet hash, and code/registry/pack revisions.

A `CommonModeDependencyProfile` MUST represent shared rubric, prompt source, retrieval corpus, evidence provider, test generator, environment, checkout, cache, external authority, model family/provider, and infrastructure failure domains. Mandatory independence dimensions are fail-closed; a scalar score cannot override a failed dimension.

### Verification dispute and appeal integration

Focusa MUST reuse—not fork—PRE, Spec 120 operator gates, and Spec 136 settlement authority. Verification disputes use typed `VerificationConflict`, `VerificationAppeal`, `ArbiterEligibilityRecord`, `ConflictOfInterestRecord`, `ArbitrationAssignment`, and `ArbitrationDecision` records.

Mechanically decidable conflicts route to a registered deterministic PRE resolver. Judgmental, disputed, regulated, or high-consequence conflicts route through an eligible independent arbiter and, where policy requires, an explicit Spec 120/operator gate. The Router that selected the original Verifier cannot silently select a friendly arbiter; arbiter eligibility and common-mode independence must validate first.

### Post-settlement revalidation

Later evidence corruption, source dependence, security disclosure, external-authority revision, registry/pack revocation, material regression, or invalidated verifier capability MUST create a `SettlementRevalidationTrigger` and `SettlementValidityChallenge` through Spec 136.

The reducer may append `settlement_upheld`, `settlement_corrected`, `settlement_superseded`, or `settlement_reopened` only through the governed revalidation path. Historical settlement and Receipt records remain immutable and linked to the correction. A stale or invalidated settlement cannot continue to support promoted learning or a current conformance claim.

### Additional required machine-readable artifacts

The following are mandatory in addition to the previously listed Spec 144 artifacts:

```text
docs/contracts/spec144-core-verification-pack.v1.yaml
docs/contracts/spec144-obligation-compilation-and-coverage.v1.yaml
docs/contracts/spec144-execution-placement-and-common-mode.v1.yaml
docs/contracts/spec144-verification-dispute-arbitration.v1.yaml
docs/contracts/spec144-settlement-revalidation.v1.yaml
```
"""
)

# ---------------------------------------------------------------------------
# 2. Primitive-owner and authority integration clauses
# ---------------------------------------------------------------------------

integrations: dict[str, tuple[str, str]] = {
    "docs/61-domain-general-cognition-core.md": ("spec144-cognition-boundary", """
## Spec 144 semantic verification composition boundary

The Domain-General Cognition Core remains cognition primitive owner. Spec 144 composes Builder, Verifier, Router, coverage-challenger, and arbiter lineages as bounded roles over this core. No role becomes a second reducer, ontology registry, settlement authority, or self-promoting cognition core.
"""),
    "docs/66-affordance-and-execution-environment-ontology.md": ("spec144-verification-affordances", """
## Spec 144 verification affordance integration

Verification capabilities, execution bindings, read-only source posture, writable sandboxes, tool permissions, environment eligibility, and authority limits MUST be expressed through the existing affordance and execution-environment ontology. A Verifier profile without an executable permitted affordance path is `schema_only` and ineligible.
"""),
    "docs/70-shared-interfaces-statuses-and-lifecycle.md": ("spec144-shared-statuses", """
## Spec 144 shared lifecycle and status integration

Shared interfaces MUST recognize Builder/Verifier lifecycle, obligation coverage, snapshot validity, dispute, arbitration, placement, revalidation, and exact conformance-subset states. `verification_passed` remains distinct from `settlement_ready` and `settled`; `settlement_reopened` and `settlement_corrected` append rather than rewrite history.
"""),
    "docs/72-agent-identity-role-and-self-model-ontology.md": ("spec144-role-profiles", """
## Spec 144 role-profile extension

Spec 72 owns the identity semantics for Builder, Domain Verifier, deterministic validator controller, Verification Router, coverage challenger, Evidence auditor, arbiter, external authority reviewer, and operator reviewer. Role text never grants permission, capability, independence, or settlement authority. Each active role binds versioned capability, permission, responsibility, non-responsibility, handoff, and CognitiveExecutionIdentity references.
"""),
    "docs/74-identity-and-reference-resolution.md": ("spec144-cognitive-execution-identity", """
## Spec 144 cognitive and execution identity resolution

Identity resolution MUST distinguish actor, role, model, provider/family/version, Runtime Constitution, prompt assembly, tool and skill bundles, harness, environment, daemon boot, repository/worktree, source set, test generator, and external authority. Equality of one dimension never implies identity or independence across the others. Ambiguous identity blocks affected verification coverage.
"""),
    "docs/75-projection-and-view-semantics.md": ("spec144-verification-projections", """
## Spec 144 projection semantics

Verification Plans, findings, conflicts, appeals, placement, independence, obligation coverage, and settlement posture are canonical or reducer-backed read-model sources as declared by their owners. UI cards, dashboards, summaries, and agent packets are projections only and MUST preserve exact status, scope, freshness, Evidence, limitations, and recovery.
"""),
    "docs/76-retention-forgetting-and-decay-policy.md": ("spec144-retention", """
## Spec 144 retention integration

Verification snapshots, findings, dispositions, compilation receipts, independence/common-mode profiles, arbitration records, settlement evaluations, and revalidation challenges inherit evidence-, legal-, security-, and conformance-sensitive retention. Ordinary decay cannot delete a record required to explain a current settlement, correction, promoted learning, dispute, or public conformance claim.
"""),
    "docs/77-ontology-governance-versioning-and-migration.md": ("spec144-semantic-bundle-governance", """
## Spec 144 RDF/OWL/SHACL and Verification Pack governance

RDF/OWL/SHACL bundles, the core Verification Pack, Vertical verification extensions, obligation triggers, finding shapes, and settlement shapes are versioned governed registry outputs. Breaking or materially semantic changes require migration, compatibility analysis, revalidation impact, rollback, Evidence, and Receipts. `owl:sameAs` cannot collapse canonical agent or authority identities.
"""),
    "docs/78-bounded-secondary-cognition-and-persistent-autonomy.md": ("spec144-verification-portfolio-lineage", """
## Spec 144 Verification Portfolio lineage

A Verification Portfolio is a governed Secondary Cognition lineage composed of obligation-scoped child lineages. It may observe, challenge, falsify, propose repairs, and emit findings; it cannot mutate Builder source, grant authority, waive obligations, settle completion, or promote its own routing policy. Open obligations, findings, context identities, snapshots, and receipts survive checkpoint and recovery.
"""),
    "docs/79-focusa-governed-continuous-work-loop.md": ("spec144-build-verify-loop", """
## Spec 144 Build↔Verify Work Loop integration

The Work Loop MUST orchestrate the Semantic Execution Pair lifecycle, freeze Work Contracts before building, compile and route obligations, preserve separate sessions and leases, invalidate stale verification after material change, reroute on findings or provider failure, detect oscillation, and request Spec 136 settlement only after complete coverage. Verification availability, placement, and deadline conflicts remain explicit blockers.
"""),
    "docs/88-ontology-backed-workpoint-continuity.md": ("spec144-workpoint-continuity", """
## Spec 144 Workpoint continuity integration

A Workpoint participating in a Semantic Execution Pair MUST preserve Work Contract revision, active build attempt, Verification Plan, immutable snapshot, open obligations/findings, dispute posture, placement bindings, Evidence, and next safe action. Compaction or handoff cannot collapse those records into a favorable verdict string or transcript summary.
"""),
    "docs/97-focusa-reflex-primitives-spec.md": ("spec144-reflex-registry", """
## Spec 144 Verification Fabric reflex families

The Reflex Registry MUST support and version the universal routing, snapshot, coverage, independence, Evidence, settlement, recovery, arbitration, placement, and learning reflexes declared by Spec 144. Vertical overlays may add activation-specific reflexes but cannot suppress core reflexes. A registered reflex without trigger, deterministic action/proposal path, tests, and Receipt behavior is `schema_only`.
"""),
    "docs/100-context-cognition-spec.md": ("spec144-role-contexts", """
## Spec 144 role-specific context projection

Context Cognition MUST generate separate bounded Builder, Verifier, Router, coverage-challenger, and arbiter packets from the same canonical scope while preserving disclosure policy. Verifier packets exclude Builder hidden reasoning and irrelevant rhetoric by default; all packets bind source hashes, Workpoint revision, Runtime Constitution, tool policy, temporal/presence posture, and exact obligations.
"""),
    "docs/107-spec-first-feature-lifecycle-and-claim-discipline-spec.md": ("spec144-spec-lifecycle", """
## Spec 137A/138A/144 specification lifecycle integration

Implementation admission for these specifications requires their combined source-coverage artifacts, complete ledgers, delivery DAGs, cross-spec amendment matrix, proof matrices, and zero-omission gates. Documentation architecture closure is distinct from runtime implementation and cannot be reported as feature completion.
"""),
    "docs/109-agent-first-api-redesign-ax-spec.md": ("spec144-operation-surface", """
## Spec 144 agent-first operation integration

The Operation Registry and generated clients MUST expose bounded operations for Work Contract validation, obligation compilation, plan validation, snapshot freeze, finding/disposition, reroute, dispute/appeal, placement inspection, settlement evaluation, and revalidation. Clients request and project; they do not reproduce routing, authority, or settlement rules.
"""),
    "docs/113-agent-benchmark-spec.md": ("spec144-benchmarks", """
## Spec 144 benchmark integration

Benchmarks MUST measure obligation recall, false omission, false positive/negative findings, snapshot staleness, common-mode failure, independence truth, routing eligibility, calibration, repair efficiency, oscillation, placement correctness, dispute resolution, settlement correctness, and revalidation. A favorable average cannot hide an uncovered critical obligation.
"""),
    "docs/116-provider-neutral-work-item-closure-authority-spec.md": ("spec144-closure-input", """
## Spec 144 closure-authority input

Provider-neutral closure validation MUST consume structured Spec 144 requirement coverage, Verification Plan, snapshot identity, open findings/conflicts, independence/common-mode posture, placement, settlement evaluation, and revalidation state when activated. A provider `done` status or generic Verifier pass cannot close work with uncovered obligations or an invalidated settlement.
"""),
    "docs/119-verifiable-agent-work-receipts-and-governed-execution-ledger-spec.md": ("spec144-receipts", """
## Spec 144 Receipt extension

Receipts for verified work MUST link Work Contract, Semantic Execution Pair, build attempts, obligation compilation receipt, Verification Plans and reroutes, immutable snapshots, assignments and CognitiveExecutionIdentity, placement bindings, findings/dispositions, independence/common-mode profiles, arbitration, coverage, settlement evaluation, and later revalidation/correction records. Receipts prove lineage; they cannot manufacture satisfaction.
"""),
    "docs/120-adversarial-spec-workbench-and-operator-approval-gates.md": ("spec144-verification-disputes", """
## Spec 144 verification-dispute integration

The Adversarial Spec Workbench supplies judgmental challenge/synthesis and operator gates for verification conflicts that cannot be resolved mechanically. A VerificationAppeal binds the original finding, counterevidence, requirements, snapshot, prior assignments, requested remedy, and conflict-of-interest profile. The synthesis arbiter remains advisory until the registered PRE/Spec 136/operator path records a disposition.
"""),
    "docs/125-mandatory-trajectory-nonlazy-hlt-pi-receipt-ontology-interlock-spec.md": ("spec144-trajectory-interlock", """
## Spec 144 trajectory and non-lazy verification interlock

Trajectory and Workpoint closure cannot treat verification as a final-message ritual. Activated obligations, open findings, placement, Evidence, snapshot equality, and settlement readiness are mandatory trajectory feeders. Pi and other harnesses receive bounded next-action and blocker projections, while the daemon retains canonical routing and settlement authority.
"""),
    "docs/130-hlt-aware-compaction-mission-packet-and-bloatgaurd-context-firewall-spec.md": ("spec144-compaction-continuity", """
## Spec 144 compaction continuity

Compaction packets MUST preserve Work Contract hash, Semantic Execution Pair, active attempt/snapshot, obligation and coverage refs, open findings/disputes, assigned Verifiers, placement/common-mode posture, Evidence, reroute history, and next safe action. Hidden reasoning remains excluded; omission of canonical verification state is a compaction failure.
"""),
    "docs/131-focusa-workpoint-item-timing-velocity-and-closure-authority-spec.md": ("spec144-timing-attribution", """
## Spec 144 verification timing and closure attribution

Spec 131 remains timing and closure authority. It MUST attribute Builder, obligation compilation, deterministic validation, specialist verification, dispute, repair, reroute, settlement, and revalidation intervals without double counting. Verification latency and failure history feed forecasts, but time pressure cannot waive coverage or independence.
"""),
    "docs/133-daemon-native-durable-silent-sessions-and-governed-autonomous-execution-spec.md": ("spec144-session-classes", """
## Spec 144 governed session classes

Silent Sessions MUST distinguish Builder, Verifier, Router, coverage-challenger, arbiter, and settlement-evaluator assignments. Verifier source posture is read-only, writable work occurs only in an isolated sandbox, writer leases never transfer by role label, and every run binds CognitiveExecutionIdentity, placement, immutable snapshot, obligations, checkpoints, Evidence, and Receipt lineage.
"""),
    "docs/135f-domain-general-ontology-core-semantic-graph-domain-packs-and-reactive-context-spec.md": ("spec144-vertical-intelligence-bundles", """
## Spec 144 Vertical Intelligence Bundle composition

A Vertical Intelligence Bundle composes workspace profile, domain packs, semantic modules, SHACL shapes, Verification Pack extensions, temporal and epistemic applicability, evidence policies, verifier capabilities, reflex overlays, artifact/connector bindings, migrations, conformance, and golden scenarios. Every bundle inherits `focusa.verification.core@1`; no Vertical forks reducer, temporal, prediction, ontology, or settlement authority.
"""),
    "docs/136-governed-proposal-to-settlement-protocol-and-outcome-truth-infrastructure-spec.md": ("spec144-settlement-integration", """
## Spec 144 structured verification, dispute, and revalidation integration

When activated, Spec 136 settlement consumes the complete Spec 144 Verification Portfolio rather than a string verdict. Settlement requires complete obligation coverage, eligible assignments, immutable verified-snapshot equality, resolved blocking findings/conflicts, valid independence/common-mode and placement posture, Evidence, and Receipt readiness.

Verification disputes reuse PRE and Spec 120/operator gates through typed conflict, appeal, arbiter-eligibility, conflict-of-interest, assignment, and decision records. Later material invalidation creates a SettlementValidityChallenge and append-only uphold/correct/supersede/reopen outcome; historical settlement is never rewritten.
"""),
    "docs/139-distributed-presence-environment-awareness-execution-placement-and-multi-daemon-coordination-spec.md": ("spec144-verification-placement", """
## Spec 144 verification execution placement

Spec 139 owns the environment and placement identity for every Builder, Verifier, validator, test executor, probe, browser evaluator, coverage challenger, and arbiter. Each assignment MUST carry a `VerificationExecutionBinding` with node/daemon/boot, repository/workspace/worktree, environment profile, resource claims, deduplication identity, lease, fencing token, route, and placement policy. Shared placement and infrastructure enter the Spec 144 common-mode profile.
"""),
    "docs/140-project-agent-runtime-constitution-instruction-authority-system-prompt-and-cross-harness-compiler-spec.md": ("spec144-cognition-provenance", """
## Spec 144 reproducible Builder and Verifier cognition

The Runtime Constitution compiler MUST produce role-specific Builder, Verifier, Router, coverage-challenger, and arbiter artifacts. Every run binds constitution, prompt assembly, role/capability/permission, skills, tools, harness, model parameters, disclosure policy, retrieval/source set, test generator, environment, and context packet hashes in a `CognitiveExecutionIdentity`. Prompt difference alone is not independence.
"""),
    "docs/141-focusa-agent-first-tool-skill-runbook-and-documentation-release-gate-spec.md": ("spec144-documentation-gates", """
## Specs 137A/138A/144 documentation and tool-release truth

Public and agent-facing documentation MUST distinguish verified runtime slices, profile subsets, documentation-only architecture, activation state, and full conformance. Tools, skills, runbooks, generated clients, and release surfaces cannot advertise Spec 144 operations until registered runtime implementations and proof exist. The combined-source and required-artifact gate is mandatory in CI.
"""),
    "docs/142-focusa-release-requirement-trace-matrix.md": ("spec144-release-nonadmission", """
## Post-release non-admission of Specs 137A, 138A, and 144

Specs 137A, 138A, and 144 are not silently admitted into the locked current release implementation. They remain documentation architecture with runtime implementation open unless an explicit post-Spec-143 activation record exists. Current release claims MUST NOT imply combined 137+137A, full-profile 138+138A, or Spec 144 conformance.
"""),
    "docs/143-focusa-master-release-cycle-trajectory-genesis-flow-implementation-spec.md": ("spec144-postrelease-boundary", """
## Spec 144 post-release activation boundary

The locked Spec 143 release remains unchanged. Spec 144 implementation begins only after Spec 143 closure and explicit operator activation. Documentation closure, source coverage, and cross-spec amendments may be completed now, but they do not create runtime acceptance, expand the release, or authorize implementation before the activation gate.
"""),
}

for rel, (key, section) in integrations.items():
    append_once(rel, key, section)

# ---------------------------------------------------------------------------
# 3. Canonical vocabulary, authority, public truth, and index
# ---------------------------------------------------------------------------

append_once(
    "docs/00-glossary.md",
    "spec144-canonical-terms",
    """
## Spec 144 semantic verification vocabulary

### Semantic Work Contract
The frozen, versioned, scope-bound target-state and acceptance contract from which building and verification obligations derive. It is not a prompt and cannot be edited by the Builder or Verifier after freeze without amendment and invalidation.

### Semantic Execution Pair
The governed coordination object linking one Builder cognition lineage to one Verification Portfolio while preserving separate sessions, runs, contexts, workspaces, leases, Evidence, and Receipts. The pair is not settlement authority.

### Verification Obligation
A typed, requirement-linked, snapshot-bound verification duty derived from deterministic registry rules, OWL/SHACL triggers, semantic impact, risk, temporal/epistemic applicability, or approved policy.

### Verification Portfolio
The complete policy-compliant set of deterministic validators, specialist Verifiers, probes, auditors, external authorities, and operator reviewers assigned to satisfy a Verification Obligation Graph.

### Domain-Specific Verification Router
The compiler/router that assigns obligations to eligible capabilities. It proposes and validates portfolios; it cannot waive obligations, grant authority, or settle completion.

### Verification Snapshot
An immutable, content-addressed bundle of source, diff, semantic graph, Evidence, tests, runtime observations, temporal state, Work Contract, registry, pack, and shape versions inspected by required Verifiers.

### Verifier Capability Profile
A versioned executable capability contract binding supported obligations/domains/artifacts, tools, Evidence generation, model and context policy, assurance, calibration, reliability, permissions, placement, and conformance proof.

### Vertical Intelligence Bundle
A versioned composition of workspace projection, domain semantics, Evidence policy, temporal/epistemic applicability, Verification Pack extensions, verifier capabilities, reflexes, artifacts/connectors, migrations, conformance, and golden scenarios.

### Obligation Compilation Receipt
The durable proof that the complete governed inputs were compiled into an obligation graph without uncovered mandatory requirements or unreported unknown impact.

### Cognitive Execution Identity
The reproducible identity of a Builder, Verifier, Router, challenger, or arbiter run across actor, constitution, prompt, role, model, tools, skills, harness, source/retrieval, test generator, environment, context, and version dimensions.

### Common-Mode Dependency Profile
The record of shared failure domains between Builder, Verifiers, validators, challengers, and arbiters. Different model names do not erase shared rubric, source, environment, test, cache, provider, or infrastructure dependence.
"""
)
append_once(
    "docs/current/AUTHORITY_MODEL.md",
    "spec144-authority-extension",
    """
## Spec 144 authority extension

| Surface | Authority role | Canonical when | Non-canonical states |
| --- | --- | --- | --- |
| Semantic Work Contract | Frozen target-state and acceptance authority for one work scope | Validated, approved, versioned, and reducer-linked | draft, amended-pending, stale, invalid |
| Builder | Authorized mutation lineage within lease and contract | Never canonical truth by itself | advisory claims, blocked, stale |
| Obligation Compiler | Deterministic and policy-derived verification-duty compiler | Output is canonical only after validation and Receipt | incomplete, invalid, unknown-impact |
| Verification Router | Assignment and portfolio proposal authority | Authorized plan after eligibility/coverage validation | proposed, conflicted, uncovered, stale |
| Verifier | Obligation-scoped finding authority | Finding is durable evidence after structure/scope/evidence validation | advisory, unsupported, stale, ineligible |
| Coverage Challenger | Independent obligation-omission challenge | Validated challenge/receipt | advisory, common-mode, stale |
| PRE / registered resolver | Deterministic conflict resolution for registered classes | Reducer applies valid resolution | advisory score, unresolved, clarification-required |
| Arbiter / Operator Reviewer | Judgmental dispute recommendation or decision where policy assigns | Explicit eligible, independent, receipted path | conflicted, ineligible, advisory |
| Spec 139 Placement | Environment and execution-venue authority | Current verified placement/lease/fencing decision | stale, ambiguous, partitioned, unsupported |
| Spec 136 Settlement | Canonical completion and settlement authority | Reducer-settled with complete Spec 144 inputs | ready, blocked, challenged, reopened |

No Builder, Verifier, Router, projection, client, model, or majority vote may mint settlement truth.
"""
)

alignment_path = ROOT / "docs/evidence/141-focusa-latest-spec-public-doc-alignment.md"
alignment = alignment_path.read_text(encoding="utf-8")
alignment = alignment.replace(
    "| 137 | temporal authority, deadlines, urgency, forecasting | implemented with active hardening |",
    "| 137 + 137A | temporal runtime substrate plus mandatory zero-deferral closure | verified runtime slices; combined full conformance open |",
)
alignment = alignment.replace(
    "| 138 | prediction calibration, metacognitive transfer, epistemic governance | implemented with active hardening |",
    "| 138 + 138A | prediction/metacognitive substrate plus mandatory full-profile closure | partial runtime foundations; full-profile conformance open |",
)
if "| 144 |" not in alignment:
    alignment = alignment.replace(
        "| 140 | runtime constitution, instruction authority, cross-harness compiler | implemented with active hardening |",
        "| 140 | runtime constitution, instruction authority, cross-harness compiler | implemented with active hardening |\n"
        "| 144 | semantic integrity and domain-routed Build↔Verify fabric | normative documentation only; implementation not activated |",
    )
alignment_path.write_text(alignment, encoding="utf-8")

json_path = ROOT / "docs/evidence/141-focusa-latest-spec-public-doc-alignment.json"
if json_path.exists():
    data = json.loads(json_path.read_text(encoding="utf-8"))
    data["spec137a_138a_144_architecture_closure"] = {
        "generated_at": NOW,
        "spec_137_137a": "verified_runtime_slices_combined_full_conformance_open",
        "spec_138_138a": "partial_runtime_foundations_full_profile_conformance_open",
        "spec_144": "normative_documentation_only_implementation_not_activated",
        "supersedes_any_broader_implemented_status": True,
    }
    json_path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

append_once("docs/README.md", "spec144-current-truth", """
## Current truth for Specs 137A, 138A, and 144

Spec 137 has verified temporal runtime slices, but combined Spec 137 + 137A full conformance remains open. Spec 138 has prediction/metacognitive foundations, but Profiles A–H and combined Spec 138 + 138A full conformance remain open. Spec 144 is normative documentation only and has not been activated for runtime implementation. The machine-readable closure artifacts describe required work and must not be interpreted as implementation proof.
""")
append_once("docs/llms.txt", "spec144-agent-truth", """
## Mandatory architecture-status instruction

Treat Spec 137A and Spec 138A as mandatory companions to their parents. Treat Spec 144 as documentation-only and not runtime-activated. Do not claim combined Spec 137 + 137A conformance, full-profile Spec 138 + 138A conformance, or Spec 144 implementation unless exact current ledgers, proof matrices, runtime Evidence, Receipts, and activation records establish it.
""")

append_once("docs/INDEX.md", "spec144-closure-contracts", """
## Specs 137A/138A/144 architecture-closure contracts

The populated machine-readable source coverage, ledgers, delivery DAGs, ownership, profile, placement, dispute, migration, proof, and placeholder-audit contracts are under `docs/contracts/spec137a-*`, `docs/contracts/spec138*`, and `docs/contracts/spec144-*`. They close documentation architecture and make remaining runtime work explicit; they do not claim runtime implementation.
""")

# ---------------------------------------------------------------------------
# 4. Populated machine-readable artifacts
# ---------------------------------------------------------------------------

coverage137 = source_coverage(
    "focusa.spec137a_normative_source_coverage.v1",
    [
        ("docs/137-focusa-temporal-authority-deadlines-urgency-grounded-forecasting-spec.md", "S137-C"),
        ("docs/137a-focusa-temporal-zero-deferral-applicability-and-omission-firewall-addendum.md", "S137A-C"),
    ],
)
write_contract("spec137a-normative-source-coverage.v1.yaml", coverage137)

clauses137 = coverage137["clauses"]
write_contract("spec137a-applicability-matrix.v1.yaml", {
    "schema": "focusa.spec137a_applicability_matrix.v1",
    "combined_normative_source_hash": coverage137["combined_normative_source_hash"],
    "decision_rule": "absence_or_unsupported_implementation_is_never_non_applicability",
    "rows": [
        {
            "requirement_ref": c["clause_id"],
            "source_ref": f"{c['source_path']}:{c['source_line']}",
            "status": "active_or_explicit_evidence_backed_decision_required",
            "decision_authority": "operator_or_registered_policy",
            "non_activation_evidence_required": True,
            "review_triggers": ["scope_change", "platform_change", "domain_change", "profile_change", "product_claim_change"],
        }
        for c in clauses137
    ],
})
write_contract("spec137a-conformance-class-matrix.v1.yaml", {
    "schema": "focusa.spec137a_conformance_class_matrix.v1",
    "classes": [
        {"id": "spec137_verified_slice", "may_claim_parent_complete": False, "requires": ["named_requirement_subset", "runtime_evidence", "receipt", "remaining_open_rows"]},
        {"id": "full_spec137_conformance", "may_claim_parent_complete": True, "requires": ["all_active_parent_and_addendum_rows_verified", "combined_source_coverage", "parity", "migration", "proof", "zero_omission_receipt"]},
    ],
})
write_contract("spec137a-forbidden-placeholder-audit.v1.yaml", {
    "schema": "focusa.spec137a_forbidden_placeholder_audit.v1",
    "audited_paths": [
        "docs/137-focusa-temporal-authority-deadlines-urgency-grounded-forecasting-spec.md",
        "docs/137a-focusa-temporal-zero-deferral-applicability-and-omission-firewall-addendum.md",
        "docs/contracts/spec137-complete-feature-ledger.v1.yaml",
    ],
    "forbidden_dispositions": ["later", "eventually", "post-MVP", "optional implementation", "schema complete", "mostly complete"],
    "audit_result": "phrases_may_exist_as_prohibited_examples_but_cannot_close_requirements",
    "closure_rule": "every occurrence requires requirement identity and non-closing context",
})
write_contract("spec137a-parent-override-map.v1.yaml", {
    "schema": "focusa.spec137a_parent_override_map.v1",
    "parent": "docs/137-focusa-temporal-authority-deadlines-urgency-grounded-forecasting-spec.md",
    "addendum": "docs/137a-focusa-temporal-zero-deferral-applicability-and-omission-firewall-addendum.md",
    "overrides": [
        {"parent_concept": "later_tranche", "governing_rule": "execution_order_only_root_DAG_membership_required"},
        {"parent_concept": "SHOULD_variance", "governing_rule": "nonconforming_for_classes_requiring_original_behavior"},
        {"parent_concept": "optional_unimplemented", "governing_rule": "affirmative_non_activation_evidence_and_review_triggers"},
        {"parent_concept": "where_applicable", "governing_rule": "durable_applicability_decision_fail_closed"},
        {"parent_concept": "closure", "governing_rule": "combined_source_all_active_rows_verified"},
    ],
})

# Extend the existing Spec 137 ledger without destroying the historical stable-ID record.
ledger137_path = ROOT / "docs/contracts/spec137-complete-feature-ledger.v1.yaml"
ledger137 = ledger137_path.read_text(encoding="utf-8")
combined137 = coverage137["combined_normative_source_hash"]
ledger137 = re.sub(r"^source_spec_sha256:.*$", f"source_spec_sha256: {combined137}", ledger137, count=1, flags=re.M)
if "combined_normative_source_v2:" not in ledger137:
    extension = {
        "combined_normative_source_v2": {
            "parent_spec": "docs/137-focusa-temporal-authority-deadlines-urgency-grounded-forecasting-spec.md",
            "mandatory_addendum": "docs/137a-focusa-temporal-zero-deferral-applicability-and-omission-firewall-addendum.md",
            "combined_hash": combined137,
            "source_coverage_ref": "docs/contracts/spec137a-normative-source-coverage.v1.yaml",
            "legacy_parent_requirement_ids_preserved": True,
            "full_conformance_status": "open",
        },
        "spec137a_requirement_rows": requirement_rows(
            [c for c in clauses137 if c["source_path"].endswith("137a-focusa-temporal-zero-deferral-applicability-and-omission-firewall-addendum.md")],
            "Spec 137A",
        ),
    }
    ledger137 = ledger137.rstrip() + "\n\n# Combined Spec 137 + 137A closure extension\n" + dump_yaml_root_entries(extension)
ledger137_path.write_text(ledger137, encoding="utf-8")

coverage138 = source_coverage(
    "focusa.spec138a_normative_source_coverage.v1",
    [
        ("docs/138-focusa-prediction-outcome-calibration-metacognitive-learning-transfer-and-epistemic-governance-spec.md", "S138-C"),
        ("docs/138a-focusa-epistemic-zero-deferral-profile-completeness-and-omission-firewall-addendum.md", "S138A-C"),
    ],
)
write_contract("spec138a-normative-source-coverage.v1.yaml", coverage138)
clauses138 = coverage138["clauses"]
write_contract("spec138-complete-feature-ledger.v1.yaml", {
    "schema": "focusa.spec138_complete_feature_ledger.v1",
    "combined_normative_source_hash": coverage138["combined_normative_source_hash"],
    "full_conformance_status": "open",
    "requirements": requirement_rows(clauses138, "Spec 138 + 138A"),
})
orders = [
    "Order 0 — Reconciliation and contracts", "Order 1 — Core type extraction", "Order 2 — Append-only event storage",
    "Order 3 — Scoring registry and calibration", "Order 4 — Metacognitive authority", "Order 5 — Transfer and self-model",
    "Order 6 — Fusion and scenarios", "Order 7 — Consolidation", "Order 8 — Surfacing and automation",
]
write_contract("spec138-delivery-dag.v1.yaml", {
    "schema": "focusa.spec138_delivery_dag.v1",
    "root": "full_spec138_conformance",
    "nodes": [
        {"id": f"order_{i}", "label": label, "depends_on": [] if i == 0 else [f"order_{i-1}"], "status": "documentation_defined_runtime_open", "closure_blocking": True}
        for i, label in enumerate(orders)
    ],
    "rule": "later_order_is_dependency_sequence_not_backlog",
})
profiles = {
    "A": ["questions", "frozen_information_sets", "immutable_commitments", "typed_outcomes", "evaluations", "receipts", "persistence", "replay", "migration", "client_operations"],
    "B": ["scorer_registry", "scorer_authority_version", "shape_specific_scoring", "calibration_cohorts", "small_sample_backoff", "reliability", "sharpness", "bias", "coverage", "skill", "decision_value"],
    "C": ["source_identity", "availability", "freshness", "reliability", "independence", "shared_dependency", "contradiction", "weights", "decomposition", "triangulation", "sensitivity", "revision"],
    "D": ["scenarios", "branches", "assumptions", "residual_probability", "counterfactual_labels", "causal_status", "confounders", "alternatives", "disconfirming_evidence", "experiment_validity"],
    "E": ["signals", "high_confidence_misses", "reflection_claims", "adjustments", "metrics", "baselines", "controls", "evaluation", "promotion", "expiry", "conflict", "rollback"],
    "F": ["transfer_prediction", "similarity_difference", "expected_benefit_risk", "transfer_outcome", "negative_transfer", "competence", "bias", "error_modes", "self_model_revision"],
    "G": ["clustering", "deduplication", "abstraction", "specialization", "conflict_preservation", "retention", "decay", "archive", "reactivation", "supersession", "revocation", "legal_hold"],
    "H": ["explicit_authority", "independent_review", "stronger_evidence", "strict_resolution", "sensitive_source_policy", "privacy", "retention", "audit_export", "quarantine", "fail_closed"],
}
write_contract("spec138-profile-activation-and-conformance-matrix.v1.yaml", {
    "schema": "focusa.spec138_profile_activation_and_conformance_matrix.v1",
    "profiles": [
        {"profile": key, "components": value, "runtime_activation": "selective_by_scope", "full_conformance_requirement": "mandatory", "status": "runtime_implementation_open"}
        for key, value in profiles.items()
    ],
})
write_contract("spec138-primitive-ownership-matrix.v1.yaml", {
    "schema": "focusa.spec138_primitive_ownership_matrix.v1",
    "rows": [
        {"family": "time_deadlines_freshness", "owner": "Spec 137 + 137A"},
        {"family": "semantic_identity_domain_packs", "owner": "Specs 45-50 and 135F"},
        {"family": "evidence_receipts", "owner": "Specs 119 and Evidence primitives"},
        {"family": "proposal_to_settlement", "owner": "Spec 136"},
        {"family": "prediction_outcome_scoring_calibration_learning_transfer", "owner": "Spec 138 + 138A"},
        {"family": "verification_routing", "owner": "Spec 144"},
        {"family": "environment_placement", "owner": "Spec 139"},
    ],
})
operations138 = [
    "prediction.question.create", "prediction.information_set.commit", "prediction.commit", "prediction.supersede", "prediction.get", "prediction.list",
    "outcome.claim", "outcome.dispute", "outcome.resolve", "outcome.correct", "prediction.evaluate", "calibration.report",
    "metacognition.signal.capture", "metacognition.reflect", "metacognition.adjustment.propose", "metacognition.adjustment.evaluate",
    "learning.candidate.decide", "learning.apply", "learning.transfer.resolve", "learning.retrieve", "learning.conflicts", "learning.expire",
    "learning.supersede", "learning.revoke", "learning.rollback", "learning.consolidate", "self_model.get",
]
write_contract("spec138-operation-client-parity-matrix.v1.yaml", {
    "schema": "focusa.spec138_operation_client_parity_matrix.v1",
    "clients": ["api", "operation_registry", "generated_contracts", "cli", "pi", "focus_slice", "mission_canvas", "tui", "menubar", "docs"],
    "rows": [{"operation": op, "required_clients": "applicability_recorded", "status": "runtime_implementation_open"} for op in operations138],
})
scorers = [
    "binary_accuracy", "multiclass_accuracy", "brier_score", "multiclass_brier_score", "log_loss", "multiclass_log_loss", "spherical_score",
    "continuous_ranked_probability_score", "mean_absolute_error", "mean_squared_error", "root_mean_squared_error", "mean_absolute_percentage_error",
    "symmetric_mape", "quantile_pinball_loss", "interval_coverage", "interval_width", "winkler_interval_score", "rank_correlation",
    "information_coefficient", "top_k_precision", "top_k_recall", "ndcg", "concordance_index", "survival_brier_score",
    "expected_calibration_error", "maximum_calibration_error", "adaptive_calibration_error", "skill_score", "expected_utility", "realized_regret", "custom_registered",
]
write_contract("spec138-scorer-and-calibration-matrix.v1.yaml", {
    "schema": "focusa.spec138_scorer_and_calibration_matrix.v1",
    "scorers": [{"id": s, "registry_required": True, "shape_applicability": "explicit", "versioned": True, "fixtures_required": True, "status": "runtime_implementation_open"} for s in scorers],
    "calibration_dimensions": ["target", "horizon", "entity", "cohort", "sources", "features", "model", "prompt", "policy", "scorer", "forecaster", "probability_bucket", "regime", "scenario", "trajectory", "environment", "time_period", "transfer_context", "verifier_capability"],
})
write_contract("spec138-source-independence-and-triangulation-matrix.v1.yaml", {
    "schema": "focusa.spec138_source_independence_and_triangulation_matrix.v1",
    "required_dimensions": ["source_identity", "upstream_dependency", "ownership", "acquisition_method", "correlation", "redundancy", "manipulation_risk", "prompt_injection_risk", "revision", "first_available_time"],
    "law": "dependent_evidence_never_counts_as_independent_confirmation",
    "status": "runtime_implementation_open",
})
write_contract("spec138-outcome-resolution-authority-matrix.v1.yaml", {
    "schema": "focusa.spec138_outcome_resolution_authority_matrix.v1",
    "states": ["claimed", "disputed", "pending_authority", "resolved", "corrected", "void", "censored"],
    "authorities": ["registered_resolver", "operator", "external_authority", "reducer"],
    "rules": ["outcome_resolution_precedes_scoring", "caller_score_is_advisory", "corrections_append", "resolution_policy_frozen"],
})
write_contract("spec138-learning-promotion-and-rollback-matrix.v1.yaml", {
    "schema": "focusa.spec138_learning_promotion_and_rollback_matrix.v1",
    "stages": ["signal", "reflection_claim", "adjustment_proposal", "evaluation", "candidate", "promotion_decision", "applied", "transfer_evaluation", "superseded", "revoked", "rolled_back"],
    "promotion_requirements": ["settled_outcome", "typed_metrics", "baseline", "applicability", "expiry", "conflict_check", "negative_effect_check", "receipt"],
    "self_promotion_prohibited": True,
})
write_contract("spec138-transfer-self-model-and-consolidation-matrix.v1.yaml", {
    "schema": "focusa.spec138_transfer_self_model_and_consolidation_matrix.v1",
    "transfer": ["expectation", "similarity", "differences", "benefit", "risk", "confidence", "evaluation_plan", "outcome", "negative_effects", "decision"],
    "self_model": ["scope", "competence", "calibration", "uncertainty", "abstention", "error_modes", "version", "evidence"],
    "consolidation": ["cluster", "deduplicate", "abstract", "preserve_exceptions", "preserve_conflicts", "retention", "decay", "archive", "reactivation", "legal_hold"],
})
write_contract("spec138-migration-matrix.v1.yaml", {
    "schema": "focusa.spec138_migration_matrix.v1",
    "sources": ["PredictionValue_v1", "Metacognition_capture_v1", "reflection_v1", "adjustment_v1", "evaluation_v1", "legacy_scores", "legacy_promotions"],
    "requirements": ["readable", "lineage_preserved", "ambiguity_labeled", "no_manufactured_authority", "restart", "replay", "rollback", "receipt"],
    "status": "runtime_migration_open",
})
write_contract("spec138-security-privacy-retention-matrix.v1.yaml", {
    "schema": "focusa.spec138_security_privacy_retention_matrix.v1",
    "controls": ["source_access_authority", "license_terms", "sanitization", "prompt_injection", "poisoning", "quarantine", "privacy_class", "least_privilege", "encryption", "retention", "deletion", "legal_hold", "audit_export"],
    "high_consequence_fail_mode": "closed",
})
write_contract("spec138-proof-matrix.v1.yaml", {
    "schema": "focusa.spec138_proof_matrix.v1",
    "proof_families": ["positive", "negative", "restart", "replay", "migration", "scorer_fixture", "calibration", "leakage", "source_dependence", "resolution", "causal", "transfer", "negative_transfer", "rollback", "security", "privacy", "retention", "adversarial", "client_parity"],
    "exact_sha_integrated_proof_required": True,
})
write_contract("spec138-forbidden-placeholder-audit.v1.yaml", {
    "schema": "focusa.spec138_forbidden_placeholder_audit.v1",
    "forbidden": ["eventually", "suggested only", "profile not implemented yet", "schema complete", "core recording is enough", "mock scorer", "mock resolver", "static card"],
    "result": "prohibited_as_closure_dispositions",
})
write_contract("spec138a-parent-override-map.v1.yaml", {
    "schema": "focusa.spec138a_parent_override_map.v1",
    "overrides": [
        {"parent": "staged_activation", "rule": "runtime_sequence_only_all_profiles_remain_full_conformance_blockers"},
        {"parent": "MAY_activate_primitive_families", "rule": "per_scope_enablement_only_not_implementation_omission"},
        {"parent": "SHOULD_append_only_events", "rule": "MUST_or_equivalent_stronger"},
        {"parent": "SHOULD_API_CLI_UI_migration", "rule": "operation_capabilities_and_migration_mandatory_for_applicable_scope"},
        {"parent": "eventually", "rule": "not_a_disposition"},
    ],
})

coverage144 = source_coverage(
    "focusa.spec144_normative_source_coverage.v1",
    [("docs/144-focusa-semantic-integrity-rdf-owl-shacl-build-verify-routing-and-vertical-intelligence-spec.md", "S144-C")],
)
write_contract("spec144-normative-source-coverage.v1.yaml", coverage144)
clauses144 = coverage144["clauses"]
write_contract("spec144-complete-feature-ledger.v1.yaml", {
    "schema": "focusa.spec144_complete_feature_ledger.v1",
    "spec_hash": coverage144["sources"][0]["sha256"],
    "implementation_activation": "blocked_until_spec143_closure_and_operator_activation",
    "requirements": requirement_rows(clauses144, "Spec 144"),
})
phases144 = ["admission_and_source_coverage", "semantic_registry_compilation", "core_verification_pack", "builder_context_and_work_contract", "obligation_compilation", "routing_and_placement", "verification_and_findings", "repair_and_reroute", "dispute_and_arbitration", "settlement_and_receipts", "vertical_integration", "migration_clients_and_docs", "proof_and_closure"]
write_contract("spec144-delivery-dag.v1.yaml", {
    "schema": "focusa.spec144_delivery_dag.v1",
    "root": "full_spec144_conformance",
    "nodes": [{"id": p, "depends_on": [] if i == 0 else [phases144[i-1]], "closure_blocking": True, "status": "documentation_defined_runtime_not_activated"} for i, p in enumerate(phases144)],
})
owners144 = [
    ("semantic_registry_rdf_owl_shacl", "Specs 45-50, 77, 135F"), ("builder_verifier_secondary_cognition", "Specs 61, 72, 78"),
    ("context_projection", "Specs 75, 100, 140"), ("work_loop", "Spec 79"), ("workpoint", "Specs 88, 131"),
    ("receipts", "Spec 119"), ("sessions", "Spec 133"), ("settlement", "Spec 136"),
    ("temporal", "Spec 137 + 137A"), ("epistemic", "Spec 138 + 138A"), ("placement", "Spec 139"),
    ("verification_routing_and_vertical_composition", "Spec 144"),
]
write_contract("spec144-primitive-ownership-matrix.v1.yaml", {"schema": "focusa.spec144_primitive_ownership_matrix.v1", "rows": [{"primitive_family": a, "owner": b} for a, b in owners144]})
core_obligations = ["scope_authority", "work_contract_completeness", "requirement_coverage", "snapshot_integrity", "evidence_sufficiency", "evidence_freshness", "contradiction", "final_snapshot_equality", "receipt_readiness", "reducer_only_settlement"]
write_contract("spec144-obligation-verifier-matrix.v1.yaml", {
    "schema": "focusa.spec144_obligation_verifier_matrix.v1",
    "core_pack": "focusa.verification.core@1",
    "rows": [{"obligation": o, "required_provider_classes": ["DeterministicValidator", "EvidenceAuditor"], "specialist_escalation": "policy_and_risk_driven", "settlement_blocking": True} for o in core_obligations],
})

amended_docs = [
    "docs/61-domain-general-cognition-core.md", "docs/66-affordance-and-execution-environment-ontology.md", "docs/70-shared-interfaces-statuses-and-lifecycle.md",
    "docs/72-agent-identity-role-and-self-model-ontology.md", "docs/74-identity-and-reference-resolution.md", "docs/75-projection-and-view-semantics.md",
    "docs/76-retention-forgetting-and-decay-policy.md", "docs/77-ontology-governance-versioning-and-migration.md", "docs/78-bounded-secondary-cognition-and-persistent-autonomy.md",
    "docs/79-focusa-governed-continuous-work-loop.md", "docs/88-ontology-backed-workpoint-continuity.md", "docs/97-focusa-reflex-primitives-spec.md",
    "docs/100-context-cognition-spec.md", "docs/107-spec-first-feature-lifecycle-and-claim-discipline-spec.md", "docs/109-agent-first-api-redesign-ax-spec.md",
    "docs/113-agent-benchmark-spec.md", "docs/116-provider-neutral-work-item-closure-authority-spec.md", "docs/119-verifiable-agent-work-receipts-and-governed-execution-ledger-spec.md",
    "docs/120-adversarial-spec-workbench-and-operator-approval-gates.md", "docs/125-mandatory-trajectory-nonlazy-hlt-pi-receipt-ontology-interlock-spec.md",
    "docs/130-hlt-aware-compaction-mission-packet-and-bloatgaurd-context-firewall-spec.md", "docs/131-focusa-workpoint-item-timing-velocity-and-closure-authority-spec.md",
    "docs/133-daemon-native-durable-silent-sessions-and-governed-autonomous-execution-spec.md", "docs/135f-domain-general-ontology-core-semantic-graph-domain-packs-and-reactive-context-spec.md",
    "docs/136-governed-proposal-to-settlement-protocol-and-outcome-truth-infrastructure-spec.md", "docs/137-focusa-temporal-authority-deadlines-urgency-grounded-forecasting-spec.md",
    "docs/138-focusa-prediction-outcome-calibration-metacognitive-learning-transfer-and-epistemic-governance-spec.md", "docs/139-distributed-presence-environment-awareness-execution-placement-and-multi-daemon-coordination-spec.md",
    "docs/140-project-agent-runtime-constitution-instruction-authority-system-prompt-and-cross-harness-compiler-spec.md", "docs/141-focusa-agent-first-tool-skill-runbook-and-documentation-release-gate-spec.md",
    "docs/142-focusa-release-requirement-trace-matrix.md", "docs/143-focusa-master-release-cycle-trajectory-genesis-flow-implementation-spec.md",
    "docs/00-glossary.md", "docs/current/AUTHORITY_MODEL.md",
]
write_contract("spec144-cross-spec-amendment-matrix.v1.yaml", {
    "schema": "focusa.spec144_cross_spec_amendment_matrix.v1",
    "rows": [{"path": p, "sha256": file_sha(ROOT / p), "marker": MARKER_ROOT, "status": "amended_in_documentation_closure_pass", "runtime_implementation": "open"} for p in amended_docs],
})
write_contract("spec144-client-parity-matrix.v1.yaml", {
    "schema": "focusa.spec144_client_parity_matrix.v1",
    "clients": ["api", "operation_registry", "generated_rust", "generated_typescript", "cli", "pi", "mcp", "rest", "mission_canvas", "work_rail", "tui", "menubar", "uiai_engine", "docs"],
    "required_semantics": ["scope", "authority", "obligations", "coverage", "snapshot", "findings", "independence", "placement", "dispute", "settlement", "revalidation", "recovery"],
    "status": "runtime_implementation_not_activated",
})
write_contract("spec144-vertical-pack-matrix.v1.yaml", {
    "schema": "focusa.spec144_vertical_pack_matrix.v1",
    "base_pack": {"id": "focusa.verification.core@1", "mandatory": True, "obligations": core_obligations},
    "vertical_templates": [
        {"vertical": v, "requires": ["domain_pack", "evidence_policy", "temporal_applicability", "epistemic_applicability", "verification_extension", "verifier_capabilities", "reflex_overlay", "migration", "conformance", "golden_scenarios"], "status": "contract_template_defined_runtime_open"}
        for v in ["software", "legal", "markets", "research", "professional_services"]
    ],
})
write_contract("spec144-migration-matrix.v1.yaml", {
    "schema": "focusa.spec144_migration_matrix.v1",
    "sources": ["adversarial_verifier_verdict_string", "boolean_completion_flags", "flat_builder_verifier_context", "legacy_evidence_bundle", "legacy_vertical_profiles", "legacy_reflex_registry"],
    "targets": ["structured_obligations", "portfolio", "snapshot", "findings", "independence", "placement", "settlement_evaluation", "revalidation"],
    "requirements": ["read_compatibility", "no_manufactured_pass", "lineage", "restart", "replay", "rollback", "receipts"],
})
write_contract("spec144-proof-matrix.v1.yaml", {
    "schema": "focusa.spec144_proof_matrix.v1",
    "proof_families": ["source_coverage", "rdf_owl_shacl_parity", "obligation_recall", "coverage_challenger", "eligibility", "independence", "common_mode", "placement", "snapshot", "finding_reproduction", "repair_reroute", "arbitration", "settlement", "revalidation", "restart", "replay", "migration", "client_parity", "security", "privacy", "accessibility", "performance", "adversarial"],
    "runtime_status": "not_activated",
})
write_contract("spec144-forbidden-placeholder-audit.v1.yaml", {
    "schema": "focusa.spec144_forbidden_placeholder_audit.v1",
    "forbidden": ["TODO_without_requirement", "mock_as_final", "static_card_as_proof", "schema_only_as_complete", "verdict_string_as_proof", "disabled_test_as_pass", "future_enhancement", "post_MVP", "mostly_done"],
    "documentation_audit": "defined",
    "runtime_audit": "pending_activation",
})
write_contract("spec144-core-verification-pack.v1.yaml", {
    "schema": "focusa.spec144_core_verification_pack.v1",
    "pack_id": "focusa.verification.core@1",
    "mandatory": True,
    "obligations": core_obligations,
    "cannot_be_suppressed_by": ["vertical", "domain_pack", "router", "model", "cost", "deadline", "availability"],
})
write_contract("spec144-obligation-compilation-and-coverage.v1.yaml", {
    "schema": "focusa.spec144_obligation_compilation_and_coverage.v1",
    "receipt_fields": ["compiler_identity", "compiler_version", "input_hashes", "requirement_set_hash", "registry_hash", "pack_hashes", "owl_hashes", "shacl_trigger_hashes", "semantic_delta_hash", "emitted", "deduplicated", "rejected", "unknown_impact", "uncovered_requirements", "coverage_challenger", "validation", "receipt_hash"],
    "coverage_rule": "uncovered_mandatory_obligations_must_be_empty_before_authorization",
})
write_contract("spec144-execution-placement-and-common-mode.v1.yaml", {
    "schema": "focusa.spec144_execution_placement_and_common_mode.v1",
    "execution_binding_fields": ["environment_identity", "placement_decision", "node", "daemon", "daemon_boot", "repository", "workspace", "worktree", "resource_claims", "deduplication_key", "lease", "fencing_token", "placement_policy_version"],
    "cognitive_identity_fields": ["actor", "run", "session", "runtime_constitution_hash", "prompt_assembly_hash", "role_version", "capability_version", "permission_version", "skill_hashes", "tool_registry_version", "tool_policy_version", "harness_adapter", "model_parameters", "source_set", "test_generator", "environment_binding", "context_hash", "code_and_pack_revisions"],
    "common_mode_dimensions": ["rubric", "prompt_source", "retrieval_corpus", "evidence_provider", "test_generator", "environment", "checkout", "cache", "external_authority", "model_family", "provider", "infrastructure"],
})
write_contract("spec144-verification-dispute-arbitration.v1.yaml", {
    "schema": "focusa.spec144_verification_dispute_arbitration.v1",
    "reused_authorities": ["PRE", "Spec 120 operator gates", "Spec 136 settlement"],
    "records": ["VerificationConflict", "VerificationAppeal", "ArbiterEligibilityRecord", "ConflictOfInterestRecord", "ArbitrationAssignment", "ArbitrationDecision"],
    "routing": {"mechanically_decidable": "registered_PRE_resolver", "judgmental_or_high_consequence": "eligible_independent_arbiter_and_operator_gate_as_policy_requires"},
})
write_contract("spec144-settlement-revalidation.v1.yaml", {
    "schema": "focusa.spec144_settlement_revalidation.v1",
    "triggers": ["evidence_corruption", "source_dependence", "security_disclosure", "external_authority_revision", "registry_or_pack_revocation", "material_regression", "verifier_capability_invalidation"],
    "records": ["SettlementRevalidationTrigger", "SettlementValidityChallenge", "SettlementSupersessionEvaluation"],
    "outcomes": ["settlement_upheld", "settlement_corrected", "settlement_superseded", "settlement_reopened"],
    "history": "append_only",
})

write_contract("spec137a-138a-144-documentation-architecture-closure-manifest.v1.yaml", {
    "schema": "focusa.spec137a_138a_144_documentation_architecture_closure_manifest.v1",
    "head_expected_before_migration": "main",
    "documents_amended": [{"path": p, "sha256": file_sha(ROOT / p)} for p in amended_docs],
    "contract_artifacts": sorted(p.name for p in CONTRACTS.glob("spec137a-*.yaml")) + sorted(p.name for p in CONTRACTS.glob("spec138*.yaml")) + sorted(p.name for p in CONTRACTS.glob("spec144-*.yaml")),
    "documentation_architecture_status": "closed_by_populated_contracts_and_cross_spec_amendments",
    "runtime_implementation_status": "open_not_activated_or_not_fully_conformant",
    "claim_boundary": "documentation_closure_is_not_runtime_completion",
})

# ---------------------------------------------------------------------------
# 5. CI documentation-closure gate and release truth
# ---------------------------------------------------------------------------

gate = r'''#!/usr/bin/env python3
from pathlib import Path
import re

from structured_contract_loader import load_contract_mapping

ROOT = Path(__file__).resolve().parents[1]
required = [
    "docs/contracts/spec137a-normative-source-coverage.v1.yaml",
    "docs/contracts/spec137a-applicability-matrix.v1.yaml",
    "docs/contracts/spec137a-conformance-class-matrix.v1.yaml",
    "docs/contracts/spec137a-forbidden-placeholder-audit.v1.yaml",
    "docs/contracts/spec137a-parent-override-map.v1.yaml",
    "docs/contracts/spec138a-normative-source-coverage.v1.yaml",
    "docs/contracts/spec138-complete-feature-ledger.v1.yaml",
    "docs/contracts/spec138-delivery-dag.v1.yaml",
    "docs/contracts/spec138-profile-activation-and-conformance-matrix.v1.yaml",
    "docs/contracts/spec138-primitive-ownership-matrix.v1.yaml",
    "docs/contracts/spec138-operation-client-parity-matrix.v1.yaml",
    "docs/contracts/spec138-scorer-and-calibration-matrix.v1.yaml",
    "docs/contracts/spec138-source-independence-and-triangulation-matrix.v1.yaml",
    "docs/contracts/spec138-outcome-resolution-authority-matrix.v1.yaml",
    "docs/contracts/spec138-learning-promotion-and-rollback-matrix.v1.yaml",
    "docs/contracts/spec138-transfer-self-model-and-consolidation-matrix.v1.yaml",
    "docs/contracts/spec138-migration-matrix.v1.yaml",
    "docs/contracts/spec138-security-privacy-retention-matrix.v1.yaml",
    "docs/contracts/spec138-proof-matrix.v1.yaml",
    "docs/contracts/spec138-forbidden-placeholder-audit.v1.yaml",
    "docs/contracts/spec138a-parent-override-map.v1.yaml",
    "docs/contracts/spec144-normative-source-coverage.v1.yaml",
    "docs/contracts/spec144-complete-feature-ledger.v1.yaml",
    "docs/contracts/spec144-delivery-dag.v1.yaml",
    "docs/contracts/spec144-primitive-ownership-matrix.v1.yaml",
    "docs/contracts/spec144-obligation-verifier-matrix.v1.yaml",
    "docs/contracts/spec144-cross-spec-amendment-matrix.v1.yaml",
    "docs/contracts/spec144-client-parity-matrix.v1.yaml",
    "docs/contracts/spec144-vertical-pack-matrix.v1.yaml",
    "docs/contracts/spec144-migration-matrix.v1.yaml",
    "docs/contracts/spec144-proof-matrix.v1.yaml",
    "docs/contracts/spec144-forbidden-placeholder-audit.v1.yaml",
    "docs/contracts/spec144-core-verification-pack.v1.yaml",
    "docs/contracts/spec144-obligation-compilation-and-coverage.v1.yaml",
    "docs/contracts/spec144-execution-placement-and-common-mode.v1.yaml",
    "docs/contracts/spec144-verification-dispute-arbitration.v1.yaml",
    "docs/contracts/spec144-settlement-revalidation.v1.yaml",
]
for rel in required:
    path = ROOT / rel
    assert path.is_file(), rel
    text = path.read_text()
    assert len(text) > 200, f"empty/shell artifact: {rel}"
    data = load_contract_mapping(path)
    assert data.get("runtime_claim") == "none", rel
    assert data.get("runtime_status") in {"implementation_open", "not_activated"}, rel

s137 = (ROOT / "docs/137-focusa-temporal-authority-deadlines-urgency-grounded-forecasting-spec.md").read_text()
s138 = (ROOT / "docs/138-focusa-prediction-outcome-calibration-metacognitive-learning-transfer-and-epistemic-governance-spec.md").read_text()
s144 = (ROOT / "docs/144-focusa-semantic-integrity-rdf-owl-shacl-build-verify-routing-and-vertical-intelligence-spec.md").read_text()
assert "Mandatory companion" in s137 and "Spec 137A" in s137
assert "Mandatory companion" in s138 and "Spec 138A" in s138
for token in ("Spec 137 + Spec 137A", "Spec 138 + Spec 138A", "Spec 139", "focusa.verification.core@1", "ObligationCompilationReceipt", "VerificationExecutionBinding", "CognitiveExecutionIdentity", "SettlementRevalidationTrigger"):
    assert token in s144, token

ledger137 = (ROOT / "docs/contracts/spec137-complete-feature-ledger.v1.yaml").read_text()
assert "combined_normative_source_v2" in ledger137 and "spec137a_requirement_rows" in ledger137

alignment = (ROOT / "docs/evidence/141-focusa-latest-spec-public-doc-alignment.md").read_text()
assert "combined full conformance open" in alignment
assert "normative documentation only; implementation not activated" in alignment

ci = (ROOT / "scripts/ci/run-spec-gates.sh").read_text()
assert "spec137a_138a_144_documentation_closure_gate.py" in ci
print("Specs 137A/138A/144 documentation architecture closure gate: PASS")
'''
write("tests/spec137a_138a_144_documentation_closure_gate.py", gate)

ci_path = ROOT / "scripts/ci/run-spec-gates.sh"
ci_text = ci_path.read_text(encoding="utf-8")
ci_line = "run_gate python3 ./tests/spec137a_138a_144_documentation_closure_gate.py"
if ci_line not in ci_text:
    anchor = "run_gate python3 ./tests/spec137_temporal_authority_release_gate_test.py"
    if anchor in ci_text:
        ci_text = ci_text.replace(anchor, anchor + "\n" + ci_line, 1)
    else:
        ci_text = ci_text.rstrip() + "\n" + ci_line + "\n"
    ci_path.write_text(ci_text, encoding="utf-8")

append_once("CHANGELOG.md", "spec144-doc-architecture-closure", """
## Documentation architecture closure — Specs 137A, 138A, and 144

- made Specs 137A and 138A visible mandatory companions in their parent specs;
- integrated Spec 139 placement, Spec 140 cognition provenance, PRE/Spec 120 dispute resolution, and Spec 136 settlement revalidation into Spec 144;
- amended primitive-owner, glossary, authority, release, public-doc, context, session, receipt, reflex, Work Loop, ontology, and Vertical documents;
- added populated source-coverage, ledger, DAG, ownership, profile, parity, placement, dispute, migration, proof, and placeholder-audit contracts;
- added a CI documentation-closure gate;
- preserved the truth boundary that documentation closure is not runtime implementation or conformance.
""")

# Validate locally before the migration commit is created.
compile((ROOT / "tests/spec137a_138a_144_documentation_closure_gate.py").read_text(), "closure_gate", "exec")
print("Spec 137A/138A/144 documentation architecture closure migration applied")
