# SignalOS Technical Scope

# SignalOS

**The living OS that transforms noise into knowing.**

## 1. Product Summary

SignalOS is a cloud + desktop AI workroom system that turns scattered digital work into guided, proof-backed workrooms with actions, exports, memory, and reusable lessons.

The product should feel **wondrous**, contained, calm, powerful, useful, and professional.

The emotional reference is “genie in a bottle,” but the execution must be clean, spacious, uncluttered, proof-backed, and useful in real work scenarios.

Users should experience one cohesive product, not separate engines, agents, browsers, plugins, tools, or technical frameworks.

Internally, SignalOS may use:

- Focusa
- UIAI Engine
- Pi
- model providers
- browser automation
- local runtimes
- MCP tools
- OpenAPI tools
- hosted services
- external skill repositories
- local-first models
- cloud models
- exported document pipelines

Externally, the user only experiences:

- SignalOS
- Summon Bar
- Vessel Workrooms
- Signal Cards
- Signal Inspector
- Proof
- Memory
- Process
- Talents
- Routines
- Signal Packs
- Actions
- Lessons
- Exports

---

## 2. Primary Domain Strategy

### Canonical product domain

```txt
https://signalos.pro
```

`signalos.pro` is the canonical home of the SignalOS product.

It should host:

- public marketing site
- cloud/browser app
- login/accounts
- billing/subscription later
- desktop downloads
- hosted Talent library
- hosted proof services
- hosted workrooms
- user-facing docs
- pricing
- product updates
- cloud parity experience
- future team/workspace features

### Secondary technical domain

```txt
https://signalos.focusa.dev
```

`signalos.focusa.dev` remains useful, but should not become a second product requiring separate maintenance.

It should be used for:

- staging
- beta testing
- private technical previews
- Focusa ecosystem bridge
- internal demos
- integration testing
- “powered by Focusa” proof surface
- fallback/testing URL

### Focusa root domain role

```txt
https://focusa.dev
```

`focusa.dev` remains the engine/platform/technical credibility brand.

It should be used for:

- Focusa engine
- developer/agentic infrastructure
- memory/prediction/proof layer
- technical documentation
- product ecosystem credibility
- internal architecture references

### Domain rule

SignalOS is the user-facing product.

Focusa is the engine/platform layer.

```txt
SignalOS = product
Focusa = engine/platform
signalos.pro = product home
signalos.focusa.dev = bridge/staging/labs
focusa.dev = technical ecosystem
```

---

## 3. Full Cloud Parity Requirement

SignalOS must be developed as both:

```txt
Cloud/browser app
Tauri desktop app
```

from the beginning.

The browser/cloud app at `signalos.pro` should be a first-class product, not merely a marketing site.

The desktop app should add local-first/private capabilities, but the cloud app must remain fully useful on its own.

### Shared core experience

Both cloud and desktop must support:

- Signal Rail
- Summon Bar
- Vessel Workrooms
- Signal Inspector
- Signal Objects
- standard object menus
- Signal Packs
- Routines
- Talents
- generated UI
- exports
- proof display
- memory display
- process display
- lessons
- safe action previews
- outcome capture

### Cloud/browser strengths

Cloud version should support:

- hosted workrooms
- hosted proof
- hosted Talent library
- cloud model calls
- account/license access
- saved workrooms
- browser-safe routines
- cloud exports
- team features later
- hosted scheduled routines later
- hosted CRM/email/calendar connectors later

### Desktop/Tauri strengths

Desktop version should support:

- everything browser supports
- local files
- local models
- local cache
- local proof cache
- native save dialogs
- private mode
- local runtime bridging
- better offline behavior
- local Talents where allowed
- local provider routing
- OS-level export/file workflows

### Product promise

```txt
SignalOS works in the cloud and on desktop.

The cloud app gives full access to the core SignalOS experience.

The desktop app adds local-first privacy, local models, local files, native exports, and deeper runtime control.
```

---

## 4. Core Product Rule

Users should never experience Focusa, UIAI Engine, or Pi as separate products.

Everything is wrapped into SignalOS.

### User-facing translation layer

| Internal Concept | User-Facing Concept |
|---|---|
| Focusa | Memory / Lessons / Predictions / Continuity |
| UIAI Engine | Proof / Sources / Evidence |
| Pi | Process / Routine execution |
| Tools / MCP / skills | Talents |
| Browser session | Source check / Proof capture |
| Workpoint | Active routine goal |
| Trajectory | Goal path |
| Metacognition | Lessons / What SignalOS learned |
| Prediction record | Signal prediction |
| Tool call | Process step |
| Agent | Routine operator |

Normal users should see SignalOS language.

Advanced users may inspect deeper internals through the **Raw** tab, but only after enabling Advanced mode.

---

## 5. Product Positioning

SignalOS is not a traditional chatbot, dashboard, CRM assistant, browser agent, or automation builder.

SignalOS is a **cloud + desktop AI workroom generator**.

The user gives intent.  
SignalOS creates the right workspace.  
SignalOS gathers proof.  
SignalOS recommends actions.  
The user approves, corrects, exports, or teaches.  
SignalOS remembers what worked.

### Simple positioning

```txt
SignalOS turns messy digital work into guided AI workrooms with proof, actions, memory, and learning.
```

### Stronger positioning

```txt
SignalOS is a wondrous AI cockpit where users summon routines, inspect proof, approve actions, export useful work products, and teach the system what worked.
```

### Category positioning

SignalOS sits near tools like Claude Desktop, Perplexity Comet, and agentic AI computers, but it should not be positioned as a clone of any of them.

```txt
Claude Desktop = chat + tools/connectors
Perplexity Comet = browser + AI
Perplexity Computer = cloud agent delegation
SignalOS = generated workrooms + proof + memory + exports + safe actions
```

---

## 6. Deployment Target

### Primary production domain

```txt
https://signalos.pro
```

### Secondary bridge/staging domain

```txt
https://signalos.focusa.dev
```

### VPS install root

```txt
/opt/signalos
```

### Recommended VPS structure

```txt
/opt/signalos
  /current
  /releases
  /shared
    /config
    /data
    /logs
    /talents
    /proof
    /memory
    /cache
    /exports
```

### Symlink pattern

```txt
/opt/signalos/current -> /opt/signalos/releases/2026-06-19-001
```

