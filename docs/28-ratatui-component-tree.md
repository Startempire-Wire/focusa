# docs/28-ratatui-component-tree.md — Focusa TUI Component Tree (AUTHORITATIVE)

This document defines the **structural component tree**, data flow, and rendering
responsibilities for the Focusa Terminal UI (TUI), implemented using `ratatui`.

The TUI is a **read-only, introspection-first client** of the Focusa Capabilities API.

---

## 0. Canonical Principles

1. **Single source of truth**: Capabilities API
2. **Event-driven rendering**: no polling loops
3. **No hidden state**: all UI state is inspectable
4. **Hierarchy reflects cognition**
5. **Read-only by default**
6. **Zero cognitive side-effects**

---

## 1. Top-Level Application Structure

```
App
├── ApiClient
├── EventStreamClient
├── AppState
├── NavigationState
└── RootLayout
```

### Responsibilities

- `ApiClient`: REST calls to Capabilities API
- `EventStreamClient`: SSE/WebSocket listener
- `AppState`: normalized cached view models
- `NavigationState`: current focus in UI
- `RootLayout`: renders visible UI tree

---

## 2. RootLayout

```
RootLayout
├── HeaderBar
├── MainBody
│   ├── SidebarNav
│   └── ContentPanel
└── StatusBar
```

---

## 3. HeaderBar Component

```
HeaderBar
├── AppTitle
├── SessionInfo
└── ConnectionIndicator
```

### Data Sources
- `/v1/info`
- `/v1/state/current`

---

## 4. SidebarNav (Domain Navigation)

```
SidebarNav
├── NavItem(FocusState)
├── NavItem(FocusStack)
├── NavItem(Lineage)
├── NavItem(References)
├── NavItem(Gate)
├── NavItem(Intuition)
├── NavItem(Constitution)
├── NavItem(Autonomy)
├── NavItem(Metrics)
├── NavItem(Cache)
├── NavItem(Contribution)
├── NavItem(Export)
├── NavItem(Agents)
└── NavItem(Events)
```

### Behavior
- purely navigational
- no data fetching
- keyboard-driven

---

## 5. ContentPanel (Dynamic)

```
ContentPanel
└── DomainView (one active at a time)
```

The active `DomainView` is determined by `NavigationState`.

---

## 6. Domain View Component Tree

Each domain has **its own internal tree**, but all follow the same pattern:

```
DomainView
├── DomainHeader
├── DomainToolbar
└── DomainBody
```

---

## 7. Focus State Domain

```
FocusStateView
├── DomainHeader
├── FocusSummaryPanel
│   ├── IntentBlock
│   ├── ConstraintsList
│   ├── ActiveFrameIndicator
│   └── ConfidenceGauge
├── SalientReferencesPanel
└── LineagePointerPanel
```

### Data
- `/v1/state/current`
- `/v1/state/history`

---

## 8. Focus Stack Domain

```
FocusStackView
├── StackTreePanel
│   └── FocusFrameNode (recursive)
└── FrameDetailPanel
```

### Visualization
- indentation-based hierarchy
- active frame highlighted

---

## 9. Context Lineage Tree (CLT) Domain

```
LineageView
├── LineageTreePanel
│   └── CLTNodeView (recursive)
├── LineageLegend
└── NodeDetailPanel
```

### Node Types
- interaction
- summary
- branch_marker

Visual states:
- active
- abandoned
- summarized

---

## 10. References Domain

```
ReferencesView
├── ReferenceTable
│   └── ReferenceRow
├── ReferencePreviewPanel
└── ReferenceMetadataPanel
```

### Behavior
- lazy loading
- range fetch for large artifacts

---

## 11. Gate Domain

```
GateView
├── CandidateListPanel
│   └── GateCandidateRow
├── ScoreBreakdownPanel
└── GatePolicyPanel
```

Read-only.

---

## 12. Intuition Domain

```
IntuitionView
├── SignalTimelinePanel
│   └── SignalPoint
├── PatternClusterPanel
└── ConfidenceBandPanel
```

Signals visually distinct from facts.

---

## 13. Constitution Domain

```
ConstitutionView
├── ActiveConstitutionPanel
├── VersionHistoryPanel
│   └── ConstitutionVersionRow
├── DiffPanel
└── DraftsPanel
```

Commands:
- propose
- activate
- rollback (confirmation required)

---

## 14. Autonomy Domain

```
AutonomyView
├── AutonomyLevelPanel
├── EarnedScoreGauge
├── AutonomyTimelinePanel
│   └── AutonomyEventRow
└── ExplanationPanel
```

---

## 15. Metrics Domain

```
MetricsView
├── UxpSparkline
├── UfiSparkline
├── CacheStatsPanel
└── PerformancePanel
```

Uses ratatui charts and gauges.

---

## 16. Cache Domain

