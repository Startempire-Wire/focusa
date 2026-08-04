# FOCUSA-TRANSITION-001 — Desktop Preview, Build, and Release Milestone Contract

**Status:** mandatory execution companion  
**Date:** 2026-08-04  
**Parent:** `FOCUSA-TRANSITION-001-mission-canvas-to-desktop-handoff.md`

---

## 1. MacBook worktree and upstream rule

The agent performing the Mission Canvas refactor and initial Focusa Desktop implementation works from the existing MacBook worktree.

During preservation, decomposition and implementation before an explicit milestone approval:

- commit locally on a dedicated transition/refactor branch;
- do not commit or push directly to `origin/main`;
- do not push commits onto the existing shared Mission Canvas branches;
- do not create or move tags;
- do not publish a GitHub Release;
- do not force-push any branch;
- do not upload MacBook-built release artifacts;
- do not trigger the canonical release pipeline from the MacBook.

The phrase “do not commit directly to upstream” means that the agent must not mutate shared upstream authority branches or release refs while refactoring. Local commits are required for preservation and review.

When an operator explicitly approves publication, push only a dedicated review branch or patch set. Merge to `main` occurs through reviewed repository workflow, not by direct MacBook push.

---

## 2. One local Rust toolchain

The MacBook SHALL use one pinned project Rust toolchain.

Rules:

- use the repository `rust-toolchain.toml` or one explicitly pinned toolchain;
- do not install multiple Rust toolchains for repeated milestone builds;
- do not repeatedly run toolchain bootstrap/install scripts;
- do not create per-worktree Rust installations;
- do not run local `cargo build --release` for shipping artifacts;
- reuse Cargo caches and the same target strategy where safe;
- record the exact Rust, Cargo, Node, package-manager, Tauri and platform versions in milestone evidence.

If a toolchain mismatch appears, correct the pinned contract once rather than adding another local toolchain.

---

## 3. Browser-first continuous preview

Focusa Desktop is the primary application. Its SvelteKit application SHALL be continuously previewable in a normal browser during development.

Between native milestones:

```text
SvelteKit/Vite development server
  -> continuous browser preview
  -> UIAI Engine browser session
  -> screenshots, responsive checks, console/network diagnostics,
     interaction verification and milestone Evidence
```

The agent should keep the browser preview current after each meaningful UI or control-plane slice.

UIAI Engine is the required browser evaluation and proof path. Do not add Playwright or another browser authority.

Continuous browser preview does not replace native validation. It reduces repeated Rust/Tauri compilation while preserving rapid UI feedback.

---

## 4. Native Tauri milestone builds

A full Focusa Desktop Tauri shell must be built and opened at these completion gates:

```text
5%
25%
50%
75%
100%
```

Each gate is a product milestone, not merely a percentage claim. The agent must record the exact completed scope and Evidence.

### 4.1 Five percent — native shell proof

Required:

- app identity and Tauri configuration;
- SvelteKit shell launches inside Tauri;
- product-neutral shell/navigation baseline;
- daemon connection placeholder or truthful unavailable state;
- browser preview and full native shell screenshots;
- no canonical mutation and no duplicated Mission state.

### 4.2 Twenty-five percent — Workstream-aware vertical skeleton

Required:

- Workstream-aware Context Control contract;
- Mission Deck/Overview skeleton;
- Mission Canvas current-work projection using typed fixtures or safe read-only data;
- workspace/command manifests;
- semantic Desktop state skeleton;
- browser and native proof.

No global-current reducer assumptions may be cemented.

### 4.3 Fifty percent — interactive local application

Required:

- real daemon discovery/read path;
- exact Workstream presentation;
- first GUI/CLI/agent parity slice;
- truthful Workpoint/Trajectory/Evidence projection;
- Desktop presenter/control operations for implemented surfaces;
- continuous UIAI Engine tests plus full native shell proof.

### 4.4 Seventy-five percent — release-candidate integration gate

Required:

- stable native shell and navigation;
- Workstream-aware semantic control plane;
- major implemented workspaces integrated;
- Pi Work Surface at least bounded and testable, or an explicit release blocker;
- migration/compatibility matrix;
- installer/update/recovery posture;
- focused and broad proof green enough for a development release;
- operator approval to publish the dedicated review branch/commit.

At this gate, release initiation moves to the approved KnownHost release host.

### 4.5 One hundred percent — closure candidate

Required:

- complete agreed Desktop scope;
- required Pi/PTy and agent-control parity;
- migration/cleanup gates complete or explicitly release-blocked;
- full browser and native Evidence;
- regression, packaging, update and rollback proof;
- Spec 158 and transition closure status accurately reported.

A 100% label is not allowed while critical scope is merely hidden behind placeholders.

---

## 5. KnownHost canonical release initiation at 75%

The public repository SHALL NOT contain private hostnames, IP addresses, credentials or operator SSH details.

The MacBook agent resolves the approved KnownHost release host through the private operator runbook or fresh `agent-kb-api` data.

The approved host may be reached from the MacBook through:

- the established Tailscale path; or
- the approved direct SSH path.

At the 75% milestone, after operator approval:

1. preserve and commit the local state;
2. publish only the approved dedicated review branch/commit, never direct to `main`;
3. connect to the approved KnownHost release host;
4. fetch remote state and verify the exact approved commit;
5. ensure the required change is merged or otherwise placed on the canonical release ref according to operator policy;
6. initiate the canonical Focusa release cycle from that host;
7. use the repository’s canonical release command and workflow chain;
8. verify CI → Release → Deploy Live Daemon → audit/self-heal/watchdog;
9. confirm the development release appears in the Focusa GitHub repository;
10. record release tag, commit, workflow runs, artifacts, Evidence and rollback posture.

Canonical command, when the approved release ref is ready:

```bash
scripts/create-dev-release-tag.sh --base 0.9 --push
```

Do not replace this with a local MacBook release build, direct `cargo build --release`, manual artifact upload, partial workflow dispatch or ad hoc tag.

If the canonical pipeline fails, fix the pipeline or product through the reviewed branch and rerun from the approved host. Do not ship around the failure.

---

## 6. Milestone Evidence packet

Each 5/25/50/75/100 milestone produces:

```text
milestone percentage
commit and local branch
implemented scope
explicitly incomplete scope
browser preview URL/origin class without private credentials
UIAI Engine session/Evidence refs
native Tauri shell screenshots
console/network/diagnostic summary
focused tests
broader gates run
Rust/Node/Tauri versions
Workstream authority audit
known blockers
next milestone plan
rollback/recovery notes
```

At 75% and 100%, add:

```text
approved release commit
KnownHost release-host verification without public host details
canonical tag
GitHub Release URL/reference
workflow run references
artifact checksums/signing status
update and rollback proof
```

---

## 7. Browser preview and Tauri parity

The browser preview and native shell must consume the same authored workspace packages and semantic control contracts.

Environment-specific adapters may differ:

```text
browser preview
  HTTP/SSE or fixture adapters

Tauri shell
  local daemon, native presenter, PTY, keychain, filesystem and updater adapters
```

Do not maintain a separate mock product UI that diverges from the native application.

---

## 8. Completion law

The agent may claim a milestone only when:

- the browser preview is current;
- UIAI Engine proof is recorded;
- the full Tauri shell launches at the milestone;
- exact Workstream authority assumptions are documented;
- incomplete scope is explicit;
- no direct upstream/main/release mutation occurred from the MacBook;
- at 75%, the canonical release is initiated from the approved KnownHost host after approval.
