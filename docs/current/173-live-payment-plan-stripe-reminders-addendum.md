# Live Additions 2026-08-18 — License ↔ Actual Software + Payment Plans + Stripe + Reminders

> On-the-fly doc so nothing is missed. Keep this as source of truth until specced formally.

## 1) Goal restated (no confusion)

- **Connect licenses to actual Focusa software** — every EDD license (`wp_edd_licenses` → `focusa_live_<id>_…`) must validate from the app/daemon via a stable read path.
- **Enter payments against price** when sold on installments/payment plan.
- **Payment reminders** if balance remains.
- **Stripe-aware** — Stripe knows about the plan/payments, not a shadow ledger.

These are 4 distinct layers. Doc keeps them separate.

---

## 2) Actual Software Connection (license truth)

**Where focusa lives:** `wpuiai.com` EDD Software Licensing owns `wp_edd_licenses` (`id, license_key, status, download_id, price_id, customer_id, user_id, expiration, date_created`) + `wp_edd_licensemeta` + `wp_edd_license_activations`.

**How the app registers/validates:**

- EDD standard: `?edd_action=activate_license / check_license / get_version` and modern REST under `edd-sl/v1` — licenses minted here pass those checks (status=active, expiration, activation limit).
- Vendor hardening (`class-focusa-license-production.php`): `wpuiai_aic_pre_validate_license` filter + `wpuiai_aic_focusa_license_issued` + per-machine seats table `wp_wpuiai_license_machines` (`license_id, machine_id, registered_at`) via `X-Machine-Id`.
- **New canonical read:** `GET /wp-json/wpuiai-ai-cloud/v1/license/verify?license_key=…` or `?email=…` — returns `focusa.license_verify.v1`:

```json
{
  "ok": true,
  "schema": "focusa.license_verify.v1",
  "license": { "id": 33, "download_id": 1736, "product": "Focusa Operator (Lifetime)", "status": "active", "price": 697, ... },
  "payment_plan": { "total_price": 697, "paid_amount": 200, "remaining_amount": 497, "installments_total": 3, "installments_paid": 1, "status": "active" },
  "seats": { ... license_truth ... }
}
```

The Focusa daemon/CLI should use `verify` as proof-of-registration. Masked `license_key` is returned; full key only when queried by key.

**Invariants:** `dev_mode` (`uiai_dev_mode=0` on prod + row must exist) never short-circuits prod. EDD row is canonical. No iframe admin — WP-Admin is source of truth.

---

## 3) Payment Plan Model (new tables)

**Plugin loader:** `wpuiai-ai-cloud-admin.php:552` now `require_once …/includes/class-license-payment-plan.php`.

**Tables (MyISAM, `dbDelta` in `WPUIAI_AIC_License_Payment_Plan::install()`):**

### `wp_wpuiai_license_payment_plans`
| column | type | notes |
|---|---|---|
| `id` | BIGINT UNSIGNED PK | |
| `license_id` | BIGINT UNSIGNED UNIQUE | FK logical → `edd_licenses.id` (1 plan per license) |
| `download_id` | BIGINT UNSIGNED | product at plan creation |
| `total_price` | DECIMAL(10,2) | authoritative price (from `edd_price` or admin input) |
| `currency` | VARCHAR(10) | default `USD` |
| `installments_total` | INT UNSIGNED | e.g. 3 |
| `installments_paid` | INT UNSIGNED | incremented on each record |
| `paid_amount` | DECIMAL(10,2) | sum recorded |
| `remaining_amount` | DECIMAL(10,2) | `total - paid`, 0 => completed |
| `status` | VARCHAR(20) | `single` (1-install), `active`, `overdue`, `completed` |
| `plan_type` | VARCHAR(20) | `manual` or `stripe` |
| `order_id` | BIGINT UNSIGNED NULL | EDD order if tied |
| `next_due_date` | DATE NULL | future use |
| `notes` | TEXT NULL | |
| `created_at/updated_at` | DATETIME | |

Transitions: `remaining <= 0.01 => completed`; `installments_paid >= installments_total && remaining>0 => overdue`.

