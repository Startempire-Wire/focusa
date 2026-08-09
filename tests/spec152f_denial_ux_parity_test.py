#!/usr/bin/env python3
"""Spec 152F.05.06 — Standardize denial, purchase, and recovery UX.

Proves the cross-presenter message catalog: a stable error registry plus
CLI/desktop/TUI/Pi/facade message fixtures bound to one canonical set of
plain-language blocked-action/reason/retained-access/one-safe-next-action
messages with stable account/evaluation/checkout/recovery links. Messages are
semantically identical across the Rust contract, the committed JSON artifact,
and the website/Pi JS fixture; every denied/limited/feature message preserves
a route to purchase or recovery; accessibility (retained access always
listed) and privacy (no internal route/lease details, no false urgency, no
account enumeration, no raw email/key/token, no dead-end paywalls) checks
pass.

Surfaces covered
- stable error registry: crates/focusa-license/src/denial_ux.rs (typed,
  fail-closed DenialUxErrorCode) + docs/contracts/spec152f-denial-ux-catalog.v1.json
- CLI/desktop/TUI/Pi/facade message fixtures: the catalog artifact, the JS
  fixture public/activation/focusa-denial-ux-catalog.mjs, and read-only
  cross-checks of each presenter's existing fixture binding
- account/evaluation/checkout/recovery links: stable relative paths shared by
  the Rust contract, the catalog artifact, and the JS fixture

Exact verification: python3 tests/spec152f_denial_ux_parity_test.py

No cargo build, no live network, no publication.
"""

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "docs/contracts"
RUST = (ROOT / "crates/focusa-license/src/denial_ux.rs").read_text(encoding="utf-8")
LIB = (ROOT / "crates/focusa-license/src/lib.rs").read_text(encoding="utf-8")
CATALOG = json.loads((CONTRACTS / "spec152f-denial-ux-catalog.v1.json").read_text(encoding="utf-8"))
MJS = (ROOT / "public/activation/focusa-denial-ux-catalog.mjs").read_text(encoding="utf-8")

STATES = [
    "pending_unverified", "verified_no_license", "active_paid", "offline_grace",
    "expired", "refunded_or_revoked", "missing_or_corrupt",
]
FAMILIES = [
    "account_recovery", "read_projection", "base_focusa", "automation",
    "team_remote", "release_proof", "premium_updates", "customer_data_export",
    "internal_maintenance",
]
ALWAYS_REACHABLE = [
    "navigation", "status", "account", "read", "export", "recovery",
    "repair", "update", "uninstall",
]
LINK_IDS = ["account", "evaluation", "checkout", "recovery"]
EXPECTED_LINKS = {
    "account": "/account",
    "evaluation": "/activate/evaluate",
    "checkout": "/activate/checkout",
    "recovery": "/activate/recovery",
}
CODE_TO_CONST = {
    "ENTITLEMENT_BASE_REQUIRED": "MSG_BASE_REQUIRED",
    "ENTITLEMENT_FEATURE_REQUIRED": "MSG_FEATURE_REQUIRED",
    "ENTITLEMENT_REQUIRED": "MSG_REQUIRED",
    "ENTITLEMENT_LIMIT_EXHAUSTED": "MSG_LIMIT_EXHAUSTED",
    "ENTITLEMENT_RECOVERY_ONLY": "MSG_RECOVERY_ONLY",
    "ENTITLEMENT_SNAPSHOT_MISSING": "MSG_SNAPSHOT_MISSING",
    "ENTITLEMENT_ROUTE_UNCLASSIFIED": "MSG_ROUTE_UNCLASSIFIED",
    "ENTITLEMENT_POLICY_UNKNOWN": "MSG_POLICY_UNKNOWN",
    "ENTITLEMENT_RESERVATION_FAILED": "MSG_RESERVATION_FAILED",
    "ENTITLEMENT_IDEMPOTENCY_REQUIRED": "MSG_IDEMPOTENCY_REQUIRED",
}

# Customer-visible hazards (substrings) that must never appear in a message.
FORBIDDEN_SUBSTRINGS = [
    "/v1", "sha256", "@", "http://", "https://",
    "immediately", "urgent", "act now", "will be deleted", "will be lost",
    "lose access", "expire soon", "too late",
]
# Whole-word hazards that must never appear as standalone words.
FORBIDDEN_WORDS = [
    "lease", "key", "token", "email", "credential", "secret",
    "node", "sequence", "digest", "signature",
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
    'import { projectDenialUxMessage, messageForErrorCode, denialUxLink, '
    'denialUxCatalog } from "file://%s/public/activation/focusa-denial-ux-catalog.mjs";'
) % ROOT


