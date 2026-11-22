# Cross-Vertical Focusa × UIAI Embedded Intelligence Experiments

**Status:** experimental extension to the existing 18-task Spec 114 benchmark, unmeasured.  
**Source:** `SOVOS_PAPER_CROSS_VERTICAL_PORTABILITY_STUDY_2026-11-22.json` (12 case designs, not a fixed future domain cap).  
**Research consumers:** SOVOS preprint PR #49 and *The Entrepreneur's Path* appendix PR #104.  
**Owning primary handoff:** `SOVOS_PAPER_EXPERIMENT_SERVER_AGENT_HANDOFF_2026-11-22.md` in Focusa PR #731.

## A. Scientific thesis and software meaning

Focusa is a reusable **governed cognitive runtime**, not a coding-specific assistant prompt. Its domain-neutral primitives include intention/commitment, verified scope, Workstreams, Workpoints, temporal and epistemic authority, Evidence, Receipts, metacognition, session continuity and bounded learning. UIAI Engine is an independently governed **computer observation/action/verification engine**, not a new Focusa memory or a replacement for domain-native APIs. Product runtimes (OpenClaw, Pi and others) provide agent inference and delegation. The owning product supplies its domain meaning, source data, authoritative state, deterministic operations and accepted-outcome oracle.

The research hypothesis is that **the same reference primitives can be reused across applications and time horizons**, with a measurable integration contribution above what a capable agent or a browser alone provides.

Do **not** implement a new combined daemon, global source database or universal permission store to prove this. Portable behavior comes from contracts and independently owned runtime seams. For any source product with an adequate typed API, direct API/CLI wins over browser use. UIAI matters where external application UI, research/vision, or browser evidence is actually required.

## B. Domain examples — source-reviewed, different maturity

| Vertical | Domain-owned truth | Focusa role | UIAI role | Inspectable current state |
|---|---|---|---|---|
| Wirebot | Human-owner relationship and qualified owner-world source projections | Cognition, continuity, authority, Evidence | Browser action/verification when needed | Private Core/App integration is source-defined; exact deployment/effect acceptance must be checked |
| WorkRouter | Opportunity, source, route, delivery, provider and market status | Trusted temporal/epistemic/semantic primitives and routed work continuity | Agent browser read and approved computer work | Actual UIAI client and Focusa conformance code exist; conformance claims remain implemented-unverified |
| Wirebase | Companies, licensed sources, entity resolution, counterparty reports, acquisition opportunity | Research task lifecycle, evidence promotion, scope and learning | Approved browsing/OSINT source observation where APIs inadequate | Design/specification only; no deployed production application |
| Focusa Market Lab | Frozen information sets, markets, strategy, broker/risk/accounting/settlement | Time, prediction, self-evaluation, Workpoint, receipts, learning | Research, human oversight and browser evidence outside hot market path | Detailed normative design; initial slice exists; real 24h colony acceptance not demonstrated |

The detailed product repos are private; an externally repeatable paper must publish *synthetic* domain adapters/fixtures and independent oracle code or use the already-public Focusa/UIAI portions. The research may describe private source reviewed at specific commits but must not claim that inaccessible tests are independently reproducible.

## C. Distinguish availability from continuous inference

Twenty-four-hour autonomous operation is **a managed obligation**, not 24 hours of nonstop language-model inference. Test how the system preserves goals, listens to qualified changes and deadlines, recovers from model/provider outages, respects expiry and spends compute only when useful. Market Lab's two-timescale architecture is instructive: a persistent agent identity has a durable mailbox/constitution and uses leased inference workers when needed; precompiled, versioned reflexes handle time-critical effects inside deterministic risk constraints.

Measure: completed scheduled/conditional obligations, missed/late events, false attentions, stale actions prevented, source-state recovery, unplanned model calls, process RSS/CPU, cost per accepted outcome and owner interventions. A fast simulated clock tests state-machine invariants but is **not** a real 24-hour uptime claim. A real 24-hour synthetic-only monitored exercise with source/agent traces is a separate result.

