# Spec 184 — Focusa Ambient Operator: Mobile/Wearable Presence, Conversation, Meeting Capture, Identity Routing, and Sync

**Status:** DRAFT canonical architecture direction, 2026-09-05.  
**Canonical human architecture authority:** Verious Smith III under the repository architecture-authority policy.  
**Historical basis:** operator-supplied cross-account architecture handoff describing prior `Proposal 135M`, Focusa earbuds/ambient voice, Focusa Mobile/NavisAI, Radar, Foreman and Wirebot integration. No current default-branch artifact named `135M` exists in the current frozen 135A–135K series. **This specification preserves the architectural ideas without reusing stale proposal numbering.**  
**Primitive owners preserved:** Doc 08 Expression Engine, Spec 53 pairing, Spec 72 identity/role, Spec 137 temporal authority, Spec 139 presence/environment, Spec 164 Workstream Root, Spec 181 Voice/Conversation, Spec 182 Foreman, Spec 183 Radar, Spec 136 settlement, Spec 156 credentials, Spec 117A mobile/PWA living-field direction.  
**Cross-system companions:** Veragensia Doc 197 voice-native Agent Computer and Doc 199 Ambient Operator/Omarchy integration; UIAI Engine control/voice binding; Wirebot Core Context Core/Phone Bridge/voice adapters.

---

## 0. One-line definition

> **Focusa Ambient Operator is the mobile/wearable projection of the Focusa operating environment: a continuously available, privacy-bounded human interface that can carry presence/context, conduct auditable voice conversations and meetings, route the human to Wirebot or the correct Project Foreman, surface Radar attention, and synchronize safely with self-hosted Focusa/Veragensia without making the phone, earbuds, transcript, or cloud service a second cognitive authority.**

The earbuds are one possible audio endpoint. The phone is one possible mobile runtime. The architecture is broader than either device.

---

## 1. Product shape

```text
                         HUMAN
                           |
                 +---------+---------+
                 |                   |
              EARBUDS              PHONE
            ears / mouth        sensors / UI
                 |                   |
                 +---------+---------+
                           |
                    AMBIENT OPERATOR
                           |
             +-------------+-------------+
             |             |             |
        Conversation     Presence       Meeting
         Spec 181       projection      capture
             |             |             |
             +-------------+-------------+
                           |
                    Identity Router
                 +---------+---------+
                 |                   |
              Wirebot            Foreman
          Chief of Staff       Workstream role
                 |                   |
                 +---------+---------+
                           |
                        Focusa
                  authority / proof
                           |
              +------------+------------+
              |                         |
            Radar                    Execution
       proactive attention       Pi/UIAI/Veragensia
```

The human should be able to move through life without carrying a desktop interaction model in their head.

---

## 2. Core distinction: Ambient Operator != ambient surveillance

The system may be **ambiently available** without **ambiently retaining everything**.

Required listening/capture modes:

```text
off
context_only
wake_word
conversation
meeting
private_note
execution_supervision
```

### `off`

No microphone capture and no active ambient context publication beyond explicitly required device health.

### `context_only`

No conversation recording. Device/runtime may publish bounded context signals such as coarse presence, activity, driving, DND, meeting state, battery, device/earbud presence, connectivity, and interruptibility under owner policy.

### `wake_word`

Local/on-device wake detection where practical. Pre-trigger audio exists only in a short volatile ring buffer and is discarded when no wake event occurs.

### `conversation`

Active Spec-181 ConversationSession between the human and one or more agents.

### `meeting`

Multi-human ConversationSession with explicit capture/retention/consent policy, speaker diarization, transcript revision lineage, and optional agent participation.

### `private_note`

One bounded owner utterance captured as a personal note/candidate without opening an unrestricted room recording.

### `execution_supervision`

A voice session remains bound to a running Foreman/worker/UIAI/Veragensia operation for natural steering, pause, evidence review and recovery.

---

## 3. Presence ownership

Focusa does **not** become the raw GPS/sensor collector for the owner's life.