### Shared config

```txt
/opt/signalos/shared/config/signalos.env
```

### Shared data

```txt
/opt/signalos/shared/data
```

### Shared logs

```txt
/opt/signalos/shared/logs
```

### Shared Talents

```txt
/opt/signalos/shared/talents
```

### Shared exports

```txt
/opt/signalos/shared/exports
```

### System config paths

```txt
/etc/signalos/signalos.env
/var/lib/signalos
/var/log/signalos
```

### Suggested services

```txt
signalos-api.service
signalos-web.service
signalos-worker.service
signalos-uiai.service
signalos-focusa.service
signalos-pi-bridge.service
```

### Suggested local ports

```txt
SignalOS Web:       127.0.0.1:3440
SignalOS API:       127.0.0.1:3441
SignalOS Worker:    127.0.0.1:3442
Focusa Daemon:      127.0.0.1:8787
UIAI Engine:        127.0.0.1:8790
Pi Bridge:          127.0.0.1:8795
```

### Reverse proxy

Production:

```txt
signalos.pro
  → 127.0.0.1:3440
```

API:

```txt
signalos.pro/api/*
  → 127.0.0.1:3441
```

Bridge/staging:

```txt
signalos.focusa.dev
  → same deployment in beta/staging/labs mode
```

or later:

```txt
signalos.focusa.dev
  → redirect to signalos.pro
```

For MVP, both domains can point to the same app with environment-based mode flags.

---

## 7. Environment Variables

Example production env:

```env
SIGNALOS_ENV=production
SIGNALOS_PUBLIC_URL=https://signalos.pro
SIGNALOS_API_URL=https://signalos.pro/api
SIGNALOS_ALT_URL=https://signalos.focusa.dev

SIGNALOS_DATA_DIR=/opt/signalos/shared/data
SIGNALOS_LOG_DIR=/opt/signalos/shared/logs
SIGNALOS_TALENTS_DIR=/opt/signalos/shared/talents
SIGNALOS_PROOF_DIR=/opt/signalos/shared/proof
SIGNALOS_MEMORY_DIR=/opt/signalos/shared/memory
SIGNALOS_EXPORT_DIR=/opt/signalos/shared/exports

FOCUSA_DAEMON_URL=http://127.0.0.1:8787
UIAI_ENGINE_URL=http://127.0.0.1:8790
PI_BRIDGE_URL=http://127.0.0.1:8795

SIGNALOS_WEB_PORT=3440
SIGNALOS_API_PORT=3441
SIGNALOS_WORKER_PORT=3442

SIGNALOS_ENABLE_HOSTED_TALENTS=true
SIGNALOS_ENABLE_EXTERNAL_TALENT_IMPORT=false
SIGNALOS_ENABLE_ADVANCED_RAW=false

SIGNALOS_PRIMARY_DOMAIN=signalos.pro
SIGNALOS_SECONDARY_DOMAIN=signalos.focusa.dev
SIGNALOS_BRIDGE_DOMAIN_MODE=staging

OPENAI_API_KEY=
ANTHROPIC_API_KEY=
OPENROUTER_API_KEY=
OLLAMA_BASE_URL=http://127.0.0.1:11434
LMSTUDIO_BASE_URL=http://127.0.0.1:1234
```

---

## 8. High-Level Architecture

```txt
SignalOS
  ├─ Browser App
  │   ├─ public app at signalos.pro
  │   ├─ account/license layer
  │   ├─ hosted workrooms
  │   ├─ hosted Talents
  │   ├─ hosted proof
  │   ├─ hosted exports
  │   └─ browser-safe routines
  │
  ├─ Tauri Desktop App
  │   ├─ local-first experience
  │   ├─ local files
  │   ├─ local models
  │   ├─ local cache
  │   ├─ native export save dialogs
  │   ├─ local runtime bridging
  │   └─ private mode
  │
  ├─ Shared UI System
  │   ├─ Signal Rail
  │   ├─ Vessel Workroom
  │   ├─ Signal Inspector
  │   ├─ Summon Bar
  │   ├─ Vessel UI renderer
  │   ├─ Signal Object registry
  │   └─ standard object menus
  │
  ├─ Internal Engines
  │   ├─ Memory engine
  │   ├─ Proof engine
  │   ├─ Process engine
  │   ├─ Model router
  │   ├─ Mutation guard
  │   └─ Export engine
  │
  ├─ Talent System
  │   ├─ Talent registry
  │   ├─ Talent manifest format
  │   ├─ compatibility scanner
  │   ├─ adapter generator
  │   ├─ permission scanner
  │   ├─ runtime dispatcher
  │   └─ hosted library later
  │
  └─ Connectors
      ├─ web/source URLs
      ├─ local files
      ├─ local models
      ├─ cloud models
      ├─ CRM later
      ├─ email later
      ├─ calendar later
      └─ third-party APIs later
```

---

## 9. Browser and Tauri Parity

SignalOS should be developed as both:

```txt
Browser app
Tauri desktop app
```

from the beginning.

The goal is full browser parity and desktop superiority.

### Browser version

Target:

```txt
https://signalos.pro
```

Browser version should support:

- account login
- Signal Packs
- Summon Bar
- generated workrooms
- Talents library
- hosted proof
- hosted model calls
- saved workrooms
- exports
- browser-safe routines

### Tauri desktop version

Desktop version should support everything browser supports, plus:

- local files
- local models
- local storage
- local proof cache
- native save dialogs
- local provider routing
- local runtime bridging
- private mode
- better offline behavior
- local Talents where allowed

### Shared app architecture

Do not build two separate apps.

Build one shared app system with two shells.

```txt
/apps
  /web
  /desktop

/packages
  /app-shell
  /design-system
  /vessel-ui
  /schemas
  /talents
  /export-engine
  /runtime-bridge
  /model-router
  /mutation-guard
  /connectors
```

### Platform capability matrix

