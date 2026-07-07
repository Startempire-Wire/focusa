# Focusa License FAQ

## Can I try Focusa personally?

Yes. Personal, educational, evaluation, and non-commercial local use is allowed under the source-available license terms.

## Can I use Focusa inside my company or team?

Commercial, company, team, internal production, hosted-service, or client-delivery use requires a separate commercial license from Startempire Wire.

## Can I use Focusa for paid client work?

A commercial license is required for paid client work, managed agent operations, redistribution, or embedding Focusa into a paid product/service.

## Can I fork Focusa?

Only under the terms in `LICENSE.md`. Forking does not remove the commercial-use restrictions.

## Does Focusa become open source later?

See `LICENSE.md` for the Business Source License change date and future license terms.

## Why are there caps on lifetime licenses?

Lifetime deals are **capped** during Phase 1 so the operator can fund the
Phase 2 transition (yearly subscription or per-major-version bump) without
undercutting future recurring revenue. Once a cap is hit, the registry
stops selling new keys at that tier; existing keys stay valid forever.

**Draft caps as of `2026-07-06`** (numbers are working estimates, not
finalized — see [Spec 119 §10](docs/SPEC_119_LIFETIME_TO_RECURRING_TRANSITION.md)):

- Operator Lifetime: **150**
- UIAI Engine Operator: **150**
- Bundle (Focusa + UIAI Engine): **50**
- Founders Forge: **15**

The companion UIAI Engine cap exists because some buyers pair Focusa with
other browser tools (Playwright, Puppeteer, Selenium, etc.) — the cap is
a soft ceiling, not a demand gate.

After Phase 1 closes, Focusa moves to yearly subscription or per-major-version
pricing (see [Spec 119](docs/SPEC_119_LIFETIME_TO_RECURRING_TRANSITION.md) for
the transition plan). Existing lifetime keys keep working.

## What if I buy a Bundle and later want the Forge cohort?

(See [Spec 119 §10.3 Q3](docs/SPEC_119_LIFETIME_TO_RECURRING_TRANSITION.md#q3--does-a-bundle-buyer-get-the-cohort-extras-that-come-with-forge) — operator iterating, draft.)
Bundle is price-discount only; Forge is cohort + 1:1 access. A Bundle purchase
does not auto-upgrade to Forge. There is no upgrade credit path defined yet
between Bundle and Forge. If you anticipate wanting Forge, buy Forge directly.

## What if Phase 2 transitions while I'm on a lifetime key?

Lifetime keys bought during Phase 1 are grandfathered — they keep working
forever, regardless of when Phase 2 begins. Phase 2 affects new buyers only.
Your existing license file at `~/.config/focusa/license.json` does not need
to change. If a registry re-validation fails (e.g., key revoked), the daemon
has a 7-day offline grace period (Spec 112 §4.6).

## Do caps apply to renewals?

Caps apply to **new activations**, not renewals. Lifetime keys have no
renewal concept (they never expire). Annual keys (post-Phase 2) renew
automatically and don't count against the Phase 1 cap.

## I'm an Operator. Can the UIAI Engine cap hit before the Operator cap?

Yes. The two caps are independent counters. If demand for UIAI Engine is
higher than for Focusa alone (some operators pair Focusa with Playwright,
Puppeteer, or Selenium instead), the UIAI Engine 150 cap could fill first.
Operationally this means the registry stops selling UIAI Engine while
Focusa continues.

## Where are commercial terms?

See `COMMERCIAL.md`, `SUPPORT_TERMS.md`, `TRADEMARKS.md`, and `CONTRIBUTING.md`.
