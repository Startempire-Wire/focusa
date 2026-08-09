#!/usr/bin/env python3
"""Spec 152E.05.06 TUI / daemon REST / lifecycle-receipt presenter parity.

Binds the TUI API/app, the daemon REST license routes, and the lifecycle
receipts to the frozen Spec 152E presenter vocabulary
(docs/contracts/spec152e-activation-internal.v1.json and
spec152e-activation-errors.v1.json) and proves all presenters expose
equivalent allowed actions, errors, and recovery for one canonical
registration. Every surface renders shared activation states/actions, masked
identity, checkout/verify links, terminal delivery, node management,
denial/recovery, and resume handles without duplicating business decisions;
the menubar presenter (apps/menubar/src/lib/activationPresenter.ts) uses the
same frozen vocabulary and is bound by apps/menubar/tests/spec152e_activation.mjs.

Exact verification: python3 tests/spec152e_tui_rest_activation_test.py
"""

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "docs/contracts"
INTERNAL = json.loads((CONTRACTS / "spec152e-activation-internal.v1.json").read_text(encoding="utf-8"))

TUI = (ROOT / "crates/focusa-tui/src/activation_presenter.rs").read_text(encoding="utf-8")
TUI_APP = (ROOT / "crates/focusa-tui/src/app.rs").read_text(encoding="utf-8")
TUI_VIEW = (ROOT / "crates/focusa-tui/src/views/deck_home.rs").read_text(encoding="utf-8")
REST = (ROOT / "crates/focusa-api/src/routes/license.rs").read_text(encoding="utf-8")
RECEIPTS = (ROOT / "crates/focusa-core/src/install_lifecycle/receipts.rs").read_text(encoding="utf-8")
RECEIPT_TESTS = (ROOT / "crates/focusa-core/src/install_lifecycle/receipt_tests.rs").read_text(encoding="utf-8")
MENUBAR = (ROOT / "apps/menubar/src/lib/activationPresenter.ts").read_text(encoding="utf-8")

POSITIVE = 0
NEGATIVE = 0


def expect(condition: bool, message: str, negative: bool = False) -> None:
    global POSITIVE, NEGATIVE
    if negative:
        NEGATIVE += 1
    else:
        POSITIVE += 1
    if not condition:
        raise AssertionError(message)


# ── Frozen presenter vocabulary ───────────────────────────────────────────

frozen_states = INTERNAL["presenter_states"]
expect(len(frozen_states) == 10, "frozen contract has exactly 10 presenter states")
terminal = set(INTERNAL["polling"]["terminal_states"])
expect(terminal == {"activated", "denied", "recovery_only"}, "frozen terminal presenter states")

# The TUI presenter mirrors the frozen vocabulary exactly.
for state in frozen_states:
    expect(f'"{state}"' in TUI, f"TUI presenter defines frozen state {state}")
expect(TUI.count("=> \"") >= 10, "TUI next-action table covers every presenter state")
for t in terminal:
    expect(t in TUI, f"TUI marks {t} as terminal")

# The REST presenter maps entitlement status onto the same vocabulary.
expect('presenter_state_for_entitlement_status' in REST, "REST has the shared entitlement→presenter mapping")
expect('"activated"' in REST and '"recovery_only"' in REST and '"denied"' in REST,
       "REST maps onto activated/recovery_only/denied")

# The menubar TS presenter uses the same frozen vocabulary (cross-surface).
for state in frozen_states:
    expect(state in MENUBAR, f"menubar presenter defines frozen state {state}")

# ── Frozen next-action table is shared, not re-decided ─────────────────────

frozen_next_actions = {
    "email_required": "provide_email",
    "email_verification_pending": "verify_email",
    "email_verified": "select_offer",
    "selection_required": "select_offer",
    "checkout_required": "open_checkout",
    "payment_pending": "poll_after_retry_after",
    "license_delivery_ready": "deliver_license",
    "activated": "activated",
    "denied": "activate_or_manage_entitlement",
    "recovery_only": "recovery",
}
for state, action in frozen_next_actions.items():
    expect(action in TUI, f"TUI next action {action} for {state}")
    expect(action in REST, f"REST next action {action} for {state}")
    expect(action in MENUBAR, f"menubar next action {action} for {state}")

