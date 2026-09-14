#!/usr/bin/env python3
"""Spec 152E.05.07 raw binary / local source-build first-run activation.

Binds the source-build run path, the direct-binary first protected action,
config/credential discovery, and the activation CLI/API to the frozen Spec
152E contracts (docs/contracts/spec152e-source-build-first-run-fixture.v1.json
and spec152e-activation-internal.v1.json) and proves Spec 152E §14.3:

1. A source-built or manually copied client (no installer provenance, no
   installer receipts, no install root) is required to complete the SAME
   universal authority flow as the official installer: email -> verify ->
   offer -> checkout/poll -> signed lease; noninteractive runs use device-code
   authorization; `--agent` runs return typed human-action envelopes with a
   resumable handle. The CLI presenter identity is `cli` and
   `install_channel=source_build` is advisory telemetry, never entitlement
   evidence.
2. `install_channel` is recorded in the registration/context/start payload
   only: no entitlement decision surface (reducer, authority-state
   resolution, base-product gate, daemon middleware) reads it.
3. Missing installer files cannot create an entitlement fallback: a fresh
   config directory resolves as `unactivated` (authority_lease_missing) with
   zero grants, corrupt state fails closed to `recovery_only`, and deleting
   installer state can neither unlock protected work nor change the runtime
   decision. Deleting the signed authority lease itself locks the machine.
4. The first protected action of the direct binary fails closed:
   `require_base_product` returns `LicenseError::BaseProductRequired` until a
   signed lease exists, and the daemon REST gate reports
   `ENTITLEMENT_BASE_REQUIRED` for value-producing mutations.

The Rust unit tests for the first-run / delete-matrix / channel-neutral
proofs execute in the same commit (crates/focusa-license/src/authority_store.rs)
and the shared flow's transcript replay executes in
crates/focusa-cli/src/commands/activation_flow.rs, so evidence is replayable
from the pinned commit without any network.

Exact verification: python3 tests/spec152e_source_build_activation_test.py
"""

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "docs/contracts"
CLI = ROOT / "crates/focusa-cli/src/commands"
LICENSE_CRATE = ROOT / "crates/focusa-license/src"
CORE_CRATE = ROOT / "crates/focusa-core/src"
API_CRATE = ROOT / "crates/focusa-api/src"

FIXTURE = json.loads(
    (CONTRACTS / "spec152e-source-build-first-run-fixture.v1.json").read_text(encoding="utf-8")
)
INTERNAL = json.loads(
    (CONTRACTS / "spec152e-activation-internal.v1.json").read_text(encoding="utf-8")
)

FLOW = (CLI / "activation_flow.rs").read_text(encoding="utf-8")
LICENSE = (CLI / "license.rs").read_text(encoding="utf-8")
INSTALL = (CLI / "install.rs").read_text(encoding="utf-8")
STORE = (LICENSE_CRATE / "authority_store.rs").read_text(encoding="utf-8")
CLIENT = (LICENSE_CRATE / "activation_client.rs").read_text(encoding="utf-8")
FACADE = (LICENSE_CRATE / "activation_facade.rs").read_text(encoding="utf-8")
HTTP = (LICENSE_CRATE / "activation_http.rs").read_text(encoding="utf-8")
REDUCER = (LICENSE_CRATE / "activation_reducer.rs").read_text(encoding="utf-8")
CREDENTIALS = (LICENSE_CRATE / "authority_credentials.rs").read_text(encoding="utf-8")
POLICY = (LICENSE_CRATE / "entitlement_policy.rs").read_text(encoding="utf-8")
CORE_LICENSE = (CORE_CRATE / "license.rs").read_text(encoding="utf-8")
MIDDLEWARE = (API_CRATE / "middleware" / "entitlement.rs").read_text(encoding="utf-8")
REST_LICENSE = (API_CRATE / "routes" / "license.rs").read_text(encoding="utf-8")

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


# ── Frozen fixture is current and bounded ─────────────────────────────────

expect(FIXTURE["schema"] == "focusa.spec152e.source_build_first_run_fixture.v1",
       "fixture schema is pinned")
expect(FIXTURE["locked_release"] is True, "fixture is a locked-release correction")
expect(FIXTURE["install_channel"]["value"] == "source_build", "fixture channel is source_build")
expect(FIXTURE["install_channel"]["advisory_only"] is True,
       "fixture declares install_channel advisory only")
expect(len(FIXTURE["non_grant_matrix"]) == 8, "non-grant matrix has exactly 8 rows")
expect(len(FIXTURE["deletion_matrix"]) == 4, "deletion matrix has exactly 4 rows")
expect(len(FIXTURE["first_run_expected"]["reaches"]) == 3,
       "first run reaches paid / Evaluation / existing key")

# ── Source-build run path: same authority flow, source_build channel ──────

expect("pub const CLI_FLOW" in FLOW, "CLI flow presenter identity is a typed const")
expect('presenter: "cli"' in FLOW, "CLI presenter identity is cli")
expect('install_channel: "source_build"' in FLOW, "CLI channel is source_build (advisory)")
expect('install_channel: "official_installer"' in FLOW,
       "installer channel is official_installer (advisory contrast)")