The mobile/device owning domain may collect and derive context according to platform permission and owner policy. For the Startempire reference deployment, Wirebot Context Core is an existing owner-domain implementation for phone/location/activity/interruptibility context.

Focusa consumes only a bounded **Ambient Presence Projection** when relevant.

```yaml
schema: focusa.ambient_presence_projection.v1
projection_id:
operator_principal_ref:
device_principal_ref:
source_system_ref:
observed_at:
freshness_ref:

context:
  coarse_place: home | office | away | transit | unknown
  activity: stationary | walking | driving | meeting | focused | social | unknown
  interruptibility: low | medium | high | unknown
  timezone_ref:
  device_present:
  earbuds_present:
  network_posture:

location:
  class: none | on_device_only | coarse | precise_task_scoped
  precise_resource_ref:

privacy_policy_ref:
consent_policy_ref:
evidence_refs: []
```

Precise coordinates SHOULD remain with the owning context service unless an exact task requires and authorizes them.

A `precise_resource_ref` is a scoped handle, not license to copy GPS into Focusa history.

---

## 4. Ambient Presence Session

```yaml
schema: focusa.ambient_session.v1
ambient_session_id:
operator_principal_ref:
device_principal_ref:
pairing_ref:
mode:
started_at:
ended_at:

workstream_scope_ref:
foreman_ref:
wirebot_principal_ref:
conversation_ref:
radar_subscription_ref:
presence_projection_ref:

input_endpoint_refs: []
output_endpoint_refs: []
privacy_policy_ref:
retention_policy_ref:
sync_state_ref:
```

The session binds interaction context. It does **not** create application permission.

---

## 5. Identity routing

The Ambient Operator can expose more than one conversational level without inventing separate memories.

### 5.1 Chief-of-Staff route

Explicit address such as:

```text
"Wirebot, what needs my attention today?"
```

routes to the configured Chief-of-Staff principal/runtime, whose broader owner-authorized context may aggregate many Workstreams and external life/business systems.

### 5.2 Project Foreman route

Explicit address such as:

```text
"Foreman, what's happening with Focusa?"
```

routes to Spec-182 Foreman only after an exact Workstream can be resolved.

### 5.3 Resolution order

```text
explicit named principal/role
→ explicit project/workstream
→ already-bound Ambient/Conversation scope
→ bounded recent reference
→ clarify
```

Never route consequential speech by a guessed project merely because the phone recently showed it.

### 5.4 Synthetic voice is not identity

Agent voices are presentation. Principal and role refs travel separately in every utterance and action.

---

## 6. Earbuds and wearable endpoints

Earbuds are **audio endpoints**, not cognitive authorities.

A commodity Bluetooth headset can provide:

- microphone;
- playback;
- media/gesture buttons where the platform exposes them;
- proximity/connection state.

Future Focusa-specific hardware MAY add:

- dedicated wake/push-to-talk control;
- hardware mute/privacy indication;
- physical bounded approval/deny controls;
- physical emergency stop;
- low-latency reconnect;
- improved microphone array/battery behavior.

No hardware button gains authority merely because it exists. It becomes a valid authority input only when bound through a trusted paired-device/presence policy and exact pending operation.

---

## 7. Physical-control semantics

Potential future mappings may include:

```text
single tap      → acknowledge / repeat
hold            → push-to-talk
bounded approve → resolve exact currently announced secure prompt
long hold       → request emergency stop/freeze
remove earbuds  → pause/duck ambient conversation by policy
```

Mappings are configurable and device-specific.

A physical approval MUST include:

- trusted paired device identity;
- exact pending prompt/operation;
- freshness/expiry;
- owner presence policy;
- resulting authority Receipt.

Generic media-button events MUST NOT become universal approval tokens.

---

## 8. Wake-word contract

Wake-word detection is an attention mechanism, not authentication.

Preferred flow:

```text
microphone
→ short volatile local ring buffer
→ local wake/VAD detector
    no wake → discard
    wake    → start/attach ConversationSession
→ ASR / utterance
→ normal Focusa operation path
```

Rules:

