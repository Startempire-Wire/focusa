# 179 — Focusa v0.9.184-dev Release Definition Ledger

**Status:** release-blocking living ledger
**Range:** `v0.9.183-dev..3ff3824ab`
**Authority:** source commits, Spec 174, Spec 177, and trusted release metadata

## Purpose

Define every behavior, contract, control, authority, and maintenance change added
since `v0.9.183-dev` in plain language. This ledger is not release notes and does
not claim unexecuted behavior. It tells operators and future agents what each
new thing means, where truth lives, how failure appears, how it is verified, and
whether compatibility changes.

Ledger-only maintenance commits touching only this file, its test/evidence, or
its bead status are excluded from the range to avoid a self-referential commit
coverage loop. Every product, CI, contract, security, or release commit remains
mandatory coverage. Final release adjudication must advance the Range endpoint
and refresh rows for later substantive commits.

## Commit coverage

| Commit | Definition | Owning source | User/operator impact | Failure meaning | Verification / compatibility |
|---|---|---|---|---|---|
| `ebe3666cf` | **Spec-gate retrigger after runner repair:** a no-product-change CI commit used to prove repaired runner shims and `/root/.local/bin` permissions. | `.github/workflows/ci.yml` history | Restarts authoritative gates after infrastructure repair. | A failed retrigger means runner/toolchain remains unhealthy; it does not mean product code regressed by this commit. | GitHub run evidence; behavior-compatible. |
| `4f957fff8` | **Manual CI dispatch:** `ci.yml` can be started through `workflow_dispatch` without a source push. | `.github/workflows/ci.yml` | Operators can verify runner recovery on demand. | Dispatch unavailable means CI control-plane/config drift. | Workflow syntax/static gate; additive compatibility. |
| `4028c0e3d` | **Unused `firstKey` binding repair:** renamed a deliberately unused destructured value to `_firstKey` so TypeScript/lint truth matches intent. | Pi-extension source in commit | No runtime behavior change. | A lint failure means source hygiene gate regression. | Pi-extension typecheck/lint; compatible. |
| `622494676` | **Stale eslint suppression removal:** deleted an obsolete `no-console` disable where warning output is allowed. | Pi-extension source in commit | No runtime behavior change; warning remains observable. | New lint failure means the current lint policy differs from source assumptions. | Pi-extension lint/typecheck; compatible. |
| `0f7a68ebc` | **North Star progressive-disclosure reconciliation:** aligns the contract test/card shape with the redesigned compact-first North Star experience. | `apps/pi-extension/src/north-star.ts` and related tests/contracts | Operator sees concise orientation first and expands details intentionally. | Contract mismatch means UI and tested projection disagree; never silently fall back to stale layout. | North Star tests + TypeScript; visual behavior change, API-compatible. |
| `bf965f81e` | **Cross-host background output preservation:** records bounded output tail durably and returns it across daemon/CLI/extension boundaries; also includes actual child working-directory behavior and portable process handling. | `background_jobs.rs`, `background_job_store.rs`, API route, CLI `bg` | Detached jobs complete with truthful bounded output and execute in requested `--cwd`. | Missing tail, ignored cwd, or monitor loss is explicit degraded/failure state—not success. | Producer/consumer/cwd tests; additive SQLite migration; older rows may have no tail. |
| `6105cff9f` | **Fail-closed AppVeyor release provider:** adds a second provider capable of expressing the canonical release gates without deleting artifact, identity, signature, provenance, deploy, receipt, or rollback requirements. | `.appveyor.yml`, Spec 177, release topology, installers/trust tests | Releases can proceed when GitHub is unavailable only if AppVeyor proves the same authorities. | Missing secret, artifact, signature, exact SHA, BYOC deploy authority, or receipt blocks release. | AppVeyor validator + Spec177 tests; provider-additive, not a weakened fallback. |
| `00d56835f` | **AppVeyor release-closure work item:** durable bead tracking the emergency-provider completion conditions and blockers. | `.beads/issues.jsonl` | Gives agents one auditable operational closure record. | Open/blocked status means release work remains; it is not product runtime failure. | Bead ledger; no runtime compatibility effect. |
| `febb56a47` | **Spec 51 debug-redirection removal:** removes leftover `/tmp/spec51-debug.log` output redirection from the proxy parity test. | `tests/proxy_mode_b_parity_test.sh` | Test failures remain visible in canonical output instead of a hidden temporary log. | Failure now surfaces directly and must be fixed. | Spec 51 parity test; no product behavior change. |
| `9bbb448c6` | **Emergency AppVeyor signer identity pin:** explicitly trusts the bounded Google service-account identity used for emergency Sigstore provenance. | trust-key config, installers, Spec 177 | Installers can verify emergency-provider provenance from the named identity. | Any different issuer/subject is untrusted and blocks install/release. | Trust metadata tests; additive migration identity, not wildcard trust. |
| `dd85f01b1` | **Release signing-key rotation:** installs a new Ed25519 release-verification key and new Tauri updater public key after proving old private keys unrecoverable. | trusted-key config, Tauri config, installers/tests | New artifacts use durable keys stored first in Bitwarden; existing desktop installs need one manual reinstall for the new Tauri key. | Old/private-key use, signature mismatch, or absent durable authority blocks release/update. | Key smoke verification + trust tests; intentional OTA compatibility break for already-installed old Tauri clients. |
| `0ebb3003d` | **Bead ledger synchronization:** records the latest release/key-rotation task state. | `.beads/issues.jsonl` | Preserves operational handoff. | Stale bead notes mean planning drift, not runtime failure. | JSONL validity / `bd`; no runtime effect. |
| `0794bbe06` | **v0.9.184-dev surface stamp:** updates authoritative package/version surfaces to the pending development version. | Cargo/package/Tauri/README/generated contract surfaces | All shipped components identify the same release candidate. | Version disagreement blocks preflight and release. | canonical version-surface gate; intended version boundary. |
| `e8f6e7999` | **Cargo.lock version synchronization:** updates workspace package entries in the lockfile after stamping. | `Cargo.lock` | Native builds resolve the stamped workspace consistently. | Stale lock entries block clean-tree/preflight or produce version inconsistency. | Cargo metadata/build; dependency versions otherwise unchanged. |
| `04d9d4165` | **Distribution-manifest source refresh:** binds generated distribution metadata to the current release commit chain. | Spec141 distribution manifest | Provenance points at the actual candidate source rather than a stale commit. | Stale `source_commit` blocks deterministic preflight. | manifest freshness gate; metadata-only compatibility. |
| `3fd7898c4` | **Spec 174 workforce Chrome MVP decomposition:** defines the reliable observe→orient→create→orchestrate→audit/recover vertical slice and its 18-node dependency graph. | Specs 174 and 178; Spec174 epic | Makes Chrome a governed workforce control/orientation client while daemon remains authority. | Missing dependency/proof keeps the release held; specification alone is not shipped behavior. | Spec174 static gate + eventual five production proofs; additive feature plan. |
| `965322848` | **Spec 174 contract-freeze gate:** adds machine assertions for contract names, permissions, 18 nodes, line bounds, verdict fields, and DAG acyclicity. | `tests/174_workforce_extension_spec_test.py` | Prevents weak executors from silently changing architecture or scope. | Gate failure means spec/taskgraph drift and blocks implementation. | `PASS: Spec 174 contracts and 18-node weak-model task graph are frozen`; test-only. |
| `3ff3824ab` | **Spec 174 contract-freeze closure:** closes bead 174-00 after the static gate and evidence pass. | `.beads/issues.jsonl` | Unblocks only nodes 174-01, 174-02, and 174-03. | Closure does not claim extension implementation; later nodes remain open. | Bead graph/closure evidence; no runtime effect. |