### `wp_wpuiai_license_payments`
| column | type | notes |
|---|---|---|
| `id` | BIGINT PK | |
| `plan_id` | BIGINT | FK → plans.id |
| `license_id` | BIGINT | denorm for quick lookup |
| `installment_number` | INT NULL | sequential |
| `amount` | DECIMAL(10,2) | |
| `payment_method` | VARCHAR(40) | `manual`, `stripe`, `edd`, etc |
| `transaction_ref` | VARCHAR(255) | Stripe `pi_…` / `in_…` / manual note |
| `notes` | TEXT NULL | |
| `recorded_by` | BIGINT NULL | WP user id |
| `paid_at` | DATETIME | when payment considered made |

**Idempotency:** one plan per license (`UNIQUE license_id`). Duplicate `create_plan` returns `plan_exists`. Payments are append-only.

---

## 4) Admin Grant + Manage UI

**File:** `includes/class-admin-license-grant.php` (now 33,710 bytes).

- **Menu:** `WP-Admin → Focusa•UIAI Licenses` (`focusa-uiai-licenses`, pos 31, `dashicons-admin-network`). Tabs Grant|Manage|UIAI Nodes|Bundles|Audit.
- **Grant form additions:** checkbox `Enable installments` → fields `Total price`, `Installments (2-36)`, `Type manual|stripe`. Helper auto-fills total from product title `$…`. Description explains Stripe webhook (`…/stripe/payment-plan-webhook`, metadata `license_id`) + EDD Stripe `pk_live_…` presence + reminder cron.
- **Table columns (Manage):** ID, Product (title+download_id+price_id), License Key (masked + Copy/Reveal), Customer (user+email+customer_id), Price (`$697.00`/`Free`/`—` via `edd_price`/`edd_variable_prices`), Date Created + `human_time_diff`, Expires, Remaining (`activation_count/_edd_sl_limit` + bar; Unlimited if 0/null), Status badge, **Payment plan badge** (via join to plans table — TODO polish to show `Paid $x / $y  2/3`), Quick Actions: View, Manage, Activate/Deactivate (`wpuiai_aic_activate_license`), Copy Key, **Record Payment** (modal → `payment-record`), **Send Reminder** (throttled).
- **JS:** grant form toggles `#wpuiai-plan-fields`, AJAX `wpuiai_grant_license`. Quick actions use `wpuiaiGrant.licensesNonce`.

**Current patch status:** form toggle + stripe note landed; stats banner insertion was incomplete (first attempt duplicated marker). Table payment-plan column still TODO for final polish — grant creates plan, verify shows it, but row badge not yet rendered.

---

## 5) Record Payments (how to enter payments against price)

All paths converge on `WPUIAI_AIC_License_Payment_Plan::record_payment(license_id, amount, method, txn, notes, recorded_by)`.

| Surface | How |
|---|---|
| **WP-Admin** | Manage row → Record Payment → amount/method/txn → `POST /wp-json/wpuiai-ai-cloud/v1/admin/payment-record` `{license_id, amount, method, transaction_ref, notes}` |
| **WP-CLI** | `wp --path=public_html wpuiai license pay <license_id> <amount> --method=stripe --txn=pi_… --notes="…" ` |
| **REST (admin)** | `POST /wp-json/wpuiai-ai-cloud/v1/admin/payment-record` (cap `manage_options`, nonced) |
| **Stripe webhook** | auto (see §7) → no manual entry |
| **EDD order hook** | future: `edd_complete_purchase` → find plan by `order_id` → auto-record (stub ready) |

Each call updates `paid_amount / remaining_amount / installments_paid / status` atomically.

---

## 6) Reminders

**Schedule:** `WPUIAI_AIC_License_Payment_Plan::install()` → `wp_schedule_event(time()+3600, 'daily', 'wpuiai_license_payment_reminders')`. Cron: `wpuiai_license_payment_reminders` ~ daily (next run visible via `wp cron event list`).

**Handler:** `cron_send_reminders()` — queries `plans WHERE status IN ('active','overdue','single') AND remaining>0.01` joined to `edd_licenses + edd_customers`. For each:

- Throttle: `get_metadata('edd_license', license_id, 'wpuiai_last_payment_reminder')` — skip if <6d since last (active/single) or <2d if `overdue`.
- Email customer (`wp_mail`, `text/plain`): subject `[Reminder]/[Overdue] <Title> — $remaining remaining`, body shows `Total/Paid/Remaining`, `installments_paid/total`, `status`, license prefix, Stripe pay note if `plan_type=stripe` (`https://wpuiai.com/checkout/?license_id=…`), and "license is active and already registered with the software".
- On send: `update_metadata('edd_license', …, 'wpuiai_last_payment_reminder', time())`, `error_log("[wpuiai-payments] reminder sent …")`.
- Admin digest: `wp_mail(admin_email, "[wpuiai] N reminder(s) sent", …)` if any sent.

**Manual trigger:** `wp --path=public_html cron event run wpuiai_license_payment_reminders` or `wp eval 'WPUIAI_AIC_License_Payment_Plan::cron_send_reminders();'`.

**WP-CLI / REST preview:** `GET /wp-json/wpuiai-ai-cloud/v1/admin/payment-plan/<license_id>` shows `recent_payments`; overdue flagged.

---

## 7) Stripe Awareness (making Stripe aware of it all)

**EDD Stripe live:** `edd_settings['stripe_live_publishable_key']=pk_live_51L0mDb…`, `stripe_live_secret_key=sk_live_51L0mDb…`, `default_gateway=stripe`. No extra config needed for read.

**How Stripe knows:**

1. **At plan creation (`plan_type=stripe`):** `create_plan(…, 'stripe')` marks `plan_type=stripe`. Stripe customer is resolved via `stripe_customer_id_for_license()` — looks up `edd_customermeta.stripe_customer_id` / `_stripe_customer_id` for that license's `customer_id`, or `edd_customer` metadata. If present, future webhook will map. `ensure_stripe_sync(license_id)` verifies `Stripe\Stripe` SDK (loads `edd-stripe/vendor/autoload.php` or `edd/vendor/stripe/stripe-php/init.php`) and `Stripe::setApiKey(secret)` — returns `stripe_customer_id` + `test` flag.
2. **Payment reconciliation:** Two directions:
   - Manual in WP-Admin → record `method=stripe` with `txn=pi_…/in_…`; `verify` now shows `recent_payments` with Stripe txn; seats truth unchanged but finance sees it.
   - Stripe → WP via **webhook** `POST /wp-json/wpuiai-ai-cloud/v1/stripe/payment-plan-webhook` (public, no auth — optional Sig verify via `wpuiai_stripe_webhook_secret` + `Stripe-Signature` header). Handler parses `event.type` + `data.object.metadata[license_id|wpuiai_license_id]` (preferred) or fallback `customer_email` → `edd_customers.email` → latest license. Extracts `amount_received/amount_paid/amount` (cents/100) + `pi/id` → `record_payment(license_id, amount, 'stripe', txn_id, 'Stripe webhook: <type>')`. Returns `202` if no auto-match (advises adding metadata).
3. **What to set when creating Stripe objects:** when creating a PaymentIntent/Invoice in Stripe Dashboard or via API, add `metadata.license_id=<wp_edd_licenses.id>` (or `wpuiai_license_id`). Then webhook auto-links. No license_id → webhook still stores 202 and admin can manually map via CLI `wp wpuiai license pay … --txn=pi_…`.
4. **Options:** `wpuiai_stripe_secret_key` (override), `wpuiai_stripe_webhook_secret` (Stripe `whsec_…` for Sig verify), `wpuiai_stripe_sync_mode` (future). No secret committed.

**Remaining Stripe wiring TODO:** UI "Stripe Pay Link" generation per plan (currently placeholder checkout URL), and EDD `edd_complete_purchase` hook to auto-create plan + auto-record when order completes.

**Check live stripe:** `wp --path=public_html option get edd_settings | tr ',' '\n' | grep stripe` — `sk_live_…` present. `wp eval 'WPUIAI_AIC_License_Payment_Plan::ensure_stripe_sync(33);'` → confirms SDK + customer.

---

## 8) REST / CLI Reference (new)

