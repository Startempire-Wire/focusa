# Focusa Agent Docs Index

This is the bounded, public-safe starting point for AI agents working in the Focusa repo. Use it before broad code changes or after context loss.

## 1. What Focusa is

Focusa is the local-first proof and continuity layer for AI coding agents. It keeps long-running work attached to a typed Workpoint, linked Evidence, and a next safe action so agents do not rely on chat tail memory.

**Pi + the Focusa Pi extension is the default/reference Focusa-aware harness integration.** Pi is fundamental to the reference agent experience, but Focusa daemon/core remains cognitive authority and canonical state must not become Pi-private. Compatible non-Pi harnesses remain first-class through thin adapters and generated Focusa capability contracts.

**Focusa Desktop is a presenter over Focusa authority.** In supported full Veragensia Agent Computer profiles it is the default governed human work/cognition/conversation presentation surface; it does not evaluate or invent authority independently.

**Voice/Conversation is a first-class Focusa primitive.** Doc 08 Expression Engine owns semantic expression—what Focusa says now—and Spec 181 owns spoken ConversationSession/Participant/Utterance/TranscriptRevision/SpokenOutput lineage and the local-first Conversation Ledger. Conversation can be extensively retained and audited without becoming canonical memory or authority.

**Project Foreman, Radar, and Ambient Operator are now current primitives.** Spec 182 defines one Workstream's persistent project-intelligence role projection, Spec 183 proactive scoped attention, and Spec 184 the paired mobile/wearable/meeting surface. Historical `Radar Spec 164` and `135M` are proposal provenance only; current Spec 164 remains Workstream-rooted runtime.

## 2. Architecture map

| Layer | Purpose | Key locations |
| --- | --- | --- |
| CLI | Operator and agent command surface | `crates/focusa-cli/src/commands/` |
| API daemon | Local typed HTTP API | `crates/focusa-api/src/routes/` |
| Core | reducers, Workpoints, Evidence, runtime state, persistence | `crates/focusa-core/src/` |
| Expression Engine | deterministic semantic expression; modality-neutral content before text/audio rendering | `docs/08-expression-engine.md`, `crates/focusa-core/src/expression/` |
| Voice / Conversation | spoken-session participants, ASR hypotheses/corrections, utterances, group conversation, spoken-output lineage, Conversation Ledger | `docs/181-focusa-voice-conversation-expression-and-auditable-interaction-spec.md` |
| Project Foreman | persistent Workstream-scoped project-responsible intelligence across models/surfaces/workers | `docs/182-focusa-project-foreman-workstream-intelligence-projection-spec.md` |
| Radar | proactive scoped observations, Episodes, Signals, attention economics and Foreman routing | `docs/183-focusa-radar-proactive-observation-episodes-signal-economics-and-attention-routing-spec.md` |
| Ambient Operator | paired mobile/wearable presence, meetings, voice routing, offline/private sync | `docs/184-focusa-ambient-operator-mobile-wearable-presence-meeting-and-sync-spec.md` |
| Work loop + Silent Sessions | governed execution, durable runs, steering, receipts | `crates/focusa-core/src/silent_sessions/`, `docs/133-silent-sessions-final-release-proof.md` |
| Mission Canvas + Work Rail | scoped work surfaces, interviews, artifacts, generated UI | `docs/135-series-current-manifest.md`, `apps/menubar/` |
| Connectors + domains | provider-neutral context, auth lifecycle, software/domain projections | `crates/focusa-core/src/connectors.rs`, `docs/contracts/spec135/` |
| Credential Authority + authentication | project-scoped requirements, grants, leases, provider custody, controlled injection, MFA/TOTP, revocation | `docs/156-focusa-project-scoped-credential-authority-secret-broker-delegated-autonomy-mfa-totp-and-cross-surface-injection-spec.md` |
| TUI / Mission Deck | terminal cockpit | `crates/focusa-tui/` |
| Pi reference harness | all Focusa Pi tools, authority hooks, compaction/OTA/runtime bridge | `apps/pi-extension/`, `docs/52-pi-extension-contract.md` |
| Agent machine contracts | Pi/MCP/OpenAI/CLI/REST schemas and Agent Card | `docs/contracts/spec141/generated-capability-v2/` |
| Skills + runbooks | progressive agent onboarding and recovery playbooks | `.pi/skills/`, `apps/pi-extension/skills/` |
| Focusa Desktop / menubar presenter | native/Tauri governed presentation; no direct authority bypass | `apps/menubar/`, `docs/contracts/spec152f-desktop-action-map.v1.json` |
| Public docs | current reference, onboarding, lifecycle, and specs | `README.md`, `docs/`, `docs/current/` |

