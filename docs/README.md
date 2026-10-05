# Focusa Documentation Router

**Status:** CURRENT docs index  
**Purpose:** route humans and agents to the correct source without mixing current runtime truth, normative specs, evidence, public docs and historical design material.

Focusa is under active development. Do not infer current implementation from an old numbered spec or dated audit. Current source/runtime status and current agent guidance take precedence over historical design prose.

## Choose your entry point

| You are trying to… | Start here |
|---|---|
| Understand/install Focusa | [`../README.md`](../README.md) |
| Evaluate public capabilities | [`PUBLIC_INDEX.md`](PUBLIC_INDEX.md) |
| Work as an AI/build agent | [`agent/01-focusa-agent-docs-index.md`](agent/01-focusa-agent-docs-index.md) |
| Check what is implemented now | [`current/CURRENT_RUNTIME_STATUS.md`](current/CURRENT_RUNTIME_STATUS.md) |
| Use the CLI/API | [`current/CLI_REFERENCE_CURRENT.md`](current/CLI_REFERENCE_CURRENT.md), [`current/API_REFERENCE_CURRENT.md`](current/API_REFERENCE_CURRENT.md) |
| Discover Focusa tools | [`focusa-tools/README.md`](focusa-tools/README.md) |
| Understand current tool routing | [`current/FOCUSA_TOOL_CHOREOGRAPHY_MAP.md`](current/FOCUSA_TOOL_CHOREOGRAPHY_MAP.md) |
| Understand generated UI / Mission Canvas | [`135-series-current-manifest.md`](135-series-current-manifest.md) |
| Understand external product composition | [`65-visual-ui-focusa-integration.md`](65-visual-ui-focusa-integration.md) |
| Verify release/proof state | [`current/VALIDATION_AND_RELEASE_PROOF.md`](current/VALIDATION_AND_RELEASE_PROOF.md) |

## Current agent-awareness references

These names are also validated by repository tooling and should remain discoverable:

- [`current/AGENT_AWARENESS_QUICKSTART.md`](current/AGENT_AWARENESS_QUICKSTART.md) — **Agent Awareness Quickstart**.
- [`current/FOCUSA_AGENT_UTILITY_CARD.md`](current/FOCUSA_AGENT_UTILITY_CARD.md) — Focusa utility/startup card.
- [`current/FOCUSA_FRIENDLY_ONBOARDING.md`](current/FOCUSA_FRIENDLY_ONBOARDING.md) — **Friendly Focusa Q** onboarding.
- [`current/FOCUSA_TOOL_CHOREOGRAPHY_MAP.md`](current/FOCUSA_TOOL_CHOREOGRAPHY_MAP.md) — **Focusa Tool Choreography Map**.
- [`current/TOOL_RESULT_ENVELOPE_V1.md`](current/TOOL_RESULT_ENVELOPE_V1.md) — common tool-result/recovery envelope.

## Documentation classes

### Current runtime / operational truth

`docs/current/` contains maintained current-state guides, runtime status, references, security/runbook material and operational contracts. A file under `current/` may still describe a proposal if it says so explicitly; read its status header.

Key examples:

- [`current/CURRENT_RUNTIME_STATUS.md`](current/CURRENT_RUNTIME_STATUS.md)
- [`current/PRODUCTION_CONSISTENCY_POLICY.md`](current/PRODUCTION_CONSISTENCY_POLICY.md)
- [`current/UIAI_BROWSER_DIAGNOSTICS_FOCUSA_INTEGRATION_SPEC.md`](current/UIAI_BROWSER_DIAGNOSTICS_FOCUSA_INTEGRATION_SPEC.md)
- [`current/PROJECT_INTELLIGENCE_FLYWHEEL.md`](current/PROJECT_INTELLIGENCE_FLYWHEEL.md)

### Normative / numbered specifications

Numbered `docs/*.md` specifications define architecture, contracts or target behavior for their declared concern. They are not automatically implementation proof.

Use current manifests when a spec family has one. Examples:

- [`135-series-current-manifest.md`](135-series-current-manifest.md) — generated UI / professional workspace family.
- [`181-184-voice-foreman-radar-ambient-operator-current-manifest.md`](181-184-voice-foreman-radar-ambient-operator-current-manifest.md) — Voice, Foreman, Radar, Ambient Operator family.
- [`quantitative-scientific-cognition-series-current-manifest.md`](quantitative-scientific-cognition-series-current-manifest.md) — quantitative/scientific cognition family.

### Machine contracts

`docs/contracts/` contains generated and hand-maintained schemas, manifests, capability projections and ledgers. These are contract/projection truth for their declared surface, not proof that every consumer/runtime is deployed.

### Evidence

`docs/evidence/` contains bounded proof, audits and dated acceptance artifacts. Evidence supports a claim; it does not become current architecture or runtime state merely because it exists.

### Tool documentation

`docs/focusa-tools/` is the current human-facing index for `focusa_*` tool families and per-tool docs. Machine parity lives under `docs/contracts/spec141/generated-capability-v2/`.

### Public documentation

`PUBLIC_INDEX.md` is the public/evaluator router. Public docs must remain public-safe and should not be used as private operational authority.

### Historical / superseded material

Older design docs, audits and proposal provenance remain useful for lineage. When a current manifest, current-runtime guide, or explicit supersession statement exists, it outranks older prose. Do not delete history solely to make the tree prettier; label or route around it.

## Product-family composition boundary

Focusa is a reusable engine behind other product families. Wirebot/SOVOS may compose Focusa Context, Evidence, Prediction, Constraint, Trajectory, Workpoint, Metacognition, Verification, Receipt and trusted generated-UI primitives into product-specific experiences without moving those product semantics into Focusa.

See [`65-visual-ui-focusa-integration.md`](65-visual-ui-focusa-integration.md). The new SOVOS/Wirebot mathematical-intelligence and higher-order composition registries are consumer target-state work, not proof of current Focusa implementation.

## Documentation hygiene

Before creating a new document:

1. find the owning current guide/spec/manifest;
2. update that owner when possible;
3. create a new file only for a genuinely new bounded concern;
4. state whether it is current runtime truth, normative target/spec, implementation plan, dated evidence, compatibility path or historical lineage;
5. add it to the appropriate router/index if humans or agents need to discover it routinely;
6. never create a parallel authority, renderer, task store, Evidence store or status source merely to make documentation easier.

An index should route; it should not become another full specification.