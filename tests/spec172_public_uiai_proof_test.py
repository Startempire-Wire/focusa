#!/usr/bin/env python3
"""Spec 172 UIAI public pricing, route, and contradiction-removal proof gate
(build-independent, deterministic, offline).

Proves the redacted UIAI evidence packet
(docs/contracts/spec172-uiai-public-proof.v1.json) is:
- schema-frozen and owned by atom focusa-vbcqu.20.15.39;
- a faithful, bounded, redacted capture of the LIVE public surfaces recorded
  on 2026-08-09 (UIAI Engine browser reads plus origin HTTP reads);
- deterministically indexed (the claims digest is recomputed here and must
  match the frozen digest byte-for-byte);
- cross-consistent with the accepted Section 18.1 baseline contract and the
  accepted public-facade convergence contract;
- free of raw email, keys, tokens, customer rows, credentials, and card data.

The gate does NOT assert that the live public sites are converged: the packet
explicitly records which Section 18.2 contradictions remain live (R01..R09)
with their blockers, so a weak model cannot close by editing the packet to
claim removal without live evidence. No network, no authenticated capture, no
cargo build, no publication.
"""

import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "docs/contracts"

PROOF = json.loads((CONTRACTS / "spec172-uiai-public-proof.v1.json").read_text(encoding="utf-8"))
PROOF_RAW = (CONTRACTS / "spec172-uiai-public-proof.v1.json").read_text(encoding="utf-8")
BASELINE = json.loads((CONTRACTS / "spec172-public-commerce-baseline.v1.json").read_text(encoding="utf-8"))
CONVERGENCE = json.loads((CONTRACTS / "spec172-public-facade-convergence.v1.json").read_text(encoding="utf-8"))

EMAIL_RE = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
SECRET_RE = re.compile(r"(?i)(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+|focusa_live_[0-9]+_[0-9a-f]+")
LICENSE_KEY_RE = re.compile(r"FOCUSA-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}")
CARD_RE = re.compile(r"\b(?:\d[ -]?){13,16}\b")

positive = 0
negative = 0


def check(condition: bool, message: str, kind: str = "positive") -> None:
    global positive, negative
    if not condition:
        raise AssertionError(f"FAIL ({kind}): {message}")
    if kind == "positive":
        positive += 1
    else:
        negative += 1


# ---------------------------------------------------------------------------
# 1. Packet identity and authority.
# ---------------------------------------------------------------------------
check(PROOF["schema"] == "focusa.spec172_uiai_public_proof.v1", "proof packet schema")
check(PROOF["version"] == 1, "proof packet version")
check(PROOF["atom"] == "focusa-vbcqu.20.15.39", "proof packet atom owner")
check(PROOF["title"] == "172.05.08 Capture UIAI public pricing, route, and contradiction-removal proof", "proof packet title")
check(PROOF["authority"]["section_18_1"].endswith("#181-public-evidence-basis"), "authority cites Section 18.1")
check(PROOF["authority"]["section_18_2"].endswith("#182-required-convergence"), "authority cites Section 18.2")
check((ROOT / PROOF["authority"]["spec"]).is_file(), "authority spec file exists")
check((ROOT / PROOF["authority"]["baseline_contract"]).is_file(), "baseline contract file exists")
check((ROOT / PROOF["authority"]["convergence_contract"]).is_file(), "convergence contract file exists")
prices = PROOF["authority"]["canonical_prices"]
check(prices == {
    "focusa_operator_lifetime_v1": "697.00",
    "uiai_operator_lifetime_v1": "697.00",
    "focusa_uiai_operator_bundle_lifetime_v1": "1254.60",
}, "canonical prices frozen in authority")

