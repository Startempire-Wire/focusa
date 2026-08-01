REPLACEMENT FOR PREVIOUS HANDOFF: ADAPTIVE COMPOSITION, OCCUPANCY, AND NO-DEAD-CHROME LAW

The Mission Canvas is not a fixed dashboard with permanent panel slots.

It is a dynamically resolved professional workspace. Only contributions that
can provide meaningful, relevant and authorized content occupy the Canvas.

CORE LAW

- No dead chrome.
- No empty panels.
- No reserved geometry without meaningful content.
- No permanent “Unavailable,” “Degraded,” “Not configured,” or “No items”
  cards in the primary workspace.
- Missing content changes layout rather than producing placeholders.
- Internal capability and migration truth must be preserved without forcing it
  into the main operator interface.

WORKSPACE RESOLUTION

Every render must resolve:

workspace profile
+ activity mode
+ focused Work Surface
+ canonical read model
+ available operations
+ capabilities
+ permissions
+ viewport/device
+ project constraints
+ user profile preferences
→ ResolvedWorkspaceProjection

A Workspace Profile defines candidate contributions, priorities, adjacency,
renderer bindings, density and geometric preferences. It does not define a
rigid set of permanently visible boxes.

CONTRIBUTION ELIGIBILITY

Before layout, evaluate every candidate panel, inspector section, rail,
queue and control for:

- semantic relevance;
- applicable activity mode;
- meaningful content;
- operator authority;
- runtime capability;
- active Work Surface relationship;
- viewport suitability.

A contribution that cannot provide meaningful content is omitted before
geometry is calculated.

Do not first create a layout and then fill unresolved positions with empty
states.

EMPTY CONTRIBUTIONS

When a contribution has no content:

- remove it;
- remove its heading, border and reserved spacing;
- expand or promote remaining contributions;
- recompute a visually complete layout;
- retain creation through the toolbar, command palette, Work Surface add
  action, inspector action or prompt editor where appropriate.

Do not use large empty panels as feature discovery.

OPTIONAL CONTRIBUTIONS

Optional contributions with no useful content disappear entirely.

Their saved preference, position and configuration remain durable internally.
When content becomes available, the contribution may return in its preferred
region through deterministic layout resolution.

REQUIRED SEMANTICS

A semantic requirement does not imply an always-visible panel.

Required authority, safety, proof and compliance information appears when:

- relevant information exists;
- the state affects the current workflow;
- an available action depends on it;
- operator attention is required.

If an operation cannot be performed safely, remove or block that operation at
the point of action. Do not reserve an empty workspace panel to announce that
fact.

CAPABILITY ABSENCE

Do not expose dead workspace controls or dead profile choices.

If a capability is not present:

- omit its dependent contributions;
- omit actions that cannot work;
- reflow remaining content;
- preserve the diagnostic cause internally;
- explain it contextually only when the operator explicitly requests the
  capability or invokes a command that requires it.

The main workspace must not become a capability-error dashboard.

ACTIVE CAPABILITY LOSS

If a capability disappears during use:

- preserve already loaded content when safe;
- mark freshness only where it matters;
- remove dead controls;
- collapse contributions that can no longer provide value;
- reorganize the remaining workspace;
- preserve Session, Attachment, Workpoint, drafts and canonical state;
- restore the contribution automatically when the capability returns.

Use a bounded transient notification only when operator awareness is needed.
Do not leave a permanent empty placeholder.

INSPECTOR COMPOSITION

Inspector sections are conditional contributions.

Empty or irrelevant sections are removed. Remaining sections close their gaps
and reorder according to the active profile and activity mode.

Examples:

- no contention → omit Contention;
- no browser context → omit Browser Isolation;
- no writer lease → omit Writer Lease;
- no recovery event → omit Recovery Point;
- no Evidence items → omit the large Evidence section unless Evidence posture
  affects the current action.

WORK RAIL COMPOSITION

When populated, Work Rail may be surface-local, project aggregate or labeled
cross-project advisory.

When no meaningful entries exist:

- collapse the Work Rail region;
- expand the focused workspace or queue/composer area;
- preserve New Workpoint and related creation through contextual actions.

QUEUE COMPOSITION

When Steering and Follow-up both contain items:
- show them in the profile’s preferred arrangement.

When only one is populated:
- allow it to span the queue region.

When neither is populated:
- remove both queue regions;
- expand the Prompt Editor upward;
- retain compact queue-creation affordances near recipient routing.

WORK SURFACE STRIP

Show actual open, pinned and contextually important Work Surfaces.

Do not render empty tabs for every possible surface type.

The Add Surface action provides discovery and creation.

PROFILE SELECTOR

The primary Workspace selector displays profiles that can currently produce a
meaningful projection for the project.

Do not show dead options labeled unavailable, degraded or unsupported.

Profile installation, domain-pack setup and advanced configuration belong in
the explicit workspace-management flow, not the daily workspace selector.

LAYOUT RECOMPOSITION

The layout engine must support:

- deterministic contribution ranking;
- preferred adjacency;
- minimum and maximum spans;
- merge and tab rules;
- single-pane, split, stack, grid and inspector arrangements;
- viewport-aware recomposition;
- per-profile layout memory;
- smooth interruptible transitions;
- preservation of focused Work Surface and operator context.

A panel removal must result in a deliberate, balanced composition—not an
empty hole and not a random rearrangement.

NO SEMANTIC COUNTERFEITING

A missing contribution may not be replaced with a semantically different
panel merely to fill space.

The resolver fills space by expanding, promoting, merging or rearranging
remaining valid contributions.

It does not invent substitute data or reinterpret unknown semantic IDs.

INTERNAL DIAGNOSTICS

Omitted contributions may be recorded internally with reasons such as:

- no_relevant_content
- not_applicable
- capability_not_present
- not_authorized
- merged
- compacted
- suspended
- viewport_omitted

These reasons are available to diagnostics, history, Controls and migration
tools. They are not automatically rendered as operator-facing cards.

PROOF REQUIREMENTS

Prove all of the following:

1. An empty optional contribution leaves no visible panel, border, heading or
   unused geometry.
2. Removing a panel causes deterministic and visually balanced reflow.
3. A single populated queue spans the queue area.
4. Two empty queues collapse and the Prompt Editor expands.
5. An empty Work Rail collapses without losing New Workpoint capability.
6. Empty inspector sections disappear and remaining sections close gaps.
7. Work Surface tabs represent actual surfaces rather than possible types.
8. Switching profiles recomputes composition from candidate contributions.
9. Per-profile layout memory survives panel disappearance and return.
10. Capability loss does not produce a dashboard of unavailable cards.
11. Canonical state, session identity, drafts and focus survive recomposition.
12. UIAI Engine Eval proves transitions at supported viewport sizes.
13. Evidence and Receipts identify the resolved profile, activity mode,
    contribution set and layout revision tested.

FORBIDDEN

- fixed dashboard slots filled with empty-state cards;
- persistent “Unavailable” panels in the main workspace;
- persistent “Degraded workspace” banners;
- disabled feature controls with no immediate purpose;
- dead profile options in the everyday selector;
- blank queue lanes;
- blank Work Rail regions;
- blank inspector sections;
- semantically incorrect substitute panels;
- client-local ad hoc reflow logic;
- layouts that leave holes after contribution removal.
