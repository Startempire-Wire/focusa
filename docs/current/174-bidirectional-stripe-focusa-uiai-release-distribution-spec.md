# 174 — Bidirectional Stripe ↔ WordPress ↔ Focusa ↔ UIAI Engine + Release Distribution

**Status:** spec — documents live system + required delta to prove tmstripe well triggers real before monetized release
**Date:** 2026-08-18
**Authors:** WPUIAI / Focusa
**Applies to:** `wpuiai.com` (`wpuiai-ai-cloud-admin` + `wpuiai`), `focusa` Rust (license/core/cli/api), Stripe (test `sk_test_51Svk…` + live `sk_live_51L0m…`), distribution via `install.focusa.dev` + WP EDD checkout
**Depends:** Spec 173 grant/verify, Spec 152E registry, `class-license-payment-plan.php` v1.0.0, `class-focusa-license-production.php`, `focusa-core/license.rs` `focusa_license/lib.rs` `focusa-api/routes/license.rs`, `wpuiai/includes/class-license.php` + `class-license-cache.php`

---

## 1. Goal

Make money and distribute:

* A buyer on `wpuiai.com` checkout — or Sir V3's one-click grant — is instantly valid inside **both** runtimes (Focusa daemon + UIAI Engine plugin) and inside **Stripe** (payment truth), bidirectionally.
* Stripe can push payment state to WP, WP can push plan state to Stripe, Focusa and UIAI each pull and also report usage/seats back — no shadow ledger.
* Prove it with **tmstripe (Stripe test mode)** — a real test PaymentIntent triggers the webhook and moves the license from `remaining 397 -> 0` with receipt, before any live sale.
* Then ship the two artifacts that actually sell: `focusa` build (CLI + daemon + menubar) and `wpuiai` / `wpuiai-ai-cloud-admin` plugin build, via the same install channel.

---

## 2. System Map (four peers, one truth)

```
Buyer ──> EDD Checkout (Stripe Elements) ─┐
                                          ├─> wp_edd_licenses (focusa_live_1736_… active)
Sir V3 ─> WP-Admin Focusa•UIAI Licenses ──┘        │
       (do_grant: resolve_edd_customer -> insert license, create_plan)
                                                   │
                          ┌────────────────────────┼────────────────────────┐
                          │                        │                        │
              wp_wpuiai_license_payment_plans   wp_wpuiai_license_machines  wp_uiai_client_keys
              (total/paid/remaining,            (seat cap 3, per-machine   (client_id/secret,
               status single|active|overdue|     mailbox_verified)          tier=Operator)
               completed, plan_type manual|     │
               stripe, installments)            └─> license_truth() ──┐
                          │                                             │
                          │  Stripe                                     │  Verify
             ┌────────────┴──────────┐                       ┌───────────┴──────────┐
             │ Stripe Customer/Price │                       │ GET /wp-json/wpuiai-ai-cloud/v1/
             │ Subscription /        │<─── WP->Stripe ───────│ license/verify?key=…│
             │ PaymentIntent         │   (create_plan         │ license/validate    │
             │ metadata.license_id ──┼──> Stripe->WP ───────>│ license/status      │
             └───────────┬───────────┘   webhook              └───────┬──────────────┘
                         │        POST /stripe/payment-plan-webhook       │
                         │              record_payment (idempotent)       │
                         │                                                │
              ┌──────────┴─────────┐              ┌──────────────────────┴──────────────────┐
              │ Focusa Rust        │              │ UIAI Engine (WordPress plugin `wpuiai`)│
              │ crates/focusa-core │              │ WPUIAI_License + WPUIAI_License_Cache │
              │  license.rs        │──validate──> │  API_URL = /license/validate           │
              │  license_developer │  POST key     │  1h cache -> 5m transient             │
              │  focusa_license    │  X-License-Key│  tier + limits -> features            │
              │  focusa-cli license│              │  wpuiai-ai-cloud-admin DB usage tables│
              │  focusa-api /v1/   │<─ report ───>│  POST usage -> wp_uiai_*_usage        │
              │  license/status    │  seats/lease │  seats also via license_truth()       │
              └────────────────────┘              └───────────────────────────────────────┘
```

