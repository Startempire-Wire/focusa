#!/usr/bin/env python3
"""Generate the authoritative Spec 135 Mission Canvas completion pivot DAG.

This generator deliberately separates normative planning from implementation. It
encodes the dependency order required by the current adaptive-composition
handoff, the Spec 135 host/renderer contract, and the existing repository state.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
GRAPH_PATH = ROOT / "docs/contracts/spec135-mission-canvas-completion-dag.v2.json"
REPORT_PATH = ROOT / "docs/agent/spec135-mission-canvas-completion-pivot-plan.md"

AUTHORITY = [
    "operator steering in the current Pi session",
    "docs/contracts/spec135/authoritative-handoff/spec135_agent_handoff_apple_principles.md",
    "docs/contracts/spec135/authoritative-handoff/focusa_activity_mode_recomposition.png",
    "docs/contracts/spec135/authoritative-handoff/focusa_dynamic_vertical_recomposition.png",
    "docs/contracts/spec135-mission-canvas-host-renderer-contract.v1.yaml",
    "docs/135-series-current-manifest.md",
    "docs/agent/spec135-implementation-acceleration-directive.md",
    "Spec 135 through 135K normative sources referenced by the current manifest",
]

STATE_FINDINGS = [
    {
        "id": "STATE-001",
        "classification": "authority_current",
        "finding": "The adaptive-composition replacement and two recomposition images are preserved as current authority; the prior fixed-layout handoff and image are superseded history.",
        "evidence": ["commit:883cf670", "commit:060d7909"],
    },
    {
        "id": "STATE-002",
        "classification": "closure_reconciled",
        "finding": "The legacy 73-requirement ledger is reconciled in place, the Svelte-hosted amendment is separate, and master acceptance remains evidence-derived rather than file-existence-derived.",
        "evidence": [
            "docs/contracts/spec135-complete-feature-ledger.v1.yaml",
            "docs/contracts/spec135-delivery-dag.v1.yaml",
            "docs/contracts/spec135-master-final-acceptance.v1.json",
        ],
    },
    {
        "id": "STATE-003",
        "classification": "desktop_svelte_renderer_partial",
        "finding": "ResolvedWorkspaceProjection is implemented in Core/API and the recursive Svelte renderer foundation is active in apps/desktop, but the complete Pi-overlay capability inventory is not yet translated.",
        "evidence": ["apps/desktop/src/lib/mission-canvas/", "crates/focusa-core/src/mission_canvas/"],
    },
    {
        "id": "STATE-004",
        "classification": "pi_overlay_source_inventory",
        "finding": "The Pi extension contains source models and behaviors that remain mandatory translation input; it is not the production Mission Canvas GUI destination.",
        "evidence": ["apps/pi-extension/src/mission-canvas-model.ts", "apps/pi-extension/src/mission-canvas-widget.ts", "apps/pi-extension/src/work-rail-widget.ts", "apps/pi-extension/src/mission-canvas-session-inventory.ts"],
    },
    {
        "id": "STATE-005",
        "classification": "reusable_core_foundation",
        "finding": "Focusa Core and API already expose durable Mission Canvas Work Surface, state, binding, Work Rail, event-stream, operation-registry, OpenAPI, generated-client, A2UI/Lit, and Focusa element foundations.",
        "evidence": [
            "crates/focusa-core/src/types.rs",
            "crates/focusa-api/src/routes/mission_canvas_surfaces.rs",
            "docs/contracts/spec135/generated-contract-v1/operation-registry.json",
            "packages/a2ui-renderer/",
            "packages/focusa-elements/",
        ],
    },
    {
        "id": "STATE-006",
        "classification": "layout_foundation_implemented",
        "finding": "Core owns contribution resolution, eligibility, deterministic layout, persistence, profile memory, and omission diagnostics; the Svelte Desktop host recursively renders the bounded projection.",
        "evidence": ["crates/focusa-core/src/types.rs:MissionCanvasStateRecord", "apps/desktop/src/lib/mission-canvas/ProjectionLayoutRenderer.svelte"],
    },
    {
        "id": "STATE-007",
        "classification": "generated_ui_bound",
        "finding": "A2UI/Lit and Focusa elements remain the generated renderer foundations; Svelte hosts those trusted renderers while complete generated C.R.I.S.T. action binding remains translation work.",
        "evidence": ["packages/a2ui-renderer/", "packages/focusa-elements/", "apps/desktop/src/lib/mission-canvas/contributions/GeneratedSurfaceContribution.svelte"],
    },
    {
        "id": "STATE-008",
        "classification": "projection_only",
        "finding": "The Svelte menubar MissionCanvasView remains a bounded projection; the production adaptive workspace destination is apps/desktop, not menubar and not the Pi overlay.",
        "evidence": ["apps/menubar/src/lib/components/MissionCanvasView.svelte", "apps/desktop/src/lib/mission-canvas/DesktopMissionCanvasRuntime.svelte"],
    },
    {
        "id": "STATE-009",
        "classification": "provider_blocked",
        "finding": "Beads auto-import rejects legacy issue type security and exposes zero issues; task materialization cannot be trusted until provider repair and reconciliation complete.",
        "evidence": ["bd info: invalid issue type security at issue 305"],
    },
    {
        "id": "STATE-010",
        "classification": "baseline_failure",
        "finding": "The canonical Spec gate baseline advances through corrected reopened acceptance and then stops at SPEC135-M4 exact surface-binding E2E equality.",
        "evidence": ["/tmp/spec135-current-baseline.log", "tests/spec135_m4_surface_bindings_e2e_test.py"],
    },
    {
        "id": "STATE-011",
        "classification": "proof_missing",
        "finding": "The current rich-GUI proof is explicitly invalidated; same-session toggle, real rich splits, vertical recomposition, generated C.R.I.S.T., responsive/accessibility, reconnect, and UIAI runtime evidence remain open.",
        "evidence": ["docs/contracts/spec135-mission-canvas-agent-first-gui-proof.v1.json"],
    },
    {
        "id": "STATE-012",
        "classification": "cross_platform_boundary",
        "finding": "Mission Canvas must ship in the Pi extension through the canonical Git release pipeline on macOS, Windows, and Linux; no macOS-only or separate-product implementation is conformant.",
        "evidence": ["operator confirmation", "AGENTS.md", "docs/canonical-live-release-pipeline.md"],
    },
]

FILE_RECONCILIATION = [
    {"path": "AGENTS.md", "state": "partial_update_required", "preserve": "Project discipline, release law, Spec 135 preflight", "pivot": "Reference adaptive-composition authority and completion DAG; prohibit fixed-slot/dead-chrome closure."},
    {"path": "docs/135-series-current-manifest.md", "state": "partial_update_required", "preserve": "Current series authority and reopened Svelte-hosted truth", "pivot": "Incorporate replacement authority, F13/F14, adaptive occupancy requirements, and generated DAG."},
    {"path": "docs/agent/spec135-implementation-acceleration-directive.md", "state": "strong_foundation_update_required", "preserve": "Fixed stack, foundation train, Svelte-hosted sequence, proof classes", "pivot": "Insert contribution resolver/no-dead-chrome train before Svelte-hosted shell construction."},
    {"path": "docs/contracts/spec135/authoritative-handoff/spec135_agent_handoff_apple_principles.md", "state": "current_authority", "preserve": "All replacement text verbatim", "pivot": "Translate every law and proof requirement into machine contracts/tests without modifying source."},
    {"path": "docs/contracts/spec135/authoritative-handoff/focusa_activity_mode_recomposition.png", "state": "current_visual_example", "preserve": "Populated activity-mode visual intent", "pivot": "Use as fixture reference, never permanent inventory."},
    {"path": "docs/contracts/spec135/authoritative-handoff/focusa_dynamic_vertical_recomposition.png", "state": "current_visual_example", "preserve": "Populated vertical visual intent", "pivot": "Use as fixture reference, never permanent inventory."},
    {"path": "docs/contracts/spec135-mission-canvas-host-renderer-contract.v1.yaml", "state": "authoritative_but_conflicted", "preserve": "Rich-host boundary, same-session invariant, product boundaries, invalid-proof rules", "pivot": "Reclassify six regions as supported semantic contributions rather than always-visible geometry."},
    {"path": "docs/contracts/spec135-complete-feature-ledger.v1.yaml", "state": "stale_false_closure", "preserve": "73 requirement definitions, owners, dependencies, tests", "pivot": "Reconcile statuses with Project Card/Beads/master acceptance and add F13/F14/adaptive requirements."},
    {"path": "docs/contracts/spec135-delivery-dag.v1.yaml", "state": "stale_false_closure", "preserve": "Legacy requirement edges and foundation ordering", "pivot": "Regenerate from truthful statuses and this completion DAG."},
    {"path": "docs/contracts/spec135-proof-matrix.v1.yaml", "state": "revalidation_required", "preserve": "Existing evidence taxonomy and unaffected proof", "pivot": "Map thirteen occupancy proofs and all Svelte-hosted/cross-platform runtime scenarios."},
    {"path": "docs/contracts/spec135-master-final-acceptance.v1.json", "state": "truthfully_reopened", "preserve": "2/14 generated acceptance and reopen reasons", "pivot": "Regenerate only from runtime evidence after all reopened gates pass."},
    {"path": "docs/contracts/spec135-interaction-mode-toggle.v1.json", "state": "partial_foundation", "preserve": "Mode precedence, durable mode, headless behavior", "pivot": "Add independent host resolution and real same-session ON/OFF proof."},
    {"path": "docs/contracts/spec135-mission-canvas-agent-first-gui-proof.v1.json", "state": "invalidated", "preserve": "Invalidation rationale and historical evidence refs", "pivot": "Replace with generated real-window runtime evidence; never hand-edit to pass."},
    {"path": "apps/pi-extension/src/config.ts", "state": "translation_source_and_compatibility", "preserve": "Interaction-mode persistence and precedence", "pivot": "Translate Mission Canvas presentation preferences into Desktop state; retain only Agent TUI and compatibility routing in Pi."},
    {"path": "apps/pi-extension/src/commands.ts", "state": "translation_source_and_compatibility", "preserve": "Pi command entry and shared controller intent", "pivot": "Route Canvas presentation to the exact Desktop Workstream; do not mount the production Canvas through ctx.ui.custom."},
    {"path": "apps/pi-extension/src/mission-canvas-tool.ts", "state": "translation_source_and_compatibility", "preserve": "Agent-first control surface and status actions", "pivot": "Return truthful Desktop presentation lifecycle results and evidence refs."},
    {"path": "apps/pi-extension/src/session.ts", "state": "translation_source_and_agent_tui_integration", "preserve": "Session/attachment context and startup hooks", "pivot": "Bind exact Desktop Workstream/Attachment handoff while retaining authentic Agent TUI integration."},
    {"path": "apps/pi-extension/src/mission-canvas-shell.ts", "state": "terminal_fallback_only", "preserve": "Truthful terminal-guided fallback and same-session command routing", "pivot": "Remove from Svelte Mission Canvas closure path and never auto-present as complete GUI."},
    {"path": "apps/pi-extension/src/mission-canvas-model.ts", "state": "partial_projection_model", "preserve": "Useful view derivations and naming where semantically valid", "pivot": "Replace client-composed fixed cards with generated ResolvedWorkspaceProjection types."},
    {"path": "apps/pi-extension/src/mission-canvas-layout.ts", "state": "invalid_for_rich_layout", "preserve": "Only migration input and terminal fallback behavior", "pivot": "Replace process-local map/string splits with canonical durable pane tree and server resolver."},
    {"path": "apps/pi-extension/src/mission-canvas-accessibility.ts", "state": "terminal_only_partial", "preserve": "Accessibility intent and fallback preferences", "pivot": "Implement graphical semantic accessibility, focus, scaling, contrast, and reduced-motion contracts."},
    {"path": "apps/pi-extension/src/mission-canvas-session-inventory.ts", "state": "reusable_partial", "preserve": "Session-kind/isolation concepts", "pivot": "Source actual open/pinned/contextual surfaces from canonical API; omit possible-type placeholders."},
    {"path": "apps/pi-extension/src/crist-canvas.ts", "state": "invalid_transcript_projection", "preserve": "Stage-to-operation mapping concepts", "pivot": "Render trusted A2UI/Lit C.R.I.S.T. stages inside focused rich Work Surfaces."},
    {"path": "apps/pi-extension/package.json", "state": "compatibility_and_agent_tui_only", "preserve": "Pi extension entrypoint, Node support, tests", "pivot": "Keep compatibility presentation and Agent TUI integration; package the production Svelte Mission Canvas under apps/desktop."},
    {"path": "apps/menubar/src/lib/components/MissionCanvasView.svelte", "state": "menubar_peek_partial", "preserve": "Canonical-read-model discipline and accessibility snippets", "pivot": "Remain a distinct small projection; do not expand into Pi Svelte Mission Canvas owner."},
    {"path": "apps/menubar/src-tauri/", "state": "technology_reference_not_owner", "preserve": "Existing Tauri/webview/release knowledge where reusable", "pivot": "Use only through an approved Pi-extension-owned portable host architecture; retain product boundaries."},
    {"path": "crates/focusa-core/src/types.rs", "state": "reusable_durable_foundation", "preserve": "MissionCanvasWorkSurfaceRecord, MissionCanvasStateRecord, bindings, revisions", "pivot": "Add contribution/projection/layout-tree/profile/activity/memory/draft/lifecycle types."},
    {"path": "crates/focusa-api/src/routes/mission_canvas_surfaces.rs", "state": "reusable_durable_foundation", "preserve": "Scoped CRUD, bindings, revisions, evidence/receipts", "pivot": "Add canonical projection, registry, layout memory, lifecycle, and draft routes; fix M4 E2E baseline."},
    {"path": "docs/contracts/spec135/generated-contract-v1/operation-registry.json", "state": "reusable_missing_operations", "preserve": "100 generated operations and ownership discipline", "pivot": "Add projection, registry, layout, host lifecycle, draft, diagnostics, and proof operations."},
    {"path": "packages/generated/spec135/typescript/", "state": "reusable_regeneration_required", "preserve": "Generated-client architecture", "pivot": "Regenerate after new schemas/OpenAPI; prohibit handwritten duplicate host payloads."},
    {"path": "packages/a2ui-renderer/", "state": "reusable_generated_ui_foundation", "preserve": "@a2ui/lit 0.9.1 permanent renderer, action binding, validation", "pivot": "Mount inside actual rich Work Surface and prove trusted runtime interactions."},
    {"path": "packages/focusa-elements/", "state": "reusable_component_foundation", "preserve": "Generated trusted Focusa components", "pivot": "Add any missing production components through catalog generation and Svelte Mission Canvas accessibility proof."},
    {"path": "scripts/ci/run-spec-gates.sh", "state": "reusable_gate_runner_update_required", "preserve": "Canonical isolated daemon and Spec gate ordering", "pivot": "Add completion DAG, Svelte Mission Canvas, no-dead, UIAI, cross-platform, and strict runtime closure gates."},
    {"path": ".github/workflows/", "state": "release_foundation_update_required", "preserve": "Canonical Git build/release matrices", "pivot": "Build/package/install/test Pi Svelte Mission Canvas assets on macOS, Windows, and Linux."},
    {"path": ".beads/", "state": "provider_blocked", "preserve": "Historical issue source", "pivot": "Migrate invalid security issue type and materialize approved graph with dependency edges."},
]

PHASES: list[dict[str, Any]] = []


def task(title: str, targets: list[str], result: str, proof: str = "contract", refs: list[str] | None = None) -> dict[str, Any]:
    return {
        "title": title,
        "targets": targets,
        "expected_result": result,
        "proof_class": proof,
        "requirement_refs": refs or [],
    }


def phase(pid: str, title: str, purpose: str, waves: list[tuple[str, list[dict[str, Any]]]], refs: list[str]) -> None:
    PHASES.append({"id": pid, "title": title, "purpose": purpose, "waves": waves, "requirement_refs": refs})


phase(
    "P00",
    "Recovery, authority, and truthful baseline",
    "Make the planning and task surfaces trustworthy before any production UI work.",
    [
        ("identity", [
            task("Verify parent project, working subpath, branch, remote, and PR identity", [".focusa-project.json", ".git", "AGENTS.md"], "One scope envelope names the Focusa parent project and this exact Mission Canvas working subpath without treating a marker from another host as the current root.", "contract"),
            task("Fetch and reconcile the current PR branch without losing operator handoff artifacts", ["git:feature/spec-135-context-connectors-b2"], "Local and remote branch heads match and the working tree is clean before graph materialization.", "runtime"),
            task("Restore a canonical Spec 135 planning Workpoint and writer lease", ["Focusa Workpoint", "Focusa work-loop"], "The graph mission has a canonical Workpoint, exact target refs, writer identity, blockers, and next action.", "runtime"),
            task("Repair Beads legacy issue-type import failure", [".beads/", "Beads import source"], "Legacy security records migrate to a supported type without deleting issue history; bd exposes the canonical issue inventory.", "integration"),
            task("Reconcile Project Card remaining requirements with repository ledgers", ["docs/contracts/spec135-complete-feature-ledger.v1.yaml", "docs/contracts/spec135-delivery-dag.v1.yaml"], "The exact remaining requirement IDs and statuses agree across Project Card, Beads, feature ledger, delivery DAG, and proof matrix.", "contract"),
        ]),
        ("baseline", [
            task("Capture a clean code, test, proof, and release baseline", ["scripts/ci/run-spec-gates.sh", "apps/pi-extension/", "crates/", "packages/"], "A timestamped evidence packet records HEAD, tool versions, platform, checks run, pass/fail, and dirty-file effects.", "runtime"),
            task("Diagnose and repair SPEC135-M4 binding E2E baseline failure", ["tests/spec135_m4_surface_bindings_e2e_test.py", "crates/focusa-api/src/routes/mission_canvas_surfaces.rs"], "The exact binding round-trip passes against an isolated daemon and retains restart durability and scope-denial behavior.", "integration", ["SPEC135-M4"]),
            task("Classify every existing Mission Canvas artifact as reusable, partial, invalid, superseded, or missing", ["apps/pi-extension/", "apps/menubar/", "docs/contracts/", "tests/"], "The audit names each file, canonical owner, retained behavior, invalid claims, and required successor.", "contract"),
            task("Move stale closure claims to truthful reopened states without deleting foundations", ["docs/contracts/spec135-master-final-acceptance.v1.json", "docs/contracts/spec135-proof-matrix.v1.yaml"], "No ledger or proof reports Svelte Mission Canvas completion from terminal, static, handwritten, or file-existence evidence.", "contract"),
        ]),
        ("governance", [
            task("Declare work items for every implementation lane", ["Beads", "Focusa Workpoints"], "Each lane has requirement refs, targets, actions, recipients, permissions, evidence, receipts, dependencies, and drift boundaries.", "contract"),
            task("Create isolated writer scopes and attachment identities", ["git worktrees", "Spec 135G Attachments"], "Parallel lanes cannot mutate the same files or canonical objects without explicit contention handling.", "runtime"),
            task("Keep PR 110 draft and install a false-closure firewall", ["GitHub PR 110", "scripts/ci/run-spec-gates.sh"], "Merge readiness remains false until runtime Svelte Mission Canvas receipts satisfy every reopened gate.", "integration"),
            task("Approve this completion DAG as the implementation sequence authority", [str(GRAPH_PATH.relative_to(ROOT))], "Operator approval freezes dependency order; later changes require versioned supersession and rationale.", "operator"),
        ]),
    ],
    ["SPEC135-F0", "SPEC135-F1", "SPEC135-E1", "SPEC135-Z1", "SPEC135-Z3"],
)

phase(
    "P01",
    "Authority reconciliation and no-dead-chrome contract",
    "Convert the replacement handoff into normative, machine-testable authority before schemas or rendering.",
    [
        ("precedence", [
            task("Encode authority precedence and supersession", ["docs/135-series-current-manifest.md", "docs/contracts/spec135/authoritative-handoff/"], "Replacement text outranks images and older contracts for occupancy; images are populated examples; superseded material cannot close gates.", "contract"),
            task("Reclassify six canonical regions as semantic contribution capabilities", ["docs/contracts/spec135-mission-canvas-host-renderer-contract.v1.yaml"], "Region support remains required while visibility and geometry are conditional on eligible meaningful content.", "contract", ["SPEC135-M1", "SPEC135-M3"]),
            task("Encode the Desktop Svelte Mission Canvas ownership invariant", ["AGENTS.md", "docs/135-series-current-manifest.md"], "Focusa Desktop owns Mission Canvas presentation while Core owns runtime authority; Pi retains Agent TUI and compatibility handoff responsibilities.", "contract"),
            task("Encode populated-image-example semantics", ["docs/contracts/spec135/authoritative-handoff/"], "Tests cannot infer permanent tabs, rails, queues, panels, or geometry merely because they appear in a populated reference image.", "contract"),
        ]),
        ("occupancy", [
            task("Define contribution eligibility inputs", ["docs/contracts/spec135-adaptive-composition.v1.yaml"], "Profile, activity, focused surface, read model, operations, capabilities, permissions, viewport, project constraints, and preferences are all required resolver inputs.", "contract"),
            task("Define meaningful-content and semantic-relevance predicates", ["docs/contracts/spec135-adaptive-composition.v1.yaml"], "Eligibility cannot be satisfied by placeholders, synthetic substitute data, migration warnings, or empty collections.", "contract"),
            task("Define omission reasons and diagnostic visibility", ["docs/contracts/spec135-adaptive-composition.v1.yaml"], "Every omitted contribution has a canonical internal reason that is not automatically rendered in the primary workspace.", "contract"),
            task("Define capability absence and active-loss behavior", ["docs/contracts/spec135-adaptive-composition.v1.yaml"], "Dead controls disappear, safe loaded content is retained, affected regions recompose, and canonical state survives capability return.", "contract"),
            task("Define semantic anti-counterfeiting law", ["docs/contracts/spec135-adaptive-composition.v1.yaml"], "Unknown or missing semantic IDs are never replaced with unrelated content to fill geometry.", "contract"),
        ]),
        ("composition", [
            task("Define conditional inspector composition", ["docs/contracts/spec135-adaptive-composition.v1.yaml"], "Empty sections are absent and remaining sections close gaps in deterministic profile/activity order.", "contract"),
            task("Define Work Rail collapse and creation affordance", ["docs/contracts/spec135-adaptive-composition.v1.yaml"], "An empty rail consumes no heading, border, or space while New Workpoint remains contextually reachable.", "contract"),
            task("Define one-queue, two-queue, and zero-queue composition", ["docs/contracts/spec135-adaptive-composition.v1.yaml"], "One queue spans, two follow profile arrangement, zero disappear and allow Prompt Editor expansion.", "contract"),
            task("Define actual-only Work Surface strip", ["docs/contracts/spec135-adaptive-composition.v1.yaml"], "Only open, pinned, or contextually important surfaces appear; Add Surface owns discovery.", "contract"),
            task("Define meaningful-only profile selector", ["docs/contracts/spec135-adaptive-composition.v1.yaml"], "Daily selector contains only profiles capable of producing a meaningful projection; installation lives in management flow.", "contract"),
            task("Define deterministic reflow and balanced-composition criteria", ["docs/contracts/spec135-adaptive-composition.v1.yaml"], "Removal expands, promotes, merges, tabs, or rearranges valid contributions without holes or random movement.", "contract"),
        ]),
        ("quality", [
            task("Define interaction-quality and no-compromised-slop gate", ["docs/contracts/spec135-ux-quality-bar.v1.yaml"], "Pointer, keyboard, focus, motion, density, loading, empty, error, and recovery behavior must be production-complete before closure.", "contract", ["SPEC135-K3", "SPEC135-K4"]),
            task("Define supported viewport and device matrix", ["docs/contracts/spec135-responsive-matrix.v1.yaml"], "Desktop widths, minimum window sizes, scaling, high contrast, reduced motion, and supported platform combinations are explicit.", "contract", ["SPEC135-U6", "SPEC135-Q3"]),
            task("Map thirteen replacement proof requirements to requirement IDs", ["docs/contracts/spec135-proof-matrix.v1.yaml"], "Every no-dead-chrome proof has owner, scenario, viewport, evidence, receipt, and merge gate.", "contract", ["SPEC135-Z1"]),
            task("Install static drift tests for replacement authority", ["tests/spec135_adaptive_composition_authority_test.py"], "CI fails on fixed-slot requirements, permanent unavailable cards, dead options, local ad-hoc reflow, or image-as-inventory claims.", "unit"),
        ]),
    ],
    ["SPEC135-F0", "SPEC135-F1", "SPEC135-M1", "SPEC135-M3", "SPEC135-K3", "SPEC135-K4", "SPEC135-Z1"],
)

phase(
    "P02",
    "Canonical composition schemas and generated contracts",
    "Define the complete portable data model before reducer, API, client, host, or UI implementation.",
    [
        ("identity-types", [
            task("Define ContributionId, ContributionKind, RegionPreference, and SemanticBindingId", ["schemas/spec135/mission-canvas/"], "Stable opaque IDs and versioned enums reject unknown mutations while preserving unknown read data for migration.", "unit"),
            task("Define CandidateContribution", ["schemas/spec135/mission-canvas/candidate-contribution.schema.json"], "Candidate includes semantic owner, profile/activity applicability, renderer binding, priority, adjacency, spans, merge/tab policy, and capability requirements.", "unit"),
            task("Define ContributionEligibilityContext", ["schemas/spec135/mission-canvas/eligibility-context.schema.json"], "All resolver inputs are typed, scoped, revisioned, and free of client-invented authority.", "unit"),
            task("Define EligibilityDecision and OmissionDiagnostic", ["schemas/spec135/mission-canvas/eligibility-decision.schema.json"], "Decision records eligible/omitted/merged/compacted/suspended outcome, reason, evidence refs, and deterministic rule version.", "unit"),
            task("Define ResolvedContribution", ["schemas/spec135/mission-canvas/resolved-contribution.schema.json"], "Resolved item contains renderer, data ref, operations, authority, freshness, geometry constraints, and accessibility metadata.", "unit"),
        ]),
        ("projection-types", [
            task("Define canonical LayoutNode pane tree", ["schemas/spec135/mission-canvas/layout-node.schema.json"], "Single, split, stack, grid, tabs, and inspector nodes are recursive, ratio-bounded, revisioned, and renderer-independent.", "unit"),
            task("Define ResolvedWorkspaceProjection", ["schemas/spec135/mission-canvas/resolved-workspace-projection.schema.json"], "Projection identifies scope, profile, activity, focused surface, contribution set, layout tree, omission diagnostics, operation bindings, revision, and event cursor.", "unit"),
            task("Define deterministic projection digest", ["schemas/spec135/mission-canvas/projection-digest.schema.json"], "Equivalent canonical inputs produce the same normalized digest across process, reconnect, and platform.", "unit"),
            task("Define WorkspaceProfile and ActivityMode records", ["schemas/spec135/mission-canvas/workspace-profile.schema.json", "schemas/spec135/mission-canvas/activity-mode.schema.json"], "Profiles and modes supply candidates and preferences but cannot force ineligible visibility.", "unit"),
            task("Define registry entry schemas", ["schemas/spec135/mission-canvas/registries.schema.json"], "WorkspaceProfile, Panel, HomeCanvas, WorkSurfaceRenderer, ArtifactRenderer, Terminology, and DomainSemanticBinding registries are versioned and portable.", "unit"),
        ]),
        ("state-types", [
            task("Define durable ProfileLayoutMemory", ["schemas/spec135/mission-canvas/profile-layout-memory.schema.json"], "Memory preserves preferred placement for absent contributions without reserving geometry.", "unit"),
            task("Define CanvasDraftState", ["schemas/spec135/mission-canvas/canvas-draft.schema.json"], "Pi and Canvas drafts have explicit ownership, revision, recipient, attachment, and conflict semantics.", "unit"),
            task("Define HostLifecycleState and HostRendererResolution", ["schemas/spec135/mission-canvas/host-lifecycle.schema.json"], "Renderer selection, process/window identity, session binding, focus, reconnect, and fallback reasons are typed independently from interaction mode.", "unit"),
            task("Define CapabilityProjection and AvailableOperation filtering", ["schemas/spec135/mission-canvas/capability-projection.schema.json"], "Unavailable or unauthorized operations are absent before rendering and diagnostic causes remain internal.", "unit"),
            task("Define layout mutation commands and optimistic concurrency", ["schemas/spec135/mission-canvas/layout-mutation.schema.json"], "Open, focus, pin, group, reorder, split, compare, suspend, rehydrate, and close require expected revisions and explicit attachments.", "unit"),
        ]),
        ("events-and-proof", [
            task("Define projection and lifecycle event taxonomy", ["schemas/spec135/mission-canvas/events.schema.json"], "Candidate, eligibility, projection, layout, focus, draft, capability, host, reconnect, and receipt events support replay and deduplication.", "unit"),
            task("Define recomposition Evidence and Receipt envelopes", ["schemas/spec135/mission-canvas/recomposition-proof.schema.json"], "Proof identifies profile, activity, candidates, eligible set, omissions, layout revision, viewport, platform, session, attachment, and event cursor.", "unit"),
            task("Define responsive evaluation fixtures", ["schemas/spec135/mission-canvas/eval-scenario.schema.json"], "Scenario has canonical seed state, capability/permission changes, viewport sequence, expected projection digests, and continuity assertions.", "unit"),
            task("Define migration envelopes from terminal/process-local layouts", ["schemas/spec135/mission-canvas/migration.schema.json"], "Migration preserves canonical surfaces and preferences while discarding invalid reserved geometry and records a receipt.", "unit"),
        ]),
        ("operations", [
            task("Add projection read and resolve operations", ["docs/contracts/spec135/generated-contract-v1/operation-registry.json"], "Typed operations read canonical inputs and resolve projections without granting mutation authority.", "contract", ["SPEC135-F4", "SPEC135-J1"]),
            task("Add profile, activity, registry, and layout-memory operations", ["docs/contracts/spec135/generated-contract-v1/operation-registry.json"], "Each operation has exact scope, owner, idempotency, permission, event, and invalidation contracts.", "contract", ["SPEC135-F4"]),
            task("Add Svelte Mission Canvas lifecycle operations", ["docs/contracts/spec135/generated-contract-v1/operation-registry.json"], "Launch, focus, status, close, reconnect, and fallback operations bind to one Pi session and attachment.", "contract", ["SPEC135-F13", "SPEC135-F14"]),
            task("Add draft synchronization and recipient-routing operations", ["docs/contracts/spec135/generated-contract-v1/operation-registry.json"], "Draft and send operations preserve explicit recipient and attachment authority across host transitions.", "contract"),
            task("Add recomposition diagnostics and proof operations", ["docs/contracts/spec135/generated-contract-v1/operation-registry.json"], "Diagnostics are explicit tools/Controls surfaces, never automatic empty workspace cards.", "contract"),
        ]),
        ("generation", [
            task("Regenerate JSON Schema 2020-12 bundle", ["docs/contracts/spec135/generated-contract-v1/"], "All new types validate portable fixtures and reject malformed authority, revisions, geometry, and semantic IDs.", "contract", ["SPEC135-F2"]),
            task("Regenerate OpenAPI 3.0.3", ["docs/contracts/spec135/generated-contract-v1/openapi-3.0.3.json"], "Every operation has request, response, error, security, and compatibility schema parity.", "contract", ["SPEC135-F2"]),
            task("Regenerate TypeScript clients and validators", ["packages/generated/spec135/typescript/"], "Generated client compiles in Pi extension and Svelte Mission Canvas without handwritten duplicate payload types.", "contract", ["SPEC135-F3"]),
            task("Regenerate capability snapshot and UI action bindings", ["docs/contracts/spec135/generated-contract-v1/ui-capability-snapshot.fixture.json", "docs/contracts/spec135/generated-contract-v1/ui-action-bindings.fixture.json"], "Renderers consume only registry-declared capabilities and actions.", "contract", ["SPEC135-F4"]),
            task("Update compatibility lock and protocol handshake", ["docs/contracts/spec135/generated-contract-v1/compatibility-lock.yaml"], "Old clients fail closed or use truthful terminal fallback; no silent schema mismatch reaches the Svelte Mission Canvas.", "integration", ["SPEC135-J2"]),
            task("Add generated-contract parity and determinism gates", ["tests/spec135_resolved_projection_contract_test.py"], "Schema, OpenAPI, operation registry, client, fixtures, and digest vectors remain byte-stable and complete.", "unit"),
        ]),
    ],
    ["SPEC135-F2", "SPEC135-F3", "SPEC135-F4", "SPEC135-J1", "SPEC135-J2", "SPEC135-G1", "SPEC135-G2"],
)

phase(
    "P03",
    "Core resolver, reducer, persistence, API, and event authority",
    "Implement adaptive composition in canonical server-owned logic; clients render projections and never invent reflow.",
    [
        ("storage", [
            task("Add canonical mission_canvas composition module", ["crates/focusa-core/src/mission_canvas/"], "Composition types and services have one canonical owner separate from terminal and Svelte clients.", "unit"),
            task("Add SQLite migrations for profiles, modes, registries, layout trees, memory, drafts, and lifecycle", ["crates/focusa-core/migrations/"], "All new durable records migrate forward, rollback where supported, and preserve existing surfaces/state/bindings.", "integration"),
            task("Implement append-only composition and layout events", ["crates/focusa-core/src/mission_canvas/events.rs"], "Every accepted mutation emits scoped replayable events with monotonic revisions and idempotency keys.", "unit"),
            task("Implement repository queries and transactional writes", ["crates/focusa-core/src/mission_canvas/repository.rs"], "Projection inputs and mutations are atomic, scope-exact, restart durable, and concurrency checked.", "integration"),
        ]),
        ("eligibility", [
            task("Implement candidate collection from registries", ["crates/focusa-core/src/mission_canvas/resolver.rs"], "Candidates are assembled from active profile, activity, focused surface, installed domain packs, and canonical state.", "unit"),
            task("Implement semantic relevance and meaningful-content evaluation", ["crates/focusa-core/src/mission_canvas/eligibility.rs"], "Empty, irrelevant, unauthorized, unsupported, and viewport-inappropriate contributions are omitted before layout.", "unit"),
            task("Implement operation, permission, and capability filtering", ["crates/focusa-core/src/mission_canvas/eligibility.rs"], "Dead controls and actions never enter the resolved projection.", "unit"),
            task("Implement omission diagnostics", ["crates/focusa-core/src/mission_canvas/diagnostics.rs"], "Reasons are durable and queryable but excluded from primary projection unless explicitly requested.", "unit"),
            task("Implement active capability-loss transitions", ["crates/focusa-core/src/mission_canvas/capability.rs"], "Safe content retention, stale marking, control removal, collapse, restoration, and notification decisions are deterministic.", "unit"),
        ]),
        ("layout", [
            task("Implement deterministic contribution ranking", ["crates/focusa-core/src/mission_canvas/layout.rs"], "Priority, adjacency, focus, profile, activity, and stable ID tie-breakers produce deterministic order.", "unit"),
            task("Implement span and geometry constraint solver", ["crates/focusa-core/src/mission_canvas/layout.rs"], "Min/max spans and viewport constraints resolve without overflow, overlap, or unused holes.", "unit"),
            task("Implement merge, tab, stack, split, grid, and inspector rules", ["crates/focusa-core/src/mission_canvas/layout.rs"], "Valid contributions compose into supported layout nodes without semantic substitution.", "unit"),
            task("Implement queue occupancy rules", ["crates/focusa-core/src/mission_canvas/layout.rs"], "Two, one, and zero queue cases match authority and resize Prompt Editor deterministically.", "unit"),
            task("Implement Work Rail and inspector occupancy rules", ["crates/focusa-core/src/mission_canvas/layout.rs"], "Empty regions vanish completely and remaining content expands or promotes deliberately.", "unit"),
            task("Implement no-dead-chrome invariant checker", ["crates/focusa-core/src/mission_canvas/invariants.rs"], "Resolved output cannot contain empty panels, blank geometry, dead options, or unrelated substitute contributions.", "property"),
        ]),
        ("memory", [
            task("Implement per-profile layout memory reducer", ["crates/focusa-core/src/mission_canvas/memory.rs"], "Preferred positions survive contribution disappearance without appearing in active geometry.", "unit"),
            task("Implement contribution return placement", ["crates/focusa-core/src/mission_canvas/memory.rs"], "Returning content uses compatible remembered placement and otherwise deterministic current rules.", "unit"),
            task("Implement focus and draft preservation", ["crates/focusa-core/src/mission_canvas/state.rs"], "Focused surface, editor draft, recipient, selection, and scroll anchors survive recomposition and reconnect.", "integration"),
            task("Implement viewport-specific memory without semantic forks", ["crates/focusa-core/src/mission_canvas/memory.rs"], "Responsive preferences may differ geometrically but retain the same canonical contribution identities and state.", "unit"),
            task("Implement legacy layout migration", ["crates/focusa-core/src/mission_canvas/migration.rs"], "Process-local or fixed-slot preferences become durable contribution preferences with migration receipt and no fake panels.", "integration"),
        ]),
        ("api", [
            task("Expose resolved projection read endpoint", ["crates/focusa-api/src/routes/mission_canvas_projection.rs"], "Exact project/workstream/session/attachment/profile/activity/viewport request returns one revisioned projection and digest.", "integration"),
            task("Expose profile, activity, registry, and memory endpoints", ["crates/focusa-api/src/routes/mission_canvas_projection.rs"], "Reads and governed mutations enforce scope, permissions, expected revisions, and idempotency.", "integration"),
            task("Expose host lifecycle and draft synchronization endpoints", ["crates/focusa-api/src/routes/mission_canvas_host.rs"], "Only the bound Pi extension instance can control its rich window and drafts.", "integration"),
            task("Publish projection deltas through durable event stream", ["crates/focusa-api/src/routes/events.rs"], "Reconnect from cursor replays deduplicated projection and lifecycle changes without rebuilding from transcript tail.", "integration", ["SPEC135-J3", "SPEC135-J4"]),
            task("Emit Evidence and Receipts for recomposition and lifecycle", ["crates/focusa-api/src/routes/mission_canvas_projection.rs"], "Proof refs identify exact canonical inputs, output digest, platform, viewport, and authority.", "integration"),
        ]),
        ("core-proof", [
            task("Add eligibility table tests", ["crates/focusa-core/tests/mission_canvas_eligibility.rs"], "Every omission reason, authority combination, and capability state has positive and negative cases.", "unit"),
            task("Add layout property and golden-vector tests", ["crates/focusa-core/tests/mission_canvas_layout.rs"], "Permutation, removal, return, and viewport vectors are deterministic and hole-free.", "property"),
            task("Add persistence, restart, and concurrency tests", ["crates/focusa-core/tests/mission_canvas_persistence.rs"], "State, memory, drafts, revisions, and idempotency survive restart and reject stale writers.", "integration"),
            task("Add API scope and permission E2E", ["crates/focusa-api/tests/mission_canvas_projection_e2e.rs"], "Cross-project, cross-session, cross-attachment, and unauthorized operations fail closed.", "integration"),
            task("Add resolver performance benchmark", ["crates/focusa-bench/benches/mission_canvas_projection.rs"], "Projection and delta resolution meet declared p50/p95/p99 budgets at maximum supported contribution counts.", "performance"),
        ]),
    ],
    ["SPEC135-G1", "SPEC135-G2", "SPEC135-G3", "SPEC135-G4", "SPEC135-G5", "SPEC135-G6", "SPEC135-J3", "SPEC135-J4", "SPEC135-J5", "SPEC135-Q1", "SPEC135-Q3"],
)

phase(
    "P04",
    "Focusa Desktop Mission Canvas Svelte lifecycle",
    "Mount, restore, and govern the authoritative Mission Canvas in the Desktop Svelte GUI tab while retaining Pi as the separate authentic Agent TUI surface.",
    [
        ("architecture", [
            task("Write Desktop Mission Canvas lifecycle call-stack design and ADR", ["docs/transitions/FOCUSA-TRANSITION-001-mission-canvas-desktop-callgraph.yaml", "docs/transitions/FOCUSA-TRANSITION-001-spec135-svelte-translation-matrix.md"], "The design binds every original overlay behavior to the Svelte destination while Core and exact Workstream identity retain authority.", "design"),
            task("Define Desktop Canvas lifecycle and Pi attachment events", ["apps/desktop/src/lib/mission-canvas/event-client.ts", "docs/contracts/spec135/mission-canvas-v1/operation-registry.yaml"], "Canvas lifecycle and Pi attachment/tool events use stable exact-scope IDs without making the Pi overlay the presentation owner.", "contract"),
            task("Define Desktop Svelte root mount and restore adapter", ["apps/desktop/src/lib/shell/MissionCanvasShell.svelte"], "Mount, dispose, focus restoration, and complete-tab restoration are explicit and idempotent.", "contract"),
            task("Prohibit competing Canvas hosts and browser execution outside UIAI", ["docs/contracts/spec135-mission-canvas-host-renderer-contract.v1.yaml", "apps/desktop/src/lib/shell/MissionCanvasShell.svelte"], "Mission Canvas cannot transfer runtime authority to a second resolver, hidden sidecar UI, or non-UIAI browser executor.", "security"),
            task("Map Desktop runtime and Pi attachment invariants", ["apps/desktop/src/lib/shell/pi-attachment-contract.ts", "apps/desktop/src/lib/mission-canvas/exact-scope.ts"], "Desktop presentation and Agent TUI attachment responsibilities are explicit and cannot mint each other's authority.", "design"),
            task("Add exact-identity Desktop lifecycle security gate", ["apps/desktop/src/lib/mission-canvas/DesktopMissionCanvasRuntime.svelte", "apps/desktop/src/lib/shell/pi-attachment-contract.ts"], "Presentation, steering, and restoration fail closed for a foreign or incomplete Workstream/Attachment chain.", "security"),
        ]),
        ("lifecycle", [
            task("Mount Mission Canvas through the Desktop Svelte root", ["apps/desktop/src/lib/shell/MissionCanvasShell.svelte", "apps/desktop/src/lib/mission-canvas/DesktopMissionCanvasRuntime.svelte"], "Mission Canvas replaces the complete inner GUI tab and renders the canonical projection without a fixed dashboard.", "runtime"),
            task("Restore Agent TUI through the deterministic app-wide switch", ["apps/desktop/src/lib/shell/MissionCanvasShell.svelte", "apps/desktop/src/lib/shell/AgentTuiSurface.svelte"], "Switching tabs restores the authentic PTY-backed Pi surface without leaving Mission Canvas chrome behind.", "runtime"),
            task("Implement idempotent present, focus, hide, and status control", ["apps/desktop/src/lib/mission-canvas/DesktopMissionCanvasRuntime.svelte", "docs/contracts/spec135/mission-canvas-v1/operation-registry.json"], "Repeated lifecycle operations converge on one exact Desktop Workstream presentation.", "runtime"),
            task("Stream exact Pi Work Surface activity into Mission Canvas", ["apps/desktop/src/lib/shell/PtyTerminal.svelte", "apps/desktop/src/lib/mission-canvas/event-client.ts"], "Canonical attachment events can update eligible Work Surface contributions without duplicating terminal transcript state.", "runtime"),
            task("Preserve unsent governed draft input across tab switches", ["apps/desktop/src/lib/mission-canvas/draft-controller.svelte.ts", "apps/desktop/src/lib/mission-canvas/contributions/PromptEditorContribution.svelte"], "Switching to Agent TUI and back does not discard or misroute a canonical synchronized draft.", "runtime"),
            task("Reject duplicate Svelte roots and stale runtime disposal", ["apps/desktop/src/lib/shell/MissionCanvasShell.svelte", "apps/desktop/src/lib/mission-canvas/projection-controller.svelte.ts"], "Only the current runtime generation can render, refresh, or dispose the active Canvas.", "runtime"),
        ]),
        ("state", [
            task("Persist interaction mode and workspace profile preferences", ["apps/desktop/src/lib/shell/sidebar-preferences.ts", "apps/desktop/src/lib/mission-canvas/profile-memory-controller.svelte.ts"], "Presentation preferences survive restart without becoming authority or contribution eligibility.", "runtime"),
            task("Keep Desktop shell state attachment-scoped and non-canonical", ["apps/desktop/src/lib/mission-canvas/exact-scope.ts"], "Local presentation state cannot select, infer, or repair canonical identity.", "security"),
            task("Migrate prior Pi-overlay presentation preferences to Desktop", ["apps/pi-extension/src/config.ts", "apps/desktop/src/lib/shell/sidebar-preferences.ts"], "Recognized presentation preferences migrate once while overlay-only host state is quarantined.", "migration"),
            task("Bind Canvas state to exact Workstream and Attachment identity", ["apps/desktop/src/lib/mission-canvas/exact-scope.ts", "apps/desktop/src/lib/shell/pi-attachment-contract.ts"], "No project-root, CWD, tab, or latest-record heuristic can bind Canvas state.", "security"),
            task("Record Desktop restart and restoration receipts", ["apps/desktop/src/lib/mission-canvas/event-client.ts", "docs/contracts/evidence/"], "Restoration evidence identifies exact scope, runtime generation, revision, and resulting presentation.", "evidence"),
        ]),
        ("compatibility", [
            task("Define Agent TUI capability handshake", ["apps/desktop/src/lib/shell/pi-attachment-contract.ts"], "Desktop reports PTY attachment capabilities without treating Agent TUI as the Mission Canvas host.", "contract"),
            task("Enforce Desktop and Core API compatibility", ["apps/desktop/src/lib/mission-canvas/http-transport.ts", "docs/contracts/spec135/mission-canvas-v1/operation-registry.json"], "Generated contract and operation-registry mismatches block mutation and expose recovery.", "security"),
            task("Implement Desktop development reload workflow", ["apps/desktop/package.json", "apps/desktop/vite.config.ts"], "Svelte changes reload without fabricating canonical runtime state or leaking stale subscriptions.", "runtime"),
            task("Reconnect Canvas projection after daemon interruption", ["apps/desktop/src/lib/mission-canvas/event-client.ts", "apps/desktop/src/lib/mission-canvas/invalidation-controller.ts"], "Replay resumes from the durable cursor and stale projections remain visibly stale until replaced.", "runtime"),
            task("Verify Svelte, Tauri, A2UI, and trusted-element compatibility", ["apps/desktop/package.json", "packages/a2ui-renderer/", "packages/focusa-elements/"], "The supported renderer stack mounts generated and trusted surfaces without a competing schema renderer.", "test"),
            task("Keep Agent TUI as the local degraded recovery root", ["apps/desktop/src/lib/shell/AgentTuiSurface.svelte", "apps/desktop/src/lib/shell/PtyTerminal.svelte"], "When Canvas is unavailable, the authentic Pi PTY remains usable without being misrepresented as the Mission Canvas GUI.", "runtime"),
        ]),
        ("distribution", [
            task("Package Mission Canvas in Focusa Desktop", ["apps/desktop/package.json", "apps/desktop/src-tauri/tauri.conf.json"], "Desktop owns the production Mission Canvas GUI assets while Pi retains only Agent TUI and compatibility integration.", "release"),
            task("Install and update Desktop Mission Canvas safely", ["apps/desktop/", "docs/transitions/FOCUSA-TRANSITION-001-preview-build-and-release-milestones.md"], "Updates preserve exact-scope state and generated contract compatibility and support rollback.", "release"),
            task("Verify current macOS Desktop lifecycle", ["apps/desktop/src-tauri/", "docs/contracts/evidence/"], "The native app opens Mission Canvas first and Agent TUI second with correct complete-surface switching.", "test"),
            task("Verify portable Desktop behavior", ["apps/desktop/src-tauri/", "tests/"], "Platform behavior remains capability-aware without changing Core projection semantics.", "test"),
            task("Back up and roll back Desktop state and installed artifacts", ["apps/desktop/", "docs/transitions/FOCUSA-TRANSITION-001-preview-build-and-release-milestones.md"], "Rollback restores the prior compatible Desktop runtime without altering canonical Core records.", "release"),
            task("Run installed Desktop application smoke test", ["apps/desktop/package.json", "tests/"], "Installed Desktop proves canonical Canvas startup, dynamic recomposition, Agent TUI switching, and truthful recovery.", "test"),
        ]),
    ],
    ["SPEC135-F4", "SPEC135-F5", "SPEC135-F6", "SPEC135-F9", "SPEC135-F10", "SPEC135-F11", "SPEC135-F12", "SPEC135-F13", "SPEC135-F14", "SPEC135-V18", "SPEC135-V34"],
)
phase(
    "P05",
    "Authoritative adaptive Svelte-hosted shell",
    "Render the handoff/reference design faithfully inside the current Pi terminal with deterministic occupancy and no dead chrome.",
    [
        ("foundation", [
            task("Lock Pi TUI APIs and dependency-free Canvas renderer", ["apps/pi-extension/package.json", "apps/pi-extension/src/mission-canvas-view.ts"], "The renderer uses approved Pi TypeScript/TUI APIs and introduces no browser framework.", "build"),
            task("Implement reference palette, typography, borders, and density", ["apps/pi-extension/src/mission-canvas-view.ts"], "Dark neutral canvas, purple selection, semantic status colors, and compact professional density match the handoff direction.", "ui"),
            task("Build project/workspace/session top bar and actual Work Surface strip", ["apps/pi-extension/src/mission-canvas-view.ts"], "The frame matches reference hierarchy and only actual surfaces occupy tabs.", "ui"),
            task("Bind current project/workstream/session labels without placeholder authority", ["apps/pi-extension/src/commands.ts", "apps/pi-extension/src/mission-canvas-view.ts"], "Top-bar identity comes from current scoped state and omits invented values.", "integration"),
        ]),
        ("composition", [
            task("Implement persistent activity rail and compact activity tabs", ["apps/pi-extension/src/mission-canvas-view.ts"], "All twelve activity modes are navigable and responsive.", "ui"),
            task("Implement resolved contribution card renderer", ["apps/pi-extension/src/mission-canvas-view.ts"], "Panels are renderer-bound contributions rather than fixed dashboard slots.", "ui"),
            task("Enforce occupancy and no-dead-chrome before geometry", ["apps/pi-extension/src/mission-canvas-view.ts", "apps/pi-extension/tests/mission-canvas-reference-design.test.mjs"], "Empty headings, borders, unavailable cards, and holes are absent.", "ui"),
            task("Implement deterministic one/two-column responsive layout", ["apps/pi-extension/src/mission-canvas-view.ts"], "Narrow, stacked, and desktop widths preserve focus and balanced reflow.", "ui"),
            task("Implement semantic status tones and color-independent markers", ["apps/pi-extension/src/mission-canvas-view.ts"], "Purple, green, blue, amber, red, and cyan reinforce but never solely encode meaning.", "accessibility"),
            task("Limit contribution count and line budgets", ["apps/pi-extension/src/mission-canvas-view.ts", "apps/pi-extension/tests/mission-canvas-performance.test.mjs"], "The workspace remains dense, bounded, and responsive under large canonical projections.", "performance"),
        ]),
        ("interaction", [
            task("Implement keyboard activity/profile/surface navigation", ["apps/pi-extension/src/mission-canvas-shell.ts", "apps/pi-extension/src/mission-canvas-view.ts"], "Focus remains visible and the editor remains usable.", "accessibility"),
            task("Implement per-profile layout memory", ["apps/pi-extension/src/mission-canvas-view.ts"], "Mode preference survives contribution disappearance and profile return.", "ui"),
            task("Bind Canvas actions to current Pi session", ["apps/pi-extension/src/mission-canvas-shell.ts", "apps/pi-extension/src/commands.ts"], "Prompt submission and control actions target the current attachment explicitly.", "integration"),
            task("Render contextual failures without persistent unavailable panels", ["apps/pi-extension/src/mission-canvas-view.ts"], "Capability loss removes dead controls and preserves internal diagnostics.", "ui"),
            task("Preserve selected activity through projection refresh", ["apps/pi-extension/src/mission-canvas-view.ts"], "Canonical refresh never resets operator focus or profile layout memory.", "ui"),
        ]),
        ("queues-and-editor", [
            task("Compose Steering and Follow-up queues by occupancy", ["apps/pi-extension/src/mission-canvas-view.ts"], "Two queues share a row, one spans, and zero leave no geometry.", "ui"),
            task("Keep Prompt Editor bound to current Pi session", ["apps/pi-extension/src/mission-canvas-shell.ts"], "The existing Pi editor target and unsent input semantics are preserved.", "integration"),
            task("Project live transcript and tool activity", ["apps/pi-extension/src/mission-canvas-shell.ts", "apps/pi-extension/src/mission-canvas-tool.ts"], "Visible activity comes from the current Pi session.", "integration"),
            task("Collapse empty Work Rail while preserving creation", ["apps/pi-extension/src/mission-canvas-view.ts"], "No blank rail geometry remains and New Workpoint stays discoverable contextually.", "ui"),
            task("Expand current-session Prompt Editor when queues are absent", ["apps/pi-extension/src/mission-canvas-shell.ts", "apps/pi-extension/tests/mission-canvas-reference-design.test.mjs"], "Empty queues consume no geometry and the editor remains the primary creation surface.", "ui"),
        ]),
        ("accessibility", [
            task("Implement narrow, stacked, and desktop composition", ["apps/pi-extension/src/mission-canvas-view.ts"], "Supported widths never overflow or strand controls.", "accessibility"),
            task("Preserve focus and keyboard-only operation", ["apps/pi-extension/src/mission-canvas-shell.ts"], "All primary navigation is keyboard reachable and focus returns to editor.", "accessibility"),
            task("Support high contrast, reduced motion, and color-independent status", ["apps/pi-extension/src/mission-canvas-accessibility.ts", "apps/pi-extension/src/mission-canvas-view.ts"], "Semantic state never relies on color alone.", "accessibility"),
            task("Expose bounded accessible labels", ["apps/pi-extension/src/mission-canvas-accessibility.ts"], "Surface, state, isolation, and action labels remain readable.", "accessibility"),
            task("Verify full-width ANSI rendering without overflow", ["apps/pi-extension/tests/mission-canvas-performance.test.mjs"], "Visible cell width remains within the active Pi terminal viewport.", "accessibility"),
        ]),
        ("reference-proof", [
            task("Render activity-mode reference states", ["apps/pi-extension/tests/mission-canvas-reference-design.test.mjs"], "Overview and Context compositions match the authoritative populated examples structurally.", "visual"),
            task("Render professional profile reference states", ["apps/pi-extension/tests/mission-canvas-reference-design.test.mjs"], "Software, Legal, Markets, and Research recompositions are semantic, not color-only.", "visual"),
            task("Render sparse and capability-loss states", ["apps/pi-extension/tests/mission-canvas-reference-design.test.mjs"], "Sparse state contains no dead chrome or persistent unavailable cards.", "visual"),
            task("Generate deterministic ANSI and PNG evidence", ["scripts/render-spec135-pi-native-evidence.py", "docs/evidence/spec135-pi-native-reference-renders.png"], "Reference renders are reproducible and reviewable on this Mac.", "visual"),
            task("Compare generated renders with authoritative handoff images", ["docs/evidence/spec135-pi-native-reference-renders.png", "docs/contracts/spec135/authoritative-handoff/focusa_activity_mode_recomposition.png", "docs/contracts/spec135/authoritative-handoff/focusa_dynamic_vertical_recomposition.png"], "Structure, hierarchy, palette direction, occupancy, and profile semantics are reviewed explicitly.", "visual"),
            task("Capture live current-Pi Canvas after extension reload", ["docs/evidence/spec135-pi-native-live-capture.png"], "A real current-terminal capture proves the installed extension rather than only a renderer fixture.", "visual"),
        ]),
    ],
    ["SPEC135-M1", "SPEC135-M2", "SPEC135-M3", "SPEC135-M4", "SPEC135-M5", "SPEC135-M6", "SPEC135-U6", "SPEC135-V1", "SPEC135-V2", "SPEC135-V3", "SPEC135-V4", "SPEC135-V5", "SPEC135-V6", "SPEC135-Q4"],
)

phase(
    "P06",
    "Live Pi session and complete Work Surface mechanics",
    "Make the active Pi transcript one live Work Surface and implement durable professional workspace operations.",
    [
        ("pi-session", [
            task("Define pi_session renderer binding", ["apps/pi-extension/ui/src/lib/surfaces/pi-session/"], "Binding references the active harness session and attachment without transcript duplication.", "contract", ["SPEC135-M4"]),
            task("Render live Pi transcript and tool events", ["apps/pi-extension/ui/src/lib/surfaces/pi-session/PiSessionSurface.svelte"], "Ordered assistant, user, tool, error, and lifecycle events stream with virtualization and stable anchors.", "component"),
            task("Route prompt send to the bound Pi session", ["apps/pi-extension/src/Svelte Mission Canvas/agent-bridge.ts"], "Canvas editor sends through Pi/AgentExecutionAdapter with exact recipient and attachment.", "runtime"),
            task("Route abort, stop, approval, and tool interactions", ["apps/pi-extension/src/Svelte Mission Canvas/agent-bridge.ts"], "Controls preserve Pi authority, confirmation, and evidence semantics.", "runtime"),
            task("Synchronize Pi and Canvas drafts", ["apps/pi-extension/src/Svelte Mission Canvas/drafts.ts"], "Unsent text survives focus changes and ON/OFF without duplicate send or last-writer ambiguity.", "integration"),
            task("Preserve transcript focus and scroll on recomposition", ["apps/pi-extension/ui/src/lib/surfaces/pi-session/PiSessionSurface.svelte"], "New events follow only when appropriate; user reading position and selection remain stable.", "component"),
        ]),
        ("inventory", [
            task("Render canonical Work Surface inventory", ["apps/pi-extension/ui/src/lib/surfaces/store.ts"], "Open, pinned, grouped, suspended, and contextual surfaces come only from canonical state.", "component"),
            task("Implement open and focus mutations", ["apps/pi-extension/ui/src/lib/surfaces/actions.ts"], "Operations use generated client, expected revision, explicit attachment, and optimistic UI that rolls back on rejection.", "integration", ["SPEC135-G3"]),
            task("Implement pin and unpin mutations", ["apps/pi-extension/ui/src/lib/surfaces/actions.ts"], "Pinned state is durable and affects candidate priority without forcing empty visibility.", "integration"),
            task("Implement reorder mutation", ["apps/pi-extension/ui/src/lib/surfaces/actions.ts"], "Order persists per profile/layout memory and remains deterministic across reconnect.", "integration"),
            task("Implement group and ungroup mutation", ["apps/pi-extension/ui/src/lib/surfaces/actions.ts"], "Group topology is canonical, accessible, and restored after restart.", "integration"),
        ]),
        ("pane-mechanics", [
            task("Implement horizontal split mutation", ["apps/pi-extension/ui/src/lib/surfaces/layout-actions.ts"], "Real pane tree changes, both surfaces remain live, ratios persist, and no Markdown representation is used.", "integration", ["SPEC135-G2", "SPEC135-G6"]),
            task("Implement vertical split mutation", ["apps/pi-extension/ui/src/lib/surfaces/layout-actions.ts"], "Real pane tree changes with responsive constraints and durable revision.", "integration"),
            task("Implement split resize persistence", ["apps/pi-extension/ui/src/lib/surfaces/layout-actions.ts"], "Pointer/keyboard resize writes bounded canonical ratio and restores after restart.", "integration"),
            task("Implement compare composition", ["apps/pi-extension/ui/src/lib/surfaces/layout-actions.ts"], "Compatible surfaces compare without changing canonical semantic IDs or inventing content.", "integration"),
            task("Implement suspend projection", ["apps/pi-extension/ui/src/lib/surfaces/actions.ts"], "Projection leaves active layout while canonical runtime/state may continue according to surface policy.", "integration"),
            task("Implement close projection", ["apps/pi-extension/ui/src/lib/surfaces/actions.ts"], "Close affects presentation unless explicit governed runtime-stop action is confirmed.", "integration"),
            task("Implement rehydrate projection", ["apps/pi-extension/ui/src/lib/surfaces/actions.ts"], "Suspended/closed projection restores from durable canonical references, not transcript reconstruction.", "integration"),
        ]),
        ("renderers", [
            task("Implement Project Overview Work Surface", ["apps/pi-extension/ui/src/lib/surfaces/project-overview/"], "Only meaningful mission/project/workpoint/evidence contributions render; empty status cards do not.", "component"),
            task("Implement Silent Session Work Surface", ["apps/pi-extension/ui/src/lib/surfaces/silent-session/"], "Run generation, output cursor, steering, approvals, health, and cleanup remain bound to exact session identity.", "integration"),
            task("Implement Document Work Surface", ["apps/pi-extension/ui/src/lib/surfaces/document/"], "Canonical document/artifact refs render with provenance, actions, and responsive reading/editing.", "component"),
            task("Implement Research Work Surface", ["apps/pi-extension/ui/src/lib/surfaces/research/"], "Sources, claims, freshness, contradictions, and evidence render from canonical operations.", "component"),
            task("Implement Evidence Work Surface", ["apps/pi-extension/ui/src/lib/surfaces/evidence/"], "Proof, receipts, artifacts, gaps, verification, and promotion are meaningful and authority-bound.", "component"),
            task("Implement Provider Item Work Surface", ["apps/pi-extension/ui/src/lib/surfaces/provider/"], "Provider-native identity/status/actions preserve provider ownership and Focusa attachment semantics.", "component"),
            task("Implement Custom Work Surface registration", ["apps/pi-extension/ui/src/lib/surfaces/custom/"], "Trusted registered renderers can participate without arbitrary HTML/JS or unknown semantic reinterpretation.", "security"),
        ]),
        ("rail-and-queues", [
            task("Bind Work Rail to surface-local scope", ["apps/pi-extension/ui/src/lib/shell/WorkRail.svelte"], "Focused surface tasks display with explicit recipient and attachment.", "integration", ["SPEC135-M2"]),
            task("Bind Work Rail to project aggregate scope", ["apps/pi-extension/ui/src/lib/shell/WorkRail.svelte"], "Project items aggregate deterministically without cross-project mutation ambiguity.", "integration"),
            task("Bind labeled cross-project advisory rail", ["apps/pi-extension/ui/src/lib/shell/WorkRail.svelte"], "Advisory items are read-only unless explicit recipient switch and confirmation occur.", "security"),
            task("Implement New Workpoint contextual action", ["apps/pi-extension/ui/src/lib/shell/WorkRailActions.svelte"], "Creation remains available when rail is collapsed and follows Trajectory proposal/checkpoint authority.", "integration"),
            task("Implement steering queue delivery", ["apps/pi-extension/ui/src/lib/queues/steering.ts"], "Steering recipient, preview, contention, writer lease, and delivery receipt are exact.", "integration"),
            task("Implement follow-up queue delivery", ["apps/pi-extension/ui/src/lib/queues/follow-up.ts"], "Deferred delivery waits for current run and retains recipient/scope across reconnect.", "integration"),
        ]),
        ("surface-proof", [
            task("Add live Pi Work Surface runtime test", ["tests/spec135_live_pi_surface_e2e_test.py"], "Prompt, stream, tool event, abort, draft, and scroll continuity run through actual host.", "runtime"),
            task("Add pane mechanics runtime test", ["tests/spec135_real_split_rehydrate_e2e_test.py"], "Real split/group/reorder/resize/suspend/close/rehydrate survive restart and use same canonical runtime.", "runtime", ["SPEC135-G6"]),
            task("Add renderer contract suite", ["apps/pi-extension/ui/tests/work-surface-renderers.test.ts"], "Each renderer proves meaningful-content eligibility, operations, authority, empty omission, and accessibility.", "component"),
            task("Add rail and queue occupancy E2E", ["tests/spec135_rail_queue_recomposition_e2e_test.py"], "Empty, one, and multiple item states prove collapse/span/reflow and recipient continuity.", "runtime"),
        ]),
    ],
    ["SPEC135-M2", "SPEC135-M4", "SPEC135-G1", "SPEC135-G2", "SPEC135-G3", "SPEC135-G4", "SPEC135-G5", "SPEC135-G6"],
)

phase(
    "P07",
    "Workspace profiles, activity modes, and vertical recomposition",
    "Implement both independent resolver axes using registries and shared canonical state.",
    [
        ("registries", [
            task("Implement WorkspaceProfileRegistry", ["crates/focusa-core/src/mission_canvas/registries.rs"], "Profiles are installed/versioned canonical data with candidate contributions and preferences.", "unit", ["SPEC135-V1"]),
            task("Implement ActivityModeRegistry", ["crates/focusa-core/src/mission_canvas/registries.rs"], "Modes contribute semantic candidates and terminology without hard-coded client panels.", "unit"),
            task("Implement Panel and HomeCanvas registries", ["crates/focusa-core/src/mission_canvas/registries.rs"], "Panel semantics, renderer bindings, and home candidates remain data-driven and capability-filtered.", "unit"),
            task("Implement WorkSurfaceRenderer and ArtifactRenderer registries", ["crates/focusa-core/src/mission_canvas/registries.rs"], "Renderer selection is versioned, trusted, and portable.", "unit"),
            task("Implement Terminology and DomainSemanticBinding registries", ["crates/focusa-core/src/mission_canvas/registries.rs"], "Vertical language changes without changing canonical IDs or reducer ownership.", "unit"),
        ]),
        ("activity-modes", [
            task("Implement Overview activity composition", ["docs/contracts/spec135/profiles/activity-overview.json"], "Project-home projection includes only meaningful current mission, focus, active work, evidence, queues, and editor contributions.", "fixture"),
            task("Implement Context activity composition", ["docs/contracts/spec135/profiles/activity-context.json"], "Facts, semantic graph, freshness, assumptions, and conflicts appear only when meaningful.", "fixture"),
            task("Implement Role and Interview activity compositions", ["docs/contracts/spec135/profiles/activity-role-interview.json"], "Role and Grill Interview contributions bind canonical operations and omit empty stages.", "fixture"),
            task("Implement Spec and Tasks/Work activity compositions", ["docs/contracts/spec135/profiles/activity-spec-tasks.json"], "Spec 120, task plans, Workpoints, rail, and artifacts compose around focused work.", "fixture"),
            task("Implement Sessions activity composition", ["docs/contracts/spec135/profiles/activity-sessions.json"], "Actual Pi, Silent, UIAI, Docs, and provider sessions render from inventory without possible-type placeholders.", "fixture"),
            task("Implement Documents, Research, and Evidence compositions", ["docs/contracts/spec135/profiles/activity-docs-research-evidence.json"], "Each mode promotes its semantic work and omits unrelated empty regions.", "fixture"),
            task("Implement History and Controls compositions", ["docs/contracts/spec135/profiles/activity-history-controls.json"], "History/diagnostics are explicit modes and do not occupy daily workspace by default.", "fixture"),
        ]),
        ("verticals", [
            task("Implement General profile", ["docs/contracts/spec135/profiles/general.json"], "General terminology, density, candidates, renderers, and actions produce meaningful nontechnical workspace.", "fixture", ["SPEC135-V1"]),
            task("Implement Software Engineering profile", ["docs/contracts/spec135/profiles/software.json"], "Code, tests, terminal/CI, spec, security, docs, and Pi transcript use actual artifacts and canonical state.", "fixture", ["SPEC135-V2"]),
            task("Implement Legal profile", ["docs/contracts/spec135/profiles/legal.json"], "Matters, redlines, authorities, deadlines, and evidence recompose the same mission/session without copied state.", "fixture", ["SPEC135-V3"]),
            task("Implement Markets profile", ["docs/contracts/spec135/profiles/markets.json"], "Thesis, catalysts, scenarios, sources, risk, and confidence use domain semantics without semantic counterfeiting.", "fixture", ["SPEC135-V4"]),
            task("Implement Research profile", ["docs/contracts/spec135/profiles/research.json"], "Source matrix, claim graph, synthesis, contradictions, confidence, and evidence bind canonical research state.", "fixture", ["SPEC135-V5"]),
            task("Implement Custom profile composition", ["docs/contracts/spec135/profiles/custom.schema.json"], "Custom packs are versioned, validated, permission-scoped, and cannot override core authority or trusted renderer rules.", "fixture", ["SPEC135-V6"]),
        ]),
        ("memory-and-switching", [
            task("Implement profile switching recomposition", ["apps/pi-extension/ui/src/lib/workspace/profile.ts"], "Switch requests canonical projection, preserves focus/surfaces/drafts, and changes more than color.", "integration"),
            task("Implement activity switching recomposition", ["apps/pi-extension/ui/src/lib/workspace/activity.ts"], "Same profile/session resolves different meaningful composition with deterministic focus restoration.", "integration"),
            task("Implement profile-by-activity layout memory", ["crates/focusa-core/src/mission_canvas/memory.rs"], "Preferences persist by profile/activity/viewport and survive contribution disappearance/return.", "integration"),
            task("Implement profile viability filtering", ["crates/focusa-core/src/mission_canvas/resolver.rs"], "Selector excludes profiles that cannot produce meaningful authorized projection.", "unit"),
            task("Implement domain-pack install and management", ["apps/pi-extension/ui/src/lib/workspace/WorkspaceManagement.svelte"], "Setup and unavailable diagnostics are explicit management actions, not dead daily options.", "integration"),
        ]),
        ("vertical-proof", [
            task("Add activity-mode recomposition fixture matrix", ["tests/fixtures/spec135/activity-mode-recomposition/"], "Each supported mode has input, candidates, omissions, projection digest, and expected visual semantics.", "fixture"),
            task("Add vertical recomposition fixture matrix", ["tests/fixtures/spec135/vertical-recomposition/"], "Same canonical mission/session state yields distinct profile compositions and no client-local vertical hard coding.", "fixture"),
            task("Add profile/activity Cartesian contract tests", ["crates/focusa-core/tests/mission_canvas_profiles.rs"], "All viable combinations resolve deterministically; nonviable combinations are omitted from selectors.", "unit"),
            task("Add profile memory disappearance/return E2E", ["tests/spec135_profile_layout_memory_e2e_test.py"], "Placement returns predictably after content/capability disappears and comes back.", "runtime"),
            task("Add UIAI visual recomposition scenarios", ["tests/uiai/spec135/recomposition/"], "Reference populated states and sparse states prove visual quality, reflow, focus, and no dead chrome.", "visual"),
        ]),
    ],
    ["SPEC135-V1", "SPEC135-V2", "SPEC135-V3", "SPEC135-V4", "SPEC135-V5", "SPEC135-V6", "SPEC135-Alpha7", "SPEC135-Alpha8"],
)

phase(
    "P08",
    "Generated C.R.I.S.T. and UIAI browser Work Surfaces",
    "Integrate trusted generated workflows and browser artifacts inside focused Work Surfaces without product-boundary drift.",
    [
        ("generated-runtime", [
            task("Bind permanent A2UI/Lit renderer into Svelte Mission Canvas", ["apps/pi-extension/ui/src/lib/surfaces/generated/"], "Generated surfaces render through @a2ui/lit 0.9.1 and Focusa trusted components inside focused Work Surfaces.", "integration", ["SPEC135-I1"]),
            task("Load Focusa Custom Elements catalog", ["packages/focusa-elements/", "apps/pi-extension/ui/"], "Only cataloged version-compatible components render; unknown components fail safely and diagnostically.", "security", ["SPEC135-F10"]),
            task("Bind generated actions to Operation Registry", ["apps/pi-extension/ui/src/lib/surfaces/generated/actions.ts"], "Generated UI cannot call undeclared operations or bypass confirmation, permission, attachment, evidence, or receipt rules.", "security", ["SPEC135-F4"]),
            task("Implement generated-surface progress and recovery", ["apps/pi-extension/ui/src/lib/surfaces/generated/GeneratedSurface.svelte"], "Long operations expose interruptible progress, retry, recovery, and canonical status without transcript-only fallback.", "component"),
        ]),
        ("crist", [
            task("Implement Context generated Work Surface", ["docs/contracts/spec135/generated-contract-v1/", "apps/pi-extension/ui/src/lib/surfaces/generated/"], "Context ingestion/retrieval/claims use trusted inputs, validation, canonical actions, and evidence.", "runtime", ["SPEC135-C1", "SPEC135-C2", "SPEC135-C3"]),
            task("Implement Role generated Work Surface", ["apps/pi-extension/ui/src/lib/surfaces/generated/"], "Role seed/draft/review use canonical role operations and resume state.", "runtime", ["SPEC135-RI1"]),
            task("Implement Interview generated Work Surface", ["apps/pi-extension/ui/src/lib/surfaces/generated/"], "Grill Interview branching, docs, objections, pause/resume, and closure package are live generated UI.", "runtime", ["SPEC135-RI2", "SPEC135-RI3", "SPEC135-RI4"]),
            task("Implement Spec generated Work Surface", ["apps/pi-extension/ui/src/lib/surfaces/generated/"], "Spec 120 workbench sessions, sections, validation, and evidence render through trusted generated components.", "runtime", ["SPEC135-ST1"]),
            task("Implement Tasks generated Work Surface", ["apps/pi-extension/ui/src/lib/surfaces/generated/"], "Task plan, dependency graph, Beads materialization, Workpoint launch, and receipts are live generated UI.", "runtime", ["SPEC135-ST2", "SPEC135-ST3", "SPEC135-ST4"]),
        ]),
        ("uiai-surface", [
            task("Implement UIAI browser Work Surface binding", ["apps/pi-extension/ui/src/lib/surfaces/uiai-browser/"], "Surface references exact UIAI session/origin/context and never owns browser credentials or canonical execution.", "integration", ["SPEC135-U1"]),
            task("Render browser screenshot, snapshot, diagnostics, and artifact views", ["apps/pi-extension/ui/src/lib/surfaces/uiai-browser/"], "Artifacts remain typed, provenance-linked, isolated, and responsive.", "component", ["SPEC135-U1"]),
            task("Bind governed browser operations", ["apps/pi-extension/ui/src/lib/surfaces/uiai-browser/actions.ts"], "Mutations require trusted origin/session, confirmation, diagnostics, evidence intake, and cleanup.", "security", ["SPEC135-U2"]),
            task("Preserve UIAI Engine Cockpit product boundary", ["docs/contracts/spec135-client-operation-parity.v1.json"], "Mission Canvas may host a projection but does not absorb cockpit target management, FPV, test lab, or browser execution ownership.", "contract"),
        ]),
        ("generated-proof", [
            task("Replace transcript C.R.I.S.T. proof with rich runtime traversal", ["tests/spec135_generated_crist_rich_host_e2e_test.py"], "Every stage renders, accepts input, invokes generated operation, updates live state, and produces evidence/receipt in actual host.", "runtime", ["SPEC135-I2"]),
            task("Add generated UI trust-boundary tests", ["packages/a2ui-renderer/tests/"], "Unknown components/actions, malformed payloads, unsafe HTML/JS, and stale capabilities fail closed.", "security"),
            task("Add UIAI Work Surface isolation E2E", ["tests/spec135_uiai_surface_isolation_e2e_test.py"], "Cross-origin, cross-session, credential, attachment, and cleanup boundaries are proven.", "runtime", ["SPEC135-G5"]),
            task("Capture generated and browser Evidence/Receipts", ["docs/contracts/evidence/spec135-rich-surfaces/"], "Artifacts identify operation, renderer/catalog versions, session/attachment, event cursor, viewport, and result.", "evidence"),
        ]),
    ],
    ["SPEC135-F9", "SPEC135-F10", "SPEC135-I1", "SPEC135-I2", "SPEC135-U1", "SPEC135-U2", "SPEC135-G5"],
)

phase(
    "P09",
    "Continuity, recovery, security, accessibility, and performance hardening",
    "Prove the workspace remains calm, safe, immediate, portable, and canonical under interruption and stress.",
    [
        ("continuity", [
            task("Prove Canvas ON/OFF identity invariants", ["tests/spec135_canvas_toggle_continuity_e2e_test.py"], "Project, workstream, instance, session, attachment, model, tools, transcript, Workpoint, surfaces, layout, queues, evidence, permissions, and cursors remain identical.", "runtime", ["SPEC135-F14"]),
            task("Prove unsent Pi and Canvas draft preservation", ["tests/spec135_canvas_draft_continuity_e2e_test.py"], "Drafts survive ON/OFF, focus, profile/activity changes, host crash, daemon restart, and reconnect without duplicate send.", "runtime"),
            task("Prove durable replay and deduplication", ["tests/spec135_canvas_event_replay_e2e_test.py"], "Disconnect/reconnect resumes from cursor with no missing or duplicate visible actions.", "runtime", ["SPEC135-J4"]),
            task("Prove focus, selection, scroll, and pane restoration", ["tests/spec135_canvas_focus_restore_e2e_test.py"], "Operator context survives all supported recompositions and lifecycle transitions.", "runtime"),
            task("Prove active capability loss and return", ["tests/spec135_capability_loss_recomposition_e2e_test.py"], "Dead controls/regions disappear, safe content/state persist, notification is bounded, and preferred placement returns.", "runtime"),
        ]),
        ("security", [
            task("Threat-model current Pi TUI, generated UI, extension events, and local API", ["docs/contracts/spec135-Svelte Mission Canvas-threat-model.v1.md"], "Assets, origins, tokens, IPC, CSP, permissions, attachments, file access, clipboard, and generated actions have mitigations and tests.", "security", ["SPEC135-Q2"]),
            task("Enforce no-sidecar, terminal-escape, and trusted-origin policy", ["apps/pi-extension/Svelte Mission Canvas/", "apps/pi-extension/ui/"], "No arbitrary remote content, eval, inline unsafe scripts, or untrusted navigation enters the Svelte Mission Canvas.", "security"),
            task("Enforce secrets and credential non-projection", ["crates/focusa-core/src/mission_canvas/", "apps/pi-extension/ui/"], "Projection and diagnostics redact secrets and preserve browser/provider credential ownership.", "security"),
            task("Enforce cross-project advisory and mutation boundaries", ["tests/spec135_canvas_cross_project_security_e2e_test.py"], "Cross-project content is labeled/read-only until explicit target and authority confirmation.", "security"),
            task("Generate SBOM, license, and dependency audit evidence", ["scripts/release/", "docs/contracts/evidence/spec135-Svelte Mission Canvas/"], "All Svelte Mission Canvas dependencies and platform assets pass canonical release security gates.", "security", ["SPEC135-Q1", "SPEC135-Q5"]),
        ]),
        ("accessibility", [
            task("Meet keyboard-only completion for every operation", ["apps/pi-extension/ui/tests/accessibility/"], "No pointer-only action exists; focus order and shortcuts remain coherent after recomposition.", "accessibility", ["SPEC135-U6"]),
            task("Meet screen-reader semantic contract", ["apps/pi-extension/ui/tests/accessibility/"], "Landmarks, names, roles, states, live updates, errors, queues, panes, and generated surfaces are understandable without visual layout.", "accessibility"),
            task("Meet contrast, text scaling, high-contrast, and reduced-motion contract", ["apps/pi-extension/ui/tests/accessibility/"], "Supported platform modes preserve content, focus, and action clarity without clipping or motion dependency.", "accessibility"),
            task("Validate pointer/touch targets and direct manipulation alternatives", ["apps/pi-extension/ui/tests/accessibility/"], "Targets meet declared size and every drag/resize has keyboard and menu equivalents.", "accessibility"),
        ]),
        ("responsive", [
            task("Validate minimum terminal and compact projection", ["tests/uiai/spec135/viewports/"], "Composition remains balanced and operable at minimum supported dimensions without dead chrome or horizontal traps.", "visual"),
            task("Validate standard and wide terminal projection", ["tests/uiai/spec135/viewports/"], "Available geometry improves information density without permanent empty regions.", "visual"),
            task("Validate terminal scaling and cell-width matrix", ["tests/uiai/spec135/viewports/"], "125%, 150%, 200%, and supported text scaling preserve semantics and operations.", "visual"),
            task("Validate dynamic resize interruption", ["tests/uiai/spec135/viewports/"], "Rapid resize/recomposition preserves focus, drafts, session, and deterministic final projection.", "visual"),
        ]),
        ("performance", [
            task("Set projection resolution latency budgets", ["docs/contracts/spec135-q3-performance-budgets.v1.yaml"], "Cold/warm p50/p95/p99 budgets cover resolver, API, event delta, host launch, and reconnect.", "performance", ["SPEC135-Q3"]),
            task("Virtualize large transcript, rail, evidence, and surface inventories", ["apps/pi-extension/ui/src/lib/virtualization/"], "Maximum supported datasets remain responsive and accessible without dropping canonical items.", "performance"),
            task("Eliminate recomposition layout thrash", ["apps/pi-extension/ui/tests/performance/"], "Resize, eligibility changes, and profile switches stay within frame/commit budgets and do not flicker placeholders.", "performance"),
            task("Measure memory and resource cleanup", ["tests/spec135_rich_host_resource_stress_test.py"], "Repeated ON/OFF, surfaces, splits, and reconnects do not leak windows, processes, subscriptions, or unbounded caches.", "performance"),
            task("Run cross-platform long-session stress", [".github/workflows/spec135-Svelte Mission Canvas.yml"], "macOS, Windows, and Linux sustain declared session duration and event volume with correct recovery.", "performance"),
        ]),
        ("hardening-proof", [
            task("Capture accessibility Evidence and Receipt", ["docs/contracts/evidence/spec135-Svelte Mission Canvas/accessibility/"], "Automated and manual results bind host version, platform, viewport, settings, projection digest, and remaining exceptions.", "evidence"),
            task("Capture performance Evidence and Receipt", ["docs/contracts/evidence/spec135-Svelte Mission Canvas/performance/"], "Raw metrics, environment, thresholds, regressions, and pass/fail are durable and reproducible.", "evidence"),
            task("Capture recovery Evidence and Receipt", ["docs/contracts/evidence/spec135-Svelte Mission Canvas/recovery/"], "Crash, daemon restart, network interruption, capability loss, and event-gap recovery traces are canonical.", "evidence"),
            task("Revalidate Q1 through Q6 against Svelte-hosted Canvas", ["docs/contracts/spec135-proof-matrix.v1.yaml"], "Security, privacy, performance, recovery, licensing/SBOM, and release gates are no longer inherited from terminal-only proof.", "integration", ["SPEC135-Q1", "SPEC135-Q2", "SPEC135-Q3", "SPEC135-Q4", "SPEC135-Q5", "SPEC135-Q6"]),
        ]),
    ],
    ["SPEC135-J3", "SPEC135-J4", "SPEC135-Q1", "SPEC135-Q2", "SPEC135-Q3", "SPEC135-Q4", "SPEC135-Q5", "SPEC135-Q6", "SPEC135-U6"],
)

phase(
    "P10",
    "UIAI Engine evaluation and thirteen no-dead-chrome proofs",
    "Prove the replacement handoff through runtime state transitions, supported viewports, and canonical evidence.",
    [
        ("eval-harness", [
            task("Create governed UIAI Svelte Mission Canvas session launcher", ["tests/uiai/spec135/harness/"], "Eval opens the actual Pi-controlled host, binds Focusa scope, captures diagnostics, and cleans up sessions.", "runtime", ["SPEC135-F11"]),
            task("Create canonical fixture seeding and reset", ["tests/uiai/spec135/fixtures/"], "Each scenario starts from explicit state/capabilities/permissions and leaves no cross-test contamination.", "integration"),
            task("Create projection/evidence correlation helper", ["tests/uiai/spec135/harness/"], "Screenshot, DOM/accessibility tree, diagnostics, event cursor, projection digest, and Receipt share one scenario ID.", "integration"),
            task("Create cross-platform viewport matrix runner", [".github/workflows/spec135-Svelte Mission Canvas.yml"], "Same scenarios run on macOS, Windows, and Linux at declared viewports and accessibility settings.", "ci"),
        ]),
        ("occupancy-proofs", [
            task("Prove empty optional contribution leaves no visible or geometric trace", ["tests/uiai/spec135/scenarios/no-empty-contribution.json"], "DOM, accessibility tree, screenshot, and computed layout contain no heading, border, landmark, track, or hole.", "visual"),
            task("Prove deterministic balanced panel-removal reflow", ["tests/uiai/spec135/scenarios/panel-removal-reflow.json"], "Repeated identical removal yields identical projection digest and balanced final composition.", "visual"),
            task("Prove single queue spans queue area", ["tests/uiai/spec135/scenarios/single-queue-span.json"], "One populated queue occupies canonical queue region without blank sibling lane.", "visual"),
            task("Prove zero queues collapse and editor expands", ["tests/uiai/spec135/scenarios/zero-queues-editor-expand.json"], "Both queue DOM regions vanish and Prompt Editor receives resolved expanded geometry.", "visual"),
            task("Prove empty Work Rail collapses with New Workpoint retained", ["tests/uiai/spec135/scenarios/empty-work-rail.json"], "Rail leaves no geometry and contextual New Workpoint action remains governed and operable.", "visual"),
            task("Prove empty inspector sections disappear and gaps close", ["tests/uiai/spec135/scenarios/inspector-occupancy.json"], "Only relevant sections remain in deterministic order with no blank separators.", "visual"),
        ]),
        ("resolver-proofs", [
            task("Prove tabs represent actual Work Surfaces", ["tests/uiai/spec135/scenarios/actual-surface-tabs.json"], "Possible but unopened/irrelevant surface types produce no tab; Add Surface remains available.", "visual"),
            task("Prove profile switch recomputes from candidates", ["tests/uiai/spec135/scenarios/profile-recomposition.json"], "Same canonical state yields expected profile-specific eligible set and geometry, not color-only change.", "visual"),
            task("Prove per-profile memory survives disappearance and return", ["tests/uiai/spec135/scenarios/layout-memory-return.json"], "Preferred valid placement returns with same memory revision and no reserved interim hole.", "visual"),
            task("Prove capability loss avoids unavailable dashboard", ["tests/uiai/spec135/scenarios/capability-loss.json"], "Dependent controls/contributions vanish, safe content remains, workspace reflows, and diagnostics stay internal/contextual.", "visual"),
            task("Prove semantic anti-counterfeiting", ["tests/uiai/spec135/scenarios/no-semantic-substitute.json"], "Unknown/missing semantic contribution causes omission/reflow rather than unrelated filler.", "visual"),
        ]),
        ("continuity-proofs", [
            task("Prove canonical state, session, drafts, and focus survive recomposition", ["tests/uiai/spec135/scenarios/recomposition-continuity.json"], "Before/after IDs, revisions, draft bytes, focus semantic ID, and Workpoint match.", "runtime"),
            task("Prove transitions at every supported viewport", ["tests/uiai/spec135/scenarios/viewport-transition-matrix.json"], "All declared viewport/device/platform combinations produce valid deterministic projections and accessible interactions.", "visual"),
            task("Prove receipts identify profile, activity, contribution set, and layout revision", ["tests/uiai/spec135/scenarios/recomposition-receipt.json"], "Evidence/Receipt fields correlate exactly with runtime projection and artifacts.", "evidence"),
            task("Prove active capability restoration", ["tests/uiai/spec135/scenarios/capability-return.json"], "Contribution returns automatically through resolver and remembered preference without session/focus loss.", "runtime"),
        ]),
        ("eval-closure", [
            task("Run UIAI diagnostics-first failure triage", ["tests/uiai/spec135/results/"], "Console, exception, network, accessibility, and visual failures produce bounded Focusa evidence and block promotion.", "evidence"),
            task("Run populated reference visual comparison", ["tests/uiai/spec135/scenarios/reference-populated-states.json"], "Activity and vertical reference states meet visual intent without forcing their inventory into sparse states.", "visual"),
            task("Run nontechnical usability dogfood", ["tests/uiai/spec135/scenarios/nontechnical-dogfood.json"], "Users can understand, navigate, recover, and complete workflows without terminal knowledge or dead controls.", "usability", ["SPEC135-Alpha8"]),
            task("Promote only passing UIAI artifacts", ["docs/contracts/evidence/spec135-Svelte Mission Canvas/uiai/"], "Promotion requires score above baseline and threshold; failed candidates remain versioned and rollbackable.", "evidence"),
        ]),
    ],
    ["SPEC135-F11", "SPEC135-U3", "SPEC135-U4", "SPEC135-U5", "SPEC135-U6", "SPEC135-K2", "SPEC135-K3", "SPEC135-K4"],
)

phase(
    "P11",
    "Cross-spec reconciliation, release, and final closure",
    "Regenerate all truth from runtime evidence, reconcile every requirement, and ship only through the canonical pipeline.",
    [
        ("reconciliation", [
            task("Reconcile all 73 legacy requirement records", ["docs/contracts/spec135-complete-feature-ledger.v1.yaml"], "Each requirement status is derived from current implementation and runtime proof; unaffected valid evidence is retained, affected evidence is revalidated.", "contract"),
            task("Add F13/F14 and adaptive-composition requirements to Delivery Contract", ["docs/contracts/spec135-complete-feature-ledger.v1.yaml"], "New authority is not hidden inside prose and participates in DAG, parity, proof, and closure matrices.", "contract"),
            task("Regenerate delivery DAG from actual dependencies", ["docs/contracts/spec135-delivery-dag.v1.yaml"], "Graph is acyclic, statuses truthful, ready frontier computed, and final closure depends on all reopened gates.", "contract"),
            task("Regenerate client/framework/proof/parity matrices", ["docs/contracts/spec135-*.yaml", "docs/contracts/spec135-*.json"], "Pi Svelte Mission Canvas, API, CLI, TypeScript, UIAI Cockpit boundary, and fallbacks have complete operation and proof parity.", "contract"),
            task("Reconcile 135 through 135K migration and supersession", ["docs/contracts/spec135-cross-spec-closure.v1.json"], "Amendments, renamed semantics, terminal fallback, fixed-layout artifacts, and rollback paths are explicit.", "contract", ["SPEC135-E1"]),
        ]),
        ("integration", [
            task("Run complete unit and contract suite", ["scripts/ci/run-spec-gates.sh"], "Schemas, reducers, registry, clients, components, provider, and static drift gates pass from clean checkout.", "ci"),
            task("Run complete runtime integration suite", ["scripts/ci/run-spec-gates.sh"], "Real daemon, Pi extension, Svelte Mission Canvas, event replay, Work Surfaces, generated UI, and lifecycle tests pass.", "ci"),
            task("Run macOS Svelte-hosted Canvas release candidate", [".github/workflows/"], "Canonical pipeline builds, signs where configured, packages, installs, and proves the extension on macOS.", "ci"),
            task("Run Windows Pi-extension terminal compatibility candidate", [".github/workflows/"], "Canonical pipeline builds, packages, installs, and proves the extension on Windows.", "ci"),
            task("Run Linux Pi-extension terminal compatibility candidate", [".github/workflows/"], "Canonical pipeline builds, packages, installs, and proves the extension on supported Linux environment.", "ci"),
            task("Run Pi-extension upgrade, downgrade, rollback, and reconnect matrix", ["tests/spec135_release_migration_e2e_test.py"], "Compatible upgrades preserve state; incompatible versions fail closed; rollback restores last valid host/projection without state fork.", "runtime"),
        ]),
        ("acceptance", [
            task("Regenerate interaction-mode and Svelte-hosted reference proof", ["docs/contracts/spec135-interaction-mode-toggle.v1.json", "docs/contracts/spec135-mission-canvas-agent-first-gui-proof.v1.json"], "Artifacts are generated from runtime traces and mark Svelte Mission Canvas accepted only when all lifecycle invariants pass.", "evidence", ["SPEC135-K1", "SPEC135-M1"]),
            task("Regenerate Work Surface and multiplexing proof", ["docs/contracts/spec135-multiplexing-concurrency-proof.v1.json"], "Real pane/group/suspend/rehydrate behavior replaces Markdown/process-local representations.", "evidence", ["SPEC135-G6"]),
            task("Regenerate vertical and generated C.R.I.S.T. proof", ["docs/contracts/spec135/generated-contract-v1/"], "Runtime Svelte Mission Canvas traces prove actual registry recomposition and trusted generated interactions.", "evidence"),
            task("Regenerate performance, recovery, security, and accessibility proof", ["docs/contracts/spec135-q*.yaml", "docs/contracts/evidence/spec135-Svelte Mission Canvas/"], "All quality proof is Svelte Mission Canvas-specific and cross-platform.", "evidence"),
            task("Regenerate master acceptance from evidence status", ["docs/contracts/spec135-master-final-acceptance.v1.json"], "All fourteen gates pass from referenced runtime evidence; no file-existence or handwritten truth inputs remain.", "evidence", ["SPEC135-Z1"]),
        ]),
        ("dogfood", [
            task("Run Alpha 9 Pi light-switch traversal", ["docs/contracts/evidence/spec135-alpha9/"], "Real operator uses Pi OFF, Canvas ON, profile/activity changes, surfaces, generated UI, browser, recovery, and OFF without state loss.", "runtime"),
            task("Run professional vertical dogfood", ["docs/contracts/evidence/spec135-vertical-dogfood/"], "Software, Legal, Markets, and Research users complete representative work without dead chrome or semantic confusion.", "usability"),
            task("Run sparse-state and capability-loss dogfood", ["docs/contracts/evidence/spec135-sparse-dogfood/"], "Workspace remains complete and calm when most contributions are absent or capabilities change.", "usability"),
            task("Resolve all captured friction or explicitly block release", ["docs/contracts/spec135-k2-friction-capture.v1.json"], "No severe unresolved friction, accessibility failure, data-loss risk, or compromised interaction remains.", "usability", ["SPEC135-K2", "SPEC135-K3"]),
        ]),
        ("release", [
            task("Create final Workpoint and Trajectory closure packet", ["Focusa Workpoint", "Focusa Trajectory"], "Mission, evidence, checks, acceptance risks, remaining gaps, and exact authority all prove done rather than inferred.", "evidence"),
            task("Generate permanent integration and clean-lineage evidence", ["docs/contracts/spec135/generated-contract-v1/spec135-z2-permanent-integration-evidence.json", "docs/contracts/spec135/generated-contract-v1/spec135-z3-worktree-lineage-proof.json"], "No temporary bypass, stale worktree, hidden fork, or unpushed artifact remains.", "evidence", ["SPEC135-Z2", "SPEC135-Z3"]),
            task("Pass strict closure gate", ["docs/contracts/spec135/generated-contract-v1/spec135-z4-closure-gate-result.json"], "Strict mode passes all required checks and rejects any reopened/partial/terminal-only gate.", "ci", ["SPEC135-Z4"]),
            task("Merge PR 110 only after all required checks and receipts pass", ["GitHub PR 110"], "PR is mergeable, up to date, fully reviewed, non-draft, and has zero failing/pending required checks.", "operator"),
            task("Release only through canonical GitHub pipeline", ["docs/canonical-live-release-pipeline.md", ".github/workflows/"], "Signed/versioned artifacts and updater metadata originate from canonical pipeline; no local binary deployment occurs.", "release", ["SPEC135-Z5"]),
            task("Verify installed release on macOS, Windows, and Linux", ["docs/contracts/evidence/spec135-release/"], "Installed artifacts launch from Pi, preserve session, recompose, recover, and report exact released versions on all platforms.", "runtime", ["SPEC135-Z5"]),
        ]),
    ],
    ["SPEC135-E1", "SPEC135-Z1", "SPEC135-Z2", "SPEC135-Z3", "SPEC135-Z4", "SPEC135-Z5"],
)


def build_graph() -> dict[str, Any]:
    nodes: list[dict[str, Any]] = []
    edges: list[dict[str, str]] = []
    previous_phase_gate: str | None = None
    sequence = 0
    for p in PHASES:
        previous_wave_gate = previous_phase_gate
        phase_task_ids: list[str] = []
        for wave_index, (wave_name, tasks) in enumerate(p["waves"], start=1):
            wave_task_ids: list[str] = []
            for task_index, item in enumerate(tasks, start=1):
                sequence += 1
                node_id = f"{p['id']}-W{wave_index:02d}-T{task_index:02d}"
                deps = [previous_wave_gate] if previous_wave_gate else []
                node = {
                    "id": node_id,
                    "sequence": sequence,
                    "phase": p["id"],
                    "wave": wave_name,
                    "title": item["title"],
                    "status": "planned",
                    "depends_on": deps,
                    "parallel_within_wave": True,
                    "targets": item["targets"],
                    "expected_result": item["expected_result"],
                    "proof_class": item["proof_class"],
                    "requirement_refs": sorted(set(p["requirement_refs"] + item["requirement_refs"])),
                    "authority_refs": AUTHORITY[:6],
                    "drift_gate": "No implementation may weaken canonical authority, invent semantics, render dead chrome, fork runtime, or self-assert proof.",
                }
                nodes.append(node)
                wave_task_ids.append(node_id)
                phase_task_ids.append(node_id)
                for dep in deps:
                    edges.append({"from": dep, "to": node_id})
            sequence += 1
            wave_gate = f"{p['id']}-W{wave_index:02d}-GATE"
            nodes.append({
                "id": wave_gate,
                "sequence": sequence,
                "phase": p["id"],
                "wave": wave_name,
                "title": f"{p['title']} — {wave_name} exit gate",
                "status": "planned",
                "depends_on": wave_task_ids,
                "parallel_within_wave": False,
                "targets": [],
                "expected_result": "Every task in this wave has implementation evidence, required tests, and no unresolved drift before dependent work starts.",
                "proof_class": "gate",
                "requirement_refs": p["requirement_refs"],
                "authority_refs": AUTHORITY,
                "drift_gate": "Gate fails closed on missing evidence, partial-only implementation, stale scope, or authority conflict.",
            })
            for dep in wave_task_ids:
                edges.append({"from": dep, "to": wave_gate})
            previous_wave_gate = wave_gate
        sequence += 1
        phase_gate = f"{p['id']}-GATE"
        nodes.append({
            "id": phase_gate,
            "sequence": sequence,
            "phase": p["id"],
            "wave": "phase_exit",
            "title": f"{p['title']} — phase exit gate",
            "status": "planned",
            "depends_on": [previous_wave_gate] if previous_wave_gate else [],
            "parallel_within_wave": False,
            "targets": [],
            "expected_result": p["purpose"],
            "proof_class": "gate",
            "requirement_refs": p["requirement_refs"],
            "authority_refs": AUTHORITY,
            "drift_gate": "The next phase remains blocked until this phase is complete and evidence-backed.",
        })
        if previous_wave_gate:
            edges.append({"from": previous_wave_gate, "to": phase_gate})
        previous_phase_gate = phase_gate
    for node in nodes:
        node["translation_contract_ref"] = "CARDINAL-135-SVELTE-001"
    graph = {
        "schema": "focusa.spec135.mission_canvas_completion_dag.v2",
        "version": 2,
        "status": "operator_approved_p00_execution",
        "mission": "Translate the complete unchanged Spec 135-series Mission Canvas planned as a Pi TUI overlay into the Focusa Desktop Mission Canvas Svelte GUI tab, preserving every function while enabling dynamic, vertical-moldable generated UI without improvisation.",
        "cardinal_translation_rule": {
            "id": "CARDINAL-135-SVELTE-001",
            "operator_directive": "Mission Canvas (Pi TUI overlay) becomes Focusa Desktop Mission Canvas (Svelte application GUI tab).",
            "source_authority": "The unchanged Spec 135 master and 135A-135K requirements remain the functional source; Pi-overlay wording identifies source behavior, not the destination host.",
            "destination": "apps/desktop Mission Canvas Svelte GUI tab",
            "separate_surface": "The Agent TUI tab remains the authentic PTY-backed Pi terminal and is not the Mission Canvas destination.",
            "required_translation": [
                "Translate every planned overlay model, widget, command, interaction, state, Work Rail behavior, session behavior, generated C.R.I.S.T. flow, UIAI artifact flow, and adaptive layout behavior into Svelte or an approved trusted renderer hosted by Svelte.",
                "Preserve Core ownership of eligibility, geometry resolution, authority, operations, persistence, events, identity binding, and recomposition; Svelte renders canonical projections and never becomes a second resolver.",
                "Use A2UI/Lit and trusted Focusa Custom Elements for generated UI while Svelte owns the Desktop composition host.",
                "Allow activities, profiles, vertical packs, capabilities, work surfaces, evidence, and runtime events to mold and recompose the GUI dynamically.",
                "Treat visual handoffs as examples of the adaptive visual grammar, never as fixed dashboards or substitutes for functional translation.",
            ],
            "completion_gate": "No task is complete if it only reproduces terminal presentation, static screenshots, fixture-only behavior, or a fixed Svelte screen while any cited Spec 135 behavior remains untranslated.",
        },
        "authority_precedence": AUTHORITY,
        "operator_confirmations": {
            "completion_dag_approved_by_continue_steering": True,
            "approval_scope": "P00 execution; later phases remain dependency-gated",
            "replacement_text_outranks_images_and_older_contracts_for_occupancy": True,
            "images_are_populated_examples_not_fixed_inventory": True,
            "quality_compromise_allowed": False,
            "implementation_owner": "Focusa Desktop Mission Canvas Svelte GUI tab; Pi extension remains compatibility and Agent TUI integration only",
            "required_platforms": ["macOS", "Windows", "Linux"],
            "release_path": "canonical Git/GitHub release pipeline only",
        },
        "trajectory_alignment": {
            "trajectory_id": "trajectory:project-fnv1a64:18ae08d0b81be2af:mission-canvas-complete-20260724",
            "hlt": "Bring the Focusa Mission Canvas Spec series 135 into full existence and alignment with the current Focusa Core Product and make it fully operational.",
            "mlg": "Close all 30 remaining requirements in the machine-readable Spec 135 ledger in dependency order.",
            "stg": "Complete the ready frontier RI4, V3, V4, V5, P2, P4, Q1, Q3, Q4, and E1, then advance newly unblocked slices.",
            "ready_frontier": ["SPEC135-RI4", "SPEC135-V3", "SPEC135-V4", "SPEC135-V5", "SPEC135-P2", "SPEC135-P4", "SPEC135-Q1", "SPEC135-Q3", "SPEC135-Q4", "SPEC135-E1"],
            "dependency_advancement": ["P2→P3/P5/C4", "Q1→Q2→Q6", "V3/V4/V5→V6→Alpha7→Alpha8"],
            "reconciliation_warning": "The repository ledger currently reports 73/73 verified and must be repaired to expose the canonical 30 remaining IDs before task materialization.",
        },
        "current_state_findings": STATE_FINDINGS,
        "file_reconciliation": FILE_RECONCILIATION,
        "pivot": {
            "from": "Spec 135-series behavior expressed for a Pi TUI overlay, terminal/fixed-region projections, and stale self-asserted closure",
            "to": "complete functional translation into the Focusa Desktop Mission Canvas Svelte GUI tab, with canonical adaptive projection, generated UI, vertical recomposition, and the Agent TUI retained separately",
            "preserve": [
                "canonical Focusa reducers and runtime ownership",
                "durable Work Surface/state/binding and Work Rail foundations",
                "operation registry, OpenAPI, generated TypeScript client, durable event stream",
                "A2UI/Lit renderer and Focusa trusted elements",
                "Pi interaction-mode/config/session foundations",
                "terminal projection as truthful fallback only",
            ],
            "replace_or_reprove": [
                "fixed-slot and process-local layout assumptions",
                "terminal shell as complete GUI",
                "Markdown vertical and C.R.I.S.T. representations",
                "static screenshots and handwritten proof JSON",
                "stale 73/73 verified ledger state",
            ],
        },
        "phase_count": len(PHASES),
        "task_count_excluding_gates": sum(len(tasks) for p in PHASES for _, tasks in p["waves"]),
        "node_count": len(nodes),
        "edge_count": len(edges),
        "phases": [{"id": p["id"], "title": p["title"], "purpose": p["purpose"], "requirement_refs": p["requirement_refs"]} for p in PHASES],
        "nodes": nodes,
        "edges": edges,
        "critical_path": [f"{p['id']}-GATE" for p in PHASES],
        "implementation_start_gate": "P00-GATE",
        "first_production_dependency": "P02-W01-T01 after P00 and P01 operator/evidence gates",
        "final_gate": f"{PHASES[-1]['id']}-GATE",
    }
    normalized = json.dumps(graph, sort_keys=True, separators=(",", ":")).encode()
    graph["graph_digest_sha256"] = hashlib.sha256(normalized).hexdigest()
    return graph


def validate(graph: dict[str, Any]) -> None:
    nodes = graph["nodes"]
    ids = [n["id"] for n in nodes]
    assert len(ids) == len(set(ids)), "duplicate node id"
    known = set(ids)
    for node in nodes:
        assert node["title"] and node["expected_result"]
        for dep in node["depends_on"]:
            assert dep in known, f"unknown dependency {dep} for {node['id']}"
    adjacency: dict[str, list[str]] = {i: [] for i in ids}
    indegree = {i: 0 for i in ids}
    for edge in graph["edges"]:
        assert edge["from"] in known and edge["to"] in known
        adjacency[edge["from"]].append(edge["to"])
        indegree[edge["to"]] += 1
    queue = [i for i, degree in indegree.items() if degree == 0]
    visited = 0
    while queue:
        current = queue.pop()
        visited += 1
        for child in adjacency[current]:
            indegree[child] -= 1
            if indegree[child] == 0:
                queue.append(child)
    assert visited == len(ids), "dependency graph contains a cycle"
    assert graph["task_count_excluding_gates"] >= 200, "graph is not granular enough"
    assert graph["operator_confirmations"]["quality_compromise_allowed"] is False


def render_report(graph: dict[str, Any]) -> str:
    lines = [
        "# Spec 135 Mission Canvas Completion Pivot Plan",
        "",
        "**Status:** Operator approved P00 execution; later phases remain dependency-gated",
        f"**Graph:** `{GRAPH_PATH.relative_to(ROOT)}`",
        f"**Graph digest:** `{graph['graph_digest_sha256']}`",
        f"**Granularity:** {graph['task_count_excluding_gates']} implementation tasks, {graph['node_count']} total nodes including gates, {graph['edge_count']} dependency edges",
        "",
        "## 1. Necessary pivot",
        "",
        "Apply `CARDINAL-135-SVELTE-001`: the unchanged Spec 135 master and 135A-135K remain the complete functional source originally expressed for a Pi TUI overlay, while the production destination is the Focusa Desktop Mission Canvas Svelte GUI tab. Translate every planned model, widget, command, interaction, state, Work Rail behavior, generated C.R.I.S.T. flow, UIAI artifact flow and adaptive layout behavior. The Agent TUI remains a separate authentic PTY-backed Pi terminal tab. Core resolves eligibility and geometry into `ResolvedWorkspaceProjection`; Svelte and approved A2UI/Lit or trusted Custom Element renderers render it without inventing a second resolver or fixed dashboard.",
        "",
        "## 2. Current-state evaluation",
        "",
        "| ID | Classification | Finding | Evidence |",
        "|---|---|---|---|",
    ]
    for finding in graph["current_state_findings"]:
        lines.append(f"| {finding['id']} | `{finding['classification']}` | {finding['finding']} | {'<br>'.join(finding['evidence'])} |")
    trajectory = graph["trajectory_alignment"]
    lines += [
        "",
        "## 3. Trajectory alignment", "",
        f"- **Trajectory:** `{trajectory['trajectory_id']}`",
        f"- **HLT:** {trajectory['hlt']}",
        f"- **MLG:** {trajectory['mlg']}",
        f"- **STG:** {trajectory['stg']}",
        f"- **Ready frontier:** `{', '.join(trajectory['ready_frontier'])}`",
        f"- **Dependency advancement:** `{'; '.join(trajectory['dependency_advancement'])}`",
        f"- **Reconciliation warning:** {trajectory['reconciliation_warning']}",
        "",
        "## 4. File-by-file reconciliation", "",
        "| Path | Current classification | Preserve | Pivot |",
        "|---|---|---|---|",
    ]
    for item in graph["file_reconciliation"]:
        lines.append(f"| `{item['path']}` | `{item['state']}` | {item['preserve']} | {item['pivot']} |")
    lines += [
        "",
        "## 5. Preserve, replace, and re-prove",
        "",
        "### Preserve",
    ]
    lines += [f"- {item}" for item in graph["pivot"]["preserve"]]
    lines += ["", "### Replace or re-prove"]
    lines += [f"- {item}" for item in graph["pivot"]["replace_or_reprove"]]
    lines += ["", "## 6. Phase dependency chain", ""]
    for p in graph["phases"]:
        lines.append(f"### {p['id']} — {p['title']}")
        lines.append("")
        lines.append(p["purpose"])
        lines.append("")
        lines.append(f"Requirement refs: `{', '.join(p['requirement_refs'])}`")
        lines.append("")
        lines.append("| Seq | Node | Wave | Task | Depends on | Proof | Expected result | Targets |")
        lines.append("|---:|---|---|---|---|---|---|---|")
        for node in [n for n in graph["nodes"] if n["phase"] == p["id"]]:
            lines.append(
                f"| {node['sequence']} | `{node['id']}` | {node['wave']} | {node['title']} | "
                f"{', '.join(f'`{d}`' for d in node['depends_on']) or '—'} | `{node['proof_class']}` | "
                f"{node['expected_result']} | {'<br>'.join(f'`{t}`' for t in node['targets']) or '—'} |"
            )
        lines.append("")
    lines += [
        "## 7. Execution law",
        "",
        "1. No phase starts before its incoming phase gate passes.",
        "2. Tasks inside a wave may run in parallel only with isolated writer scopes and explicit Attachments.",
        "3. A wave gate requires implementation evidence, tests, and drift review for every task in that wave.",
        "4. Static files, screenshots, source substrings and handwritten pass JSON never satisfy runtime Svelte Mission Canvas gates.",
        "5. The primary workspace never renders dead chrome, empty placeholders, disabled discovery controls, or semantically substituted filler.",
        "6. macOS, Windows and Linux remain first-class from host packaging through final installed-release verification.",
        "7. PR 110 remains draft until the final strict closure gate and all required GitHub checks pass.",
        "",
        "## 8. Immediate next move after approval",
        "",
        "Execute P00 only: repair task-provider visibility, reconcile the remaining-requirement truth, fix the M4 baseline, classify current files, and freeze the corrected authority/ledger state. Do not begin Svelte Mission Canvas UI styling or component construction until P01 and P02 contracts pass.",
        "",
    ]
    return "\n".join(lines)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    graph = build_graph()
    validate(graph)
    graph_text = json.dumps(graph, indent=2, ensure_ascii=False) + "\n"
    report_text = render_report(graph)
    if args.check:
        assert GRAPH_PATH.read_text() == graph_text, f"stale generated graph: {GRAPH_PATH}"
        assert REPORT_PATH.read_text() == report_text, f"stale generated report: {REPORT_PATH}"
        print(f"Spec 135 completion DAG: PASS ({graph['task_count_excluding_gates']} tasks, {graph['node_count']} nodes, {graph['edge_count']} edges)")
        return
    GRAPH_PATH.write_text(graph_text)
    REPORT_PATH.write_text(report_text)
    print(f"Generated {GRAPH_PATH.relative_to(ROOT)}")
    print(f"Generated {REPORT_PATH.relative_to(ROOT)}")
    print(f"tasks={graph['task_count_excluding_gates']} nodes={graph['node_count']} edges={graph['edge_count']} digest={graph['graph_digest_sha256']}")


if __name__ == "__main__":
    main()