**REST:**
- `GET /wp-json/wpuiai-ai-cloud/v1/license/verify?license_key=focusa_live_…` — public, includes `payment_plan` + `seats` (see §2). Cache: none (live).
- `POST /wp-json/wpuiai-ai-cloud/v1/admin/payment-plan` — `manage_options`, `{license_id, download_id?, total_price?, installments, plan_type}` → `create_plan`.
- `POST /wp-json/wpuiai-ai-cloud/v1/admin/payment-record` — `manage_options`, `{license_id, amount, method?, transaction_ref?, notes?}` → `record_payment`.
- `GET /wp-json/wpuiai-ai-cloud/v1/admin/payment-plan/<license_id>` — `manage_options`, returns plan+payments.
- `POST /wp-json/wpuiai-ai-cloud/v1/stripe/payment-plan-webhook` — public, Stripe event JSON, auto-records if metadata present.

**WP-CLI (via `as-user wpuiai --path=public_html`):**
- `wp wpuiai license plan-create <license_id> --total=697 --installments=3 [--plan_type=stripe]`
- `wp wpuiai license pay <license_id> <amount> --method=stripe --txn=pi_… --notes=…`
- `wp wpuiai license verify <license_key>`
- `wp cron event run wpuiai_license_payment_reminders`
- `wp eval 'WPUIAI_AIC_License_Payment_Plan::ensure_stripe_sync(123);'`

---

## 9) Files / Diffs (so we don't miss)

- **New:** `includes/class-license-payment-plan.php` (25,976 bytes, `1.0.0`, `WPUIAI_AIC_License_Payment_Plan`) — tables + verify + plan/payment REST + Stripe webhook + reminders + CLI `wpuiai license plan-create|pay|verify`.
- **Modified:** `wpuiai-ai-cloud-admin.php:552` — `require_once …/class-license-payment-plan.php`.
- **Modified (untracked):** `includes/class-admin-license-grant.php` (33,710 bytes) — grant form now has `#wpuiai-plan-toggle` → `#wpuiai-plan-fields` (total/installments/type) + inline description; reminder/stripe banner partially patched (needs polish).
- **DB live:** `wp_wpuiai_license_payment_plans`, `wp_wpuiai_license_payments` created (MyISAM, see §3). Existing `wp_wpuiai_license_machines`, `wp_wpuiai_license_meta` untouched.
- **Cron live:** `wpuiai_license_payment_reminders` daily ~17:03 (see `wp cron event list`).

**Git (plugin):** `M wpuiai-ai-cloud-admin.php` + `?? includes/class-license-payment-plan.php` + `?? includes/class-admin-license-grant.php` (grant file appears untracked due to prior gitignore; will need `git add -f`).

**Not yet specced/committed:** prior `wpuiai-plugin/` scaffold (`admin-grant-endpoint.php`) is the release spec source — new plan tables not yet mirrored there; will need to sync before tag.

---

## 10) Confusion Prevention — what each piece is NOT

- **License connection ≠ payment plan** — a license is valid and seats are enforced even if `remaining>0`; `payment_plan` is financial, not gating validity (unless you later gate).
- **Manual plan ≠ Stripe plan** — `manual` means admin records payments; `stripe` means webhook auto-records and customer gets Stripe pay link. Both use same tables.
- **Reminder ≠ Stripe email** — WP `cron_send_reminders` sends from `wpuiai.com` (cheap, throttled), not from Stripe. Stripe's own invoice emails are separate (EDD Stripe handles those).
- **Verify ≠ admin** — `license/verify` is public read; payment-record/plan-create are `manage_options` only.

---

## 11) Operator notes (Sir V3 rules)

- `dev_mode` stays fixture: `wpuiai.com` has `uiai_dev_mode=0` + real row check. New `verify` does not bypass it.
- Do not use admin account `verious.smith@gmail.com` for storefront ops — use `wpuiai` cPanel user + `test-license-creator` (shop_manager) for grants.
- Keep `HOSTNAME` bypasses out of WP — WP is prod truth, not `FOCUSA_HOME_SERVER` bypass.
- Next polish before close: finish grant table payment-plan column, ensure `plan_enabled` POST path in `ajax_grant` actually calls `create_plan`, add `edd_complete_purchase` auto-record test, set `wpuiai_stripe_webhook_secret` in Stripe dashboard → `wp option update wpuiai_stripe_webhook_secret whsec_…`, run real license → plan → pay → verify → email end-to-end, then `php -l` all, `git add -f`, tag.

