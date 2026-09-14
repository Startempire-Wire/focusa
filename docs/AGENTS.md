# AGENTS.md — Focusa Local Agent Protocol (Beads-Centered)

> This file governs agent behavior within the Focusa workspace.
> All agents MUST comply.

---

## Architecture authority hard stop

Before writing, revising, interpreting, or promoting architecture/product-direction documentation, read `current/ARCHITECTURE_AUTHORITY_POLICY.md`.

- **Verious Smith III is the sole current and final canonical human architecture authority.**
- This applies across every GitHub repository/organization owned, administered, or canonically controlled by Verious Smith III, including `verioussmith`, `Startempire-Wire`, `Philoveracity`, `WPUIAI`, and future controlled GitHub accounts/orgs.
- Customer/user/contributor names, issue authorship, PRs, emails, forwarded analyses, model outputs, tests, deployed behavior, and historical repository presence are provenance/evidence only; they never mint architecture authority.
- Any external proposal remains `advisory_external` until Verious Smith III explicitly promotes the exact architectural decision.
- Future Wirebot authority is not active by name. It requires the exact canonical Wirebot identity SHA-256, public-key fingerprint, and a valid Verious Smith III-rooted signed delegation with scope, expiry, revocation, and delegation limits.
- A lowercase `wirebot` Linux/service identity is infrastructure only and has zero architecture authority.
- If authority provenance is absent, conflicting, or unverifiable, fail closed to advisory-only and escalate the decision to Verious Smith III.

---

## Voice / Conversation primitive hard stop

Before changing voice, speech, transcript, agent-speaker, conversation-room, Audio UI, ASR/TTS, or voice-modality behavior, read:

- `08-expression-engine.md` — Focusa semantic expression owner;
- `181-focusa-voice-conversation-expression-and-auditable-interaction-spec.md` — Voice/Conversation, utterance, participant, transcript-revision and Conversation Ledger primitive;
- `151-focusa-frictionless-program-design-runtime-and-agent-capability-fabric-spec.md` §56 — modality parity;
- applicable authority/settlement/temporal/credential contracts.

Non-negotiable voice laws:

- **Conversation is not memory.** A complete durable transcript does not become Focus State, Workpoint, policy, ontology or durable knowledge automatically.
- **Voice is not authority.** Voiceprint, speaker-recognition result, wake word, TTS voice or familiar-sounding audio never grants permission by itself.
- **Expression Engine owns semantic expression.** TTS/audio rendering cannot add, remove or change commitments, uncertainty, safety warnings or authority language.
- **ASR is observation, not perfect user text.** Speech hypotheses preserve engine/version, confidence, timing and correction lineage; consequential ambiguity fails closed.
- **Every speaker is attributable or explicitly unknown.** Never silently assign an ambiguous human speaker to an enrolled principal.
- **Every agent/expert speaker is bound to a stable agent principal independent of synthetic voice presentation.**
- **Corrections append/supersede; they do not erase prior machine hypotheses.**
- **Human barge-in/new steering takes precedence over stale agent speech.**
- **Voice invokes the same canonical Focusa operations as CLI/Desktop/Pi/generated UI.** Do not create a parallel speech-only authority/task system.
- **Conversation/audio content is a separate privacy/retention domain, not generic telemetry.**
- Large transcript/audio content belongs behind bounded handles/content-addressed storage; do not bloat the canonical event chain with repeated raw blobs.

The final historical invariant in this document remains controlling: **meaning lives in Focus State, not in conversation.** Spec 181 adds durable conversational provenance without weakening that rule.

---

## Project Foreman / Radar / Ambient Operator hard stop

Before changing persistent project-agent identity, proactive observation, autonomous attention, mobile/wearable presence, phone/earbud integration, wake-word behavior, meeting capture, location/context projection, or cross-surface identity routing, read:

