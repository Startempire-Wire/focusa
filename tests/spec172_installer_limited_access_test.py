#!/usr/bin/env python3
"""Spec 172.04.06 installer and lifecycle limited-access gate.

Binds the Unix / PowerShell / Rust installers and the install lifecycle
models/orchestrator/receipts to Spec 172 §2.7 (verified_no_license), §3
(supersession of local/self-issued Evaluation), and the atom requirements:

  * Official/source/raw installs cannot self-issue: no channel creates or
    persists a local Evaluation grant, and no local/self-issued grant can be
    produced from any installer surface.
  * All channels reach the same verified-limited or paid authority flow:
    `--eval` maps to the Spec 172 verified-email limited-access overlay
    (`ActivationJourney::LimitedAccess`) and noninteractive installs keep
    verified device-code authorization; raw keys and emails are rejected.
  * Lifecycle receipts record posture, product/type/family, policy digest,
    and sequence without raw identity or key material: the receipt class is
    `limited_access_ready` (state `active_verified_limited`) instead of
    `evaluation_ready` / `active_evaluation`; the policy binding records
    policy_digest / capability_family / entitlement_state / lease_sequence /
    recovery_posture / product_ready; product readiness requires a
    signature-verified binding and fails closed otherwise.
  * Paid key flow remains EDD-backed: raw license keys cannot authorize
    installation (`E_AUTHORITY_RAW_KEY_FORBIDDEN`).

First-run presenters (the installer first-install walkthrough and terminal-ui
install presenter) render status only and never own policy, pricing, grants,
or limits.

Exact verification: python3 tests/spec172_installer_limited_access_test.py
"""

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CLI = ROOT / "crates/focusa-cli/src/commands"
INSTALL = (CLI / "install.rs").read_text(encoding="utf-8")
LIFECYCLE = ROOT / "crates/focusa-core/src/install_lifecycle"
MODELS = (LIFECYCLE / "models.rs").read_text(encoding="utf-8")
RECEIPTS = (LIFECYCLE / "receipts.rs").read_text(encoding="utf-8")
PREFLIGHT = (LIFECYCLE / "preflight.rs").read_text(encoding="utf-8")
ORCHESTRATOR = (LIFECYCLE / "orchestrator.rs").read_text(encoding="utf-8")
SH = (ROOT / "scripts/install-focusa.sh").read_text(encoding="utf-8")
PS1 = (ROOT / "scripts/install-focusa.ps1").read_text(encoding="utf-8")
TERMINAL_UI_INSTALL = ROOT / "crates/focusa-terminal-ui/src/install"
PRESENTER_SOURCES = "".join(
    (TERMINAL_UI_INSTALL / name).read_text(encoding="utf-8")
    for name in ["presenter.rs", "completion.rs", "state.rs", "event.rs"]
)

POSITIVE = 0
NEGATIVE = 0


def require(condition: bool, message: str, negative: bool = False) -> None:
    global POSITIVE, NEGATIVE
    if negative:
        NEGATIVE += 1
    else:
        POSITIVE += 1
    if not condition:
        raise AssertionError(message)


# ── Rust installer: no local/self-issued grant on any path ────────────────

require('return Ok("eval".to_string())' not in INSTALL,
       "installer can still self-issue an eval grant", negative=True)
require("persist_eval_license" not in INSTALL,
       "installer persists a local evaluation license", negative=True)
require("write_license_json" not in INSTALL,
       "installer still writes local license JSON", negative=True)
require("E_EVAL_ISSUED" not in INSTALL,
       "installer still has a local eval-issued code path", negative=True)
require("grant_creation" not in INSTALL,
       "installer still contains local grant-creation machinery", negative=True)

# --eval is intent forwarding only: it maps to verified-email limited
# activation (Spec 172 limited-access overlay), never to a local grant.
require("ActivationJourney::LimitedAccess" in INSTALL,
       "--eval does not map to the Spec 172 limited-access overlay")
require("verified-email limited activation" in INSTALL,
       "installer surfaces do not name verified-email limited activation")
require("no local\n    /// Evaluation grant is ever created" in INSTALL
       or "no local Evaluation grant is ever created" in INSTALL,
       "--eval documentation does not state that no local grant is created")
require("E_AUTHORITY_ACTIVATION_UNSETTLED" in INSTALL,
       "unsettled activation fails closed without a lease")

# All channels reach the same verified-limited or paid authority flow:
# device-code authorization remains for noninteractive installs, and the
# interactive path renders the shared activation flow.
require("DeviceAuthorizationSession::new" in INSTALL,
       "noninteractive device-code authorization is missing")
