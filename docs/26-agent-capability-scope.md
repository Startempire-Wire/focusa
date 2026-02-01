# docs/26-agent-capability-scope.md — Agent Capability Scope Model (AUTHORITATIVE)

This document defines how **agents** interact with Focusa as **capability consumers**.

Agents are not plugins.
Agents do not mutate cognition.
Agents **observe, query, propose, and explain**.

---

## 0. Canonical Principle

> **Agents may see everything.  
> Agents may change nothing directly.**

All agent action occurs through:
- observation
- proposal
- explanation
- command requests (subject to policy)

---

## 1. Agent Identity Model

Each agent registered with Focusa has:

```json
{
  "agent_id": "agent_uuid",
  "name": "string",
  "kind": "assistant | auditor | researcher | visualizer | trainer",
  "constitution_version": "semver",
  "default_permissions": [ "scope:*" ],
  "allowed_commands": [ "command_type" ],
  "created_at": "iso8601"
}
```

Agents are **long-lived identities**, distinct from:
- models
- harnesses
- sessions

---

## 2. Agent Capability Scopes

Scopes are **additive** and **explicit**.

### 2.1 Core Read Scopes (Default ON)

```json
{
  "state:read": true,
  "lineage:read": true,
  "references:read": true,
  "gate:read": true,
  "intuition:read": true,
  "constitution:read": true,
  "autonomy:read": true,
  "metrics:read": true,
  "cache:read": true,
  "events:read": true
}
```

This allows:
- full cognition introspection
- replay and explanation
- zero authority

---

### 2.2 Advisory Scopes (Optional)

```json
{
  "intuition:submit": true,
  "constitution:propose": true
}
```

These allow agents to:
- submit advisory signals
- draft constitution updates

They do **not** allow activation.

---

### 2.3 Command Request Scope (Highly Restricted)

```json
{
  "commands:request": true
}
```

Allows agents to:
- request commands
- receive approval/denial

Agents **cannot self-approve**.

---

## 3. Agent → Focusa Interaction Pattern

Agents follow this loop:

1. **Observe**
   - read Focus State
   - read CLT
   - read metrics

2. **Analyze**
   - detect patterns
   - compare outcomes
   - reason over lineage

3. **Propose**
   - suggestion (advisory)
   - constitution draft
   - command request

4. **Explain**
   - rationale
   - evidence
   - confidence

This mirrors human cognition and preserves safety.

---

## 4. Prohibited Agent Actions

Agents may NEVER:
- mutate Focus State
- bypass Focus Gate
- escalate autonomy
- write directly to Reference Store
- alter cache policy
- approve data contribution
- activate constitution versions

Violations are hard failures.

---

## 5. Why This Works

- Agents become *cognitive collaborators*
- Humans retain authority
- Focusa remains deterministic
- Multiple agents can coexist safely

---

## 6. Canonical Rule

> **Agents reason *with* Focusa — they do not reason *instead of* Focusa.**