1. pre-trigger audio is not retained by default;
2. wake detection SHOULD be local when practical;
3. wake word does not establish speaker identity;
4. wake word does not grant capability;
5. false wakes are observable and tunable;
6. user can audibly/visually query and change listening mode;
7. platform suspension/interruption is surfaced honestly.

---

## 9. Meeting capture

Meeting capture is a first-class ConversationSession profile, not a recorder bolted onto Radar.

Required provenance:

- meeting/session identity;
- start/end/time authority;
- recording/capture policy;
- participant candidates;
- diarization/speaker confidence;
- source audio segment handles where retained;
- ASR hypotheses;
- accepted/corrected transcript revisions;
- agent participation where applicable;
- utterance → proposal/action/Evidence/Receipt lineage;
- retention/legal-hold/deletion state.

### 9.1 Conversation remains distinct from canonical mission state

A meeting statement such as:

```text
"We'll ship Friday."
```

may become:

```text
CommitmentCandidate
```

with speaker/source/confidence, but does not silently change a Workpoint, deadline, policy or contract.

Promotion remains explicit through existing Focusa operations.

### 9.2 Consent and recording policy

The system MUST expose whether capture/recording is active and preserve the configured consent policy.

Because recording rules vary by jurisdiction, organization, meeting type and participant relationship, Focusa MUST NOT claim that a generic `meeting=true` state proves recording is lawful. A deployment must configure the applicable consent/recording policy and the app must fail closed or request human confirmation where required.

This is a product policy/UX requirement, not legal advice embedded in the reducer.

---

## 10. Radar integration

Spec 183 Radar may receive bounded Ambient observations such as:

```text
owner entered meeting
owner is driving / low interruptibility
paired earbuds disconnected
meeting transcript produced a follow-up candidate
important commitment deadline approaching
owner returned to a high-interruptibility state
```

Radar MUST NOT receive raw continuous microphone audio merely to decide whether to interrupt.

Ambient attention flow:

```text
Radar notices
→ Foreman investigates if project-specific
→ Focusa determines action/authority posture
→ Ambient Operator surfaces only the valuable interruption
```

This is the preferred alternative to generic push-notification floods.

---

## 11. Foreman interaction modes

Ambient Operator supports:

### Quick command

```text
"Foreman, have the verifier inspect the latest failure."
```

### Live discussion

```text
"Let's rethink the browser identity model."
```

### C.R.I.S.T. interview

```text
Foreman actively interviews the owner to refine project Context/Role/Spec/Tasks.
```

### Review

```text
"Walk me through what changed overnight and prove the deployment result."
```

### Execution supervision

```text
"Stop there."
"Run the tests."
"Read back what changed."
"Try the alternative."
"Give me control."
```

All modes invoke the same canonical operations as Desktop/Pi/API.

---

## 12. Sync architecture

Ambient Operator MUST be transport-abstract.

```text
AmbientSync
   |
   +-- nearby BLE / Bluetooth control
   +-- local Wi-Fi / LAN
   +-- Tailscale/private network
   +-- Internet relay where explicitly enabled
   +-- offline encrypted queue
```

### 12.1 Bluetooth role

Bluetooth/BLE is appropriate for:

- earbud audio routing through the mobile OS;
- nearby-device discovery;
- proximity;
- small control messages;
- bounded state deltas;
- pairing assist;
- offline-nearby emergency/status interactions where implemented.

It SHOULD NOT be the only canonical transport for hours of meeting audio, large transcript segments, rich Evidence, or workspaces.

### 12.2 Bulk/default synchronization

For the reference self-hosted model, authenticated LAN/Tailscale/private-network transport is preferred when available. Cloud relay is optional.

---

## 13. Ambient Sync Envelope

```yaml
schema: focusa.ambient_sync_envelope.v1
envelope_id:
device_principal_ref:
pairing_ref:
device_sequence:
created_at:
expires_at:

scope_ref:
conversation_segment_refs: []
presence_projection_refs: []
utterance_refs: []
control_intent_refs: []
receipt_refs: []
ack_refs: []

privacy_class:
encryption_ref:
signature_ref:
idempotency_key:
```

Requirements:

- monotonic device sequence or equivalent replay defense;
- idempotency for retries;
- encrypted at rest while queued;
- authenticated/encrypted in transit;
- exact pairing/device identity;
- no credential values in ordinary envelopes;
- source timestamps plus trusted receipt time;
- conflict/reconciliation rather than last-write-wins for authority-bearing state.

The mobile app syncs **proposals/observations/conversation segments**, not direct reducer database writes.

---

## 14. Offline behavior

The mobile surface remains useful while disconnected.

It MAY:

- record an authorized meeting locally;
- transcribe locally if capability exists;
- capture a private note;
- retain pending Conversation Ledger segments;
- show last-known state clearly marked stale;
- queue nonconsequential proposals/intents.

It MUST NOT pretend it can:

- prove remote Workpoint freshness;
- approve an expired remote prompt;
- execute a remote action without connectivity unless a pre-authorized local capability exists;
- infer successful sync from queue persistence.

On reconnect:

```text
pairing/auth refresh
→ sync envelope replay with idempotency
→ Focusa reconciliation
→ receipts/acks
→ prune acknowledged queue according to retention
```

---

## 15. Phone/context adapter boundary

The Startempire reference deployment already has a Wirebot Core **Phone Bridge / Context Core** prototype that periodically publishes phone state and derives operator context/location.

That existing implementation is useful as an adapter/proving ground, not as a new Focusa canonical state store.

Convergence law:

```text
Phone Bridge / native Companion sensors
→ Context Core owner-domain context
→ bounded Ambient Presence Projection
→ Focusa/Radar/Foreman when applicable
```

Do not expand the legacy `/signals/phone` path into a raw microphone/audio-upload endpoint. Conversation/audio uses the Spec-181/184 conversation sync path.

---

## 16. Legacy Wirebot voice adapter relationship

Existing Wirebot voice/`wbt` conversation storage may remain operational during migration.

Long-term convergence:

```text
legacy Wirebot voice turn
→ adapter
→ Focusa ConversationSession / Utterance / Expression refs
```

The legacy SQLite store becomes compatibility/history storage, not a competing canonical Conversation Ledger for Focusa-aware conversations.

Migration SHOULD preserve original IDs, timestamps, model/tool metadata and audio handles as provenance.

---

## 17. Mobile surface shape

The phone should not become a shrunken Focusa Desktop.

Recommended primary lenses:

```text
FOREMAN / TALK
INBOX / ATTENTION
ACTIVITY / HISTORY
```

with quick access to:

- Workstream selection;
- Wirebot/Foreman identity route;
- current Workpoint/Trajectory;
- active workers/Silent Sessions;
- Radar Signals/Episodes;
- approvals/secure prompts;
- Evidence/Receipts;
- conversation transcript/history;
- meeting capture;
- privacy/listening state;
- device/pairing/sync health.

Rich Mission Canvas remains available through mobile PWA/native view where appropriate.

---

## 18. Mobile platform lifecycle is a runtime fact

The architecture MUST represent mobile OS suspension/audio restrictions honestly.

Required states include:

```text
ambient_ready
wake_listening
active_conversation
meeting_recording
audio_interrupted
bluetooth_route_changed
background_restricted
suspended_by_platform
offline
syncing
recovering
```

A mobile app that the OS suspended is not `always_listening` merely because the product intends ambient availability.

Platform-specific implementation lives in adapters; these states remain portable.

---

## 19. Privacy/retention classes

Ambient data needs separate retention controls:

```text
PRESENCE_SIGNAL
    minimal / bounded

TRANSCRIPT
    searchable conversational provenance

RAW_AUDIO
    separate optional retention

PRETRIGGER_WAKE_BUFFER
    volatile discard by default

PRECISE_LOCATION
    owning-domain local / task-scoped by default

PROMOTED_SEMANTIC_STATE
    existing Focusa primitive retention rules
```

Deleting raw audio need not delete a user-approved transcript or promoted Workpoint; deleting one class follows its own lawful/owner policy and leaves an audit tombstone where required.

---

## 20. Operations

Initial operation families SHOULD include:

