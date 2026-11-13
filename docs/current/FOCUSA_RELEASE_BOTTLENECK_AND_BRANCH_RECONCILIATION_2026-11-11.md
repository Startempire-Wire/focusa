# Focusa release bottleneck & orphan-branch reconciliation — agent handoff

**Recorded:** November 11, 2026 PT (November 12 UTC)  
**Status:** Source- and GitHub-run-grounded audit + one narrow proposed CI repair. **NOT** a production release, a complete release-unblock claim, canonical workset admission, or permission to bypass a required gate.  
**Scope:** Focusa current `main`, current CI and release workflow, still-open Focusa PRs and non-PR branches, plus critical related Wirebot/SOVOS/App PRs. UIAI Engine is outside this audit.  
**Exact source baseline inspected:** Focusa main `b1141339aae9` (OpenClaw awareness compatibility fixes).  
**Current public GitHub Latest:** `v0.9.207`, published 2026-10-12 with 124 assets. This does not prove every installed consumer has upgraded; draft releases and a newer main are not an official newer distribution.

## 0. Executive finding: why agents spend so long on releases

The principal constraint is **a coupled release proof chain with independent external build providers**, compounded by source/installed drift, fragile CI admission and stale branch queues. Agents often retry a large exact-SHA release when a narrow source failure, unavailable provider receipt, wrong artifact identity or control-plane limitation prevents the chain from completing. Repeated retries do **not** repair these conditions.

Do not call all old issues current outages. The workflow contains partial fixes, notably an explicit AppVeyor API queue step for #656 and Spec-178 provider-substitution logic for #403. Both still require **live exact-tag/provider acceptance**, not source claims.

### Current immediate CI failure — precise, bounded fix

