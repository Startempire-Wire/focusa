# Scope Failure and Relevance Tracing

## Purpose

Define how Focusa should trace, classify, and learn from scope failures such as:
- answering the wrong question
- importing nearby but irrelevant prior context
- over-broadening the answer subject
- failing to exclude adjacent threads

---

## Core Thesis

Scope failures should not be treated as vague assistant mistakes.

They should be:
- named
- classified
- traced
- reviewable
- improvable

This allows Focusa to reduce answer distortion over time.

---

## Failure Classes

## scope_contamination
Adjacent prior context distorts the current answer.

## wrong_question_answered
The answer addresses a neighboring question instead of the asked one.

## answer_broadening
The answer expands beyond the allowed scope without justification.

## adjacent_thread_leakage
A prior nearby thread is imported despite being irrelevant.

## context_overcarry
Too much prior context is carried forward by default.

---

## Trace Events

Focusa should eventually emit:
- `current_ask_determined`
- `query_scope_built`
- `relevant_context_selected`
- `irrelevant_context_excluded`
- `scope_verified`
- `scope_contamination_detected`
- `wrong_question_detected`
- `answer_broadening_detected`
- `scope_failure_recorded`

---

## Review Questions

After an answer, Focusa should be able to inspect:
1. Did the answer address the actual current ask?
2. Did any adjacent prior topic distort the answer?
3. Was broadening justified by the ask, or was it accidental?
4. What context should have been excluded?
5. What guardrail would have prevented the failure?

---

## Success Condition

Scope Failure and Relevance Tracing is successful when Focusa can make scope distortion visible enough that it becomes preventable instead of recurring silently.
