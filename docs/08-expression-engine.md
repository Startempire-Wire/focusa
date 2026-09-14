# docs/08-expression-engine.md — Expression Engine

## Purpose

The **Expression Engine** converts the current Focus State into language suitable for model invocation and governed human-facing expression.

It governs *what is said now*, not *what is known*.

Focusa historically describes the Expression Engine as the system's **voice**. [Spec 181](181-focusa-voice-conversation-expression-and-auditable-interaction-spec.md) now makes that relationship explicit for spoken interaction:

```text
Focus State / Workpoint / current ask
        ↓
Expression Engine
        ↓
ExpressionOutput
        ↓
text / visual presentation / speech renderer
```

The Expression Engine owns semantic expression. It does **not** own microphone capture, ASR, TTS rendering, speaker identity, audio devices, conversation storage, authority, or memory.

---

## Core Invariants

1. Deterministic output for fixed canonical inputs and profile/version
2. Explicit structure
3. Bounded token/content usage
4. No silent truncation
5. No reasoning or planning
6. No memory mutation
7. **Modality-neutral semantic output** — text, visual and spoken surfaces project the same governed meaning
8. **Synthesis cannot amend expression** — TTS/rendering may change presentation but not commitments, uncertainty, authority language, or semantic content
9. **Conversation is not memory** — an ExpressionOutput can be durably linked to a conversation without making the transcript canonical cognition

---

## Input

- Focus State (active frame)
- Selected parent frame context
- Current Ask / operator steering where applicable
- Optional surfaced candidates (annotated)
- Invocation metadata
- Expression profile
- Target modality/profile metadata where presentation constraints affect length/structure

The target modality may influence bounded presentation choices such as spoken brevity. It MUST NOT alter underlying authority or silently remove material warnings.

---

## Output Structure (Canonical)

1. System framing
2. Active intent
3. Constraints
4. Decisions
5. Relevant artifacts (handles only)
6. Failures (if relevant)
7. Next steps
8. Invocation-specific instructions

For human-facing conversation, the output is represented by a stable `focusa.expression_output.v1` reference defined by Spec 181 before speech synthesis or other presentation rendering.

---

## Token / Expression Budgeting

### Priority Order

1. Intent
2. Constraints
3. Decisions
4. Current state
5. Next steps
6. Failures
7. Artifacts

Lower-priority sections are shortened first.

All reduction is:

- explicit;
- logged;
- recoverable through the structured/full representation;
- prohibited from hiding material safety, authority, uncertainty, failure or consequence information.

Spoken output may use a concise delivery profile while the complete structured/text result remains available through the Conversation Ledger or owning surface.

---

## Degradation Strategy

If budget or modality constraints are exceeded:

- emit degradation state/event;
- annotate missing/deferred sections;
- preserve a handle to the complete structured representation where one exists;
- never silently drop meaning required for safe interpretation.

If speech rendering is unavailable, ExpressionOutput remains valid and may be shown through another surface. TTS failure is presentation failure, not cognitive failure.

---

## Forbidden Behaviors

- Implicit untracked summarization of material meaning
- Unregistered dynamic prompt shaping
- Content inference presented as source fact
- Memory mutation
- TTS/speech provider adding or changing semantic content
- Synthetic voice identity being interpreted as agent identity or authority
- A spoken-only result disappearing without a durable text/structured representation in a voice conversation

---

## Voice / Conversation Binding

Spec 181 owns:

- ConversationSession;
- ConversationParticipant;
- AudioSegment;
- SpeechHypothesis;
- UtteranceRecord;
- TranscriptRevision;
- ExpressionOutput reference semantics;
- SpokenOutput;
- Conversation Ledger;
- speaker attribution and correction lineage;
- interruption/floor semantics.

The outbound rule is:

> Every consequential agent utterance that is spoken in a governed conversation binds the exact ExpressionOutput that was rendered.

This allows the user to later determine **what the agent meant to say**, **what audio was rendered**, **whether playback was interrupted**, and **which actions/evidence followed**.

---

## Acceptance Criteria

- Output is reproducible under its declared profile/version
- Token/content usage is predictable and bounded
- Meaning is preserved
- Failures and uncertainty remain visible
- Spoken rendering cannot change the governed semantic output
- Every governed spoken response can be traced back to its ExpressionOutput
- Conversation history remains distinct from canonical memory/state

---

## Summary

The Expression Engine ensures **clarity without overload**, expressing only what matters *now*.

Spec 181 extends that original "system voice" concept into a complete auditable spoken-conversation primitive without moving cognition, identity, authority, or memory into the speech layer.
