# docs/25-capability-permissions.md — Capability Permissions Model (AUTHORITATIVE)

This document defines how **access control and permissions** work for the
Focusa Capabilities API and CLI.

Permissions are **capability-scoped**, **explicit**, and **least-privilege by default**.

---

## 0. Canonical Principle

> **Observation is cheap. Authority is expensive.**

Read access is broad.  
Write access is narrow, explicit, and auditable.

---

## 1. Permission Model Overview

Permissions are expressed as **capability scopes**:

```
<domain>:<action>
```

Examples:
- `state:read`
- `lineage:read`
- `constitution:propose`
- `commands:submit`
- `contribute:approve`

---

## 2. Permission Classes

### 2.1 Read Permissions

Read permissions are **non-destructive** and safe.

Examples:
- `state:read`
- `lineage:read`
- `references:read`
- `metrics:read`
- `cache:read`
- `events:read`

Read permissions may still be filtered by policy (e.g., private refs).

---

### 2.2 Command Permissions

Command permissions allow **intentional mutation via commands**.

Examples:
- `commands:submit`
- `constitution:activate`
- `contribute:pause`
- `export:start`

Command permissions always require:
- policy validation
- audit logging
- autonomy checks

---

### 2.3 Administrative Permissions

Reserved for the local owner.

Examples:
- `admin:tokens`
- `admin:shutdown`
- `admin:config`

Not exposed to agents.

---

## 3. Default Permission Sets

### 3.1 Local Owner (CLI / UI)

```json
{
  "read:*": true,
  "commands:submit": true,
  "constitution:*": true,
  "contribute:*": true,
  "export:*": true,
  "admin:*": true
}
```

---

### 3.2 Agent (Default)

```json
{
  "state:read": true,
  "lineage:read": true,
  "references:read": true,
  "metrics:read": true,
  "intuition:read": true,
  "autonomy:read": true,
  "commands:submit": false
}
```

Agents can **observe cognition**, not control it.

---

### 3.3 External Tool / Integration

```json
{
  "state:read": true,
  "lineage:read": true,
  "events:read": true,
  "commands:submit": false
}
```

---

## 4. Permission Enforcement Rules

1. Every API request is authenticated
2. Permissions are checked per endpoint
3. Commands require explicit permission
4. Lack of permission → `403 forbidden`
5. Permissions are never inferred

---

## 5. Token Types

### 5.1 Owner Token
- Full permissions
- Stored locally
- Rotatable

### 5.2 Agent Token
- Scoped permissions
- Bound to agent_id
- Revocable

### 5.3 Integration Token
- Read-only by default
- Expirable

---

## 6. Policy Interaction

Permissions are necessary but not sufficient.

Even with permission:
- Focus Gate may block
- Autonomy level may prevent action
- Contribution policy may deny export
- Constitution rules may override

**Policy always wins over permission.**

---

## 7. Auditing & Visibility

Every denied or allowed command logs:
- token_id
- agent_id
- permission checked
- outcome
- reason

Users can inspect:
```bash
focusa commands log <command_id>
```

---

## 8. Permission Introspection

```bash
focusa agents capabilities <agent_id>
```

Returns:
- effective permissions
- denied scopes
- policy overrides

---

## 9. Extension Rules (Future)

New capability domains MUST:
- define read vs write scopes
- default to read-only
- document invariants
- be added to this file

No silent expansion allowed.

---

## 10. Canonical Rule

> **Permissions grant access.  
> Policy grants authority.  
> Cognition grants action.**
