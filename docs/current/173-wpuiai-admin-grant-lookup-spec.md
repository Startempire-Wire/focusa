# 173 — WPUiai Admin Grant + Frontend Lookup Spec (Focusa • UIAI Engine • EDD • WordPress)

**Status:** draft — investigation complete, awaiting Sir V3 approval before scaffold
**Covers:** `wpuiai.com` EDD → Focusa / UIAI Engine entitlement grant without ceremony, WordPress admin UI, CLI grant, and frontend lookup/retrieval on every surface
**Depends:** Spec 152E product registry (`docs/contracts/spec152e-edd-product-registry.v1.yaml`), Spec 172 Operator/Bundle (`docs/contracts/spec172-edd-operator-products.v1.yaml`), `spec152e-activation-call-stack.v1.yaml`, `activation_http.rs` / `activation_client.rs` / `entitlement.rs` / `orchestrator.rs` home-bypass `b576c03b3`

---

## 1. Investigation Summary (connecting parts verified)

**EDD server (wpuiai.com):** Prices `focusa_operator_lifetime_v1 697.00`, `uiai_operator_lifetime_v1 697.00`, `focusa_uiai_operator_bundle_lifetime_v1 1254.60` (minor 69700/125460), `operator_shared_v1` 1 seat / 3 nodes, whole-order 30-day refund. Authority base `https://wpuiai.com/wp-json/wpuiai-ai-cloud/v1/` (`/wp-json/wpuiai-ai-cloud/v1/license/validate`, `/license/status?license_key=`, `/authority/` namespace). Current issuance is checkout → order → projection → license → signed lease; admin has no one-click grant.

**Focusa runtime:** `crates/focusa-license/src/activation_http.rs` frozen constants `/v1/activation/start` (relative join) + `X-Request-Id`/`Idempotency-Key` + `404/405 → AUTHORITY_UNAVAILABLE`. `activation_client.rs` `ActivationSession::begin()` consumes `registration_id` from authority; `orchestrator.rs` `validate_request_at()` enforces `entitlement` for install/update; `entitlement.rs` `entitlement_gate_layer` denies `ENTITLEMENT_BASE_REQUIRED`. Home-bypass patch `b576c03b3` adds `is_home_dev_bypass() → FOCUSA_DEV_MODE|FOCUSA_HOME_SERVER|FOCUSA_TEST_MODE==1` early return — cargo/daemon/CLI no longer block home.

**WordPress:** `WP-Admin` `EDD → Customers/Orders/Licenses` list tables, `manage_options` cap, `WP_List_Table`, `admin_enqueue_scripts` React panel — canonical pattern.

**Lookup surfaces:** `GET /wp-json/wpuiai-ai-cloud/v1/license/verify?email=&key=`, `GET /wp-json/wpuiai-ai-cloud/v1/license/status?license_key=`, daemon `GET /v1/license/status`, CLI `focusa license status --json`, Pi tool `focusa_license_status` — all read `SignedEnvelope` lease + projection.

---

## 2. Objective

Sir V3 can grant **any license (Focusa / UIAI Engine / Bundle) to any email in <2 clicks, no checkout/poll/verification ceremony**, and grant is **immediately queryable by frontend lookup/retrieval** (wpuiai.com frontend, Focusa app, UIAI Engine, CLI, Pi) with idempotency, audit, and revocation.

---

## 3. Server Contract (wpuiai.com)

### 3.1 Admin grant endpoint
```
POST /wp-json/wpuiai-ai-cloud/v1/admin/grant-license
Auth: WordPress Application Password (admin) + capability manage_options
Headers: X-Request-Id (uuid), X-Idempotency-Key (required)
Body: {
  "email": "user@example.com",
  "product_code": "focusa_operator_lifetime_v1 | uiai_operator_lifetime_v1 | focusa_uiai_operator_bundle_lifetime_v1",
  "price_version": "focusa_operator_lifetime_v1.697.00.v1",
  "grant_reason": "admin_grant",
  "operator_seats": 1,
  "idempotency_key": "uuid"
}
Responses:
  201 { license_id, license_key, lease: SignedEnvelope, projection_id, grants, node_limit: 3, evidence_ref: "sha256:..." }
  200 same on idempotent replay (same Idempotency-Key + same email+product)
  400 E_PRODUCT_UNKNOWN | E_PRICE_MISMATCH | E_EMAIL_UNMASKABLE
  409 E_IDEMPOTENCY_CONFLICT
```
Atomic DB txn: `edd_customers` (find-or-create) → `edd_orders` (complete) → `edd_order_items` → `licenses` → `focusa_projections` (sequence 1) → `SignedEnvelope` lease. Meta `focusa_grant_source=admin`.

