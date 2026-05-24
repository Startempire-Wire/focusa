# `focusa_project_identity`

**Family:** `project_identity`  
**Label:** Project Identity

## Purpose

Resolve the active ProjectIdentity before trusting project-bound Workpoints, Trajectory packets, evidence, or carryover context.

## When to use

- At project start/resume when the project folder is unclear.
- Before accepting a Workpoint/Trajectory packet as canonical.
- When a packet, cwd, Beads root, git root, or operator-provided project folder might point at different projects.

## Parameters

- `cwd` — optional cwd/project path hint; defaults to Pi session cwd.
- `project_root` — optional expected project folder/root.

## Expected result

Returns a bounded ProjectIdentity with `status`, `project_id`, `canonical_name`, `project_root`, `fingerprint`, `confidence`, `signals`, `mismatches`, `verified_at`, and quorum details. It also returns `project_summary` and `summary_lines` as the canonical compact project card: project/root/repo, stack, workspace kind, key dirs, root/live/local/wp/app/auth/graphql/api URLs, deployment environment/target/location/command, source confidence, and authority boundary. Marker-backed plus repo/live-root scanned `project_urls` and `deployment` fields expose these facts when present; scans include repo docs, SvelteKit/app files, `wp-config.php`, likely `/home/<site>/public_html` files, deploy scripts, and workflows. It marks unsafe broad roots such as `/root` as `status=unsafe_project_root`, `canonical=false`. Pi results include `details.tool_result_v1` with `status`, `failure_class`, `canonical`, `degraded`, recovery posture, and `next_tools`.

## Failure and recovery

- `failure_class=scope_mismatch`: resolve mismatched marker/git/beads/cwd/operator signals, then retry unchanged only after scope is corrected.
- `canonical=false` or `confidence=low|medium`: treat identity as advisory/degraded and verify before canonical resume.
- `daemon_unavailable`: continue from current repo and Beads as noncanonical fallback, then retry `/v1/project/identity`.

## Example

```text
focusa_project_identity cwd="/home/wirebot/focusa"
```

## Contract summary

- Family: Project Identity.
- Side effects: `read_state`.
- Result envelope: `tool_result_v1` with `failure_class`, canonical/degraded status, retry posture, side effects, evidence refs, and next tools when applicable.
- API routes: `GET /v1/project/identity`
- CLI commands: `focusa project identity`
- Parity: `domain`; exemptions: `domain_cli_only`.
- Core surface: Spec96 ProjectIdentity quorum and project-folder safety.
- Live check: contract_static plus /v1/project/identity safe probe and quorum status.
- Contract source: `docs/current/focusa-tool-contracts.json`.

## Source
Backed by `GET /v1/project/identity`.