## Feature and contract glossary

| Term | Plain-language definition | Authority / proof |
|---|---|---|
| Progressive disclosure | Show the smallest useful North Star orientation first; reveal supporting detail only on request. | North Star source/contracts at `0f7a68ebc`. |
| Background job monitor | The `focusa bg run` process that owns child execution, status transition, logging, and completion submission. | Spec 165 and background job code. |
| `output_tail` | Bounded final job-output excerpt stored in the completion envelope for cross-host consumers; not the whole log. | Background job store/API/CLI tests. |
| `--cwd` | Requested child working directory; it must be passed to the actual spawned command, not merely recorded as metadata. | Background job runtime/cwd tests. |
| `monitor_lost` | Durable failure status meaning the monitor died before canonical settlement; never equivalent to running or success. | Spec 165/background status reconciliation. |
| AppVeyor emergency provider | Secondary CI/release authority used when canonical GitHub execution is externally unavailable, while preserving release truth. | `.appveyor.yml`, Spec 177. |
| Fail-closed release lane | A lane that stops when required evidence/authority is unavailable rather than skipping a gate or claiming partial success. | Spec 177 and tests. |
| Hosted lane | AppVeyor-managed worker used for bounded build/test/artifact work. | Release topology. |
| BYOC production lane | Operator-connected AppVeyor worker/cloud named `focusa-production` required for live deployment authority. | Spec 177; currently a blocker until connected. |
| Exact-SHA artifact | Artifact built from and provenance-bound to the exact tagged commit, not a nearby branch head. | Release topology/manifests. |
| Provider-neutral receipt | Release/deploy evidence with a stable contract independent of GitHub or AppVeyor implementation. | Spec 177 ledger/provenance sections. |
| Emergency signer | `aegis-drive-sync@tech-empire-258307.iam.gserviceaccount.com`, temporarily trusted for Google OIDC/Sigstore provenance. | Trusted release keys + installer tests. |
| Release Ed25519 key | Focusa's detached release-metadata/artifact signature authority; current key id `focusa-release-2026-08-24-4ed9c92b`. | Trusted release keys; private authority in Bitwarden. |
| Tauri updater key | Minisign-compatible key embedded in desktop clients to authenticate OTA payloads; current key number `5f216ee7de6246e8`. | `tauri.conf.json`; private authority in Bitwarden. |
| Manual reinstall boundary | Existing desktop clients embedding the old Tauri public key cannot authenticate new-key OTA; one manual trusted reinstall migrates them. | Rotation decision + updater trust model. |
| `source_commit` | Distribution-manifest field identifying the source commit represented by generated release metadata. | Spec141 distribution manifest freshness gate. |
| Chrome observation | Explicit active-tab title/URL/origin capture only; not page body, cookie, form, history, screenshot, or DOM capture. | Spec 174 §§2.1, 9.2. |
| Orientation packet | Reviewed objective, exclusions, active-tab observation, project/work-item/role bindings used to form a bounded session mission. | `focusa.browser_orientation.v1`. |
| Workforce roster | Authorized projection of daemon-native Silent Sessions; never a browser-owned agent database. | `GET /v1/silent-sessions`. |
| Durable approval issuance | Additive endpoint that derives and persists exact action consent server-side before approval-required session mutation. | Spec 174 §6; implementation not yet landed. |
| Exact target | Current `{session_id, run_id, generation}` required to prevent controlling a stale/restarted run. | Spec 133 lifecycle contracts. |
| Cursor replay | Reconnect behavior that resumes durable SSE history from the last successfully rendered sequence before joining live tail. | `focusa.stream_event.v1`, Spec 174 §11.2. |
| Observe→orient→create→orchestrate→audit/recover | Spec 174 MVP golden loop and release-blocking outcome. | Specs 174/178. |
| Release held | No `v0.9.184-dev` tag, publication, or deployment may proceed until Spec174 final verdict and the pre-existing release gates pass. | Spec178 §11 and operator directive. |

## Current truth at this ledger endpoint

- `v0.9.184-dev` tag: not created.
- Release: held by operator.
- Spec 174 implementation: not yet shipped; only contract node 174-00 is closed.
- AppVeyor production BYOC: still required before live deployment.
- GitHub Actions billing lock: external blocker remains unless separately resolved.
- Rotated key private authority: Bitwarden first; CI secret stores are write-only deployment copies.