### 3.2 Complementary endpoints
```
POST /wp-json/wpuiai-ai-cloud/v1/admin/revoke-license  { email, license_id, reason }
POST /wp-json/wpuiai-ai-cloud/v1/admin/refund-order     { email, order_id, whole_order: true }
GET  /wp-json/wpuiai-ai-cloud/v1/admin/license?email=&product_code=
```

### 3.3 Lookup/retrieval (public)
```
GET /wp-json/wpuiai-ai-cloud/v1/license/verify?email=&license_key= → { valid, grants, seats, nodes, lease_digest }
GET /wp-json/wpuiai-ai-cloud/v1/license/status?license_key= → { state: active|revoked|refunded, lease }
GET /v1/license/status (Focusa daemon) → same
```
Guarantee: after 201 grant, verify+status return valid:true within 2s. Edge cache `private, max-age=60, ETag: <lease_digest>` invalidated on revoke/refund.

---

## 4. WordPress Admin UI

**Placement:** `WP-Admin → Focusa • UIAI Licenses` (top-level, dashicons-admin-network, position 31, below EDD). Tabs: `Grant | Manage | UIAI Nodes | Bundles | Audit` — `WP_List_Table` + React via `admin_enqueue_scripts`.

**Grant tab:** Header `Grant any license instantly — no checkout`. Form: Email [type-ahead EDD customers + WP users] | Product [cards: Focusa $697 / UIAI $697 / Bundle $1,254.60] | Reason | [Grant]. On submit: `fetch(grant-license, { X-Idempotency-Key })` → toast `Granted — key f… (copy)` + `View lease JSON` + `Verify` button that calls `verify` and shows green `Verified`.

**Manage table:** `EDD Licenses` adds column `Lease` + row actions `Verify | Revoke | Refund whole order`.

**UIAI Nodes:** List `node_id, last_seen, grant` for 3 shared nodes, per-node revoke.

**Audit:** `focusa_admin_grant_log` table: `at, admin_user, email, product_code, license_id, evidence_ref, request_id` — export CSV.

---

## 5. Focusa / UIAI Engine Integration

**CLI:** `focusa admin grant-license --email <e> --product <code> [--reason] [--registry https://wpuiai.com] [--json]` — uses `WPUIAI_ADMIN_TOKEN` env, same headers. Also `focusa license status --email --key`.

**Daemon:** `GET /v1/license/status` already serves grant; home-bypass ensures `FOCUSA_HOME_SERVER=1` never blocks.

**Pi/TUI:** `focusa_license_status` tool + TUI `License` panel poll `verify` — grant appears without restart.

**Mapping:** Reuses `spec152e-edd-product-registry` price_versions; drift fails `E_PRICE_MISMATCH`. Bundle writes two grants with `exact_union` digest `80d4034f...`.

---

## 6. Frontend Lookup Systems

- `wpuiai.com/check-license` — email+key → `fetch(verify)` → `Valid / Grants / Seats / Nodes`.
- Checkout `thank-you` — `fetch(status?license_key=)` to gate `Download Focusa`.
- `app.focusa.dev` — `GET /v1/license/status` on launch → `Activated` badge.
- UIAI Engine FPV — `verify` for `uiai_operator_lifetime_v1` gate.
- Cache via `lease_digest` ETag; grant invalidates so lookup fresh.

---

## 7. Security & Idempotency

- `manage_options` only; Application Password scoped.
- `X-Idempotency-Key` required, 7-day store, 409 on mismatch.
- `X-Request-Id` traced.
- Email masked before logs; never logs raw `license_key` — only `key_prefix 8 + digest`.

---

## 8. Acceptance (no scaffold until green)

- `php tests/spec173_admin_grant_lookup_test.php` — grant Focusa/UIAI/Bundle to 3 emails via admin endpoint (idempotent replay), then `verify` + `status` + `daemon license/status` + `focusa license status` each return `valid:true` within 2s, lease verifies with `embedded_production_trust_roots`.
- `python tests/spec173_admin_ui_contract_test.py` — asserts `WP-Admin → Focusa • UIAI Licenses` menu, Grant tab React panel, Manage lease column, `focusa_admin_grant_log` exists.
- `bash tests/spec173_home_bypass_gate.sh` — with `FOCUSA_HOME_SERVER=1`, `focusa bg run --name t -- sleep 1` + `cargo test --list` succeed without `ENTITLEMENT_*`.
- Manual: Sir V3 grants `focusa_operator_lifetime_v1` to `verious.smith@gmail.com` on staging, then `curl verify` + `focusa license status` green.

---

## 9. Rollout

Staging `wpuiai.com` (ovh-w1) → grant to `sirv3-verify@focusa.dev` → verify staging `verify` + `focusa-release` daemon with `FOCUSA_AUTHORITY_ORIGIN=https://staging.wpuiai.com` → production.
