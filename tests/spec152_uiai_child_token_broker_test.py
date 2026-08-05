#!/usr/bin/env python3
"""Build-independent contract gate for the authority-only UIAI child-token broker."""

from pathlib import Path

source = Path("crates/focusa-license/src/uiai_child_token.rs").read_text()
for marker in [
    "UiaiChildTokenRequest",
    "AuthorityChildTokenEnvelope",
    "UiaiChildTokenBroker",
    'active_bound(focusa_parent, "focusa"',
    'active_bound(uiai_grant, "uiai-engine"',
    "requested_features",
    "requested_limits",
    "NonceReplay",
    "UIAI_CHILD_TOKEN_MAX_TTL_MINUTES",
    "parent_lease_digest",
    "uiai_grant_sequence",
    "SensitiveCredential::new",
    "token_persisted_in_receipt: false",
    "revoke_parent",
]:
    assert marker in source, marker
for forbidden in ["SigningKey", "Signer", "self_sign", "customer_email", "access_token:"]:
    assert forbidden not in source, forbidden
assert source.index("validate_request(request") < source.index("SensitiveCredential::new")
assert source.index("self.accepted_nonces.insert") < source.index("self.cache.insert")
print("Spec152 UIAI authority child-token broker gate: PASS")
