# Spec 187 — Autonomous Agentic Flow State: cohesive next-stable release workset

**Status:** operator-requested outcome and source workset prepared; implementation, runtime admission, credential-use authorization and installed acceptance are NOT implied.  
**Task owner:** `focusa-ak5sv` — Autonomous Agentic Flow State release workset; 18 dependency-ordered child tasks.  
**Normative authoring inputs:** [requirements](contracts/spec187/requirements.v1.json), [release workset](contracts/spec187/release-workset.v1.json). These define acceptance and task scope, not mutable execution state.  
**Existing program owners:** #252 (complete agent runtime), #253 (one cohesive lifecycle), #101 (managed documentation/skills convergence). Their broader acceptance remains intact.

## Operator outcome and authority

On 2026-10-15 the operator requested a cohesive workset for the next release: continuous authorized execution, including after compaction; available approved secrets and tools used effectively; no lost intent, redundant approval rituals, false blockers or avoidable reply-stops; related issues closed only on evidence; documentation, deep skills and Pi guidance kept cohesive.

This request is the source of the workset assembly and release-outcome scope. Existing full-stable-release and approved KH/OVH rollout grants remain bounded to their actual destinations. Issue authorship, this document, a stored secret, a tool listing or a model-generated plan is not a new grant. No new provider/account, secret reveal, dependency installation, paid automation, demo disruption, foreign writer takeover or unlimited budget is implied.

**Flow invariant:** while an eligible, authorized, in-scope action and execution budget exist, advance it and verify its result without an unnecessary human restart. Preserve the purpose and exact next action across compaction, crashes, temporary failures and legitimate pauses. A model turn ending is not a mission-ending decision.

“Never stopping” means **no avoidable loss of momentum**, not bypassing safety, legality, ownership, consent, explicit stop, genuine unavailable authority or an exhausted budget. A real pause is correct behavior when it is specific, truthful, durable and resumable.

## 1. What this is — and is not

