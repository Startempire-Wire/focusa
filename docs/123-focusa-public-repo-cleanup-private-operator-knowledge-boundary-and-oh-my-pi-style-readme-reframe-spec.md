# Spec 123 — Public Repo Cleanup, Private Operator Knowledge Boundary, and Oh-My-Pi-Style README Reframe

**Status:** Proposed
**Scope:** Public repo cleanup + private local knowledge layout + README/product-presentation restructuring
**Repo:** `Startempire-Wire/focusa`
**No-commit directive:** This spec is planning/execution guidance only. Do not commit while drafting or running exploratory commands.
**Primary outcome:** Make Focusa look like a professional public product repo while preserving full local/private access for agents through `.focusa-private/`.
**Boundary with Spec 124:** This spec owns public-facing repo presentation, public/private docs separation, and README framing; Spec 124 owns the functional CLI/onboarding architecture.

---

## 0. Executive summary

Focusa’s codebase and docs now contain enough real product surface to present publicly, but the public repo currently mixes four things that should be separated:

1. **Public product proof**
2. **Current supported operator docs**
3. **Private strategy/commercial/vendor planning**
4. **Raw runtime/evidence/transcript artifacts**

Spec 123 creates a clean boundary:

- Public repo says:
  **Focusa is the local-first proof and continuity layer for AI coding agents.**

- Private local folder keeps:
  SaaS plans, SignalOS strategy, pricing/cap math, vendor/license details, raw transcripts, internal proof, install/purchase audits, launch planning, and agent-KB material.

This spec also introduces an `oh-my-pi`-inspired README layout:

> Hero screenshot → install → numbered visual feature sections → short overview snippets while scrolling.

For Focusa, that means the README should not feel like a dense spec index. It should feel like a confident product page with visual proof:

```text
01 · Resume after compaction
Overview text
Screenshot/capture

02 · Evidence that survives handoff
Overview text
Screenshot/capture

03 · Context Authority stops off-mission mutation
Overview text
Screenshot/capture
```

---

## 1. Current repo reality this spec is grounded in

### 1.1 README reality

Current `README.md` still has:

- CI / Release / Dev Release badges
- hardcoded version badge `0.9.25-dev`
- current snapshot version `v0.9.74-dev`
- quickstart that uses `scripts/install-daemon.sh`
- dense early vocabulary: ProjectIdentity, Continuity ID, HLT, MLG, STG, Waypoints, Workpoints, Evidence Refs, Context Cognition, Context Authority
- menubar correctly framed as preview/not flagship

Current issue:

```text
README says version badge: 0.9.25-dev
README snapshot says: v0.9.74-dev
README quickstart uses: bash scripts/install-daemon.sh /usr/local
```

Problem:

`install-daemon.sh` is a daemon deployer path, not the clean public install path.

### 1.2 Public strategy docs currently exposed

The public repo currently contains docs that expose future architecture and business strategy, especially:

```text
docs/115-focusa-cloud-control-plane-tool-gateway-master-spec.md
docs/SIGNALOS_TECHNICAL_SCOPE.md
docs/SPEC_119_LIFETIME_TO_RECURRING_TRANSITION.md
docs/118-focusa-license-tiers-spec.md
LICENSE-FAQ.md
```

These include cloud/control-plane plans, future domain maps, SignalOS positioning, pricing/cap math, lifetime limits, revenue ceiling, Founders Forge details, bundle mechanics, and transition strategy.

### 1.3 Public install/purchase audit docs currently exposed

The repo also contains public docs around install/purchase/vendor work:

```text
docs/INSTALL_PURCHASE_GAP_AUDIT_2026-07-07.md
docs/INSTALL_PURCHASE_ACHIEVEMENTS_2026-07-07.md
docs/INSTALL_PURCHASE_VENDOR_CLOSE_2026-07-07.md
docs/DISCREPANCIES_LEDGER_2026-07-07.md
```

These are useful internally, but publicly they expose vendor/backend details, registry behavior, dev-mode validation history, Stripe/license-row TODOs, support/admin URLs, and acceptance commands.

### 1.4 Public proof and transcript reality

The repo contains public proof and raw evidence that should be sanitized:

```text
release-proof/latest.json
release-proof/v0.9.74-dev.openproof.json
docs/evidence/FRESH_OPERATOR_DRY_RUN_2026-07-05.md
docs/evidence/GAPS_4_9_LIVE_PROOF_2026-07-05.md
docs/evidence/fresh-operator-dry-run-2026-07-05/
docs/evidence/freshop-q-gaps-4-9/
proof/claude-code-adapter-20260706-215713.txt
```

Issues include:

- local host paths
- `/home/wirebot`
- root prompts
- raw tmux transcripts
- production/VPS context
- stale binary drift details
- internal daemon paths
- raw operational proof better suited for private evidence

### 1.5 Agent instructions reality

There is currently:

```text
docs/AGENTS.md
```

But there is not a root-level:

```text
AGENTS.md
```

Root `AGENTS.md` is useful because many coding agents automatically look there first.

### 1.6 Runtime state reality

`.gitignore` already treats runtime/local state as local-only:

```text
/data/
.focusa/
.tmp/
focusa.sqlite*
focusa-daemon.lock
ecs/handles/
ecs/handles-index/
ecs/objects/
device-pairing/
hlt-ledger/
```

But committed runtime artifacts still appear to exist under `ecs/objects/`.

### 1.7 Oh-my-pi README pattern to copy

The admired `oh-my-pi` pattern is:

```text
Hero image
Centered one-line product claim
Badges
Install commands
Short capability statement
Numbered feature sections
Screenshot/capture per feature
Brief overview text per feature
```

Focusa should adopt the pattern, but with Focusa’s product truth:

```text
Proof-backed continuation
Evidence refs
Context Authority
Mission Deck/TUI
Pi extension
Local-first API
Menubar preview
Public proof cards
```

---

## 2. Product-presentation target

### 2.1 Public product sentence

Use this as the public repo’s plain-language center:

```text
Focusa is the local-first proof and continuity layer for AI coding agents.
```

### 2.2 Public pitch

