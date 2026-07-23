#!/usr/bin/env python3
"""Validate F10 trusted Focusa Svelte Custom Elements and A2UI registration."""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ELEMENTS = ROOT / "packages/focusa-elements"
RENDERER = ROOT / "packages/a2ui-renderer"
manifest = json.loads((ELEMENTS / "src/component-manifest.json").read_text())
package = json.loads((ELEMENTS / "package.json").read_text())
lock_text = (ELEMENTS / "package-lock.json").read_text().lower()
generator = (ELEMENTS / "scripts/generate-elements.mjs").read_text()
component = (ELEMENTS / "src/TrustedComponent.svelte").read_text()
renderer = (RENDERER / "src/index.ts").read_text()
catalog_source = (RENDERER / "src/focusa-catalog.ts").read_text()
renderer_test = (RENDERER / "tests/renderer.test.mjs").read_text()
contract = json.loads((ROOT / "docs/contracts/spec135/generated-contract-v1/a2ui-catalog.json").read_text())
capability_source = (ROOT / "crates/focusa-api/src/routes/agent_capabilities.rs").read_text()
capability_fixture = json.loads((ROOT / "docs/contracts/spec135/generated-contract-v1/ui-capability-snapshot.fixture.json").read_text())

required = {
    "FocusaStageShell", "FocusaProgressStepper", "FocusaPrimaryAction",
    "FocusaNextStepCard", "FocusaSourceConnectorCard", "FocusaDropzone",
    "FocusaImportScopePreview", "FocusaContextSummary", "FocusaContextClaimReview",
    "FocusaContradictionCard", "FocusaRoleSeed", "FocusaRoleDraft", "FocusaRedline",
    "FocusaGroundingSources", "FocusaQuestionCard", "FocusaRecommendationCard",
    "FocusaAnswerInput", "FocusaInterviewBranchProgress", "FocusaReadinessMeter",
    "FocusaSpecSectionStatus", "FocusaObjectionCard", "FocusaApprovalCard",
    "FocusaTaskPlan", "FocusaDependencyGraph", "FocusaProviderCapabilityCard",
    "FocusaWorkpointLaunch", "FocusaEvidenceSummary", "FocusaReceiptCard",
    "FocusaRecoveryCard", "FocusaAdvancedDetails", "FocusaHelpPopover",
}
assert len(manifest) == 31
assert {item["name"] for item in manifest} == required
assert len({item["tag"] for item in manifest}) == 31
assert all(item["tag"].startswith("focusa-") for item in manifest)

assert package["dependencies"] == {"svelte": "5.55.9"}
assert package["devDependencies"]["vitest"] == "3.2.4"
assert package["devDependencies"]["@testing-library/svelte"] == "5.2.9"
assert "playwright" not in lock_text
assert 'customElement="${component.tag}"' in generator
assert "src/generated/" in (ELEMENTS / ".gitignore").read_text()

for marker in (
    'role={kind === "recovery" ? "alert" : undefined}', 'role="progressbar"',
    'aria-live="polite"', 'aria-busy={busy}', 'class="primary"',
    "prefers-reduced-motion", "prefers-contrast", "@container",
    "data-terminal-fallback", "Advanced details", "focusa-action",
):
    assert marker in component
for forbidden in ("fetch(", "localStorage", "sessionStorage", "permission", "reducer"):
    assert forbidden not in component

for marker in (
    "new Catalog<LitComponentApi>", "...basicCatalog.components.values()",
    "...focusaA2uiComponents", "A2uiController", "ActionSchema",
):
    assert marker in catalog_source
for marker in (
    "allowedActionNames", "dispatchAction", "#renderUnsupported",
    "FocusaRecoveryCard", "No action was executed", "#withRecoveryFallback",
):
    assert marker in renderer
assert "unknown components and actions fail closed with explicit recovery" in renderer_test

inline = {item["catalogId"]: item for item in contract["capabilities"]["v0.9"]["inlineCatalogs"]}
focusa = inline["https://focusa.dev/a2ui/v0_9/catalog.json"]
component_names = set(focusa["components"])
assert required <= component_names
assert len(component_names) == 49  # 18 maintained basic primitives + 31 Focusa elements
assert contract["package_lock"]["svelte"] == "5.55.9"
assert contract["package_lock"]["@focusa/elements"] == "0.9.120-dev"
assert "focusa-svelte-elements-0.9.120-dev" in capability_source
assert "focusa-svelte-elements-0.9.120-dev" in capability_fixture["client_capabilities"]

print("Spec 135 F10 Svelte Custom Elements: PASS (31 trusted elements, A2UI catalog, recovery/accessibility)")