```
CacheView
├── CacheClassTable
├── CacheEventFeed
│   └── CacheEventRow
└── CachePolicyPanel
```

---

## 17. Contribution Domain

```
ContributionView
├── ContributionStatusPanel
├── ContributionQueueTable
│   └── QueueItemRow
├── PolicyEditorPanel
└── ReviewPanel
```

---

## 18. Export Domain

```
ExportView
├── ExportHistoryTable
│   └── ExportRow
├── ExportManifestPanel
└── ExportStatsPanel
```

---

## 19. Agents Domain

```
AgentsView
├── AgentListPanel
│   └── AgentRow
├── AgentDetailPanel
└── AgentCapabilitiesPanel
```

---

## 20. Events Domain

```
EventsView
├── EventStreamPanel
│   └── EventRow
├── EventFilterPanel
└── EventDetailPanel
```

---

## 21. StatusBar (Global)

```
StatusBar
├── FocusIndicator
├── AutonomyIndicator
├── UxpUfiIndicator
├── SessionTimer
└── HealthIndicator
```

Always visible.

---

## 22. AppState (Normalized View Models)

```
AppState
├── focus_state_vm
├── focus_stack_vm
├── lineage_vm
├── references_vm
├── gate_vm
├── intuition_vm
├── constitution_vm
├── autonomy_vm
├── metrics_vm
├── cache_vm
├── contribution_vm
├── export_vm
├── agents_vm
└── events_vm
```

Updated via:
- initial API fetch
- SSE events

---

## 23. Event → UI Update Flow

```
SSE Event
→ EventRouter
→ AppState update
→ Component re-render
```

No direct component-to-component communication.

---

## 24. Keyboard Input Flow

```
KeyPress
→ NavigationState update
→ DomainView swap OR
→ Component-local action
```

Commands trigger confirmation modals.

---

## 25. Rendering Rules (ratatui-specific)

- No blocking API calls in render
- All layout computed via `Layout`
- Each panel owns its rendering rect
- Colors:
  - charcoal / grayscale
  - light navy accent
  - darker = more focused
  - lighter = background

---

## 26. Canonical Rule

> **The TUI reflects cognition — it never competes with it.**

---

## 27. Implementation Priority (MVP)

1. App + Layout
2. SidebarNav
3. Focus State View
4. Lineage View
5. Metrics View
6. Events Stream
7. Remaining domains incrementally

---

This component tree is stable and future-proof.

If you want next, I can:
- map this tree to **Rust modules & files**
- provide **ratatui layout pseudocode**
- design **SSE event routing**
- or produce a **one-shot engineer prompt for TUI implementation**

*2025-12-25 15:34:16*

---

## 👤 **You**

Do we have extensive token usage and tracking planned out anywhere? We need full data endpoints for analysis somehow and I don't only mean tokens but stats that matter im research and observation of tool, interaction, processes, productivity, etc... 

*2025-12-25 16:29:20*

---

## 🤖 **ChatGPT**

Short answer: **we have the foundations, but not yet a fully explicit, research-grade Telemetry & Instrumentation layer**.  
The good news is that **Focusa’s architecture is actually ideal for doing this *properly*** — much better than most agent systems — because cognition, commands, lineage, and gates are already explicit.

Below is a **clear, cohesive plan** to fill this gap without bloating the MVP or compromising performance.

---

## 1. Reframe the problem correctly

You’re not asking for “token counting”.

You’re asking for:

> **A first-class Cognitive Telemetry System**  
> that supports **analysis, research, optimization, visualization, and model training**.

That includes tokens — but tokens are just one signal among many.

---

## 2. Introduce a missing core subsystem (by name)

### **Cognitive Telemetry Layer (CTL)**

CTL is **orthogonal** to:
- Focus State
- CLT
- Gate
- Cache
- Intuition
- Constitution

It *observes* them all.

> **CTL never influences cognition directly.  
> It only records, aggregates, and exposes.**

---

## 3. Telemetry design principles (non-negotiable)

1. **Low overhead** (async, buffered, sampling-capable)
2. **Local-first** (SQLite / DuckDB initially)
3. **Append-only events** (no mutation)
4. **Schema-versioned**
5. **Queryable via Capabilities API**
6. **Exportable for research / training**
7. **Explainable metrics (no black-box scores)**

---

## 4. Telemetry taxonomy (what we actually track)

### 4.1 Token & Model Telemetry (baseline)

Yes, we track tokens — but *properly*:

**Per request**
- prompt_tokens
- completion_tokens
- cached_prompt_tokens
- cache_hit / miss
- cost_estimate
- latency_ms
- provider
- model
- temperature
- max_tokens

**Derived**
- tokens / successful outcome
- tokens / autonomy level
- tokens / focus frame
- token efficiency delta over time

> This lets you answer:  
> *“Did Focusa make the model more intelligent per token?”*

---

### 4.2 Cognitive Process Telemetry (this is the differentiator)