```text
Keep AI coding agents on mission.

Focusa gives long-running AI coding work a durable Workpoint, linked Evidence, and a next safe action outside the fragile chat window.
```

### 2.3 Public promise

```text
When the model compacts, the session changes, or the next agent takes over, Focusa gives it the mission, proof, and continuation contract instead of making it guess from transcript memory.
```

### 2.4 What not to lead with

Do not lead with:

- HLT
- MLG
- STG
- ProjectIdentity
- Context Cognition
- Context Authority internals
- License-tier mechanics
- SaaS/cloud architecture
- SignalOS
- raw proof transcript details

Those can exist lower in the docs or private strategy.

---

## 3. New README layout inspired by oh-my-pi

### 3.1 README target outline

Replace the current dense top with this shape:

```md
<p align="center">
  <img src="docs/assets/readme/focusa-hero.png" alt="Focusa Mission Deck" width="900">
</p>

<h1 align="center">Focusa</h1>

<p align="center">
  <strong>Keep AI coding agents on mission.</strong><br>
  <sub>Local-first proof, Workpoints, Evidence, and continuation for long-running coding agents.</sub>
</p>

<p align="center">
  badges...
</p>

Focusa is the local-first proof and continuity layer for AI coding agents.

When a coding session gets long, context compacts, the mission drifts, proof gets buried, or another agent takes over, Focusa preserves the work as a proof-backed Workpoint with linked Evidence and a next safe action.

## Install

```bash
curl -fsS https://install.focusa.dev/focusa | bash -s -- --eval
focusa start
focusa init --quickstart
focusa doctor
```

## Five-minute proof

```bash
focusa workpoint checkpoint --title "First Focusa proof"
focusa evidence link <workpoint-id> "test: cargo test --workspace"
focusa workpoint resume --copy-prompt
```

## The Focusa you can use today

### 01 · Resume after compaction

![Focusa resume packet](docs/assets/readme/01-resume-after-compaction.webp)

A Workpoint gives the next agent a typed continuation contract instead of a transcript guess. It carries mission, scope, next action, and proof links.

### 02 · Evidence that survives handoff

![Focusa evidence refs](docs/assets/readme/02-evidence-refs.webp)

Focusa stores proof as Evidence refs tied to the active Workpoint: tests, files, routes, screenshots, command output, and release checks.

### 03 · Context Authority stops off-mission mutation

![Focusa Context Authority](docs/assets/readme/03-context-authority.webp)

Before risky changes, Focusa checks whether the current task, project, environment, and install role match the operator’s actual intent.

### 04 · Mission Deck in the terminal

![Focusa TUI Mission Deck](docs/assets/readme/04-mission-deck.webp)

The TUI gives operators a compact mission cockpit: current focus, Workpoint, proof state, trajectory, health, and next safe action.

### 05 · Pi extension integration

![Focusa Pi extension](docs/assets/readme/05-pi-extension.webp)

The Pi extension lets agents call Focusa directly, linking Workpoints, Evidence, memory, trajectory, and recovery tools without inventing a parallel state system.

### 06 · Local daemon, typed API

![Focusa local daemon](docs/assets/readme/06-local-api.webp)

Focusa runs beside the agent as a local Rust daemon. State stays on your machine or VPS. The HTTP API is typed, inspectable, and scriptable.

### 07 · Menubar cockpit preview

![Focusa menubar preview](docs/assets/readme/07-menubar-preview.webp)

The macOS/Tauri menubar is a preview surface. The primary Operator Preview is daemon, CLI, TUI, Workpoints, Evidence, and Pi integration.

### 08 · Public proof, redacted by default

![Focusa public proof cards](docs/assets/readme/08-public-proof.webp)

Focusa can produce public-safe proof summaries without exposing local paths, raw transcripts, license data, or private operator state.
```

### 3.2 Screenshot asset plan

Create this folder:

```text
docs/assets/readme/
```

Add or generate:

```text
docs/assets/readme/focusa-hero.png
docs/assets/readme/01-resume-after-compaction.webp
docs/assets/readme/02-evidence-refs.webp
docs/assets/readme/03-context-authority.webp
docs/assets/readme/04-mission-deck.webp
docs/assets/readme/05-pi-extension.webp
docs/assets/readme/06-local-api.webp
docs/assets/readme/07-menubar-preview.webp
docs/assets/readme/08-public-proof.webp
```

Asset guidelines:

- Use terminal/TUI screenshots where real.
- Use static captured output when GUI proof is not final.
- Do not use fake app UI that implies a shipped Mac app.
- Do not include local usernames, IPs, root prompts, raw license data, or private paths.
- If a screenshot is simulated, label it as “illustrative terminal capture.”
- Prefer actual `focusa` output with sanitized project roots such as:

```text
~/projects/focusa-demo
```

Not:

```text
/home/wirebot/focusa
/root/...
```

### 3.3 README badge cleanup

Replace the stale hardcoded badge:

```text
version-0.9.25--dev
```

With either:

```text
version-0.9.74--dev
```

or remove the version badge and let Release badge carry release status.

Recommended badge set:

```md
[![CI](https://github.com/Startempire-Wire/focusa/actions/workflows/ci.yml/badge.svg)](https://github.com/Startempire-Wire/focusa/actions/workflows/ci.yml)
[![Release](https://github.com/Startempire-Wire/focusa/actions/workflows/release.yml/badge.svg)](https://github.com/Startempire-Wire/focusa/actions/workflows/release.yml)
![License](https://img.shields.io/badge/license-BSL--1.1-orange)
![Source Available](https://img.shields.io/badge/source-available-orange)
![Rust](https://img.shields.io/badge/rust-1.91%2B-dea584?logo=rust)
![Svelte](https://img.shields.io/badge/Svelte-5-ff3e00?logo=svelte)
![Local First](https://img.shields.io/badge/local--first-agent%20infrastructure-7c3aed)
![Operator Preview](https://img.shields.io/badge/status-operator%20preview-22c55e)
```

Avoid primitive badges such as:

```text
Workpoints
Trajectory
Context Authority
Evidence Backed
```

Those make the top visually noisy. Put them in the feature sections instead.

