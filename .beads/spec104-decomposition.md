# Spec 104 Implementation Beads

## P1: Singleton elimination continued (21 items)

### PI-06 — Report/evidence singleton removal
- priority: P1
- parent: focusa-n68k
- description: state.ts: remove lastReportSummary from singleton; derive from scoped workpoint only

### PI-07 — Drive/turn state removal
- priority: P1
- parent: focusa-n68k
- description: state.ts: remove drive/turn state from singleton authority; turn state from scoped source only

### PI-11 — Tool bridge state removal
- priority: P1
- parent: focusa-n68k
- description: state.ts: remove tool_request/turn_count from singleton; tool bridge state scope-keyed

### PI-14 — Frame recovery scope cleanup
- priority: P1
- parent: focusa-n68k
- description: state.ts: packet-driven scope with blocked mismatch; frame recovery blocks on mismatched root

### PI-15 — Persisted bridge state non-canonical
- priority: P1
- parent: focusa-n68k
- description: state.ts: adapter cache only, never canonical authority; persisted bridge cannot be replayed

### PI-C01 — Session bootstrap scope hardening
- priority: P1
- parent: focusa-n68k
- description: session.ts: bootstrap from typed scoped authority; cold session creates correct typed identity

### PI-C02 — Session lifecycle restore hardening
- priority: P1
- parent: focusa-n68k
- description: session.ts: resume only from canonical typed packets; switching session never revives prior workpoint

### PI-C03 — Restore logic scoped pipeline
- priority: P1
- parent: focusa-n68k
- description: session.ts: scoped restore pipeline; no continuity-only trajectory restore

### PI-C04 — Awareness singleton removal
- priority: P1
- parent: focusa-n68k
- description: awareness.ts: render from typed scoped packets; awareness on mismatch shows blocked scope

### PI-C05 — Compaction singleton removal
- priority: P1
- parent: focusa-n68k
- description: compaction.ts: derive from canonical typed resume packets; compaction after scope switch never emits prior project

### TOOL-03 — Project identity tool scope enforcement
- priority: P1
- parent: focusa-n68k
- description: tools.ts: focusa_project_identity from ScopeRef only; broad cwd returns blocked

### TOOL-06 — Session identity builder scope
- priority: P1
- parent: focusa-n68k
- description: tools.ts: central typed builder with no singleton authority; canonical tools fail closed

### TOOL-08 — Agent prompt tool scope
- priority: P1
- parent: focusa-n68k
- description: tools.ts: focusa_agent_prompt from typed bootstrap; broad cwd returns blocked scope

### TOOL-09 — Utility card/post-compaction scope
- priority: P1
- parent: focusa-n68k
- description: tools.ts: utility_card from typed bootstrap packets; post-compaction card never emits prior root

### TRAJ-01 — HLT scoping hardening
- priority: P1
- parent: focusa-n68k
- description: trajectory: HLT root-scoped, not continuity-scoped; continuity churn preserves HLT

### TRAJ-02 — HLT bootstrap placeholder isolation
- priority: P1
- parent: focusa-n68k
- description: trajectory: generic bootstrap never canonical; valid historical HLT outranks generic

### TRAJ-03 — Trajectory evidence ref scoping
- priority: P1
- parent: focusa-n68k
- description: trajectory: evidence refs require root/scope match; Pi/menubar never surfaces generic HLT as canonical

### API-09 — Ontology read index scope-keying
- priority: P1
- parent: focusa-n68k
- description: ontology.rs: scope-keyed ONTOLOGY_READ_INDEX; ontology query for root A never returns root B data

### MW-01 — Auth middleware scope preservation
- priority: P1
- parent: focusa-n68k
- description: auth.rs: preserve typed scope through auth; authenticated requests preserve exact scope

### MW-02 — Error envelope scope preservation
- priority: P1
- parent: focusa-n68k
- description: error_envelope.rs: blocked/advisory in error envelopes; mismatch returns machine-readable blocked

### MW-03 — JSON guard scope rejection
- priority: P1
- parent: focusa-n68k
- description: json_guard.rs: reject malformed scope_kind; malformed payload fails before route execution

## P2: Consumer surface scope migration (18 items)

