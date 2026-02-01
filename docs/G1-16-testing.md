# docs/16-testing.md — Testing & Acceptance (MVP)

## Testing Philosophy
Focusa correctness > cleverness.

We test:
- determinism
- stability
- long-horizon behavior
- integration neutrality

---

## Test Levels

### Unit Tests
- Focus Stack invariants
- Focus Gate pressure updates
- ASCC merge rules
- ECS store/resolve
- Prompt Assembly slot logic

---

### Integration Tests
- Daemon + CLI
- Turn lifecycle
- Worker pipeline
- Persistence across restart

---

### Harness Smoke Tests
- Wrap a harness CLI
- Run a multi-turn session
- Verify:
  - prompt size stabilizes
  - focus stack maintained
  - no corruption of output

---

### Long-Session Test
Script:
- 100+ turns
- multiple focus pushes/pops
- repeated artifacts
Pass criteria:
- prompt size plateaus
- memory bounded
- daemon remains responsive

---

## Acceptance Criteria (MVP)
1. Focus maintained over long sessions
2. Context does not grow unbounded
3. Priorities surface without hijacking
4. CLI and GUI reflect true state
5. Works as a proxy with a real harness

---

## Non-Regression
- Any change to prompt assembly requires snapshot comparison
- Any change to Focus Gate heuristics requires replay tests

---

## Final MVP Definition (Restated)
> A local cognitive runtime that stabilizes focus, compresses context without loss of meaning, and integrates transparently with existing AI harnesses.

If all tests pass, MVP is complete.

---

# UPDATE

# docs/16-testing.md (UPDATED) — New Acceptance Criteria

## New Acceptance Tests (Required)

- Multi-session isolation test
- Prompt degradation test
- Pinned item persistence test
- Time-based Focus Gate surfacing test
- Cross-session ECS access rejection test

---

## MVP Completion Gate (Updated)

MVP is complete only when:
- No silent degradation exists
- Human override always wins
- Long sessions remain stable
- Focus never auto-shifts
- All failures are observable