Human flow research emphasizes clear goals, timely feedback and a suitable challenge/capability relationship: [A Scoping Review of Flow Research](https://www.frontiersin.org/journals/psychology/articles/10.3389/fpsyg.2022.815665/full). Here the term is an operational quality contract, not a claim about an agent's subjective experience.

The feature composes existing owners: Specs 79 (loop), 97 (recovery primitives), 100 (bounded context), 103/155 (typed execution), 125 (accepted purpose), 130/130a (compaction), 133 (sessions), 137 (time/budget), 138 (evidence-grounded learning), 140 (instructions/approval), 141 (tools/skills/docs), 143/149 (release/workset), 156 (credentials), 158/186 (durable-state/restore).

There is **no new scheduler, authority kernel, policy store, secret vault, outcome ledger or shadow daemon**. Any “flow” presentation is a projection of the existing work, authority, resource, executor and evidence records. It cannot grant permission or declare completion.

## 2. Verified starting point and admission boundary

The authoring worktree is `/tmp/focusa-cohesion-release-20261014`, a Git worktree of `/home/wirebot/focusa`, origin `https://github.com/Startempire-Wire/focusa.git`. At preparation the authoritative daemon is unavailable. Therefore no new canonical Workset/run, writer lease, budget renewal or live attachment is asserted here.

Existing source repairs for replay, complete instruction retention, exact native anchors and contract diagnostics have test evidence; they are not installed Flow State acceptance. The unpublished 0.9.208 source candidate and prior release evidence must be reconciled before publication. The version is selected by the canonical controller; this workset does not manually promote a version or release manifest.

The project-local task queue records the operator-requested decomposition now. Source preparation and named owner-authorized recovery do not fabricate a healthy daemon. When the actual authority is restored, use the existing Spec149/155 creation/admission owners with a freshly verified project/workstream, task membership, execution bindings, approval references and budgets. Generate real IDs there. Historical orientation is evidence, not a current lease.

Avoid a repair circularity: authoring, source verification and the already-authorized runtime-repair route must not depend on claiming the broken runtime admitted this new workset. Candidate sandbox proofs remain sandbox proofs. Installed replay, admission and actual agent continuation are mandatory later acceptance, not retroactive execution permission.

## 3. Completion contract and issue boundary

All 18 requirements in `contracts/spec187/requirements.v1.json` are required; a native Workset may settle only after its canonical dispositions are `met` with linked evidence. Each requirement needs versioned contract, producer, actual consumer, claimed cross-version and installed end-to-end proof. Structural validation alone does not prove dispatch or installation.

The 15 closure candidates are:

| Issue | Flow failure | Owning implementation steps |
|---|---|---|
| #685 | Replay prevents a usable runtime | FLOW-01 |
| #660 | Correct work cannot pass lifecycle/scope admission | FLOW-02, FLOW-10 |
| #657 | Enabled loop does not execute and drains budget | FLOW-04, FLOW-10 |
| #679 | Session requests queue without proven runner delivery | FLOW-05 |
| #683 | Complete instructions/restrictions do not survive | FLOW-03, FLOW-10 |
| #684 | Accepted goal restore does not restore actual history | FLOW-03, FLOW-10 |
| #261 | Foundational authorization/recovery/error paths contradict each other | FLOW-02, FLOW-09, FLOW-11 |
| #680 | Resource exhaustion breaks persistence/execution while health misleads | FLOW-01, FLOW-04, FLOW-11 |
| #682 | Diagnostics manufacture stale-contract blockers | FLOW-11 |
| #686 | Reporting dependency prevents a runtime-repair release | FLOW-11, FLOW-14 |
| #299 | Scoped broker/use/delegation/injection contracts are not fully delivered | FLOW-06, FLOW-07, FLOW-08 |
| #526 | Spec156 executable delivery and full acceptance remain incomplete | FLOW-06, FLOW-07, FLOW-08 |
| #410 | Restored renewable MFA connector passes without functional readiness | FLOW-08 |
| #417 | Vault access depends on hardcoded credential material | FLOW-07 |
| #590 | Supported signed-authority renewal is broken | FLOW-09 |

This is a **closure candidate list, not a promise to paper-close 15 tickets**. Audit every original acceptance criterion, related approved spec requirement and remaining consumer/device/provider scope. #299/#526 are one credential delivery lane but separate tickets; #261 is broader than one error message. If a bounded flow slice passes while their remaining approved scope is unmet, keep the issue open and record exact remaining work. #101/#252/#253 do not close automatically. Existing installer, device-poll, native-platform, quota, backup and entitlement issues become explicit dependencies if the actual claimed consumer path requires them; they are not silently waived or imported as unrelated work.

## 4. Dependency-ordered workset

The authoring JSON is the single machine-readable source for exact task IDs, scopes, done-conditions, issue mapping and dependencies. The table is a readable routing view, not a second execution authority.

| Step | Deliverable | Depends on |
|---|---|---|
| FLOW-00 | Workset contract, traceability, task graph and validation | — |
| FLOW-01 | Replay-safe runtime recovery and truthful storage/resource readiness | 00 |
| FLOW-02 | Valid scope/lifecycle admission and retained-grant revalidation | 01 |
| FLOW-03 | Complete requests and actual accepted-goal restoration | 00 |
| FLOW-04 | Real loop transport, ready-frontier progression and bounded accounting | 02 |
| FLOW-05 | Actual protected-runner start/input/output/terminal delivery | 02 |
| FLOW-06 | Exact credential requirements, grants and real protected-use enforcement | 02 |
| FLOW-07 | Approved vault route, no hardcoded login, value-free injection | 06 |
| FLOW-08 | Functional restored connector and renewable one-use MFA injection | 06 |
| FLOW-09 | Provider-issued renewal and preserved existing rights | 02 |
| FLOW-10 | Native post-compaction/restart continuation without another prompt | 03,04,05,06 |
| FLOW-11 | Effective capability selection and typed bounded recovery | 02,04,06,09 |
| FLOW-12 | Existing docs, skill registry/mirrors and Pi guidance converge | 03–11 |
| FLOW-13 | Actual candidate-agent reference mission and negative matrix | 01,07,08,09,10,11 |
| FLOW-14 | Whole-issue evidence audit and all mandatory release readiness | 12,13 |
| FLOW-15 | Canonical full signed stable release and approved deployment | 14 |
| FLOW-16 | Installed real-agent proof and whole-issue closure | 15 |
| FLOW-17 | Canonical requirement settlement, outcome learning and ongoing upkeep | 16 |

Source readiness in an implementation step permits its dependent source work; it is not installed requirement settlement. FLOW-00 may finish as a preparation task without claiming the feature works. FLOW-16/17 are mandatory: a passed unit test, source push, signed asset or written instruction file is not the outcome.

### Call stack and authority owners

Operator instruction → native harness request/source-turn capture → existing scope/approval/admission owner → CallGraph/loop eligible frontier → real executor/runner → scoped credential broker and tool adapter → existing durable records → observed consumer outcome → canonical evidence/requirement settlement → next eligible action.

Use the existing typed call-stack-design operation when healthy and attach its design to the verified current Workpoint. The source graph compiler only projects these planning dependencies into the actual Spec155 structural type. It performs no admission, scheduling, dispatch, grant creation, budget reset or Workset settlement. The checked-in graph is explicitly a synthetic structural fixture; real scope and authority must come from the canonical owner at admission.

## 5. Retained approval and secret-use envelope

For each executable operation, resolve the existing authorized principal, project/workstream/frame/attempt, allowed operation and effects, exact consumer and target, credential role/reference, exposure mode, validity/use count, challenge support, revocation, budget and evidence requirements. Existing approved grants are revalidated automatically after recovery; mere presence of an approval string is insufficient. Fresh source-turn corrections win within their actual scope.

Use canonical Agent-KB and approved broker/vault references, not a parallel credential inventory. Check documented execution user/host and existing authenticated routes before claiming access unavailable. `credentials_verify` is advisory; actual broker enforcement grants or denies use. Secret values, OTPs, cookies and access/refresh tokens never enter this document, model context, argv, screenshots or receipts. Test with synthetic leakage canaries; real injection evidence contains value-free handles and outcomes only.

Renewable SMS/email OTP, TOTP, passkey/security key and normal OAuth flows remain limited to their granted provider/target/consumer. OTP permission never becomes inbox permission. Recovery codes and finite break-glass assets are forbidden to agents and all delegated processes. Preserve supported Android/iOS and customer boundaries; only claim supported-device/provider coverage with actual proof. Unknown renewability, new account choice and irreducible consent produce a precise resumable pause, not unsafe improvisation.

Provider authority/entitlement renewal is distinct from secret transport. Preserve lifetime/customer/developer rights and exact node binding. No local key, trust-root, lease or entitlement fabrication is a recovery path.

## 6. Reference mission and measurable acceptance

Use an approved isolated candidate environment first, then the actual approved installed destinations and each harness/device/provider profile claiming this capability. A real model, actual native SDK/session, real tool execution and protected approved broker use are required for positive end-to-end acceptance. Synthetic contracts/fixtures prove boundaries; they do not prove live access, delivery or model consumption.

**Reference mission:** start one sufficiently specified, pre-approved bounded task with dependencies, an authenticated external operation, independent ready work, consumer verification and an authorized delivery destination. Complete the eligible stages without another continue prompt. Force two successive native compactions, one session/process restart and a model/session transition. Inject transient tool/transport and broker-readiness failures. Reconcile effects, revalidate valid grants and continue. Then prove explicit correction, consent and budget pauses work correctly and resume only after their condition is legitimately resolved.

The implementation adds/extends tests in the existing owning suites; FLOW-13 selects exact supported SDK/provider/harness/device profiles from live capability contracts and records revisions. Do not replace the model with an echo fixture or count filtered zero-test targets as execution.

| Scenario | Required proof |
|---|---|
| FLOW-T01 | Full mission, actual model/tools, two compactions, complete intent and automatic next eligible action |
| FLOW-T02 | Long request and restrictions; exact anchor revision/digest; corrupt/missing state cannot substitute another generation |
| FLOW-T03 | Process/daemon crash; durable replay; original identity/ownership; reconcile unknown effects and resume |
| FLOW-T04 | Model/native session change and native instruction reload; same purpose; actual delivered-context consumption |
| FLOW-T05 | Wrong project/tenant/workstream, stale writer/frontier, expired/revoked grant and explicit operator correction stay fenced |
| FLOW-T06 | Read-only discussion/independent work proceeds where permitted; missing introspection is not stale-contract or absent-work proof |
| FLOW-T07 | Protected broker use plus renewable exact-bound MFA; values never enter model/log/argv/evidence |
| FLOW-T08 | Wrong consumer/target/generation, used/expired challenge, missing consent and unknown renewability deny correctly |
| FLOW-T09 | No attached transport: truthful not-running state, correct accounting, no silent budget renewal |
| FLOW-T10 | Real runner receives start/input; stale generation, duplicate delivery, cancellation and stop fence queued effects |
| FLOW-T11 | Transient tool failure: bounded repair/retry, effect reconciliation, independent ready work and retained exact next action |
| FLOW-T12 | Genuine consent/auth/provider/resource/budget blocker: precise durable pause; legitimate later resolution resumes work |
| FLOW-T13 | Quota/disk pressure and failed durable write: truthful health, intact data and supported recovery; no unrelated deletion |
| FLOW-T14 | Delayed callbacks, crash-after-side-effect and replay cannot duplicate or misattribute accepted effects |
| FLOW-T15 | Restored connector actually delivers/injects eligible challenges; a URL/title-only false positive fails |
| FLOW-T16 | Vault consumer leakage canaries; no hardcoded login; exact-field custody; approved rotation/revoke behavior |
| FLOW-T17 | Each claimed mobile/device/cross-surface profile receives real-device proof and verified private teardown |
| FLOW-T18 | Supported provider renewal, node/account binding and lifetime/developer/customer rights; malformed and revoked cases |
| FLOW-T19 | Capability discovery selects effective appropriate tools with bounded context; registration alone is not readiness |
| FLOW-T20 | Optional reporting outage cannot form a repair deadlock; mandatory authority/quality failures remain blocking |
| FLOW-T21 | Docs/skills/generated mirrors/runtime contracts agree; native reload plus actual running-agent consumption |
| FLOW-T22 | Full release matrix, signatures, trust/install, installed mission, complete issue acceptance and canonical settlement |

Record observed start/end/elapsed time, eligible work completed, actual external/consumer outcomes, avoidable human re-prompts, false reply-stops, false blockers, duplicate effects, leakage-canary hits, unauthorized actions, recovery duration, waiting/accounting and legitimate pauses. Desired counts for avoidable re-prompts/stops, false completion, duplicate effects, leaked values and unauthorized actions are **zero** in the approved reference run. Report actual values and denominators; do not invent duration, urgency, throughput or a performance regression threshold before measuring the canonical comparable baseline.

A stop is avoidable only if a ready authorized operation and valid budget existed and no real controlling boundary applied. A necessary model-turn yield to the actual executor is not a false mission stop. Required operator consent, explicit stop or exhausted budget is not a defect to bypass.

## 7. Proof execution and rollback

Heavy work runs on OVH through the approved Focusa executor/dispatcher; KH remains control plane. Recheck actual resource and invocation safety. Reuse pinned installed toolchains/dependencies and existing serialization; do not install speculative dependencies or run heavy payloads on KH.

Authoring checks (no runtime mutation):

```bash
python3 tests/187_autonomous_flow_workset_contract_test.py
python3 scripts/compile-flow-workset.py --check-fixture
cargo test --offline --locked -p focusa-core --test 187-flow-workset-contract
```

Implementation checks reuse the current package/owner suites: actual compaction/preload API consumers, replay regressions, native Pi lifecycle/request-binding/session tests, credential/SMS/entitlement/runner tests, strict Spec gates, skill generator/parity and exact consumer tests. Add missing positive/negative cases at their existing owners. Execute full quality/release checks through the current canonical scripts; fast/static checks never substitute for strict or installed proof.

For a real graph projection, supply freshly verified scope and observed provenance to the compiler. The result still needs the canonical validate/admit/bind/dispatch owners. A source artifact cannot create a lease or reconcile an unknown runtime effect.

Rollback: preserve current source/unrecognized changes, prior signed releases, customer data, event/checkpoint history, rights and evidence. Each implementation frame identifies its exact touched owner and rollback before mutation. Roll back code/config via the supported scoped mechanism, never reset history, rewrite immutable tags, use unsigned install shortcuts, clear production databases or invent a local authority. Credential revocation/rotation and protected-process cancellation use their exact existing owners and required grants. Keep failed evidence and resume the remaining authorized frontier.

## 8. Documentation and release cohesion

Update the existing entry guide, cohesive-project-flow guide, Pi extension/skills guide and owning deep skills through `config/agent-skills-v2.json` and its generator. Reuse work-loop, Workpoint, session-recovery, tool-discovery, security/auth/licensing, evidence/outcomes and docs-maintenance skills; do not add a redundant flow skill or copy of policy. Instructions distinguish source design, installed support, canonical admission, successful execution and accepted completion.

Existing Spec141 maintenance triggers cover behavior/schema/SDK/recovery changes, pre-release sweeps and bounded between-release reviews. Keep examples, permissions, failure paths and native reload guidance current. Do not enable billing-disabled schedules, add a second watcher or claim an autonomous audit schedule was installed.

The current candidate's source repairs and CI recovery belong in the delivery evidence, but are not the whole Flow State feature. Before release, verify all source and claimed candidate proof; after trusted installation, run FLOW-16 with the actual runtime and native agent. Only then close whole accepted issues and settle the canonical workset. If a required original-spec capability or provider/device proof is missing, record the exact blocker and remaining issue—not a successful Flow State release or an invented empty queue.
