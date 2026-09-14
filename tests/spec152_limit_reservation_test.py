#!/usr/bin/env python3
"""Static/source acceptance for durable atomic entitlement limit reservations."""

from pathlib import Path

persistence = Path("crates/focusa-core/src/runtime/persistence_sqlite.rs").read_text()
middleware = Path("crates/focusa-api/src/middleware/entitlement.rs").read_text()

for marker in [
    "CREATE TABLE IF NOT EXISTS entitlement_limit_reservations",
    "reservation_id TEXT PRIMARY KEY",
    "status IN ('reserved', 'committed', 'released')",
    "connection.transaction()?",
    "SELECT COALESCE(SUM(units), 0)",
    "consumed.saturating_add(units) > available",
    "entitlement reservation idempotency conflict",
    "settle_entitlement_limit",
]:
    assert marker in persistence, marker

reserve = middleware.index("reserve_route_limit(&state, &request)")
execute = middleware.index("next.run(request).await", reserve)
settle = middleware.index("settle_entitlement_limit", execute)
assert reserve < execute < settle, "reservation is not pre-side-effect with post-result settlement"
for marker in [
    'get("Idempotency-Key")',
    "ENTITLEMENT_IDEMPOTENCY_REQUIRED",
    "ENTITLEMENT_RESERVATION_FAILED",
    "Sha256::digest",
    "response.status().is_success()",
]:
    assert marker in middleware, marker
assert "HashMap" not in middleware, "limit reservations are process-local instead of durable"

print("Spec152 durable entitlement limit reservation gate: PASS")