def rust_str_const(name):
    """Extract `pub const <name>: &str = "..."` from the Rust contract."""
    match = re.search(
        rf'pub const {name}: &str = "((?:[^"\\]|\\.)*)";',
        RUST,
    )
    if not match:
        raise AssertionError(f"missing Rust const {name}")
    return match.group(1)


# ── 1. Contract artifact presence ─────────────────────────────────────────

check("pub mod denial_ux" in LIB, "denial_ux module is registered in focusa-license")
for exported in [
    "DenialUxErrorCode", "DenialUxMessage", "DenialUxErrorSpec", "DenialUxKind",
    "DenialUxError", "denial_ux_message_for", "denial_ux_message_for_code",
    "denial_ux_link", "embedded_denial_ux_catalog", "RETAINED_ACCESS",
    "PUBLIC_MESSAGE_RULES", "DENIAL_UX_LINK_IDS", "DENIAL_UX_SCHEMA",
    "DENIAL_UX_CATALOG_JSON",
]:
    check(f"{exported}" in LIB, f"rust contract re-exports {exported}")
check(CATALOG["schema"] == "focusa.spec152f.denial_ux_catalog.v1", "catalog artifact schema")
check(CATALOG["contract_version"] == 1, "catalog artifact version")
check("projectDenialUxMessage" in MJS, "js fixture exports the fail-closed projector")
check("messageForErrorCode" in MJS, "js fixture exports the code lookup")
check("denialUxCatalog" in MJS, "js fixture exports the frozen contract surface")

# ── 2. Stable error registry (Rust typed, fail closed) ────────────────────

check("pub enum DenialUxErrorCode" in RUST, "rust contract has typed DenialUxErrorCode")
check("pub fn from_label" in RUST, "rust contract has fail-closed label lookup")
check("pub struct DenialUxErrorSpec" in RUST, "rust contract has DenialUxErrorSpec")
check("pub const DENIAL_UX_CATALOG_JSON" in RUST, "rust contract embeds the catalog artifact")
check("include_str!" in RUST, "rust contract embeds the artifact bytes")
check("pub fn denial_ux_link" in RUST, "rust contract has stable link lookup")
check("pub fn denial_ux_message_for" in RUST, "rust contract derives grid messages")
check("pub fn denial_ux_message_for_code" in RUST, "rust contract projects registry messages")
check("preserves_route" in RUST, "rust messages prove a route to purchase or recovery")

# No setters / no commercial selectors on the message contract.
check("pub fn set_" not in RUST, "rust contract has no setters (no set_ methods)")
check("pub fn enable_" not in RUST, "rust contract has no enable_ toggles")
check("pub fn disable_" not in RUST, "rust contract has no disable_ toggles")
# No grant/price selector fields on the message or spec structs (the
# docstring may name them as forbidden concepts; the struct fields must not).
for struct_name in ["pub struct DenialUxErrorSpec", "pub struct DenialUxMessage"]:
    body = RUST.split(struct_name)[1].split("impl ")[0]
    for forbidden in ["grants", "price", "prices", "feature_activation", "runtime_policy"]:
        check(forbidden not in body,
              f"{struct_name} has no {forbidden} field")

# Frozen surface sizes match the artifact.
for item in ALWAYS_REACHABLE:
    check(f'"{item}"' in RUST, f"rust retained-access set includes {item!r}")
check("pub const RETAINED_ACCESS: [&str; 9]" in RUST, "retained access has exactly 9 entries")
check("pub const PUBLIC_MESSAGE_RULES: [&str; 7]" in RUST, "public message rules have 7 entries")
check("pub const DENIAL_UX_LINK_IDS: [&str; 4]" in RUST, "link ids have 4 entries")
check("pub const DENIAL_UX_ACTIONS: [&str; 7]" in RUST, "action ids have 7 entries")
check("[DenialUxErrorCode; 10]" in RUST, "error registry has 10 codes")

# Every registry code from the artifact has a typed Rust variant.
for entry in CATALOG["error_registry"]:
    code = entry["code"]
    check(f'"{code}"' in RUST, f"rust contract carries {code}")

# ── 3. Registry parity: Rust constants == JSON artifact == JS fixture ──────

rust_messages = {name: rust_str_const(name) for name in CODE_TO_CONST.values()}
for entry in CATALOG["error_registry"]:
    const_name = CODE_TO_CONST[entry["code"]]
    check(rust_messages[const_name] == entry["public_message"],
          f"rust MSG const matches artifact public_message for {entry['code']}")

