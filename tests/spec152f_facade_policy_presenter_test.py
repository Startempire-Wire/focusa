#!/usr/bin/env python3
"""Spec 152F.05.05 — Bind branded facades to policy explanation only.

Proves the facade policy presenter contract: every registered branded facade
descriptor/page and the Focusa website presenters display the SAME canonical
authority decision (capability family — base or one of the four premium
families — plus the Evaluation/purchase/recovery action and a safe masked
status from authority), and no facade can select grants, prices, feature
activation, or runtime policy, or turn dormant/premium fields on or off.

Surfaces covered
- registered facade descriptors/pages: docs/contracts/spec152e-facade-registry.v1.json
  (read-only cross-check) and the branded page presenter
  public/activation/focusa-facade-policy-presenter.mjs
- authority denial/recovery envelopes: the facade presenter consumes only the
  safe envelope fields (state, next_action, masked identity); it never reads
  lease, token, or credential material
- Focusa website presenters: focusa-registration.mjs / page.html remain
  presenter-only; the new policy presenter binds the same decision everywhere

Exact verification: python3 tests/spec152f_facade_policy_presenter_test.py

No cargo build, no live network, no publication.
"""

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "docs/contracts"
RUST = (ROOT / "crates/focusa-license/src/facade_policy_presenter.rs").read_text(encoding="utf-8")
LIB = (ROOT / "crates/focusa-license/src/lib.rs").read_text(encoding="utf-8")
REDUCER = (ROOT / "crates/focusa-license/src/activation_reducer.rs").read_text(encoding="utf-8")
MJS = (ROOT / "public/activation/focusa-facade-policy-presenter.mjs").read_text(encoding="utf-8")
REGISTRATION = (ROOT / "public/activation/focusa-registration.mjs").read_text(encoding="utf-8")
PAGE_HTML = (ROOT / "public/activation/page.html").read_text(encoding="utf-8")
SECURITY = (ROOT / "public/activation/focusa-facade-security.mjs").read_text(encoding="utf-8")
REGISTRY = json.loads((CONTRACTS / "spec152e-facade-registry.v1.json").read_text(encoding="utf-8"))

REGISTERED_FACADES = [row["facade_id"] for row in REGISTRY["facades"]]
EXPECTED_FACADES = [
    "focusa_install_v1",
    "focusa_marketing_v1",
    "focusa_forge_v1",
    "focusa_arena_v1",
    "uiai_engine_v1",
    "wpuiai_public_v1",
]

FACADE_PRESENTER_FIELDS = [
    "family", "posture", "action", "action_label", "explanation",
    "recovery_action", "masked_status", "always_reachable",
]

FACADE_PRESENTER_FORBIDDEN_FIELDS = [
    "grants", "prices", "price", "feature_activation", "runtime_policy",
    "dormant", "product_selection", "product_code", "limit_bucket", "limits",
    "lease", "tokens", "keys", "customer_email", "raw_status", "redirect_url",
]

POSITIVE = 0
NEGATIVE = 0


def check(condition, message, kind="positive"):
    global POSITIVE, NEGATIVE
    if not condition:
        raise AssertionError(f"FAIL ({kind}): {message}")
    if kind == "positive":
        POSITIVE += 1
    else:
        NEGATIVE += 1


def run_node(script):
    """Run one node script that imports the website policy presenter."""
    proc = subprocess.run(
        ["node", "-e", script],
        cwd=ROOT,
        capture_output=True,
        text=True,
        timeout=120,
    )
    if proc.returncode != 0:
        raise AssertionError(f"node subprocess failed: {proc.stderr.strip()}")
    return proc.stdout.strip()


IMPORT = (
    'import { projectFacadePolicyDecision, facadePolicyContract } from '
    '"file://%s/public/activation/focusa-facade-policy-presenter.mjs";'
) % ROOT

# ── 1. Contract artifact presence ─────────────────────────────────────────