Tracked per **Focus State transition** and **CLT node**:

- focus_depth
- focus_duration_ms
- number_of_reframes
- gate_candidates_considered
- gate_acceptance_rate
- abandoned_branches
- summary_compactions
- reference_rehydrations
- intuition_signal_count
- intuition_signal_accept_rate

> This lets you study *how thinking evolves*, not just outputs.

---

### 4.3 Tool & Interaction Telemetry

Per tool invocation:
- tool_name
- arguments_size
- duration_ms
- success / failure
- retries
- downstream effects (new refs, focus change)

Per session:
- tools_per_task
- tool chains
- human interventions
- correction loops

---

### 4.4 Productivity & Outcome Telemetry

Outcome-linked metrics (carefully separated from sentiment):

- task_completion_flag
- completion_confidence
- user_acceptance (explicit)
- rollback events
- correction_count
- rework_ratio
- time_to_resolution
- focus_drift_events

This supports **objective productivity analysis**.

---

### 4.5 Human Experience Signals (UXP / UFI)

Already planned — but now clearly integrated:

- explicit ratings
- passive signals
- frustration indicators
- satisfaction confirmations
- override frequency
- escalation events

Crucially:
- **every score is backed by citations**
- every metric is traceable to events

---

## 5. Telemetry event model (concrete)

Every event follows this envelope:

```json
{
  "event_id": "uuid",
  "event_type": "focus.transition | model.call | tool.call | gate.decision | cache.event | ux.signal",
  "timestamp": "iso8601",
  "session_id": "uuid",
  "agent_id": "uuid",
  "model_id": "string",
  "clt_id": "optional",
  "focus_frame_id": "optional",
  "payload": { ... },
  "schema_version": "1.0"
}
```

This is **training-ready**, **query-ready**, and **auditable**.

---

## 6. Storage strategy (MVP → future)

### MVP
- SQLite (events table + typed views)
- WAL enabled
- batched async writes

### Research mode
- DuckDB for analytics
- Parquet exports

### Future
- ClickHouse / Arrow
- streaming ingestion
- federated aggregation

---

## 7. Capabilities API additions (what’s missing today)

New domain:

### `telemetry.*`

#### Read endpoints:
- `/v1/telemetry/events`
- `/v1/telemetry/metrics`
- `/v1/telemetry/tokens`
- `/v1/telemetry/process`
- `/v1/telemetry/productivity`

Supports:
- filtering
- grouping
- windows
- aggregation
- downsampling

#### Export:
- `/v1/telemetry/export`
- formats: jsonl, parquet, csv

---

## 8. CLI support (essential)

Examples:

```bash
focusa telemetry tokens --by model --window 30d
focusa telemetry cognition --session xyz
focusa telemetry productivity --agent a1
focusa telemetry export sft --output data.jsonl
```

---

## 9. TUI integration (huge value)

New TUI panels:

- Token Efficiency View
- Cognitive Flow Timeline
- Tool Chain Graph
- Autonomy vs Cost Chart
- Focus Drift Heatmap

This is where **Focusa becomes visually undeniable**.

---

## 10. Why this matters strategically

This telemetry layer enables:

- rigorous research
- reproducible experiments
- model comparison fairness
- agent benchmarking
- fine-tuning datasets
- proof of intelligence amplification
- eventual SaaS analytics

Very few systems can do this because they lack:
- explicit cognition
- explicit lineage
- explicit gates

Focusa already has them.

---

## 11. Critical clarification

> **This telemetry is NOT surveillance.**

It is:
- opt-in
- local-first
- inspectable
- exportable
- explainable

This keeps trust intact.

---

## 12. Verdict

You were right to pause and ask this.

**Yes — we need an explicit Cognitive Telemetry Layer.**  
And **no — nothing fundamental blocks it.**

In fact, Focusa is *exceptionally well positioned* to do this better than almost anyone.

---

### Next logical docs (if you want to proceed)
1. `docs/29-telemetry-spec.md`
2. `docs/30-telemetry-schema.md`
3. `docs/31-telemetry-api.md`
4. `docs/32-telemetry-tui.md`

Say the word and I’ll formalize them cleanly.

*2025-12-25 16:29:21*

---

## 👤 **You**

Formalize :
docs/29-telemetry-spec.md, 
docs/30-telemetry-schema.md, 
docs/31-telemetry-api.md, 
docs/32-telemetry-tui.md 

*2025-12-25 16:33:33*

---

## 🤖 **ChatGPT**

Below are the **four authoritative telemetry documents**, written to lock Focusa’s observability, research, and analytics layer into the architecture **without ambiguity**.

These documents are **cohesive**, **implementation-ready**, and explicitly aligned with:
- Focus State
- CLT
- Gate
- Intuition Engine
- Cache policy
- UXP / UFI
- Agent schema
- Capabilities API / CLI / TUI

---