---

## 4. Public/private knowledge boundary

### 4.1 Chosen private location

Use:

```text
.focusa-private/
```

Do not use `docs/private/` as the main store. It is too easy for agents to treat it as public docs.

### 4.2 Private folder map

```text
.focusa-private/
  INDEX.md
  strategy/
    115-focusa-cloud-control-plane-tool-gateway-master-spec.md
    SIGNALOS_TECHNICAL_SCOPE.md
  commercial/
    118-focusa-license-tiers-spec.full-internal.md
    SPEC_119_LIFETIME_TO_RECURRING_TRANSITION.md
    LICENSE-FAQ.full-internal.md
    install-purchase/
      INSTALL_PURCHASE_GAP_AUDIT_2026-07-07.md
      INSTALL_PURCHASE_ACHIEVEMENTS_2026-07-07.md
      INSTALL_PURCHASE_VENDOR_CLOSE_2026-07-07.md
      DISCREPANCIES_LEDGER_2026-07-07.md
  evidence/
    raw/
      FRESH_OPERATOR_DRY_RUN_2026-07-05.md
      GAPS_4_9_LIVE_PROOF_2026-07-05.md
    transcripts/
      fresh-operator-dry-run-2026-07-05/
      freshop-q-gaps-4-9/
    release-proof/
      latest.internal.json
      v0.9.74-dev.openproof.internal.json
  runtime/
    ecs-objects-before-public-cleanup/
    ecs-handles-before-public-cleanup/
    ecs-handles-index-before-public-cleanup/
  launch/
  agent-kb/
```

### 4.3 Private index

Create:

```text
.focusa-private/INDEX.md
```

Content:

```md
# Focusa Private Operator Docs

This folder is local-only and ignored by git.

Agents should use this folder for:

- SaaS/control-plane strategy
- SignalOS strategy
- pricing and lifetime cap planning
- install/purchase backend details
- raw proof transcripts
- vendor-side license/registry work
- internal launch notes
- local runtime artifacts preserved before public cleanup
- agent-kb server knowledge

Agents must not copy files from this folder into public tracked paths unless explicitly instructed.

Before touching any of the following, read this index and the matching private folder:

- commercial license strategy
- pricing/caps
- SignalOS
- Focusa Cloud
- raw proof/evidence
- public proof policy
- install/purchase flow
- license registry
- launch positioning
```

---

## 5. `.gitignore` update

Append:

```gitignore
# Private operator / agent strategy docs
.focusa-private/
_private/
agent-kb/
docs/private/

# Internal proof / transcript staging
release-proof/internal/
docs/evidence/raw/
docs/evidence/transcripts/

# Runtime / local-only Focusa artifacts
ecs/objects/
ecs/handles/
ecs/handles-index/
device-pairing/
hlt-ledger/
```

Notes:

- Some runtime paths are already ignored.
- Repeating them in a clear grouped block is intentional.
- This makes the public/private boundary obvious to humans and agents.

---

## 6. Files to move private or sanitize

### 6.1 Move private and remove public tracked copy

Move these out of the public repo:

```text
docs/115-focusa-cloud-control-plane-tool-gateway-master-spec.md
docs/SIGNALOS_TECHNICAL_SCOPE.md
docs/SPEC_119_LIFETIME_TO_RECURRING_TRANSITION.md
docs/INSTALL_PURCHASE_GAP_AUDIT_2026-07-07.md
docs/INSTALL_PURCHASE_ACHIEVEMENTS_2026-07-07.md
docs/INSTALL_PURCHASE_VENDOR_CLOSE_2026-07-07.md
docs/DISCREPANCIES_LEDGER_2026-07-07.md
release-proof/latest.json
release-proof/v0.9.74-dev.openproof.json
docs/evidence/FRESH_OPERATOR_DRY_RUN_2026-07-05.md
docs/evidence/GAPS_4_9_LIVE_PROOF_2026-07-05.md
docs/evidence/fresh-operator-dry-run-2026-07-05/
docs/evidence/freshop-q-gaps-4-9/
ecs/objects/
ecs/handles/
ecs/handles-index/
```

### 6.2 Keep public but rewrite/sanitize

Rewrite these instead of deleting:

```text
README.md
LICENSE-FAQ.md
SECURITY.md
docs/118-focusa-license-tiers-spec.md
docs/AGENTS.md
scripts/install-focusa.sh
scripts/install-focusa.ps1
crates/focusa-cli/src/commands/license.rs
```

### 6.3 Add new public files

Add:

```text
AGENTS.md
docs/PUBLIC_INDEX.md
docs/ROADMAP.md
docs/PUBLIC_PROOF_POLICY.md
docs/INSTALL_PURCHASE_PUBLIC_STATUS.md
docs/evidence/OPERATOR_PREVIEW_DRY_RUN_SUMMARY_2026-07-05.md
release-proof/public/latest.json
scripts/guard-public-surface.sh
docs/assets/readme/
```

---

## 7. Specific file actions

### 7.1 Move Spec 115 private

```bash
mkdir -p .focusa-private/strategy

cp -p docs/115-focusa-cloud-control-plane-tool-gateway-master-spec.md \
  .focusa-private/strategy/115-focusa-cloud-control-plane-tool-gateway-master-spec.md

git rm docs/115-focusa-cloud-control-plane-tool-gateway-master-spec.md
```

### 7.2 Move SignalOS private

```bash
cp -p docs/SIGNALOS_TECHNICAL_SCOPE.md \
  .focusa-private/strategy/SIGNALOS_TECHNICAL_SCOPE.md

git rm docs/SIGNALOS_TECHNICAL_SCOPE.md
```

### 7.3 Move Spec 119 private

```bash
mkdir -p .focusa-private/commercial

cp -p docs/SPEC_119_LIFETIME_TO_RECURRING_TRANSITION.md \
  .focusa-private/commercial/SPEC_119_LIFETIME_TO_RECURRING_TRANSITION.md

git rm docs/SPEC_119_LIFETIME_TO_RECURRING_TRANSITION.md
```

### 7.4 Preserve and sanitize Spec 118

