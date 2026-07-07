# Spec 119 — Downloadable Lifetime → Yearly/Major-Version Transition

**Status:** Living transition plan for the local-first downloadable Focusa
(not Focusa Cloud SaaS — see Spec 115 for that separate product line).
**Owner:** Verious Smith (operator decision authority).
**Cuts with:** Spec 118 (license tiers) — this is the *time-bound* companion that
explains when each tier shape is in effect.

---

## 1. Two-phase model

### Phase 1 — Lifetime (current)

```text
Operator Lifetime License ............ $697  one-time, forever
UIAI Engine Operator License ......... $697  one-time, forever
Bundle (Focusa + UIAI Engine) ........ contact sales
Founders Forge cohort ................ $7,500 ($1,500 deposit path)
```

Source: `https://install.focusa.dev/license` (live) and Spec 118 §2.

All buyers receive:

- All current and future Focusa minor versions (X.Y.*).
- All current and future Focusa *major* versions until the transition trigger.
- License key activates `commercial_use = true`, `tier: "operator"` /
  `tier: "founders-forge"`, features per registry handshake.
- 14-day refund window (per `LICENSE-FAQ.md`).

### Phase 2 — Yearly OR Major-Version (triggered later)

When the operator signals the transition, lifetime stops being sold.
The model switches to one of two shapes (operator decides at trigger time).

#### 2A. Yearly recurring

```text
Operator Annual ............. $99/yr  (or $149/yr)
UIAI Engine Annual .......... $129/yr
Bundle Annual .............. $199/yr
Founders Forge .............. continues at $7,500 lifetime for the cohort
```

Includes:

- Current major + all minors within the paid year.
- Major-version upgrades require renewal (or upgrade add-on).

#### 2B. Major-version pay-per-bump

```text
Operator Lifetime .......... $697 one-time for current major only
Major-version upgrade pass . $199 per major version (v2, v3, v4, ...)
UIAI Engine same ........... $697 one-time + $199 per major
Bundle ..................... $1,497 one-time + $499 per major
Founders Forge .............. $7,500 lifetime (covers all majors)
```

Includes:

- Lifetime access to the major version bought at activation.
- Optional per-major upgrade pass to keep current.
- Skipping a major = staying on prior major indefinitely.

#### 2C. Hybrid (recommended default)

```text
Operator Lifetime .......... $1,497 one-time, current major only
Operator Annual + Major ..... $99/yr, includes all majors while active
Major-version upgrade ....... $199 one-time per major (lifetime holders)
UIAI Engine ................. $1,497 lifetime OR $129/yr annual
Bundle ...................... $2,497 lifetime OR $249/yr annual
Founders Forge .............. $7,500 lifetime, all majors always
```

Includes the safety net of both flavors so neither audience is locked out.

---

## 2. Operator decision matrix

| Trigger | Phase 2 shape | Registry `tier` value | `LocalLicense::tier` |
|---|---|---|---|
| Lifetime not yet closed | Phase 1 | `operator` (lifetime) | `operator` |
| Lifetime closed, yearly chosen | Phase 2A | `operator-annual` | `operator` (mode unchanged) |
| Lifetime closed, per-major chosen | Phase 2B | `operator-major-N` (where N = current major) | `operator` |
| Lifetime closed, hybrid chosen | Phase 2C | `operator-annual` OR `operator-major-N` | `operator` (mode unchanged) |

The runtime `LicenseMode::Operator` value does **not** change shape across phases
— only the *meaning* of buying the tier changes. The daemon sees no
behavioral difference; only the registry's `commercial_use` envelope and the
`features[]` array differ (Phase 2 may add a `paid_through` or `current_major`
field that the local file carries but the daemon currently ignores).

---

## 3. What stays the same across both phases

- `LocalLicense` shape (Spec 118 §1 derives `LicenseMode` from `eval`,
  `commercial_use`, `features`, and `tier`).
- `feature_enabled(feature)` / `require_feature(feature)` API in
  `crates/focusa-core/src/license.rs`.
