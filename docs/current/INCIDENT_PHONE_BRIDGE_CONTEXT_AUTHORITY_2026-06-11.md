# Incident: Phone Bridge pairing exposed Focusa context-authority failure

**Status:** incident record / workorder source  
**Date range:** 2026-06-11 → 2026-06-12 UTC  
**Project root:** `/home/wirebot/focusa`  
**Continuity:** `focusa-cont-focusa-94854d7e-ef7d-44dc-8870-345372ead878`  
**Recovered runtime state:** CLI `0.9.25-dev`, daemon `0.9.25-dev`, daemon user `wirebot`, bind `127.0.0.1:8787`  
**Primary failure class:** preserved context did not become mutation-time authority.


## Architecture translation

The detailed architecture translation and bead decomposition source lives at:

- `docs/current/CONTEXT_AUTHORITY_ARCHITECTURE_WORKORDER_SPEC_2026-06-12.md`

This companion spec maps each incident symptom to Focusa architecture needs, enforcement points, CLI/API/data-model surfaces, tests, and implementation beads.

Implementation/current guide: `docs/current/CONTEXT_AUTHORITY_CURRENT.md`.

---

## 1. Executive thesis

This incident was not a simple memory failure and not only an agent judgment failure. It was a **Focusa context-authority failure** during a real Phone Bridge Flow test.

Focusa preserved many relevant facts:

- We were actively building Focusa itself in real time.
- The active project root was `/home/wirebot/focusa`.
- The VPS was a live Focusa build/development host, not a clean consumer install.
- The daemon and CLI already existed.
- We were testing pairing/Phone Bridge against an already installed and running Focusa.
- `focusa hlt` CLI/history surfaces already existed.

Those facts did not become a hard operational contract at the moment of risky mutation. The agent substituted a consumer-install action — downloading/installing a GitHub release Linux binary — for the correct live-build-host action: inspect existing state, rebuild locally if needed, restart the daemon, and run `focusa pair`.

Core diagnosis:

> Focusa preserved context, but did not enforce context authority at the action boundary.

Second diagnosis:

> TL/HLT are not operating according to spec: active trajectory projection can surface generic or polluted ladder state instead of falling back to prior verbatim HLT history and marking uncertainty/degradation.

---

## 2. Impact

### User impact

- Operator confidence degraded.
- Pairing initiation was delayed.
- The agent created confusion about whether Focusa needed to be installed on the VPS even though Focusa was already installed/running.
- The agent temporarily broke `/usr/local/bin/focusa` by installing an incompatible GitHub Linux asset.

### System impact

- No evidence of destroyed Focusa data.
- No git reset/clean/destructive repository operation was performed.
- `.beads` was not deleted.
- Focusa daemon was restarted and recovered.
- Final state after recovery was healthy:
  - repo at `v0.9.25-dev`
  - CLI `0.9.25-dev`
  - daemon `0.9.25-dev`
  - daemon running as `wirebot`
  - daemon lock matched running daemon

### Direct technical damage

`/usr/local/bin/focusa` was temporarily overwritten with a GitHub release binary incompatible with this AlmaLinux VPS glibc.

Observed error:

```text
/usr/local/bin/focusa: /lib64/libm.so.6: version `GLIBC_2.29' not found
/usr/local/bin/focusa: /lib64/libc.so.6: version `GLIBC_2.29' not found
/usr/local/bin/focusa: /lib64/libc.so.6: version `GLIBC_2.30' not found
/usr/local/bin/focusa: /lib64/libc.so.6: version `GLIBC_2.32' not found
/usr/local/bin/focusa: /lib64/libc.so.6: version `GLIBC_2.33' not found
/usr/local/bin/focusa: /lib64/libc.so.6: version `GLIBC_2.34' not found
/usr/local/bin/focusa: /lib64/libc.so.6: version `GLIBC_2.39' not found
```

---

## 3. Recovered state

After recovery:

```text
Project root: /home/wirebot/focusa
Git tag: v0.9.25-dev
CLI: focusa 0.9.25-dev
Daemon health: {"ok":true,"version":"0.9.25-dev"}
Daemon PID: 3616130
Daemon user: wirebot
Bind: 127.0.0.1:8787
Lock: focusa-daemon.lock matched daemon PID
```

The final daemon restart used local repo-built binaries, not downloaded release assets.

---

## 4. Timeline

### 4.1 Phone Bridge foundation

Work began with the goal of shipping Focusa MVP Operator Preview and making Phone Bridge Flow reliable.

Completed foundation work:

- Phone Bridge Flow terminology established:
  - **Phone Bridge Flow**
  - **Focusa Connect Page**
  - **Bridge Room**
  - **Mac Handoff Offer**
  - **Mac Completion Payload**
- Bridge Room API implemented.
- Focusa Connect Page implemented.
- Mac FirstRunConnect flow implemented.
- Manual Mac Completion Payload fallback implemented.
- `scripts/create-dev-release-tag.sh --push` patched to wait for CI + Release workflows.
- Release stamper patched to update CLI/API/core/TUI/Mac package surfaces.
- Stale daemon UX improved.
- Robust URL auto-detection added.

Relevant baseline:

```text
v0.9.22-dev
3162f94 chore: stamp menubar 0.9.22-dev
```

### 4.2 Public URL/proxy work reframed into adaptive transport

The operator asked for everything needed before pushing another tag, including missing public URL/proxy infrastructure.

Initial implementation created:

- `scripts/setup-phone-bridge-url.sh`
- docs for public URL/proxy setup
- static tests

Then the operator corrected the model:

> We will be on many different setups, so this process needs to be contained within Focusa and adaptive to many different local and remote setups.

Corrected model:

- Public reverse proxy is only one adapter.
- Focusa must own adaptive transport resolution.
- No live webserver mutation as default.

Implemented:

- `scripts/phone-bridge-transport.sh`
  - `detect`
  - `check --url`
  - `write --url`
  - `options`
  - `proxy-snippets`
- old `setup-phone-bridge-url.sh` became a compatibility shim.
- docs/tests updated.

Relevant commits:

```text
d64115b feat: add phone bridge public url setup
dc0909e feat: add adaptive phone bridge transport resolver
```

### 4.3 Add everything before another tag

The operator said:

> ADD EVERYTHING BEFORE PUSHING ANOTHER TAG. EVERYTHING YOU ARE AWARE OF

Known missing areas were addressed:

- automatic Mac callback / no-paste completion
- adaptive transport resolver integration
- docs/tests

Implemented automatic Mac callback:

- Mac starts Tauri `TcpListener` on `0.0.0.0:0`.
- Mac resolves best local IP.
- QR offer includes `mac_callback`.
- Focusa Connect Page POSTs Mac Completion Payload to callback URL after approval.
- Mac polls callback store and auto-saves token.
- manual paste fallback remains.

Relevant commits/releases:

```text
deefd6d feat: add automatic Mac callback for Phone Bridge Flow
cb89d89 chore: stamp menubar 0.9.23-dev
v0.9.23-dev released with CI + Release green
```

### 4.4 Auto-detect “must just work” correction

The operator said:

> We are supposed to AUTO DETECT! THIS NEEDS TO JUST WORK

Root issue:

- Helper existed, but `focusa pair` itself still required too much manual detect/check behavior.

Implemented:

- `focusa pair` starts/repairs daemon before transport detection.
- detects stale daemon version and restarts it.
- prefers paired CLI/daemon binaries.
- probes configured URLs, non-local API URLs, hostname, public IP, private/Tailscale, local fallback.
- accepts a candidate only when both `/connect` and Bridge Room API respond.
- reports checked candidates in JSON.

Relevant commits/releases:

```text
18ed161 fix: make phone bridge transport auto-detect self-healing
a036b7d chore: stamp menubar 0.9.24-dev
v0.9.24-dev released with CI + Release green
```

### 4.5 Robust logging/error reporting added to both sides

Operator requested:

> ADD ROBUST LOGGING AND ERROR REPORTING AS WELL

Then clarified:

> to both sides

Implemented:

CLI side:

- per-candidate `connect_probe`
- per-candidate `bridge_api_probe`
- `failure_class`
- `message`
- `http_status`
- top-level diagnostics:
  - `checked_count`
  - `rejected_count`
  - `first_rejection`
  - `selected_source`
  - `selected_url`
  - `daemon_repair`
  - `operator_hint`
- human summary of checked/rejected candidates
- daemon version mismatch repair logs

Daemon/API side:

- Bridge Room lifecycle logs:
  - room started
  - Mac offer rejected/accepted
  - approval completed
- API responses include `diagnostics` and `next_step_hint`.
- approval reports `mac_callback_present`, `token_present`, fallback hint.

Relevant commits/releases:

```text
4ec5d87 fix: add phone bridge diagnostics
9b0879f chore: stamp menubar 0.9.25-dev
v0.9.25-dev released with CI + Release green
```

### 4.6 Operator downloaded new app; pairing initiation began

The operator said the new Mac app had been downloaded and the process needed to be initiated.

Correct expected process:

1. confirm local repo/runtime state
2. ensure CLI/daemon are local repo-compatible
3. restart/rebuild daemon only if needed
4. run `focusa pair`
5. phone opens Focusa Connect Page
6. phone scans Mac QR
7. phone approves
8. Mac callback stores token

Observed state:

```text
Repo: /home/wirebot/focusa
Repo tag: v0.9.25-dev
/usr/local/bin/focusa: stale 0.9.22-dev
running daemon health: 0.9.23-dev
Mac app: v0.9.25-dev downloaded by operator
```

Correct interpretation:

```text
This is a live Focusa build host with stale runtime binaries.
Use local repo build/restart path.
```

Incorrect agent interpretation:

```text
Installed binary stale; install GitHub release asset.
```

### 4.7 Incorrect GitHub release asset installation

The agent downloaded GitHub release Linux assets for `v0.9.25-dev` and installed them into `/usr/local/bin`:

- `focusa-v0.9.25-dev-x86_64-unknown-linux-gnu`
- `focusa-daemon-v0.9.25-dev-x86_64-unknown-linux-gnu`

This was wrong for two reasons:

1. The VPS was the live build host, so local repo build was the authority.
2. The GitHub Linux binary required newer glibc than the VPS had.

Result:

- `/usr/local/bin/focusa` temporarily failed to execute.
- No data was destroyed.
- The mistake caused justified operator concern.

### 4.8 Recovery

Correct recovery:

- build from local repo `/home/wirebot/focusa`
- install local build outputs
- restart daemon
- ensure daemon runs as `wirebot`
- ensure lock matches live PID

Recovery path:

```bash
cd /home/wirebot/focusa
cargo build --release --locked -p focusa-api --bin focusa-daemon -p focusa-cli --bin focusa
install -m 755 target/release/focusa /usr/local/bin/focusa
install -m 755 target/release/focusa-daemon /usr/local/bin/focusa-daemon
focusa stop
focusa start
```

Final recovered state:

```text
CLI: 0.9.25-dev
Daemon: 0.9.25-dev
Daemon user: wirebot
Daemon bind: 127.0.0.1:8787
Daemon lock: matched PID
Git tracked files: clean
```

### 4.9 Post-incident analysis with operator

The operator identified that the issue was deeper than one bad command.

Discussed failures:

- environment blindness
- ignoring existing setup
- ignoring prior turn context that we were building Focusa live
- preserved context not operationalized
- missing action-boundary authority
- planning vs implementation discipline failure
- TL/HLT not operating according to spec
- `focusa hlt` CLI/history contract not honored

Important operator corrections:

- The issue was not just a missing install-context flag.
- The issue was ignoring what was already set up.
- We were in the middle of testing pairing for an already installed and running Focusa.
- TL/HLT enforcement may have helped, but TL/HLT are currently wrong often.
- TL/HLT are not operating according to spec.
- HLT should be based on prior verbatim HLT history when current HLT is unsure.
- There is already an HLT CLI; do not invent a new requirement.

### 4.10 HLT/TL investigation

Relevant spec doctrine from `docs/current/TRAJECTORY_GTM_AND_GAPS.md`:

```text
Official trajectory ladder: HLT → MLG → STG → Waypoints → Workpoint.
Workpoints remain the canonical immediate continuation contract.
Trajectory is advisory orientation, not task authority.
Operator steering wins.
Evidence is the completion currency.
Low-memory reliability beats rich-context ambition.
```

Existing CLI:

```bash
focusa hlt ls
focusa hlt history
focusa hlt set
focusa hlt verify
focusa hlt mlg
focusa hlt stg
focusa hlt waypoint
```

Observed broken output:

```text
HLT: Maintain and improve Focusa within verified project scope
MLG/STG: polluted by aborted install-context implementation Workpoint text
clarity: proceed / no blocking reasons
```

Implementation issue observed in `trajectory_view_payload`:

- when no persisted HLT/desired end state exists, it bootstraps:

```text
Maintain and improve {project_label} within verified project scope
```

- this generic HLT can appear like a real active HLT.
- prior verbatim HLT history was not used as the fallback authority.
- MLG/STG can derive from focus/current Workpoint text instead of valid HLT.

Correct behavior expected by operator:

- TL is derived from HLT.
- HLT should have historical verbatim records.
- When current HLT is unsure, use previous verbatim HLT record.
- `focusa hlt history` is the existing surface for this.
- generic bootstrap should be degraded placeholder, not active ladder authority.

---

## 5. Root causes

### 5.1 Environment/provenance blindness

The agent did not treat the current environment as authority.

Existing facts ignored:

- live repo existed
- daemon existed
- CLI existed
- project root known
- recent work was on Focusa itself
- current task was pairing test, not installation

### 5.2 Wrong inference from version mismatch

Version mismatch was interpreted as:

```text
Install release asset.
```

It should have been interpreted as:

```text
Live build host stale runtime; rebuild/restart locally.
```

### 5.3 Missing mutation preflight

Binary replacement happened without a formal preflight.

Required preflight should have included:

```text
Current ask: initiate pairing
Environment: live Focusa build host
Repo: v0.9.25-dev
CLI: stale
Daemon: stale/running
Proposed mutation: replace /usr/local/bin/focusa from GitHub release asset
Verdict: blocked
Safe action: local rebuild/restart from /home/wirebot/focusa
```

### 5.4 Context preserved but not enforced

Focusa preserved context, but it did not become action authority.

Central failure:

```text
Memory existed.
Authority did not.
```

### 5.5 TL/HLT spec failure

TL/HLT projection produced or accepted generic/polluted ladder data:

- generic HLT
- polluted MLG/STG
- clarity “proceed” despite wrong ladder
- no fallback to prior verbatim HLT

### 5.6 HLT CLI/history contract not honored

The HLT CLI exists. The issue is not missing surface area.

The issue is:

- active HLT/TL projection does not reliably use `focusa hlt history`
- prior verbatim HLT record is not restored when active HLT is unsure
- `hlt verify` did not catch generic/polluted ladder state

### 5.7 Planning/implementation boundary failure

When the operator used exploratory language such as:

```text
Maybe we can add some flag...
```

The agent nearly jumped into implementation. That is the same class of failure as the binary install:

- insufficient intent classification
- poor discipline around planning vs mutation

### 5.8 Daemon/runtime hygiene gaps

The recovery exposed additional hygiene needs:

- daemon should run as project owner
- lock should match live PID
- CLI and daemon versions should match
- daemon repair should preserve provenance
- glibc compatibility should be checked before any binary swap

---

## 6. Spec compliance failures

### 6.1 Focusa context preservation vs action authority

Expected:

```text
Preserved context constrains action.
```

Observed:

```text
Preserved context was advisory and ignored during mutation.
```

### 6.2 Trajectory Ladder

Expected:

```text
HLT → MLG → STG → Waypoints → Workpoint
```

Observed:

```text
Workpoint/current-focus text polluted MLG/STG.
Generic HLT substituted for verbatim HLT.
```

### 6.3 HLT history fallback

Expected:

```text
When current HLT is unsure, use prior verbatim HLT record.
```

Observed:

```text
Generic bootstrap HLT was used instead.
```

### 6.4 `focusa hlt verify`

Expected:

```text
Detect generic HLT, polluted MLG/STG, missing verbatim evidence/history.
```

Observed:

```text
No effective block; clarity allowed proceed.
```

### 6.5 Operator steering

Expected:

```text
Operator current ask and recent context outrank inferred projection.
```

Observed:

```text
Agent substituted an install task during pairing initiation.
```

---

## 7. Workorder candidates

### Workorder A — Operational Context Gate

Add a mutation preflight gate for risky actions.

Risky actions include:

- overwriting `/usr/local/bin/focusa`
- overwriting `/usr/local/bin/focusa-daemon`
- downloading release assets
- killing/restarting daemon
- removing or moving daemon lock
- mutating pairing state

Gate inputs:

- current ask
- Workpoint
- project root
- repo tag/head
- daemon pid/user/version/bind
- CLI path/version/provenance
- HLT current/history/verify
- environment role
- proposed action

Gate output:

```json
{
  "verdict": "allow|block|needs_operator_confirmation",
  "reason": "...",
  "safe_alternative": "..."
}
```

Acceptance test:

- On live build host, proposed GitHub release binary install is blocked.
- Safe alternative is local rebuild/restart.

### Workorder B — Focusa Environment Contract

Create machine-readable install/environment contract.

Possible fields:

```json
{
  "schema": "focusa.environment_contract.v1",
  "install_role": "live_build_host",
  "project_root": "/home/wirebot/focusa",
  "owner": "wirebot",
  "machine_kind": "vps",
  "binary_policy": {
    "preferred_source": "local_repo_build",
    "release_asset_install_allowed": false
  },
  "pairing_state": "never_paired|paired|unknown",
  "host": {
    "os": "AlmaLinux",
    "arch": "x86_64",
    "glibc": "detected"
  }
}
```

Important: this complements HLT; it does not replace HLT.

### Workorder C — Binary provenance and compatibility

Add provenance to CLI/daemon:

```bash
focusa --version --json
focusa-daemon --version --json
```

Expected fields:

- version
- git sha
- build profile
- build host
- source type:
  - local_repo_build
  - release_asset
  - package_manager
- glibc/build target
- project root if available

Before binary replacement:

- verify glibc compatibility
- verify release asset allowed by environment contract
- back up existing binary
- print preflight

### Workorder D — HLT history fallback compliance

Fix TL/HLT so prior verbatim HLT history is used when current HLT is unsure.

Acceptance criteria:

- `focusa hlt history` returns prior verbatim HLT records.
- `focusa hlt ls` uses latest valid verbatim HLT when active projection is generic/unclear.
- generic bootstrap HLT is marked degraded placeholder.
- generic bootstrap HLT cannot produce `clarity_gate.recommended_action = proceed`.
- `hlt verify` fails on generic/polluted active ladder.

### Workorder E — TL derivation discipline

Ensure:

```text
HLT → MLG → STG → Waypoints → Workpoint
```

Rules:

- MLG derives from valid HLT.
- STG derives from HLT/MLG.
- Waypoints derive from STG/MLG.
- Workpoint remains canonical immediate continuation.
- Workpoint/current_focus can inform STG only when compatible with valid HLT.
- Workpoint text cannot become HLT/MLG by projection accident.

### Workorder F — Task substitution detector

Detect when proposed action belongs to a different workflow than current ask.

Scenario:

```text
Current ask: initiate pairing
Proposed action: install release asset
Verdict: block / explain task substitution
```

### Workorder G — Planning vs implementation mode gate

Classify operator prompt mode:

- discussion/planning
- diagnosis
- implementation
- runtime operation
- destructive/high-risk operation

Exploratory prompts must not trigger code.

Acceptance test:

```text
Operator: “Maybe we can add a flag...”
Expected: produce plan/spec only, no code changes.
```

### Workorder H — Daemon runtime hygiene

Enforce:

- one daemon per bind
- daemon user matches project owner unless explicitly configured
- lock PID matches process
- stale lock is reported and repaired safely
- CLI/daemon version mismatch reports actionable diagnostics
- repair path respects binary provenance/environment contract

### Workorder I — Incident replay golden tests

Create a golden scenario test:

```text
Given:
  project_root=/home/wirebot/focusa
  install_role=live_build_host
  repo=v0.9.25-dev
  daemon=0.9.23-dev running
  cli=0.9.22-dev installed
  current_ask=initiate Phone Bridge pairing

