#!/usr/bin/env python3
"""Spec 135I-3 nontechnical onboarding journey proof."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
SRC=(ROOT/"apps/pi-extension/src/nontechnical-onboarding.ts").read_text()
INDEX=(ROOT/"apps/pi-extension/src/index.ts").read_text()
DOG=json.loads((ROOT/"docs/contracts/spec135/generated-contract-v1/spec135-alpha8-nontechnical-dogfood-proof.json").read_text())
assert 'registerCommand("focusa-start"' in SRC
assert "registerNontechnicalOnboarding(pi)" in INDEX
assert 'registerMessageRenderer("focusa-onboarding"' in INDEX
for phrase in ("Continue my project","Start guided setup","Add project documents","Review saved answers","Recover a paused setup"):
    assert phrase in SRC
for law in ("will not be asked for them again","pause safely and resume","Advanced details stay hidden","safe recovery action"):
    assert law in SRC
for forbidden in ("curl ","/v1/","continuity_id","idempotency_key","stack trace"):
    assert forbidden not in SRC, forbidden
assert DOG["requirement_id"] == "SPEC135-ALPHA8"
assert DOG["acceptance"]
assert all(DOG["acceptance"].values())
print("Spec 135 I3 nontechnical onboarding journey: PASS")