**DB truth tables** (`$wpdb->prefix` = `wp_`):

* `wp_edd_licenses` (EDD 3.x): `id, license_key focusa_live_…, status active|expired|revoked, download_id 1736|1735, customer_id, user_id, expiration NULL, date_created`
* `wp_wpuiai_license_payment_plans`: `id, license_id UNIQUE, download_id, total_price 697.00, installments_total/paid, paid_amount/remaining_amount, status single|active|overdue|completed, plan_type manual|stripe, next_due_date`
* `wp_wpuiai_license_payments`: `id, plan_id, license_id, installment_number, amount, payment_method manual|stripe, transaction_ref pi_…/sub_…, paid_at`
* `wp_wpuiai_license_machines`: `license_id, machine_id UX, registered_at, last_seen_at, revoked_at` (seat enforcement, cap via `wpuiai_aic_seat_cap_per_license` = 3)
* `wp_uiai_client_keys`: `client_id, client_secret, license_id, tier, status`
* `wp_wpuiai_license_meta` + `wp_edd_customermeta` (`stripe_customer_id`)

---

## 3. Stripe Bidirectional — Contract

### 3.1 Stripe config (live + test = tmstripe)

`edd_settings` (WP option):

* `stripe_live_publishable_key pk_live_51L0m…`, `stripe_live_secret_key sk_live_51L0m…`, `stripe_test_secret_key sk_test_51Svk…`, `stripe_test_publishable_key pk_test_51Svk…`, `test_mode 0|1`, `stripe_test_mode_enabled`, `default_gateway stripe`
* Override: constant `STRIPE_SECRET_KEY` or option `wpuiai_stripe_secret_key` (preferred for tmstripe toggle), webhook secret `wpuiai_stripe_webhook_secret whsec_…`

**tmstripe** = run with `edd_settings.test_mode = 1` so `stripe_settings()` picks `sk_test_51Svk…`. Live flips to `0` -> `sk_live_`. Same code, same webhook URL, two keys.

### 3.2 WP -> Stripe (outbound — the missing half, now spec'd)

Trigger: `POST /wp-json/wpuiai-ai-cloud/v1/admin/payment-plan` with `plan_type=stripe` **or** `WPUIAI_AIC_License_Payment_Plan::create_plan(…, 'stripe')`.

Steps (idempotent, `UX license_id`):

1. `ensure_stripe_sync(license_id)` — load SDK (`easy-digital-downloads/libraries/Stripe/init.php` -> `EDD\Vendor\Stripe\Stripe` or `\Stripe\Stripe`), `Stripe::setApiKey(secret)`, fetch `stripe_customer_id` from `edd_customermeta`.
2. If no Stripe customer, `Stripe\Customer::create([email=>customer_email, metadata=>[wpuiai_customer_id, license_id]])` and persist `stripe_customer_id` via `edd_add_customer_meta`.
3. Ensure Stripe Product/Price for `download_id`: lookup `wpuiai_stripe_price_1736` option; if missing, `Stripe\Product::create([name=>get_the_title(1736), metadata=>[download_id]])`, then `Stripe\Price::create([product, unit_amount=>69700, currency=>usd, metadata=>[download_id, price_version]])`, persist `price_id`.
4. Create priced intent for the plan: for installments>1, prefer `Stripe\Subscription::create([customer, items=>[price], metadata=>[license_id, download_id, wpuiai_license_id=>license_id], collection_method=>send_invoice, days_until_due=>7])` OR `Stripe\PaymentLink::create([line_items=>[price, quantity=>1], metadata=>[license_id], after_completion=>…])`. For single, `Stripe\PaymentIntent::create([amount=>total*100, currency=>usd, customer, metadata=>[license_id], description=>product_title])`. Store returned `stripe_subscription_id` / `stripe_payment_intent_id` / `checkout_url` in `wpuiai_license_meta` (`stripe_subscription_id`, `stripe_checkout_url`).
5. Return `{ ok:true, plan_id, stripe:{ customer_id, price_id, subscription_id|payment_intent_id, checkout_url } }`. Replay-safe via `plan_exists` early return.

