#!/usr/bin/env python3
"""Fail-closed checks for Focusa public FAQ copy and externally released identity claims.

This tool is a policy/CI gate. It DOES NOT send email, approve messages or
change WordPress. Outbound connectors must call it and separately enforce
authorized human approval before dispatch.
"""
from __future__ import annotations

import argparse
import html
import hashlib
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DIR = ROOT / "docs" / "public" / "buyer-faq"
FAQ = DIR / "buyer-faq.v1.json"
FACTS = DIR / "authority.v1.json"
PAGE = DIR / "wordpress-page-14.proposed.html"
JSONLD = DIR / "FAQPage.v1.jsonld"

# Intentionally matches only existing WordPress accordion structure; do not
# regenerate a page in a way that silently damages theme/Gutenberg markup.
ACC = re.compile(
    r"(?P<open><details>\s*<summary>)(?P<question>.*?)(?P<middle></summary>\s*<p>)(?P<answer>.*?)(?P<close></p>\s*</details>)",
    re.DOTALL,
)
OLD_INTRO = (
    "These are questions that real buyers asked before deciding whether Focusa "
    "was right for their business. We removed company-specific details so every "
    "buyer can use the answers."
)
NEW_INTRO = (
    "Questions prospective buyers ask about Focusa, written in general terms to protect "
    "customer-specific details. Each answer distinguishes established business facts "
    "from proposed configurations and deployment-specific proof."
)


