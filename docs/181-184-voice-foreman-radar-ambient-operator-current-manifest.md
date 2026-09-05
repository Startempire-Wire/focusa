# Focusa 181–184 Current Manifest — Voice, Foreman, Radar, Ambient Operator

**Status:** current architecture map, 2026-09-05.  
**Canonical human architecture authority:** Verious Smith III.  
**Purpose:** make the current ownership/dependency model discoverable without reusing historical proposal numbering.

## Current canonical specs

| Spec | Current owner | One-line purpose |
|---|---|---|
| [181](181-focusa-voice-conversation-expression-and-auditable-interaction-spec.md) | Focusa Conversation / Expression | auditable full-duplex voice, utterances, speakers, transcript lineage, Conversation Ledger |
| [182](182-focusa-project-foreman-workstream-intelligence-projection-spec.md) | Project Foreman | persistent Workstream-scoped project-intelligence role projection across models/surfaces |
| [183](183-focusa-radar-proactive-observation-episodes-signal-economics-and-attention-routing-spec.md) | Radar | proactive scoped observations, Episodes, Signals, attention economics and Foreman routing |
| [184](184-focusa-ambient-operator-mobile-wearable-presence-meeting-and-sync-spec.md) | Ambient Operator | mobile/wearable presence, meetings, identity routing, offline/private sync and earbuds/phone projection |

## Historical proposal reconciliation

An operator-supplied cross-account architecture handoff recovered earlier design history:

- `Focusa Radar` was remembered as reaching a **Spec 164 proposal**;
- Ambient Voice/Earbuds was remembered as **Proposal 135M**;
- `Project Foreman` was strongly developed but apparently lacked one definitive standalone spec.

Current repository truth has moved:

```text
Current Spec 164
    = Workstream-Rooted Canonical Runtime

Current Spec 135 series
    = frozen at 135A–135K

Therefore:
    historical Radar 164 == proposal provenance only
    historical 135M == proposal provenance only
```

The current canonical numbers are 182 Foreman, 183 Radar and 184 Ambient Operator.

## Primitive ownership

```text
Spec 164 Workstream Root
        |
        +--> Spec 182 Foreman
        |       persistent project-responsible agent role
        |
        +--> Spec 183 Radar
                proactive scoped attention

Spec 181 Conversation
        |
        +--> Spec 184 Ambient Operator
                mobile/wearable/meeting projection

Spec 139 Presence/Placement
        +--> Radar consumes operational reality
        +--> Ambient consumes/publishes bounded presence projections

Spec 135/135B C.R.I.S.T.
        +--> forms/refines project understanding used by Foreman

Spec 133 + Spec 79
        +--> Foreman supervises governed workers/work loops

Spec 136
        +--> consequential effects settle through normal receipts/outcome truth
```

## Foreman / Radar / Ambient distinction

```text
RADAR
    notices, deduplicates, groups Episodes, scores attention

FOREMAN
    understands one Workstream, investigates, prepares, delegates, executes under grant

FOCUSA CORE
    governs scope, authority, Workpoint, Evidence, Receipts and canonical state

AMBIENT OPERATOR
    lets the human carry that same governed system through phone/earbuds/wearables

WIREBOT / CHIEF OF STAFF
    may reason across life/business/many Workstreams and delegate to exact Foremen
```

Wirebot is not a global Foreman and Foreman is not another Wirebot.

## Conversation / meeting truth

The system may preserve exhaustive conversational provenance while maintaining:

```text
conversation != memory
voice != authority
transcript != Workpoint
speaker label != principal authorization
```

A meeting statement can become a typed candidate, then pass through the existing governance/promotion path.

## Mobile/context boundary

For the Startempire reference deployment:

```text
Phone / sensors
    ↓
Wirebot Context Core
    raw owner-domain context such as phone/location/activity/interruptibility
    ↓
bounded Ambient Presence Projection
    ↓
Focusa Radar / Foreman when applicable
```

Raw precise location and ambient sensor data do not become Workstream cognition by default.

## Veragensia / UIAI integration

The implementation companions are:

- Veragensia [Doc 197](https://github.com/Startempire-Wire/veragensia/blob/main/docs/197-veragensia-voice-native-agent-computer-audio-ui-and-conversation-continuity-spec.md) — voice-complete Agent Computer;
- Veragensia [Doc 199](https://github.com/Startempire-Wire/veragensia/blob/main/docs/199-veragensia-ambient-operator-companion-sync-and-omarchy-integration-spec.md) — Linux/Omarchy, trusted audio, Companion sync and mobile integration;
- UIAI `UIAI_VERAGENSIA_COMPUTER_CONTROL_AND_VOICE_BINDING_2026-09-04.md` — browser/computer execution binding.

Focusa does not move browser/computer execution into Radar or Ambient Operator.

## Implementation direction

Each owning spec includes stable implementation/acceptance slices. Those slices are **not task status**. Before code implementation, materialize them into the repository-local `br` graph with exact dependencies and done conditions.

Do not create GitHub issues or another Markdown backlog merely to mirror those slices.