| Feature | Browser | Tauri |
|---|---:|---:|
| Summon Bar | Yes | Yes |
| Vessel Workrooms | Yes | Yes |
| Signal Inspector | Yes | Yes |
| Generated UI | Yes | Yes |
| Talents Library | Yes | Yes |
| Native Talents | Yes | Yes |
| Hosted Talents | Yes | Yes |
| Local Talents | Limited | Yes |
| Hosted proof | Yes | Yes |
| Local proof cache | Limited | Yes |
| File import | Limited | Yes |
| File export | Browser download | Native save dialog |
| Local models | Limited | Yes |
| Cloud models | Yes | Yes |
| CRM connectors later | Yes | Yes |
| Offline mode | Limited | Better |
| Scheduled background jobs | Hosted | Local or hosted |

### Platform declaration rule

Every feature must declare support:

```txt
web
tauri
both
hosted-only
desktop-only
```

Example:

```json
{
  "feature": "export_pdf",
  "platforms": ["web", "tauri"],
  "web_mode": "download",
  "tauri_mode": "save_dialog",
  "requires": ["export_engine"]
}
```

---

## 10. Locked Design Direction

SignalOS needs a strong design language from the top down.

The app should feel:

- grid-based
- clean
- spacious
- calm
- premium
- clear
- never cluttered
- professional with slight play
- wondrous but not childish
- subtle but contrasting
- high-clarity
- proof-oriented
- trustworthy

SignalOS should not feel like:

- a generic SaaS dashboard
- a chat app
- a dense admin console
- an IDE
- a CRM table
- a browser with an assistant
- a noisy automation tool

It should feel like a **cloud + desktop AI vessel** that creates the right workspace for the user’s task.

---

## 11. Locked UI Framework Decision

### Primary frontend stack

```txt
SvelteKit
Svelte 5
Tailwind CSS
shadcn-svelte
custom SignalOS design tokens
custom Vessel UI component layer
```

### Decision

Use **shadcn-svelte** as the base component system.

Do **not** use Semantic UI / Fomantic UI as the core framework.

Semantic/Fomantic may be studied for naming clarity, broad component patterns, and human-readable UI conventions, but SignalOS should not depend on it.

### Why shadcn-svelte

shadcn-svelte is the better fit because it is:

- Svelte-native
- Tailwind-compatible
- source-owned component code
- customizable
- design-token friendly
- compatible with generated UI architecture
- usable as a component registry foundation
- better suited for a Tauri + browser shared app

SignalOS should not look like default shadcn. It should use shadcn-svelte as a foundation and build a custom SignalOS visual language on top.

---

## 12. Design System Package

Create:

```txt
/packages/design-system
```

Purpose:

- tokens
- layout primitives
- typography
- spacing
- color hierarchy
- component variants
- motion rules
- shadow/glow rules
- export/report styling
- accessibility rules
- browser/Tauri visual parity

---

## 13. Design Principles

### 13.1 Grid-first

Everything should align to a clear grid.

Use:

```txt
12-column desktop grid
8px spacing base
16px / 24px / 32px section rhythm
consistent panel gutters
strict alignment for cards/tables
```

Generated UI must still feel composed.

### 13.2 Spacious by default

Whitespace is a product feature.

Avoid:

- dense stacking
- over-nested cards
- tiny text
- too many buttons
- noisy side panels
- raw data overload
- multi-color clutter

Every workroom should have:

- clear top summary
- clear sections
- strong hierarchy
- generous breathing room
- obvious next action
- proof available but not visually loud

### 13.3 Subtle contrast

Color should communicate state, not decorate randomly.

Use strong contrast for:

- selected object
- primary action
- warning
- blocked state
- proof highlight
- pending approval

Use subtle contrast for:

- metadata
- secondary cards
- inactive states
- background structure

### 13.4 Professional with slight play

The app must be usable by executives, founders, researchers, salespeople, and everyday power users.

The product can feel wondrous through transformation, motion, glass, glow, and the Vessel concept, but the content must stay practical and clear.

Good:

```txt
soft glow
glass panels
signal dots
animated vessel states
clean cards
crisp tables
quiet hover effects
```

Avoid:

```txt
cartoon genie
novelty animation
game-like interface
messy particles
overdone neon
purple SaaS blob overload
```

### 13.5 Landing page default

The public landing/default brand surface should be:

```txt
light
airy
clean
wondrous
mostly blank
editorial
sky-like
```

Reference feeling:

```txt
looking up at a beautiful clear sky on a bright day with a few small clouds
```

Landing-page rules:

```txt
white or near-white background
very pale sky tones
strong negative space
title + tagline + one supporting sentence by default
subtle alive feeling through restrained motion
no dashboard preview clutter by default
```

Motion rules for the default landing:

```txt
soft cloud drift
faint atmospheric wash
quiet pulse in sky blue when needed
very subtle depth
no heavy glow fields by default
```

Dark theme may exist later, but it is not the default public expression.
Soft glows belong to dark-theme or alternate-theme contexts, not the default landing.

---

## 14. Visual Language

### Base palette

SignalOS should use a professional palette system with a light default for public landing surfaces and optional dark surfaces for deeper product/app contexts.

#### Default public landing palette

```txt
Base background:
white / near-white / pale sky mist

Primary surface:
minimal or absent by default

Secondary surface:
very soft blue-gray or cloud-white

Text:
charcoal / soft black / cool dark gray

Primary accent:
sky blue

Secondary accent:
soft teal or restrained violet

Warning:
amber

Danger:
muted red

Success:
soft green

Proof / evidence accent:
blue-cyan

Memory / lesson accent:
violet

Action / approval accent:
teal or green
```

#### Optional dark product palette

```txt
Base background:
near-black / deep graphite

Primary surface:
charcoal glass

Secondary surface:
soft slate

Text:
warm white / cool gray

Primary accent:
cyan-blue or electric teal

Secondary accent:
soft violet

Warning:
amber

Danger:
muted red

Success:
soft green

Proof / evidence accent:
blue-cyan

Memory / lesson accent:
violet

Action / approval accent:
green or teal
```

### Color roles

```txt
Cyan / blue:
proof, source, evidence, verification

Violet:
memory, learning, lessons, routines

Teal / green:
approved action, safe completion, resolved state

Amber:
warning, uncertainty, needs review

Red:
blocked, unsafe, destructive, high-risk

Neutral gray:
metadata, secondary text, inactive content
```

### Rule

Every color must mean something.

No decorative color without semantic purpose.

---

## 15. Typography

Use a clean modern sans-serif.

Recommended:

```txt
Inter
Geist
IBM Plex Sans
```

Use a mono font only for:

```txt
Raw tab
object IDs
JSON
logs
technical metadata
short code-like values
```

Recommended mono fonts:

```txt
JetBrains Mono
Geist Mono
IBM Plex Mono
```

### Typography hierarchy

```txt
Page title:
large, calm, clear

Section heading:
medium, semibold

Card title:
small/medium, semibold

Metadata:
small, muted

Signal summary:
readable body text

Raw/debug:
mono, compact
```

---

## 16. Main User Interface

SignalOS uses four persistent regions.

```txt
┌──────────────┬───────────────────────────────┬──────────────────────────────┐
│ Signal Rail  │ Vessel / Signal Workroom      │ Signal Inspector / Lens      │
│              │                               │                              │
│ Today        │ Generated UI, Signal Cards,   │ Tabs:                        │
│ Packs        │ routines, tables, memos,      │ - Summary                    │
│ Routines     │ proof-backed outputs,         │ - Proof                      │
│ Actions      │ approval cards, decisions     │ - Memory                     │
│ Outcomes     │                               │ - Process                    │
│ Lessons      │                               │ - Metadata                   │
│ Talents      │                               │ - Raw                        │
│ Settings     │                               │                              │
├──────────────┴───────────────────────────────┴──────────────────────────────┤
│ Summon Bar: Ask, steer, approve, teach, or mutate…                          │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## 17. Signal Rail

The left-side navigation.

Sections:

```txt
Today
Packs
Routines
Actions
Outcomes
Lessons
Talents
Settings
```

The rail answers:

- Where am I?
- Which pack is active?
- Which routines are available?
- What needs attention?
- What actions are waiting?
- What has SignalOS learned?
- Which Talents are installed?

---

## 18. Vessel Workroom

The center canvas is the generated workspace.

A Workroom can be:

```txt
Comparison Workroom
Research Workroom
Decision Memo Workroom
Lead Rescue Workroom
Meeting Prep Workroom
Content Planning Workroom
Admin Triage Workroom
```

The Workroom should not feel like chat.

It should show:

```txt
SignalBrief
SignalCard
PredictionCard
ProofCard
ActionCard
LessonCard
ComparisonTable
RiskMatrix
SourceList
DecisionMemo
Checklist
ApprovalCard
OutcomeCaptureForm
RoutineStatus
```

---

## 19. Signal Inspector

The right-side deep-dive panel.

Tabs:

```txt
Summary
Proof
Memory
Process
Metadata
Raw
```

### Tab roles

| Tab | Purpose |
|---|---|
| Summary | Human explanation of selected item |
| Proof | Sources, screenshots, captures, evidence |
| Memory | Predictions, lessons, preferences, routine memory |
| Process | Current routine step, model/provider, tool flow |
| Metadata | Object fields, connector data, source details |
| Raw | JSON, payloads, internal refs, debug output |

### Behavior

- Collapsible by default.
- Pinnable for power users.
- Selection-driven.
- Context-aware.
- Auto-switches when useful.

Examples:

```txt
Click “Why?” → Summary or Memory
Click “Proof” → Proof
Click an action → Process
Click an object field → Metadata
Enable Advanced → Raw
```

---

## 20. Summon Bar

The bottom input replaces traditional chat.

The Summon Bar allows users to:

```txt
Ask
Act
Steer
Teach
Approve
Correct
Mutate
```

Examples:

```txt
Compare these three listings.
Show me what matters today.
Generate seller questions.
Weight maintenance records more heavily.
Remember that I care more about repairability than cosmetics.
That recommendation was wrong.
Save this as a decision memo.
```

### States

```txt
Idle:
What do you want SignalOS to handle?

Active routine:
Steer this routine…

Pending action:
Approve, edit, or cancel…

