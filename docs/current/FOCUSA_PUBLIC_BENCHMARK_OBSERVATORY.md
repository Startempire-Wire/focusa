# Focusa Public Benchmark Observatory

Status: current implementation contract for Spec 114 Phase 0  
Primary spec: `docs/114-public-benchmark-flywheel-spec.md`  
Extends: `docs/113-agent-benchmark-spec.md`

## Purpose

The Focusa Public Benchmark Observatory is the public-safe evidence surface for answering one market question:

> Same agent, same task: how much better is the agent with Focusa than without Focusa?

The default public story is always **Focusa-vs-No-Focusa**. Diagnostic ablations explain why Focusa helps, but never replace the primary comparison.

## Public Domains

| Domain | Role | Boundary |
|--------|------|----------|
| `bench.focusa.dev` | Public observatory and benchmark leaderboard | Static/public-safe read model by default |
| `evals.focusa.dev` | Technical eval docs and methodology | May reference internal run IDs, not raw private payloads |
| `proof.focusa.dev` | Immutable public proof bundles | Redacted snapshots and evidence receipts only |

Local daemon `/v1/evals/*` is not exposed directly to the public internet.

## Existing Site Infra Pattern

Observed Focusa sites are all under cPanel user `focusadev`:

| Site | Pattern |
|------|---------|
| `focusa.dev` | WordPress on LiteSpeed/cPanel, docroot `/home/focusadev/public_html` |
| `arena.focusa.dev` | WordPress/custom theme, public-safe receipt positioning |
| `forge.focusa.dev` | WordPress child theme + Gravity Forms/Stripe |
| `engine.focusa.dev` | WordPress child theme for UIAI Engine marketing |
| `perpetua.focusa.dev` | Exception: SvelteKit static frontend plus local Go API proxied by `.htaccess` to `127.0.0.1:8090/api/*` with `X-Agent-Key` auth |

Deployment decision:

1. MVP `bench.focusa.dev` should follow the simple cPanel/WordPress/static-artifact pattern when the site only needs public-safe benchmark pages and JSON snapshots.
2. If interactive replay search, signed run lookup, dynamic OG images, or agent-facing proof services are required, use the Perpetua hybrid pattern: static frontend + local Go API behind cPanel/LiteSpeed proxy.
3. Public APIs serve generated/redacted artifacts only; internal Eval Ledger endpoints remain private.
4. Deploys must run as cPanel user `focusadev` or immediately restore ownership with `fix-user-perms focusadev`; public benchmark deployment must not create root-owned files in `/home/focusadev`.

## Required Benchmark Arms

Every public report must include:

1. `no_focusa` — raw harness baseline.
2. `full_focusa` — complete Focusa runtime.

Diagnostic arms:

3. `passive_focusa` — docs/prompts only.
4. `tool_only_focusa` — `focusa_*` tools only.

## Public Report Sections

A public snapshot must include:

- Focusa-vs-No-Focusa headline card.
- LLM model × scenario uplift matrix.
- Agent Power Index and Focusa Uplift Score.
- Operator Burden Reduction.
- Category breakdown across Spec 113 L1-L12 tasks.
- Failure-to-fix board.
- Task replay theater with public-safe event timeline.
- Honesty rail: limitations, inconclusive runs, regressions, private holdout boundary.
- Evidence refs, scoring commit, environment digest, model matrix digest.

## Public Snapshot Sources

Recommended generated artifacts:

```text
public/bench/latest.json
public/bench/snapshots/<snapshot_id>.json
public/bench/releases/<focusa_version>.json
public/bench/models/<model_id>.json
```

These artifacts are generated from completed Eval Ledger runs and redaction-approved snapshots.

## Public Snapshot Gate

A snapshot may publish only when all are true:

```text
publish_allowed = true
redaction_status = passed
secret_scan_status = passed | not_required_no_raw_payload
evidence_refs_public_safe only
no raw logs
no raw token payloads
no raw private prompts
no private file contents
no sensitive browser diagnostics
no unredacted project paths
no raw diffs unless explicitly public-safe
private holdout bodies excluded
```

## Claim Rules

Valid claim template:

> On `focusa-agent-bench-vX`, Focusa improved `<metric>` from `<no_focusa>` to `<full_focusa>` versus the No-Focusa baseline (`Δ=<delta>`, 95% CI `<ci>`), using `<model/version>` across `<n>` matched trials. Raw artifacts: `<evidence_ref>`.

Invalid claims:

- “Focusa makes all LLMs better.”
- “Focusa is 2x better” without run ID, model version, CI, and evidence refs.
- Claims derived from predicted values rather than completed Eval Ledger runs.

## Non-Negotiables

- Measured claims only.
- Deny-by-default public display.
- Failures are product assets.
- Private holdout tasks are never exposed.
- Eval writes go through `/v1/evals/*`; telemetry remains read/export-only.
- Public proof bundles prefer handles and aggregates over raw logs.
