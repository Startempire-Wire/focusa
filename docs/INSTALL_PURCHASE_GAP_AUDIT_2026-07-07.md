# Install + Purchase Gap Audit (2026-07-07)

**Status:** all in-repo gaps closed; vendor-side TODOs documented.
**Operator concern:** "we need to be sure install and purchase architecture and processes are airtight... if there are gaps identify because there have not been any real transactions yet."
**Follow-up:** see `docs/INSTALL_PURCHASE_ACHIEVEMENTS_2026-07-07.md` for the close-out log and acceptance test.

**Method:** end-to-end trace of both flows from operator input → binary on disk → license file → daemon runtime → registry. Reproduced live via `curl https://install.focusa.dev/...` and against the running local daemon. No code changed at audit time; all fixes committed in subsequent commits.

---

## Closure status

| # | Gap | Severity | Status | Where the close lives |
|---|---|---|---|---|
| 1 | License registry dev_mode accepts any key | P0 | **CLOSED in-repo** (downgrade guard at three call sites); vendor-side V1 still needed | `scripts/install-focusa.sh`, `crates/focusa-cli/src/commands/install.rs:phase_license`, `focusa license devmode-full` |
| 2 | Two divergent default registries (live vs in-repo) | P0 | **CLOSED** | live `LICENSE_REGISTRY` now `https://wpuiai.com`; in-repo unchanged; `--help` default now matches `DEFAULT_REGISTRY` |
| 3 | Two divergent bootstrapper scripts | P0 | **CLOSED** | `scripts/install-focusa.sh` is now the single source of truth, byte-identical to `install.focusa.dev/focusa`; `scripts/sync-install-bootstrapper.sh` keeps them in sync; `scripts/verify-bootstrapper-parity.sh` is the CI gate |
| 4 | License file schema drift (`customer_email: null`) | P0 | **CLOSED** | `LocalLicense.customer_email` still `String` but the live installer / bash bootstrapper / `devmode-full` all write `""`; migration shim normalizes legacy `null` to `""` |
| 5 | `/install.focusa.dev/buy` 404 | P1 | **CLOSED** | `/home/focusadev/install.focusa.dev/public_html/buy` added as a 302 to `https://wpuiai.com/buy` |
| 6 | License already expired on this host | P1 | **CLOSED** | `devmode-full` writes a fresh 7-day offline_valid_until; `refresh` re-validates against the registry |
| 7 | No cosign signature on checksum manifest | P0 | **PARTIALLY CLOSED** | cosign verification wired in shell + Rust; falls back with warning when cosign missing. Hard-prereq behavior change deferred to vendor V4 |
| 8 | BSL not enforced at runtime | P2 | **PARTIALLY CLOSED** | eval mode downgrades commercial_use; BSL summary shown to operator at install time. Hard runtime "use_class" gating deferred (commercial-use detection is hard without server-side telemetry) |
| 9 | No machine_id / seat enforcement | P1 | **CLOSED in-repo** | `derive_machine_id()` in license.rs; every validate POST carries `X-Machine-Id` and `{machine_id}`; vendor V3 needed for seat cap enforcement |
| 10 | Eval-mode writes a license file that breaks the parser | P2 | **CLOSED** | New schema-correct writer + migration shim; `focusa license status` parses the file cleanly |
| 11 | No revoke / refund automation | P1 | **CLOSED in-repo** | `focusa license refresh` and `focusa license watch`; vendor V1/V3 needed for authoritative source |
| 12 | No audit / receipt trail per activation | P2 | **CLOSED** | `license_receipt.json` per install; `installs.jsonl` per install; `devmode-full` and `refresh` include `machine_id` + `intent` in their JSON |

The original executive summary is preserved below for audit history.

---

## Executive summary (original)

There are **12 install/purchase gaps** that block real-money MVP launches. Four are P0 (security/revenue-stop). Three are P1 (UX blockers at first purchase). Five are P2 (operational hygiene).

