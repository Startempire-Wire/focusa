# Vendor-side Close-out (V1 + V2 + V3) — 2026-07-07

**Status:** V1, V2, V3 implemented and verified end-to-end against the
live `wpuiai.com` registry. V4 (cosign hard prereq) intentionally NOT
touched per operator directive.

## What changed in the WordPress plugin

New file: `wpuiai-ai-cloud-admin/includes/class-focusa-license-production.php`

Modified files:
- `wpuiai-ai-cloud-admin/wpuiai-ai-cloud-admin.php` — loads the new
  class and runs the install() hook
- `wpuiai-ai-cloud-admin/includes/class-settings.php` — the existing
  `rest_validate_license` now reads `X-Machine-Id` and calls
  `machine_seat_check()` before responding

## V1 — Promote validation out of dev_mode

How to promote: `wp aic focusa promote` (or
`POST /wp-json/wpuiai-ai-cloud/v1/focusa/production/promote` with
`confirm="yes-i-take-responsibility"`).

What it does:
- Sets `uiai_dev_mode = '0'` (real EDD license row lookup runs)
- Returns before/after readiness check

Readiness check (`wp aic focusa check` /
`GET /focusa/production/check`):
- counts active license rows in `wp_edd_licenses`
- shows current `dev_mode` flag and seat cap
- recommends `safe_to_promote` only when dev_mode is on AND at least one
  real active row exists

Test verified: with `dev_mode = 0`, a POST to
`/license/validate` with a real key returns
`{valid: true, license_id: N, status: active, ...}`. With dev_mode = 1,
the same endpoint returns `status: dev_mode` (operator test fixture).

## V2 — License issuance (Stripe webhook shape)

New endpoint:
`POST /wp-json/wpuiai-ai-cloud/v1/focusa/license/issue`
- body: `{email, download_id, payment_id, tier}`
- headers: `X-Webhook-Secret: <wp aic_webhook_secret>`
- side effects:
  - creates a stub `wp_edd_customers` row if email is new
  - inserts a row in `wp_edd_licenses` with
    `status=active, license_key=focusa_live_<download_id>_<16 hex>`
  - writes `focusa_email`, `focusa_tier`, `focusa_issued_via` meta into
    `wp_wpuiai_license_meta` (idempotent on `(license_id, meta_key)`)
  - schedules the key for email delivery via a transient

Hook on `edd_complete_purchase` (priority 20) so any real
EDD purchase auto-mints a focusa_live_ key if the download_id maps to
a license-tier product.

CLI: `wp aic focusa issue <email> <download_id> [--tier=...]`

Test verified:
- issued `focusa_live_453_38c07350702dc58c` via REST
- issued `focusa_live_453_f7c35e572135c673` via REST
- issued `focusa_live_453_6b55023590d14f68` via REST
- all three are in `wp_edd_licenses`, `status=active`

## V3 — Per-machine seat enforcement

Schema: `wp_wpuiai_license_machines` (id, license_id, machine_id,
machine_label, registered_at, last_seen_at, revoked_at, source)
with `UNIQUE(license_id, machine_id)` and indexes on
`(license_id)`, `(machine_id)`, `(revoked_at)`.

Behavior in `rest_validate_license`:
- Reads `X-Machine-Id` header
- Existing active row → refresh `last_seen_at`, return success
- New machine:
  - count active seats (license_id, revoked_at IS NULL)
  - if `< cap` (default 3, configurable via
    `wpuiai_aic_seat_cap_per_license` option) → insert row, return
    success
  - if `>= cap` → return `valid: false, status: revoked, reason:
    seat_cap_reached, cap: N, active: M` with HTTP 403

Test verified (cap=3, license_id=11):
- `smoke-A` → valid (registered, active=1)
- `smoke-B` → valid (registered, active=2)
- `smoke-C` → valid (registered, active=3)
- `smoke-D` → blocked (`status=revoked, cap=3, active=3`)

In-repo `focusa license refresh` picks up the seat-cap rejection
correctly via the existing `RegistryValidateOutcome` error path:
```
[refresh] step=registry_post status=blocked registry_status=revoked
[refresh] recovery_hint: "registry reports license state 'revoked'.
Run `focusa license activate <KEY>` with a current key, or contact
https://wpuiai.com/wp-admin for reissue."
```

Revoke a machine explicitly:
- `wp aic focusa revoke-machine <license_id> <machine_id>`
- `POST /focusa/license/revoke-machine` with `X-Webhook-Secret`

## V4 — NOT touched per operator directive

Cosign signature verification as a hard prerequisite is deferred. The
in-repo `scripts/install-focusa.sh` and Rust installer still
warn-and-fallback when cosign is missing; this is documented as
vendor V4 in `docs/INSTALL_PURCHASE_ACHIEVEMENTS_2026-07-07.md` and
left untouched.

## Current server state

- `uiai_dev_mode = '1'` (operator test fixture active)
- `seat_cap_per_license = 3` (default; configurable)
- `wp_wpuiai_license_machines` table created
- `wp_wpuiai_license_meta` table created
- 7 active license rows in `wp_edd_licenses`
- 3 issued `focusa_live_*` keys (ids 8, 9, 10)
- 1 license row with 3 enrolled machines (smoke-A, smoke-B, smoke-C)

To run the production flow: `wp aic focusa promote` (or POST to
`/focusa/production/promote` with `confirm="yes-i-take-responsibility"`).
To roll back: `wp option patch update uiai_dev_mode 1`.

## Acceptance test

```bash
# V1
wp aic focusa promote           # dev_mode = 0
wp aic focusa check             # safe_to_promote, dev_mode=false
# V2
SECRET=$(wp option get wpuiai_aic_webhook_secret)
curl -s -X POST -H "X-Webhook-Secret: $SECRET" \
  -H "Content-Type: application/json" \
  -d '{"email":"buyer@x.com","download_id":453,"payment_id":1,"tier":"operator"}' \
  https://wpuiai.com/wp-json/wpuiai-ai-cloud/v1/focusa/license/issue
# V3 — fill seats
for M in a b c; do
  curl -s -X POST -H "X-License-Key: $KEY" -H "X-Machine-Id: $M" \
    -H "Content-Type: application/json" -d "{\"license_key\":\"$KEY\"}" \
    https://wpuiai.com/wp-json/wpuiai-ai-cloud/v1/license/validate
done
# V3 — 4th machine refuses
curl -s -X POST -H "X-License-Key: $KEY" -H "X-Machine-Id: d" \
  -H "Content-Type: application/json" -d "{\"license_key\":\"$KEY\"}" \
  https://wpuiai.com/wp-json/wpuiai-ai-cloud/v1/license/validate
# Expect: status=revoked, reason=seat_cap_reached, cap=3, active=3
```