# ---------------------------------------------------------------------------
# 2. Capture posture.
# ---------------------------------------------------------------------------
capture = PROOF["capture"]
check(capture["capture_method"] == "uiai_engine_browser_read", "capture method is UIAI Engine browser read")
check(capture["supplemental_method"] == "live_origin_http_read", "supplemental method is live origin HTTP read")
check(capture["authentication_required"] is False, "no authenticated capture")
check(capture["uiai_service_health"] == "healthy", "UIAI service reported healthy at capture")
reach = capture["browser_plane_reachability"]
check(reach["https://engine.focusa.dev/"] == "reachable", "engine reachable from UIAI plane")
check(reach["https://wpuiai.com/"] == "reachable", "wpuiai reachable from UIAI plane")
check(reach["https://focusa.dev/"] == "unreachable_navigation_timeout_origin_read_used", "focusa unreachable from UIAI plane, origin read used")
check(reach["https://install.focusa.dev/"] == "unreachable_navigation_timeout_origin_read_used", "install unreachable from UIAI plane, origin read used")

posture = PROOF["evidence_posture"]
check(posture == {
    "classification": "bounded_redacted_public_claim_proof",
    "public_copy_is_entitlement_authority": False,
    "raw_capture_embedded": False,
    "raw_email_or_key_embedded": False,
    "authenticated_or_customer_data_capture": False,
    "no_anonymous_or_local_grant_proof": True,
    "service_status_after_capture": "available",
}, "evidence posture frozen")

# ---------------------------------------------------------------------------
# 3. Sources must mirror the accepted Section 18.1 baseline exactly.
# ---------------------------------------------------------------------------
baseline_sources = {row["surface"]: row["urls"] for row in BASELINE["sources"]}
proof_sources = {row["surface"]: row["urls"] for row in PROOF["sources"]}
check(set(proof_sources) == set(baseline_sources), "proof covers the same four surfaces as the baseline")
for surface, urls in baseline_sources.items():
    check(proof_sources[surface] == urls, f"surface {surface} URL list matches the Section 18.1 baseline")

# ---------------------------------------------------------------------------
# 4. UIAI session evidence entries are bounded and internally consistent.
# ---------------------------------------------------------------------------
sessions = PROOF["uiai_sessions"]
check(len(sessions) == PROOF["proof_index"]["uiai_session_count"], "session count matches proof index")
reachable_sessions = [s for s in sessions if s["reachable"]]
unreachable_sessions = [s for s in sessions if not s["reachable"]]
check({s["surface"] for s in reachable_sessions} == {"engine", "wpuiai", "wpuiai_edd"}, "reachable UIAI sessions cover engine and WPUIAI")
check({s["surface"] for s in unreachable_sessions} == {"focusa", "install"}, "unreachable UIAI sessions are focusa and install")
session_ids = [s["session_id"] for s in sessions if s["session_id"]]
check(len(session_ids) == len(set(session_ids)), "session ids are unique")
for s in sessions:
    check(s["url"].startswith("https://"), f"{s['surface']} session URL is https")
    check(s["evidence_ref"], f"{s['surface']} session has an evidence ref")
    if s["reachable"]:
        check(s["read_schema"] == "uiai.browser_read.v2", f"{s['surface']} reachable read schema")
        check(s["evidence_ref"].startswith("uiai-browser:session="), f"{s['surface']} evidence ref is a browser ref")
        diag = s["diagnostics"]
        check(diag["exceptions"] == 0, f"{s['surface']} session had zero exceptions")
        check(diag["console"] >= 0 and diag["failed_requests"] >= 0, f"{s['surface']} diagnostics counts are non-negative")
    else:
        check(s["class"] == "timeout", f"{s['surface']} unreachable class is timeout")
        check(s["evidence_ref"].startswith("uiai-error:uiai-error-"), f"{s['surface']} evidence ref is a UIAI error ref")
# engine and WPUIAI sessions recorded clean diagnostics
for s in reachable_sessions:
    if s["surface"] in ("engine", "wpuiai", "wpuiai_edd"):
        check(s["diagnostics"]["exceptions"] == 0, f"{s['surface']} UIAI diagnostics show zero exceptions")

