---
name: focusa
description: Focusa cognitive runtime integration. Use when working with Focus State, decisions, constraints, or cognitive governance.
---

# Focusa Cognitive Runtime

## What is Focusa?
Focusa is the cognitive runtime that preserves focus, decisions, and constraints across sessions.

## Key Concepts
- **Focus Frame**: Current work context (title, goal, beads issue)
- **Focus State**: intent, decisions, constraints, failures, next_steps, open_questions
- **Thread Thesis**: Refined understanding of what's being worked on
- **CLT (Context Lineage Tree)**: Append-only history of interactions

## Tools Available
- `focusa_decide` — Record a significant decision with rationale
- `focusa_constraint` — Record a constraint from any source
- `focusa_failure` — Record a failure for learning

## Rules
1. Check constraints before acting — do not violate them
2. Do not contradict prior decisions without explicit reasoning
3. Use focusa_decide for architectural choices, approach selections, commitments
4. Use focusa_constraint for limitations, requirements, hard rules
5. Use focusa_failure for build errors, test failures, wrong assumptions

## API
- `GET /v1/focus/stack` — Current Focus Stack
- `POST /v1/focus/update` — Update Focus State delta
- `GET /v1/autonomy` — ARI score and autonomy level
- `POST /v1/reflect/run` — Trigger reflection
