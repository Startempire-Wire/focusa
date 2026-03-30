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

### Export (Datasets)
- `focusa export <dataset_type> --output <path> [--format jsonl|parquet ...]`
  - See: `docs/21-data-export-cli.md`

### Multi-device Sync (Local-first)
- `focusa sync peers list`
- `focusa sync peers add <url>`
- `focusa sync push [--peer <id>]`
- `focusa sync pull [--peer <id>]`
- `focusa sync now [--peer <id>]`
  - Policy: bidirectional sync; imports as observations; per-thread ownership.
  - See: `docs/43-multi-device-sync.md`

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

### Reflection Loop (Overlay)
- `focusa reflect run [--window 1h] [--budget 800] [--idempotency-key k]`
- `focusa reflect history [--limit 20] [--mode manual|scheduled] [--stop-reason low_confidence] [--since RFC3339] [--until RFC3339] [--cursor-before RFC3339]`
- `focusa reflect status` (includes scheduler telemetry counters + stop_reason summary)
- `focusa reflect scheduler status` (includes window counter telemetry)
- `focusa reflect scheduler enable|disable`
- `focusa reflect scheduler set [--interval-seconds N] [--max-iterations-per-window N] [--cooldown-seconds N] [--low-confidence-threshold 0.9] [--no-delta-min-event-delta N]`
- `focusa reflect scheduler tick [--window 1h] [--budget 800]`

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