# ---------------------------------------------------------------------------
# 5. Origin reads: statuses and routes recorded live.
# ---------------------------------------------------------------------------
reads = {row["url"]: row for row in PROOF["origin_reads"]}
check(len(reads) == PROOF["proof_index"]["origin_read_count"], "origin read count matches proof index")
check(reads["https://focusa.dev/"]["status"] == 200, "focusa root origin read 200")
check(reads["https://focusa.dev/pricing/"]["status"] == 200, "focusa pricing origin read 200")
check(reads["https://focusa.dev/llms.txt"]["status"] == 200, "focusa llms.txt origin read 200")
check(reads["https://focusa.dev/.well-known/agent-commerce.json"]["status"] == 200, "focusa agent-commerce origin read 200")
check(reads["https://focusa.dev/LICENSE"]["status"] == 404, "focusa.dev/LICENSE origin read 404")
check(reads["https://focusa.dev/COMMERCIAL.md"]["status"] == 404, "focusa.dev/COMMERCIAL.md origin read 404")
check(reads["https://engine.focusa.dev/"]["status"] == 200, "engine root origin read 200")
check(reads["https://engine.focusa.dev/LICENSE"]["status"] == 404, "engine LICENSE origin read 404")
check(reads["https://engine.focusa.dev/COMMERCIAL.md"]["status"] == 404, "engine COMMERCIAL.md origin read 404")
check(reads["https://install.focusa.dev/"]["status"] == 200, "install root origin read 200")
for route in ("https://install.focusa.dev/focusa", "https://install.focusa.dev/bundle",
              "https://install.focusa.dev/engine", "https://install.focusa.dev/powershell",
              "https://install.focusa.dev/checksums", "https://install.focusa.dev/terms/",
              "https://install.focusa.dev/license"):
    check(reads[route]["status"] == 404, f"install route {route} origin read 404")
check(reads["https://wpuiai.com/wp-sitemap-posts-download-1.xml"]["status"] == 200, "WPUIAI EDD sitemap origin read 200")
check(reads["https://raw.githubusercontent.com/Startempire-Wire/focusa/main/LICENSE.md"]["status"] == 200, "repo LICENSE.md origin read 200")
check(reads["https://raw.githubusercontent.com/Startempire-Wire/focusa/main/COMMERCIAL.md"]["status"] == 200, "repo COMMERCIAL.md origin read 200")

# ---------------------------------------------------------------------------
# 6. Canonical claims: every verified claim has bounded evidence.
# ---------------------------------------------------------------------------
claims = {row["code"]: row for row in PROOF["canonical_claims"]}
check(len(claims) == PROOF["proof_index"]["verified_claim_count"], "verified claim count matches proof index")
check({c["code"] for c in PROOF["canonical_claims"]} == {f"V{i:02d}" for i in range(1, 11)}, "verified claim codes are V01..V10")
for row in PROOF["canonical_claims"]:
    check(row["status"] == "verified", f"{row['code']} status verified")
    check(len(row["bounded_claim"]) > 20, f"{row['code']} has a bounded claim")
    check(len(row["evidence_refs"]) >= 1, f"{row['code']} has at least one evidence ref")

# ---------------------------------------------------------------------------
# 7. Remaining contradictions: mapped to the baseline and Section 18.2.
# ---------------------------------------------------------------------------
remaining = {row["code"]: row for row in PROOF["remaining_contradictions"]}
check(len(remaining) == PROOF["proof_index"]["remaining_contradiction_count"], "remaining contradiction count matches proof index")
baseline_codes = {row["code"] for row in BASELINE["observed_claims"]}
replacement_map = {row["current"]: row["required"] for row in BASELINE["required_replacements"]}
check({"R01", "R02", "R03", "R04", "R05", "R06", "R07", "R08", "R09"} == set(remaining), "remaining contradiction codes are R01..R09")
for row in PROOF["remaining_contradictions"]:
    check(row["status"] == "remaining", f"{row['code']} status remaining")
    check(len(row["blocker"]) > 10, f"{row['code']} records a blocker")
    check(len(row["evidence_refs"]) >= 1, f"{row['code']} has evidence refs")
# every baseline contradiction is classified (verified or remaining) exactly once
classified = {row.get("baseline_code") for row in PROOF["canonical_claims"]} | {
    r["baseline_code"] for r in PROOF["remaining_contradictions"] if r.get("baseline_code")
}
check(baseline_codes <= classified, "every baseline observed claim is classified")
# prices and legacy-separation baselines are verified; the rest remain
check("standalone_prices_697" not in {r.get("baseline_code") for r in PROOF["remaining_contradictions"]},
      "standalone prices baseline is not listed as remaining")