```bash
cp -p docs/118-focusa-license-tiers-spec.md \
  .focusa-private/commercial/118-focusa-license-tiers-spec.full-internal.md
```

Then rewrite public `docs/118-focusa-license-tiers-spec.md` as:

```md
# Focusa License Modes and Gates — Public Reference

**Status:** Public reference for license modes and command gates.

## License modes

| Mode | Intended use |
|---|---|
| Evaluation | Personal, educational, evaluation, and non-commercial local use |
| Operator | Single-operator commercial use |
| Team | Team/multi-seat commercial use |
| Enterprise | Enterprise, hosted, regulated, or custom deployments |

## Public commercial-use boundary

Commercial, company, team, hosted-service, client-delivery, redistribution, or embedding use requires a commercial license.

## Public command gates

Some commercial distribution and proof surfaces require a commercial license, including packaged release proof, export artifacts, packaged installers, and advanced handoff surfaces.

## Current Operator Preview

Operator Preview focuses on:

- daemon
- CLI
- TUI/Mission Deck
- Workpoints
- Evidence
- Context Authority
- proof-backed continuation
- Pi integration

Internal cap counts, transition planning, registry mechanics, and vendor-side implementation details are maintained privately.
```

### 7.5 Rewrite License FAQ

Preserve internal copy:

```bash
cp -p LICENSE-FAQ.md \
  .focusa-private/commercial/LICENSE-FAQ.full-internal.md
```

Rewrite public FAQ:

```md
# Focusa License FAQ

## Can I try Focusa personally?

Yes. Personal, educational, evaluation, and non-commercial local use is allowed under the source-available license terms.

## Can I use Focusa inside my company or team?

Commercial, company, team, internal production, hosted-service, or client-delivery use requires a separate commercial license from Startempire Wire.

## Can I use Focusa for paid client work?

A commercial license is required for paid client work, managed agent operations, redistribution, or embedding Focusa into a paid product/service.

## Can I fork Focusa?

Only under the terms in `LICENSE.md`. Forking does not remove commercial-use restrictions.

## Does Focusa become open source later?

See `LICENSE.md` for the Business Source License change date and future license terms.

## Where are commercial terms?

See `COMMERCIAL.md`, `SUPPORT_TERMS.md`, `TRADEMARKS.md`, and `CONTRIBUTING.md`.

Specific launch offers, team terms, and enterprise terms are handled through the official Focusa purchase/support path.
```

### 7.6 Move install/purchase audit docs private

```bash
mkdir -p .focusa-private/commercial/install-purchase

cp -p docs/INSTALL_PURCHASE_GAP_AUDIT_2026-07-07.md \
  .focusa-private/commercial/install-purchase/ 2>/dev/null || true

cp -p docs/INSTALL_PURCHASE_ACHIEVEMENTS_2026-07-07.md \
  .focusa-private/commercial/install-purchase/ 2>/dev/null || true

cp -p docs/INSTALL_PURCHASE_VENDOR_CLOSE_2026-07-07.md \
  .focusa-private/commercial/install-purchase/ 2>/dev/null || true

cp -p docs/DISCREPANCIES_LEDGER_2026-07-07.md \
  .focusa-private/commercial/install-purchase/ 2>/dev/null || true

git rm --ignore-unmatch docs/INSTALL_PURCHASE_GAP_AUDIT_2026-07-07.md
git rm --ignore-unmatch docs/INSTALL_PURCHASE_ACHIEVEMENTS_2026-07-07.md
git rm --ignore-unmatch docs/INSTALL_PURCHASE_VENDOR_CLOSE_2026-07-07.md
git rm --ignore-unmatch docs/DISCREPANCIES_LEDGER_2026-07-07.md
```

Add public replacement:

```text
docs/INSTALL_PURCHASE_PUBLIC_STATUS.md
```

Content:

```md
# Install + Purchase Public Status

Focusa supports evaluation installs and commercial license activation paths.

Current public surfaces:

- `install.focusa.dev/focusa` for installer bootstrap
- `install.focusa.dev/license` for commercial license explanation
- `focusa license status`
- `focusa license doctor`

Internal registry, transaction, vendor, webhook, and purchase-pipeline implementation details are maintained privately.
```

### 7.7 Move raw release proof private

```bash
mkdir -p .focusa-private/evidence/release-proof release-proof/public

cp -p release-proof/latest.json \
  .focusa-private/evidence/release-proof/latest.internal.json 2>/dev/null || true

cp -p release-proof/v0.9.74-dev.openproof.json \
  .focusa-private/evidence/release-proof/v0.9.74-dev.openproof.internal.json 2>/dev/null || true

git rm --ignore-unmatch release-proof/latest.json
git rm --ignore-unmatch release-proof/v0.9.74-dev.openproof.json
```

Add:

```text
release-proof/public/latest.json
```

Content:

```json
{
  "status": "operator_preview",
  "version": "0.9.74-dev",
  "summary": "Focusa Operator Preview public proof.",
  "gates": [
    { "name": "cargo test --workspace", "status": "passed" },
    { "name": "cargo clippy --workspace -- -D warnings", "status": "passed" },
    { "name": "focusa preflight", "status": "passed" },
    { "name": "static spec audit", "status": "passed" },
    { "name": "public docs scan", "status": "passed" },
    { "name": "daemon health", "status": "passed" }
  ],
  "public_limitations": [
    "Mac menubar is preview, not the primary launch surface.",
    "Operator Preview focuses on daemon, CLI/TUI, Workpoints, Evidence, and proof-backed continuation."
  ]
}
```

### 7.8 Move raw transcript proof private