No live trade, broker order, regulated filing, third-party outreach, purchasing, credential use or customer-data scan is authorized by this study.

## D. Research questions and discriminating controls

**R1 — Reuse:** Do the same Focusa primitive contracts and server operations function under distinct domain adapters without duplicating canonical domain state? Measure code/contract reuse, cross-domain compatibility failures and per-domain source-verified task success.

**R2 — Interaction value:** With the same underlying model and equivalent task affordances, what incremental contribution do Focusa governance/continuity and UIAI verified interaction each make to accepted outcomes? Only compare feasible arms: `baseline`, `+Focusa`, `+UIAI`, `+Focusa+UIAI`. For nonbrowser market hot paths UIAI is **not applicable**; do not invent an artificial browser dependency.

**R3 — Continuity:** Under compaction, runtime migration and worker replacement, does a stable cognitive session/Workstream preserve the exact goal and external-result record? Keep ad hoc as a valid route before formal promotion. Distinguish session identities, Workstream ownership and partner identity.

**R4 — Self-knowledge:** Does source-qualified capability/competence introspection reduce invalid tool selection and false claims when tools, model version, grants, budgets and sensors change? A missing unique `self-knowledge` skill file is a discovery target, not license to build a new authority store.

**R5 — Bounded independent observation:** Does UIAI's evidence distinguish the actual source effect from page appearance, read-only research from formal Evidence, and changed targets from stale action? WorkRouter's present browser adapter creates a plausible `uiai-browser:session=<id>:read` string; source-owned handle validity must be independently resolved rather than inferred from its shape.

**R6 — Functional integration:** Does source-qualified reciprocal feedback (verified outcome → correction/learning → next action) outperform one-way read, common-input-only and no-memory-return controls? **This is an operational causal intervention, not IIT `Phi`**. Physical-substrate IIT 4.0 toy experiments remain separate and version-pinned.

## E. Initial synthetic case set

The machine-readable manifest defines three source-specific task designs per vertical: `WB1-3`, `WR1-3`, `DB1-3`, `ML1-3`. Each must gain a resettable synthetic fixture, current call references and independent oracle before execution. Wirebase cases are design fixtures until a lawful source adapter exists. Market Lab uses synthetic financial events only and preserves deterministic order/risk constraints.

Benchmark agents must report both *observed successes and failures*. The requested PR shall include actual task/arm/repetition rows, source outcomes, baseline/augmented configuration digests, measured results, 95% intervals where defined, nonapplicable cells, and redaction-approved reproducibility artifacts. Do not substitute synthetic proposed values for observations.

## F. Figures and third-party participation

Generate figures from actual result rows:
- across-vertical outcome/delta with paired uncertainty and sample denominators;
- cross-domain transfer tests (knowledge/competence improvements **without leaking domain data**);
- 24-hour heartbeat/missed-event and model-use timeline;
- cost/quality/attention frontier, independently graded;
- causal reciprocal feedback ablation with a common-input-only control.

Include a source-owned replay graph linking owner objective, exact Focusa ad hoc/project binding, UIAI observation if used, approved execution, independent domain source effect, Evidence/Receipt, and revisited goal. A scientific reviewer should be able to reproduce at least one failure and one success from a synthetic fixture, without needing access to private customers.

A figure displaying conceptual boxes and arrows must be labeled *architecture*; a matrix with declared test arms must be labeled *experiment design*; a quantitative figure requires actual rows and an accessible scorer. Never call hypothetical bars measured data.

## G. Agent response

Deliver primary Spec 114 experiment implementation/results as required by the parent handoff, then report cross-vertical cases through the **same experimental result owner**, with links to any domain-owned consumer PRs only where the implementation changes those products. Preserve existing source writers and work graphs. A study can be partially measured by vertical; mark missing tests as `unmeasured`, `incomparable`, `unavailable` or `not_applicable` with explicit reason.

Expected response comment on Focusa PR #731: actual benchmark/result PR URL, exact source SHA, table of tested verticals and cases, evidence digest, independent oracle/reviewer status, measured figures or a clear unmeasured report. This handoff does not itself start an agent on a server.