check("pub mod facade_policy_presenter" in LIB, "facade_policy_presenter module is registered in focusa-license")
for exported in [
    "FacadeFamily", "FacadeNextAction", "FacadePolicyDecision", "FacadeMaskedStatus",
    "FacadePresenterError", "facade_family", "safe_masked_status",
    "FACADE_PRESENTER_FIELDS", "FACADE_PRESENTER_FORBIDDEN_FIELDS",
    "FACADE_ALWAYS_REACHABLE", "FACADE_STATUS_ALLOWLIST",
]:
    check(f"pub use facade_policy_presenter::{{{exported}" in LIB or
          f"{exported}" in LIB.split("pub use facade_policy_presenter::")[1].split("};")[0],
          f"rust contract re-exports {exported}")
check("focusa.spec152f.facade_policy_presenter.v1" in MJS, "website presenter contract schema is frozen")
check("focusa-facade-policy-status" in MJS, "website presenter binds an accessible status region")

# ── 2. Rust contract static checks ────────────────────────────────────────

check("pub enum FacadeFamily" in RUST, "rust contract has typed FacadeFamily")
check("pub enum FacadeNextAction" in RUST, "rust contract has typed FacadeNextAction")
check("pub struct FacadePolicyDecision" in RUST, "rust contract has FacadePolicyDecision")
check("pub struct FacadeMaskedStatus" in RUST, "rust contract has FacadeMaskedStatus")
check("pub fn facade_family" in RUST, "rust contract has facade_family mapper")
check("pub fn safe_masked_status" in RUST, "rust contract has safe_masked_status")
check("FamilyNotPresentable" in RUST, "internal maintenance is not facade-presentable (fail closed)")

# The presenter struct exposes NO commercial selector fields and NO setters.
struct_body = RUST.split("pub struct FacadePolicyDecision")[1].split("impl FacadePolicyDecision")[0]
for forbidden in ["grants", "prices", "feature_activation", "runtime_policy", "product_code",
                  "limit_bucket", "limits", "dormant", "lease", "token"]:
    check(forbidden not in struct_body, f"FacadePolicyDecision has no {forbidden} field")
check("pub fn set_" not in RUST, "rust contract has no setters (no set_ methods)")
check("pub fn enable_" not in RUST, "rust contract has no enable_ toggles")
check("pub fn disable_" not in RUST, "rust contract has no disable_ toggles")

# Frozen field lists match the shared vocabulary exactly.
rust_fields_section = RUST.split("pub const FACADE_PRESENTER_FIELDS")[1].split("];")[0]
for field in FACADE_PRESENTER_FIELDS:
    check(f'"{field}"' in rust_fields_section, f"rust FACADE_PRESENTER_FIELDS includes {field}")
rust_forbidden_section = RUST.split("pub const FACADE_PRESENTER_FORBIDDEN_FIELDS")[1].split("];")[0]
for field in FACADE_PRESENTER_FORBIDDEN_FIELDS:
    check(f'"{field}"' in rust_forbidden_section, f"rust FORBIDDEN includes {field}")
check('"recovery_only"' in RUST, "rust status allowlist includes recovery_only")
check("FACADE_ALWAYS_REACHABLE" in RUST, "rust contract carries the always-reachable set")

# The projection derives only from the canonical decision (fail closed).
check("reduce_entitlement_state" in RUST or "EntitlementStateDecision" in RUST,
      "rust projection binds the canonical decision type")
check("fn facade_next_action_for_posture" in RUST, "posture-to-action is a canonical projection")
check("fn facade_next_action_for_status" in RUST, "status-to-action is a canonical projection")

# Rust unit tests exist for the contract.
for test in [
    "fn base_family_projects_the_canonical_base_decision",
    "fn denied_base_decision_projects_evaluate_action",
    "fn premium_family_projects_purchase_or_deny_and_is_premium",
    "fn always_reachable_families_never_sell_or_pitch_premium",
    "fn recovery_only_status_always_shows_recovery_action",
    "fn internal_maintenance_is_not_facade_presentable",
    "fn masked_status_accepts_only_authority_labels_and_masked_identities",
    "fn presenter_field_contract_is_frozen_and_forbids_commercial_selectors",
]:
    check(test in RUST, f"rust unit test {test} exists")

