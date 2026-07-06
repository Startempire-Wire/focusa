# Newbie Onboarding and Walkthrough Experience QA

Purpose: ensure every onboarding stage and walkthrough is stellar, beautiful, and seamless for a human first-timer.

## Audit checklist (per stage)

### 1. First install
- [ ] Installer is one command: `bash scripts/install-daemon.sh /usr/local`.
- [ ] Output is plain-language; no jargon hidden in walls of text.
- [ ] Failure modes tell the user what to do next.

### 2. Daemon start
- [ ] `focusa start` works after install.
- [ ] Health check is one command: `curl -fsS http://127.0.0.1:8787/v1/health`.
- [ ] Failure hint points to `focusa doctor --scope host`.

### 3. Project bind
- [ ] `focusa init --quickstart` writes `.focusa-project.json` in seconds.
- [ ] Output shows `project_id` and explains what was written.

### 4. First mission
- [ ] `focusa walkthrough show --walkthrough first-mission` explains five steps:
  daemon → bind project → create Workpoint → attach evidence → resume.
- [ ] Each step has a plain-language “why” before any command.

### 5. Agent handoff
- [ ] `focusa walkthrough show --walkthrough agent-handoff` shows the next agent’s view of mission/Workpoint/boundaries/proof.

### 6. No Proof, No Done
- [ ] `focusa walkthrough show --walkthrough no-proof-no-done` makes evidence discipline visible.

### 7. Help overlay
- [ ] `h` or `?` opens Mission Deck help.
- [ ] Help explains Workpoint, Evidence, Recall, Mission Ladder, authority badges in plain language.

### 8. Mission Deck TUI
- [ ] Title: **Focusa Mission Deck**.
- [ ] Default tab: **Deck Home**.
- [ ] Deck Home shows: scope badge, proof meter, one primary next action, intent/focus, mission ladder, beginner orientation.
- [ ] `n` jumps to next safe action, `/` opens Recall, `Tab` switches tabs.

### 9. Recovery states
- [ ] Disconnected/unbound/no-workpoint/no-evidence/resumable/blocked states are visible to the user, not hidden behind errors.

### 10. Evidence education
- [ ] Beginner copy rules are used: “no proof yet” instead of canonical Workpoint errors.

## Source-backed claim matrix

| Claim | Source | Verified? |
|---|---|---|
| Mission Deck title and Deck Home default tab | `crates/focusa-tui/src/views/deck_home.rs`, headless proof | Yes |
| Beginner Mode decision tree | `crates/focusa-tui/src/beginner_mode.rs`, headless proof | Yes |
| Walkthroughs first-mission, agent-handoff, no-proof-no-done | `crates/focusa-cli/src/commands/walkthrough.rs` | Yes |
| Help overlay topics | `crates/focusa-tui/src/views/help_overlay.rs` | Yes |
| Proof Meter and Scope Badge states | `crates/focusa-tui/src/views/proof_status.rs` | Yes |
| Mission Ladder levels | `crates/focusa-tui/src/views/mission_ladder.rs` | Yes |
| Recall is advisory | `crates/focusa-tui/src/views/recall.rs` | Yes (lightweight surface) |
| Full Recall expansion | spec bead `focusa-117-arch.29` | Roadmap only |

## Open polish items

- `focusa-117-arch.24` Final TUI beautification pass.
- `focusa-117-arch.27` Full public GitHub Focusa sweep and onboarding docs polish.
- `focusa-117-arch.28` Final pre-MVP polish across every layer.

## Acceptance criteria for “stellar”

A first-time evaluator should be able to:

1. Clone → install → start → bind → open Mission Deck in five minutes.
2. Read every Deck Home, help, and walkthrough panel without prior Focusa knowledge.
3. Recover gracefully from disconnected, unbound, no-workpoint, no-evidence, or blocked states.
4. Attach or declare proof before declaring work done.
5. Verify scope before changing files.

If any step feels slow, confusing, or jargon-heavy, file a polish bead before launch.