```text
ambient.session.start
ambient.session.stop
ambient.mode.get
ambient.mode.set
ambient.presence.publish
ambient.sync.push
ambient.sync.ack
ambient.device.status
ambient.identity.resolve
ambient.meeting.start
ambient.meeting.stop
ambient.note.capture
ambient.queue.status
ambient.queue.retry
ambient.privacy.view
```

Spec 181 retains ownership of conversation/utterance/expression operations. Spec 182 retains Foreman operations. Spec 183 retains Radar operations.

---

## 21. Implementation slices

These are acceptance slices, **not a second task tracker**. Create one `br` parent and materialize repo-specific tasks from these requirements when implementation begins.

### F184-S1 — Core Ambient contracts

- `AmbientSession`;
- `AmbientPresenceProjection`;
- `AmbientSyncEnvelope`;
- operations/reducer/event contracts;
- privacy/retention classifications.

### F184-S2 — Pairing/device identity

- reuse Spec 53 pairing;
- device key/identity;
- revocation;
- replay-safe sequence/idempotency;
- offline queue identity.

### F184-S3 — Conversation/meeting profile

- Spec-181 ConversationSession profile;
- meeting mode;
- speaker/diarization provenance;
- candidate extraction without automatic promotion;
- raw-audio/transcript retention separation.

### F184-S4 — Foreman/Wirebot identity router

- exact Workstream Foreman route;
- Chief-of-Staff route;
- ambiguous-scope clarification;
- stable principal attribution.

### F184-S5 — Radar attention channel

- high-value Signal/Episode projection;
- interruptibility-aware delivery;
- notice/ask-more/prepare/approve/deny paths;
- no generic alert flood.

### F184-S6 — Reference Phone Bridge adapter

- convert current Wirebot Context Core phone/location state into bounded projection;
- preserve exact source/freshness;
- keep raw GPS owned outside Focusa by default;
- do not add audio to `/signals/phone`.

### F184-S7 — Legacy voice convergence

- import/bridge `wbt` conversations into Spec-181 refs;
- preserve legacy provenance;
- no duplicate canonical conversation authority.

### F184-S8 — Native mobile Companion

- Android-first reference implementation because a real Phone Bridge already exists there;
- background lifecycle states;
- wake/PTT/conversation/meeting UX;
- Bluetooth audio route handling;
- encrypted offline queue;
- Tailscale/LAN sync;
- platform permission/recording indicators.

### F184-S9 — iOS parity

- native background audio/location rules;
- equivalent paired-device/sync/privacy behavior;
- no private Apple APIs;
- documented feature parity/degradation matrix.

### F184-S10 — Wearable/physical controls

- commodity earbud media-control mapping where safe;
- optional purpose-built hardware contract later;
- physical secure approval/stop only with trusted binding;
- device loss/revocation tests.

---

## 22. Acceptance invariants

Ambient Operator is valid only when:

1. phone/earbuds are surfaces/adapters, not canonical cognition;
2. `wake_word` can be available without retaining continuous pre-trigger room audio by default;
3. meeting transcript preserves speaker/ASR correction provenance;
4. conversation remains distinct from promoted Focusa state;
5. Context Core/location data reaches Focusa only through bounded privacy-aware projection;
6. exact GPS is not copied into project state by default;
7. offline mobile state is visibly stale and cannot fabricate remote success;
8. sync is replay-safe/idempotent and device-authenticated;
9. Bluetooth can be used for nearby control/proximity but is not the sole bulk-sync architecture;
10. Wirebot and Foreman routes preserve distinct scope/identity;
11. Radar interrupts only through bounded Signal/Episode attention policy;
12. legacy `wbt`/Phone Bridge can migrate without becoming second Focusa authorities;
13. mobile OS suspension/background restrictions are modeled honestly;
14. raw audio, transcript, presence, precise location and promoted semantic state have separate retention/privacy policies;
15. no architecture authority is granted by a spoken name, voiceprint, device, or ambient runtime.

---

## 23. Final principle

> **Ambient Operator makes Focusa continuously reachable in life. It does not make every moment canonical, every microphone always-recording, or every nearby device authoritative.**