After result:
Tell SignalOS what happened so it can learn…
```

---

## 21. Vessel UI: Generated Interface System

SignalOS generates UI on the fly through safe schemas.

### Key rule

The model must not generate raw Svelte, HTML, JavaScript, or CSS.

The model generates structured JSON.

SignalOS validates JSON.

Trusted Svelte components render the UI.

### Flow

```txt
User intent
→ routine selection
→ Workroom schema generation
→ schema validation
→ trusted component render
→ user interaction
→ action approval
→ outcome capture
→ lesson stored
```

### Generated UI can choose

```txt
layout type
sections
component types
data bindings
actions
inspector defaults
empty states
```

### Generated UI cannot choose

```txt
random colors
random typography
arbitrary CSS
custom HTML
untrusted JS
unregistered components
unapproved interaction patterns
```

---

## 22. Workroom Schema Example

```json
{
  "kind": "signalos.workroom",
  "version": "1.0",
  "id": "wr_compare_001",
  "title": "Boat Listing Comparison",
  "pack": "deal_hunter",
  "routine": "compare_anything",
  "agent": "deal_scout",
  "layout": {
    "center": {
      "type": "stack",
      "sections": ["brief", "options", "risks", "proof", "recommendation", "questions"]
    },
    "inspector": {
      "tabs": ["summary", "proof", "memory", "process", "metadata", "raw"],
      "default": "summary"
    }
  },
  "sections": {
    "brief": {
      "type": "signal_brief",
      "title": "3 listings compared"
    },
    "options": {
      "type": "comparison_table",
      "title": "Options"
    },
    "risks": {
      "type": "risk_matrix",
      "title": "Red Flags"
    },
    "recommendation": {
      "type": "prediction_card",
      "title": "Safest option"
    }
  }
}
```

---

## 23. Component Registry

Initial trusted registry:

```txt
SignalBrief
SignalCard
PredictionCard
ProofCard
ActionCard
LessonCard
ComparisonTable
RiskMatrix
SourceList
DecisionMemo
Checklist
ApprovalCard
OutcomeCaptureForm
RoutineStatus
AgentStepTimeline
EvidenceDrawer
MetadataPanel
RawJsonPanel
TalentCard
ExportPanel
```

Every component must support:

```txt
selection
standard object menu
inspector routing
keyboard navigation
export serialization
responsive behavior
browser/Tauri parity
```

---

## 24. Signal Objects

Every visible UI item is a standardized Signal Object.

Examples:

```txt
Signal Card
Prediction Card
Proof Card
Action Card
Lesson Card
Table Row
Source
Memo Section
Routine
Talent
Outcome
```

### Core rule

```txt
Everything visible is selectable.
Everything selectable has a standard menu.
Every action maps to a typed command.
Every mutation requires preview and approval.
Every useful result can create proof, memory, outcome, or lesson.
```

### Signal Object schema

```json
{
  "id": "sig_123",
  "kind": "signalos.object",
  "type": "prediction_card",
  "title": "Option B appears safest",
  "summary": "Option B has fewer visible red flags but still needs service records.",
  "source_refs": ["source:listings:boat_b"],
  "evidence_refs": ["proof:uiai:789"],
  "memory_refs": ["memory:prediction:abc"],
  "process_refs": ["process:step:001"],
  "status": "active",
  "confidence": 0.74,
  "severity": "medium",
  "capabilities": [
    "inspect",
    "ask",
    "act",
    "prove",
    "teach",
    "keep",
    "reveal"
  ],
  "primary_actions": [
    "explain_why",
    "show_proof",
    "generate_questions"
  ],
  "default_inspector_tab": "summary",
  "metadata": {}
}
```

---

## 25. Standard Object Menus

Every Signal Object has the same interaction grammar.

Menu groups:

```txt
Inspect
Ask
Act
Prove
Teach
Keep
Reveal
```

### Inspect

```txt
Open in Lens
Show Summary
Show Metadata
Show Timeline
Show Raw Object
```

### Ask

```txt
Ask about this
Explain why
Compare with…
Find related items
What changed?
What should I do next?
```

### Act

```txt
Run recommended action
Create task
Draft message
Save memo
Add to routine
Re-run routine
Mark resolved
Dismiss
```

### Prove

```txt
Show evidence
Open source
Capture screenshot
Link evidence
Verify again
Add source note
```

### Teach

```txt
This was useful
This was wrong
Remember this preference
Capture outcome
Promote lesson
Do not use this source again
```

### Keep

```txt
Pin
Tag
Add to brief
Add to collection
Hide
Archive
Export
```

### Reveal

```txt
Show memory packet
Show proof packet
Show process trace
Show JSON
Copy object ID
```

Inline primary actions:

```txt
[Why?] [Proof] [Act] […]
```

---

## 26. Mutation Safety Model

Any action that changes something must follow this path:

```txt
Intent
→ proposed action
→ preview
→ user approval
→ execute
→ proof/result
→ outcome capture
```

### Mutation envelope

```json
{
  "kind": "signalos.mutation",
  "id": "mut_123",
  "label": "Create follow-up task",
  "target_refs": ["hubspot:deal:123"],
  "risk_level": "medium",
  "requires_approval": true,
  "preview": {
    "action": "Create task",
    "title": "Confirm procurement path",
    "due": "tomorrow 9:00 AM"
  },
  "approval_options": ["approve", "edit", "cancel"],
  "proof_required": true,
  "outcome_capture": true
}
```

### Risk levels

```txt
Safe read
Local write
External write
Communication send
Destructive action
```

### MVP mutation rule

MVP should only allow:

```txt
safe reads
local saves
exports
approved local outcome capture
approved lesson capture
```

Avoid external writes in MVP.

---

## 27. Talents System

SignalOS uses **Talents** as the user-facing word for skills, plugins, tools, and capabilities.

A Talent is a capability SignalOS can use inside a workroom.

A Talent may come from:

```txt
SignalOS native code
MCP server
OpenAPI spec
GitHub skill repo
SKILL.md-style folder
hosted library
local script
browser automation recipe
community marketplace package later
```

The user only sees:

```txt
SignalOS gained a new Talent.
```

They should not see raw MCP servers, plugins, scripts, or agent tools unless they enable Advanced mode.

---

## 28. Talent Library UI

Signal Rail section:

```txt
Talents
```

Talent Library sections:

```txt
Installed
Recommended
Compatible
Needs Setup
Experimental
Blocked
```

Talent card fields:

```txt
Name
Plain-language benefit
Supported Packs
Supported Routines
Permissions needed
Risk level
Setup status
Proof support
Memory support
Compatibility score
Install / Enable button
```

Example Talent card:

```txt
Research Web

Finds public sources, captures proof, and creates source-backed summaries.

Works with:
Research, Deal Hunter, Revenue, Creator

Needs:
Internet access

Risk:
Read-only

Status:
Ready
```

---

## 29. Talent Manifest

Every Talent normalizes into a SignalOS Talent Manifest.

```json
{
  "kind": "signalos.talent",
  "version": "1.0",
  "id": "talent.research.web",
  "name": "Research Web",
  "description": "Finds and summarizes public sources with proof.",
  "source": {
    "type": "native",
    "origin": "signalos"
  },
  "capabilities": [
    "research",
    "source_capture",
    "summary",
    "evidence"
  ],
  "inputs": {
    "type": "object",
    "properties": {
      "query": {
        "type": "string"
      },
      "depth": {
        "type": "string",
        "enum": ["quick", "standard", "deep"]
      }
    },
    "required": ["query"]
  },
  "outputs": {
    "signal_objects": ["source", "proof_card", "summary_card"],
    "evidence_refs": true,
    "workroom_sections": ["source_list", "brief", "proof"]
  },
  "permissions": {
    "network": true,
    "browser": true,
    "files": false,
    "external_write": false,
    "shell": false,
    "credentials": false
  },
  "ui": {
    "preferred_components": ["SourceList", "ProofCard", "DecisionMemo"],
    "fallback": "generic_result_cards"
  },
  "safety": {
    "risk_level": "read_only",
    "requires_approval": false,
    "sandbox_required": false
  },
  "compatibility": {
    "status": "ready",
    "score": 0.92
  }
}
```

---

## 30. Talent Compatibility Scanner

SignalOS needs a compatibility scanner before enabling external Talents.

Scanner statuses:

```txt
Ready
Needs Setup
Adapter Needed
Limited
Unsafe
Unsupported
```

### Ready

Talent works with the current object model, UI renderer, permissions, and routines.

### Needs Setup

Requires API key, login, folder selection, local binary, or provider config.

### Adapter Needed

Useful, but outputs do not map cleanly to SignalOS UI or objects yet.

### Limited

Works only in specific packs, with specific providers, or read-only mode.

### Unsafe

Requests risky permissions, suspicious prompts, shell access, credential access, payment actions, or broad external mutation.

### Unsupported

Cannot run in current app/runtime.

---

## 31. Compatibility Scan Categories

Each Talent should be scanned for:

```txt
Capability fit
Data contract fit
UI fit
Permission fit
Runtime fit
Model fit
Security fit
Proof fit
Learning fit
Pack fit
Routine fit
```

### Capability fit

```txt
Research
Compare
Extract
Summarize
Draft
Transform
Watch
Mutate
Export
```

### Data contract fit

```txt
Cards
Tables
Proof items
Actions
Lessons
Outcomes
Memos
Forms
```

### UI fit

```txt
Card
Table
Timeline
Checklist
Memo
Form
Evidence drawer
Raw fallback
```

### Permission fit

```txt
Network
Browser
Files
External write
Shell
Credentials
Payments
Local model
Cloud model
```

### Runtime fit

```txt
Node
Python
Go
Docker
Local binary
Browser
Remote API
GPU
```

### Security fit

```txt
unknown publisher
unsigned package
prompt injection risk
hidden instructions
excessive permissions
credential exfiltration risk
shell execution
network exfiltration
untrusted update source
```

### Proof fit

```txt
source URL
screenshot
text extraction
artifact handle
file ref
trace
verification result
```

### Learning fit

```txt
prediction possible
outcome capturable
lesson promotable
routine improvable
```

---

## 32. Talent Execution Modes

Every Talent runs under one mode:

```txt
Native
Adapter
Sandbox
Hosted
Manual
Blocked
```

### Native

Built into SignalOS.

### Adapter

External skill wrapped through a SignalOS manifest.

### Sandbox

Runs in a contained local or remote runtime.

### Hosted

Runs on SignalOS cloud/VPS.

### Manual

SignalOS can guide the user, but execution is not automated yet.

### Blocked

Unsafe or incompatible.

---

## 33. Talent Adapter System

When a Talent is close but incompatible, SignalOS should propose an adapter.

Example:

```txt
This Talent can extract product specs, but SignalOS does not know how to map its output yet.

