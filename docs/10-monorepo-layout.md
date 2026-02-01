# docs/10-monorepo-layout.md — Focusa Monorepo Layout (MVP)

## Purpose

This document defines the **canonical monorepo structure** for Focusa, optimized for:
- local-first execution
- fast iteration
- strict separation of concerns
- future folding into NavisAI

This layout is **authoritative** for MVP implementation.

---

## Technology Stack (Locked)

| Layer | Technology |
|---|---|
| Core Runtime | Rust |
| IPC / API | Local HTTP (JSON) |
| CLI | Rust |
| UI | SvelteKit |
| Desktop Shell | Tauri |
| State Storage | Local filesystem (JSON + JSONL) |
| Task Memory | Beads |

---

## Repository Root

```
focusa/
├─ README.md
├─ PRD.md
├─ AGENTS.md
├─ Cargo.toml
├─ Cargo.lock
├─ package.json
├─ pnpm-workspace.yaml
├─ .gitignore
├─ .env.example
├─ docs/
├─ crates/
├─ apps/
├─ packages/
├─ scripts/
└─ data/
```

---

## `/crates` — Rust Core (Authoritative Cognition)

```
crates/
├─ focusa-core/
│  ├─ src/
│  │  ├─ lib.rs
│  │  ├─ runtime/
│  │  │  ├─ daemon.rs
│  │  │  ├─ session.rs
│  │  │  ├─ events.rs
│  │  │  └─ persistence.rs
│  │  ├─ focus/
│  │  │  ├─ stack.rs
│  │  │  ├─ frame.rs
│  │  │  └─ state.rs
│  │  ├─ intuition/
│  │  │  ├─ engine.rs
│  │  │  ├─ signals.rs
│  │  │  └─ aggregation.rs
│  │  ├─ gate/
│  │  │  ├─ focus_gate.rs
│  │  │  └─ candidates.rs
│  │  ├─ reference/
│  │  │  ├─ store.rs
│  │  │  ├─ artifact.rs
│  │  │  └─ gc.rs
│  │  ├─ expression/
│  │  │  ├─ engine.rs
│  │  │  ├─ serializer.rs
│  │  │  └─ budget.rs
│  │  └─ adapters/
│  │     ├─ mod.rs
│  │     ├─ openai.rs
│  │     ├─ letta.rs
│  │     └─ passthrough.rs
│  └─ Cargo.toml
│
├─ focusa-cli/
│  ├─ src/
│  │  ├─ main.rs
│  │  ├─ commands/
│  │  │  ├─ focus.rs
│  │  │  ├─ stack.rs
│  │  │  ├─ gate.rs
│  │  │  ├─ intuition.rs
│  │  │  ├─ refs.rs
│  │  │  └─ debug.rs
│  │  └─ output.rs
│  └─ Cargo.toml
│
└─ focusa-api/
   ├─ src/
   │  ├─ main.rs
   │  ├─ routes/
   │  │  ├─ session.rs
   │  │  ├─ focus.rs
   │  │  ├─ gate.rs
   │  │  ├─ intuition.rs
   │  │  └─ reference.rs
   │  └─ server.rs
   └─ Cargo.toml
```

**Rules**
- `focusa-core` owns all cognition
- CLI and API are thin facades
- No UI logic in Rust

---

## `/apps` — User-Facing Applications

```
apps/
├─ menubar/
│  ├─ src/
│  │  ├─ routes/
│  │  │  └─ +layout.svelte
│  │  ├─ components/
│  │  │  ├─ FocusBubble.svelte
│  │  │  ├─ FocusStackView.svelte
│  │  │  ├─ IntuitionPulse.svelte
│  │  │  ├─ GatePanel.svelte
│  │  │  └─ ReferencePeek.svelte
│  │  ├─ stores/
│  │  │  ├─ focus.ts
│  │  │  ├─ intuition.ts
│  │  │  └─ gate.ts
│  │  ├─ styles/
│  │  │  └─ tokens.css
│  │  └─ app.d.ts
│  ├─ src-tauri/
│  │  ├─ src/main.rs
│  │  └─ tauri.conf.json
│  └─ package.json
```

**Rules**
- UI is **read-mostly**
- No direct Focus State mutation
- All writes go through API

---

## `/packages` — Shared Frontend Code

```
packages/
├─ ui-tokens/
│  ├─ colors.ts
│  ├─ motion.ts
│  └─ hierarchy.ts
├─ api-client/
│  ├─ focus.ts
│  ├─ intuition.ts
│  └─ reference.ts
└─ types/
   ├─ focus.ts
   ├─ intuition.ts
   └─ gate.ts
```

---

## `/data` — Local Runtime State (Ignored by Git)

```
data/
├─ sessions/
├─ focus/
├─ reference/
├─ events/
└─ beads/
```

---

## NavisAI Compatibility

- `focusa-core` is embeddable
- API boundaries are stable
- UI can be subsumed later
- No architectural dead ends

---

## Summary

This monorepo:
- enforces cognitive boundaries
- supports fast iteration
- preserves long-term extensibility
- keeps Focusa small and precise