- Eval users stay eval-only.
- 6 gated commands in §3 of Spec 118 stay gated.
- License file path stays `~/.config/focusa/license.json` (chmod 600).
- 7-day offline grace + 24h re-validation cadence (Spec 112 §4.6).

---

## 4. What changes at the transition trigger

### Code (`crates/focusa-core/src/license.rs`)

**No required code changes** for the phase transition itself. The tier field
already accepts arbitrary strings; the runtime derives mode from a small set
of canonical tier names per Spec 118 §1:

```rust
match self.tier.as_str() {
    "operator" => LicenseMode::Operator,
    "founders-forge" | "founders_forge" => LicenseMode::FoundersForge,
    "team" => LicenseMode::Team,
    "enterprise" => LicenseMode::Enterprise,
    _ => LicenseMode::Operator, // fallback for new tier strings
}
```

Future tier strings (`operator-annual`, `operator-major-2`, ...) all fall
through to `Operator` mode — the daemon behavior is unchanged.

**Optional additions** (only if Phase 2A/2B/2C need runtime awareness):
- `paid_through: Option<String>` field on `LocalLicense` (ISO 8601).
- `current_major: Option<u32>` field on `LocalLicense` (e.g., `2` for v2.x).
- `focusa license status` JSON includes those fields when set.
- `focusa license doctor` warns when `paid_through` < today or
  `current_major` < installed major.

### Registry (WPUIAI backend at wpuiai.com)

- Add `paid_through` to the `/wp-json/wpuiai-ai-cloud/v1/license/validate`
  response shape.
- Add `current_major` to the same.
- Gate new `operator` purchases behind a cutoff date.
- Honor grandfathering per §5.

### `https://install.focusa.dev/license` page

- Replace "Focusa Operator Lifetime License: $697" copy with phase-specific
  copy when triggered.
- Add a `paid_through` disclosure for annual buyers.
- Surface `current_major` and the upgrade price for lifetime holders.

### `docs/PRD.md` and `LICENSE-FAQ.md`

- Add a "Pricing schedule" section pointing at this spec.
- Reflect whichever Phase 2 shape is active.

---

## 5. Grandfathering (proposed)

**Default rule (recommended):** any license key issued *before* the
transition trigger date keeps its Phase 1 lifetime promise forever.

- A buyer who paid $697 lifetime on `2026-07-15` keeps that key active in
  `LocalLicense` indefinitely, even after Phase 2 begins.
- Their key continues to receive all majors at no extra charge.
- They are not retroactively billed.

**Exception for Phase 2B (per-major):** if the lifetime promise is rewritten
to mean "lifetime of the major at purchase time," the
contributor license agreement (CLA) and `legal/COMMERCIAL_LICENSE_TEMPLATE.md`
must be updated to reflect that, and existing keys need to be grandfathered
either by extending their tier record or by issuing major-version upgrade
passes for free.

---

## 6. Trigger mechanics

Operator declares the transition by:

1. Setting a `pricing_phase` record in the WordPress registry admin:
   `phase_1` → `phase_2a` (or `2b`/`2c`).
2. Updating `https://install.focusa.dev/license` copy.
3. Emailing all existing buyers with a 14-day notice + FAQ.
4. Updating `docs/PRD.md` and `docs/LICENSE-FAQ.md`.

The trigger does NOT require code changes, daemon restart, or customer action.
New buyers flow through the new shape; existing keys stay valid.

---

## 7. Operator decisions needed before trigger fires

1. **Which Phase 2 shape?** 2A (yearly), 2B (major-version bump), 2C (hybrid).
2. **Trigger date.** When does Phase 1 close?
3. **Refund window post-trigger.** 14 days default, configurable.
4. **Founders Forge behavior.** Stays lifetime forever, or migrates with the
   cohort to whatever the buyer agrees to?
5. **Existing customers.** Full grandfathering, partial, or none?

This spec holds the *shape*; the operator chooses the values when ready.

---

## 8. Source of truth

