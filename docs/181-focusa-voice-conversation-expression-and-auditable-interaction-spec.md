# 181 — Focusa Voice, Conversation, Expression, and Auditable Interaction Primitive

**Status:** DRAFT canonical primitive direction, 2026-09-04.
**Canonical human architecture authority:** Verious Smith III under the repository architecture-authority policy.
**Primitive owner:** Focusa Core for conversation meaning/lineage; Expression Engine for semantic expression; execution surfaces remain owned by their runtimes.
**Preserves:** Doc 08 Expression Engine, Spec 151 modality parity, Workpoint/Trajectory authority, Spec 136 settlement, Spec 137 temporal authority, Spec 140 runtime constitution, Spec 141 generated capabilities.

## 1. Decision

Voice is a first-class Focusa interaction modality, not a transcription plugin and not a separate cognitive system.

Focusa already defines the **Expression Engine** as the system's voice: it governs what current Focus State is expressed without becoming memory or planning authority. This specification extends that principle into spoken interaction while preserving the original boundary:

```text
what Focusa knows / means
        !=
conversation transcript
        !=
what is spoken now
        !=
audio encoding or TTS voice
```

The target contract is:

> A Focusa-aware system can conduct continuous, interruptible, multi-speaker spoken interaction; preserve complete inspectable conversational provenance; turn speech into the same canonical operations available to other surfaces; and speak responses derived from the same governed state without making conversation itself canonical memory or authority.

## 2. Foundational laws

1. **Conversation is not memory.** A durable transcript may be complete and searchable without becoming canonical Focus State, Workpoint, policy, ontology, or durable knowledge automatically.
2. **Voice is not authority.** A recognizable voice, voiceprint, TTS voice, speaker label, wake word, or diarization result never grants permission by itself.
3. **One operation, many modalities.** Voice, CLI, TUI, Desktop, Pi, generated UI, and compatible agents invoke the same canonical Focusa operations rather than separate voice-only semantics.
4. **Speech ingress is evidence-bearing observation.** ASR output is a hypothesis linked to source audio/timing and confidence, not unquestionable user text.
5. **Expression precedes synthesis.** Focusa Expression Engine owns the semantic text/content to express; TTS owns rendering that content into audio.
6. **Audio rendering is presentation.** Voice model, accent, prosody, speed, emotional style, and codec do not change semantic authority.
7. **Every speaker is attributable or explicitly unknown.** Never silently assign an ambiguous human voice to an enrolled principal.
8. **Every agent speaker has a stable principal ref.** A synthesized voice or display name is not agent identity.
9. **Interruption is first-class.** Human barge-in, agent interruption, overlapping speech, cancellation, and floor transfer are explicit events.
10. **No keyboard/mouse dependency may be introduced by Focusa operations.** Any operation intended for ordinary human use must be invokable and understandable through a nonvisual structured surface suitable for voice projection.
11. **High-consequence ambiguity fails closed.** Low-confidence or materially ambiguous speech never silently becomes an irreversible mutation.
12. **Complete conversational provenance is preserved locally by default.** The user can inspect what each participant said, what the system believed was said, what was spoken back, and what actions followed.
13. **Raw transcript volume does not pollute the canonical event log.** Large transcript/audio payloads live in bounded content-addressed conversation storage; Focusa events carry handles, hashes, participant/scope metadata, and semantic links.
14. **Promotion is explicit.** A conversational claim becomes Workpoint state, Evidence, memory, knowledge, policy, or ontology only through its existing promotion/verification path.
15. **Speaker attribution confidence and content confidence are separate.** The system may understand the words while being uncertain who said them, or identify the speaker while being uncertain about the words.
16. **Corrections append.** Revised ASR, corrected speaker attribution, and user transcript corrections preserve prior hypotheses and append a superseding revision.
17. **Private audio is not telemetry.** Audio and transcript content follow conversation retention/privacy policy and are not exported under generic metrics/event collection.

## 3. Relationship to Doc 08 Expression Engine

Doc 08 remains the semantic expression owner.

The canonical outbound sequence is:

```text
Focus State / Workpoint / current ask
        ↓
Expression Engine
        ↓
ExpressionOutput
        ↓
Speech Renderer / TTS
        ↓
SpokenOutput artifact
```