Suggested adapter:
- Map `items[]` to ComparisonTable rows
- Map `source_url` to Proof source
- Map `warnings[]` to Risk Cards
- Map `recommendation` to Signal Card

[Create Adapter] [Edit Mapping] [Skip]
```

Adapters should be mappings, not arbitrary generated code.

Adapter maps:

```txt
external input schema → SignalOS input schema
external output schema → Signal Objects
external result fields → Workroom components
external errors → Process messages
external evidence → Proof refs
```

---

## 34. Talent Sources

MVP should support curated sources only.

Initial sources:

```txt
SignalOS native Talents
curated local Talent folder
curated hosted Talent library
curated MCP server definitions
curated OpenAPI specs later
```

Future sources:

```txt
GitHub skill repos
SKILL.md folders
public MCP registries
community marketplace
team/private Talent libraries
paid premium Talents
```

---

## 35. Talent Storage

### VPS Talent storage

```txt
/opt/signalos/shared/talents
  /installed
  /curated
  /blocked
  /manifests
  /adapters
  /scan-reports
```

### Desktop local Talent storage

```txt
~/.signalos/talents
  /installed
  /manifests
  /adapters
  /scan-reports
  /cache
```

---

## 36. Talent Install Flow

User flow:

```txt
Open Talent Library
→ Browse Recommended Talents
→ Click Talent
→ SignalOS scans compatibility
→ SignalOS shows permissions and risk
→ User enables Talent
→ Talent appears in compatible Routines and menus
```

External import flow:

```txt
Import Talent
→ paste URL or choose local folder
→ scanner reads manifest/spec/docs
→ scanner creates compatibility report
→ if compatible, create SignalOS Talent Manifest
→ if partial, suggest adapter
→ if unsafe, block by default
```

---

## 37. Native MVP Talents

Initial native Talents:

```txt
Research Web
Summarize Source
Compare Options
Find Red Flags
Generate Questions
Create Decision Memo
Capture Outcome
Remember Lesson
Show Proof
Explain Why
Export Workroom
```

---

## 38. Signal Packs

Signal Packs are bundled work modes.

Each pack contains:

```txt
routines
Talents
default UI templates
agent presets
model policy
proof rules
memory rules
export presets
```

### MVP Signal Packs

Only start with:

```txt
Research Pack
Deal Hunter Pack
```

### Research Pack

Promise:

```txt
Turn messy research into proof-backed briefs.
```

Initial routines:

```txt
Compare Anything
Deep Research Brief later
Source Proof Pack later
```

### Deal Hunter Pack

Promise:

```txt
Compare expensive decisions, find red flags, and ask better questions.
```

Initial routines:

```txt
Compare Anything
Listing Analyzer later
Seller Questions later
```

### Future Signal Packs

```txt
Revenue Pack
Creator Pack
Life Admin Pack
Career Pack
Founder Pack
Agency Pack
Real Estate Pack
Boat Buying Pack
Grant Writing Pack
Legal Prep Pack
```

---

## 39. MVP Routine: Compare Anything

The first MVP routine should be:

```txt
Compare Anything
```

Supported by:

```txt
Research Web
Summarize Source
Compare Options
Find Red Flags
Generate Questions
Create Decision Memo
Capture Outcome
Remember Lesson
Export Workroom
```

Input types:

```txt
URLs
pasted notes
manual option entries
text snippets
files later
```

Output:

```txt
Comparison table
Pros/cons
Risk flags
Proof sources
Recommendation
Questions to ask
Decision memo
Outcome capture
Lesson
Exports
```

Use cases:

```txt
boat listings
car listings
software tools
vendors
services
job opportunities
products
travel options
local businesses
```

---

## 40. Model Provider Layer

SignalOS should support three user-facing AI modes.

### Private

```txt
Use local models when possible.
Ask before cloud use.
Best for sensitive work.
```

### Balanced

```txt
Use local models for simple/private tasks.
Use cloud models for hard reasoning.
Default recommended mode.
```

### Power

```txt
Use the strongest available model when quality matters.
Best for complex research and polished reports.
```

### Supported provider targets

```txt
Ollama
LM Studio
llama.cpp server
OpenAI
Anthropic
OpenRouter
Gemini later
Groq / Fireworks later
Custom OpenAI-compatible endpoint later
```

### Provider display rule

Users should not be overwhelmed by model details.

Normal users see:

```txt
Private
Balanced
Power
```

Power users can open provider details in Settings or Process tab.

---

## 41. Portability and Export Layer

SignalOS must treat every workroom as portable data.

Users should be able to turn SignalOS output into useful work formats.

Create:

```txt
/packages/export-engine
```

Purpose:

```txt
serialize workrooms
export reports
export tables
export evidence
compile memos
export proof bundles
preserve metadata
support business workflows
```

### Data portability rule

User data should never be trapped.

Every major object must be exportable:

```txt
Workroom
Signal Card
Prediction
Proof item
Lesson
Routine
Talent manifest
Comparison table
Decision memo
Source list
Outcome
```

---

## 42. Export Formats

### MVP export formats

```txt
Markdown
JSON
CSV
PDF
```

### Post-MVP export formats

```txt
XLSX
DOCX
HTML
ZIP proof bundle
PNG screenshot
clipboard formats
```

### Business-focused exports

```txt
Decision memo PDF
Comparison spreadsheet
Source/proof appendix
Red flag report
Seller/vendor questions
Executive brief
Research packet
CRM-ready summary later
```

---

## 43. Export Types

### Workroom Export

Exports the full generated workroom.

Includes:

```txt
title
pack
routine
date
inputs
outputs
cards
tables
recommendation
actions
proof refs
memory refs
metadata
```

Formats:

```txt
JSON
Markdown
PDF
HTML later
```

### Decision Memo

Polished human-readable report.

Includes:

```txt
summary
recommendation
comparison
risks
proof
open questions
next actions
```

Formats:

```txt
PDF
Markdown
DOCX later
```

### Spreadsheet Export

For tables and comparisons.

Includes:

```txt
option table
scoring
risk matrix
source links
notes
questions
```

Formats:

```txt
CSV
XLSX
```

### Proof Bundle

Exports source evidence.

Includes:

```txt
source URLs
screenshots
extracted text
timestamps
proof notes
evidence refs
summary
```

Formats:

```txt
ZIP
Markdown
PDF appendix
JSON
```

### Raw Archive

For power users.

Includes:

```txt
workroom JSON
Signal Objects
Talent outputs
proof refs
memory refs
process trace
metadata
```

Formats:

```txt
JSON
ZIP
```

---

## 44. Export Technical Choices

### PDF

Use two PDF paths:

```txt
1. HTML-to-PDF for polished reports
2. Programmatic PDF generation for structured/simple documents
```

Recommended:

```txt
Playwright for high-fidelity HTML-to-PDF
pdf-lib for programmatic PDF creation/modification
```

### CSV

Use native CSV generation.

Rules:

```txt
escape values properly
preserve headers
support UTF-8
include metadata rows optionally
```

### XLSX

Recommended candidates:

```txt
ExcelJS
SheetJS
```

Initial choice:

```txt
ExcelJS for styled workbook exports
CSV as first MVP fallback
```

SheetJS can be evaluated if broader spreadsheet import/export support becomes necessary.

### Markdown

Use SignalOS internal report serializer.

Markdown should be clean enough to paste into:

```txt
Notion
Google Docs
GitHub
email
AI tools
project docs
```

### JSON

Use canonical SignalOS schemas.

JSON should be useful for:

```txt
backup
debug
migration
import
automation
future sync
```

---

## 45. Export UX

Every workroom should have an export button.

Placement:

```txt
top-right of Workroom
standard object menu → Keep / Export
Signal Inspector → Metadata / Raw export
```

Export modal:

```txt
Export this workroom