| Question | Authority |
|---|---|
| What tier maps to what runtime mode? | `crates/focusa-core/src/license.rs::LocalLicense::mode()` |
| What is the current pricing? | `https://install.focusa.dev/license` + Spec 118 §2 |
| What command is gated by what capability? | `crates/focusa-cli/src/commands/license.rs::license_gate_matrix()` |
| When did Phase 2 start? | WordPress registry admin + this spec's §6 trigger record |
| What is grandfathered? | this spec's §5 + commercial addendum the buyer signed |

---

## 9. Open follow-up (do not block MVP)

- Bead: `focusa-119-paid-through-field` — add `paid_through: Option<String>`
  to `LocalLicense` (only if Phase 2A is selected).
- Bead: `focusa-119-current-major-field` — add `current_major: Option<u32>`
  to `LocalLicense` (only if Phase 2B is selected).
- Bead: `focusa-119-install-page-refresh` — script the install.focusa.dev
  copy swap from registry `pricing_phase` record.
- Bead: `focusa-119-grandfather-notice` — email template + suppression list
  for the 14-day pre-trigger notice.

All three (or whichever subset matches the chosen Phase 2 shape) are
non-blocking until the operator decides.

---

## 10. Lifetime cap matrix (DRAFT — operator iterating)

> **Status: DRAFT, not finalized.** Numbers below are the operator's working
> estimates as of `2026-07-06`. Each row will move to "settled" only when
> the operator explicitly says so. The struct (what each cap means,
> how it counts, how it interacts) is the part that matters; the numbers
> are easy to change later via a single registry admin update.

### 10.1 Cap table (draft)

| Tier | Phase 1 price | Cap (draft) | Status |
|---|---|---|---|
| Operator Lifetime | $697 one-time | **150** | iterating |
| UIAI Engine Operator | $697 one-time | **150** | iterating — companion to Focusa; some buyers use other browser tools (Playwright, Puppeteer, Selenium) so demand may undershoot |
| Bundle (Focusa + UIAI Engine) | 10% off sum ($1,254.60) | **50** | iterating — separate ledger question open (§10.3) |
| Founders Forge (paid cohort) | $7,500 ($1,500 deposit path) | **15** | iterating — this is the largest single revenue line at current draft |

**Combined draft ceiling (if every cap fills):** ~$384,330.

```text
Focusa       150 × $697     = $104,550
UIAI Engine  150 × $697     = $104,550
Bundle        50 × $1,254   = $62,730
Founders Forge 15 × $7,500 = $112,500
─────────────────────────────────────────────
Total                          $384,330
```

### 10.2 What each cap means (draft)

- **Operator Lifetime cap = 150** means the registry will sell **at most 150
  unique `tier: "operator"` keys** during Phase 1. After 150 activations
  the registry returns `cap_reached` on new activations; existing 150 keys
  keep working.
- **UIAI Engine Operator cap = 150** is symmetric. Same rule.
- **Bundle cap = 50** means at most 50 bundle keys sold during Phase 1.
- **Founders Forge cap = 15** is the cohort size — at most 15 cohort seats
  total, regardless of deposit-vs-full payment.

### 10.3 Open ledger questions (still being decided)

These are the structural choices the operator is weighing. None are
blocking — drafts can hold all three options until decided.

**Q1 — Does a Bundle sale consume from both the Focusa 150 and the UIAI 150?**

**Operator draft decision (2026-07-06):** No — bundle sales are **not**
separate. Bundles consume from both individual caps (Option C).

| Option | What it means | Status |
|---|---|---|
| A. Bundle consumes from BOTH | 50 bundles + 100 individual Focusa + 100 individual UIAI = 150/150/50 caps filled; cap binding = whichever hits first | superseded |
| B. Bundle is independent ledger | 50 bundles + 150 Focusa + 150 UIAI = 350 unit ceiling (caps stack) | rejected |
| **C. Bundle counts once across BOTH (DRAFT)** | Bundle = 1 from Focusa 150 + 1 from UIAI 150. Max bundles = min(50, remaining F, remaining U). | **operator lean** |

**Mechanical effect of C:**

```text
50 bundles  + 100 Focusa individual  + 100 UIAI individual  = 150/150 filled
0  bundles  + 150 Focusa individual  + 150 UIAI individual  = 150/150 filled (no bundles)
50 bundles  + 0   Focusa individual  + 0   UIAI individual  = 50/150 filled (early)
```

