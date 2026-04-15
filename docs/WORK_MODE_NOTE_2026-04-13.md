# Work Mode Note — 2026-04-13

## Operator expectation
- Do not emit conversational checkpoint replies during tranche execution.
- Keep working within a single turn until a real blocker or meaningful tranche milestone.
- Log checkpoints to `docs/REBASELINE_SINGLE_WRITER_SUMMARY_2026-04-13.md` instead of using chat as a progress sink.

## Runtime reality
- In this harness, once the assistant emits a chat reply, that turn ends.
- No additional tool execution can occur until the next user message arrives.
- Therefore the correct operating mode is: long silent tool-using turns, minimal replies, no interim progress pings.