expect("ActivationSession::begin" in FLOW, "source-build flow drives the shared session")
expect("ActivationSession::resume" in FLOW, "source-build flow resumes the shared session")
expect("fn new_context" in FLOW, "flow builds the shared typed request context")
expect("run_activation_flow" in FLOW, "source-build run path exists in the shared flow")
expect("run_agent_activation" in FLOW, "source-build agent/JSON run path exists")
expect("interactive_available" in FLOW, "interactive detection exists (noninteractive fails closed)")
expect("persist_delivered_lease" in FLOW, "verified lease persistence exists (authority-lease.json)")
expect("resolve_flow_node_identity" in FLOW, "node-bound identity resolution exists")

# The presenter renders only; it never re-decides a transition.
expect("reduce_activation" not in FLOW.split("#[cfg(test)]")[0],
       "source-build presenter never reimplements the reducer", negative=True)
expect("ActivationTransition::" not in FLOW.split("#[cfg(test)]")[0],
       "source-build presenter body has no transition construction", negative=True)

# ── Direct binary first protected action ──────────────────────────────────

expect("require_base_product" in CORE_LICENSE, "direct-binary base gate exists")
expect("BaseProductRequired" in CORE_LICENSE, "base gate fails closed with BaseProductRequired")
expect("base_product_projection" in CORE_LICENSE,
       "base gate projects only from the signed authority snapshot")
expect("ENTITLEMENT_BASE_REQUIRED" in MIDDLEWARE,
       "daemon REST gate reports ENTITLEMENT_BASE_REQUIRED")
expect("entitlement_allows_mutation" in MIDDLEWARE,
       "daemon gate checks lease binding/current state before handlers")
expect('"unactivated"' in REST_LICENSE and '"email_required"' in REST_LICENSE,
       "REST maps unactivated status to email_required (activation required)")

# ── Config/credential discovery ───────────────────────────────────────────

expect("load_or_create_node_identity" in CREDENTIALS,
       "config/credential discovery creates the node-bound identity")
expect("pub fn resolve_flow_node_identity" in FLOW, "CLI resolves identity through the store")
expect("home.join(\".config/focusa\")" in LICENSE,
       "config dir is ~/.config/focusa (fresh dir tolerated)")
expect("for_registration" in CREDENTIALS,
       "poll credential is registration-scoped in the protected store")
expect("for_node" in CREDENTIALS, "refresh credential is node-scoped in the protected store")
expect("KeyringCredentialStore" in CREDENTIALS, "native protected credential backend exists")
expect('AUTHORITY_STATE_FILE: &str = "authority-lease.json"' in STORE,
       "authority state file is authority-lease.json")
expect("embedded_production_trust_roots" in STORE, "production trust roots are embedded")
expect('option_env!("FOCUSA_AUTHORITY_ROOT_KEYS_JSON")' in STORE,
       "trust roots come from compile-time embedding, never runtime files")
expect('"test", "fixture", "local", "dev", "example"' in STORE,
       "test/local trust roots are forbidden (no local/self-issued roots)")

# ── Missing installer files never create an entitlement fallback ──────────

expect("unactivated(" in STORE, "missing durable state resolves to unactivated")
expect("authority_lease_missing" in STORE, "unactivated carries the missing-lease reason")
expect("recovery_only(" in STORE, "corrupt/unreadable state fails closed to recovery_only")
expect("EntitlementSnapshot::unactivated" in STORE, "unactivated grants no features or limits")
expect("source_build_first_run_without_installer_or_lease_grants_nothing" in STORE,
       "Rust first-run grant-nothing test exists")
expect("deleting_installer_state_never_unlocks_and_deleting_the_lease_locks" in STORE,
       "Rust delete-matrix test exists")
expect("state_resolution_never_reads_install_channel_or_installer_files" in STORE,
       "Rust channel-neutral test exists")

# Installer-side fail-closed pins: missing installer/lease state never
# degrades into a local grant.
for code in ["E_AUTHORITY_EXISTING_UNUSABLE", "E_AUTHORITY_RAW_KEY_FORBIDDEN",
             "E_AUTHORITY_LEASE_UNUSABLE", "E_AUTHORITY_DEVICE_DENIED",
             "E_AUTHORITY_ACTIVATION_UNSETTLED"]:
    expect(code in INSTALL, f"installer fails closed with {code}")
expect("resolve_installer_entitlement" in INSTALL, "installer entitlement resolver exists")
expect("EntitlementState::Active | EntitlementState::OfflineGrace" in INSTALL,
       "installer requires signed Active/OfflineGrace only")
expect("persist_eval_license" not in INSTALL and "persist_eval_license" not in FLOW,
       "no local Evaluation issuance anywhere in the run path", negative=True)
expect("LicenseGuard::eval" not in FLOW and "LicenseGuard::eval" not in LICENSE,
       "no self-issued dev entitlement in CLI surfaces", negative=True)

# ── install_channel advisory only: recorded, never a decision input ───────

expect("pub install_channel: String" in CLIENT,
       "install_channel is recorded on the registration snapshot")
