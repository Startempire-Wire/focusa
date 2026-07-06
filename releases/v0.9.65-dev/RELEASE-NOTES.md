# v0.9.65-dev — 2026-07-06

Highlights:

- §5.12.11 — Cross-agent recent-turns adapter contract + daemon routes
  (/v1/turns/recent, /v1/events/recall-trigger).
- Adapter 1 (Pi): capture on turn_end, inject on
  before_agent_start/model_select, force-emit on session_compact,
  recall-intent detection in input hook.
- Adapter 2 (Claude Code): bash hook script
  (focusa-claude-code-hook.sh) capturing on Stop/PostToolUse/PreCompact,
  injecting on SessionStart/UserPromptSubmit. See
  adapters/claude-code/README.md for install.
- §5.11.5 — Provider Policy Ledger route backed by SQLite with staleness
  flip (focusa-rtcz).
- §5.12.10 — Operator recall-intent trigger word set
  (direct_recall / implicit_prior / coherence_loss / repetition /
  operator_steering).
- Axum 0.8 path syntax fix (`/v1/bloatgaurd/optical/ledger/:provider`
  → `/v1/bloatgaurd/optical/ledger/{provider}`).

Binary artifacts in this release:

- focusa-daemon-linux-x86_64 — Focusa metacognition daemon,
  binds FOCUSA_BIND (default 127.0.0.1:8787).
- claude-code-hook.sh — Adapter 2 drop-in.

Verify:

```bash
sha256sum -c SHA256SUMS
./focusa-daemon-linux-x86_64 # daemon binds 8787 by default
echo '{"session_id":"x","cwd":"/tmp","hook_event_name":"SessionStart"}'   | bash claude-code-hook.sh # should exit 0
```