Latest inspected [CI run 43142046273](https://github.com/Startempire-Wire/focusa/actions/runs/43142046273) for main `b1141339aae9`:
- **Rust:** success.
- **Release Automation (static), Structural Guards, Architecture Authority Policy, duplication gate:** success.
- **Spec Gates (strict):** failure in **Pi compaction and session-recovery release gate**, `apps/pi-extension/src/tools.ts` **lines 5984 and 5985**, ESLint `prefer-const` for variables `result` and `body`.
- **Menubar GitHub job:** skipped under current provider route; this is not independent macOS release proof.
- `npm ci` reported 15 dependency advisories (4 high). They did **not** cause this CI exit, but the lockfile audit requires separately scoped security triage.
- The rustup action logged an unsupported `--force-non-host` argument, but the reported *strict job exit* came from the two lint errors. Do not misdiagnose this as a failed Rust code build.

This PR changes exactly those two declarations from `let` to `const`, with no intended behavior change. **Acceptance still needs a fresh exact-head CI run.** The edit alone is not proof that the full release pipeline will pass.

## 1. Root-cause categories — verified, historical, and still to prove

| Priority | Bottleneck | Evidence / current reading | Exact next verification or remedy |
|---|---|---|---|
| P0 | Exact candidate-SHA CI currently red | [CI 43142046273](https://github.com/Startempire-Wire/focusa/actions/runs/43142046273), two ESLint errors | Land narrow fix after fresh Pi lint/check/tests and CI. Check subsequent blocking jobs at same candidate SHA. |
| P0 | External Windows/macOS producer and receipt choreography | [#656](https://github.com/Startempire-Wire/focusa/issues/656), [#569](https://github.com/Startempire-Wire/focusa/issues/569), [#352](https://github.com/Startempire-Wire/focusa/issues/352), [#403](https://github.com/Startempire-Wire/focusa/issues/403); `.github/workflows/release.yml` currently queues AppVeyor explicitly | For next **immutable exact tag**, prove AppVeyor build queued/ran on requested SHA, Codemagic Mac receipt bound to that SHA, Windows/macOS assets attached to exact draft, then canonical receipt and final publication. No inference from a producer's green badge or comments. |
| P0 | Draft-release intake and overloaded build queues | [#569](https://github.com/Startempire-Wire/focusa/issues/569), [open PR #520](https://github.com/Startempire-Wire/focusa/pull/520) | Resolve draft lookup/tag-name mismatch and remove redundant AppVeyor job fanout. Review #520 **against current main**; branch is hundreds of commits behind and not approved for blind merge. |
| P0 | Integrity of signed native updater and release assets | [#670](https://github.com/Startempire-Wire/focusa/issues/670), [#389](https://github.com/Startempire-Wire/focusa/issues/389) | Assert provider-native Tauri `.sig` is not overwritten by release Ed25519 sidecars. Verify `latest.json`, manifest, provenance, checksum, updater signatures and one real customer upgrade path. An asset count by itself is insufficient. |
| P0 | Production/control-plane resource exhaustion or false-green health | [#680](https://github.com/Startempire-Wire/focusa/issues/680) documented prior hard user quota and SQLite write failures despite `/health=ok`; CI workflow notes shared-OVH memory pressure | Recheck **current** writable storage/quota, daemon DB transaction success, Cargo target capacity and runner pressure. This issue is historical evidence, not proof today's quota is exhausted. Preserve customer DB/backups; do not remove data as cleanup. |
| P1 | Fragile runner/provider availability and stale identities | [#542](https://github.com/Startempire-Wire/focusa/issues/542) runner label drift, [#403](https://github.com/Startempire-Wire/focusa/issues/403) historical hosted Spec132 billing lock | Verify actual online runner labels, exact-SHA route, provider quotas and their current auth, plus fail-fast unavailable receipts. Spec178 substitution is now coded in release workflow; prove it actually satisfies the final receipt reducer. |
| P1 | Release gates cannot distinguish installed-state failures from source readiness | [#534](https://github.com/Startempire-Wire/focusa/issues/534), [#532](https://github.com/Startempire-Wire/focusa/issues/532), [#504](https://github.com/Startempire-Wire/focusa/issues/504) | Read current installed binary, daemon route inventory, license/entitlement, DB write and current exact scope; test a real consumer, not only `systemctl active` or static tool metadata. |
| P1 | Workpoint/canonical background admission can block release agents | [#660](https://github.com/Startempire-Wire/focusa/issues/660) lifecycle-stage migration block; [#702](https://github.com/Startempire-Wire/focusa/issues/702) repair-stage CLI contract defect; [#707](https://github.com/Startempire-Wire/focusa/pull/707) activation recovery separate | Recover via the owning authorized Focusa work/lifecycle operation and independent readback; never bypass admission with an ungoverned local background job or fabricate a completion receipt. |
| P2 | Stable batch/window gate and long waits add elapsed time by design | `scripts/release-gate.py`: significant score=8; window score=4; PT 11:00/16:00 windows; age fallback 24 h; `release.yml` 150-minute external receipt job; `create-dev-release-tag.sh` 10,800-second release wait | Use the existing dev/preview/rc lane for feedback and pinned internal proofs; stable keeps exact signed full-matrix acceptance. A documented owner-approved critical release may use the normal force reason to override *batch timing* only, **never** security, CI, provider, completeness or signature gates. |

### Systemic improvement principle

**Failure-classify once; repair at the owning boundary; reuse exact-SHA proof; attempt full release only after its prerequisites are ready.** This is an agent workflow correction, not a new runtime, scheduler, ledger or alternate publication path.

## 2. Release DAG and specific reason long waits occur

```text
scope + exact current main + version-policy + candidate source/manifest
    │
    ├── exact-SHA CI (Rust / Pi strict / source contracts)
    │        └── BLOCK NOW: two prefer-const lint errors on source main
    │
    ├── native release gate / stamped exact immutable tag
    │
    └── Release workflow:
           PR inclusion + candidate CI receipts + stable version contract
           └── draft GitHub Release, pinned tag/SHA
                 ├── OVH Linux binaries + Pi/installers/generated clients
                 ├── AppVeyor Windows matrix + signed exe/NSIS/MSI
                 └── Codemagic macOS binaries + signed app/dmg/updater
                        ↓ exact provider asset/receipt reconciliation
                    integrity, signature, source/provenance, OTA/readback
                        ↓
                 publish **one** verified complete release
                        ↓
                  independently test actual installed customers/daemon
```

The current workflow explicitly uses `queue-appveyor-windows`, `rust-check`, `final-release-gap-gate`, `create-release`, `external-menubar-receipts` and other publisher stages; `scripts/wait-for-external-release-assets.py` has a default 45-minute bounded wait, while the GitHub external menubar receipt job permits 150 minutes. **The wait length is a symptom.** If wrong/absent assets are expected to arrive from a provider that cannot queue or upload, repeated polling merely prolongs a deterministic failure.

A fast local Rust compile or a successful `cargo build` **does not** equal a validated stable GitHub release. Conversely, failure to publish every supported binary does **not** prove that a carefully bounded internal source build cannot be tested. Represent those states separately.

## 3. Unmerged Focusa PRs and agent fix branches

Observed **14 open Focusa PRs** (GitHub live listing, November 11 PT). The most relevant release/agent-correctness candidates:

| PR | Branch | Source status and disposition |
|---|---|---|
| [#707](https://github.com/Startempire-Wire/focusa/pull/707) | `fix/592-missing-poll-credential` | Draft activation repair; compare with main shows **1 ahead / 7 behind**. Five bounded tests reportedly passed, whole Rust integration still had unrelated failures. Independent rebase/review/acceptance first. |
| [#655](https://github.com/Startempire-Wire/focusa/pull/655) | `fix/533-preserve-lifecycle-on-refresh` | Workpoint lifecycle fix; **2 ahead / 243 behind**. Assess exact data-loss behavior and current code before cherry-pick or reimplementation. |
| [#633](https://github.com/Startempire-Wire/focusa/pull/633) | `fix/431-noninteractive-operator-context` | Avoid terminal-stealing interactive shell; **2 ahead / 303 behind**. Reverify Pi startup against current main. |
| [#632](https://github.com/Startempire-Wire/focusa/pull/632) | `fix/611-self-model-read-diagnostics` | Typed recovery for read-only cognition; **2 ahead / 303 behind**. Re-evaluate tool/API parity and current #709 awareness integration. |
| [#629](https://github.com/Startempire-Wire/focusa/pull/629) | `fix/384-daemon-inactive-recovery` | Actionable daemon failure states; **4 ahead / 305 behind**; do not replace newer main’s broader auth/errors with stale copy. |
| [#635](https://github.com/Startempire-Wire/focusa/pull/635) | `fix/393-upgrade-lifecycle-parity` | CLI upgrade recovery; **2 ahead / 303 behind**. Confirm source/client parity still missing. |
| [#520](https://github.com/Startempire-Wire/focusa/pull/520) | `fix/352-appveyor-artifact-intake` | Candidate exact Windows artifact intake / no external GitHub write token; **7 ahead / 814 behind**. High stale-branch risk; rederive minimal fix on current release chain, do not merge wholesale. |
| [#634](https://github.com/Startempire-Wire/focusa/pull/634) | `fix/618-generated-ui-ladder` | Mission Canvas improvement; **2 ahead / 303 behind**, not required to fix present CI lint. |
| [#110](https://github.com/Startempire-Wire/focusa/pull/110) | `feature/spec-135-context-connectors-b2` | Very large historic feature PR, **122 ahead / 2924 behind**, 300-file compare cap; unsafe as one-shot release merge. Reconcile slice-by-slice against current main. |
| [#76](https://github.com/Startempire-Wire/focusa/pull/76) | `local/work-loop-completion` | Very large work-loop PR, **170 ahead / 3284 behind**, 300-file compare cap; treat as historical source/evidence, not directly mergeable release patch. |
| [#604](https://github.com/Startempire-Wire/focusa/pull/604), [#120](https://github.com/Startempire-Wire/focusa/pull/120) | Docs-only proposals | Not proof of runtime closure. #120 is stacked on #110; verify base before attempting integration. |

Additional open Focusa PRs: [#102](https://github.com/Startempire-Wire/focusa/pull/102) Mission Canvas, [#123](https://github.com/Startempire-Wire/focusa/pull/123) Menubar deep links; prioritize them by measured current consumer need, not opening date.

**Branch-only search:** Focusa branch listing returned **at least 300 refs across the first three 100-item pages**, including many `self-heal/*`, `fix/618-*`, release and archival branches. This is **not** evidence of 300 independent unmerged fixes. Before an agent claims abandoned work: compare `main...branch`, inspect changed files/test evidence, look for equivalent changes already on main, group duplicates and distinguish stale history from genuinely novel functional deltas. Never bulk-merge a `self-heal` branch or prune other agents’ active work without owners/review.

### Cross-repository open PRs that influence runtime claims but are not required CI fixes

- [Wirebot Core #82](https://github.com/Startempire-Wire/wirebot-core/pull/82) — **draft** Fusion Engine skill/Discord implementation, not installed native runtime proof.
- [Wirebot Core #88](https://github.com/Startempire-Wire/wirebot-core/pull/88), [SOVOS #35](https://github.com/Startempire-Wire/agent-driven-life-business-os/pull/35), [#37](https://github.com/Startempire-Wire/agent-driven-life-business-os/pull/37), [Wirebot App #21](https://github.com/Startempire-Wire/Wirebot-App/pull/21), [#23](https://github.com/Startempire-Wire/Wirebot-App/pull/23) — integration/docs/UX handoffs. Treat design proposals as separate from Focusa stable release blockers.
- Broader Startempire-Wire open-PR search returned **40 PRs** across multiple repositories. Do not make a release contingent on all unrelated PRs becoming merged.

## 4. Minimal agent execution sequence to reach the next truthful release

1. **Freeze candidate identity:** capture `main` SHA, current `Cargo.lock`, actual installed Focusa version/hash, intended consumer versions, supported OS release surface and current external-provider health. Read the existing [release strategy](https://github.com/Startempire-Wire/focusa/blob/main/docs/release-strategy.md), [canonical pipeline](https://github.com/Startempire-Wire/focusa/blob/main/docs/canonical-live-release-pipeline.md), [release workflow](https://github.com/Startempire-Wire/focusa/blob/main/.github/workflows/release.yml) and [production consistency policy](https://github.com/Startempire-Wire/focusa/blob/main/docs/current/PRODUCTION_CONSISTENCY_POLICY.md).
2. **Fix only the first active deterministic blocker:** for `b1141339aae9`, correct the two `prefer-const` errors in `apps/pi-extension/src/tools.ts`; do not rebuild the entire release to diagnose them. Run Pi `npm run check`, `npm run lint`, `node --test tests/*.test.mjs`, then exact-head CI and independent consumer proof. This PR contains that narrow candidate.
3. **Check release-provider readiness before tagging:** proactively query current AppVeyor project/queue, Codemagic workflow and signing assets, runner labels/free space/writable DB, current artifact intake/draft lookup and native updater signature policy. If a provider or authority route is unavailable, emit a typed blocker and stop this release attempt before consuming a 45–150-minute waiter. Do not mislabel offline providers as a green stable release.
4. **Triage release-critical branches only:** classify #520 and #707 as source-only until current-main rebase or bounded patch plus tests; separately validate #655, #632/#633/#629 and any live Workpoint blocker. Preserve independent feature PRs; no giant reconciliation merge to unblock a tiny CI issue.
5. **Run the smallest legitimate channel:** dev/preview/rc artifact or pinned internal build for fast feedback with explicit limitations. Stable uses exact candidate CI, signed full platform matrix, immutable tag, verified provider receipts, complete manifest and OTA updater. `create-dev-release-tag.sh --push --no-wait-ci --no-wait-deploy` is a documented supported nonwaiting mechanism; do not publish a stable tag while CI is failing or treat a dry-run as a release.
6. **Prove an actual installation:** old→new update on representative Linux/macOS/Windows, signed installer/checksum and trust root, daemon health **and DB write**, CLI/Pi endpoint parity, Workpoint/CallGraph route and Focusa-awareness/OpenClaw native tool discovery. Record exact consumer/receiver evidence and rollback.
7. **Close only verified issues:** maintain one release evidence packet mapping each blocker to source fix SHA, test log, installed version, independent receiver and issue closure. Do not close a task merely because a PR was merged or a job exited 0.

## 5. Targeted measurable improvements to release throughput

Do not replace Focusa's current release authority, CI provider owners, signed update trust, or workset/Beads. Improve the existing pipeline:

- **Pre-tag readiness report** with one result per gate: `source_ci`, `signed_entitlement`, `workpoint_admission`, `writable_storage`, `windows_provider`, `macos_provider`, `linux_build`, `draft_asset_intake`, `updater_signature`, `installed_consumer`. Each returns `ready|blocked|unknown` + exact reason/evidence + one next action. No false-green on unknown.
- **Fail-fast provider preflight** before long release pollers; reuse exact-tag provider receipts and recheck immutable revision, rather than issuing duplicate 150-minute waiters or auto-resetting correct state.
- **Incremental acceptance:** focused source/Pi check first, full locked product and provider matrix only after upstream gates are green. Preserve cross-platform signed public release as an all-or-nothing operation.
- **Release journal metrics:** elapsed time per stage, queue latency, build time, idle polling, retries, failure class, bytes/resource usage, unchanged-SHA reruns, signature/asset identity defects, and latest published vs installed version. Compare *per accepted release*, not raw build count.
- **Stale-PR reconciliation:** select current source delta with tests and no lost protections. Do not demand closure of unrelated specifications before a narrowly scoped patch release.

## 6. Official build vs official stable release: current decision

**Current main source build:** Rust CI was green on `b1141339aae9`, so a bounded pinned developer/native build is plausible; independently run it on the authorized build host and verify actual installed command/daemon. Rust-green alone is not confirmation that the exact requested build has been produced.

**New official public stable release:** **NOT READY / NOT PROVEN** at this audit. Strict Pi CI was red, several potentially release-relevant source branches remain unmerged, and historical provider/signature/draft failures require current end-to-end acceptance. Do not claim that all 14 PRs or every open P0 must merge; decide the exact release acceptance scope from current contracts and priority, with genuine proofs.

**Definition of done for this handoff PR:** The two lint declarations are corrected in source; this document gives a source-qualified, non-bypass route to a green CI/release attempt. CI and live provider/build/deploy acceptance are **separate downstream work**, not finished by the documentation merge.

## 7. Handoff controls for autonomous agents

- Preserve canonical repo/worktree, customer data, existing pending release drafts, active agents' branches and all signing credentials. Never clean `/usr/local/lib/focusa` SQLite or production user data to regain build capacity.
- Operate on approved OVH build hosts; KH is the production/control plane under repo `AGENTS.md`. Release workflow/provider selection must be current verified topology, not the agent's guessed localhost.
- Check exact authority/tenant/project scope before mutations; a blocked lifecycle/admission state is a real blocker, not permission for `bd close --force` or ungoverned job launch.
- Avoid blind `--force-release`. That flag may bypass only the scored batch/time gate with an explicit authorized reason; never skip license/security/trust/CI/receipt completeness checks.
- Observe source, signed-artifact, installed-runtime and **actual receiver** outcomes separately. Keep unsuccessful attempts visible rather than rewriting them as completed.

## 8. Free-credit alternative build provider pilot — owner-directed addition

**Purpose:** Reduce dependence on GitHub-hosted compute while the account billing restriction remains. Test **Semaphore for native macOS**, **GitLab CI for Windows**, and keep the **existing OVH builder for Linux**. Retain AppVeyor and Codemagic as incumbent fallback/build producers. This is an exploratory pilot; it does **not** change the approved Focusa release workflow, build/signing authority, trust roots, GitHub source repository or supported platform set.

**Published allowance context, subject to live eligibility and provider changes:** Semaphore has advertised a USD $15 monthly credit allowance and macOS pricing near $0.09/minute; GitLab Free has advertised 400 compute minutes monthly and a Windows beta runner. **No account entitlement, macOS chip/runner availability, Windows runner access, free credit balance, geographical access or project eligibility has been verified for this Focusa owner.** Check provider dashboards and up-to-date terms *before* promising free cross-platform capacity. Never incur a charge or add a payment method without explicit authorization.

### Pilot work packets and owner outputs

| Packet | Action | Evidence / success condition |
|---|---|---|
| ALT-01 — Eligibility/cost | Verify Semaphore organization free credits, native macOS runner architecture, duration/parallelism, public/private repo terms, billing guard; verify GitLab namespace free minutes, Windows beta availability, runner OS/architecture and eligibility. Record expected credit use; impose zero paid overage where possible. | Provider-specific `available / unavailable / unknown`, cost/minute and remaining budget; missing eligibility blocks that lane only. |
| ALT-02 — Source integrity | Connect **read-only** repository checkout or short-lived scoped fetch to GitHub; pin identical immutable Focusa commit SHA, Cargo.lock, Node/Bun lockfiles and build recipe for both providers and OVH baseline. No branch-head drift, no customer secrets or production credentials on trial pipelines. | Both providers attest exact commit, dependency revisions, toolchain and runner identity. |
| ALT-03 — Semaphore/macOS | Try native Rust CLI, daemon, session runner, TUI and Tauri Menubar app builds for required macOS architectures. Explicitly distinguish ordinary build success from Apple developer signing/notarization and native Tauri updater signatures; handle latter only through approved credential custody or canonical final signing process. | Native macOS artifacts, target architecture/version, build logs and checksums; signing/native-update verification marked independently, not inferred. |
| ALT-04 — GitLab/Windows | Try Windows x64 native Rust binaries and the actual Tauri NSIS/MSI packaging flow. Check architecture and native runner availability **before** assuming Windows ARM64 coverage; if unsupported, leave ARM64 with existing proven provider or independently proven cross-build path. | Windows binary + installer compile/link/installer smoke, artifact hashes, source provenance; missing ARM64 remains explicit. |
| ALT-05 — Existing release intake | Export build artifacts/receipts via a short-lived, controlled transfer to the **existing Focusa release authority**. Prefer canonical release workflow pull/intake over giving external CI long-lived GitHub release-write tokens. Verify SHA, provenance, artifact families, signatures, version stamps, immutable candidate tag and duplicate/overwrite refusal. | Separate source-producer, external builder and canonical publisher audit records; no broadened authority. Do not silently swap Spec-178 provider while contract still names Codemagic/AppVeyor. |
| ALT-06 — Measurement/decision | Compare Semaphore, GitLab, Codemagic, AppVeyor and OVH on queue wait, execution duration, retry rate, available credits, real cost, signed package completeness and exact installed acceptance. Run one failure recovery/timeout case. | Choose `adopt / keep as backup / reject` per OS lane with dated evidence, not assumptions. |

### Pilot order and safety

1. **First:** run ALT-01 eligibility and zero-charge/overage guard. Evaluate macOS **Semaphore first** because native Apple build capacity is the scarce resource; GitLab Windows is the next independent test.
2. **Second:** invoke one **pinned-source nonpublication build** on each eligible provider. Do not recreate Focusa's entire release pipeline, move repositories, create customer releases, or change release trust roots for the pilot.
3. **Third:** validate complete artifacts, native Tauri updater signatures, exact SHA and independent install/smoke tests. Use signed production-channel release only after existing provider/receipt contract is updated and all original release invariants pass.
4. **Fourth:** report measured minutes, credits, wait time and failed jobs. Avoid idle 45–150-minute pollers when a provider is not enrolled/ready. Keep OVH Linux and the existing provider lanes operational until an alternative is independently accepted.

### Definition of pilot completion

- [ ] Actual eligibility verified for Semaphore macOS with zero unapproved charges.
- [ ] Actual eligibility verified for GitLab Windows with zero unapproved charges, or documented unsupported status.
- [ ] Same immutable Focusa SHA compiled on each available provider.
- [ ] macOS required architecture(s) and Windows x64 plus ARM64 coverage truthfully reported.
- [ ] Rust/Tauri packages independently inspected, native updater signing preserved, and representative installation tested.
- [ ] Intake uses existing source-of-truth release authority and cannot overwrite or publish unapproved assets.
- [ ] Measured cost/latency/reliability comparison and one adopt/backup/reject decision per lane recorded.

**Important:** A green pilot build is **not** a public stable Focusa release and does not justify closing the existing AppVeyor/Codemagic/upload/signature issues. The owner may decide to change the provider strategy later based on real results; no implied migration or payment is authorized here.