### 2.1 Current authority and recovery model

- Exact authority is `project_root + continuity_id`; parent repositories and worktrees are ranked binding candidates, then verified before mutation.
- Workstream Root (Spec 164) is the durable project runtime root; state does not become daemon-global merely for convenience.
- Workpoint is immediate action authority; Trajectory supplies destination, current state, gap, and waypoints.
- Focus State is the bounded decision/constraint/failure journal, not a transcript replacement.
- **Conversation Ledger is provenance/audit history, not Focus State or memory authority.** Full transcripts may survive while meaning/continuation still comes from Focusa canonical state.
- **Project Foreman is a role projection over one Workstream's canonical intelligence, not a hidden session memory.** Model/provider/harness switching changes runtime attachment rather than project identity.
- **Radar is observation/attention, not authority.** Radar Signals/Episodes never directly mint a Workpoint, grant, or canonical fact.
- **Ambient Operator is a paired surface, not a second brain.** Raw phone/life context remains in its owning domain until a bounded projection is explicitly relevant.
- ASR output is a speech hypothesis with confidence/correction lineage. Consequential ambiguity does not silently become operator instruction.
- Speaker/agent attribution remains explicit; synthetic TTS voice is presentation, not principal identity or authority.
- Silent Sessions are daemon-native. Exact `session_id`, `run_id`, `generation`, approval, and idempotency values govern mutations.
- Proactive compaction preserves canonical Workpoint/Trajectory packets and queues governed automatic rollover after bounded transport exhaustion.
- Cache-safe context keeps stable prefixes and current user-tail authority while classifying degraded fallbacks explicitly.
- Mission Canvas binds Work Surfaces to canonical operations and project scope; browser/UIAI capabilities remain session-and-origin bound.
- Customer lifecycle requires verified install/repair, trusted update or OTA rollback, and uninstall that preserves user data unless purge is explicit.
- Pi/reference-harness convenience never changes these authority rules; non-Pi adapters must consume the same canonical state rather than duplicate it.
- Focusa Desktop and other presenters render/forward shared operation decisions; they do not create local entitlement or reducer bypasses.

### 2.2 Voice / Conversation fast path

When a change touches speech, audio, transcript, group conversation, speaker identity, spoken output, or Veragensia Audio UI:

1. Read `docs/08-expression-engine.md`.
2. Read `docs/181-focusa-voice-conversation-expression-and-auditable-interaction-spec.md`.
3. Preserve `conversation != memory`, `voice != authority`, and `TTS rendering != semantic expression`.
4. Preserve raw audio/transcript privacy separately from telemetry.
5. Bind spoken requests to the same canonical operation/authority path as CLI/Desktop/Pi.
6. Bind each governed spoken agent utterance to its exact `ExpressionOutput` and agent principal.
7. Preserve transcript/speaker correction lineage instead of rewriting history.
8. In group conversations, preserve independent agent/expert speaker identity and action/Evidence/Receipt lineage.

### 2.3 Foreman / Radar / Ambient fast path

When a change touches persistent project-agent identity, proactive monitoring, Radar Signals/Episodes, mobile/earbud presence, wake words, meeting capture, Context Core projection, or Wirebot cross-project routing:

1. Read `docs/181-184-voice-foreman-radar-ambient-operator-current-manifest.md`.
2. Read Spec 164 before changing Workstream/Foreman identity.
3. Read Spec 139 before changing environment presence or execution placement.
4. Read Specs 182–184 for Foreman/Radar/Ambient ownership.
5. Resolve exact Workstream before Foreman mutation/delegation.
6. Keep Radar source/freshness/fingerprint/Episode lineage and never promote observation directly into authority.
7. Keep life-context raw sensors/GPS in their owner domain by default; publish bounded Ambient presence projections only when relevant.
8. Keep meeting/wake/voice under Spec 181; conversation content does not become Radar storage or project memory.
9. Mobile sync submits typed, authenticated, replay-safe operations/segments rather than reducer/database writes.
10. Wirebot/Chief of Staff may aggregate bounded cross-Workstream projections but does not become a global Focusa project singleton.