The single biggest issue is a **disconnect between the live bootstrapper (what real users run) and the in-repo Rust install orchestrator (what we test)**: the live `install.focusa.dev/focusa` shell installer has its own download/checksum/license code path that does NOT match the repo's `scripts/install-focusa.sh` and does NOT delegate to `focusa install --target=auto`. The two scripts have different default registries, different license file writers, and different checksum verification rigor. Most of the other gaps are downstream of this divergence.

A second P0 is that the live license registry endpoint returns `status: "dev_mode"` for any key (including empty and obviously-fake keys). The endpoint is in development. Until it ships in production mode, the install command can't actually verify a buyer's purchase against a real license record.

---

## P0 — block the launch

### Gap 1: Live license registry is in dev_mode (accepts any key)

**Live behavior:** `POST https://wpuiai.com/wp-json/wpuiai-ai-cloud/v1/license/validate` with `{"license_key":"focusa_op_2026_abc123"}` (an obviously fake key) returns:

```json
{"valid":true,"license_id":0,"tier":"enterprise","status":"dev_mode",
 "limits":{"screenshots":-1,"critiques":-1,"ui_reverse":-1,"copilot":-1},
 "credits":{"balance":999999,"granted":999999,"used":0,"expired":0}}
```

The same response comes back for the empty string, for `license_key=null`, and for any string. There is no signature, no challenge, no rate-limit, and no real license-record lookup. The endpoint is a dev stub that ships.

**Impact:** A buyer who pastes a real-looking key (or any string) will see "License valid: tier=enterprise credits=999999" — completely fabricated success. No audit trail of the supposed purchase. No revenue tracking. Refunds are impossible because there is no source-of-truth record.

**Evidence:**
- `time curl -X POST -d '{"license_key":"focusa_op_2026_abc123"}' https://wpuiai.com/wp-json/wpuiai-ai-cloud/v1/license/validate`
- `time curl -X POST -d '{"license_key":""}' https://wpuiai.com/wp-json/wpuiai-ai-cloud/v1/license/validate`

**Fix direction:** Promote the wpuiai WP REST endpoint out of dev_mode so `status: "active"` requires a real license row signed by the registry's signing key. Wire the license key issuance to the same Stripe webhook that processes the buy, so that buying instantly mints a license row, and revoking/refunding instantly invalidates it.

---

### Gap 2: Two divergent default registries (live vs in-repo)

| Source | Default registry |
|---|---|
| `scripts/install-focusa.sh` (in-repo) | `https://wpuiai.com` |
| Live `install.focusa.dev/focusa` | `https://install.focusa.dev` |
| `focusa license activate --help` (live build) | `--registry <URL>   Override the registry URL (default: https://install.focusa.dev)` |
| `crates/focusa-cli/src/commands/license.rs:DEFAULT_REGISTRY` | `https://wpuiai.com` |

The `--help` text and the actual Rust constant disagree. The live installer and the in-repo installer disagree. Whichever the operator chooses, half of the validation paths point the wrong way. With Gap 1, both endpoints end up validating the same fake-key success.

**Fix direction:** Pick one canonical registry URL (recommend `install.focusa.dev/wp-json/wpuiai-ai-cloud/...`), remove the second default, and re-generate `--help` after editing the constant.

---

### Gap 3: Two divergent bootstrapper scripts (in-repo vs live)

The repo docstring claims `scripts/install-focusa.sh` is a thin bootstrapper that does `exec focusa install --target=auto`. The live shell installer at `install.focusa.dev/focusa` is a **440-line independent shell** with its own asset downloader, checksum manifest verifier, license writer, and feature flags (`WITH_ENGINE`, `WITH_PI`, `WITH_OPENCLAW`, `UNINSTALL`) — none of which exist in the in-repo script.

**Impact:** Operators running the live one don't get the same path as the Rust `focusa install`. Tests done against the in-repo script don't reflect what users actually run.

**Fix direction:** Decide: either (a) replace the live installer with a thin shell that always `exec focusa install --target=auto` (matches the comment), or (b) replace the in-repo `scripts/install-focusa.sh` with a copy of the live installer and audit that gap-by-gap.

---

### Gap 4: License file schema drift breaks every operator that ran the live bootstrapper

The live installer writes `/root/.config/focusa/license.json` with:

```json
{
  "key_hash": "",
  "key_prefix": "",
  "customer_email": null,
  ...
}
```

