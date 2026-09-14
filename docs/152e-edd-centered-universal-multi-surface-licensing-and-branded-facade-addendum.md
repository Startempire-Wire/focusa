# Spec 152E — EDD-Centered Universal Multi-Surface Licensing and Branded Facade Addendum
**Status:** Required — release-blocking addendum to Specs 150A and 152/152A–D
**Created:** 2026-08-05
**Scope:** WPUIAI.com EDD authority, Focusa and UIAI products, branded websites, install gateways, official and source-built clients, CLI/agent activation, customer identity, payment, Evaluation, license delivery, device activation, signed leases, refunds, migration, and legacy retirement
**Authority boundary:** EDD on WPUIAI.com is the sole customer, commerce, human-license, and entitlement authority. Non-WPUIAI domains are presenters and proxies only.
## 1. Purpose
Focusa licensing is presently divided across EDD on WPUIAI.com, a custom registry on `install.focusa.dev`, direct Stripe webhooks, installer-local state, separate Focusa and UIAI installer paths, website presenters, and runtime clients.
This addendum consolidates those surfaces into one customer experience and one authority chain:

```text
website | installer | CLI | agent | desktop | source build
                              ↓
                    branded domain facade
                              ↓
                   WPUIAI.com authority kernel
                              ↓
       verified identity → EDD customer → EDD order/license
                              ↓
              device registration → signed lease
```

No presenter, installer, local runtime, facade domain, or Stripe metadata may independently create entitlement truth.
## 2. Mandatory decisions
1. WPUIAI.com EDD is the canonical authority for customer identity, checkout, orders, refunds, and human license keys.
2. WPUIAI.com hosts the authority account, device, sequence, and lease-signing state derived from EDD truth.
3. `install.focusa.dev`, `focusa.dev`, `forge.focusa.dev`, `arena.focusa.dev`, approved UIAI domains, desktop clients, installers, and terminal clients are facades or presenters.
4. A submitted email creates only a pending registration attempt.
5. No EDD customer, canonical authority account, checkout, Evaluation, license, node, or lease may be created until mailbox control is verified.
6. Every website and client uses the same registration state machine and API contract.
7. Paid checkout uses EDD's configured Stripe gateway. Clients and facades never collect card PAN, expiry, or CVC.
8. EDD Software Licensing produces the sole human-facing license key.
9. The same authority-issued key is delivered through transactional email and, when requested, a one-time terminal delivery envelope.
10. Runtime authorization requires a signed, device-bound lease. A human key alone is not ongoing execution authority.
11. Evaluation is an EDD-backed, authority-issued, expiring entitlement. Local `--eval` issuance is forbidden.
12. Source builds, raw binaries, package installs, official installers, and agent-driven installs follow the same authority flow.
13. Existing install-site and synthetic records are migration inputs, never co-equal authority.
14. Spec 158 remains excluded; this addendum does not grant or select cognitive or Workstream authority.
## 3. Verified current-state inventory
The implementation must begin from these deployed facts, not an inferred clean slate.
### 3.1 WPUIAI.com
WPUIAI.com currently runs:

- Easy Digital Downloads; EDD Software Licensing; EDD Recurring Payments; EDD Stripe checkout/payment support; WPUIAI AI Cloud Admin; a Focusa production-license integration.
Canonical EDD records include:

```text
wp_edd_customers
wp_edd_customer_email_addresses
wp_edd_orders
wp_edd_order_items
wp_edd_order_transactions
wp_edd_licenses
wp_edd_license_activations
wp_edd_subscriptions
```

Existing routes include:

```text
/wpuiai-ai-cloud/v1/license/validate
/wpuiai-ai-cloud/v1/license/limits
/wpuiai-ai-cloud/v1/focusa/license/issue
/wpuiai-ai-cloud/v1/focusa/license/revoke-machine
```

The Focusa production integration hooks `edd_complete_purchase`, reads the EDD customer email and cart, and can create `focusa_live_...` rows in `wp_edd_licenses`.
### 3.2 EDD mapping gaps