Formats:
[PDF] [Markdown] [CSV] [JSON]

Include:
[x] Recommendation
[x] Comparison table
[x] Red flags
[x] Proof sources
[x] Questions
[x] Metadata
[ ] Raw JSON
```

Advanced users can save export presets:

```txt
Decision Memo preset
Spreadsheet preset
Proof Bundle preset
Executive Brief preset
Raw Archive preset
```

### MVP export minimum

For Compare Anything:

```txt
Export comparison as CSV
Export decision memo as Markdown
Export decision memo as PDF
Export raw workroom as JSON
```

---

## 46. SaaS Elements

SignalOS should be cloud + desktop, with `signalos.pro` as the canonical SaaS home.

SaaS elements:

```txt
account/license activation
subscription status
cloud model credits
hosted proof sessions
hosted Talent library
desktop update metadata
premium Signal Packs
premium Talents
cloud memory sync later
team workspaces later
CRM connectors later
scheduled routines later
hosted agent runtime later
```

MVP SaaS elements:

```txt
signalos.pro landing/app page
cloud/browser app
account/license placeholder
desktop download/update page
hosted Talent manifest library
optional hosted proof endpoint
basic cloud config
```

### Domain-specific SaaS roles

```txt
signalos.pro
  canonical product home
  cloud app
  login/account
  hosted Talents
  desktop downloads
  user-facing docs
  future billing/pricing

signalos.focusa.dev
  technical preview
  beta/staging
  Focusa bridge
  internal demo
  integration testing

focusa.dev
  engine/platform home
  developer/technical credibility
```

---

## 47. Suggested MVP Repo Structure

```txt
signalos
  /apps
    /desktop
    /web
    /api
  /packages
    /app-shell
    /core
    /design-system
    /vessel-ui
    /schemas
    /talents
    /connectors
    /model-router
    /mutation-guard
    /export-engine
    /runtime-bridge
  /routines
    /compare-anything
  /packs
    /research
    /deal-hunter
  /talents
    /native
      /research-web
      /summarize-source
      /compare-options
      /find-red-flags
      /generate-questions
      /create-decision-memo
      /capture-outcome
      /remember-lesson
      /show-proof
      /explain-why
      /export-workroom
  /docs
    TECHNICAL_SCOPE.md
    MVP_PLAN.md
    DESIGN_SYSTEM.md
    TALENTS_SPEC.md
    VESSEL_UI_SPEC.md
    SIGNAL_OBJECT_SPEC.md
    EXPORT_ENGINE_SPEC.md
    BROWSER_TAURI_PARITY.md
    DOMAIN_STRATEGY.md
  /deploy
    /vps
      install.sh
      signalos-api.service
      signalos-web.service
      signalos-worker.service
      nginx.conf.example