### 2.4 All-Pi-tool and skill discovery

1. `focusa_agent_card` reports the runtime tool count, complete installed skill/runbook inventory, interfaces, auth, and registry digest.
2. `focusa_tool_search` finds the narrowest capability without hot-loading every schema.
3. `focusa_tool_describe` cold-loads one strict contract; `focusa_tool_graph` or `focusa_tool_bundle` expands only the selected workflow.
4. `docs/contracts/spec141/generated-capability-v2/pi-tools.json` is the machine projection for every Focusa Pi tool.
5. `docs/focusa-tools/tools/focusa_<name>.md` is the human reference for each tool.
6. Background execution primitives: `focusa_bg_run` / `focusa_bg_run_many` / `focusa_bg_status` — the canonical non-TBQ dispatch. Completions arrive on the agent front terminal via the `background_job_completion` SSE envelope (bounded `output_tail`). Multi-agent work = N workloop-bound Silent Sessions, never raw shells.
7. Production consistency (DEFAULT, every surface): five proofs — versioned contract, producer tests, consumer-side tests, cross-version interop, live e2e — per `docs/current/PRODUCTION_CONSISTENCY_POLICY.md`.
8. Fast-forward multiplier (2x/4x/6x/8x…): operator-conceived #312 — FanoutPlan round-robin task division across parallel sessions with per-lane policy budgets (docs/169).
9. Load the matched `.pi/skills/<skill>/SKILL.md`, then its numbered runbook under `references/`.

A release gate must prove runtime tool count = contracts = Pi descriptors = per-tool docs, and installed skills/runbooks = packaged skill/runbook copies.

## 3. Canonical command surface

Start with:

```bash
focusa help all
focusa help migration
focusa project
focusa setup wizard --dry-run
focusa first-mission --project-root "$PWD" --dry-run --json
focusa status operator --json
```

Core continuity commands:

```bash
focusa workpoint checkpoint --project-root "$PWD" --continuity-id demo --mission "Mission" --next-action "Next slice" --json
focusa workpoint evidence-link --target-ref tests --result "smoke passed" --evidence-ref "test:smoke" --json
focusa workpoint resume --project-root "$PWD" --continuity-id demo --copy-prompt
```

Background execution and lifecycle discovery:

```bash
focusa silent --help
focusa tui --headless-self-test
focusa update --help
bash scripts/install-focusa.sh --dry-run
bash scripts/install-focusa.sh --uninstall        # preserves user data
focusa uninstall --dry-run --keep-data
```

Safety and proof commands:

```bash
focusa action preflight --current-ask "change binary" --kind binary_replace --target /usr/local/bin/focusa --source github_release_asset --install-role live_build_host --project-root "$PWD" --json
focusa cleanup --safe --project-root "$PWD" --dry-run --json
scripts/guard-public-surface.sh
bash tests/spec_cli_cross_phase_smoke_test.sh
```

## 4. API and daemon rules

- Default daemon URL: `http://127.0.0.1:8787`.
- Health route: `GET /v1/health`.
- Workpoint resume route: `POST /v1/workpoint/resume` with a JSON body.
- Telemetry snapshot route: `GET /v1/telemetry/snapshot`.
- Project-scoped mutations must use a verified safe project root.
- Daemon-global advisory surfaces must say they are advisory and non-canonical.
- Future voice/conversation, Foreman, Radar, and Ambient routes must preserve canonical scope/operation/authority semantics and use bounded handles for large audio/transcript content.

## 5. Workpoints, Evidence, and Trajectory