- The current purchase hook lacks an enforceable allowlist of Focusa product/download IDs.; Its comment says credit packs are skipped, but the implementation does not prove that exclusion.; Current `focusa_live_...` rows associated with EDD Download 453 use synthetic payment IDs and have no matching EDD orders.; Download 453 is titled `WPUIAI Pro Lifetime`; it is not a durable, explicit Focusa product boundary.; Standard EDD Software Licensing keys and custom Focusa keys can diverge or duplicate.; Terminal issuance and verified-email promotion are absent.
### 3.3 install.focusa.dev
The site currently contains a separate custom license registry with direct API/Stripe-created records and routes for validation, creation, activation, status, features, and Stripe webhooks.
It uses a different WordPress database from WPUIAI.com. No shared database, trigger, view, cron bridge, or durable EDD synchronization currently reconciles the two systems.
The site receives direct Stripe webhook traffic and has independently issued Focusa/UIAI records. These records are not canonical after this addendum takes effect.
### 3.4 Deployed installer paths
The actual public files are:

```text
/installers/install-focusa.sh
/installers/install-focusa.ps1
/installers/install-engine.sh
/installers/install-bundle.sh
```

The advertised `/focusa` and `/bundle` convenience URLs currently return 404 and must be repaired.
The deployed Focusa shell validates existing keys against:

```text
https://wpuiai.com/wp-json/wpuiai-ai-cloud/v1/license/validate
```

That establishes a one-way EDD validation connection, but it does not create/merge a customer, verify email, start checkout, poll an order, deliver a new key, bind a node, or issue a signed lease.
The deployed Unix and PowerShell installers retain local Evaluation paths. The Focusa shell also accepts `--email` primarily for local receipt state, not canonical identity promotion. The Engine and Bundle paths use different licensing routes. These behaviors must converge.
## 4. Target authority topology

```text
┌──────────────────────────────────────────────────────────────┐
│ Presenters                                                   │
│ Focusa web · Forge · Arena · install site · Menubar · TUI   │
│ CLI · agent JSON · source build · official installer         │
└──────────────────────────────┬───────────────────────────────┘
                               │ one public contract
┌──────────────────────────────▼───────────────────────────────┐
│ Registered branded facades                                   │
│ branding · origin binding · safe redirects · bounded proxy   │
│ no customer/payment/license/lease authority                   │
└──────────────────────────────┬───────────────────────────────┘
                               │ authenticated upstream request
┌──────────────────────────────▼───────────────────────────────┐
│ WPUIAI.com authority kernel                                  │
│ identity · EDD customer · products · orders · SL keys        │
│ Evaluation · nodes · sequences · signer · audit/outbox       │
└──────────────────────────────┬───────────────────────────────┘
                               │ signed lease
┌──────────────────────────────▼───────────────────────────────┐
│ Focusa/UIAI runtime                                           │
│ verify · store · refresh · enforce · recovery-only           │
└──────────────────────────────────────────────────────────────┘
```

Facades may cache public product descriptions and bounded registration state, but cache contents are never entitlement authority.
## 5. Universal registration state machine

```text
attempt_created
→ email_challenge_sent
→ email_verified
→ account_promoted
→ offer_selected
→ checkout_pending | evaluation_review | existing_key_review
→ entitlement_issued
→ terminal_delivery_ready
→ device_registered
→ lease_issued
→ delivered
```

Terminal states:

```text
expired | denied | refunded | revoked | superseded | recovery_only
```

Every transition is idempotent, append-audited, account-scoped, facade-scoped, and correlated by opaque request ID.
## 6. Verified identity and customer promotion
### 6.1 Pending registration
Email submission creates a bounded pending record containing:

- registration UUID; encrypted normalized email; keyed email lookup digest; verification challenge hash; challenge issue/expiry timestamps; attempt and rate-limit state; facade ID and requested product code; installation channel and presenter; device/node public key when available; request and idempotency IDs; consent candidates, not accepted consent; safe redirect handles.
The pending table is not an EDD customer table. Expired and failed attempts remain bounded audit records and cannot receive entitlement.
### 6.2 Verification
Mailbox control is proven through a single-use magic link or short-lived one-time code. Responses are enumeration-resistant and do not disclose whether an email already exists.
Verification requirements:

- challenge TTL; hashed single-use verifier; bounded attempts and resend rate; facade/origin binding; exact registration binding; replay prevention; bounce and delivery-failure handling; no verification through Stripe email alone.
A syntactically valid or Stripe-supplied email is not verified identity.
### 6.3 Atomic promotion
Only after verification, one authority transaction:
1. resolves an existing verified email identity;
2. resolves or creates the authority account;
3. resolves or creates the EDD customer;
4. links an optional WordPress user without creating duplicates;
5. links prior evidence-backed EDD orders and licenses;
6. records transactional consent separately from promotional consent;
7. marks the verified identity primary or linked;
8. advances the registration to `account_promoted`.
Email aliases are never merged through provider-specific dot/plus rewriting. Conflicting paid records require verified purchase/account evidence, not string similarity.
### 6.4 Checkout email integrity
Focusa/UIAI EDD products require a verified registration token before checkout. If checkout supplies a different email, fulfillment pauses until the new email is verified and safely linked. Payment success alone cannot bypass this rule.
## 7. Canonical account and data model
EDD customers remain canonical commerce identities. Additional authority tables live in the WPUIAI.com database.
### 7.1 `wp_wpuiai_authority_accounts`
Required fields:

- opaque account UUID; EDD customer ID; optional WordPress user ID; Stripe customer ID when present; status and status reason; created/updated timestamps; highest entitlement sequence; migration provenance.
### 7.2 `wp_wpuiai_email_identities`
Required fields:

- identity UUID and account UUID; normalized email and keyed lookup digest; verified timestamp and method; primary/linked state; transactional and promotional consent timestamps; bounce, suppression, and revocation state; source and migration evidence.
### 7.3 `wp_wpuiai_activation_registrations`
Required fields:

- registration UUID; account/customer references when promoted; facade, presenter, channel, and product code; state and reason; verification and offer state; EDD cart/order/item/license references; node ID/public key; poll credential hash; terminal-delivery status; request/idempotency IDs; created, expires, and settled timestamps.
### 7.4 `wp_wpuiai_authority_nodes`
Required fields:

- node UUID; account, license, and product references; device public key and assurance class; active/deactivated/revoked status; activation and last-seen timestamps; node-limit reservation and settlement references.
### 7.5 `wp_wpuiai_authority_leases`
Required fields:

- lease UUID; account/license/node/product references; monotonic sequence; authority key ID; envelope digest; issue, not-before, refresh, offline, and expiry times; active/superseded/refunded/revoked status; issuance and settlement event references.
### 7.6 `wp_wpuiai_authority_outbox`
A durable transactional outbox records EDD order, license, refund, revoke, expiry, customer, node, and lease transitions. Dispatch failure cannot lose canonical state. Replay is idempotent.
## 8. Product and grant registry
A server-owned registry maps public product codes to exact EDD products and grants.
Required product families:

```text
focusa_operator
uiai_engine_operator
focusa_uiai_bundle
focusa_evaluation
```

Each mapping declares:

- EDD download and optional price ID; human product name; Stripe/EDD price authority; license duration; activation/node limit; product grants; feature grants; commercial rights; Evaluation eligibility and duration; supported facades; refund and upgrade behavior; lifecycle email templates.
Clients and facades submit only a public product code. They cannot submit EDD IDs, prices, tiers, features, commercial flags, allowed products, or limits.
The current implicit use of Download 453 is not accepted as the production mapping without an explicit registry decision and migration.
## 9. Branded facade registry
Every customer-facing origin is registered with:

- stable facade ID; exact origins and callback origins; supported products; brand name/assets; transactional sender identity; verification, checkout, success, cancel, manage, and recovery paths; locale policy; presenter capabilities; rate and abuse policy reference.
Initial candidates include Focusa install/marketing, Forge, Arena, and approved UIAI domains. Registration is explicit; wildcard authority is forbidden.
Facade requests require timestamp, request ID, idempotency key, exact origin, and authenticated server-to-server credentials. Redirect targets use allowlisted handles, never caller-supplied arbitrary URLs.
Browser sessions remain facade-scoped. Cross-domain cookies are not authority. Opaque continuation tokens are short-lived and bound to registration, facade, action, and nonce.
## 10. Public activation API
Every presenter consumes the same semantic operations. Facades expose branded paths and proxy them to the authority kernel.

```text
POST /v1/activation/start
POST /v1/activation/verify
GET  /v1/activation/offers
POST /v1/activation/select-offer
POST /v1/activation/checkout
POST /v1/activation/existing-license
POST /v1/activation/poll
POST /v1/lease/refresh
GET  /v1/nodes
POST /v1/nodes/deactivate
GET  /v1/account/manage-link
```

