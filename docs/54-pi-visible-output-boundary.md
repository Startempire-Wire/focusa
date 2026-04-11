# Pi Visible Output Boundary

## Purpose

This document prevents loops and channel confusion caused by internal Focusa context being echoed into visible user output.

## Channel Separation Rule

There are two different channels:

### Internal context channel
Used for:
- injected ontology slices
- daemon-provided focus state
- steering text
- hidden runtime directives
- internal execution metadata

### Visible response channel
Used for:
- user-facing answers
- explanations
- requests for clarification
- progress summaries
- visible results

These channels must remain separate.

## Prohibited Output

Pi must never echo into visible output:
- full focus state blocks
- injected daemon summaries
- internal steering directives
- raw internal context hook text
- hidden ontology slice payloads
- internal-only action payloads

## Anti-Echo Safeguards

The extension must implement safeguards such that:
- injected internal blocks are tagged internal-only
- visible-output generation excludes internal-only blocks
- copied response text does not re-ingest internal payloads as user-facing content
- replay and compaction do not accidentally promote internal state text into visible history

## Internal prominence boundary

The problem is not only visible leakage.
It is also prompt-level prominence.

Even if internal state is never echoed verbatim, it may still distort attention if injected too aggressively.

### Rule
Internal context must be bounded in both visibility and attentional prominence.

## Success Condition

This document is satisfied when internal Focusa/ontology context reliably guides Pi without appearing in visible output unless explicitly requested through a separate safe rendering path.
