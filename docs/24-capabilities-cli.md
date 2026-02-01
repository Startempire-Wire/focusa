# docs/24-capabilities-cli.md — Focusa Capabilities CLI Specification (AUTHORITATIVE)

This document defines the **Focusa CLI** as a first-class, exhaustive client of the
Focusa Capabilities API.

The CLI is not a “dev helper.”  
It is a **cognitive observability and command surface** for humans and agents.

---

## 0. Canonical Principles

1. **CLI parity with API**: anything observable via API must be observable via CLI.
2. **No hidden mutations**: all writes are explicit commands.
3. **Human + agent usable**: machine-readable output is mandatory.
4. **Local-first**: CLI always targets the local Focusa daemon.
5. **Calm power**: no surprise actions, no implicit escalation.

---

## 1. CLI Structure

```bash
focusa <domain> <action> [subaction] [flags]
```

Top-level domains map **1:1** with Capability Domains:

```
state
lineage
references
gate
intuition
constitution
autonomy
metrics
cache
contribute
export
agents
events
commands
```

---

## 2. Output Modes (Mandatory)

All commands support:

```bash
--format table|json|jsonl|yaml
--quiet
--explain
```

Defaults:
- human-facing → `table`
- scripting → `json`

---

## 3. Core CLI Domains

### 3.1 State

```bash
focusa state show
focusa state history
focusa state diff --from 41 --to 42
focusa state stack
```

Flags:
- `--agent <id>`
- `--session <id>`
- `--limit`
- `--cursor`

---

### 3.2 Lineage (CLT)

```bash
focusa lineage head
focusa lineage tree
focusa lineage node <clt_id>
focusa lineage path <clt_id>
focusa lineage children <clt_id>
focusa lineage summaries
```

Special:
- `tree` prints ASCII tree by default
- `--depth` controls traversal

---

### 3.3 References

```bash
focusa references list
focusa references show <ref_id>
focusa references meta <ref_id>
focusa references search "<query>"
```

Large artifacts:
- auto-paged
- `--range offset:length`

---

### 3.4 Gate

```bash
focusa gate policy
focusa gate scores
focusa gate explain <candidate_id>
```

Read-only in MVP.

---

### 3.5 Intuition

```bash
focusa intuition signals
focusa intuition patterns
```

Optional advisory submission (restricted):
```bash
focusa intuition submit --file signal.json
```

---

### 3.6 Constitution

```bash
focusa constitution show
focusa constitution versions
focusa constitution diff 1.1.0 1.2.0
focusa constitution drafts
```

Commands (write surface):
```bash
focusa constitution propose --from-current
focusa constitution activate <version>
focusa constitution rollback <version>
```

All write actions require confirmation unless `--yes`.

---

### 3.7 Autonomy

```bash
focusa autonomy status
focusa autonomy ledger
focusa autonomy explain <event_id>
```

---

### 3.8 Metrics

```bash
focusa metrics uxp
focusa metrics ufi
focusa metrics session <session_id>
focusa metrics perf
```

Supports:
- `--window 7d|30d`
- `--trend`

---

### 3.9 Cache

```bash
focusa cache status
focusa cache policy
focusa cache events
```

Intentional bust (command):
```bash
focusa cache bust --reason "<text>"
```

---

### 3.10 Contribution

```bash
focusa contribute status
focusa contribute enable
focusa contribute pause
focusa contribute review
focusa contribute policy edit
focusa contribute purge
```

---

### 3.11 Export

```bash
focusa export history
focusa export manifest <export_id>
```

Start export (command):
```bash
focusa export start sft --output ./data.jsonl
```

---

### 3.12 Agents

```bash
focusa agents list
focusa agents show <agent_id>
focusa agents capabilities <agent_id>
```

---

### 3.13 Events (Streaming)

```bash
focusa events stream
```

Flags:
- `--types focus_state.updated,cache.bust`
- `--since <iso8601>`

---

### 3.14 Commands (Audit)

```bash
focusa commands list
focusa commands status <command_id>
focusa commands log <command_id>
```

---

## 4. Safety & Confirmation Rules

Commands that mutate state MUST:
- show a summary
- require confirmation
- support `--dry-run`
- support `--yes` for automation

---

## 5. Exit Codes

- `0` success
- `1` invalid usage
- `2` policy violation
- `3` not authorized
- `4` internal error

---

## 6. Canonical Rule

> **If the CLI cannot explain what happened, the system is wrong.**
