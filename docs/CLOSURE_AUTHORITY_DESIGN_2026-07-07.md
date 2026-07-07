# Closure Authority — Programmatic Design (2026-07-07)

**Status:** design, brainstorming.
**Goal:** Implement Spec 116 end-to-end with **real** evidence verification
(not stubs), and a provider-neutral adapter pattern that does not make
beads (bd) the sole authority. Asana, Linear, GitHub Issues, GitLab,
Jira, and future providers plug in via a single trait.

## The bug the spec fixes

Today `bd close focusa-123` runs unconditionally. Late detection in CI or
pre-push is useful but insufficient — closure prevention must happen at
close time. Spec 116 says: **Focusa validates closure truth; providers
store and display the closure.** bd is adapter #1, not the source of
truth.

## Architecture

### 1. Provider-neutral adapter pattern

```
crates/focusa-core/src/work_item/
  mod.rs                     # Provider trait, Registry, types
  provider.rs                # WorkItemProvider enum + detection
  claim.rs                   # ClosureClaim + EvidenceCitation
  evidence.rs                # EvidenceVerifier + 7 verifiers
  lifecycle.rs               # Prepare -> Validate -> Authorize -> Submit -> Reconcile
  policy.rs                  # closure.toml + profile loading
  audit.rs                   # closure-audit.jsonl appender
  guard_shim.rs              # PATH-injected command interceptor
  adapters/
    mod.rs
    bd.rs                    # bd adapter
    linear.rs                # Linear adapter
    asana.rs                 # Asana adapter
    github.rs                # GitHub Issues adapter
    gitlab.rs                # GitLab adapter
    jira.rs                  # Jira adapter
    none.rs                  # no-op adapter for repos without a provider
```

Each adapter implements the same `ProviderAdapter` trait:

```rust
#[async_trait]
pub trait ProviderAdapter: Send + Sync {
    fn provider(&self) -> WorkItemProvider;
    fn capabilities(&self) -> ProviderCapabilities;
    async fn detect(&self) -> DetectionResult;
    async fn resolve(&self, ref_id: &str) -> Result<WorkItem>;
    async fn validate_ref(&self, ref_id: &str) -> Result<RefStatus>;
    async fn prepare_claim(&self, claim: &ClosureClaim) -> Result<PreparedClaim>;
    async fn submit_claim(&self, prepared: &PreparedClaim) -> Result<SubmitResult>;
    async fn reconcile(&self, submit: &SubmitResult) -> Result<ReconcileResult>;
}
```

A new provider = one new file in `adapters/`. The CLI, REST API, doctor,
and audit pipeline do not change.

### 2. ClosureClaim model (matches spec §7.4)

```rust
pub struct ClosureClaim {
    pub schema: String,                  // "focusa.closure_claim.v1"
    pub claim_id: String,                // ULID
    pub idempotency_key: String,

    pub work_item: WorkItemRef,          // {provider, provider_item_id, project_root, external_url?}
    pub project_root: PathBuf,
    pub continuity_id: String,
    pub workpoint_id: Option<String>,

    pub actor_id: String,                // operator email or agent name
    pub agent_session_id: Option<String>,

    pub closure_summary: String,
    pub closure_kind: ClosureKind,       // code | docs | deploy | investigation | no_code | admin

    pub code_refs:  Vec<EvidenceCitation>,
    pub spec_refs:  Vec<EvidenceCitation>,
    pub proof_refs: Vec<EvidenceCitation>,  // tests + endpoints
    pub deploy_refs: Vec<EvidenceCitation>,
    pub artifact_refs: Vec<EvidenceCitation>,

    pub policy: ClosurePolicy,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: ClaimStatus,             // draft | valid | authorized | submitted | reconciled | blocked | expired
}
```

### 3. **Real** evidence verification (not stubs)

The previous `bd-evidence` push hook was bypassed by appending generic
templates. This design makes every citation actually verifiable.