- **Workpoint** is the immediate continuation contract: mission, scope, current action, next action, blockers, and proof handles.
- **Evidence** is proof linked to the active Workpoint: tests, files, route checks, screenshots, command output, or release checks.
- **Trajectory** is advisory north-star context: long-term direction and current gap. It orients work but does not override a canonical Workpoint.
- **Context Authority** decides whether a proposed action matches the task, project, environment, and install role.
- **Conversation Ledger** records attributable interaction provenance and action links; it does not become a replacement Workpoint, Focus State, durable knowledge store, or authority system.
- **Project Foreman** presents one Workstream's persistent project responsibility; it does not create a new task/memory/permission universe.
- **Radar** turns approved observations into scoped Episodes/Signals/attention decisions; it does not turn detection into authorization.
- **Ambient Operator** routes paired mobile/wearable context and conversation into the same canonical operations.

Never treat transcript tail, old spoken discussion, Radar notification text, Foreman persona wording, or synthetic voice identity as canonical authority when Workpoint/scope/authority gates are available.

## 6. Update and release policy

- Use the GitHub release pipeline for public install/release artifacts.
- Keep CLI/daemon versions paired.
- Run focused tests for changed crates, then broader smoke tests when command surfaces change.
- Public release gates include the public-surface guard and cross-phase CLI smoke script.
- Do not publish local-only runtime data, private audio/transcripts, phone/location projections, or internal proof bundles as public release proof.

## 7. Public/private boundary rules

Agent-facing docs must stay public-safe.

Do not add:

- private host paths
- private admin URLs
- secrets, tokens, keys, or customer data
- full private chat/voice transcripts or raw audio
- speaker voiceprints/biometric material
- precise personal location or raw phone-sensor history
- local runtime databases, ledgers, or pairing state
- internal launch strategy or commercial calculations

Use public-safe replacements:

| Unsafe category | Public-safe wording |
| --- | --- |
| host-specific paths | `~/projects/focusa-demo` or `$PWD` |
| backend/admin URLs | `https://focusa.dev/support` or `https://install.focusa.dev/license` |
| full conversation dumps | synthetic transcript fixtures, bounded proof summaries or Evidence refs |
| life-context/location | synthetic/coarse example projections |
| license/customer records | public license terms and support path |

## 8. Software layout checklist for agents

Before code changes:

1. `git fetch origin`
2. `git status --short --branch`
3. Read this doc and the linked spec/current reference for the touched surface.
4. Identify the active bead/work item.
5. Make the smallest scoped change.
6. Run focused proof.
7. Update bead notes, commit, and push for normal public code repos.

## 9. Helpful references

- README product overview: `README.md`
- Voice/Foreman/Radar/Ambient manifest: `docs/181-184-voice-foreman-radar-ambient-operator-current-manifest.md`
- Voice/Conversation primitive: `docs/181-focusa-voice-conversation-expression-and-auditable-interaction-spec.md`
- Project Foreman: `docs/182-focusa-project-foreman-workstream-intelligence-projection-spec.md`
- Radar: `docs/183-focusa-radar-proactive-observation-episodes-signal-economics-and-attention-routing-spec.md`
- Ambient Operator: `docs/184-focusa-ambient-operator-mobile-wearable-presence-meeting-and-sync-spec.md`
- Workstream Root: `docs/164-workstream-rooted-canonical-runtime-design.md`
- Distributed Presence/Placement: `docs/139-distributed-presence-environment-awareness-execution-placement-and-multi-daemon-coordination-spec.md`
- Expression Engine: `docs/08-expression-engine.md`
- Spec151 modality parity: `docs/151-focusa-frictionless-program-design-runtime-and-agent-capability-fabric-spec.md`
- Pi reference contract: `docs/52-pi-extension-contract.md`
- Non-Pi parity/adapters: `docs/current/NON_PI_AGENT_FOCUSA_USAGE.md`
- Current CLI reference: `docs/current/CLI_REFERENCE_CURRENT.md`
- Public-surface guard: `scripts/guard-public-surface.sh`
- Cross-phase smoke: `tests/spec_cli_cross_phase_smoke_test.sh`
- Workpoint CLI implementation: `crates/focusa-cli/src/commands/workpoint.rs`
- Project command implementation: `crates/focusa-cli/src/commands/project.rs`
- API route implementations: `crates/focusa-api/src/routes/`

Installation and evaluation authority: authority-issued under Spec 152 (spec152) — the verified bootstrapper resolves a signed, node-bound authority lease; local self-issued evaluation is forbidden. Recovery posture: authority reissue.