Same combined $384,330 ceiling. Bundle cap of 50 becomes the binding
constraint (stops bundle sales even when individual caps have room).
Updates after this decision: **none required to §10.1**, but §10.6 trigger
now flags Option C as the answer to Q1.

**Q2 — Does a Founders Forge buyer consume a slot in any of the other caps?**

**Operator draft decision (2026-07-06):** **Forge is separate.** Does
NOT count against Focusa 150, UIAI 150, or Bundle 50.

| Option | What it means | Status |
|---|---|---|
| A. Forge independent of caps | 15 Forge + 150 + 150 + 50 = 365 unit ceiling | superseded |
| B. Forge counts against both caps | 15 Forge + 135 individual Focusa + 135 individual UIAI + 50 Bundle, maxing at 150/150/50 | rejected |
| **C. Forge separate, complete package (DRAFT)** | 15 Forge sits on its own ledger. Forge is the only path to get both products + cohort extras at $7,500. Individual caps untouched by Forge sales. | **operator lean** |

**Mechanical effect of C:**

```text
15 Forge         = 15 complete Focusa+UIAI+extras buyers (cohort)
+ 50 Bundle      = 50 buyers, each consuming 1 F + 1 U
+ 100 Focusa     = 100 individual
+ 100 UIAI       = 100 individual
= 365 unit ceiling across all four tracks
= $384,330 revenue cap
```

Forge being separate keeps the cohort signal clean: the 15 Forge buyers
are *cohort members*, not just premium-purchasers. They get the
1:1 access, the cohort extras, and a different relationship to the
operator than a $697 buyer. Putting them on the same ledger would
muddy that signal.

Updates after this decision: **none required to §10.1 cap table**. The
trigger email in §10.6 should explicitly state "Forge is separate" so
the registry admin doesn't accidentally cross-attribute Forge sales
into the Focusa 150 counter.

**Q3 — Does a Bundle buyer get the cohort extras that come with Forge?**

Bundle at $1,254.60 is mostly a price-saver. Forge at $7,500 is mostly a
cohort. The two probably serve different audiences, but the spec should
state whether bundle buyers can later upgrade to Forge (and how).

### 10.4 Display (TBD)

Not yet decided:

- **Public counter:** show remaining count on install.focusa.dev
  ("47 of 150 Operator Lifetime remaining") — drives scarcity.
- **Silent inventory:** no public counter, just a hard cutoff when caps hit.
- **Hybrid:** public counter only on the landing page, not on every page.

### 10.5 What changes in code when the draft becomes settled

When the operator says "lock the caps," these things change:

1. The `tier: "operator"` cap (150) and `tier: "founders-forge"` cap (15)
   get set in the WordPress registry admin.
2. `https://install.focusa.dev/license` page is updated to show the caps
   (either via public counter or via "limited to 150" copy).
3. `docs/118-focusa-license-tiers-spec.md` §2 gets a note in each tier row
   referencing the cap.
4. `LICENSE-FAQ.md` gets a "Why are there lifetime caps?" entry.
5. A new static test (proposed: `tests/spec_119_lifetime_caps_static_test.sh`)
   verifies the cap numbers in the spec match the cap numbers in the
   registry admin (manual one-time check at lock time).

**No code changes in `crates/focusa-core/src/license.rs`.** The cap is a
registry-side concern, not a runtime concern.

### 10.6 When the draft closes

Once the operator locks the caps, this section flips from DRAFT to SETTLED
and the **operator's email is the trigger**:

> Subject: Lock lifetime caps at the draft values
> Body: Confirm the four numbers (150 / 150 / 50 / 15) and which ledger
>       option (A/B/C) for Bundle accounting and Forge accounting.

After that email, the registry admin applies the caps, install.focusa.dev
updates copy, and the docs flip from DRAFT to SETTLED.

### 10.7 What is NOT a draft

The structural shape — caps exist, they have counters, they trigger a
registry-side cutoff, existing keys stay valid — is **not** a draft. That
shape is settled by the spec. Only the **numbers** and **ledger overlap
choice** are draft.