```rust
pub trait EvidenceVerifier: Send + Sync {
    fn kind(&self) -> EvidenceKind;     // code | spec | test | endpoint | artifact | workpoint | ci | deploy
    async fn verify(&self, citation: &EvidenceCitation) -> VerifyResult;
}

pub struct VerifyResult {
    pub verified: bool,
    pub result: String,                // "sha256:abc123…", "200 OK", "test passed", "permission denied"
    pub evidence_url: Option<String>,  // github permalink, daemon URL, file://, etc.
}
```

Seven concrete verifiers, one per kind:

| Kind | Verifier | Real check |
|---|---|---|
| `code` | `CodeVerifier` | File exists, non-empty, git-blame + line range resolves. Result includes the SHA256 of the cited line range. |
| `spec` | `SpecVerifier` | File exists, contains a non-trivial section heading matching the citation's `ref` topic, last-modified within the claim's `expires_at`. |
| `test` | `TestVerifier` | File exists. If `run: true` (default), shell out to `bash <test>` or `python3 <test>` and capture exit code + tail. |
| `endpoint` | `EndpointVerifier` | HTTP probe to `ref`. Result: status code, content-type, first 200 bytes. Optional body SHA256. |
| `artifact` | `ArtifactVerifier` | File exists. SHA256 must match `expected_sha256` if provided. |
| `workpoint` | `WorkpointVerifier` | Daemon `GET /v1/workpoint/resolve?id=<workpoint_id>`. Cross-references the Workpoint's own `evidence_refs`. |
| `ci` | `CiVerifier` | `gh run view <run_id> --json status,conclusion,headSha` (or GitLab/Jira equivalent). Result: pass/fail + head commit + duration. |
| `deploy` | `DeployVerifier` | `curl /v1/health` + `focusa doctor` on the deployed host. Result: `version, ok, uptime_ms`. |

Every citation has a `result` field that records what was actually
observed. The claim is `valid` only when every required citation
verifies. The closure-audit.jsonl row for the claim records every
verifier call so the operator can replay.

### 4. Closure lifecycle (matches spec §9)

```
prepare    -> walk the Workpoint, the project state, and the spec; build the
             ClosureClaim with auto-discovered citations
validate   -> run every verifier; produce per-citation VerifyResult; flip
             claim.status to "valid" only if all required citations pass
authorize  -> check ClosurePolicy, actor, agent_session, git state; require
             FOCUSA_OPERATOR=1 (or override) for non-allow-listed actors
submit     -> call provider adapter submit_claim(); provider mutates the
             task manager only after authorization
reconcile  -> re-fetch the work item from the provider, verify status changed
             to the expected end state, write audit row, link Workpoint
```

Each stage returns a typed envelope. Failures use
`focusa.closure_block.v1` with a `recovery_hint` that names the next
concrete action.

### 5. CLI surface (matches spec §12 + new commands)

```bash
focusa work-item close <id> --from-workpoint <WP_ID>           # full lifecycle
focusa work-item close <id> --from-workpoint <WP_ID> --profile release_proof
focusa work-item close <id> --override --reason "..."          # break-glass
focusa work-item closure prepare <id>                            # stage 1 only
focusa work-item closure validate <claim_id>                      # stage 2 only
focusa work-item closure authorize <claim_id>                     # stage 3 only
focusa work-item closure submit <claim_id>                        # stage 4 only
focusa work-item closure reconcile <claim_id>                     # stage 5 only
focusa work-item provider-guard evaluate --provider bd --command "bd close <id>"
focusa work-item providers list
focusa work-item providers add linear --api-key <KEY> [--team=<TEAM>]
focusa work-item providers add asana  --api-key <KEY> [--workspace=<GID>]
focusa work-item providers add github --repo OWNER/REPO [--token=$GH_TOKEN]
focusa doctor closure
focusa install closure-guard --auto
```

`focusa install closure-guard --auto` (matches spec §13):
1. detect provider (probe for bd, linear-cli, asana-cli, gh, glab, jira);
2. install the matching adapter (no-op for built-in ones; downloads asana-cli
   if missing and the user opted in);
