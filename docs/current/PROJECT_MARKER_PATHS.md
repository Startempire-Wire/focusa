# Project Marker Paths — One Preferred Way (#243)

**Status:** canonical guidance. Every marker producer routes through
`focusa-core/src/project_marker.rs`.

## One preferred path

```bash
focusa init --project-root <path> --quickstart   # preferred: create or enrich
```

Agents and scripts must never hand-write `.focusa-project.json`.

## Path responsibilities

| Command | Responsibility | Marker behavior |
| --- | --- | --- |
| `focusa init --project-root <path>` | New or existing local project: create/enrich the marker | created / already-valid / migrated (identity preserved) |
| `focusa onboard --remote <git-url>` | Record a git URL for a locally accessible project root | created with repo_remote; refuses conflicting remotes |
| `focusa project verify` | Verify identity — never writes | read-only |
| Genesis lifecycle (Spec 135B) | Runtime ProjectGenesisRecord — separate from the marker file | does not produce the marker; points at `focusa init` when missing |
| Repair | Corrupted marker | `repair_marker()` restores from the pre-migration backup with identity verification (internal; CLI surface in IR2) |

## Guarantees (from the core service)

- Atomic writes (temp + fsync + rename); interrupted writes never leave a
  partial marker.
- Idempotent outcomes; conflicting identity is refused, never overwritten.
- Legacy/minimal v1 markers are detected and enriched without changing
  `project_id`/`project_root`.
- Directory-ownership preservation: markers for cPanel/other-user
  directories return `blocked_permission` instead of writing root-owned
  files.
- Preview mode before any mutation; pre-migration backup on change.