# ── 3. Authority denial/recovery envelope binding ─────────────────────────

# The facade presenter consumes only safe envelope fields; it never reads
# lease, credential, or key material from the activation envelope.
check("lease_envelope" not in MJS, "website presenter never reads lease_envelope")
check("one_time_key" not in MJS, "website presenter never reads one_time_key material")
check("poll_credential" not in MJS, "website presenter never reads poll credentials")
check("masked_status" in MJS and "masked_email" in MJS, "website presenter renders safe masked status only")
# The activation output envelope itself is presenter-safe: no price/grant fields.
for forbidden in ['"price"', '"grants"', '"features"', '"limits"']:
    check(forbidden not in REDUCER.split("pub struct ActivationOutputEnvelope")[1].split("impl ActivationOutputEnvelope")[0],
          f"ActivationOutputEnvelope has no {forbidden} field")

# ── 4. Registered facade descriptors cross-check ──────────────────────────

check(sorted(REGISTERED_FACADES) == sorted(EXPECTED_FACADES),
      "registry carries exactly the six registered facades")
check("facades" in MJS, "website presenter binds registered facade ids")
mjs_contract = run_node(f'{IMPORT}; console.log(JSON.stringify(facadePolicyContract));')
mjs_contract = json.loads(mjs_contract)
check(sorted(mjs_contract["facades"]) == sorted(EXPECTED_FACADES),
      "website presenter contract binds exactly the registered facades")
check(mjs_contract["role"] == "presenter_only", "facade role stays presenter_only")
check(mjs_contract["authority"] == "WPUIAI.com EDD", "facade authority stays WPUIAI.com EDD")
check(mjs_contract["fields"] == FACADE_PRESENTER_FIELDS, "website presenter fields match the contract")
check(sorted(mjs_contract["forbidden_fields"]) == sorted(FACADE_PRESENTER_FORBIDDEN_FIELDS),
      "website presenter forbidden fields match the contract")
check(sorted(mjs_contract["families"]) == [
    "always_reachable", "automation", "base_focusa", "premium_updates",
    "release_proof", "team_remote",
], "website presenter family vocabulary is exactly base + four premium + always-reachable")

# Registry descriptors carry no presenter-owned commercial fields.
registry_text = (CONTRACTS / "spec152e-facade-registry.v1.json").read_text(encoding="utf-8")
for row in REGISTRY["facades"]:
    check(row["status"] == "registered_presenter", f"{row['facade_id']} is a registered presenter")
    for forbidden in ["grants", "price", "prices", "feature_activation", "runtime_policy"]:
        check(forbidden not in json.dumps(row), f"{row['facade_id']} descriptor has no {forbidden}")

# ── 5. Runtime spoof tests (node) ─────────────────────────────────────────

# 5a. Every facade explains the SAME authority decision.
same_decision = run_node(f'''
{IMPORT};
const envelope = {{
  family: "base_focusa", posture: "deny", reason: "deny",
  status: "expired", masked_email: "c***@example.com", next_action: "activate_or_manage_entitlement",
}};
const outputs = facadePolicyContract.facades.map((id) => projectFacadePolicyDecision(id, envelope));
const first = JSON.stringify(outputs[0]);
for (const id of facadePolicyContract.facades) {{
  const out = projectFacadePolicyDecision(id, envelope);
  if (JSON.stringify(out) !== first) throw new Error(`facade {{{{id}}}} diverges from the shared decision`);
  if (JSON.stringify(out).includes(id)) throw new Error("decision output leaks the facade id");
}}
console.log(JSON.stringify({{ same: true, count: outputs.length, keys: Object.keys(outputs[0]).sort() }}));
''')
same = json.loads(same_decision)
check(same["same"] is True and same["count"] == 6, "all six facades project the identical decision")
check(sorted(same["keys"]) == sorted(FACADE_PRESENTER_FIELDS),
      "decision output keys are exactly the frozen presenter fields")