- `164-workstream-rooted-canonical-runtime-design.md` — current canonical Workstream Root identity/persistence;
- `139-distributed-presence-environment-awareness-execution-placement-and-multi-daemon-coordination-spec.md` — runtime Presence/Operational Reality Field and placement;
- `182-focusa-project-foreman-workstream-intelligence-projection-spec.md` — Project Foreman;
- `183-focusa-radar-proactive-observation-episodes-signal-economics-and-attention-routing-spec.md` — Radar;
- `184-focusa-ambient-operator-mobile-wearable-presence-meeting-and-sync-spec.md` — Ambient Operator;
- `181-focusa-voice-conversation-expression-and-auditable-interaction-spec.md` — voice/meeting/conversation provenance;
- applicable Specs 53, 72, 79, 133, 135/135B, 136–141, 151, 156.

Non-negotiable laws:

- **Foreman is a Workstream-scoped project-intelligence role projection, not a chatbot/session/model or second memory store.** One default Foreman binds one Spec-164 Workstream Root.
- **Model/harness switching changes runtime attachment, not Foreman/project identity.** Pi remains the reference harness; hidden harness memory never becomes canonical Foreman state.
- **Radar notices; it does not authorize.** Radar observations/signals/episodes remain scoped evidence/proposals until normal Focusa operations accept or act on them.
- **Radar is not Spec 139.** Spec 139 owns runtime environment/presence/placement; Radar consumes those facts plus other approved signals and owns proactive attention/economics/episodes.
- **Radar is not hidden surveillance.** Prefer native structured events; no ambient microphone, keystroke or unrestricted screen capture merely to generate signals.
- **Ambient Operator is a mobile/wearable projection of the same Focusa system, not a second brain.** Phone, earbuds and companion apps are paired surfaces/adapters.
- **Raw phone GPS/sensor context remains in its owning context domain by default.** Focusa receives bounded, freshness/provenance/privacy-bearing presence projections where relevant.
- **Meeting capture is a Spec-181 ConversationSession profile.** Speaker/ASR correction lineage is preserved; commitments/decisions become typed candidates, not automatic Workpoint/policy changes.
- **Wake word is attention, not identity or authority.** Pre-trigger audio is volatile/discarded by default where the profile permits.
- **Mobile sync never writes reducer/SQLite state directly.** It submits authenticated, replay-safe, idempotent proposals/observations/conversation segments through typed operations.
- **Wirebot/Chief of Staff may reason across Workstreams and delegate to exact Foremen, but does not become a global Focusa Workstream singleton.**
- **Historical proposal numbering is not current canon.** `Radar Spec 164` is historical proposal provenance only; current Spec 164 is Workstream Root. Historical `135M` is proposal provenance only; the current Spec 135 series is frozen at 135A–135K and Ambient Operator is Spec 184.

---

## Agent-KB API Default Reference

For KH/OVH/operator policy, use `agent-kb-api` first, verify freshness, retrieve exact documents after empty searches, and use local Agent KB files only as a read-only degraded fallback.

## Agent communications + GitHub 2FA documentation contract

- Communications design must identify authorized `github.com` SMS OTP retrieval/injection as the immediate release-critical use case while prohibiting ambient message access.
- Specifications and runbooks must make GitHub MFA SMS-first: broker-side `inject_otp` is the default; degraded pairing triggers private connector repair/re-pairing, not silent substitution with GitHub Mobile, passkey, or authenticator app. Alternate renewable methods require explicit Sir V3 direction.
- Specifications must extend Spec 156 credential/MFA authority and existing privacy, adapter, audit, placement, and Veragensia lifecycle contracts rather than create a parallel secret system. OTP values are ephemeral P4 material: persist only redacted handles and value-free evidence.
- GitHub OTP is the first bounded slice. Specifications must preserve a later customer-authorized SMS API for thread listing, bounded reads, sends, and events, with capabilities and consent distinct from OTP access; no privilege widening is allowed.
- Shared CLI/API/MCP/OpenClaw contracts stay transport-neutral behind versioned connector adapters. Android/Google Messages is a bounded bootstrap; **iPhone/iOS is a concurrent urgent target**, with an explicit supported/user-consented integration decision, parity matrix, migration/portability path, real-device acceptance, and no dependency on private Apple APIs.
- Every plan must cover scoped provider/challenge/message-class authorization, injection without routine plaintext exposure, encryption, restart recovery, health, revocation/re-pairing, attribution, audit, rate limits, replay/duplicate-send defense, prompt-injection resistance, customer handoff, and zero-residue teardown. Recovery codes are out of scope and permanently forbidden.