```bash
mkdir -p .focusa-private/evidence/raw .focusa-private/evidence/transcripts

cp -p docs/evidence/FRESH_OPERATOR_DRY_RUN_2026-07-05.md \
  .focusa-private/evidence/raw/ 2>/dev/null || true

cp -p docs/evidence/GAPS_4_9_LIVE_PROOF_2026-07-05.md \
  .focusa-private/evidence/raw/ 2>/dev/null || true

cp -a docs/evidence/fresh-operator-dry-run-2026-07-05 \
  .focusa-private/evidence/transcripts/ 2>/dev/null || true

cp -a docs/evidence/freshop-q-gaps-4-9 \
  .focusa-private/evidence/transcripts/ 2>/dev/null || true

git rm --ignore-unmatch docs/evidence/FRESH_OPERATOR_DRY_RUN_2026-07-05.md
git rm --ignore-unmatch docs/evidence/GAPS_4_9_LIVE_PROOF_2026-07-05.md
git rm -r --ignore-unmatch docs/evidence/fresh-operator-dry-run-2026-07-05
git rm -r --ignore-unmatch docs/evidence/freshop-q-gaps-4-9
```

Add:

```text
docs/evidence/OPERATOR_PREVIEW_DRY_RUN_SUMMARY_2026-07-05.md
```

Content:

```md
# Operator Preview Dry Run Summary — 2026-07-05

A first-run operator walkthrough identified launch-readiness improvements:

1. Quickstart needed to be closer to the top of README.
2. Orientation and next-step guidance needed to be clearer.
3. Installed binary and documented commands needed to match.
4. Project-root creation needed to be friendlier.
5. First-run CLI presentation needed polish.

Raw transcripts are retained privately because they include host paths and operator environment details.
```
`

### 7.9 Remove committed runtime artifacts

```bash
mkdir -p .focusa-private/runtime

cp -a ecs/objects .focusa-private/runtime/ecs-objects-before-public-cleanup 2>/dev/null || true
cp -a ecs/handles .focusa-private/runtime/ecs-handles-before-public-cleanup 2>/dev/null || true
cp -a ecs/handles-index .focusa-private/runtime/ecs-handles-index-before-public-cleanup 2>/dev/null || true

git rm -r --ignore-unmatch ecs/objects
git rm -r --ignore-unmatch ecs/handles
git rm -r --ignore-unmatch ecs/handles-index
```

---

## 8. Root AGENTS.md and docs/AGENTS.md integration

### 8.1 Add root AGENTS.md

Create:

```text
AGENTS.md
```

Content:

```md
# Focusa Agent Instructions

Start public orientation with:

1. `README.md`
2. `docs/PUBLIC_INDEX.md`
3. `docs/GTM_FIVE_MINUTE_PROOF.md`
4. `docs/RELEASE_INSTALL_POSTCARD.md`

If present, private operator docs live at:

- `.focusa-private/INDEX.md`
- `$FOCUSA_PRIVATE_DOCS_DIR`
- `/opt/focusa-private`
- `/root/agent-kb/focusa`

Read `.focusa-private/INDEX.md` before making changes involving:

- SaaS/cloud strategy
- SignalOS
- pricing/caps/commercial transition
- raw proof
- install/purchase backend
- license registry/vendor work
- launch plans

Never commit:

- `.focusa-private/`
- `_private/`
- `docs/private/`
- raw transcripts
- runtime objects
- local host paths
- admin URLs
- customer/license data

Detailed local agent protocol remains in `docs/AGENTS.md`.
```

### 8.2 Update docs/AGENTS.md

Keep existing Beads/Focus rules.

Add near the top:

```md
## Public / Private Docs Boundary

Private operator docs may exist locally at `.focusa-private/`.

Agents must read `.focusa-private/INDEX.md` before touching SaaS strategy, SignalOS, commercial pricing/caps, install/purchase backend, raw proof, launch planning, or vendor/license registry work.

Agents must never commit `.focusa-private/`, raw transcripts, runtime objects, local host paths, admin URLs, customer data, or license data.
```

---

## 9. New public docs

### 9.1 docs/PUBLIC_INDEX.md

Create:

```md
# Focusa Public Docs

Start here:

1. `../README.md` — product overview and install
2. `GTM_FIVE_MINUTE_PROOF.md` — evaluator demo
3. `RELEASE_INSTALL_POSTCARD.md` — install and first run
4. `OPERATOR_PREVIEW_SCOPE.md` — what is supported now
5. `ROADMAP.md` — public roadmap
6. `PUBLIC_PROOF_POLICY.md` — what public proof may contain
7. `current/API_REFERENCE_CURRENT.md` — API reference
8. `current/CLI_REFERENCE_CURRENT.md` — CLI reference
9. `../SECURITY.md` — vulnerability reporting
10. `../COMMERCIAL.md` — commercial-use boundary
```

### 9.2 docs/ROADMAP.md

Create or rewrite:

```md
# Focusa Roadmap

Focusa is local-first today.

The current Operator Preview focuses on:

- daemon
- CLI
- TUI/Mission Deck
- Workpoints
- Evidence
- Context Authority
- proof-backed continuation
- Pi integration

Future surfaces may include:

- managed install/update paths
- cloud coordination
- public-safe proof publishing
- team visibility
- richer cockpit surfaces

Canonical work state remains local-first and operator-controlled.
```

### 9.3 docs/PUBLIC_PROOF_POLICY.md

Create:

```md
---
public_surface: true
private_paths_forbidden:
  - .focusa-private
  - docs/private
  - ecs/objects
  - release-proof/internal
  - docs/evidence/raw
  - docs/evidence/transcripts
---

# Public Proof Policy

Public proof may include:

- tag
- commit SHA
- test status
- clippy status
- CI status
- daemon health
- public-safe limitations
- sanitized command summaries

Public proof must not include:

- raw shell transcripts
- local usernames
- absolute home paths
- hostnames
- IPs
- admin URLs
- customer emails
- license keys
- registry internals
- vendor-side TODOs
- DB corruption details
- cgroup/systemd private host internals
```

---

## 10. README detailed rewrite plan

### 10.1 Remove top-heavy primitive explanation

Current opening explains:

```text
ProjectIdentity
Continuity ID
HLT
MLG
STG
Waypoints
Workpoints
Evidence Refs
Context Cognition
Context Authority
```

Move this later under:

```md
## Architecture vocabulary
```

### 10.2 New README opening

Use:

```md
<p align="center">
  <img src="docs/assets/readme/focusa-hero.png" alt="Focusa Mission Deck" width="900">
</p>

<h1 align="center">Focusa</h1>

