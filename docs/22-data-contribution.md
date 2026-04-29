# docs/22-data-contribution.md — Opt-In Background Data Contribution (AUTHORITATIVE)

This document defines the **Focusa Opt-In Data Contribution System**:  
a local-first, user-controlled mechanism that allows Focusa users to
voluntarily contribute **high-quality cognition artifacts** for training
open models — continuously, safely, and transparently.

This system is designed to operate **in the background** without impacting
performance, autonomy, or trust.

---

## 0. Canonical Principle

> **Users contribute meaning, not surveillance.**  
> Focusa never uploads raw conversations, silent telemetry, or private intent.

Contribution is:
- explicit
- reversible
- inspectable
- limited
- ethical by construction

---

## 1. What Is Being Built (Definition)

The **Opt-In Data Contribution Layer (ODCL)** is a **post-cognition export pipeline**
that:
- observes current Focusa cognition artifacts after they are reducer-owned and evidence-linked
- filters them for eligibility
- queues them locally
- optionally uploads them for model training

It is **not** part of:
- Focus State
- CLT
- Reducer logic
- Prompt assembly
- Autonomy decision-making

---

## 2. Architectural Placement

```
Focusa Runtime
│
├── Focus State
├── Context Lineage Tree (CLT)
├── Reference Store
├── UXP / UFI / Autonomy Metrics
│
▼
Contribution Eligibility Filter
│
▼
Local Contribution Queue (Inspectable)
│
▼
User Policy Gate
│
▼
Background Export Worker
│
▼
Encrypted Dataset Sink
```

ODCL is **read-only** with respect to cognition.

---

## 3. Opt-In Lifecycle

### 3.1 Explicit Enablement

Contribution is **OFF by default**.

User must explicitly enable via:
- onboarding prompt
- CLI command
- UI toggle

No silent defaults. No dark patterns.

---

### 3.2 Contribution Policy (Local, Versioned)

Each user has a **local contribution policy object**:

```json
{
  "enabled": true,
  "dataset_types": ["sft", "preference", "contrastive", "long_horizon"],

  "min_uxp": 0.75,
  "max_ufi": 0.25,
  "min_autonomy_level": 0,

  "exclude_domains": ["private", "work", "confidential"],
  "require_manual_review": false,

  "upload_schedule": "idle_only | manual | scheduled",
  "network_policy": "unmetered_only | any",
  "power_policy": "plugged_in_only | any",

  "redaction_level": "high | medium | low",
  "consent_version": "v1.0"
}
```

Properties:
- editable at any time
- takes effect immediately
- versioned for auditability
- reversible

---

## 4. Eligibility Filtering (Conservative by Design)

A cognition artifact is eligible **only if all conditions pass**:

### 4.1 Required Conditions
- contribution enabled
- export_allowed = true
- Focus State complete
- CLT lineage intact
- no secrets detected
- license/consent valid
- UXP ≥ threshold
- UFI ≤ threshold

### 4.2 Automatic Exclusions
- private repositories
- marked confidential tasks
- failed or partial sessions
- incomplete Focus State
- missing provenance
- cached outputs reused across contexts

If uncertain → **exclude**.

---

## 5. What Gets Contributed (Strictly Limited)

### 5.1 Allowed
- Focus State snapshots
- CLT summaries (not raw turns)
- Structured tool outputs
- Reducer state transitions
- Preference / correction signals
- Autonomy outcome metrics

### 5.2 Explicitly Forbidden
- raw conversations
- raw prompts
- system messages
- provider fingerprints
- secrets or credentials
- personal identifiers

---

## 6. Local Contribution Queue

Eligible artifacts enter a **local, inspectable queue**:

```json
{
  "queue_item_id": "uuid",
  "dataset_type": "focusa_sft",
  "preview": {
    "goal": "string",
    "summary": "string",
    "outcome": "success"
  },
  "estimated_size_kb": 38,
  "status": "pending | approved | rejected | uploaded"
}
```

User capabilities:
- inspect each item
- approve / reject
- redact content
- pause queue
- purge history

Queue persists across restarts.

---

## 7. Background Export Worker

### 7.1 Execution Rules
- runs asynchronously
- never blocks cognition
- respects schedule, power, network policies
- rate-limited
- resumable
- idempotent

### 7.2 Security
- encrypted at rest
- encrypted in transit
- no stable user identifiers
- salted, non-reversible hashes
- per-batch anonymization

---

## 8. Upload Destinations (Pluggable)

Focusa supports multiple sinks:

### Option A — Central Focusa Dataset
- curated
- periodically released
- default for MVP

### Option B — Federated / P2P Dataset
- distributed aggregation
- no central authority
- future-ready

### Option C — Local / Team Export
- manual export only
- enterprise / research use

Sink selection is **explicit**.

---

## 9. CLI Interface

```bash
focusa contribute status
focusa contribute enable
focusa contribute pause
focusa contribute review
focusa contribute policy edit
focusa contribute export-now
focusa contribute purge
```

CLI must:
- show queue size
- show last upload
- show active policy
- never auto-enable

---

## 10. UI / Menubar Interface

Calm, non-nudging UI:

- Contribution: ON / OFF
- Queue size indicator
- Last upload timestamp
- Review contributions
- Pause contributions

No guilt messaging. No gamification.

---

## 11. Telemetry & Transparency

Every contribution event records:
- timestamp
- dataset type
- size
- eligibility reason
- upload outcome

Users can view:
- what was shared
- why it was eligible
- how it was used (when known)

---

## 12. Reciprocity & Trust

Contributors may receive:
- early access to trained models
- changelogs showing dataset impact
- optional attribution
- ability to fine-tune personal models

Participation is **collaboration**, not extraction.

---

## 13. Failure & Revocation

If contribution is disabled:
- queue halts immediately
- no further uploads occur
- existing uploads remain immutable
- future exports require re-consent

---

## 14. Canonical Rules (Print These)

1. **No opt-in → no export**
2. **No provenance → no export**
3. **If it surprises the user → don’t do it**
4. **Local control always wins**
5. **Contribution must earn trust every day**

---

## 15. One-Sentence Summary

> **Focusa enables ethical, background, opt-in data contribution by exporting curated cognition artifacts under full user control — turning everyday use into high-quality training data without sacrificing trust.**