TTS MUST NOT independently summarize, embellish, alter commitments, add authority language, hide uncertainty, or manufacture a different answer.

A spoken response records the exact `expression_output_ref` and digest that it rendered.

## 4. Relationship to Spec 151 modality parity

Spec 151 §56 requires important actions to remain available through applicable keyboard, screen-reader, CLI, TUI, nonvisual structured output, and future voice/mobile surfaces.

This specification activates the voice half of that direction:

> Voice is no longer treated as an unspecified future presentation surface. It is a first-class modality projection of canonical Focusa operations.

The Program Design Runtime should therefore be able to answer not only "which operation can perform this?" but also "how can the human invoke, understand, confirm, interrupt, and review it through voice?"

## 5. Core object model

### 5.1 ConversationSession

```yaml
schema: focusa.conversation_session.v1
conversation_id:
scope:
  project_ref:
  continuity_id:
  workpoint_ref:
  work_surface_ref:
started_at:
ended_at:
participant_refs: []
modality: voice | mixed | text
input_device_refs: []
output_device_refs: []
privacy_policy_ref:
retention_policy_ref:
transcript_ledger_ref:
event_cursor_ref:
```

A conversation may exist without a project only where the operation family permits unscoped interaction. Project-bound mutations still require exact Focusa scope.

### 5.2 ConversationParticipant

```yaml
schema: focusa.conversation_participant.v1
participant_ref:
kind: human | agent | system | unknown_human | external_remote
principal_ref:
display_label:
agent_role_ref:
voice_presentation_ref:
speaker_enrollment_ref:
identity_confidence:
authority_ref:
```

`voice_presentation_ref` is presentation only. `speaker_enrollment_ref` may assist attribution but MUST NOT independently authorize actions.

### 5.3 AudioSegment

```yaml
schema: focusa.audio_segment.v1
audio_segment_id:
conversation_ref:
source_participant_candidate_refs: []
started_at:
ended_at:
codec:
sample_rate_hz:
channels:
content_ref:
content_sha256:
local_storage_class:
retention_ref:
```

Raw audio MAY be disabled or retained for a shorter period than transcript records. When raw audio is not retained, the record states that explicitly.

### 5.4 SpeechHypothesis

```yaml
schema: focusa.speech_hypothesis.v1
hypothesis_id:
audio_segment_ref:
engine_ref:
engine_version:
language:
text:
word_timings: []
confidence:
alternatives: []
created_at:
```

### 5.5 UtteranceRecord

```yaml
schema: focusa.utterance.v1
utterance_id:
conversation_ref:
speaker_ref:
speaker_confidence:
source_audio_refs: []
transcript_revision_ref:
text:
language:
started_at:
ended_at:
addressed_to_refs: []
reply_to_utterance_ref:
interrupted_utterance_ref:
status: provisional | accepted | corrected | superseded | disputed
semantic_intent_refs: []
action_proposal_refs: []
evidence_refs: []
```

### 5.6 TranscriptRevision

```yaml
schema: focusa.transcript_revision.v1
revision_id:
utterance_ref:
parent_revision_ref:
source: asr | human_correction | agent_correction | diarization_reconciliation
text:
speaker_ref:
reason:
created_at:
```

Corrections never erase the original machine hypothesis.

### 5.7 ExpressionOutput

```yaml
schema: focusa.expression_output.v1
expression_output_id:
conversation_ref:
speaker_agent_ref:
source_focus_revision:
current_ask_ref:
content:
content_digest:
uncertainty_refs: []
spoken_delivery_policy_ref:
created_at:
```

### 5.8 SpokenOutput

```yaml
schema: focusa.spoken_output.v1
spoken_output_id:
expression_output_ref:
speaker_agent_ref:
voice_presentation_ref:
tts_engine_ref:
audio_artifact_ref:
started_at:
ended_at:
interrupted_at:
completion_status: completed | interrupted | failed | skipped
```

## 6. Conversation Ledger

Focusa SHALL maintain a durable **Conversation Ledger** for inspectable interaction history without conflating it with canonical cognition.

The ledger must support:

- complete ordered utterance history;
- participant/speaker attribution;
- exact agent principal attribution;
- ASR hypothesis and correction lineage;
- outbound ExpressionOutput and SpokenOutput linkage;
- word/segment timing where available;
- interruption and overlap events;
- addressed-to and reply-to relationships;
- operation/action proposal refs triggered from an utterance;
- Evidence/Receipt/settlement refs resulting from the conversation;
- transcript search and timeline replay;
- project/continuity/Workpoint binding;
- export in a stable machine-readable format;
- local retention/deletion/legal-hold policy where applicable.

Large text/audio blobs MUST be handle-addressed. The canonical Focusa event chain records content hashes/refs and consequential semantic events instead of copying every raw audio byte or full transcript into every event.

## 7. Transcript truth model

The system must distinguish:

```text
source audio
    ↓
ASR hypothesis
    ↓
accepted transcript revision
    ↓
interpreted intent
    ↓
canonical operation proposal
```

These stages MUST NOT collapse.

Particularly sensitive strings—names, numbers, financial amounts, dates, paths, command arguments, addresses, identifiers, and destructive targets—require stricter confidence/confirmation policy before consequential use.

Example:

```text
Heard: "delete project alpha"
ASR confidence: 0.63
possible alternative: "delete project alfa"

Result:
blocked pending exact target clarification
```

not:

```text
execute guessed deletion
```

## 8. Full-duplex and interruption model

Voice interaction SHOULD feel conversational rather than like a walkie-talkie.

Required states include:

```text
listening
user_speaking
agent_thinking
agent_speaking
overlap
interrupted
awaiting_clarification
awaiting_confirmation
muted
unavailable
```

Barge-in rules:

- human speech may immediately interrupt noncritical agent speech;
- interruption stops or ducks audio rendering without falsely cancelling the underlying Focusa operation;
- if spoken output contains a pending material warning/confirmation, the system preserves the unread/unheard remainder and exposes it for review;
- an interrupted response remains in the Conversation Ledger with exact stop position;
- agent resumption uses the newest user utterance as steering authority;
- no agent may continue a stale spoken monologue over newer operator direction.

## 9. Conversation floor and group interaction

Multi-human and multi-agent conversations are first-class.

A `ConversationFloorState` tracks:

```yaml
schema: focusa.conversation_floor.v1
conversation_ref:
active_speaker_refs: []
pending_speaker_refs: []
floor_generation:
interruption_policy_ref:
moderator_ref:
updated_at:
```

Overlapping speech is preserved rather than flattened into a fabricated serial transcript.

Every agent contribution MUST carry:

- stable agent principal ref;
- role/expertise ref where relevant;
- exact utterance timing;
- addressed-to/reply-to refs;
- ExpressionOutput ref;
- action/Evidence refs where applicable.

A user must be able to review a group conversation and answer questions such as:

- Which agent said this?
- Which expert disagreed?
- Which statement caused the action?
- What did I say immediately before the deployment?
- Which participant interrupted whom?
- Which claims were later corrected?

## 10. Voice identity and authentication

Speaker recognition MAY assist attribution and personalization.

It MUST NOT by itself:

- unlock credentials;
- approve spending;
- approve destructive operations;
- alter architecture authority;
- grant cross-project access;
- impersonate another enrolled principal.

High-consequence actions require the same Focusa authority and trusted-session/confirmation controls as other modalities. A voice surface must be capable of completing those controls without requiring a keyboard or mouse, but may require a separately authenticated presence/device factor.

## 11. Voice operation projection

Voice is not a giant list of magic phrases.

The surface maps natural language to canonical operation discovery and execution:

```text
spoken request
→ current-ask extraction
→ Focusa operation/capability discovery
→ bounded interpretation
→ preview/clarification when required
→ normal authority and consequence gate
→ execution
→ spoken + structured result
```

Examples:

```text
"Open the current project and tell me what's blocked."
"Have another team work on the verification failures."
"Show me the spreadsheet and fix the totals."
"Stop every agent touching production."
"What did the security agent say about this yesterday?"
"Go back to the conversation where we decided the database migration."
```

All of these route through canonical operations and transcript/Workpoint/Evidence relationships rather than bespoke speech-only code.

## 12. Complete non-keyboard/mouse usability target

For any supported full Agent Computer experience, Focusa's operation layer must permit voice projection for ordinary tasks including:

- project/work navigation;
- search and retrieval;
- application/capability discovery;
- launching and closing work surfaces;
- reading and editing documents;
- browser work;
- source/code work;
- file/resource operations;
- agent/team creation and steering;
- pause/stop/takeover/return control;
- approvals and clarifications;
- system/status queries;
- communications workflows;
- review of Evidence and Receipts;
- recovery and continuation;
- transcript/audit search;
- shutdown/restart/session lifecycle where policy permits.

A feature that can only be understood or completed with pointer/keyboard interaction fails voice modality parity unless explicitly classified as unsupported/degraded.

## 13. Spoken result policy

Speech should optimize for comprehension rather than reading every UI field aloud.

Responses may use:

- concise spoken summary;
- progressive detail on request;
- earcons/status tones for nonsemantic state;
- participant-specific synthetic voices where useful;
- spatial or channel cues where hardware supports them.

But every spoken result must have a complete structured/text representation in the ledger so the user can later inspect exactly what occurred.

## 14. Retrieval and audit

The user must be able to navigate conversation history semantically and temporally:

```text
"What did I ask right before the release failed?"
"Find the conversation with the accounting agent about invoice 42."
"Read back what the security expert recommended."
"Which agent told me this was safe?"
```

Retrieval returns transcript/utterance refs and surrounding context.

Transcript retrieval is evidence/provenance. It does not override a newer Workpoint, authoritative external record, correction, or canonical policy merely because something was said earlier.

## 15. Privacy and retention

Default posture:

```text
transcripts: durable local-first conversation history
raw audio: local-first, retention-policy controlled
cloud upload: explicit purpose/authority required
telemetry: content excluded by default
```

Conversation content MUST carry privacy classification and scope.

The implementation should permit user-visible retention controls without allowing a generic "telemetry off" switch to erase Evidence, security records, or legally/operationally required audit state. Those are separate retention domains.

## 16. Provider independence

Focusa owns the conversation contracts, not a specific ASR/TTS vendor.

Adapters may include:

- local ASR;
- cloud ASR;
- local TTS;
- cloud TTS;
- real-time speech-to-speech models;
- hardware microphone arrays;
- remote/mobile audio endpoints.

A provider MAY accelerate interaction but MUST NOT become canonical conversation, identity, authority, Workpoint, transcript, or settlement storage.

## 17. Degraded operation

Voice failure must degrade explicitly:

- microphone unavailable;
- output device unavailable;
- ASR unavailable;
- TTS unavailable;
- high-noise/low-confidence;
- speaker attribution uncertain;
- network speech provider unavailable;
- transcript persistence unavailable.

Where possible, text/structured surfaces remain usable. A full voice-profile release cannot claim voice completeness while a required operation silently falls back to keyboard-only interaction.

## 18. Acceptance invariants

A production voice surface is not accepted until it proves:

1. two-way spoken conversation without keyboard/mouse for a representative full workflow;
2. barge-in/interruption without stale-agent continuation;
3. ASR uncertainty and correction lineage;
4. multi-speaker attribution with explicit unknown/uncertain states;
5. two or more agent speakers remain independently attributable in a group conversation;
6. every spoken agent response binds an ExpressionOutput ref;
7. every consequential spoken command traverses normal Focusa authority/settlement;
8. Conversation Ledger survives restart and supports semantic/time/speaker search;
9. transcript correction preserves prior revision history;
10. raw audio/transcript content does not leak into telemetry by default;
11. transcript presence does not make conversation canonical memory;
12. voice identity alone cannot authorize consequential actions;
13. no required ordinary workflow step depends on keyboard or mouse in a declared voice-complete profile.

## 19. Cross-system consumption

Veragensia owns OS audio-device integration, secure attention, session presence and the voice-native Agent Computer experience.

UIAI Engine owns browser/computer execution and may supply computer-control observations/actions initiated by voice.

Pi and other harnesses consume the same current ask/operation/Workpoint state; they do not maintain separate voice truth.

Focusa Desktop presents Conversation Ledger, participant, transcript, action and Evidence relationships without becoming their authority.

## 20. Final principle

> The human may speak naturally. Focusa preserves what was actually observed, what it believed was said, what each participant expressed, what action was authorized, and what happened next—without confusing conversation with truth or voice with authority.