Spec requires: every Stripe object created by WP **must** carry `metadata.wpuiai_license_id = "<license_id>"` (and `metadata.license_id` alias for webhook) and `metadata.wpuiai_download_id`.

### 3.3 Stripe -> WP (inbound — already live)

Webhook: `POST /wp-json/wpuiai-ai-cloud/v1/stripe/payment-plan-webhook` (no auth; Stripe-Signature verified when `wpuiai_stripe_webhook_secret` set).

Handles `payment_intent.succeeded`, `invoice.paid`, `charge.succeeded` (payload `data.object`):

* Reads `metadata.license_id || metadata.wpuiai_license_id` => `license_id`, `amount_received|amount_paid|amount` cents/100, `id` => `transaction_ref`, `type` for audit notes.
* If no id, fallback `customer_email | receipt_email` -> `edd_customers.email -> edd_licenses.id ORDER BY id DESC LIMIT 1`.
* Calls `record_payment(license_id, amount, 'stripe', txn, "Stripe webhook: $type")` — idempotent via `UNIQUE transaction_ref` per license (`replay:true`), `START TRANSACTION ... FOR UPDATE`, updates `paid_amount, remaining_amount, installments_paid, status (completed when remaining<=0.01 else overdue when count>=total)`.
* Webhook then triggers `license/verify` cache invalidation; next `GET /license/verify?license_key=…` returns new `recent_payments[]` and `plan.remaining`.

Proved: license 34 `txn_test_1 200` + `txn_test_2 10` -> verify `paid 210 remaining 487`; license 35 `txn_e2e_full_1 300` -> `remaining 397`.

### 3.4 Payment reminders (both worlds)

WP-Cron `wpuiai_license_payment_reminders` daily. Queries `status IN(single,active,overdue) AND remaining>0.01`. Throttle via `edd_license` meta `wpuiai_last_payment_reminder` (6d single/active, 2d overdue). `wp_mail` body includes Total/Paid/Remaining, `installments x/y`, and if `plan_type=stripe` adds `Pay securely via Stripe: https://wpuiai.com/checkout/?license_id=…` (after outbound fix, replaces with real `stripe_checkout_url`).

### 3.5 tmstripe proof procedure (well trigger real)

1. Set `edd_settings.test_mode = 1` (or `wpuiai_stripe_secret_key = sk_test_51Svk…`, `wpuiai_stripe_webhook_secret = whsec_test_…` from `stripe-cli listen --forward-to https://wpuiai.com/wp-json/wpuiai-ai-cloud/v1/stripe/payment-plan-webhook`).
2. Create license via grant (or reuse 35 active, remaining 397).
3. `POST /admin/payment-plan { license_id:35, plan_type:"stripe", total_price:697, installments:3 }` -> WP creates Stripe Customer + Subscription (`metadata.license_id=35`) -> capture `payment_intent_id`.
4. Trigger real test payment: `stripe payment_intents confirm pi_… --payment-method pm_card_visa` or Stripe Dashboard Test Clock, or `stripe trigger payment_intent.succeeded` with overridden `metadata.license_id`.
5. Assert webhook returns `200 auto_recorded ok:true`, `wp wpuiai license verify <key>` shows `remaining` dropped by payment amount and `recent_payments[]` contains `method:stripe txn:pi_…`.
6. Restore `test_mode=0`, verify live key prefix `pk_live` unchanged.

