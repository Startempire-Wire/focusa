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