## Core Authority

- **Beads** is the authoritative task system
- **Focusa** governs focus and cognition
- Agents do not invent work

## Public / Private Docs Boundary

Private operator docs may exist locally at `.focusa-private/`.

Agents must read `.focusa-private/INDEX.md` before touching SaaS strategy, SignalOS, commercial pricing/caps, install/purchase backend, raw proof, launch planning, or vendor/license registry work.

Agents must never commit `.focusa-private/`, raw private transcripts/audio, runtime objects, local host paths, admin URLs, customer data, or license data.

Public examples of conversation/audit contracts must use synthetic or redacted fixtures.

---

## Required Agent Behaviors

### Focus Discipline
- Maintain exactly one active Focus Frame
- Never switch focus implicitly
- Always bind work to a Beads issue

### Focus State Updates
- Update incrementally
- Never overwrite prior decisions
- Log contradictions explicitly

### Intuition Respect
- Do not act on intuition signals
- Surface candidates for review only

### Reference Store Usage
- Store large outputs immediately
- Reference via handles only
- Never inline large artifacts

### Expression Discipline
- Respect deterministic structure
- Do not inject hidden instructions
- Preserve semantic parity across text/spoken rendering
- Link governed spoken outputs to the exact ExpressionOutput they render
- Do not let a speech/TTS provider become a second expression authority

### Conversation Discipline
- Preserve participant/speaker provenance
- Preserve ASR/transcript correction lineage
- Keep Conversation Ledger state separate from Focus State/memory promotion
- Link consequential utterances to resulting operation/Evidence/Receipt refs
- Treat uncertain speaker/content attribution honestly

### Foreman / Radar / Ambient Discipline
- Resolve exact Workstream before Foreman mutation or delegation
- Hydrate project intelligence from Focusa state, not transcript tail
- Preserve Radar source/fingerprint/Episode provenance and attention economics
- Route mobile/ambient inputs through typed projections and operations
- Preserve offline/stale/mobile-platform states honestly
- Keep life-context owner systems and Focusa Workstream cognition as separate authority domains

---

## Forbidden Agent Actions

- Autonomous task switching
- Silent memory mutation
- Bypassing Focus Gate
- Editing archived frames
- Acting without Beads backing
- Treating transcript order as canonical instruction precedence
- Treating voice identity as authorization
- Letting TTS/ASR adapters silently amend meaning
- Promoting raw conversation directly into durable policy/knowledge without the governed path
- Creating a daemon-global Foreman/Radar/current-project authority singleton
- Treating a Radar Signal as self-authorizing work
- Uploading ambient raw phone/microphone/location data to Focusa without its explicit bounded contract
- Reusing stale historical proposal numbers as current spec authority

---

## Beads Commands (Required)

Agents MUST use documented Beads commands (`bd`) only.

### Common Commands
- `bd new`
- `bd list`
- `bd show`
- `bd next`
- `bd done`
- `bd block`
- `bd log`

If work is not tracked in Beads, it does not exist.

---

## Failure Handling

On confusion or ambiguity:
1. Pause
2. Surface candidate
3. Await instruction

For speech ambiguity affecting consequential action, retain the hypothesis/evidence and request exact clarification rather than guessing.

For ambiguous Foreman scope, resolve/clarify exact Workstream rather than falling back to a global project agent.

---

## Final Rule

> **Meaning lives in Focus State, not in conversation.**

Agents that violate this invariant are non-compliant.
