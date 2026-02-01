# docs/13-cli.md — Focusa CLI Contract (MVP)

## Purpose
The CLI is the **primary control interface** for engineers.

It must be:
- scriptable
- deterministic
- JSON-friendly
- fast

---

## Binary
`focusa`

---

## Global Flags
- `--json` → machine-readable output
- `--config <path>`
- `--verbose`
- `--quiet`

---

## Commands

### Daemon Control
- `focusa start`
- `focusa stop`
- `focusa status`

---

### Focus Stack
- `focusa stack`
- `focusa focus push "<title>" --goal "<goal>"`
- `focusa focus pop`
- `focusa focus complete`
- `focusa focus set <frame_id>`

---

### Focus Gate
- `focusa gate list`
- `focusa gate suppress <candidate_id> --for 10m`
- `focusa gate resolve <candidate_id>`
- `focusa gate promote <candidate_id>`
  - promotes candidate → push focus frame

---

### Memory
- `focusa memory list`
- `focusa memory set key=value`
- `focusa memory rules`

---

### ECS
- `focusa ecs list`
- `focusa ecs cat <handle_id>`
- `focusa ecs meta <handle_id>`
- `focusa ecs rehydrate <handle_id> --max-tokens 300`

---

### Debug / Inspect
- `focusa events tail`
- `focusa events show <event_id>`
- `focusa state dump`

---

## Output Rules
- Default: human-readable
- `--json`: exact API response passthrough

---

## Error Handling
- Non-zero exit codes on failure
- Errors include:
  - message
  - correlation_id
  - suggested next action (optional)

---

## Acceptance Tests
- CLI works without GUI
- JSON output stable
- Commands idempotent where expected