<p align="center">
  <strong>Keep AI coding agents on mission.</strong><br>
  <sub>Local-first proof, Workpoints, Evidence, and continuation for long-running coding agents.</sub>
</p>

<p align="center">
  <a href="https://github.com/Startempire-Wire/focusa/actions/workflows/ci.yml"><img src="https://github.com/Startempire-Wire/focusa/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/Startempire-Wire/focusa/actions/workflows/release.yml"><img src="https://github.com/Startempire-Wire/focusa/actions/workflows/release.yml/badge.svg" alt="Release"></a>
  <img src="https://img.shields.io/badge/license-BSL--1.1-orange" alt="BSL 1.1">
  <img src="https://img.shields.io/badge/source-available-orange" alt="Source Available">
  <img src="https://img.shields.io/badge/status-operator%20preview-22c55e" alt="Operator Preview">
  <img src="https://img.shields.io/badge/local--first-agent%20infrastructure-7c3aed" alt="Local First">
</p>

Focusa is the local-first proof and continuity layer for AI coding agents.

Claude, Codex, OpenCode, OpenClaw, Pi, and other coding agents can move fast until the session gets long, context compacts, the mission drifts, proof gets buried, or the next agent has to start over.

Focusa gives the work a durable Workpoint, linked Evidence, and a next safe action outside the fragile chat window.
```

### 10.3 New install block

```md
## Install

```bash
curl -fsS https://install.focusa.dev/focusa | bash -s -- --eval
focusa start
focusa init --quickstart
focusa doctor
```

For source builds and daemon-deploy paths, see `docs/RELEASE_INSTALL_POSTCARD.md`.
```

### 10.4 New five-minute proof block

```md
## Five-minute proof

```bash
focusa doctor
focusa workpoint checkpoint --title "First Focusa proof"
focusa evidence link <workpoint-id> "test: cargo test --workspace"
focusa workpoint resume --copy-prompt
```

Expected result:

- daemon is healthy
- project is bound
- Workpoint exists
- Evidence is linked
- resume output gives the next agent a continuation packet
```

### 10.5 Oh-my-pi-style numbered visual sections

```md
## The Focusa you can use today

### 01 · Resume after compaction

![Focusa resume packet](docs/assets/readme/01-resume-after-compaction.webp)

A Workpoint gives the next agent a typed continuation contract instead of a transcript guess. It carries mission, scope, next action, and proof links.

### 02 · Evidence that survives handoff

![Focusa evidence refs](docs/assets/readme/02-evidence-refs.webp)

Focusa stores proof as Evidence refs tied to the active Workpoint: tests, files, routes, screenshots, command output, and release checks.

### 03 · Context Authority stops off-mission mutation

![Focusa Context Authority](docs/assets/readme/03-context-authority.webp)

Before risky changes, Focusa checks whether the current task, project, environment, and install role match the operator’s actual intent.

### 04 · Mission Deck in the terminal

![Focusa TUI Mission Deck](docs/assets/readme/04-mission-deck.webp)

The TUI gives operators a compact mission cockpit: current focus, Workpoint, proof state, trajectory, health, and next safe action.

### 05 · Pi extension integration

![Focusa Pi extension](docs/assets/readme/05-pi-extension.webp)

The Pi extension lets agents call Focusa directly, linking Workpoints, Evidence, memory, trajectory, and recovery tools without inventing a parallel state system.

### 06 · Local daemon, typed API

![Focusa local daemon](docs/assets/readme/06-local-api.webp)

Focusa runs beside the agent as a local Rust daemon. State stays on your machine or VPS. The HTTP API is typed, inspectable, and scriptable.

### 07 · Menubar cockpit preview

![Focusa menubar preview](docs/assets/readme/07-menubar-preview.webp)

The macOS/Tauri menubar is a preview surface. The primary Operator Preview is daemon, CLI, TUI, Workpoints, Evidence, and Pi integration.

### 08 · Public proof, redacted by default

![Focusa public proof cards](docs/assets/readme/08-public-proof.webp)

Focusa can produce public-safe proof summaries without exposing local paths, raw transcripts, license data, or private operator state.
```

### 10.6 Repository shape section

```md
## Repository shape

- `crates/focusa-api` — local daemon/API
- `crates/focusa-cli` — operator CLI
- `crates/focusa-tui` — terminal Mission Deck
- `apps/pi-extension` — Pi integration
- `apps/menubar` — preview macOS cockpit
- `docs/current` — current public docs
- `docs/focusa-tools` — generated/current tool docs
- `release-proof/public` — public-safe release proof
- `.focusa-private` — ignored local operator docs, if present
```

### 10.7 Boundaries section

```md
## Current boundaries

Focusa Operator Preview supports:

- daemon
- CLI
- TUI/Mission Deck
- Workpoints
- Evidence
- Context Authority
- proof-backed continuation
- Pi integration

Preview / not flagship:

- macOS/Tauri menubar
- native macOS lifecycle proof
- Keychain persistence
- full public proof publishing
- team/cloud sync

Not public repo material:

- raw transcripts
- private host paths
- license backend/admin details
- pricing cap math
- SignalOS strategy
- SaaS/cloud internal planning
```

---

## 11. Screenshot/capture generation instructions

### 11.1 Capture rules

Every README screenshot must be:

- real or clearly illustrative
- sanitized
- legible at GitHub README width
- low-noise
- consistent dark terminal style
- no private paths
- no local usernames
- no root prompt
- no IPs
- no license/customer data

### 11.2 Recommended capture source commands

Use sanitized project root:

```bash
export FOCUSA_DEMO_ROOT="$HOME/projects/focusa-demo"
mkdir -p "$FOCUSA_DEMO_ROOT"
cd "$FOCUSA_DEMO_ROOT"
```

Capture:

```bash
focusa doctor
focusa init --quickstart
focusa workpoint checkpoint --title "Implement API health check"
focusa evidence link <workpoint-id> "test: cargo test --workspace"
focusa workpoint resume --copy-prompt
focusa action preflight --kind binary_replace --install-role live_build_host
focusa tui --headless-self-test
curl -fsS http://127.0.0.1:8787/v1/health
```

Sanitize output before screenshots:

```text
/home/wirebot/focusa       -> ~/projects/focusa-demo
/root/...                  -> ~/.focusa/...
127.0.0.1:8787             -> keep allowed
license key/customer email -> remove
hostnames/IPs              -> remove
```

---

## 12. Public-surface guard

Create:

```text
scripts/guard-public-surface.sh
```

Content:

```bash
#!/usr/bin/env bash
set -euo pipefail

