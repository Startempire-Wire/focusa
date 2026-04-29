# Spec89 Real Release Live Proof — 2026-04-28

Operator requirement: release must be installed and proven against the real daemon/CLI/Pi tool surface, not only shell-script stress tests.

## Released artifact

- Commit at release proof start: `860d961`
- Final released commit after live fixes: `dd39dad`
- Installed daemon path: `/home/wirebot/focusa/target/release/focusa-daemon`
- Installed CLI path: `/home/wirebot/focusa/target/release/focusa`
- Running service executable: `/home/wirebot/focusa/target/release/focusa-daemon`
- Final daemon mtime after rebuild from `dd39dad`: `2026-04-28 17:23:31 -0700`
- Final daemon size: `23220160`
- Owner: `wirebot:wirebot`
- Health: `{"ok":true,"version":"0.1.0"}`

## Real bugs found and repaired during live proof

1. `/v1/workpoint/checkpoint` accepted a canonical promoted checkpoint before `/current` and `/resume` could rely on it.
   - Repair: checkpoint now waits for reducer-visible active Workpoint before returning `accepted`; otherwise returns `pending` guidance.
2. `/v1/workpoint/evidence/link` accepted evidence before `/resume` showed it in `verification_records`.
   - Repair: evidence link now waits for reducer-visible verification record before returning `accepted`; otherwise returns `pending` guidance.

## Direct live proof after repairs

Final live Workpoint:

- `workpoint_id=019dd69d-2e7e-74a0-a722-a6ed804d040f`
- mission: `Spec89 final release proof`
- action: `release_verify`
- next slice: `runtime fully functional`
- canonical: `true`

Direct API behavior proven:

- `GET /v1/health` returned ok.
- `POST /v1/workpoint/checkpoint` returned `status=accepted`, `canonical=true`, and embedded `workpoint`.
- `GET /v1/workpoint/current` immediately returned the newly accepted Workpoint ID.
- `POST /v1/workpoint/resume` immediately returned the same Workpoint ID.
- `POST /v1/workpoint/evidence/link` returned `status=accepted` and linked count `1`.
- Follow-up `POST /v1/workpoint/resume` showed `verification_records` containing `live-api:final-visible-evidence`.
- `POST /v1/focus/update` returned `status=accepted`.
- `POST /v1/metacognition/capture` stored capture `cap-1777422135944404514`.
- `GET /v1/work-loop/status` returned live idle state with `active_writer=daemon-supervisor`.

Direct CLI behavior proven against installed release binary:

- `target/release/focusa workpoint current` returned Workpoint `019dd69d-2e7e-74a0-a722-a6ed804d040f`.
- `target/release/focusa workpoint drift-check --latest-action 'release verify Spec89FocusaToolSuite live_api cli pi_tool' --expected-action-type release_verify` returned `status=no_drift`.

Direct Pi tool behavior proven in the active Pi session:

- `focusa_workpoint_resume` returned the same canonical Workpoint ID and `next=runtime fully functional`.
- `focusa_metacog_doctor` found release-validation captures and top capture `cap-1777422135944404514`.

Final proof marker:

```text
DIRECT_REAL_RELEASE_PROOF=PASS workpoint=019dd69d-2e7e-74a0-a722-a6ed804d040f
```
