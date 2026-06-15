# Spec107 Evidence Taxonomy and Schema Design
**Bead:** focusa-bwky.3  
**Spec ref:** docs/107-spec-first-feature-lifecycle-and-claim-discipline-spec.md §5  
**Status:** Draft  
**Created:** 2026-06-15

## 1. Evidence Classification

Every evidence citation must be classified as one of:

| Class | Definition | Closure-acceptable |
|-------|-----------|-------------------|
| `actual` | Evidence from the exact runtime/platform/surface required by acceptance criteria | ✅ yes |
| `partial` | Evidence that covers some but not all acceptance criteria | ⚠️ name as unfinished |
| `surrogate` | Evidence from a different surface than required (e.g., API for native Mac) | ❌ no |
| `blocked` | Proof attempt failed because of environment/dependency boundary | ⚠️ name as blocked, requires operator acceptance |
| `missing` | No evidence submitted at all | ❌ no |

Evidence is always scoped to a **work item id** (bead id).

## 2. Evidence Citation Format

All evidence citations in `bd close --reason` must use this prefix:

```
Evidence citations: <list>
```

Each citation is a **stable ref** from one of:

| Format | Example | Used for |
|--------|---------|---------|
| `tests/<file>` | `tests/spec107_evidence_gate_test.sh` | Automated test proving closure |
| `docs/<path>` | `docs/current/FOCUSA_AWARENESS_ALGORITHM_DESIGN_2026-06-15.md` | Design doc or spec |
| `crates/<path>` | `crates/focusa-core/src/evidence.rs` | Production code |
| `apps/<path>` | `apps/menubar/src-tauri/src/main.rs` | App code |
| `git:<sha>` | `git:917f8c7` | Git commit hash |
| `/v1/<route>` | `/v1/evidence/classify` | API endpoint |
| `cargo test` | `cargo test evidence_gate` | CLI test output |
| `uiai:<session_id>` | `uiai:019eccba-...` | UIAI session screenshot |

When a citation covers multiple classes (e.g., some tests pass, others fail), annotate inline:

```
Evidence citations: tests/spec107_gate_test.sh (actual: tests 1-5; partial: test 6; blocked: test 7 due to glib version)
```

## 3. Evidence Metadata Schema

For programmatic gate use (Spec107 §5 pre-close gate):

```json
{
  "evidence_entry": {
    "id": "string",           // stable handle e.g. "test:spec107_gate:01"
    "work_item_id": "string", // bead id e.g. "focusa-bwky.3"
    "class": "actual | partial | surrogate | blocked | missing",
    "runtime": "string",      // e.g. "linux-x86_64", "macos-arm64", "pi-extension"
    "platform": "string",      // e.g. "cargo", "uiai", "api", "cli"
    "surface": "string",       // e.g. "crates/focusa-core", "apps/menubar"
    "command": "string",       // exact command run
    "artifact_path": "string", // file path, URL, or session handle
    "result": "pass | fail | error",
    "result_detail": "string", // stdout snippet or error message
    "blocker_reason": "string | null",  // for class=blocked
    "missing_aspects": ["string"] | null, // for class=partial
    "surrogate_of": "string | null",     // e.g. "native macos runtime"
    "captured_at": "ISO8601"
  }
}
```

## 4. Bead evidence_policy Field

Each bead may declare an `evidence_policy` in its description or notes:

```yaml
evidence_policy:
  required_classes:
    - actual       # must have at least one actual evidence
  acceptable_classes:
    - blocked      # blocked acceptable only with operator deferral
  forbidden_classes:
    - missing
  surfaces_required:
    - runtime: linux-x86_64
      surface: crates/focusa-api
    - runtime: macos-arm64
      surface: apps/menubar/src-tauri
  citations_minimum: 1
  citation_format: "Evidence citations: tests/... ; docs/... ; git:..."
```

The preclose gate validates that `close_reason` contains required evidence_policy citations before allowing closure.

## 5. Pre-Close Gate Design

**Input:**
```json
{
  "work_item_id": "focusa-bwky.4",
  "claim_text": "Preclose claim gate implemented",
  "acceptance_criteria": ["...", "..."],
  "close_reason": "Evidence citations: crates/focusa-api/src/evidence.rs ; git:b21d5ae",
  "evidence_policy": { ... }
}
```

**Output:**
```json
{
  "decision": "allow | block",
  "evidence_class": "actual | partial | surrogate | blocked | missing",
  "missing_evidence": ["..."],
  "overclaim_risks": ["surrogate evidence: API proof for native Mac runtime"],
  "recovery_commands": ["Run: uiai browser open http://... ; Capture native macOS screenshot"],
  "citations_parsed": [
    { "ref": "crates/focusa-api/src/evidence.rs", "class": "actual", "surface": "crates/focusa-api", "runtime": "linux-x86_64" }
  ]
}
```

**Decision rules:**
- `actual` coverage ≥ required surfaces → `allow`
- `partial` or `surrogate` only → `block`
- `blocked` with explicit operator deferral → `allow`
- `missing` → `block`
- `citations_minimum` not met → `block`

## 6. Mac Pairing Regression Fixture

This fixture is mandatory and must always fail (return `block`) when run without real macOS native evidence.

**Claim:** "Mac menubar pairing E2E complete."

**Acceptance criteria (actual required):**
- macOS `.app` bundle is built and signed (or unsigned for dev)
- Keychain persistence verified
- App restart persistence verified
- Native Tauri runtime launched on macOS
- Screenshot of native macOS menubar app running

**Surrogate evidence (must not close):**
- API/web pairing proof (localhost:8787)
- `focusa device pair-list` output showing paired device
- Web browser screenshots
- Pi extension session evidence
- Linux/CI cargo test proof

**Expected gate output:**
```
decision: block
evidence_class: surrogate
overclaim_risks: ["API/web proof cannot substitute native macOS Keychain + restart persistence"]
recovery_commands: ["Run uiai on macOS device; capture native menubar app screenshot; attach Keychain query output"]
```

## 7. Evidence Taxonomy in bd close_reason

The bd preclose gate checks that `close_reason` matches this pattern:

```
Evidence citations: <cite1> [; <cite2>...] [[(class: <annotation>)]]
```

Each `citeN` must be from the stable ref formats in §2.

**Forbidden patterns:**
```
Evidence citations:         # empty after prefix
Evidence: (nothing)         # wrong prefix
Proof: test.sh              # not in allowed format
Completed successfully      # no evidence prefix
```

## 8. Relationship to Existing System

- **bd pre-push hook**: Currently checks `close_reason` for `Evidence citations:` prefix and format compliance. This design extends that to also check `evidence_policy` and runtime/surface alignment.
- **focusa_evidence_capture tool**: Captures stable handles; output should include `class` field from this taxonomy.
- **focusa_workpoint_link_evidence**: Evidence linked to workpoints should carry `class` and `surface` metadata.
- **Beads JSONL**: `close_reason` is text; `evidence_policy` can be stored in `notes` or a new `evidence_metadata` field in the JSONL schema.

## 9. Open Questions

1. Should `evidence_policy` be a formal JSON field in bead JSONL, or continue to live in notes/description?
2. Should the preclose gate be enforced in `bd close` command itself (Rust), or only in pre-push hook (bd-hooks)?
3. Should evidence be stored in a separate `focusa_evidence` table in the SQLite backend for queryable history?
4. Should partial/blocked evidence require explicit operator approval before a bead can close?
5. Should the gate emit a structured JSON response or continue with human-readable text blocks?

**Resolution owner:** focusa-bwky.4 (implementation) will resolve Q1-Q3; operator will resolve Q4-Q5.
