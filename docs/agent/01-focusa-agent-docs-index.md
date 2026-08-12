# Focusa Agent Docs Index

This is the bounded public-safe starting point for AI agents working in the Focusa repository.

## 0. Mandatory current bootstrap

Read `docs/agent/00-p0-transition-bootstrap.md` first.

Current foundational authority:

1. `docs/158-workstream-rooted-cognitive-runtime-foundation-migration-spec.md`
2. `docs/spec158/01-identity-ownership-and-reducer.md`
3. `docs/spec158/02-persistence-migration-and-quarantine.md`
4. `docs/spec158/03-client-runtime-and-desktop-contracts.md`
5. `docs/spec158/04-implementation-task-graph-and-closure.md`
6. `docs/transitions/FOCUSA-TRANSITION-001-mission-canvas-to-desktop-handoff.md`
7. `docs/transitions/FOCUSA-TRANSITION-001-preview-build-and-release-milestones.md`
8. `docs/transitions/FOCUSA-TRANSITION-001-task-graph.yaml`
9. `docs/transitions/FOCUSA-TRANSITION-001-desktop-milestones.yaml`

Any older document that treats `project_root + continuity_id`, Thread, Session, CWD, UI selection or daemon-global active/current state as complete canonical authority is superseded by Spec 158.

## 1. What Focusa is

Focusa is the local-first proof, continuity and canonical meaning layer for AI work. It keeps long-running work inside a typed Workstream, attached to a canonical Workpoint, tactical Trajectory, linked Evidence and a next safe action.

The Focusa reducer is the sole canonical mutation boundary. Models, tools, agents, UIs and external runtimes propose; the reducer canonizes meaning inside one exact Workstream partition.

**Distribution boundary:** current public builds still contain legacy Evaluation paths. New evaluator/customer distribution remains governed by Spec 152, Spec 150A and applicable Spec 152A requirements. Do not restore or infer a local Evaluation bypass.

## 2. Canonical identity

```text
ScopeRef / ProjectRootKey
  -> WorkstreamId
    -> ContinuityId
      -> AttachmentKey
        -> SessionId / InstanceId
          -> runtime object
            -> WorkSurfaceId
```

- WorkstreamId is durable cognitive workspace identity.
- ContinuityId is lineage inside a Workstream.
- Thread is legacy/historical terminology only.
- Session/Instance are temporal runtime metadata.
- Attachment binds runtime identity to one Workstream.
- Work Surface is presentation identity only.
- Workpoint is immediate action authority inside one Workstream.
- project HLT is project-level; tactical Trajectory is Workstream-owned.
- visual focus, CWD, last project, latest record and transcript tail do not grant authority.

## 3. Architecture map

| Layer | Purpose | Key locations |
| --- | --- | --- |
| CLI | Operator/agent commands and planned Desktop control | `crates/focusa-cli/src/commands/` |
| API daemon | Typed control plane and Workstream ScopeRouter | `crates/focusa-api/src/routes/` |
| Core | Workstream reducer, persistence, Workpoints and Evidence | `crates/focusa-core/src/` |
| Spec 158 | singleton removal, partitioning, replay, migration and quarantine | `docs/158-*`, `docs/spec158/` |
| Work loop/Silent Sessions | Workstream-owned governed execution | `crates/focusa-core/src/silent_sessions/`, Spec 133 |
| Mission Canvas | Workstream projections and Work Surfaces | Spec 135 plus FOCUSA-TRANSITION-001 |
| Focusa Desktop | primary rich local Focusa application | planned `apps/desktop/` and shared packages |
| Pi extension | tools, Attachment hooks, compaction, Work Rail and compatibility Canvas | `apps/pi-extension/` |
| Generated UI | A2UI Lit and Focusa Svelte Custom Elements | `packages/a2ui-renderer/`, `packages/focusa-elements/` |
| Menubar | compact lifecycle/status/handoff | `apps/menubar/` |
| Focusa.work | hosted/web projection of portable workspaces | planned environment adapters |
| Agent contracts | Pi/MCP/OpenAI/CLI/REST schemas and Agent Card | `docs/contracts/spec141/generated-capability-v2/` |
| Skills/runbooks | progressive agent onboarding | `.pi/skills/`, `apps/pi-extension/skills/` |

## 4. Mission Canvas/Desktop transition

The old primary route—building the complete rich Mission Canvas inside Pi TUI—is frozen.