for link_id, path in EXPECTED_LINKS.items():
    check(rust_str_const(f"LINK_{link_id.upper()}") == path,
          f"rust LINK_{link_id.upper()} matches artifact link {link_id}")
    check(CATALOG["links"][link_id] == path, f"artifact link {link_id} is stable")

for entry in CATALOG["error_registry"]:
    check(entry["link"] in LINK_IDS, f"registry link for {entry['code']} is a known link id")
    check(entry["safe_next_action"] in [a["id"] for a in CATALOG["actions"]],
          f"registry action for {entry['code']} is in the frozen vocabulary")

# JS fixture carries the identical registry (runtime node dump).
js_registry = run_node(f'{IMPORT}; console.log(JSON.stringify(denialUxCatalog.error_registry));')
js_registry = json.loads(js_registry)
check(len(js_registry) == len(CATALOG["error_registry"]),
      "js registry has the same size as the artifact")
for entry in CATALOG["error_registry"]:
    js_entry = next(row for row in js_registry if row["code"] == entry["code"])
    check(js_entry["public_message"] == entry["public_message"],
          f"js public_message identical for {entry['code']}")
    check(js_entry["safe_next_action"] == entry["safe_next_action"],
          f"js safe_next_action identical for {entry['code']}")
    check(js_entry["link"] == entry["link"], f"js link identical for {entry['code']}")

# ── 4. Message grid completeness: every state/family cell ─────────────────

grid = CATALOG["message_grid"]
check(len(grid) == len(STATES) * len(FAMILIES) == 63,
      f"message grid covers 7 states x 9 families ({len(grid)} cells)")
check(sorted({cell["state"] for cell in grid}) == sorted(STATES), "grid covers all states")
check(sorted({cell["family"] for cell in grid}) == sorted(FAMILIES), "grid covers all families")

denied_or_gated = 0
for cell in grid:
    label = f"{cell['state']}/{cell['family']}"
    check(cell["blocked_action"] and isinstance(cell["blocked_action"], str),
          f"{label}: blocked action is plain language")
    check(cell["reason"] and isinstance(cell["reason"], str), f"{label}: reason is present")
    retained = cell["retained_access"]
    check(isinstance(retained, list) and len(retained) >= 1, f"{label}: retained access listed")
    check(all(item in ALWAYS_REACHABLE for item in retained),
          f"{label}: retained access subset of always-reachable set")
    check(isinstance(cell["safe_next_action"], str) and cell["safe_next_action"],
          f"{label}: exactly one safe next action")
    check(cell["link"] in LINK_IDS, f"{label}: stable link id")
    check(cell["action_label"] and isinstance(cell["action_label"], str),
          f"{label}: action label present")

    # Privacy/redaction: no internal route/lease details, no false urgency,
    # no account enumeration, no raw email/key/token, no dead-end paywall.
    text = " ".join([
        str(cell["blocked_action"]), str(cell["reason"]),
        str(cell["action_label"]), str(cell["safe_next_action"]),
    ]).lower()
    for bad in FORBIDDEN_SUBSTRINGS:
        check(bad not in text, f"{label}: no forbidden substring {bad!r}")
    for word in FORBIDDEN_WORDS:
        check(not re.search(rf"\b{word}\b", text), f"{label}: no forbidden word {word!r}")
    check(not re.search(r"[a-z0-9._%+-]+@[a-z0-9.-]+", text),
          f"{label}: no raw email pattern")

    # Every gated message (denied/limited/feature) preserves a route to
    # purchase or recovery: stable link + one safe next action, never a
    # dead-end paywall.
    if cell["code"]:
        denied_or_gated += 1
        check(cell["link"] in ("evaluation", "checkout", "recovery", "account"),
              f"{label}: gated message carries a route link")
        check(cell["safe_next_action"] and cell["action_label"],
              f"{label}: gated message carries one safe next action")
    else:
        check(cell["kind"] == "available",
              f"{label}: code-less cells are available messages")
check(denied_or_gated >= 30, f"catalog gates a substantial subset ({denied_or_gated} cells)")

# Denied base cells route to Evaluation (purchase path); denied premium cells
# route to checkout (purchase path); recovery exists in the registry.
for state in ["pending_unverified", "expired", "refunded_or_revoked", "missing_or_corrupt"]:
    cell = next(c for c in grid if c["state"] == state and c["family"] == "base_focusa")
    check(cell["code"] == "ENTITLEMENT_BASE_REQUIRED", f"{state}: base denial code is stable")
    check(cell["link"] == "evaluation", f"{state}: base denial routes to evaluation")