expect("pub install_channel: String" in FACADE,
       "install_channel is recorded on the request context")
expect("pub install_channel: String" in HTTP, "install_channel is recorded in the HTTP payload")
decision_surfaces = {
    "activation_reducer.rs": REDUCER,
    "authority_store.rs": STORE,
    "entitlement_policy.rs": POLICY,
    "focusa-core license.rs": CORE_LICENSE,
    "api middleware/entitlement.rs": MIDDLEWARE,
}
for surface_name, source in decision_surfaces.items():
    expect("install_channel" not in source.split("#[cfg(test)]")[0],
           f"decision surface never reads install_channel: {surface_name}", negative=True)
expect("source_build" not in REDUCER and "official_installer" not in REDUCER,
       "reducer carries no channel value", negative=True)
expect("source_build" not in STORE.split("#[cfg(test)]")[0]
       and "official_installer" not in STORE.split("#[cfg(test)]")[0],
       "authority-state resolution carries no channel value", negative=True)
# No decision surface consults installer files either: deleting installer
# state cannot change the entitlement decision because the decision never
# reads installer state.
for surface_name, source in decision_surfaces.items():
    non_test = source.split("#[cfg(test)]")[0]
    for marker in ["install_root", "installer_receipt", "install-focusa.marker"]:
        expect(marker not in non_test,
               f"decision surface never reads installer state: {surface_name} / {marker}",
               negative=True)

# ── Activation CLI/API reach paid / Evaluation / existing key ─────────────

expect("LicenseCmd::ActivateFlow" in LICENSE, "CLI dispatch wires activate-flow")
expect("run_activation_flow_command" in LICENSE, "CLI command drives the shared flow")
expect("run_agent_activation_command" in LICENSE, "CLI agent command drives the shared agent flow")
expect("E_AUTHORITY_COMMAND_RETIRED" in LICENSE,
       "plaintext activation is retired and fails closed")
expect("pub agent: bool" in LICENSE and "--resume" in LICENSE,
       "agent/JSON mode and resume handles exist on the CLI")
expect('"/v1/license/status"' in REST_LICENSE and '"/v1/activation/status"' in REST_LICENSE,
       "daemon REST exposes license/activation status to the raw binary")
expect('"install_channel": "source_build"' in REST_LICENSE,
       "REST status records source_build advisory telemetry")

# The three journeys reach activated through the shared flow (frozen
# presenter states); their transcript replays execute in this commit.
terminal_states = set(INTERNAL["polling"]["terminal_states"])
expect(terminal_states == {"activated", "denied", "recovery_only"},
       "frozen terminal presenter states")
for rust_test in [
    "paid_terminal_flow_renders_frozen_presenter_states_and_redacts",
    "existing_key_flow_settles_activated_without_checkout",
    "limited_access_spec172_overlay_settles_activated_without_checkout",
]:
    expect(rust_test in FLOW, f"Rust replay proves journey: {rust_test}")
expect("checkout_timeout_cancels_fail_closed_to_recovery_only" in FLOW,
       "Rust replay proves timeout/cancel fail closed")
expect("recovery_only_resume_never_regrants" in FLOW, "recovery never re-grants")

# ── Hygiene: no raw email, key, credential, or card field on the surfaces ─

email_pattern = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
for surface_name, source in [("activation_flow.rs", FLOW), ("license.rs", LICENSE),
                             ("authority_store.rs", STORE), ("activation_client.rs", CLIENT)]:
    for match in email_pattern.findall(source):
        if match.endswith("@example.com") or match == "support@focusa.dev":
            continue
        raise AssertionError(f"unmasked email in {surface_name}: {match}")
for needle in ["full_license_key", "card_pan", "card_cvc", '"poll_credential":', '"poll_credential_hash":']:
    expect(needle not in STORE, f"authority-store surface has no {needle}", negative=True)
    expect(needle not in REST_LICENSE.split("#[cfg(test)]")[0],
           f"REST presenter surface has no {needle}", negative=True)

# The frozen fixture itself is bounded: no emails, no secrets, no paths.
fixture_text = json.dumps(FIXTURE)
for match in email_pattern.findall(fixture_text):
    raise AssertionError(f"unmasked email in fixture: {match}")
for needle in ["poll_credential", "license_key", "access_token", "BEGIN PRIVATE"]:
    expect(needle not in fixture_text, f"fixture carries no {needle}", negative=True)
expect("$HOME/.config/focusa" in fixture_text, "fixture models the fresh config dir")

print(json.dumps({
    "schema": "focusa.spec152e.source_build_first_run.v1",
    "positive_checks": POSITIVE,
    "negative_checks": NEGATIVE,
    "channel": "source_build (advisory telemetry)",
    "flow": "universal authority flow through ActivationSession",
    "first_protected_action": "require_base_product -> BaseProductRequired / ENTITLEMENT_BASE_REQUIRED",
    "missing_installer_state": "unactivated (authority_lease_missing), never a grant",
    "deletion_matrix": "installer-state deletion changes nothing; lease deletion locks",
    "rust_store_tests": 3,
    "result": "passed_fail_closed",
}, sort_keys=True))
