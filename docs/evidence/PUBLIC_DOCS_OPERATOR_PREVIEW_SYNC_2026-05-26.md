# Public Docs Operator Preview Sync — 2026-05-26

## Scope

Public-facing Focusa docs were synced after Operator Preview surfaces landed:

- `focusa onboard`
- `focusa status --operator`
- `focusa workpoint resume --copy-prompt`
- `scripts/demo-workpoint-happy-path.sh`

## Updated public docs

- `README.md`
- `docs/README.md`
- `docs/current/CLI_REFERENCE_CURRENT.md`
- `docs/current/CURRENT_RUNTIME_STATUS.md`
- `docs/current/DOCTOR_CONTINUE_RELEASE_PROVE.md`
- `docs/current/VALIDATION_AND_RELEASE_PROOF.md`
- `docs/current/FOCUSA_OPERATOR_PREVIEW_PROOF.md`
- `docs/current/AGENT_COMMAND_COOKBOOK.md`
- `docs/current/NON_PI_AGENT_FOCUSA_USAGE.md`
- `docs/current/WORKPOINT_LIFECYCLE_GUIDE.md`

## Accuracy posture

Operator Preview docs now frame Workpoint-first continuity as the supported first buyer workflow. Design-forward ontology/governance surfaces remain documented as advanced/current-development areas unless a current doc marks a command implemented.

## Verification commands

```bash
rg -n "status --operator|copy-prompt|demo-workpoint|focusa onboard" README.md docs/current docs/README.md templates
bash -n scripts/demo-workpoint-happy-path.sh
focusa status --operator
focusa workpoint resume --copy-prompt
```