---

## 4. Focusa Bidirectional — Contract

### 4.1 Focusa -> WP (validate / refresh / watch)

* `crates/focusa-core/src/license.rs::activate(key, registry, persist_key)` does `POST {registry}/wp-json/wpuiai-ai-cloud/v1/license/validate` with `X-License-Key: focusa_live_…` and `{license_key:key}`. Expects `{valid:true, product, tier, status, commercial_use, customer_email, features[], expires_at}`.
* On success writes `~/.config/focusa/license.json` (`chmod 600`, `key_hash=sha256(key)`, `key_prefix=first 16`, `product/tier/status/features/offline_valid_until=+7d/issued_at`), also optional `raw_key` when `--persist-key`.
* `doctor(license_file)` pings `GET {registry}/wp-json/wpuiai-ai-cloud/v1/license/status?license_key=focusa_live_probe` to set `registry_reachable`.
* CLI mirrors: `focusa license activate <key>`, `focusa license status --json`, `focusa license doctor`, `focusa license check-feature packaged_installer`, `focusa license refresh [--raw-key] [--require-real]`, `focusa license watch --interval 60`. Watch long-polls registry and rewrites local file on status change (revoke/refund propagation).

### 4.2 WP -> Focusa (truth + seats)

* `GET /wp-json/wpuiai-ai-cloud/v1/license/verify|validate|status?license_key=focusa_live_…` returns `{ ok:true, schema:focusa.license_verify.v1, license:{id, license_key masked, license_key_full if caller supplied exact key, download_id, product, status, price}, payment_plan:{…}, seats:{schema:focusa.license_truth.v1, license:{license_id, download_id, status}, nodes:[]} }`.
* `seats` comes from `WPUIAI_AIC_Focusa_License_Production::license_truth(license_id)` which joins `edd_licenses` + `Node_Registry`. Per-machine gating via `X-Machine-Id` header -> `wp_wpuiai_license_machines` (`machine_seat_check` / `seat_cap 3`, `mailbox_verification_required` for new node, auto-revoke on `edd_pre_refund`).
* `crates/focusa-license/src/lib.rs` `LicenseGuard` resolves tier via env `FOCUSA_LICENSE_KEY` + `FOCUSA_LICENSE_REGISTRY`, or `~/.config/focusa/license.json`, or `~/.focusa/license.toml`, else `eval 7-day`. `crates/focusa-core/src/license.rs::entitlement_check(feature)` is the single decision point (developer_origin bypass, `capability_for_feature`, `lease_valid_status` checks revocation/expiry/offline grace). Denied features return `LicenseError::FeatureRequiresLicense`.
* Daemon `crates/focusa-api/src/routes/license.rs::/v1/license/status` exposes local `LicenseGuard` posture (`tier, issued_at, expires_at, bsl_change_date, capabilities[] permitted|denied`) for `MainWP`/Pi.
* Focusa reports back usage indirectly: installs/updates go through `orchestrator.rs::validate_request_at` (entitlement-gated), and the daemon's license plane is re-validated on each `license refresh` tick.

### 4.3 Bidirectional liveness

WP revokes or refunds (`POST /admin/revoke-license`, EDD refund hook `edd_pre_refund -> revoke_all_machines`) — next Focusa `refresh`/`watch` sees `status=revoked`, `lease_valid=false`, capabilities flip to `denied`, `license status` shows lease invalid. Focusa activation adds a machine row; WP `deactivate` removes it via `revoke_machine(license_id, machine_id)`.

---

## 5. UIAI Engine Bidirectional — Contract

### 5.1 UIAI -> WP (validate with cache)