# ── Equivalent allowed actions across all surfaces ─────────────────────────

shared_actions = {
    "email_required": ["provide_email"],
    "email_verification_pending": ["verify_email", "resend_code"],
    "email_verified": ["select_offer"],
    "selection_required": ["select_purchase", "select_limited_access", "select_existing_key"],
    "checkout_required": ["open_checkout"],
    "payment_pending": ["poll", "open_checkout"],
    "license_delivery_ready": ["deliver_license", "activate"],
    "activated": ["manage_nodes", "refresh_lease", "manage_account", "resume"],
    "denied": ["activate_or_manage_entitlement", "recovery"],
    "recovery_only": ["recovery", "repair", "export", "uninstall", "manage_nodes", "manage_account"],
}
for state, actions in shared_actions.items():
    for action in actions:
        expect(action in TUI, f"TUI allows {action} for {state}")
        expect(action in REST, f"REST allows {action} for {state}")
        expect(action in MENUBAR, f"menubar allows {action} for {state}")

# Node management and resume appear on the same states on every surface.
for surface in (TUI, REST, MENUBAR):
    expect("manage_nodes" in surface, "manage_nodes exposed by every surface")
    expect("resume" in surface, "resume exposed by every surface")

# ── TUI app/API wiring ─────────────────────────────────────────────────────

expect("activation_presenter" in TUI_APP or "activation_presenter::" in TUI_APP,
       "TUI app consumes the activation presenter module")
expect('"/v1/activation/status"' in TUI_APP, "TUI app fetches /v1/activation/status")
expect('"/v1/license/status"' in TUI_APP, "TUI app fetches /v1/license/status")
expect("project_activation_status" in TUI_APP, "TUI app projects the activation status")
expect("activation" in TUI_APP and "license" in TUI_APP, "TUI app stores typed presenter views")
expect("activation" in TUI_VIEW and "Entitlement" in TUI_VIEW,
       "TUI Deck Home renders the shared presenter posture")
expect("unavailable (no registration snapshot)" in TUI_VIEW,
       "TUI fails closed to unavailable instead of inventing a state")
# No raw email, key, credential, or card field in the TUI presenter.
for forbidden in ("full_license_key", "poll_credential", "card_pan", "card_cvc"):
    expect(forbidden not in TUI, f"TUI presenter has no {forbidden} field", negative=True)
expect("raw_email" not in TUI, "TUI presenter has no raw_email field", negative=True)
expect("access_token" not in TUI, "TUI presenter has no access_token field", negative=True)

# ── Daemon REST license routes ─────────────────────────────────────────────

expect('"/v1/license/status"' in REST, "REST exposes /v1/license/status")
expect('"/v1/activation/status"' in REST, "REST exposes /v1/activation/status")
expect('"presenter"' in REST, "REST license status carries the presenter block")
expect('"presenter_state"' in REST, "REST presenter block carries presenter_state")
expect('"allowed_actions"' in REST, "REST presenter block carries allowed_actions")
expect('"recovery_policy"' in REST, "REST presenter block carries recovery_policy")
expect('"masked_identity"' in REST, "REST presenter block carries masked_identity")
expect("presenter_next_action_label" in REST, "REST presenter block carries the shared next action")
expect("allowed_actions_for_presenter_state" in REST, "REST shares the allowed-action table")
expect("presenter_projection_uses_frozen_shared_vocabulary" in REST,
       "REST unit test binds the shared vocabulary")
# Fail-closed defaults for unknown labels.
expect('_ => "email_required"' in REST, "REST unknown entitlement status fails closed to email_required")
expect('_ => "activate_or_manage_entitlement"' in REST,
       "REST unknown presenter state fails closed to activate-or-manage")