### API-08 — Device pairing runtime classification
- priority: P2
- parent: focusa-nodn
- description: device_pairing.rs: classify as runtime infra, not project authority

### BND-01 — Resource mode/pressure globals scope-keying
- priority: P2
- parent: focusa-nodn
- description: bounded.rs: scope-keyed runtime mode/pressure stores

### PI-C06 — Polish/reports singleton removal
- priority: P2
- parent: focusa-nodn
- description: polish.ts: derive from scoped workpoint packet only

### PI-C07 — Config/prompts typed identifiers
- priority: P2
- parent: focusa-nodn
- description: config.ts: typed prompt/packet surface identifiers

### TOOL-07 — Tool contract registry scope fields
- priority: P2
- parent: focusa-nodn
- description: tool-contracts.ts: require scope/authority in contract schema

### TOOL-10 — Pi plugin export surface scope
- priority: P2
- parent: focusa-nodn
- description: commands.ts/config.ts: typed scope on exported surfaces

### MEN-01 — Project context helper typed scope
- priority: P2
- parent: focusa-nodn
- description: projectContext.svelte.ts: consume one typed ScopeContext

### MEN-02 — Page bootstrap typed source
- priority: P2
- parent: focusa-nodn
- description: +page.svelte: one typed source of truth

### MEN-03 — API bridge typed identity
- priority: P2
- parent: focusa-nodn
- description: api.ts: use typed envelopes; remove string workspace_id

### MEN-04 — Cockpit view typed scope
- priority: P2
- parent: focusa-nodn
- description: CockpitView.svelte: render typed scope context

### MEN-05 — Context Cognition Peek typed scope
- priority: P2
- parent: focusa-nodn
- description: ContextCognitionPeek.svelte: typed scope only

### MEN-06 — Trajectory Peek typed scope
- priority: P2
- parent: focusa-nodn
- description: TrajectoryPeek.svelte: advisory-only trajectory

### MEN-07 — Work-loop Peek typed scope
- priority: P2
- parent: focusa-nodn
- description: WorkLoopPeek.svelte: typed scope with advisory state

### MEN-08 — Workpoint Peek typed scope
- priority: P2
- parent: focusa-nodn
- description: WorkpointPeek.svelte: typed scope only

### WL-02 — Work-loop consumer scope rendering
- priority: P2
- parent: focusa-nodn
- description: TUI/menubar: display typed scope + advisory/blocked status

### MW-04 — Route-family scope enforcement
- priority: P2
- parent: focusa-nodn
- description: all route families: accept/preserve typed host/project scope

### TUI-01 — focusa-tui typed scope rendering
- priority: P2
- parent: focusa-nodn
- description: TUI api/app/views: typed scope display

### MBN-01 — Menubar native Tauri bridge scope
- priority: P2
- parent: focusa-nodn
- description: main.rs: scope-bearing messages preserve typed scope

## P3: Tests, contracts, benchmarks, docs (9 items)

### API-10 — Proxy client infra classification
- priority: P3
- parent: focusa-84px
- description: proxy.rs: classify as infra-only singleton

### BEN-01 — Bench scope contamination proof
- priority: P3
- parent: focusa-84px
- description: bench/eval: typed run scope + arm scope; ON/OFF runs isolated

### BEN-02 — Benchmark integrity anti-bleed
- priority: P3
- parent: focusa-84px
- description: repeated ON/OFF runs isolated; no hidden bleed

### BEN-03 — Public proof lineage
- priority: P3
- parent: focusa-84px
- description: proof surfaces: immutable typed proof lineage

### DOC-01 — Singleton audit test
- priority: P3
- parent: focusa-84px
- description: static audit: hard singleton-surface sweep; test fails on new global

### DOC-02 — Session identity envelope test
- priority: P3
- parent: focusa-84px
- description: static test: require no singleton authority deps

### DOC-03 — Mismatch semantic test extension
- priority: P3
- parent: focusa-84px
- description: mismatch test covers all Pi/tools/menubar/adapters/TUI

### DOC-04 — Tool contract schema audit
- priority: P3
- parent: focusa-84px
- description: tool-contracts.ts: scope/authority contract checks

### DOC-05 — Scope hard-stop alignment
- priority: P3
- parent: focusa-84px
- description: align hard-stop doc + runtime; live path matches documented envelope