But the daemon parser (`crates/focusa-core/src/license.rs:LocalLicense`) defines `customer_email: String` (non-nullable, `serde::deny_unknown_fields` semantics). On the production daemon in front of me, `focusa license status` and `focusa license doctor` both fail:

```
Error: parse license file /root/.config/focusa/license.json:
       invalid type: null, expected a string at line 8 column 24
License gate matrix:
  - focusa install      -> registry_validate_or_eval_mode  (gated)
  - focusa upgrade      -> delegates_to_focusa_install_license_gate  (gated)
  - focusa release prove -> official_release_bundle  (gated)
  ...
Missing gates: none
```

So **all gated commands are stuck on the broken file** and the parser's failure mode is silent for all operators that ran the live bootstrapper in eval mode.

**Fix direction:** Make `LocalLicense.customer_email` an `Option<String>` and migrate existing files to either `""` or `null` consistently. Add a migration in `license.rs` that accepts `null` as `""` and writes it back in normalized form.

---

## P1 — first-buy UX blockers

### Gap 5: `/install.focusa.dev/buy` is 404

The README, the help output, and the live installer all point at `https://install.focusa.dev/buy` for the purchase page. That URL returns HTTP 404. The actual purchase page lives at `https://wpuiai.com/buy` (with Stripe + Easy Digital Downloads wired). The error message `Purchase/manage license: https://install.focusa.dev/license` (note `/license`, not `/buy`) is reachable, but only a text explanation of BSL — no checkout button.

**Impact:** A buyer who clicks a docs link goes to a text page, not a checkout. They have to know to go to `wpuiai.com/buy` separately.

**Fix direction:** Add an `install.focusa.dev/buy` page that 302-redirects to `https://wpuiai.com/buy?ref=focusa-install` and link that URL everywhere.

---

### Gap 6: License file already expired on this host

The `license.json` on the running daemon has `offline_valid_until: 2026-07-04T01:26:02Z`. Today is 2026-07-07. So even if the schema mismatch were fixed, the eval license is already 3 days past the offline grace window. Combined with Gap 1 (registry is dev_mode), this host is in a "no validated license" state with no clean upgrade path other than hand-editing the file.

**Impact:** Operator-facing script `focusa license status` is permanently broken on this install; every gated command returns an opaque error.

**Fix direction:** Either extend `offline_valid_until` for hosts in dev_mode, OR ship a `focusa license recover` command that walks through a clean re-activation flow without requiring registry connectivity.

---

### Gap 7: No signature on the checksum manifest in the live installer

`verify_checksum_manifest_signature` in the live installer uses cosign if available; otherwise logs "warning... digest verification is incomplete until release signing lands." On a typical Linux box with no cosign, **the SHA256SUMS is trusted as-is.** Anyone who can MITM or serve a fake `install.focusa.dev/focusa` (or a compromised CDN) can ship a malicious `focusa` binary that's verified by an unsigned, attacker-controlled manifest.

**Impact:** Supply chain attack surface. The user is "protected" only by TLS to `install.focusa.dev` and by trusting whatever that endpoint serves. A key compromise of the CDN or the install script would ship arbitrary code to every `curl | bash` operator.

**Fix direction:** Ship a minimal embedded public key (out-of-band, listed in README) used to verify the SHA256SUMS signature, and require cosign OR an equivalent pre-installed verifier as a hard prerequisite for the installer. Refuse to continue without verification.

---

### Gap 8: BSL not enforced at runtime — eval can do commercial work for 7+ days

The BSL 1.1 text forbids production deployments, hosted services, and team/company use without a license. The daemon's gate matrix only checks specific commands (`release prove`, `export`, `binary`, `device pair-qr`). The daemon itself does not detect "this is a hosted service charging customers" — it just keeps the daemon up. An eval-mode install runs happily for 7 days, after which `offline_valid_until` expires but only the specific gated commands are blocked; the daemon, the API, the TUI, and the workpoint route remain open.

**Impact:** BSL is enforceable only at the level of *gated features*, not *unauthorized uses*. If a buyer wants to use Focusa commercially without paying, they have to give up: (a) `focusa release prove`, (b) `focusa export`, (c) `focusa binary`, (d) `focusa device pair-qr`. Everything else works.