HARD_PATTERNS=(
  '/home/wirebot'
  'root@host'
  'wpuiai.com/wp-admin'
  'signalos.pro'
  '$384,330'
  '.focusa-private'
)

CAUTION_PATTERNS=(
  'MemoryMax'
  '.corrupt'
  'xShmMap'
  'production VPS'
  'tmux session'
  'Founders Forge'
  'dev_mode'
  'Stripe webhook'
  'license row'
)

TARGETS=(
  README.md
  SECURITY.md
  LICENSE-FAQ.md
  COMMERCIAL.md
  SUPPORT_TERMS.md
  docs
  release-proof
  scripts
  crates/focusa-cli/src/commands
)

for p in "${HARD_PATTERNS[@]}"; do
  if rg -n --fixed-strings "$p" "${TARGETS[@]}" 2>/dev/null; then
    echo "public-surface guard failed: hard private/internal pattern: $p" >&2
    exit 1
  fi
done

for p in "${CAUTION_PATTERNS[@]}"; do
  if rg -n --fixed-strings "$p" "${TARGETS[@]}" 2>/dev/null; then
    echo "public-surface guard warning: review pattern: $p" >&2
  fi
done

echo "public-surface guard completed"
```

Make executable:

```bash
chmod +x scripts/guard-public-surface.sh
```

Run before public launch:

```bash
scripts/guard-public-surface.sh
```

---

## 13. Backend/admin URL cleanup

### 13.1 Current concern

Buyer-facing surfaces should not show:

```text
wpuiai.com/wp-admin
```

or backend operational/admin URLs.

### 13.2 Public URL standard

Use only:

```text
https://install.focusa.dev/license
https://focusa.dev/support
support@focusa.dev
```

### 13.3 Files to inspect and rewrite

```text
scripts/install-focusa.sh
scripts/install-focusa.ps1
crates/focusa-cli/src/commands/license.rs
docs/INSTALL_PURCHASE_PUBLIC_STATUS.md
README.md
LICENSE-FAQ.md
COMMERCIAL.md
SUPPORT_TERMS.md
```

Internal backend details belong in:

```text
.focusa-private/commercial/install-purchase/
```

---

## 14. Security.md cleanup

Rewrite public `SECURITY.md` to remove placeholder wording.

Target:

```md
# Security Policy

Please report security issues to security@focusa.dev.

Do not open public issues for suspected vulnerabilities.

Include:

- affected version or commit
- affected command/API surface
- reproduction steps
- impact
- suggested mitigation, if known
```

---

## 15. No-commit execution runbook

Run this to stage the cleanup locally without committing:

```bash
set -euo pipefail

# 1. Create local-only private knowledge root
mkdir -p .focusa-private/{strategy,commercial/install-purchase,evidence/raw,evidence/transcripts,evidence/release-proof,runtime,launch,agent-kb}

# 2. Add ignore rules
cat >> .gitignore <<'EOF'

# Private operator / agent strategy docs
.focusa-private/
_private/
agent-kb/
docs/private/

# Internal proof / transcript staging
release-proof/internal/
docs/evidence/raw/
docs/evidence/transcripts/

# Runtime / local-only Focusa artifacts
ecs/objects/
ecs/handles/
ecs/handles-index/
device-pairing/
hlt-ledger/
EOF

# 3. Create private index
cat > .focusa-private/INDEX.md <<'EOF'
# Focusa Private Operator Docs

This folder is local-only and ignored by git.

Agents should use this folder for SaaS/control-plane strategy, SignalOS strategy,
pricing and lifetime cap planning, install/purchase backend details, raw proof
transcripts, vendor-side license/registry work, internal launch notes, local
runtime artifacts, and agent-kb server knowledge.

Agents must not copy files from this folder into public tracked paths unless
explicitly instructed.
EOF

# 4. Preserve strategy docs privately
cp -p docs/115-focusa-cloud-control-plane-tool-gateway-master-spec.md .focusa-private/strategy/ 2>/dev/null || true
cp -p docs/SIGNALOS_TECHNICAL_SCOPE.md .focusa-private/strategy/ 2>/dev/null || true

# 5. Preserve commercial docs privately
cp -p docs/SPEC_119_LIFETIME_TO_RECURRING_TRANSITION.md .focusa-private/commercial/ 2>/dev/null || true
cp -p docs/118-focusa-license-tiers-spec.md .focusa-private/commercial/118-focusa-license-tiers-spec.full-internal.md 2>/dev/null || true
cp -p LICENSE-FAQ.md .focusa-private/commercial/LICENSE-FAQ.full-internal.md 2>/dev/null || true

# 6. Preserve install/purchase docs privately
cp -p docs/INSTALL_PURCHASE_GAP_AUDIT_2026-07-07.md .focusa-private/commercial/install-purchase/ 2>/dev/null || true
cp -p docs/INSTALL_PURCHASE_ACHIEVEMENTS_2026-07-07.md .focusa-private/commercial/install-purchase/ 2>/dev/null || true
cp -p docs/INSTALL_PURCHASE_VENDOR_CLOSE_2026-07-07.md .focusa-private/commercial/install-purchase/ 2>/dev/null || true
cp -p docs/DISCREPANCIES_LEDGER_2026-07-07.md .focusa-private/commercial/install-purchase/ 2>/dev/null || true

# 7. Preserve raw proof privately
cp -p release-proof/latest.json .focusa-private/evidence/release-proof/latest.internal.json 2>/dev/null || true
cp -p release-proof/v0.9.74-dev.openproof.json .focusa-private/evidence/release-proof/v0.9.74-dev.openproof.internal.json 2>/dev/null || true