* `wpuiai/includes/class-license.php::WPUIAI_License` (`API_URL = https://wpuiai.com/wp-json/wpuiai-ai-cloud/v1/license/validate`, `OPTION_KEY=uiai_license_key`, `CACHE_KEY=uiai_license_cache 3600s`, `WPUIAI_License_Cache` 5m transient). `validate_remote()` POSTs `{license_key, domain}`, normalizes `tier+limits -> features {cloud_screenshots, screenshot_limit, max_width, workflow_runner, priority_support}`.
* `get_license_data()` returns cached or `free` tier on failure (`cloud_screenshots false, limit 0`). Site domain stripped of `www.`.

### 5.2 WP -> UIAI (limits + usage)

* `wpuiai-ai-cloud-admin` holds the same `license/verify` truth; admin table `Focusa•UIAI Licenses` + `wpuiai-ai-cloud-client-keys` maps licenses to UIAI `client_id/secret` tiers (`Operator` for 1736).
* Usage metering: `wp_uiai_screenshot_usage, wp_uiai_critique_usage, wp_uiai_ui_reverse_usage` etc., keyed by `license_id`. The plugin gates `max_width 1920->3840`, `screenshot_limit`, `workflow_runner` per tier. UIAI reports usage back by inserting rows and the cloud operations table `wp_uiai_cloud_operations`; WP can then throttle via tier downgrade.
* License install/update/update-check paths query the same `license/verify` -> UIAI unfurls `seats` / `payment_plan.remaining` without blocking license validity — payment balance and license validity are orthogonal (license stays `active` even with `remaining>0`; only `status=revoked/expired` blocks).

---

## 6. Payment Plan & Licensing State Diagram

```
grant (edd customer resolve verified) -> edd_license active (focusa_live_1736_…)
   │
   ├─> create_plan(total 697, installments 1..36, type manual|stripe)
   │       status=single (1) or active (>1), remaining=total
   │       (stripe branch: also create Stripe Customer+Price+Subscription/PaymentIntent)
   │
   ├─> record_payment(amount, method, txn) --idempotent txn--
   │       paid+=amount, remaining=max(0,total-paid), installments_paid++
   │       if remaining<=0.01 => completed (0 remaining)
   │       else if installments_paid>=installments_total && remaining>0 => overdue
   │       else active / single
   │       (also mirrors to Stripe if plan_type=stripe: no extra Stripe create on inbound)
   │
   ├─> Stripe webhook invoice.paid -> record_payment('stripe', txn=pi_…)
   │       same transition, replay:true on duplicate
   │
   ├─> machine_seat_check(X-Machine-Id) -> wp_wpuiai_license_machines
   │       cap 3, new machine => registered, existing => last_seen refresh, cap exceeded => seat_cap_reached
   │       refund -> revoke_all_machines (set revoked_at)
   │
   └─> verify query (public) -> license + payment_plan + seats same JSON to Focusa+UIAI+frontend
```

---

## 7. Release & Distribution (make money)

### 7.1 What ships

* **Focusa app:** Rust `focusa-cli`, `focusa-api` daemon (systemd `focusa.service`), `focusa-tui`, Menubar `com.focusa.menubar` (Tauri). License-gated features (`packaged_installer, hosted_operations, product_embedding, public_stream`) checked via `entitlement_check`.
* **UIAI Engine:** WordPress plugins `wpuiai` (client, `class-license.php`) + `wpuiai-ai-cloud-admin` (server, includes above payment plan + seats). Both enqueued at `admin_enqueue_scripts` priority 99 via `aic-responsive-global.css` (8K, versioned `filemtime`).
* **Registry/authority:** `wpuiai.com` WP, EDD 3.x, `edd_software_licensing`, Stripe + `wp_wpuiai_*` own tables; endpoints under `wpuiai-ai-cloud/v1`.

### 7.2 Build gates before any tag