3. install the guard shim — replace `bd` (or whichever provider) in
   `~/.local/bin` with a focusa wrapper that intercepts close-shaped
   commands and delegates to `focusa work-item closure submit`;
4. wire the Pi reminder/guard integration so the agent gets
   "use focusa work-item close" before it tries `bd close`;
5. write `~/.focusa/policy/closure.toml`;
6. verify `which bd` now resolves to the focusa shim;
7. run `focusa doctor closure`;
8. report the exact state in one human-readable envelope.

### 6. API surface (matches spec §11)

```
GET  /v1/work-items/providers
GET  /v1/work-items/closure/policy
GET  /v1/work-items/closure/claim/<claim_id>
POST /v1/work-items/closure/prepare
POST /v1/work-items/closure/validate
POST /v1/work-items/closure/authorize
POST /v1/work-items/closure/submit
POST /v1/work-items/closure/reconcile
POST /v1/work-items/provider-guard/evaluate
GET  /v1/doctor/closure
POST /v1/work-items/providers        # body: {provider, kind: "linear", config: {...}}
```

All blocked/failure responses use `focusa.closure_block.v1` with
`code`, `why`, `recovery_hint`, `next_tools`.

### 7. Provider guard shim (matches spec §10)

Installed by `focusa install closure-guard --auto`:

```
PATH=$HOME/.local/bin:$PATH
which bd  -> /home/<user>/.local/bin/bd   (the shim)
shim bd close <id>        -> focusa work-item closure submit --from "<bd close <id>>"
shim bd update --status closed <id>  -> same
shim bd update --status done <id>    -> same
allowed: bd show, bd list, bd ready, bd update --status in_progress
```

Equivalent shims for:
- `linear` -> `linear-cli issue close <id>` intercepted
- `asana` -> `asana-cli task complete <id>` intercepted
- `gh issue close <id>` intercepted
- `glab issue close <id>` intercepted
- `jira` -> `jira-cli issue close <id>` intercepted

Every shim is one shell script (≈40 lines) that calls back into
`focusa work-item closure submit`. The shim is auditable — every
intercept writes a `provider_guard_intercept` row to
closure-audit.jsonl.

### 8. Pre-built evidence profiles (matches spec §8)

```toml
# ~/.focusa/policy/closure-profiles/release_proof.toml
[profile]
name = "release_proof"
min_required = { code = 1, test = 1, endpoint = 2 }
required_kinds = ["code", "test", "endpoint"]

[profile.rules]
endpoint.status_in = [200, 201, 202, 204]
test.exit_code = 0
code.min_lines_changed = 1

# Operators can add profiles:
#   code_only, code_with_test, code_with_endpoint, release_proof,
#   pre_mvp_polish, doc_change, deploy_only
```

### 9. Break-glass override (matches spec §14)

```bash
focusa work-item close <id> --override --reason "..." --actor-token $OPS_TOKEN
```

Always writes `closure_override` row to closure-audit.jsonl. Disabled by
default for agents (only `FOCUSA_OPERATOR=1` or a real ops token can
issue the override). The provider guard shim checks the same gate.

### 10. Storage layout

```
~/.focusa/
  policy/
    closure.toml                       # ClosurePolicy (active profile, override policy, agent block list)
    providers/
      bd.toml
      linear.toml                      # API key, team, OAuth refresh token
      asana.toml
      github.toml
    closure-profiles/
      release_proof.toml
      pre_mvp_polish.toml
      code_only.toml
      ...
  state/
    closure-claims/<claim_id>.json     # durable ClosureClaim records
    closure-audit.jsonl                # append-only audit (every stage, every override)
    workpoints/<wp_id>/evidence.json   # existing Workpoint evidence (used by prepare)
```

### 11. Doctor surface

`focusa doctor closure` returns a single envelope:

```json
{
  "status": "ok",
  "summary": "closure prevention active",
  "details": {
    "provider": "bd",
    "adapter": "installed",
    "guard_shim": "active",
    "which_bd": "/home/verious/.local/bin/bd",
    "real_bd": "/usr/local/bin/bd",
    "policy_file": "/home/verious/.focusa/policy/closure.toml",
    "active_profile": "release_proof",
    "override_enabled_for_agents": false,
    "correct_close_path": "focusa work-item close <id> --from-workpoint <wp_id>"
  }
}
```

### 12. Why this is "programmatic and complete"

- **Programmatic:** every stage is a typed API + CLI surface. The agent
  workflow is one `focusa work-item close ...` call. The CI workflow
  is `focusa work-item provider-guard evaluate`. The operator workflow
  is `focusa doctor closure`. No grep / no ad-hoc scripts.
- **Complete:** every citation is verified by a real evidence
  verifier (file, hash, HTTP, test execution, CI run, deploy health).
  The previous "evidence citations: ..." stub strings are replaced
  with concrete `result` and `evidence_url` per citation.
- **Provider-neutral:** bd is adapter #1. Asana, Linear, GitHub, GitLab,
  Jira, and future providers plug in via a single trait. The `bd-evidence`
  push hook becomes one of many sources of evidence, not the sole
  authority. Closure truth lives in `closure-claims/<id>.json` and the
  provider state; the push hook is just a CI guard that says "the
  closure claim on disk has status=reconciled before you can push."
- **Future integrations:** the same trait serves Asana today via
  `adapters/asana.rs` (uses the Asana REST API with the Personal
  Access Token in `~/.focusa/policy/providers/asana.toml`). Adding
  Linear or Jira is a new adapter file + provider registration; no
  CLI / API / doctor / shim changes.

## Implementation phasing (proposed)

**Phase A (scaffold) — 1 PR:**
- trait + types + registry
- bd adapter (uses existing bd CLI as the executor)
- Evidence verifiers (all 7 kinds, real checks)
- closure-claim storage
- closure-audit.jsonl appender
- `focusa work-item close` end-to-end (one command runs all 5 stages)
- `focusa doctor closure`

**Phase B (provider surface) — 1 PR:**
- linear adapter
- asana adapter
- github adapter
- provider registry + `focusa work-item providers list/add/remove`

**Phase C (guard shim) — 1 PR:**
- PATH-injected shim for each provider
- `focusa install closure-guard --auto` (per spec §13)
- installer integration with `focusa install` itself

**Phase D (policy + profiles) — 1 PR:**
- ClosurePolicy + TOML loading
- pre-built profiles (release_proof, pre_mvp_polish, code_only, code_with_test, code_with_endpoint, doc_change, deploy_only)
- profile auto-selection per closure_kind

**Phase E (UI + observability) — 1 PR:**
- Pi reminder hook (agent nudges to use the new command)
- TUI card showing claim state
- audit query surface

## Why spec 116 + this design wins

- **Spec is the source of truth.** We don't reinvent the model — we
  implement the typed model from §7 and the lifecycle from §9.
- **bd-evidence hook becomes a 1-line CI guard.** It calls
  `focusa work-item provider-guard evaluate` instead of grepping
  strings out of `.beads/issues.jsonl`. The hook's job shrinks from
  "audit 485 close_reasons for format" to "verify the closure claim
  on the relevant bead has status=reconciled."
- **Asana + Linear + GitHub plug in without breaking bd.** The first
  install uses bd; the second install at a Linear shop uses Linear; the
  third at an Asana shop uses Asana. Same `focusa work-item close`
  command, same ClosureClaim, same audit.
- **Real evidence.** Every cited test, spec, endpoint, or artifact is
  actually run / fetched / hashed at close time. The audit log records
  the exact result so a reviewer can replay.

## What I want to do next

Phase A. Want me to start? It will land in one PR with a `focusa work-item close`
end-to-end, real evidence verifiers, the bd adapter, and a doctor card.

If you want to reshape the brainstorm (e.g. different storage, different
adapter boundaries, different policy DSL), say so now and I will
adjust before coding.