# 8. Preserve transcript evidence privately
cp -p docs/evidence/FRESH_OPERATOR_DRY_RUN_2026-07-05.md .focusa-private/evidence/raw/ 2>/dev/null || true
cp -p docs/evidence/GAPS_4_9_LIVE_PROOF_2026-07-05.md .focusa-private/evidence/raw/ 2>/dev/null || true
cp -a docs/evidence/fresh-operator-dry-run-2026-07-05 .focusa-private/evidence/transcripts/ 2>/dev/null || true
cp -a docs/evidence/freshop-q-gaps-4-9 .focusa-private/evidence/transcripts/ 2>/dev/null || true

# 9. Preserve runtime objects privately
cp -a ecs/objects .focusa-private/runtime/ecs-objects-before-public-cleanup 2>/dev/null || true
cp -a ecs/handles .focusa-private/runtime/ecs-handles-before-public-cleanup 2>/dev/null || true
cp -a ecs/handles-index .focusa-private/runtime/ecs-handles-index-before-public-cleanup 2>/dev/null || true

# 10. Remove public tracked internal docs
git rm --ignore-unmatch docs/115-focusa-cloud-control-plane-tool-gateway-master-spec.md
git rm --ignore-unmatch docs/SIGNALOS_TECHNICAL_SCOPE.md
git rm --ignore-unmatch docs/SPEC_119_LIFETIME_TO_RECURRING_TRANSITION.md

# 11. Remove public tracked install/purchase internals
git rm --ignore-unmatch docs/INSTALL_PURCHASE_GAP_AUDIT_2026-07-07.md
git rm --ignore-unmatch docs/INSTALL_PURCHASE_ACHIEVEMENTS_2026-07-07.md
git rm --ignore-unmatch docs/INSTALL_PURCHASE_VENDOR_CLOSE_2026-07-07.md
git rm --ignore-unmatch docs/DISCREPANCIES_LEDGER_2026-07-07.md

# 12. Remove public raw proof/transcripts
git rm --ignore-unmatch release-proof/latest.json
git rm --ignore-unmatch release-proof/v0.9.74-dev.openproof.json
git rm --ignore-unmatch docs/evidence/FRESH_OPERATOR_DRY_RUN_2026-07-05.md
git rm --ignore-unmatch docs/evidence/GAPS_4_9_LIVE_PROOF_2026-07-05.md
git rm -r --ignore-unmatch docs/evidence/fresh-operator-dry-run-2026-07-05
git rm -r --ignore-unmatch docs/evidence/freshop-q-gaps-4-9

# 13. Remove runtime artifacts
git rm -r --ignore-unmatch ecs/objects
git rm -r --ignore-unmatch ecs/handles
git rm -r --ignore-unmatch ecs/handles-index

# 14. Inspect only; no commit
git status --short
git diff --stat
```

---

## 16. Acceptance criteria

Spec 123 is accepted when:

### Public/private boundary

- `.focusa-private/` exists locally and is ignored.
- Private docs are copied into `.focusa-private/`.
- Strategic docs are removed from public tracking.
- Raw transcripts are removed from public tracking.
- Runtime artifacts are removed from public tracking.

### README presentation

- README opens with hero image, product promise, badges, install, and five-minute proof.
- README uses numbered visual feature sections.
- README no longer leads with dense internal vocabulary.
- README version badge is not stale.
- README quickstart uses the public installer path.
- Menubar is still marked preview/not flagship.

### Public docs

- `docs/PUBLIC_INDEX.md` exists.
- `docs/ROADMAP.md` is public-safe.
- `docs/PUBLIC_PROOF_POLICY.md` exists.
- `docs/INSTALL_PURCHASE_PUBLIC_STATUS.md` replaces raw install/purchase audit docs.
- `release-proof/public/latest.json` exists.
- `LICENSE-FAQ.md` no longer exposes cap counts or revenue math.
- `docs/118-focusa-license-tiers-spec.md` no longer exposes cap counts, revenue ceiling, open ledger questions, or “not enforced yet” details as public weakness.

### Agent access

- Root `AGENTS.md` exists.
- Existing `docs/AGENTS.md` remains and includes private-boundary guidance.
- Agents know to check `.focusa-private/INDEX.md`.

### Security / trust

- `SECURITY.md` no longer says placeholder.
- Public buyer-facing output does not show `wpuiai.com/wp-admin`.
- Public proof does not include raw host paths, root prompts, raw transcripts, license/customer data, or private vendor details.
- `scripts/guard-public-surface.sh` exists and runs.

---

## 17. Recommended commit sequence after review

Do not commit until the diff is inspected.

When ready, split into small commits:

```bash
git add .gitignore AGENTS.md docs/AGENTS.md .focusa-private/INDEX.md
git commit -m "chore: define private operator docs boundary"

git add README.md docs/assets/readme
git commit -m "docs: reframe README around visual operator preview"

git add docs/PUBLIC_INDEX.md docs/ROADMAP.md docs/PUBLIC_PROOF_POLICY.md docs/INSTALL_PURCHASE_PUBLIC_STATUS.md release-proof/public/latest.json
git commit -m "docs: add public-safe docs and proof surfaces"

git add LICENSE-FAQ.md docs/118-focusa-license-tiers-spec.md SECURITY.md
git commit -m "docs: sanitize license and security public wording"

git add -u docs release-proof ecs
git commit -m "chore: remove private strategy raw proof and runtime artifacts from public tracking"

git add scripts/guard-public-surface.sh
git commit -m "test: add public surface guard"
```

Before pushing:

```bash
scripts/guard-public-surface.sh
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

---

## 18. Final public positioning after Spec 123

After this cleanup, the repo should feel like:

```text
Focusa is a serious Operator Preview product.
It has a clear install path.
It proves one valuable workflow quickly.
It shows visual proof while scrolling.
It is honest about preview boundaries.
It protects private strategy.
It gives agents full local access without leaking strategy to GitHub.
```

The public repo should no longer feel like:

```text
A raw agent notebook with SaaS strategy, pricing math, license backend details,
host transcripts, and runtime objects mixed into product docs.
```

Spec 123 is the professionalization bridge between those two states.