check("legacy_wpuiai_prices" not in {r.get("baseline_code") for r in PROOF["remaining_contradictions"]},
      "legacy WPUIAI separation baseline is not listed as remaining")
for code in ("conflicting_bundle_1097", "anonymous_local_evaluation", "no_phone_home",
             "gravity_stripe_positioning", "broken_routes_and_license_links"):
    check(code in {r.get("baseline_code") for r in PROOF["remaining_contradictions"]}, f"baseline {code} remains")
# Section 18.2 required corrections preserved verbatim from the baseline
r02 = remaining["R02"]
check(r02["section_18_2_current"] == "Bundle advertised at $1,097", "R02 current claim matches Section 18.2")
check(r02["required"] == replacement_map["Bundle advertised at $1,097"], "R02 required correction matches baseline")
check(r02["required"] == "Bundle price $1,254.60", "R02 required correction is $1,254.60")
r03 = remaining["R03"]
check(r03["required"].startswith("No telemetry; bounded authority communication"), "R03 required correction is bounded-authority copy")
r04 = remaining["R04"]
check("WPUIAI EDD" in r04["required"], "R04 required correction is WPUIAI EDD")
check(remaining["R01"]["required"].startswith("Verified mailbox required"), "R01 required correction is verified mailbox")
check(remaining["R08"]["required"].startswith("Only UIAI License Type, Bundle"), "R08 required correction is product-isolated UIAI")
check(remaining["R09"]["required"].startswith("Permanent verified no-license"), "R09 required correction is permanent limited mode")
# canonical Bundle formula appears in authority and R02, never as a live price
check("1254.60" in PROOF_RAW, "canonical Bundle price present in the packet")

# ---------------------------------------------------------------------------
# 8. Deterministic proof index: digest is recomputed and must match.
# ---------------------------------------------------------------------------
index = PROOF["proof_index"]
digest_input = json.dumps(
    PROOF["canonical_claims"] + PROOF["remaining_contradictions"],
    sort_keys=True, separators=(",", ":"),
).encode("utf-8")
recomputed = hashlib.sha256(digest_input).hexdigest()
check(recomputed == index["claims_digest_sha256"], "recomputed claims digest matches the frozen digest")
check(len(index["claims_digest_sha256"]) == 64, "digest is a 64-hex sha256")
check(len(index["index_rows"]) == index["verified_claim_count"] + index["remaining_contradiction_count"],
      "index rows cover every claim")
indexed = {row["code"] for row in index["index_rows"]}
check(indexed == set(claims) | set(remaining), "index rows equal the claim codes")
for row in index["index_rows"]:
    kind = "verified" if row["code"].startswith("V") else "remaining"
    check(row["kind"] == kind, f"index row {row['code']} kind matches code prefix")
    check(len(row["evidence_refs"]) >= 1, f"index row {row['code']} has evidence refs")

# ---------------------------------------------------------------------------
# 9. Cross-check with the accepted convergence contract.
# ---------------------------------------------------------------------------
conv_types = {r["public_code"]: r for r in CONVERGENCE["canonical_policy"]["license_types"]}
check(prices["focusa_operator_lifetime_v1"] == conv_types["focusa_operator_lifetime_v1"]["price_usd"], "authority price matches convergence contract")
check(prices["uiai_operator_lifetime_v1"] == conv_types["uiai_operator_lifetime_v1"]["price_usd"], "authority UIAI price matches convergence contract")
check(prices["focusa_uiai_operator_bundle_lifetime_v1"] == conv_types["focusa_uiai_operator_bundle_lifetime_v1"]["price_usd"], "authority Bundle price matches convergence contract")
plan = CONVERGENCE["uiai_browser_proof_plan"]
check(plan["owner"] == "focusa-vbcqu.20.15.39 (Capture UIAI public pricing, route, and contradiction-removal proof)", "proof plan owner matches this atom")
check(plan["method"] == "uiai_engine_browser_read", "proof plan method matches capture method")
check(plan["authenticated_capture"] is False, "proof plan requires no authenticated capture")