# No per-facade commercial override table exists in the presenter.
check("OVERRIDE" not in MJS, "no per-facade override table in the presenter")
check("switch (facadeId)" not in MJS, "presenter never switches on facade id")
for facade_id in EXPECTED_FACADES:
    check(f'case "{facade_id}"' not in MJS and f"case '{facade_id}'" not in MJS,
          f"presenter never branches output on {facade_id}")

# 5b. Spoofed envelopes fail closed.
def spoof(env, label, facade_id="focusa_marketing_v1"):
    script = f'''
{IMPORT};
try {{
  projectFacadePolicyDecision("{facade_id}", {json.dumps(env)});
  console.log("accepted");
}} catch (error) {{
  console.log(error.message);
}}
'''
    return run_node(script)

check(spoof({"family": "base_focusa", "posture": "deny", "reason": "deny", "status": "expired",
             "masked_email": "c***@example.com", "next_action": "x"}, "baseline") == "accepted",
      "a canonical envelope is accepted", kind="negative")
for label, env, facade_id in [
    ("unknown origin", {"family": "base_focusa", "posture": "deny", "reason": "deny", "status": "expired"}, "evil_facade_v1"),
    ("unknown family", {"family": "internal_maintenance", "posture": "deny", "reason": "deny", "status": "expired"}, "focusa_marketing_v1"),
    ("invented family", {"family": "premium_magic", "posture": "deny", "reason": "deny", "status": "expired"}, "focusa_marketing_v1"),
    ("unknown posture", {"family": "base_focusa", "posture": "god_mode", "reason": "deny", "status": "expired"}, "focusa_marketing_v1"),
    ("grant selection", {"family": "base_focusa", "posture": "deny", "reason": "deny", "status": "expired", "grants": ["automation"]}, "focusa_marketing_v1"),
    ("price selection", {"family": "base_focusa", "posture": "deny", "reason": "deny", "status": "expired", "price": 9.99}, "focusa_marketing_v1"),
    ("prices selection", {"family": "base_focusa", "posture": "deny", "reason": "deny", "status": "expired", "prices": [9.99]}, "focusa_marketing_v1"),
    ("feature activation", {"family": "base_focusa", "posture": "deny", "reason": "deny", "status": "expired", "feature_activation": True}, "focusa_marketing_v1"),
    ("runtime policy", {"family": "base_focusa", "posture": "deny", "reason": "deny", "status": "expired", "runtime_policy": "whatever"}, "focusa_marketing_v1"),
    ("dormant toggle", {"family": "base_focusa", "posture": "deny", "reason": "deny", "status": "expired", "dormant": False}, "focusa_marketing_v1"),
    ("product selection", {"family": "base_focusa", "posture": "deny", "reason": "deny", "status": "expired", "product_code": "focusa_operator_lifetime_v1"}, "focusa_marketing_v1"),
    ("limit selection", {"family": "base_focusa", "posture": "deny", "reason": "deny", "status": "expired", "limits": {"nodes": 99}}, "focusa_marketing_v1"),
    ("lease injection", {"family": "base_focusa", "posture": "deny", "reason": "deny", "status": "expired", "lease": "fake-lease"}, "focusa_marketing_v1"),
    ("raw email", {"family": "base_focusa", "posture": "deny", "reason": "deny", "status": "expired", "masked_email": "customer@example.com"}, "focusa_marketing_v1"),
    ("redirect hijack", {"family": "base_focusa", "posture": "deny", "reason": "deny", "status": "expired", "redirect_url": "https://evil.example"}, "focusa_marketing_v1"),
    ("premium toggle", {"family": "base_focusa", "posture": "deny", "reason": "deny", "status": "expired", "premium": True}, "focusa_marketing_v1"),
    ("unknown extra field", {"family": "base_focusa", "posture": "deny", "reason": "deny", "status": "expired", "bonus": 1}, "focusa_marketing_v1"),
]:
    check(spoof(env, label, facade_id) == "FACADE_POLICY_DENIED" or (label == "unknown origin" and spoof(env, label, facade_id) == "FACADE_ORIGIN_DENIED"),
          f"spoof rejected: {label}", kind="negative")

