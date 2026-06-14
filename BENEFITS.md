# Why Focusa is amazing for developers

> **TL;DR.** Focusa is a local-first cognitive runtime for systematic AI execution. It gives AI coding agents durable mission state, scope, trajectory, Workpoints, Evidence Refs, Context Authority, recovery, predictions, and a public surface — all in one local-first toolchain.

---

## The pitch in one paragraph

Modern coding agents fail the same way: context compacts, the thread is lost, drift accumulates, evidence disappears, and the next session has to re-derive everything. Focusa replaces that with typed **ProjectIdentity**, **Continuity ID**, **Focus State**, **Focus Stack**, **HLT → MLG → STG → Waypoints → Workpoint**, **Evidence Refs**, **Context Cognition**, **Context Authority**, **predictions** that can be evaluated, **metacognition** that compounds, and a public surface that makes the agent's work inspectable. Local-first, Rust-fast, audit-ready.

---

## The ten things developers actually get

1. **Compaction is not a wipeout.** A typed `Workpoint` survives any model context reset. The agent's mission, target objects, verified evidence, and next action all persist in a structured envelope. `focusa_workpoint_resume` rehydrates the packet in one call. No more "what was I doing?"

2. **Evidence is structural.** Every claim can be linked to a file, a test, a route, or a screenshot. `focusa_evidence_capture` and `focusa_workpoint_link_evidence` are typed, not "screenshots in chat." When the model gets evaluated later, the proof is already attached.

3. **Trajectory is a ladder, not a vibe.** `HLT` (long-term), `MLG` (mid-level), and `STG` (short-term) plus explicit waypoints give the agent a typed north star. When it drifts, the ladder catches it. `focusa_trajectory_view`, `focusa_trajectory_assess`, and `focusa_hlt_history` make it observable.

4. **Predictions are trackable.** The agent records what it expects (`focusa_predict_record` with a confidence and a `prediction_id`). You evaluate the outcome (`focusa_predict_evaluate`). Calibration improves across sessions. You get a real number (`focusa_predict_stats`).

5. **Metacognition compounds.** Capture a lesson (`focusa_metacog_capture`), retrieve it later (`focusa_metacog_retrieve`), reflect on it (`focusa_metacog_reflect`), and turn it into an adjustment (`focusa_metacog_plan_adjust`). The next agent doesn't rediscover the same mistake.

6. **Multi-agent is first-class.** Project roots, continuity IDs, and writer arbitration let multiple agents work in the same repo without stepping on each other. `focusa_project_identity` and `focusa_work_loop_writer_status` are the locks that make this safe.

7. **Local-first, audit-ready.** Everything runs on your machine or your VPS. The daemon is a typed HTTP API. Nothing leaks to a third-party model. Static audits verify the generated tool surface stays in sync on every CI run.

8. **Real observability.** Hot-path latency, cold-path cost, resource pressure, and degraded modes are surfaced in tools (`focusa_tool_doctor`, `focusa_resource_mode`), not buried in logs. The Tauri menubar app shows live focus, workpoint, and trajectory state on macOS.

9. **Real GUI.** The Tauri menubar app is built on every push to `main` and packaged as a macOS `.app` bundle in CI. You get a real desktop surface, not a CLI you have to remember.

10. **Public surface ready.** Tools emit typed envelopes. Project cards are shareable. With `FOCUSA_PUBLIC_STREAM=1`, tool calls become typed public cards — perfect for showcasing live agent work without leaking content.

---

## How it feels in practice

### Before Focusa

```
$ pi
> Let's refactor the auth module
... 30 minutes of work ...
> Now let's add rate limiting
... 20 minutes of work ...
> What's the test coverage?
<model lost track, restates the obvious>
> Wait, what was I doing before?
<model: "I'm not sure, want me to start a new task?">
```

### After Focusa