def read(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def rows(doc: str) -> list[tuple[str, str]]:
    return [
        (html.unescape(m.group("question")).strip(),
         html.unescape(m.group("answer")).strip())
        for m in ACC.finditer(doc)
    ]


def validate_contract() -> list[str]:
    errors: list[str] = []
    faq, facts, ld = read(FAQ), read(FACTS), read(JSONLD)
    items = faq.get("questions", [])
    if len(items) != 55 or faq.get("page", {}).get("question_count") != 55:
        errors.append("FAQ scope violation: expected 55 entries")
    if len({i.get("section") for i in items}) != 9:
        errors.append("Section scope violation: expected nine groups")
    if len({i.get("question") for i in items}) != 55:
        errors.append("FAQ questions are not unique")
    if sum(i.get("review_state") == "proposed_correction" for i in items) != 13:
        errors.append("Unexpected number of corrections: owner review needed")
    if sum(i.get("review_state") == "retained_from_verified_live_page" for i in items) != 42:
        errors.append("Unreviewed silent FAQ changes detected")
    for idx, item in enumerate(items, start=1):
        if item.get("id") != f"focusa-faq-{idx:03d}" or item.get("order") != idx:
            errors.append(f"FAQ ID/order drift at entry {idx}")
        if not item.get("answer", "").strip():
            errors.append(f"Empty answer at entry {idx}")
        if item.get("review_state") == "proposed_correction" and not re.fullmatch(r"[a-f0-9]{64}", item.get("source_answer_sha256", "")):
            errors.append(f"Missing original answer digest required for safe WordPress patch: {idx}")
    p = facts.get("identity", {}).get("person", {})
    corp = facts.get("identity", {}).get("software_company", {})
    agency = facts.get("identity", {}).get("philoveracity", {})
    if p.get("name") != "Verious Smith III" or "Creator of Focusa" not in p.get("role", []) or "Principal architect of Focusa" not in p.get("role", []):
        errors.append("Owner/architect identity has been lost")
    if corp.get("name") != "Startempire Wire LLC" or corp.get("owner") != "Verious Smith III":
        errors.append("Founder company ownership has been changed")
    if set(corp.get("products_owned_and_developed", [])) != {"Focusa", "UIAI Engine"}:
        errors.append("Software company product ownership changed")
    if agency.get("owner") != "Verious Smith III" or agency.get("operating_since") != 2010 or agency.get("dba") is not True:
        errors.append("Philoveracity history/ownership/DBA lost")
    if agency.get("dba_legal_registrant") != "not_verified_for_public_contract":
        errors.append("DBA registrant asserted without legal verification")
    comm = facts.get("commercial", {})
    if comm.get("operator_deployment_usd") != 5000 or comm.get("operator_success_months") != 12 or comm.get("software_licenses_separate") is not True:
        errors.append("Published commercial scope was silently changed")
    by_q = {i["question"]: i for i in items}
    important = [
        ("Who owns and develops Focusa and UIAI Engine?", ["Verious Smith III", "creator", "principal", "Startempire Wire LLC", "UIAI Engine"]),
        ("What is Philoveracity’s role?", ["Verious Smith III", "2010", "DBA"]),
        ("What is the implementer authorized to do?", ["Verious Smith III", "creator", "architect", "Startempire Wire LLC"]),
    ]
    for q, required in important:
        answer = by_q.get(q, {}).get("answer", "")
        for phrase in required:
            if phrase.casefold() not in answer.casefold():
                errors.append(f"Ownership FAQ missing {phrase!r}: {q}")
    page = PAGE.read_text(encoding="utf-8")
    current_rows = rows(page)
    if len(current_rows) != len(items):
        errors.append(f"WordPress proposed page differs in count: {len(current_rows)}")
    for idx, (actual, item) in enumerate(zip(current_rows, items), start=1):
        if actual != (item["question"], item["answer"]):
            errors.append(f"WordPress/JSON human-visible copy mismatch at {idx}")
    if OLD_INTRO in page or NEW_INTRO not in page:
        errors.append("Public buyer FAQ intro remains misleading or is missing")
    entities = ld.get("mainEntity", [])
    if ld.get("@type") != "FAQPage" or len(entities) != len(items):
        errors.append("Schema.org FAQPage entry count mismatch")
    for idx, (node, item) in enumerate(zip(entities, items), start=1):
        if node.get("name") != item["question"] or node.get("acceptedAnswer", {}).get("text") != item["answer"]:
            errors.append(f"JSON-LD/visible FAQ mismatch at {idx}")
    if ld.get("author", {}).get("name") != "Verious Smith III":
        errors.append("FAQPage lost creator attribution")
    if any(tok in page for tok in ("OWNER DRAFT NOTE", "[[CONFIRM BEFORE SEND", "%d")):
        errors.append("Public page contains internal placeholder or theme token")
    return errors


def patch_wordpress(existing: str) -> str:
    """Patch the exact existing WordPress HTML; reject any identity/scope drift."""
    faq = read(FAQ)
    qs = faq["questions"]
    matches = list(ACC.finditer(existing))
    if len(matches) != len(qs):
        raise ValueError(f"Refuse WordPress patch: {len(matches)} FAQ blocks, expected {len(qs)}")
    for i, (match, q) in enumerate(zip(matches, qs), start=1):
        found_q = html.unescape(match.group("question")).strip()
        found_a = html.unescape(match.group("answer")).strip()
        if found_q != q["question"]:
            raise ValueError(f"Refuse WordPress patch: question {i} changed or moved")
        original_digest = hashlib.sha256(found_a.encode("utf-8")).hexdigest()
        if found_a != q["answer"] and original_digest != q.get("source_answer_sha256"):
            raise ValueError(f"Refuse WordPress patch: answer {i} has unseen concurrent changes")
    it = iter(qs)
    def substitute(match: re.Match) -> str:
        q = next(it)
        if q.get("review_state") != "proposed_correction":
            return match.group(0)
        # No HTML is inserted from model-generated FAQ answer strings.
        safe = html.escape(q["answer"], quote=False)
        return match.group("open") + match.group("question") + match.group("middle") + safe + match.group("close")
    updated = ACC.sub(substitute, existing)
    if OLD_INTRO in updated:
        updated = updated.replace(OLD_INTRO, NEW_INTRO, 1)
    elif NEW_INTRO not in updated:
        raise ValueError("Refuse WordPress patch: intro has diverged")
    if rows(updated) != [(x["question"], x["answer"]) for x in qs]:
        raise ValueError("Round-trip parity failed; refusing to publish")
    return updated


def validate_outbound(message: str) -> list[str]:
    """Secondary semantic guard; real sender MUST enforce human approval too."""
    errors: list[str] = []
    lowered = message.casefold()
    if any(x.casefold() in lowered for x in ("OWNER DRAFT NOTE", "[[CONFIRM BEFORE SEND", "[[CONFIRM")):
        errors.append("Unreviewed internal/editorial placeholder in outgoing message")
    wrong = [
        r"\b(?:only|merely|just|simply)\s+(?:an?\s+)?(?:commercial\s+)?licens(?:e|ing)\s+contact\b",
        r"\bPhiloveracity\s+is\s+(?:an?\s+)?unrelated\b",
        r"\bVerious\s+(?:Smith(?:\s+III)?)?\s+did\s+not\s+(?:create|architect|develop)\s+Focusa\b",
    ]
    if any(re.search(pattern, message, re.IGNORECASE) for pattern in wrong):
        errors.append("Forbidden/contradictory ownership shortcut")
    # A draft that explains who owns, created, or develops Focusa cannot
    # merely list a licensing contact. A generic product-price email is not
    # forced to include a biography.
    mentions_focusa = "focusa" in lowered
    identity_claim = bool(re.search(
        r"\b(?:who\s+owns|owned\s+by|owns\s+focusa|creator|created\s+focusa|architect|developer\s+of|"
        r"develops\s+focusa|commercial\s+licensing\s+contact|philoveracity\s+is|company\s+that\s+owns)\b",
        lowered))
    if mentions_focusa and identity_claim:
        for phrase in ("Verious Smith", "creator", "architect", "Startempire Wire"):
            if phrase.casefold() not in lowered:
                errors.append(f"Identity claim omits mandatory founder fact: {phrase}")
        if "uiai engine" in lowered and "startempire wire" not in lowered:
            errors.append("UIAI Engine company relationship omitted")
        if "philoveracity is" in lowered and "2010" not in lowered:
            errors.append("Philoveracity agency/DBA history omitted")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Verify machine FAQs, public HTML and JSON-LD are identical")
    parser.add_argument("--outbound-file", type=Path, help="Check a message for identity omissions; no sending is performed")
    parser.add_argument("--patch-wordpress", type=Path, help="Existing WP-CLI exported post_content, read-only input")
    parser.add_argument("--output", type=Path, help="Write a proposed patched WP HTML file; does not publish")
    args = parser.parse_args()
    if not (args.check or args.outbound_file or args.patch_wordpress):
        parser.error("Choose --check, --outbound-file, or --patch-wordpress")
    errors = validate_contract() if args.check or args.patch_wordpress else []
    if args.outbound_file:
        errors.extend(validate_outbound(args.outbound_file.read_text(encoding="utf-8")))
    if args.patch_wordpress and not errors:
        if not args.output:
            parser.error("--patch-wordpress requires --output")
        try:
            corrected = patch_wordpress(args.patch_wordpress.read_text(encoding="utf-8"))
            if args.output.resolve() == args.patch_wordpress.resolve():
                raise ValueError("Output must be different from source to preserve backup")
            args.output.write_text(corrected, encoding="utf-8")
        except ValueError as err:
            errors.append(str(err))
    for err in errors:
        print("BLOCK:", err, file=sys.stderr)
    if errors:
        return 2
    print("PASS: FAQ contract and outbound identity checks (as requested)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