When:
  agent proposes installing GitHub release asset

Then:
  block action
  reason=consumer_install_path_conflicts_with_live_build_host
  safe_alternative=local_repo_build_and_daemon_restart
```

---

## 8. Proposed priority order

1. **HLT/TL spec audit + failing tests**
2. **Operational Context Gate design**
3. **Incident replay golden scenario**
4. **Environment Contract**
5. **Binary provenance / glibc compatibility**
6. **Daemon hygiene**
7. **Planning vs implementation mode gate**

Rationale:

- HLT/TL is central to Focusa promise.
- Context gate prevents recurrence even before TL is perfect.
- Incident replay locks this exact failure class.
- Environment contract provides factual substrate.

---

## 9. Definition of fixed

This incident is fixed when all of the following are true:

1. A fresh agent on this VPS can determine it is on a live Focusa build host before mutation.
2. `focusa hlt verify` flags generic/polluted HLT/TL state.
3. `focusa hlt ls` uses prior verbatim HLT history when active HLT is unsure.
4. MLG/STG cannot be silently populated from unrelated Workpoint text.
5. Proposed GitHub release asset install is blocked on this host.
6. Safe local rebuild/restart path is recommended.
7. Pairing initiation proceeds without agent guessing install strategy.
8. Planning prompts do not trigger implementation.

---

## 10. Key operator requirements captured

- “You should not need to install anything on the VPS right?”
- “We are building Focusa live and should have it already.”
- “You also disregarded the previous turn of work where the context of everything we have been doing is working on Focusa in real time.”
- “How did you lose context with all the Focusa tools and the core premise of Focusa is keeping context?”
- “TL & HLT are not operating according to spec.”
- “The HLT should be accessible and findable like a file in Linux with the ls command.”
- “That is not a new requirement there is a CLI.”
- “TL is derived from HLT but there should be a historical record of verbatim HLTs; this is also what should auto populate the session when HLT is unsure.”

---

## 11. One-line workorder summary

Build a Focusa context-authority system that combines verified environment facts, current ask, Workpoint, and verbatim HLT history to block off-context mutations and prevent generic/polluted TL/HLT projections from guiding agents.