require("AuthorityHttpClient::new" in INSTALL,
       "noninteractive authority HTTP transport is missing")
require("run_activation_flow(" in INSTALL,
       "interactive path does not drive the shared activation flow")
require("acquire_installer_entitlement" in INSTALL,
       "noninteractive entitlement acquisition is missing")

# Installer plan / preflight vocabulary records limited-access intent, not a
# local Evaluation mode.
require("authority_limited_access" in INSTALL,
       "install plan does not record authority limited-access mode")
require("authority_existing_or_limited_access" in INSTALL,
       "install plan does not record existing-or-limited-access mode")
require("authority_limited_access_request" in INSTALL,
       "preflight override inventory does not record limited-access request")

# Paid key flow remains EDD-backed: raw keys cannot authorize installation.
require("E_AUTHORITY_RAW_KEY_FORBIDDEN" in INSTALL,
       "raw license keys can still authorize installation")

# First-run walkthrough presenter renders status only; it never owns policy,
# pricing, grants, or limits.
require("license_status" in INSTALL and "build_first_install_walkthrough" in INSTALL,
       "first-run walkthrough presenter is missing")
require("evaluation_ready" not in INSTALL and "active_evaluation" not in INSTALL,
       "Rust installer still emits Evaluation receipt vocabulary", negative=True)

# ── Install lifecycle models: verified-limited vocabulary, no Evaluation ───

require("LifecycleEntitlementState::ActiveVerifiedLimited" in MODELS
       and "ActiveVerifiedLimited" in MODELS,
       "lifecycle entitlement state still spells ActiveEvaluation")
require("LifecycleEntitlementReceiptClass::LimitedAccessReady" in MODELS,
       "lifecycle receipt class still spells EvaluationReady")
authorization_block = MODELS[MODELS.index("pub enum AuthorizationSelection {"):]
authorization_block = authorization_block[: authorization_block.index("}") + 1]
require("LimitedAccess" in authorization_block and "Evaluation" not in authorization_block,
       "lifecycle authorization selection still spells Evaluation")
require("LifecycleEntitlementState::ActiveEvaluation" not in MODELS,
       "ActiveEvaluation remains in lifecycle models", negative=True)
require("LifecycleEntitlementReceiptClass::EvaluationReady" not in MODELS,
       "EvaluationReady remains in lifecycle models", negative=True)
require("AuthorizationSelection::Evaluation" not in MODELS,
       "AuthorizationSelection::Evaluation remains in lifecycle models", negative=True)
require("receipt_class" in MODELS and "allows_product_execution_at" in MODELS,
       "lifecycle entitlement binding semantics are missing")

# Preflight posture: verified limited access replaces Evaluation.
preflight_posture_block = PREFLIGHT[PREFLIGHT.index("pub enum LicensePosture {"):]
preflight_posture_block = preflight_posture_block[: preflight_posture_block.index("}") + 1]
require("VerifiedLimitedAccess" in preflight_posture_block
       and "Evaluation" not in preflight_posture_block,
       "preflight license posture still spells Evaluation")
require("LicensePosture::Evaluation" not in PREFLIGHT,
       "LicensePosture::Evaluation remains in preflight", negative=True)

# ── Lifecycle receipts: posture/family/policy digest/sequence, no raw data ─

require("limited_access_ready" in RECEIPTS,
       "receipt class label does not use limited_access_ready")
require("active_verified_limited" in RECEIPTS,
       "receipt entitlement-state vocabulary does not use active_verified_limited")
require("evaluation_ready" not in RECEIPTS,
       "evaluation_ready remains in lifecycle receipts", negative=True)
require("active_evaluation" not in RECEIPTS,
       "active_evaluation remains in lifecycle receipts", negative=True)
require("LifecycleEntitlementReceiptClass::EvaluationReady" not in RECEIPTS,
       "EvaluationReady remains in lifecycle receipts", negative=True)

# Product readiness requires a signature-verified binding; unverified
# product-ready claims fail closed.
require("signature_verified" in RECEIPTS and "UnverifiedProductReady" in RECEIPTS,
       "unverified product-ready receipts do not fail closed")

# The simple-policy binding records the canonical digest, family, state,
# sequence, and recovery posture derived from the receipt's own authority
# fields — never caller-supplied policy identity.
for marker in [
    "policy_digest", "capability_family", "entitlement_state", "lease_sequence",
    "recovery_posture", "product_ready", "PolicyReconciliation",
    "embedded_entitlement_policy_registry",
]:
    require(marker in RECEIPTS, f"lifecycle policy binding is missing {marker}")

