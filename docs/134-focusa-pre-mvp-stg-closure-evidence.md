# 134 — Focusa Pre-MVP STG Closure Evidence

**Date:** 2026-07-12  
**Status:** verified complete  
**Root bead:** `focusa-w26jj`  
**Release/deploy boundary: not authorized or implied.** Specs 132/133 may resume from this closure; production release remains a separate operator-controlled gate.

## Closed lanes

| Lane | Scope | Result | Primary proof |
|---|---|---|---|
| STG-0 | Baseline, requirement graph, proof budgets | Closed | Dependency-linked `focusa-w26jj.*` graph and runtime memory/version baseline |
| STG-1 | Spec 125 mandatory Trajectory/non-lazy HLT | Closed | 16 trajectory tests, 22 Workpoint tests, §15.1 static and §15.2 isolated runtime suites |
| STG-2 | Spec 130 compaction/context firewall | Closed | Packet/API/Pi/CLI tests, Context Cognition runtime, bounded subagent intake, memory telemetry and 3 GiB future-session heap guard |
| STG-3 | Specs 82/94 optimization | Closed | Memory-SLO, persistence/offload, retrieval pagination, pressure/degrade, bounded payload, profile and store-growth runtime gates |
| STG-4 | Spec 128 OTA | Closed | 14 core update tests, installer/service suites, status/runtime tests, signature/provenance and deterministic atomic rollback faults |
| STG-5 | Software currency | Closed | `config/software-currency.json`, exact Pi SDK pin, workspace all-target check, Pi lint/typecheck, UIAI health, release/version and continuous drift tests |

## Regressions corrected during closure

1. Cross-account agent runtime paths such as `/home/<agent>/.cargo` now fail Context Cognition scope validation.
2. Metacognition hard caps can be lower than defaults and cursor pagination honors the cap.
3. Default ontology world payload omits the 101-entry action catalog and stays inside the 12 KiB hot-route budget.
4. OTA rollback restores the known-good target in reverse install order and never deletes a target when no backup exists.
5. Pi SDK dependency no longer uses blind `latest`; it is pinned to the tested `0.64.0` type surface.
6. Update inventory now emits fleet drift, stale count, bounded polling interval, notification requirement, and no-blind-latest policy.

## Combined acceptance

Run:

```bash
bash tests/pre_mvp_stg_combined_gate.sh
cargo test -p focusa-api routes::update::tests --quiet
cargo test -p focusa-core update::tests --quiet
cargo test -p focusa-cli update --quiet
cargo check --workspace --all-targets --quiet
npm --prefix apps/pi-extension run lint -- --quiet
npm --prefix apps/pi-extension run typecheck
```

The isolated OVH runtime evidence additionally proves:

- Spec 125 runtime/eval suite: pass.
- Context Cognition optimizer hardening: pass.
- Spec 82 memory SLO/stabilization suite: pass.
- Spec 94 live/profile/store-growth suites: pass; 240-sample live run p95 3.64 ms, peak RSS 44,308 KiB, RSS delta 0 KiB.
- Spec 128 installer/update runtime suite: pass.

## Not claimed

- No production deployment.
- No release publication.
- No commercial-license transition.
- No automatic upgrade of unmanaged UIAI/host components.