Preserve and migrate existing semantic/runtime work. Keep Pi as:

- authentic standalone coding/conversation surface;
- embedded PTY-backed Desktop Work Surface;
- Work Rail/status owner;
- bounded terminal/SSH compatibility Canvas.

Focusa Desktop becomes the primary app. It must be fully controllable by the Focusa agent through the CLI and typed tools using stable workspace, subsection, object, Work Surface and command IDs.

## 5. MacBook implementation policy

For the active Mission Canvas refactor:

- work and test in the current MacBook worktree;
- create local preservation and implementation commits;
- do not push directly to `main` or shared Mission Canvas branches;
- publish only an approved review branch/commit;
- use one pinned Rust toolchain;
- preview continuously through the shared SvelteKit browser app;
- use UIAI Engine for browser proof;
- build the complete Tauri shell at 5%, 25%, 50%, 75% and 100%;
- do not use local release builds or upload Mac artifacts;
- at 75%, initiate the canonical release from the approved KnownHost release host after operator approval.

Private hostnames, IP addresses, credentials and SSH details remain outside this public repository.

## 6. Agent readiness fast path

1. Fetch remote and preserve local work before rebase or cleanup.
2. Resolve exact ScopeRef + WorkstreamId and exact Attachment when runtime mutation matters.
3. Resume Workstream-owned Workpoint and tactical Trajectory.
4. Discover capabilities progressively: `focusa_agent_card` → `focusa_tool_search` → `focusa_tool_describe`/`focusa_tool_graph`.
5. Load the matching `.pi/skills/<name>/SKILL.md`, then its numbered runbook when required.
6. Keep runtime registration, generated contracts, descriptors and tool docs one-to-one.
7. Use daemon-native Silent Sessions with exact Workstream/Attachment/run/generation and approval/idempotency.
8. Preserve Workstream state during compaction and rollover.
9. Treat blocked, degraded, ambiguous and recovery-only states as non-completion.

## 7. Licensing/onboarding authority

Before touching install, license, Evaluation, protected modules, UIAI entitlement or first-run code, read:

1. `docs/152-mandatory-authority-licensing-evaluation-entitlements-and-unified-onboarding-spec.md`
2. `docs/150a-spec152-entitlement-overlay-and-lifecycle-integration.md`
3. `docs/152a-protected-distribution-private-feature-capsules-and-anti-tamper-spec.md`
4. `docs/contracts/spec152-supersession-and-integration-matrix.v1.yaml`
5. `docs/current/INSTALLER_UPDATE_POLICY.md`
6. `docs/current/FIRST_RUN_FLOW.md`

Private authority/server implementation belongs in the private authority repository.

## 8. API and daemon rules

- daemon health is infrastructure, not Workstream readiness;
- cognitive routes must migrate to one WorkstreamContext extractor;
- Workstream ambiguity fails closed with zero foreign cognitive payload;
- compatibility routes require explicit mapping, deprecation metadata and no latest/current fallback;
- recovery routes remain available where safe;
- authentication, pairing, loopback and source checkout do not imply entitlement or Workstream authority.

## 9. Release policy

- Use the canonical GitHub release pipeline only.
- Canonical command: `scripts/create-dev-release-tag.sh --base 0.9 --push`.
- Required chain: CI → Release → Deploy Live Daemon → audit/self-heal/watchdog.
- For the Desktop transition, run the release command from the approved KnownHost release host at the 75% gate after operator approval.
- Do not build release artifacts on the MacBook with `cargo build --release`.
- Do not manually upload artifacts, create ad hoc tags or run a partial workflow shortcut.

## 10. Public/private boundary

Do not add private host paths, hostnames, IPs, admin URLs, secrets, tokens, keys, customer data, raw transcripts, local runtime databases, commercial calculations, signing material or private worker implementation.

Use public-safe stable contracts, synthetic fixtures and redacted Evidence references.

## 11. Work checklist

1. fetch remote;
2. inspect and preserve the worktree;
3. read current bootstrap/spec/transition documents;
4. identify/claim task-graph nodes;
5. resolve exact Workstream identity;
6. make the smallest bounded change;
7. keep browser preview current and capture UIAI Engine proof;
8. run native shell only at the defined milestone gates plus bounded debugging necessity;
9. update migration ledger, milestone Evidence and task status;
10. follow the active local/upstream publication policy.
