# Spec 176 — Focusa Closure Proof Capsule, Verdict Authority, and Adversarial Judge Architecture

## Status

- **Draft v1.1 — detailed — 2026-08-22**
- Authority: operator decisions D1–D5 ( §3 ) — all approved
- Supersedes: none. Extends Spec 131 (Closure Authority), Spec 136 (settlement), Spec 138/138A (typed authority events), Spec 149 (workset ledger), Spec 155 (CallGraph), Spec 165 (completion envelopes), docs/175 (licensing lifecycle)
- Origin: 30 false `bd close` events without evidence ( #317 / #318 ); operator directive: *never allow fake closure or unverified done EVER*
- Depends on: Focusa daemon ledger, workset/CallGraph settlement, completion_authority evaluator, guard shim, Workstream/Cockpit contracts (UIAI-COCKPIT-000, UIAI-COCKPIT-005), UIAI Session API, Browser Diagnostics, FPV PWA

---

## 1. Problem

Closure in Focusa was a writable string. Agents marked items done; nothing physical required proof. The existing lifecycle (Prepare→Validate→Authorize→Submit→Reconcile), completion coverage evaluator, and guard shims were opt-in and bypassable. Result: false closures persisted undetected and releases inherited unverified work.

The fix is not more reminders. It is an architecture where:

1. `closed` is unrepresentable without a settled, evidence-bound claim.
2. Proof is produced and delivered automatically at every completion boundary.
3. Verification is continuous but cheap — settled work is never wastefully re-verified.
4. Claims are adversarially challenged by an independent Judge so trust is structural, not self-certified.
5. The system flows — verification depth scales with risk instead of gating everything uniformly.
6. Every ecosystem surface (UIAI Engine, Cockpit, FPV PWA, release checklist) speaks the same proof language.

---

## 2. Core laws (normative)

- **L1 Unrepresentability.** No work item store may hold a closed/settled state without a reference to a closure claim in `reconciled` state in the append-only ledger. Provider projections (beads, GitHub issues) are renderings of the ledger, never authority.
- **L2 No self-certification.** The hash a verdict binds to is produced by the verifier's own execution of the proof script, never by the claimant's capture. Claimant captures are hints for human review only. *(D1)*
- **L3 Proof in the envelope.** Every completion boundary — task end, workset settlement, CallGraph join, pre-build — carries a proof reference in its completion envelope. A completion without a proof reference is a malformed envelope and fails validation. *(D2, two-phase, §6)*
- **L4 Settled is settled.** A verdict is `f(canonical_hash, verifier_version, context_digest)`. While all three are unchanged, the verdict is valid forever and MUST NOT be re-executed. Sweeps are integrity checks (hash/context comparison), never re-runs.
- **L5 Append-only truth.** Disputes, revocations, and re-proofs write new ledger revisions. Settled claims are never mutated or deleted. History is never rewritten.
- **L6 Honest tiers.** Machine-verifiable guarantees apply only to computational evidence. Attestation evidence carries a disclosed, weaker trust-anchor model. The system never presents uniform trust across non-uniform evidence. *(D3)*
- **L7 Atom authority.** Acceptance atoms bind at Prepare from an authority source (spec, bead, operator scope). The claimant never defines the atoms it will be measured against. Scope drift between Prepare-atoms and Submit-evidence is a flagged event. *(D4)*

---

## 3. Operator-approved decisions

- **D1** Verifier-side re-execution is the only trusted hash source. Compute cost accepted.
- **D2** Two-phase completion envelope: `proof_pending` / `proof_settled`.
- **D3** v1 scope: computational evidence only (diff/capture/transcript/delta). Attestation tier experimental with disclosed weaker guarantees.
- **D4** Atom authority binds at Prepare from spec/bead/operator scope, never from claimant.
- **D5** Adversarial Judge model is a first-class verification actor.

---

## 4. Evidence elements — vertical-agnostic model

Closure proof is a set of typed **evidence elements**. The capsule shell is identical for all verticals; renderers and verifiers are selected by artifact type. New vertical = new renderer plugin; capsule shell unchanged.

### 4.1 Delta kinds

| Kind | Applies to | Canonical examples |
|---|---|---|
| `diff` | text/data artifacts | code hunks, contract redlines, article edits, config changes, SQL migrations |
| `capture` | visual/rendered surfaces | UIAI screenshots, frame captures, publish-URL captures, FPV mirror frames |
| `transcript` | executed interactions | `go test` / `cargo test` runs, API probes, CLI sessions, browser eval traces |
| `delta` | state before/after | DB row counts, checksums, health posture, quota, file manifests |
| `attestation` | physical/external world *(experimental, L6)* | photos, signatures, receipts, notarized scans |

### 4.2 Renderer registry

Keyed by artifact MIME/type, not by domain. Registry entry: `{ artifact_type, delta_kind, renderer_component, verifier_id, canonical_fn_version }`. Renderers are display-only and carry no trust authority. Adding support for a new document type (e.g., Figma frame, legal redline PDF) = register a renderer + verifier, no capsule change.

### 4.3 Canonical content functions (closes G4: hash instability)

Verdicts never bind to raw bytes. Each delta kind defines a canonicalization that is versioned as part of the verifier:

- **capture** — hash of `(DOM-assertion results + diagnostics verdict + viewport/env profile)`, not the PNG. Pixels are the human layer; assertions are the machine gate. Aligns with Cockpit 005 law: *"Agents SHALL verify Cockpit state semantically rather than relying on screenshots."*
- **transcript** — normalized output (durations, timestamps, absolute paths, ANSI, ordering noise stripped; TAP/JUnit parsed to structured result).
- **diff** — patch content over declared scope, whitespace-normalized, path-canonicalized.
- **delta** — canonical JSON of measured state tuple (sorted keys, stable float formatting).
- **attestation** — file hash + trust-anchor chain; no canonicalization claimed; badge renders weaker guarantee ( §14 ).

Without this, L4 memoization never fires and sweeps false-alarm. Canonical function version is part of `verifier_version` in the verdict binding.

---

## 5. Claim anatomy (normative schema)

```json
{
  "schema": "focusa.closure_claim.v1",
  "claim_id": "clm_01H…",
  "item_ref": "bd:focusa-xyz",
  "workpoint_id": "wp_…",
  "workstream_id": "ws_…",
  "continuity_id": "ctx_…",
  "created_at": "2026-08-22T…Z",
  "atoms": [
    {
      "atom_id": "atom:xyz:AC-3",
      "source": "spec:176#AC-3",
      "bound_at": "prepare",
      "bound_by": "spec:176",
      "description": "Login renders 1280×800 with no console errors"
    }
  ],
  "elements": [
    {
      "element_id": "el_01H…",
      "delta_kind": "capture",
      "artifact_type": "text/html+ui",
      "artifact_ref": "uiai-screenshot:sha256:abc123",
      "proof_script": "proofs/xyz.proof.json",
      "proof_script_hash": "sha256:…",
      "canonical_hash": "sha256:…",
      "context_digest": "sha256:…",
      "captured_by": "claimant:agent-session-…",
      "verified_by": "verifier:uiai-eval-v3",
      "verifier_version": "uiai-eval:3.2.1+canonical:1.0"
    }
  ],
  "verdicts": [],
  "state": "prepared",
  "proof_envelope": { "phase": "proof_pending", "pending_refs": ["uiai-session:…"] }
}
```

- `context_digest` — `sha256` over the closure scope (claimed files/atoms + their dependency surface snapshot at claim time). Enables free applicability checks ( §10 ).
- `proof_script_hash` — pins the exact script the verifier executed; script swap invalidates binding just like artifact swap.
- Claim lifecycle extends Spec 131's five stages with an explicit **Judged** stage ( §8 ): `prepared → validated → judged → authorized → submitted → reconciled → (disputed → re-prepared)`.

State transition table:

| From | To | Guard |
|---|---|---|
| `prepared` | `validated` | atoms bound from authority source (L7); coverage gate evaluated |
| `validated` | `judged` | T0: auto-endorse path; T1/T2: Judge challenge required ( §11 ) |
| `judged` | `authorized` | Judge endorse OR human override of challenge with cited reason |
| `authorized` | `submitted` | capsule verdicts attached; envelope carries proof ref (L3) |
| `submitted` | `reconciled` | ledger append accepted; sweeper confirms binding |
| any settled | `disputed` | signed dispute event with reason; propagates `suspect` downstream ( §16 ) |

---

## 6. Two-phase completion envelope (D2)

Evidence is often async (CI, deploy health, UIAI runs). The envelope never forces a choice between blocking and shallow proof.

- **Phase 1 — `proof_pending`** — completion envelope carries proof intent: script refs, pending run ids, declared elements. Task may be marked *task-complete*. Renders amber. Never rolls up as verified. Build-gate eligible (coverage), not release-gate eligible.
- **Phase 2 — `proof_settled`** — async evidence lands, verifiers execute, verdicts attach, claim reconciles. Item becomes *closure-settled*. Renders green. Only settled items count as complete in rollups (Spec 131 AC-33), workset settlement, CallGraph join, and pre-build release gate.

Gate granularity (closes G3 circularity):

- **Build gate** — requires proof *coverage* (every constituent item pending-or-settled with declared atoms); build produces candidate evidence for pending claims — no deadlock.
- **Release gate** — requires full settlement; no pending items allowed to roll up.

Wire format (envelope fragment):

```json
{ "completion_envelope": { "phase": "proof_pending", "proof_refs": ["proof_script:sha256:…"], "pending_runs": ["uiai-session:…","ci-run:…"] } }
{ "completion_envelope": { "phase": "proof_settled", "proof_refs": ["uiai.closure_proof_packet.v1:sha256:…"], "claim_refs": ["clm_…"] } }
```

---

## 7. Verdict protocol

Each evidence element carries per-element Accept / Reject / Request-change + note in the capsule.

1. **Hash-bound.** A verdict binds to `(canonical_hash, proof_script_hash, verifier_version, context_digest)`. Artifact swap, script swap, or context drift → binding mismatch → element returns to unsettled. Reviewing one thing and shipping another is impossible.
2. **Signed authority events.** Verdicts are Spec 138 typed authority events: keypair-signed by the issuer (operator, orchestrator, Judge), hash-chained into the ledger, anchored outside writer-controlled storage. A DB row with an actor string is not a verdict.
3. **Rejection is evidence.** Reject routes the claim to revision or to a non-factual disposition (`accepted-risk` / `cancellation` / `abandonment` per Spec 131 AC-33), recorded in the ledger, never deleted. Non-factual dispositions render with distinct color and do not satisfy verified-complete rollups.
4. **Uncited orchestrator verdicts rejected.** Any verdict issued by an agent/orchestrator must cite the verifier outputs relied upon; uncited verdicts fail validation (anti-rubber-stamp).

Verdict wire shape:

```json
{
  "verdict_id": "vrd_…",
  "element_id": "el_…",
  "decision": "accept|reject|request_change",
  "bound_hash": "sha256:…",
  "bound_verifier_version": "uiai-eval:3.2.1+canonical:1.0",
  "bound_context_digest": "sha256:…",
  "cited_verifier_outputs": ["uiai-eval:…:sha256:…"],
  "signed_by": "operator:verious|judge:gpt-5-v7|orchestrator:session-…",
  "sig": "ed25519:…",
  "note": "optional bounded reason"
}
```

---

## 8. The Judge — adversarial verification actor (D5)

### 8.1 Role

An independent model instance that challenges claims adversarially: do elements actually prove atoms? Are atoms trivial or gamed? Is the diff scoped to the claim? Does the transcript show what it asserts? What would make this claim false? The Judge is the structural answer to self-certification (G1) and to the human bottleneck (G11).

### 8.2 Independence rules

- Executes as a **different provider/model family** than the claimant where available — no correlated blind spots.
- **Blind review where feasible** — Judge receives atoms + elements + verifier outputs, not the claimant's narrative or self-assessment.
- Own identity is a signed authority event; model/version recorded on every challenge; Judge outputs are hash-bound and ledger-appended.
- Judge calibration uses existing `focusa_predict_*` / metacog surfaces: endorsements later falsified → Judge trust weight decays ( §8.5 ).

### 8.3 Powers — deliberately asymmetric

Judge may **endorse**, **challenge** (with cited reasons and missing-evidence enumeration), or **escalate-human**. Judge may **never settle**. Settlement authority stays with tier rules ( §11 ). A challenged claim returns to producer for re-proof; challenge and resolution are both ledger evidence.

```json
{ "judge_verdict": "challenge", "reasons": ["atom AC-3 trivial: 'page loads' under-proves 'no console errors'"], "missing": ["uiai-diagnostics:sha256:… with zero errors"], "cited": ["el_…:sha256:…"] }
```

### 8.4 Timeout law (normative)

A Judge challenge exceeding its SLA **escalates** (to T2 human tier) or holds pending. It **never auto-passes**. Timeout auto-approval is a fake-closure valve and is forbidden. SLA and escalation target are part of the claim's Judge assignment.

### 8.5 Judge calibration (ties to existing Focusa signals)

Judge endorsements later falsified (by dispute, sweep drift, or incident) feed `focusa_predict_record` / metacog calibration. Low-trust Judge endorsements auto-escalate to T2. Sustained accuracy promotes budget efficiency (more T0 sampling, less T1 blocking) without weakening guarantees.

---

## 9. Verifier supply chain

- Verifiers and their canonical content functions are versioned, signed, append-only authority events.
- **Revocation** — a verifier found buggy emits a revocation event → all claims whose verdicts cite that verifier version transition to `needs_reverification` → migration sweep re-executes with the successor version.
- Verifier install/upgrade on a host is a governed change (Spec 140-class authority), never silent. Verifier binaries are pinned by hash in the install manifest; `focusa work-item closure audit` verifies pin consistency.

---

## 10. Continuous verification without re-verification (L4 in practice)

The sweeper is the guarantee that no lie persists, at integrity-check cost:

```
for each settled claim:
  recompute canonical_hash availability (store HEAD check)
  recompute context_digest over live dependency surface
  if (stored canonical_hash == live canonical_hash
      && stored context_digest == live context_digest
      && verifier_version not revoked):
        append continuity confirmation: "proved at T1, continuity confirmed at T2…Tn"
  else:
        mark element stale → re-execute proof script → new verifier run
        if mismatch persists → auto-reopen + incident + operator notice
for each provider item closed without settled ledger claim:
  auto-reopen + incident
```

- O(hash-compare), never O(test-run), while bindings hold.
- Trust compounds on the ledger as a visible continuity chain.
- Provider scan retroactively defends against all historical bypass — the mechanism that would have caught the 30.

Sweeper cadence: daemon periodic (default 5m) plus on every provider close event and pre-build.

---

## 11. Trust tiers — flow without bottlenecks

Verification depth scales with risk. Routine work flows at verifier speed; scrutiny concentrates where blast radius lives.

| Tier | Applies to | Settlement path | Judge | Human key? |
|---|---|---|---|---|
| **T0 auto** | routine items, low blast radius, clean actor history | verifier pass → settle immediately | retrospective sample (default ≥10%, risk-weighted) | no |
| **T1 judged** | shared-infra touches, prior-dispute items, new surfaces, elevated risk score | Judge challenge → endorse → settle | synchronous, SLA-bounded | no |
| **T2 human** | release-gate, P0, external/financial commitment, Judge escalation, low-trust Judge output | capsule review → signed operator verdict | optional pre-brief | **required** |

- **Tier assignment** is a risk score from existing trajectory/metacog/prediction signals: item class, actor history, surface sensitivity, dispute history, file blast radius (shared infra = auto-T1).
- **Tier movement is dynamic:** T0 sampled→challenged→promotes to T1; T1 actor with sustained clean record→demotes to T0 on next claim.
- **Batch review UX:** capsule supports queue-based multi-item verdict for humans and orchestrators, per-element expand, filter by tier/risk. No rubber-stamp path exists.

Scoring inputs (normative, minimum): `{ item_class, actor_history_score, surface_sensitivity, dispute_history, file_blast_radius, verifier_confidence }` → tier threshold table in code, tuned via metacog.

---

## 12. Storage and availability

- Proof blobs (screenshots, logs, captures, diagnostics dumps) live in a **content-addressed store** keyed by canonical hash, replicated dual-server (KH↔OVH pattern), never committed to git (disk-headroom and repo-hygiene law). GC is hash-referenced; blobs unreferenced by any claim revision are eligible after retention window.
- Git carries manifest references only (`proofs/<claim_id>.manifest.json` with hashes + store refs + proof_script ref). PR integration: close-time hook posts capsule link + key frames as PR comment.
- PWA capsule verifies artifact hashes client-side on load; served offline-capable; tamper-evident by construction. Bundle is pinned by manifest hash.

Manifest shape:

```json
{ "manifest_version": 1, "claim_id": "clm_…", "store": "content-addressed:sha256:…", "artifacts": [{ "element_id":"el_…", "canonical_hash":"sha256:…", "blob_ref":"cas:sha256:…" }], "proof_script": "proofs/xyz.proof.json", "proof_script_hash":"sha256:…" }
```

---

## 13. UIAI as the capture camera — constraints and proof script

- UIAI pool is max 2 pages, serialized. Visual proof is **tiered by risk**, not universal. UI-affecting items + sampled audits; routine items emit cheap evidence (verifier re-run, diff hash). This is why T0 exists.
- Deterministic viewport profile (default `1280×800`, device scale, font set) recorded in env profile; capture canonicalization per §4.3 — never pixel-diff gates.
- WebAuthn/passkey flows can wedge pages (`uiai-engine#66`): proof scripts use test-mode auth or avoid those flows; script declares required auth mode.
- The claimant's UIAI capture is a hint; the trusted canonical hash comes from verifier re-execution of the proof script (L2).

Proof script (verifier-executable, deterministic):

```json
{
  "proof_kind": "uiai-visual",
  "viewport": { "w": 1280, "h": 800 },
  "steps": [
    { "op": "open", "url": "https://…", "assert": "nav.ok" },
    { "op": "snapshot", "ref": "@e3", "assert": "visible" },
    { "op": "screenshot", "ref": "@e3", "evidence_ref": "uiai-screenshot:sha256:…" },
    { "op": "diagnostics", "assert": "console.errors == 0" }
  ],
  "assertions": ["dom:hero.visible", "diagnostics:errors==0"],
  "expected_canonical_hash": "sha256:…"
}
```

Other proof kinds (`transcript`, `diff`, `delta`) have analogous script shapes; the verifier knows how to execute and canonicalize each.

---

## 14. Attestation tier (experimental, D3)

Physical/legal-world evidence (photos, signatures, receipts) has no re-executable verifier. Trust anchors instead: counterparty signature chains, timestamping authority, notarization. The capsule renders these with a **disclosed weaker guarantee badge** and they alone can never settle a T1/T2 item in v1. Attestation + computational corroboration (e.g., signed receipt + ledger delta) can satisfy T1 when policy explicitly allows the combination.

---

## 15. Authority unification — one sentence each

- **Lifecycle (Spec 131):** the closure state machine — Prepare→Validate→Judge→Authorize→Submit→Reconcile.
- **Completion authority (`completion_authority.rs`):** the coverage gate inside Validate — atoms × verified elements, deterministic.
- **Workset ledger (Spec 149):** the settlement record — append-only, the only system of record.
- **Proof capsule (this spec):** the Authorize surface — human/orchestrator verdict UI over hash-bound elements.
- **Judge (this spec):** the adversarial challenger inside the Judge stage — endorse/challenge/escalate, never settles.
- **Provider stores (beads/GitHub):** projections of the ledger. Closing there without ledger settlement is reverted by sweep.

No parallel closure systems. Any future closure surface must declare which role it binds into.

---

## 16. Dispute cascade (normative)

Settlement is a DAG: items settle on the strength of dependencies' settlements.

- Dispute of X writes a new ledger revision marking X `disputed` (L5) and propagates `suspect` to all downstream dependents (transitive closure over declared dependencies + file-surface overlap).
- Dependents cannot settle while a dependency is disputed/suspect; sweeper enforces at settlement time and at next sweep.
- Resolution paths: X re-proved (dependents clear), X re-scoped (dependents re-bind to new atoms), X cancelled with non-factual disposition (dependents' dependent work gets matching disposition, honestly recorded). No silent invalidation.

---

## 17. Gap traceability (design review 2026-08-22, all 14 closed)

| # | Gap | Resolution section |
|---|---|---|
| G1 | self-certification (claimant owns camera) | L2 + verifier-executed hashing + Judge ( §8 ) |
| G2 | sync proof vs async evidence | two-phase envelope ( §6 ) |
| G3 | pre-build gate circularity | coverage vs settlement gate split ( §6 ) |
| G4 | hash instability | canonical content functions ( §4.3 ) |
| G5 | context rot vs settled-forever | context_digest in verdict binding ( §5, §10 ) |
| G6 | verifier supply chain | §9 revocation + pinned manifests |
| G7 | verdict identity / non-repudiation | Spec 138 signed events ( §7.2 ) |
| G8 | dispute cascade | §16 DAG + suspect propagation |
| G9 | vertical overreach | L6 + §14 experimental tier |
| G10 | atom authority | L7 ( §5 ) + transition guard |
| G11 | human verdict scalability | trust tiers + batch UX ( §11 ) |
| G12 | proof storage durability | §12 CAS + replication |
| G13 | UIAI throughput ceiling | §13 tiered capture |
| G14 | authority drift across subsystems | §15 unification map |

---

## 18. Acceptance criteria (executable)

1. No provider item can reach closed without a ledger claim in `reconciled`; sweep proves it by construction — `focusa work-item closure audit` reports zero drift on a clean tree.
2. Every completion envelope (task/workset/join/pre-build) carries `proof_pending` or `proof_settled`; missing = malformed envelope error (unit test: envelope without proof ref rejected).
3. Verdicts bind `(canonical_hash, proof_script_hash, verifier_version, context_digest)`; artifact or script swap invalidates; context drift marks stale without re-execution.
4. Settled claims are never re-executed while bindings hold — memoization test: sweep of N settled claims performs zero verifier re-runs (counter-asserted).
5. Judge cannot settle; challenge SLA timeout escalates or holds, never auto-passes (fault-injection test: stalled Judge → T2 escalation, not settlement).
6. Verifier revocation transitions all citing claims to `needs_reverification` and migration sweep re-settles them (revocation fixture test).
7. T0/T1/T2 policy enforced; uncited orchestrator verdict rejected; attestation-only evidence cannot settle T1/T2.
8. Provider close without claim is auto-reopened with incident within one sweep interval (integration test: `bd close --force` without claim → reopen).
9. Capsule renders all four v1 delta kinds from one shell; hash verification client-side; offline load of a pinned bundle (PWA smoke: offline + hash-mismatch badge).
10. Atoms bound at Prepare from authority source; claimant-authored atoms rejected at Validate (negative test).
11. `proof_pending` items visible as amber, not counted in `verified_complete` rollups; `proof_settled` promotion is observable in ledger and capsule.
12. `bd close` shim and direct provider close both route through ledger proxy — bypass physically impossible (shim + server-side guard test).

---

## 19. Implementation order (normative)

1. **Sweeper + provider reconciliation** — retroactive defense; catches all historical lies
2. **Coverage gate wired into Validate** — `completion_authority` join + atom-authority guard
3. **Closure audit command** — `focusa work-item closure audit` (provider state vs ledger — replaces commit-message heuristics in CI + pre-push)
4. **Two-phase envelope + proof manifest** — `proof_pending`/`proof_settled`, CAS store refs, proof_script pinning
5. **Canonical content functions v1** — `transcript` + `diff` first, `capture` second ( §4.3 )
6. **Judge stage** — T1 routing, blind challenge protocol, calibration hooks, SLA escalation
7. **Capsule PWA** — renderer registry, verdict UI, client-side hash verify, PR comment hook, offline bundle
8. **Trust-tier policy engine** — risk scoring from metacog/prediction signals, threshold table
9. **Store proxy enforcement** — `bd`/provider mutation proxy — makes bypass physically impossible
10. **Attestation tier experimental** — badges, trust-anchor model, combination policy
11. **Ecosystem wiring** — packet `closure` mode, Cockpit workspace surface, FPV push, checklist consumption ( §21 )

---

## 20. Non-goals (v1)

- Pixel-perfect visual regression gates (assertions, not pixels)
- Attestation-tier settlement for consequential items
- Replacing provider task systems (they remain projections)
- Real-time Judge challenge on T0 items (sampling preserves flow)
- Cross-vertical renderer completeness (registry is intentionally incremental)

---

## 21. Ecosystem integration — UIAI Engine, Cockpit, FPV PWA (cross-referenced 2026-08-22)

Spec 176 must not fork existing UIAI/Focusa evidence surfaces. Integrations bind to what already ships.

### 21.1 Packet schema — extend, never fork

UIAI already ships `uiai.focusa_research_diagnostics_packet.v1` ( `docs/FOCUSA_PACKET_EXAMPLES_GALLERY.md` ) with modes `research|diagnose|proof`, sha256 evidence refs (`uiai-screenshot:sha256:`, `uiai-share:`, `uiai-browser:session=`), `recommended_focusa` tool routing (`focusa_evidence_capture`, `focusa_browser_diagnostics_intake`), redaction rules, and CI validators (`check-focusa-packet-examples.py`, `check-focusa-packet-drift.sh`, `smoke-focusa-packet-ci.sh`).

- Spec 176 adds a **`closure` packet mode** to this family: `uiai.focusa_closure_proof_packet.v1` carrying `claim_id`, `atoms[]`, `elements[]` (with canonical_hash + context_digest + proof_script_hash), `verifier_version`, verdict slots, and envelope phase.
- Existing validators gain closure-mode fixtures; drift checks gain closure routes; redaction rules apply unchanged (no raw image payloads, no secrets, bounded summaries + refs).
- Evidence refs from UIAI captures (screenshot/share/session/diagnostics) are legal element artifacts in a closure claim — already hash-prefixed and bounded.

Wire shape (closure packet, extends existing packet envelope):

```json
{
  "schema": "uiai.focusa_closure_proof_packet.v1",
  "mode": "closure",
  "claim_id": "clm_…",
  "workpoint_id": "wp_…",
  "workstream_id": "ws_…",
  "atoms": [{ "atom_id": "…", "source": "spec:176#AC-3" }],
  "elements": [{ "element_id":"el_…", "delta_kind":"capture", "artifact_ref":"uiai-screenshot:sha256:…", "canonical_hash":"sha256:…", "context_digest":"sha256:…", "proof_script_hash":"sha256:…" }],
  "phase": "proof_pending|proof_settled",
  "recommended_focusa": { "preferred_tool": "focusa_evidence_capture", "args_preview": { "claim_id":"clm_…", "evidence_ref":"uiai.focusa_closure_proof_packet.v1:sha256:…" } }
}
```

### 21.2 Diagnostics are evidence, not authority (alignment confirmed)

`BROWSER_DIAGNOSTICS_SPEC.md` §11 already states: *"UIAI diagnostics are evidence, not authority. Focusa should ingest bounded diagnostic snapshots through its existing evidence/prediction/Workpoint flow."* This is exactly L2: the claimant's capture (including diagnostics) is a hint; the trusted canonical hash comes from verifier re-execution of the proof script ( §4.3, §13 ). No new diagnostics surface is needed; diagnostics packets become `delta`/`transcript` elements inside a closure claim where required.

### 21.3 Session API binding

The UIAI Session API (`SESSION_API.md`: open/read/snapshot/screenshot/eval/diagnostics) is the execution substrate for `capture`-kind proof scripts ( §13 ). Proof scripts are not a new browser automation system; they are a deterministic, verifier-replayable composition of existing session operations plus assertions. Session-scoped proof runs respect the existing 2-page pool limit, which is why visual capture is tiered ( §13 ) rather than universal.

### 21.4 Cockpit 005 contracts — reuse the envelopes

`UIAI_COCKPIT_005` (Workstream-Scoped Universal Agent Control) already defines: `FocusaWorkstreamContext` (workstream_id, workpoint_id, authority_ref, verified_at), command envelopes (actor identity, idempotency key, preview/commit), result envelopes (receipt ref, Evidence proposals, typed warnings/recovery), and the `evidence.snapshot.capture` action. Its closure law requires *"immutable Evidence/receipts"* and a *"proof-and-continuation path."*

- Closure proof elements produced inside Cockpit workspaces ride the **existing result envelope** (`Evidence proposals` + `receipt ref`), not a new side channel.
- The closure capsule registers as a Cockpit work object in the existing shell (`Studio · Automations · Evidence · Activity` IA) — verdict UI is a workspace surface, not a separate app. It reuses Cockpit's scoped Work Surfaces, tabs/panes, and per-surface Attachment semantics (UIAI-COCKPIT-004).
- Cockpit's law *"Agents SHALL verify Cockpit state semantically rather than relying on screenshots"* is the cockpit-side statement of §4.3 canonicalization (assertions over pixels). Cited, not duplicated.
- Cockpit 005 §2 blocks cross-product implementation on Focusa Spec 158 / focusa#125 invariants — Spec 176's ledger/settlement claims must be checked against those invariants before cross-product rollout.

### 21.5 Contract ledger alignment

`UIAI_COCKPIT_002_C01_AGENT_FIRST_BROWSER_CONTRACT_LEDGER_v1.yaml` already mandates: `evidence_grade` tracking, *"Use UIAI Engine Eval for all browser proof"* (engine eval is the natural host for verifier re-execution of capture-kind elements), and exposing *"verification … and capsule state through progressive disclosure."* The closure capsule consumes that capsule-state surface rather than defining a second one.

### 21.6 FPV PWA — stack and notification reuse

The proof capsule PWA ( §12 ) reuses `UIAI_AGENT_FPV_PWA_SPEC` conventions: PWA shell, operator auth, token-scoped link pattern with redaction, connection-state handling, offline bundle, and **push notifications** — verdict requests (T2 human key), Judge escalations, and sweep incidents push to the operator's phone. URL structure follows the existing FPV token-scoped link pattern; no new auth surface is introduced.

### 21.7 Release proof checklist

`AGENT_SURFACE_RELEASE_PROOF_CHECKLIST.md` is the existing release-proof culture surface. Release gates consume `proof_settled` closure packets as checklist evidence instead of bespoke proof steps. The checklist's `smoke-focusa-packet-ci.sh` / `check-docs-completeness.py` hooks gain closure-packet coverage once §21.1 validators land. A release cannot be marked proven while constituent claims are `proof_pending`.

### 21.8 Entitlement surfaces

UIAI entitlement/licensing docs (`UIAI_FOCUSA_ENTITLEMENT_INTEGRATION_ADDENDUM`, `LICENSE_ENTITLEMENT_AND_ONBOARDING_ENFORCEMENT_SPEC`) govern who may execute proof capture on metered engines; closure proof runs consume the same entitlement path — no bypass lane for verification traffic. Metering applies to verifier re-executions as well.

---

## 22. Security considerations

- Verdict signing keys are per-actor; operator keys are hardware-backed where available; agent session keys are ephemeral and scoped to the session. Key rotation emits a ledger event; old verdicts remain valid (hash-bound) but new verdicts under rotated keys carry the new key id.
- Store CAS is authenticated; unauthenticated blob fetch is denied. PR-comment capsule links are capability URLs with redacted tokens (FPV pattern).
- `proof_pending` can never satisfy a release gate — this is enforced at the ledger, not just the UI. UI-only enforcement would be a bypass.

---

## 23. Appendix: end-to-end example

1. Agent prepares claim for `bd:focusa-xyz` — atoms bound from `spec:176#AC-3`, T1 scored (touches shared infra).
2. Agent lands code + proof script; envelope enters `proof_pending` (task-complete, amber).
3. CI + UIAI verifier runs execute script; canonical hashes produced; packet promoted to `proof_settled`.
4. Judge receives blind atoms+elements+verifier outputs; endorses (or challenges → agent revises).
5. Capsule renders all elements; operator batch-verdict not required for T1, but audit trail shows Judge endorsement + verifier outputs.
6. Ledger reconciles; workset settles; release gate sees `proof_settled` and proceeds. Sweeper confirms continuity every 5m thereafter.