# 5c. Premium/dormant dimensions cannot be turned on or off through any
#     output field: the output carries no premium/dormant/feature/grant key.
premium_view = run_node(f'''
{IMPORT};
const out = projectFacadePolicyDecision("wpuiai_public_v1", {{
  family: "automation", posture: "feature", reason: "require_feature",
  status: "active_paid", masked_email: "a***@example.com", next_action: "manage_license",
}});
console.log(JSON.stringify(out));
''')
premium = json.loads(premium_view)
check(premium["family"] == "automation", "premium family label is projected verbatim")
check(premium["action"] == "purchase", "premium feature posture projects the purchase action")
check("premium" not in premium, "output has no premium toggle")
check("dormant" not in premium, "output has no dormant toggle")
check("feature" not in premium, "output has no feature activation field")
check("grants" not in premium, "output has no grants field")
check("price" not in premium, "output has no price field")

# 5d. Safe masked status: unknown status is dropped (fail closed), never raw.
masked_status = run_node(f'''
{IMPORT};
const out = projectFacadePolicyDecision("focusa_forge_v1", {{
  family: "base_focusa", posture: "base", reason: "require_base",
  status: "spoofed_status", masked_email: "c***@example.com", next_action: "manage_license",
}});
console.log(JSON.stringify(out.masked_status));
''')
check(masked_status == "undefined" or masked_status == "null",
      "spoofed status never renders (masked_status dropped)")

# 5e. Recovery-only status always shows the recovery action on every facade.
recovery_view = run_node(f'''
{IMPORT};
const out = facadePolicyContract.facades.map((id) => projectFacadePolicyDecision(id, {{
  family: "base_focusa", posture: "deny", reason: "deny",
  status: "recovery_only", masked_email: "b***@example.com", next_action: "recovery",
}}));
console.log(JSON.stringify({{ actions: out.map((v) => v.action), labels: out.map((v) => v.action_label) }}));
''')
recovery = json.loads(recovery_view)
check(recovery["actions"] == ["recovery"] * 6, "every facade shows the recovery action for recovery_only")
check(recovery["labels"] == ["Continue recovery"] * 6, "every facade shows the recovery label")

# ── 6. Website presenters stay presenter-only ─────────────────────────────

check("cannot issue licenses or entitlements" in REGISTRATION,
      "registration presenter still declares no issuance authority")
check("FACADE_REQUEST_FIELD_DENIED" in REGISTRATION, "registration presenter still rejects caller commercial fields")
check("WPUIAI.com EDD is the authority" in REGISTRATION,
      "branded registration page still names WPUIAI.com EDD as the authority")
check("cannot issue licenses or entitlements" in REGISTRATION or "cannot issue" in PAGE_HTML,
      "branded page still declares no issuance authority")
check('Object.hasOwn(fields, "price")' in REGISTRATION and 'Object.hasOwn(fields, "grants")' in REGISTRATION,
      "registration presenter rejects caller-supplied price/grants (FACADE_REQUEST_FIELD_DENIED)")
check("Offers and grants are supplied by the authority" in REGISTRATION,
      "registration presenter declares authority-owned offers and grants")
check("FACADE_BINDINGS" in SECURITY, "facade security still owns exact-origin bindings (Spec 152E)")

# ── Summary ───────────────────────────────────────────────────────────────

print(f"spec152f facade policy presenter: PASS (positive={POSITIVE} negative={NEGATIVE})")
print("  same decision: every registered facade projects the identical authority decision")
print("  fail closed: grants/prices/feature activation/runtime policy/dormant toggles rejected")
print("  safe status: masked status only; spoofed statuses never render")
sys.exit(0)