for state in ["expired", "refunded_or_revoked", "missing_or_corrupt"]:
    cell = next(c for c in grid if c["state"] == state and c["family"] == "automation")
    check(cell["code"] == "ENTITLEMENT_FEATURE_REQUIRED", f"{state}: premium denial code is stable")
    check(cell["link"] == "checkout", f"{state}: premium denial routes to checkout")

# ── 5. Account/evaluation/checkout/recovery links ─────────────────────────

for link_id, path in EXPECTED_LINKS.items():
    check(path.startswith("/"), f"link {link_id} is relative (no absolute redirect)")
    check("?" not in path and "#" not in path, f"link {link_id} carries no query/fragment")
    check("@email" not in path and "token" not in path and "key" not in path,
          f"link {link_id} carries no raw identity or secret material")
    check(CATALOG["links"][link_id] == path, f"artifact link {link_id} matches expectation")

# Cross-check against the Spec 152E facade registry (read-only): checkout,
# manage, and recovery paths must agree with the registered facade paths.
facade_registry_path = CONTRACTS / "spec152e-facade-registry.v1.yaml"
facade_registry = facade_registry_path.read_text(encoding="utf-8")
check("checkout: /activate/checkout" in facade_registry, "facade checkout path matches catalog")
check("manage: /account" in facade_registry, "facade manage path matches catalog account link")
check("recovery: /activate/recovery" in facade_registry, "facade recovery path matches catalog")

# ── 6. Cross-presenter message fixture parity (read-only cross-checks) ────

for presenter, binding in CATALOG["presenter_bindings"].items():
    fixture = ROOT / binding["fixture"]
    check(fixture.exists(), f"{presenter} fixture exists: {binding['fixture']}")
    text = fixture.read_text(encoding="utf-8")
    for token in binding["binds"]:
        check(token in text, f"{presenter} fixture binds {token!r}")

# Always-reachable set is identical across desktop/TUI/facade fixtures and the
# catalog (frozen 9-entry fixture, Spec 152F P6).
for fixture_rel in [
    "docs/contracts/spec152f-menubar-action-map.v1.json",
    "crates/focusa-tui/src/activation_presenter.rs",
    "public/activation/focusa-facade-policy-presenter.mjs",
]:
    text = (ROOT / fixture_rel).read_text(encoding="utf-8")
    for item in ALWAYS_REACHABLE:
        check(item in text, f"{fixture_rel} carries always-reachable {item!r}")

# The action vocabulary is shared: each presenter fixture names the canonical
# actions (evaluate/purchase/recovery/manage) that the catalog labels.
for fixture_rel in [
    "crates/focusa-tui/src/activation_presenter.rs",
    "public/activation/focusa-facade-policy-presenter.mjs",
]:
    text = (ROOT / fixture_rel).read_text(encoding="utf-8")
    check("Evaluate" in text or "evaluate" in text, f"{fixture_rel} names the evaluate action")
    check("recovery" in text, f"{fixture_rel} names the recovery action")

# Pi adapter recovery projection is fail-closed and stable.
pi_text = (ROOT / "apps/pi-extension/src/entitlement-policy-adapter.ts").read_text(encoding="utf-8")
check("recoveryActionsFor" in pi_text, "pi adapter derives recovery actions")
check("LICENSE_STATUS_PATH" in pi_text, "pi adapter binds the status recovery path")

# ── 7. Runtime parity: JS fixture projects the identical catalog ──────────

js_grid = run_node(f'''
{IMPORT};
const STATES = {json.dumps(STATES)};
const FAMILIES = {json.dumps(FAMILIES)};
const out = [];
for (const s of STATES) {{
  for (const f of FAMILIES) {{
    const msg = projectDenialUxMessage({{ state: s, family: f }});
    out.push({{ state: s, family: f, msg }});
  }}
}}
console.log(JSON.stringify(out));
''')
js_cells = json.loads(js_grid)
check(len(js_cells) == 63, "js fixture projects all 63 grid cells")
for cell, js_cell in zip(grid, js_cells):
    label = f"{cell['state']}/{cell['family']}"
    check(js_cell["state"] == cell["state"] and js_cell["family"] == cell["family"], f"{label}: cell aligned")
    msg = js_cell["msg"]
    check(msg is not None, f"{label}: js projects a message")
    check(msg["kind"] == cell["kind"], f"{label}: kind parity")
    check(msg["code"] == cell["code"], f"{label}: code parity")
    check(msg["blocked_action"] == cell["blocked_action"], f"{label}: blocked_action parity")
    check(msg["reason"] == cell["reason"], f"{label}: reason parity")
    check(msg["safe_next_action"] == cell["safe_next_action"], f"{label}: next-action parity")
    check(msg["action_label"] == cell["action_label"], f"{label}: action_label parity")
    check(msg["link"] == cell["link"], f"{label}: link parity")
    check(msg["retained_access"] == ALWAYS_REACHABLE, f"{label}: retained access parity")
    check(msg["link_path"] == EXPECTED_LINKS[cell["link"]], f"{label}: link_path parity")