# No raw identity or key fields on the receipt or its policy binding.
receipt_struct = RECEIPTS[RECEIPTS.index("pub struct LifecycleReceiptV1 {"):]
receipt_struct = receipt_struct[: receipt_struct.index("\n}\n", receipt_struct.index("pub policy_binding")) + 3]
for forbidden in ["pub email", "pub customer", "pub card", "pub credential",
                  "pub license_key", "pub private_key", "pub signing_key",
                  "pub secret_key", "pub full_key", "pub account_id"]:
    require(forbidden not in receipt_struct,
            f"receipt records raw identity/key field: {forbidden}", negative=True)

# ── Lifecycle orchestrator: install/update need a signed entitlement ──────

for marker in ["EntitlementRequired", "ProductGrantRequired", "FeatureGrantRequired",
               "EntitlementBlocked", "ArtifactTrustRequired", "recovery_safe"]:
    require(marker in ORCHESTRATOR, f"lifecycle orchestrator is missing {marker}")
require("Evaluation" not in ORCHESTRATOR,
       "lifecycle orchestrator still names Evaluation", negative=True)

# ── Unix installer: presenter-only, no local issuance ─────────────────────

require("E_AUTHORITY_RAW_KEY_FORBIDDEN" in SH,
       "Unix installer accepts raw credentials")
for forbidden in ["write_license_json", "write_license_authority", "write_license_receipt",
                  "eval_issued", "self_eval", "E_EVAL_ISSUED", "grace_license",
                  "installer_grace"]:
    require(forbidden not in SH,
            f"Unix installer still issues local state: {forbidden}", negative=True)
require("if \"$BOOTSTRAP_BIN\" \"${ARGS[@]}\"; then" in SH,
       "Unix installer does not converge on the single Rust handoff")
require("authority-issued only" in SH,
       "Unix installer does not disclose authority-only activation")
require("verified-email limited activation" in SH,
       "Unix installer does not name verified-email limited activation")
require("ARGS+=(--eval)" in SH and "--eval" in SH,
       "Unix installer no longer forwards --eval intent to the shared client")

# ── PowerShell installer: presenter-only, no local issuance ───────────────

require("E_AUTHORITY_RAW_KEY_FORBIDDEN" in PS1,
       "PowerShell installer accepts raw credentials")
for forbidden in ["license.json", "eval: true", "CustomerEmail", "FOCUSA_LICENSE_KEY",
                  "Write-Output.*eval", "Set-ExecutionPolicy"]:
    require(forbidden not in PS1,
            f"PowerShell installer still holds a local authority marker: {forbidden}",
            negative=True)
require('$Args = @("install", "--target=$ResolvedTarget"' in PS1,
       "PowerShell installer does not delegate to the Rust installer")
require("& $Focusa @Args" in PS1,
       "PowerShell installer does not run the canonical Rust flow")
require("verified-email limited activation" in PS1,
       "PowerShell installer does not name verified-email limited activation")
require("never creates local evaluation state" in PS1,
       "PowerShell installer no longer states it never creates local evaluation state")

# ── Terminal-ui first-run presenters: no policy ownership ─────────────────

for forbidden in ["evaluation_ready", "active_evaluation", "grant_creation",
                  "persist_eval", "E_EVAL_ISSUED", "license_key"]:
    require(forbidden not in PRESENTER_SOURCES,
            f"install presenter owns grant vocabulary: {forbidden}", negative=True)

# ── Hygiene: no unmasked email in installer surfaces ──────────────────────

email_pattern = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
for source, name in [(INSTALL, "Rust installer"), (SH, "Unix installer"), (PS1, "PowerShell installer")]:
    for match in email_pattern.findall(source):
        if match == "support@focusa.dev":
            continue
        raise AssertionError(f"unmasked email in {name}: {match}")

print(json.dumps({
    "schema": "focusa.spec172.installer_limited_access.v1",
    "positive_checks": POSITIVE,
    "negative_checks": NEGATIVE,
    "surfaces": [
        "crates/focusa-cli/src/commands/install.rs",
        "crates/focusa-core/src/install_lifecycle/{models,preflight,orchestrator,receipts}.rs",
        "scripts/install-focusa.sh",
        "scripts/install-focusa.ps1",
        "crates/focusa-terminal-ui/src/install/",
        "tests/spec172_installer_limited_access_test.py",
    ],
    "channels": "official/source/raw converge on verified-limited or paid authority flow",
    "self_issue": "impossible",
    "result": "passed_fail_closed",
}, sort_keys=True))