```
$ FOCUSA_PUBLIC_STREAM=0 pi
> focusa_workpoint_checkpoint mission="auth refactor + rate limit"
  ids: workpoint_id=019ea… next_action="add rate-limit middleware"
> focusa_workpoint_resume
  resume: scope=verified, mission=auth+rate-limit, evidence=2 files linked
  next: focusa_active_object_resolve
> ... work continues with full thread, even after a 200k-token compaction ...
> focusa_workpoint_resume
  resume: scope=verified, mission=auth+rate-limit, evidence=8 files linked
  next: continue where I left off
```

The difference: the agent always knows what it was doing, what was proven, and what comes next. **The work survives the model, not the other way around.**

---

## What this is not

- **Not a model.** Focusa is the cognitive layer around the model. It does not replace Claude, GPT, or your local LLM. It makes them better.
- **Not a SaaS.** Focusa is local-first. Your state, your data, your machine. The daemon is a single binary.
- **Not a closed system.** Every tool has a doc page (`docs/focusa-tools/tools/`), a contract, and a CLI command. You can read the source, audit the surface, and extend it.
- **Not magic.** It cannot make a bad prompt good. It can make a good prompt survive.

---

## Why now

- **Agent loops are getting longer.** Long sessions are the new default. Compaction is the new failure mode.
- **Context windows are not enough.** More tokens ≠ more memory. Structure beats tokens.
- **Evidence is the new trust signal.** Buyers, auditors, and teammates want proof, not prose.
- **Multi-agent is the new mono-agent.** Multiple agents in one repo is the default. Without a lock, it's a footgun.

Focusa is the smallest toolchain that makes all four of these problems tractable today.

---

## Call stack design: the single highest-leverage artifact

Designing explicit end-to-end call stacks upfront is the most effective way to guide AI coding agents. Focusa ships a typed, append-only, evidence-linkable **Call Stack Design** tool — `focusa_call_stack_design` — that turns this practice into a first-class artifact.

Before an agent writes a feature, it writes the call stack. The shape is forced: **entry → handlers → services → adapters → storage → output**, with each step typed and bounded. The design is linkable to the active Workpoint as `focusa_evidence`, so the artifact travels with the work. Drift is observable; the design is verifiable.

Why this matters:

- **It is the smallest possible surface change with the largest behavioral impact.** A 1KB typed blueprint changes how the agent thinks about the whole feature.
- **It composes with everything else in Focusa.** Workpoint, Trajectory, Evidence, Metacognition, Predictions — all can reference the design.
- **It is a defensible moat.** No other toolchain offers typed call stack designs as a first-class artifact.
- **It is a public surface ready.** When a session opts into the public stream, visitors see the call stack design appear before the code does — and they can click through to see the actual implementation.

This is the centerpiece of Focusa's claim that "structure beats tokens." See `docs/103-call-stack-architecture-blueprint-spec.md` for the full spec.

---

## The roadmap that is already shipping

- **Today:** Focusa ships a generated tool surface summary, daemon, CLI, TUI, Tauri menubar, strict CI, and static audits. See `docs/current/generated/tool-surface-summary.md` for current tool counts, families, parity, and docs coverage.
- **This month:** public stream surface (live tool cards on focusa.dev), HLT ledger human-readable view, expanded evidence links.
- **This quarter:** multi-agent federation, public proof bundles, MCP-native adapter.

---

## How to start

```bash
# 1. Build the daemon
cargo build --release -p focusa-api

# 2. Start it
./target/release/focusa-daemon

# 3. In your project, install the Pi extension
cd apps/pi-extension && npm install && pi

# 4. Inside Pi, call
focusa_project_identity
focusa_workpoint_checkpoint mission="your mission"
focusa_workpoint_resume
```

You will be live in 5 minutes. From there, the generated Focusa tool surface is your substrate.

To design a feature before writing it:

```bash
# Inside Pi
focusa_call_stack_design mission="Add the new feature" entry_name="focusa_foo" attach_to_workpoint=true
```

The design becomes `focusa_evidence` linked to your active Workpoint. The agent then has a typed blueprint to follow.

---

## Who builds this

Focusa is built by developers who run long agent sessions every day and got tired of the same failure modes. The repo is open, the contracts are public, the roadmap is the developer's wishlist, and shipping happens in days, not quarters.

If you ship code with AI agents, Focusa is the layer that makes them ship-worthy.