```

---

## 48. Security Rules

MVP security defaults:

```txt
external Talent imports disabled by default
read-only Talents only
no arbitrary shell execution
no external write actions
no credential access without explicit setup
no sending messages/emails
no CRM writeback
no hidden browser actions
all mutations require preview and approval
all Talents require compatibility scan
all generated UI must validate against schema
all raw output hidden behind Advanced mode
all exports must be user-triggered
```

---

## 49. MVP Scope

### Build in MVP

```txt
Browser app at signalos.pro
Tauri desktop app
shared UI packages
SvelteKit + Svelte 5 + Tailwind + shadcn-svelte
custom SignalOS design system
Summon Bar
Signal Rail
Vessel Workroom renderer
Signal Inspector
Signal Object model
standard object menus
Talent Manifest format
Talent Registry
native Talents
basic compatibility scanner
Compare Anything routine
Research Pack
Deal Hunter Pack
local/cloud provider setting
basic proof capture
basic memory/lesson capture
Markdown export
JSON export
CSV export
PDF export path
platform capability matrix
domain strategy
signalos.focusa.dev staging/bridge path
```

### Do not build in MVP

```txt
CRM connectors
email sending
calendar mutation
public Talent marketplace
team accounts
paid Talent marketplace
arbitrary GitHub imports
arbitrary shell execution
complex hosted sandboxing
full automation scheduling
external writes
mobile app
plugin marketplace
many Signal Packs
large agent library
full local model manager
complex billing
```

---

## 50. Updated MVP Definition

SignalOS MVP is:

```txt
A browser + Tauri desktop app that lets a user summon a generated Compare Anything workroom, inspect proof, use standardized object actions, capture lessons, and export the result into practical work formats.
```

MVP must prove:

```txt
1. The UI feels wondrous but clean.
2. Generated workrooms are structured and not cluttered.
3. The same app works in browser and desktop.
4. signalos.pro is a real cloud app, not just a landing page.
5. signalos.focusa.dev remains useful as a beta/staging/Focusa bridge.
6. Outputs can leave the app as PDF, Markdown, CSV, and JSON.
7. Talents can power workrooms without exposing internal complexity.
8. Every visible item behaves consistently through Signal Object menus.
9. Users can teach SignalOS and see that lesson affect later work.
```

---

## 51. MVP Acceptance Criteria

The MVP is successful if a user can:

```txt
Open signalos.pro
Install/open SignalOS desktop
Choose Research or Deal Hunter
Type intent into the Summon Bar
Generate a Compare Anything Workroom
Add URLs or pasted notes
See comparison table, risks, proof, recommendation, and questions
Click any item and open the Signal Inspector
Use standard object menus
See Summary, Proof, Memory, Process, Metadata, and Raw tabs
Use native Talents inside the routine
Capture an outcome
Teach SignalOS a lesson
Export comparison as CSV
Export memo as Markdown
Export memo as PDF
Export raw workroom as JSON
Return later and see that lesson inform another comparison
```

---

## 52. First Demo Script

User opens:

```txt
https://signalos.pro
```

or the Tauri desktop app.

The Vessel glows softly.

User types:

```txt
Compare these three boat listings and tell me which one is safest.
```

SignalOS creates a Deal Hunter workroom.

The center shows:

```txt
comparison table
risk matrix
red flag cards
proof sources
seller questions
recommendation card
decision memo draft
export options
```

The right Signal Inspector shows:

```txt
Summary: why Option B looks safest
Proof: captured listing sources
Memory: prediction and remembered preferences
Process: routine steps
Metadata: listing details and criteria
Raw: generated JSON and refs
```

User clicks a red flag.

The standard menu appears:

```txt
Inspect
Ask
Act
Prove
Teach
Keep
Reveal
```

User chooses:

```txt
Prove → Show evidence
```

SignalOS opens the Proof tab with source evidence.

User types:

```txt
Remember that maintenance records matter more than cosmetic upgrades.
```

SignalOS captures a lesson.

User clicks:

```txt
Generate seller questions
```

SignalOS creates seller questions and adds them to the decision memo.

User exports:

```txt
Decision Memo PDF
Comparison CSV
Raw JSON
```

---

## 53. Post-MVP Roadmap

### Phase 2: Research Pack expansion

```txt
Deep Research Brief
Source Proof Pack
Company Brief
Claim Checker
saved research collections
Markdown/PDF proof reports
```

### Phase 3: Revenue Pack

```txt
CRM selector onboarding
GoHighLevel mode
HubSpot mode
Salesforce mode
normalized RevenueObject model
Lead Rescue
Deal Intelligence
Forecast Command
```

### Phase 4: Life Admin Pack

```txt
Gmail/Outlook read-only
Calendar read-only
document/folder watch
admin brief
renewal/bill detection
appointment prep
```

### Phase 5: Creator Pack

```txt
source-to-content workflow
newsletter/script/post routines
source proof drawer
repurpose workrooms
```

### Phase 6: Talent marketplace

```txt
curated hosted Talents
signed Talents
paid Talents
team-approved private Talent libraries
community submissions
compatibility grades
security scan reports
```

### Phase 7: Team / Pro features

```txt
shared workrooms
shared lessons
shared proof library
team routines
admin controls
billing
organization model policies
CRM/team connector access
scheduled routines
```

---

## 54. Final Product Summary

SignalOS is a browser + desktop AI workroom system.

It wraps internal engines into one cohesive user-facing product.

It uses:

```txt
signalos.pro
Summon Bar
Vessel Workrooms
Signal Inspector
Signal Objects
Talents
Routines
Signal Packs
Proof
Memory
Process
Lessons
Safe Actions
Exports
Browser/Tauri parity
Cloud/Desktop parity
```

The MVP should focus on:

```txt
Compare Anything
Research Pack
Deal Hunter Pack
Native Talents
Generated Workroom UI
Proof-backed recommendations
Outcome learning
PDF / Markdown / CSV / JSON exports
Browser + Tauri parity
signalos.pro as canonical product home
signalos.focusa.dev as bridge/staging path
```

The long-term platform becomes:

```txt
A wondrous, extensible AI workroom system that can absorb external capabilities, scan compatibility, wrap them as Talents, generate the right UI on demand, and help users solve real digital headaches with proof, memory, action, and portable outputs.
```