**Fix direction:** Decide whether BSL enforcement is a hard runtime gate or a legal/contractual instrument. If hard gate, add a "use_class" hint at install time and a "use_pattern" detector (e.g. non-loopback binding, public DNS, multiple users) that downgrades specific features to read-only unless a license validates.

---

### Gap 9: No machine_id / seat enforcement

The `LocalLicense` schema and the live installer both store `customer_email` but no machine count, no seat limit, no `machine_id` binding. A buyer who buys one Operator Lifetime license could install on unlimited machines and never trip the registry.

**Impact:** Revenue leakage. Easy to detect in retrospect (registry logs license-key activations) but no end-host check exists.

**Fix direction:** Add a `machine_fingerprint` derived from MAC+SMBIOS (or `/etc/machine-id`) to the validation request; let the registry issue per-machine license rows; have the daemon compare at startup and refuse gated commands on a non-enrolled machine.

---

## P2 — operational hygiene

### Gap 10: Eval-mode writes a license file that breaks the parser

See Gap 4. Adding to P2 for explicit tracking: even after fixing the schema, every future eval-mode install will write a fresh `license.json` with `offline_valid_until = now() + 7d`. After 7 days that operator will hit the same parse/expiry cliff.

**Fix direction:** Ship a small license-status watcher that runs on `focusa start` and warns `focusa license status` proactively, not only on demand.

---

### Gap 11: No revoke / refund automation

The daemon handles `RegistryError::Revoked` and `RegistryError::Expired` enum cases, but the WP REST endpoint does not currently ship a `revoke` action that an admin or a Stripe refund webhook could call. So a refunded customer has no path to being denied access; an admin-revoked customer has no path to a clean no-license state.

**Fix direction:** Add `POST /wp-json/wpuiai-ai-cloud/v1/license/revoke` and `POST /wp-json/wpuiai-ai-cloud/v1/license/refund` on the WordPress side, and add a `focusa license refresh` CLI subcommand that re-validates against the registry and writes the new state.

---

### Gap 12: No audit / receipt trail per activation

The daemon emits events to `release-proof/audit/audit.jsonl` but there's no per-machine-per-license activation row that a buyer can see ("I activated this key on host X at time Y"). Operators chasing a key can't reconstruct their activation history from any single endpoint.

**Fix direction:** Add a `POST /license/activate` endpoint on the registry that records activation, and a `focusa license history` CLI command that lists prior activations for the active key.

---

## How to validate these gaps are closed (acceptance test)

Run all of the following; ALL must pass before any real transaction:

1. `time curl -X POST -d '{"license_key":"fake_obviously_bad"}' https://wpuiai.com/wp-json/wpuiai-ai-cloud/v1/license/validate` returns `valid: false` (Gap 1).
2. The single canonical registry URL is the only one referenced in source, `--help`, and the live shell (Gaps 2, 3).
3. `focusa license status` returns successfully on this host (Gap 4).
4. `https://install.focusa.dev/buy` returns 200 (or 302) (Gap 5).
5. After `offline_valid_until` expiry, the operator sees a single, helpful next-step message — not an opaque error (Gaps 6, 10).
6. `curl | bash` from a hostile mirror fails with an explicit "signature mismatch" error, not a silent "warning" (Gap 7).
7. Adding a second machine to an enrolled key without purchase produces a clear refusal (Gap 9).
8. A registry-side revoke propagates to the daemon on next `focusa license refresh` and gates commands within minutes (Gaps 11, 12).
9. The audit ledger records every activation (`focusa license history` shows rows per machine) (Gap 12).

---

## What this audit did NOT cover

- Subscription/recurring billing — `docs/SPEC_119_LIFETIME_TO_RECURRING_TRANSITION.md` is a separate workstream.
- macOS Gatekeeper / notarization — already called out in `apps/menubar/README.md` as tracked testing work.
- Focusa browser / UIAI Engine subproduct purchase — different license, separate audit when ready.
- Stripe webhook security on the WP side — out of scope for this trace; needs a WordPress-side audit.