All mutations require request and idempotency IDs. Poll credentials are registration-specific, secret, expiring, and stored only as hashes server-side.
No endpoint returns unmasked customer email unless an authenticated customer workflow explicitly requires it. Generic responses use masked email and opaque IDs.
## 11. Paid customer journey

```text
focusa activate
→ choose Purchase
→ enter email
→ verify magic link/code
→ authority promotes EDD customer
→ select server-owned product
→ receive branded EDD checkout URL
→ pay through EDD Stripe gateway
→ EDD order completes
→ EDD Software Licensing issues key
→ EDD emails key/receipt
→ terminal poll receives encrypted key envelope
→ device registers
→ signed lease issues
→ runtime continues onboarding
```

The browser-facing hostname remains the approved product facade except where Stripe's hosted payment origin is intentionally visible. EDD remains the backend transaction authority.
An EDD completion hook must require:

- complete/eligible order status; verified registration/account binding; allowlisted product mapping; exact order item and price relationship; idempotent issuance state; no existing equivalent active license unless upgrade/reissue policy permits it.
## 12. Evaluation journey

```text
choose Evaluation
→ enter and verify email
→ promote/resolve EDD customer
→ evaluate account/order/license/device history
→ issue dedicated expiring EDD-backed Evaluation license
→ email and terminal delivery
→ register node
→ issue signed Evaluation lease
```

Evaluation eligibility is authority-private. It may consider verified account, prior Evaluation, payment history, license history, device registrations, refund/revoke state, and bounded anti-abuse signals.
Forbidden:

- local `--eval` records; installer-created grace licenses; unsigned Evaluation state; Evaluation from unverified email; Evaluation downgrade for an existing paid customer; duplicate Evaluation through facade switching.
## 13. Existing-license journey

```text
enter existing EDD Software Licensing key
→ enter/confirm account email
→ verify mailbox control
→ resolve EDD license/customer ownership
→ enforce status/product/node limit
→ register device
→ issue signed lease
```

A key and unrelated verified email cannot activate a node. Legacy customers without verified identity must verify before new-node activation, reissue, or account merge.
## 14. Terminal, agent, and source-build experience
### 14.1 Interactive terminal

```text
Focusa requires activation.
1. Enter existing license
2. Purchase Focusa
3. Request Evaluation
Email: customer@example.com
Verification code sent.
Code: 483921
Email verified.
Complete payment:
https://install.focusa.dev/pay/<opaque-token>
Waiting for payment...
Payment confirmed.
License: FOCUSA-XXXX-XXXX-XXXX-XXXX
A copy was emailed to c***@example.com.
Device activated.
```

### 14.2 Agent/JSON presenter
Machine-readable states include:

```text
email_required
email_verification_pending
email_verified
selection_required
checkout_required
payment_pending
license_delivery_ready
activated
denied
recovery_only
```

Agent mode must never invent an email, verification code, consent, payment confirmation, or license. It presents the URL/code, waits for the human, polls within budget, and resumes only after authority completion.
Full key output is masked by default in structured logs. An explicit customer-controlled reveal mode may return the one-time decrypted key. Credential stores receive the key and refresh credential directly.
### 14.3 Source and unofficial builds
A source-built or manually copied client follows the same activation protocol. `install_channel=source_build` is advisory telemetry, never entitlement evidence.
Public local code cannot make patching impossible. Protected capabilities remain governed by Spec 152A through server-side service or signed private capsule boundaries.
## 15. Human key and signed lease separation
The EDD Software Licensing key is the human entitlement and recovery credential. It supports account management, reactivation, and additional-node requests.
The signed lease is the machine execution credential. It contains at minimum:

- schema and product identifiers; account, EDD customer, order/item, and license references; explicit products, features, and limits; commercial rights; node ID/public-key binding; monotonic sequence; issue/not-before/offline/expiry times; authority key ID and signature; Evaluation/paid/bundle posture; refund/revoke/supersession semantics.
License keys never authorize runtime mutation without successful lease verification.
## 16. Dual-channel license delivery
EDD produces one canonical human key.
### 16.1 Email
EDD transactional email sends:

- product and order identity; full human license key; safe installation/activation instructions; account-management/recovery link; support information; no promotional content without separate consent.
Delivery attempts, bounces, and suppression are recorded without placing raw keys in generic logs.
### 16.2 Terminal
The authority encrypts a one-time delivery envelope to the registration's device public key. Poll returns that envelope after issuance. The client decrypts, displays the key once when permitted, and stores it through the protected credential adapter.
Facades, access logs, analytics, and generic agent transcripts must not receive the plaintext key.
A terminal-delivery failure does not create a second key; authenticated recovery retrieves or reissues according to EDD policy.
## 17. EDD lifecycle integration
Authority hooks cover:

- customer/email promotion; checkout creation; order completion; order failure/cancellation; Software Licensing issuance/reissue; activation/deactivation; refund/chargeback; manual revoke/suspend; subscription update/cancellation; expiry; product upgrade/downgrade; account email change.
Each hook appends an outbox event in the same transaction as canonical state. Lease sequence changes on entitlement-relevant transitions.
Periodic reconciliation compares EDD customers, orders, licenses, activations, refunds, authority accounts, nodes, and leases. Missing callbacks cannot leave stale access permanently active.
## 18. Refund, revoke, and recovery

```text
EDD refund/revoke/expiry
→ EDD license status changes
→ authority sequence increments
→ current lease is superseded/revoked
→ refresh denies protected access
→ client enters recovery_only
```

Recovery-only preserves:

- account verification; license status and management; export; diagnostics; repair; update needed for recovery; uninstall.
Refund does not delete account, order, device, evidence, or audit history.
## 19. Security and privacy requirements
1. No card data enters Focusa, an agent, a facade, or the authority API; EDD/Stripe handles it.
2. No raw email in generic logs, telemetry, URLs, shell history, or evidence artifacts.
3. No full license key in facade/access logs or default agent JSON.
4. Verification and poll secrets are hashed at rest and single-use where applicable.
5. Pending email plaintext is encrypted at rest; keyed digest supports exact lookup.
6. Authority signing keys and EDD/Stripe credentials remain server-side and outside web-readable storage.
7. Product grants come only from the server-owned registry.
8. Facade safety annotations and client claims are untrusted.
9. Checkout, issuance, delivery, activation, and refresh are idempotent.
10. Node registration uses a device public key; install channel or binary signature does not prove customer identity.
11. Direct EDD cart access for protected products requires verified registration context.
12. Existing paid records cannot be downgraded to Evaluation.
13. Email changes require re-verification and cannot silently transfer purchases.
14. Admin/manual issuance requires explicit authority, reason, evidence, and audit identity.
## 20. Stable failure semantics
Required public-safe failures include:

```text
EMAIL_REQUIRED
EMAIL_VERIFICATION_REQUIRED
EMAIL_VERIFICATION_EXPIRED
EMAIL_VERIFICATION_FAILED
EMAIL_DELIVERY_FAILED
ACCOUNT_EMAIL_MISMATCH
ACCOUNT_MERGE_REVIEW_REQUIRED
FACADE_ORIGIN_DENIED
FACADE_PRODUCT_DENIED
PRODUCT_MAPPING_REQUIRED
EDD_CUSTOMER_RESOLUTION_FAILED
EDD_CHECKOUT_REQUIRED
EDD_ORDER_PENDING
EDD_ORDER_UNVERIFIED
EDD_LICENSE_PENDING
EDD_LICENSE_UNUSABLE
EVALUATION_NOT_ELIGIBLE
LICENSE_DELIVERY_PENDING
LICENSE_DELIVERY_FAILED
LICENSE_ACCOUNT_MISMATCH
NODE_LIMIT_EXHAUSTED
AUTHORITY_UNAVAILABLE
ENTITLEMENT_REQUIRED
ENTITLEMENT_FEATURE_REQUIRED
ENTITLEMENT_LIMIT_EXHAUSTED
```

Failures return safe next actions and never raw authority, email, payment, or credential data.
## 21. Surface consolidation requirements
The following presenters must use one shared activation client/state machine:

- Unix installer; PowerShell installer; Rust installer/orchestrator; CLI license commands; daemon REST/API; TUI; Pi/agent integration; menubar/desktop first run; Focusa website registration; Forge/Arena registration; UIAI and bundle activation; local source builds.
Presenter-specific code may render prompts and links. It may not reimplement identity, product, payment, Evaluation, license, node, or lease decisions.
## 22. Migration and legacy retirement
### 22.1 Inventory
Reconcile:

- EDD customers, orders, order items, licenses, and activations; current synthetic/custom Focusa keys; install-site API/Stripe licenses; Stripe customers, payments, refunds, and product metadata; existing local installer license receipts; existing nodes and limits; UIAI/bundle records.
### 22.2 Merge rules

- Verified email plus evidence-backed purchase/license linkage may attach a record to an account.; Stripe customer/payment evidence may reconcile install-site paid records to EDD.; Raw matching email alone does not transfer ownership.; Synthetic records remain quarantined unless separately approved.; Key, order, product, refund, and sequence history is preserved.; Migration cannot reactivate refunded/revoked records or downgrade paid users.
### 22.3 Cutover
1. Create explicit EDD product mappings.
2. Add identity, registration, node, lease, and outbox schemas.
3. Deploy verification and EDD checkout flow.
4. Deploy signed lease issuance and refresh.
5. Add registered facade proxies and branded pages.
6. Migrate evidence-backed records.
7. Switch websites and clients to the universal contract.
8. Disable direct install-site issuance and local Evaluation.
9. Retain bounded legacy validation/recovery.
10. Make install-site legacy tables read-only, then retire them after reconciliation.
### 22.4 Rollback
Rollback may restore the prior software version and facade route, but it must not roll back EDD order/refund truth, verified identity, monotonic sequence, revocation, or audit history. New issuance fails closed during rollback; existing signed offline policy remains bounded.
## 23. Acceptance matrix
Release acceptance requires end-to-end proof for:

| Case | Required result |
|---|---|
| Website paid Focusa | verified email → EDD order/key → email + account delivery |
| Terminal paid Focusa | verified email → facade checkout → terminal key + signed lease |
| Agent paid Focusa | structured challenge → human verification/payment → resume activated |
| Source build | same authority flow; no installer dependency |
| Evaluation | verified account + eligibility → expiring EDD-backed key/lease |
| Existing key | verified owner email + node allocation → lease |
| UIAI purchase | explicit UIAI product grants only |
| Bundle purchase | explicit Focusa + UIAI grants in one account flow |
| Wrong product | no cross-product lease or downgrade |
| Invalid/unreachable email | pending attempt only; no customer/order/license/lease |
| Changed checkout email | fulfillment held until verification |
| Duplicate request | one account/order/license/node result |
| Prior Evaluation | authority denial or approved policy result; no local bypass |
| Paid customer requests Eval | paid posture preserved |
| Node limit | deny and offer explicit node management |
| Refund | EDD refunded → sequence increment → refresh denied/recovery-only |
| Revocation | immediate authority denial with durable audit |
| Authority outage | no new local license; existing signed offline policy only |
| Facade spoof | origin/product/redirect denied |
| Terminal delivery loss | no duplicate license; authenticated recovery path |
| Broken convenience URL | repaired facade route returns verified installer asset |
| Legacy install-site record | evidence-backed migration or quarantine |

## 24. Completion gate
This addendum is complete only when:
1. EDD customer/order/license truth is canonical and singular;
2. every registration surface requires verified mailbox control;
3. every successful registration creates or links the durable authority account and EDD customer;
4. paid and Evaluation keys are EDD-backed;
5. email and terminal delivery return the same canonical key;
6. all clients receive a device-bound signed lease;
7. non-WPUIAI domains remain branded facades without issuance authority;
8. refunds/revokes propagate into lease refresh and recovery-only posture;
9. source builds and agents use the same flow;
10. installer-local Evaluation and split registries cannot issue new entitlement;
11. migration and reconciliation prove no paid-user loss or authority rollback;
12. the acceptance matrix passes with redacted, replayable evidence;
13. the accepted Spec 152F closure (focusa-vbcqu.20.14.52, documented in docs/evidence/spec152f/focusa-vbcqu.20.14.52-acceptance.txt) and the REL.4–REL.7 governance acceptance have closed first — Spec152F cannot be bypassed by REL closure, the dependency graph has zero cycles, and stable publication still requires this exact final release acceptance.
Until then, customer/evaluator distribution and stable-release claims remain blocked.