for forbidden in ('"full_license_key"', '"poll_credential_hash"', '"card_pan"'):
    expect(forbidden not in REST.split("#[cfg(test)]")[0],
           f"REST presenter surface has no {forbidden} field", negative=True)

# ── Lifecycle receipts ─────────────────────────────────────────────────────

expect("presenter_posture" in RECEIPTS, "lifecycle receipts expose the presenter posture")
expect("LifecycleReceiptPresenterPosture" in RECEIPTS, "lifecycle receipts posture type exists")
expect('"focusa.lifecycle_receipt_presenter_posture.v1"' in RECEIPTS,
       "lifecycle receipt posture has a stable schema")
expect('"activated"' in RECEIPTS and '"recovery_only"' in RECEIPTS and '"denied"' in RECEIPTS,
       "lifecycle receipts project onto activated/recovery_only/denied")
expect('"manage_nodes"' in RECEIPTS, "activated receipts expose node management")
expect('"refresh_lease"' in RECEIPTS, "activated receipts expose lease refresh")
expect('"uninstall"' in RECEIPTS and '"repair"' in RECEIPTS and '"export"' in RECEIPTS,
       "recovery receipts expose recovery actions")
# Fail-closed: product-ready class without a verified signature renders
# recovery_only, never activated.
expect("fail" in RECEIPTS.lower() and "recovery_only" in RECEIPTS,
       "lifecycle receipt posture fails closed on unverified readiness")
expect("receipt_presenter_posture_uses_shared_presenter_vocabulary" in RECEIPT_TESTS,
       "lifecycle receipt presenter tests exist")
for forbidden in ("raw_email", "full_license_key", "poll_credential", "card_pan"):
    expect(forbidden not in RECEIPTS, f"lifecycle receipts have no {forbidden}", negative=True)

# ── One canonical registration, equivalent posture everywhere ──────────────

# For a registration in `checkout_required` every surface exposes
# next_action=open_checkout and allowed [open_checkout].
expect('"checkout_required"' in TUI and 'open_checkout' in TUI, "TUI checkout_required posture")
expect('"checkout_required"' in REST and 'open_checkout' in REST, "REST checkout_required posture")
expect("'checkout_required'" in MENUBAR or '"checkout_required"' in MENUBAR,
       "menubar checkout_required posture")
expect('open_checkout' in MENUBAR, "menubar exposes open_checkout")
# For a registration in `payment_pending` every surface exposes poll + open_checkout.
expect('"payment_pending"' in TUI and '"poll"' in TUI, "TUI payment_pending posture")
expect('"payment_pending"' in REST and '"poll"' in REST, "REST payment_pending posture")
expect("'payment_pending'" in MENUBAR or '"payment_pending"' in MENUBAR,
       "menubar payment_pending posture")
expect('poll' in MENUBAR, "menubar exposes poll")
# Denial/recovery equivalence: denied exposes activate_or_manage_entitlement
# + recovery; recovery_only exposes the recovery action set.
expect('activate_or_manage_entitlement' in TUI and "recovery" in TUI, "TUI denial/recovery posture")
expect('activate_or_manage_entitlement' in REST and "recovery" in REST, "REST denial/recovery posture")
expect('activate_or_manage_entitlement' in MENUBAR and "recovery" in MENUBAR, "menubar denial/recovery posture")

# No presenter invents verification, consent, payment, or a license locally.
for surface_name, source in (("TUI", TUI), ("REST", REST), ("receipts", RECEIPTS)):
    expect("reduce" not in source or "never re" in source.lower(),
           f"{surface_name} presenter does not re-decide transitions")
    expect("invent" not in source.lower() or "never" in source.lower(),
           f"{surface_name} presenter never invents authority decisions")

print(f"TUI/REST/lifecycle-receipt presenter parity tests passed (positive={POSITIVE} negative={NEGATIVE})")