# ---------------------------------------------------------------------------
# 10. Fail-closed validator: the packet cannot claim removal without evidence.
# ---------------------------------------------------------------------------
def validate_packet(candidate: dict) -> None:
    """Accept only a redacted, deterministic, honest packet."""
    assert candidate["schema"] == "focusa.spec172_uiai_public_proof.v1"
    assert candidate["evidence_posture"]["public_copy_is_entitlement_authority"] is False
    assert candidate["evidence_posture"]["raw_capture_embedded"] is False
    assert candidate["evidence_posture"]["authenticated_or_customer_data_capture"] is False
    claims_sec = candidate["canonical_claims"] + candidate["remaining_contradictions"]
    digest = hashlib.sha256(
        json.dumps(claims_sec, sort_keys=True, separators=(",", ":")).encode("utf-8")).hexdigest()
    assert digest == candidate["proof_index"]["claims_digest_sha256"]
    for row in candidate["remaining_contradictions"]:
        assert row["status"] == "remaining"
        assert len(row["blocker"]) > 10
        assert len(row["evidence_refs"]) >= 1
    assert {r["code"] for r in candidate["remaining_contradictions"]} >= {"R02", "R03"}
    assert any("$1,097" in r["bounded_claim"] for r in candidate["remaining_contradictions"])


def denied(mutator, message):
    candidate = json.loads(PROOF_RAW)
    mutator(candidate)
    try:
        validate_packet(candidate)
    except (AssertionError, KeyError, TypeError):
        return
    raise AssertionError(message)


denied(lambda c: c["remaining_contradictions"].__delitem__(0),
       "dropping a remaining contradiction accepted")
denied(lambda c: c["remaining_contradictions"][0].update({"status": "verified"}),
       "marking a remaining contradiction verified without evidence accepted")
denied(lambda c: c["proof_index"].update({"claims_digest_sha256": "0" * 64}),
       "frozen digest mismatch accepted")
denied(lambda c: c["remaining_contradictions"][0].update({"bounded_claim": "Bundle now shown at $1,254.60"}),
       "claiming Bundle removal without live evidence accepted")
denied(lambda c: c["evidence_posture"].update({"raw_capture_embedded": True}),
       "raw capture embedded accepted")
denied(lambda c: c["canonical_claims"][0].update({"bounded_claim": "live price is $1,097.00"}),
       "contradictory price recorded as verified accepted")

# ---------------------------------------------------------------------------
# 11. Hygiene: no raw email, key, token, customer row, credential, or card data.
# ---------------------------------------------------------------------------
for forbidden in ("customer_email", "license_key", "access_token", "cookie", "card_number",
                  "cvv", "stripe_payment_intent", "customer_id", "password", "authorization header"):
    check(forbidden not in PROOF_RAW, f"no {forbidden} in proof packet")
check(EMAIL_RE.search(PROOF_RAW) is None, "no email addresses in proof packet", "negative")
check(SECRET_RE.search(PROOF_RAW) is None, "no secret-shaped values in proof packet", "negative")
check(LICENSE_KEY_RE.search(PROOF_RAW) is None, "no license-shaped evidence in proof packet", "negative")
check(CARD_RE.search(PROOF_RAW) is None, "no card-number-shaped evidence in proof packet", "negative")
check("authenticated capture" not in PROOF_RAW.lower() or "authenticated_or_customer_data_capture\": false" in PROOF_RAW,
      "no authenticated capture claim", "negative")

result = {
    "schema": "focusa.spec172_uiai_public_proof_validation.v1",
    "verified_claims": PROOF["proof_index"]["verified_claim_count"],
    "remaining_contradictions": PROOF["proof_index"]["remaining_contradiction_count"],
    "uiai_sessions": PROOF["proof_index"]["uiai_session_count"],
    "origin_reads": PROOF["proof_index"]["origin_read_count"],
    "claims_digest_sha256": PROOF["proof_index"]["claims_digest_sha256"],
    "positive_checks": positive,
    "negative_checks": negative,
    "result": "passed_fail_closed",
}
print(json.dumps(result, sort_keys=True))
print("Spec 172 UIAI public pricing, route, and contradiction-removal proof: PASS")