```bash
timeout 90  /root/.cargo/bin/rustup run 1.91.0 cargo fmt --all -- --check
timeout 120 /root/.cargo/bin/rustup run 1.91.0 cargo clippy --workspace --all-targets -- -D warnings  # 0
timeout 60  svelte-check --tsconfig ./tsconfig.json  # 0 errors
php -l wpuiai-plugin/**/*.php  # No syntax errors
cargo test -p focusa-license -p focusa-core -- license  # lease_valid etc.
curl https://wpuiai.com/wp-json/wpuiai-ai-cloud/v1/license/verify?license_key=focusa_live_…  # ok:true
```

### 7.3 Canonical tag (only way)

```bash
bash scripts/create-dev-release-tag.sh --push  # stamps version surfaces, pushes tag
# NOT git tag -m "..." manually
```

Producing `v0.9.172` -> `v0.9.173-dev` via journal `docs/current/01-focusa-canonical-release-journal.md`, ledger `focusa-license v0.9.172 rustc 1.91.0`, menubar `apps/menubar/src-tauri/tauri.conf.json` version.

### 7.4 Install/distribution channels

* `https://install.focusa.dev` (or `https://wpuiai.com/checkout/?license_id=`) -> `install.sh` / `cargo install` / `brew`. The installer runs `focusa license activate <key> --registry https://wpuiai.com`.
* `wpuiai.com` EDD product pages 1736/1735/bundle 1254.60, Stripe Elements. After purchase -> `thank-you?license_key=focusa_live_…` -> `fetch(/license/status)` -> Download button gated.
* OTA nightlies for home servers (`FOCUSA_UPDATE_CHANNEL=nightly`, `is_home_dev_bypass()` for `kh|ovh-w1|ovh-w2|localhost` or `FOCUSA_HOME_SERVER=1`) — zero entitlement ceremony on home, production stays gated.
* Revenue: product price `edd_price 697` (+ Stripe `amount 69700` cents), one plan per license (`UNIQUE license_id`), installments paid visible in admin `Payment $rem due $paid/$total • x/y • status • Stripe` bar and API.

### 7.5 tmstripe proof bundle before going live

Artifact: `docs/evidence/finish/174-tmstripe-bidirectional-proof.txt` + webhook log `curl -i POST /stripe/payment-plan-webhook` + `license/verify` before/after JSON diff showing `paid 300->550 remaining 397->147` via Stripe test event, plus screenshots of Stripe Dashboard Test Payments `metadata.license_id=35`.

---

## 8. Acceptance (blocked until proof)

* `php -l` on `class-license-payment-plan.php`, `class-focusa-license-production.php`, `wpuiai/includes/class-license.php` — ok.
* `cargo fmt --check` + `cargo clippy -D warnings` (1.91.0) — 0.
* tmstripe live run: `test_mode=1`, create `license 35` stripe plan, confirm `pi_test_…`, webhook 200 auto_recorded, `GET /license/verify` remaining drops, idempotency replay returns `replay:true`; flip `test_mode=0` restores live key.
* Focusa: `focusa license activate focusa_live_35 … && focusa license status --json` shows `tier operator, lease_valid true, features non-empty`; daemon `GET /v1/license/status` same.
* UIAI: `WPUIAI_License::validate_remote()` returns `valid:true tier=Operator` for same key, `clear_cache()` then re-validates in <2s after grant.
* Refund path: EDD refund -> `revoke_all_machines` -> Focusa `refresh` sees `status revoked` -> `lease_valid false`.

---

## 9. Open wire before 174 closes

* Finish `WP->Stripe` outbound creator in `class-license-payment-plan.php::create_plan` stripe branch (Customer/Price/Subscription with metadata). Currently only Customer trait missing.
* Swap placeholder `https://wpuiai.com/checkout/?license_id=` with real `stripe_checkout_url` in reminder email.
* Replace `wpuiai_stripe_webhook_secret whsec_REPLACE_…` placeholder with real `whsec_…` from Stripe Dashboard -> `wp option update wpuiai_stripe_webhook_secret`.
* Flip `edd_settings.test_mode 0<->1` only for the tmstripe proof; keep live `pk_live` for customers.

