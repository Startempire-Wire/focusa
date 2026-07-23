# Spec 133 Phase 7.5 — Pi awareness and menubar projections

Date: 2026-07-23
Bead: `focusa-a6yq6.8.5`
Scope: Spec 133 operator projections

## Daemon-only menubar card

`SilentSessionsPeek.svelte` consumes only:

```text
GET /v1/silent-sessions/dashboard?limit=20
```

The runtime store/card show bounded durable session state, health, project/work item, model, activity, elapsed time, checkpoint, attention reason, evidence count, completion, daemon-advertised controls, worktree/run/generation, and recent-event count.

Full output is never inlined; the card directs expansion through daemon cursor/artifact handles. The card has no code path that creates approval, lease, Workpoint, writer, closure, or Context Authority.

The card remains available whenever the daemon is available and does not depend on a foreground Pi/plugin process.

## Pi awareness

The Pi status command now reads the same daemon dashboard endpoint and adds a bounded line:

```text
Silent sessions: <visible> visible | attention=<count> | source=daemon
```

It does not infer session state from Pi memory or legacy tmux state. Existing daemon control surfaces remain the only mutation route.

## Projection boundaries

- daemon API is the sole source;
- at most twenty sessions are shown;
- cards expose rehydrate handles rather than payload dumps;
- no projection mints authority;
- no foreground Pi dependency is introduced.

## Local non-building proof

Per operator policy, no local package build, Svelte compile, TypeScript compile, CI, or tests were run.

```bash
git diff --check
python3 <bounded source consistency assertions>
```

The assertions verified endpoint wiring, runtime-store/card linkage, bounded full-output handles, and Pi daemon fetch/status line.

Result: passed.

## Required server proof

Run only on the build server:

```bash
pnpm --dir apps/menubar check
pnpm --dir apps/menubar test
pnpm --dir apps/pi-extension typecheck
pnpm --dir apps/pi-extension test
bash tests/spec133_phase7_operator_gate.sh
```

Server/browser proof must verify daemon restart continuity, twenty-session bound, attention rendering, no inline output flood, no authority mutation, Pi-absent menubar behavior, and Pi status daemon sourcing.

## Gate disposition

Implementation and local static review are complete. Build/typecheck/UI closure remains server-owned and must pass before this bead is marked fully proven.
