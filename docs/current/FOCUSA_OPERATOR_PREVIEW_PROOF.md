# Focusa Operator Preview Proof

## Version

- Snapshot: `v0.9.13-dev` / Operator Preview
- Proof commits: `ccd4449` golden demo, `704b7fb` operator session card, `9ba6d04` copy-prompt, plus this public-doc sync
- Release posture: source-available commercial preview, Workpoint-first and proof-first

## Commercially supported preview promise

A developer can start a real AI coding session, create a Workpoint, attach evidence, recover after compaction/session drift, and continue without losing the thread.

## Verified workflows

| Workflow | Preview status | Proof / command |
|---|---|---|
| Project identity resolves | Implemented | `focusa project identity --json` / `/v1/project/identity` |
| Daemon/API health | Implemented | `focusa status`, `focusa doctor --json`, `/v1/health` |
| Workpoint checkpoint creates canonical packet | Implemented | `focusa workpoint checkpoint --project-root <root> --continuity-id <id>` |
| Workpoint resume returns usable continuation packet | Implemented | `focusa workpoint resume --mode compact_prompt` |
| Evidence link is visible in Workpoint state | Implemented | `focusa workpoint evidence-link --target-ref <ref> --result <summary>` |
| Trajectory ladder terms are explicit | Implemented/advisory | HLT → MLG → STG → Waypoints → Workpoint; defer to operator while offering route guidance |
| Trajectory view is scoped/advisory | Implemented/advisory | `focusa trajectory view --project-root <root> --mode summary` |
| Drift check catches wrong target/action | Implemented | `focusa workpoint drift-check --latest-action <action>` |
| Non-Pi manual awareness card | Implemented | `focusa awareness card --adapter-id manual --workspace-id local --agent-id cli` |
| Non-Pi Workpoint continuation prompt | Implemented | `focusa workpoint resume --copy-prompt` |
| First-run Operator Preview onboarding | Implemented in CLI | `focusa onboard --agent pi` or `focusa onboard --agent manual` |
| Operator session card | Implemented in CLI | `focusa status --operator` |

## Golden proof path

One command proves the core Operator Preview loop:

```bash
scripts/demo-workpoint-happy-path.sh
```

Manual equivalent:

```bash
# 1. Build/install Focusa binaries or use cargo run.
cargo build --release --bins

# 2. Run first-run onboarding in the checkout you want to protect.
focusa onboard --agent pi

# 3. Inspect health/repair guidance and the visible session card.
focusa doctor
focusa doctor --json
focusa status --operator

# 4. Create or resume the Workpoint.
focusa workpoint resume --mode compact_prompt

# 5. Link evidence after a real change/test.
focusa workpoint evidence-link \
  --target-ref docs/current/FOCUSA_OPERATOR_PREVIEW_PROOF.md \
  --result "Operator Preview proof reviewed and updated" \
  --evidence-ref docs/current/FOCUSA_OPERATOR_PREVIEW_PROOF.md

# 6. Validate scoped orientation.
focusa trajectory view --mode summary

# 7. Generate non-Pi continuation context if not using Pi.
focusa workpoint resume --copy-prompt
focusa awareness card --adapter-id manual --workspace-id local --agent-id cli
```

## Operator-readable proof expectations

A passing preview proof should show:

- daemon reachable,
- project root safe and detected,
- license/commercial files visible,
- Workpoint canonical or explicitly degraded,
- resume packet includes project root, continuity id, mission, next action, blockers, and evidence refs,
- Trajectory ladder is explicit: HLT (High-Level Trajectory) → MLG (Mid-Level Goal) → STG (Short-Term Goal) → Waypoints → Workpoint,
- Trajectory is advisory and does not override Workpoint/project scope,
- doctor reports exact recovery command when anything blocks,
- manual mode gives a copy/paste card for non-Pi agents.

## Known limits

- Pi is the best-supported deep harness path today.
- Manual mode is supported through CLI/API continuation cards, not every agent-specific adapter.
- The menubar GUI is useful but not the primary Operator Preview surface.
- Work-loop and metacognition are advanced preview surfaces, not the first buyer workflow.
- Some ontology/governance documents remain design-forward and should not be treated as release claims unless the README/current docs mark them implemented.
- Team/multi-user, cloud sync, marketplace, and enterprise RBAC are future surfaces.

## Release gates for Operator Preview

- [x] `focusa onboard` works in a clean checkout.
- [x] `focusa doctor` gives human and JSON repair guidance.
- [x] Workpoint checkpoint/resume works with explicit `project_root` + `continuity_id`.
- [x] Evidence link appears in Workpoint state/resume path.
- [x] Trajectory view remains advisory/scoped.
- [x] Manual awareness card works for non-Pi agents.
- [x] A golden demo script proves onboarding → Workpoint → evidence → resume → drift check.
- [x] README maturity table and known limits are visible.
- [x] Commercial/license docs are present and linked.