# ── 8. Fail-closed runtime behavior (node) ────────────────────────────────

def js_result(expr):
    return run_node(f"{IMPORT}; console.log(JSON.stringify({expr}));")

check(js_result("projectDenialUxMessage({ state: 'bogus', family: 'base_focusa' })") == "null",
      "unknown state fails closed", kind="negative")
check(js_result("projectDenialUxMessage({ state: 'expired', family: 'magic' })") == "null",
      "unknown family fails closed", kind="negative")
check(js_result("messageForErrorCode('ENTITLEMENT_MAGIC')") == "null",
      "unknown code fails closed", kind="negative")
check(js_result("projectDenialUxMessage({})") == "null",
      "empty input fails closed", kind="negative")
check(js_result("projectDenialUxMessage({ state: 'expired', family: 'base_focusa', code: 'ENTITLEMENT_BASE_REQUIRED' })") == "null",
      "grid+code ambiguity fails closed", kind="negative")
check(js_result("projectDenialUxMessage({ state: 'expired', family: 'base_focusa', masked_email: 'c***@example.com' })") == "null",
      "raw identity input fails closed", kind="negative")
check(js_result("projectDenialUxMessage({ state: 'expired', family: 'base_focusa', grants: ['automation'] })") == "null",
      "grant selection fails closed", kind="negative")
check(js_result("projectDenialUxMessage({ state: 'expired', family: 'base_focusa', price: 9.99 })") == "null",
      "price selection fails closed", kind="negative")
check(js_result("denialUxLink('https://evil.example')") == "null",
      "absolute redirect link fails closed", kind="negative")
check(js_result("denialUxLink('checkout?ref=evil')") == "null",
      "query-bearing link fails closed", kind="negative")
check(js_result("denialUxLink('account')") == '"/account"',
      "stable account link resolves")

# Every code lookup returns a message with a route.
code_messages = js_result(
    "[" + ",".join(f"messageForErrorCode({json.dumps(e['code'])})" for e in CATALOG["error_registry"]) + "]")
for entry, msg in zip(CATALOG["error_registry"], json.loads(code_messages)):
    check(msg is not None, f"code lookup resolves {entry['code']}")
    check(msg["link_path"] == EXPECTED_LINKS[msg["link"]], f"{entry['code']} link_path resolves")
    check(msg["safe_next_action"] and msg["action_label"], f"{entry['code']} has one safe next action")

# ── 9. Accessibility and privacy checks (catalog-level) ───────────────────

check(CATALOG["accessibility"]["retained_access_always_present"] is True,
      "accessibility: retained access always present")
check(CATALOG["accessibility"]["exactly_one_safe_next_action"] is True,
      "accessibility: exactly one safe next action")
check(CATALOG["accessibility"]["no_disabled_traps"] is True,
      "accessibility: no disabled traps")
check(CATALOG["privacy"]["no_raw_email"] is True, "privacy: no raw email")
check(CATALOG["privacy"]["no_raw_key_or_token"] is True, "privacy: no raw key/token")
check(CATALOG["privacy"]["no_account_enumeration"] is True, "privacy: no account enumeration")
check(CATALOG["privacy"]["no_internal_route_or_lease_details"] is True,
      "privacy: no internal route/lease details")
check(CATALOG["privacy"]["no_false_urgency"] is True, "privacy: no false urgency")
check(CATALOG["privacy"]["no_dead_end_paywalls"] is True, "privacy: no dead-end paywalls")
check(len(CATALOG["rules"]) == 7, "public message rules are frozen")
for rule in CATALOG["rules"]:
    check(isinstance(rule, str) and rule, f"rule text present: {rule!r}")

# ── Summary ───────────────────────────────────────────────────────────────

print(f"spec152f denial/purchase/recovery UX parity: PASS (positive={POSITIVE} negative={NEGATIVE})")
print(f"  catalog: {len(grid)} state/family cells, {len(CATALOG['error_registry'])} stable error codes")
print("  parity: Rust contract, JSON artifact, and JS fixture are semantically identical")
print("  redaction: no internal route/lease details, no false urgency, no raw material")
print("  routes: every gated message preserves a route to purchase or recovery")
sys.exit(0)
