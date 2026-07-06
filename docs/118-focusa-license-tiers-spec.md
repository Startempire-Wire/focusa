# Focusa License Tiers — Spec 118

**Status:** Authoritative reference for every license tier, capability, and gate
that Focusa enforces in software. Resolves the gap between
`crates/focusa-core/src/license.rs` (which defines 5 modes) and the
install-binary spec (§18.7) which only documents 3.

**Authoritative sources** (single source of truth per row):

| Source | What it owns |
|---|---|
| `crates/focusa-core/src/license.rs` | `LicenseMode` enum + `require_feature` / `feature_enabled` |
| `crates/focusa-cli/src/commands/license.rs::license_gate_matrix()` | Command → gate map (machine-readable) |
| `crates/focusa-cli/src/commands/install.rs::phase_license` | Install-time gate |
| `docs/112-install-binary-architecture-spec.md` §4.6, §18.7 | Re-validation cadence, partial feature tier handling |
| `https://install.focusa.dev/license` | Commercial pricing + BSL boundaries |

---

## 0. Lifetime caps (DRAFT — see Spec 119 §10)

The downloadable Phase 1 lifetime tiers are **capped**. The cap numbers
are a **working draft** as of `2026-07-06`, not finalized. Operator is
still iterating. The structural shape (caps exist, registry enforces,
existing keys survive) is settled; the numbers are not.

| Tier | Phase 1 price | Draft cap | See |
|---|---|---|---|
| Operator Lifetime | $697 | **150** | [Spec 119 §10.1](SPEC_119_LIFETIME_TO_RECURRING_TRANSITION.md#101-cap-table-draft) |
| UIAI Engine Operator | $697 | **150** | same |
| Bundle (Focusa + UIAI Engine) | 10% off sum ($1,254.60) | **50** | same |
| Founders Forge | $7,500 | **15** | same |

**Combined draft ceiling (if every cap fills):** ~$384,330.

**Open ledger questions** (Bundle/Forge overlap with individual caps): see
Spec 119 §10.3. Three options A/B/C are on the table for each.

**When the draft settles:** Spec 119 §10.5 walks through what changes
(registry admin, install.focusa.dev copy, this doc) — but no code change
to `crates/focusa-core/src/license.rs`.

---

## 1. Tier enum (authoritative)

`LicenseMode` in `crates/focusa-core/src/license.rs:37`:

```rust
pub enum LicenseMode {
    Evaluation,
    Operator,
    FoundersForge,
    Team,
    Enterprise,
}
```

`LocalLicense::mode()` derives the runtime mode from `eval`, `commercial_use`,
`features`, and `tier`:

| Source field | → Mode |
|---|---|
| `eval == true` | `Evaluation` |
| `commercial_use == false` AND `features.is_empty()` | `Evaluation` |
| `tier == "operator"` | `Operator` |
| `tier == "founders-forge"` or `"founders_forge"` | `FoundersForge` |
| `tier == "team"` | `Team` |
| `tier == "enterprise"` | `Enterprise` |
| anything else with `commercial_use == true` | `Operator` (default fallback) |

After the **Change Date** (`2030-01-01` per `LICENSE.md`) the BSL terms convert
to Apache 2.0 and every restriction in this doc expires. That post-2030 state
is **Open** (not an enum variant — the license file is irrelevant after the
Change Date).

---

## 2. Per-tier reference

### 2.1 Evaluation

- **Source:** `LocalLicense::evaluation()` constructor in `license.rs:102`.
- **Who:** Anyone running the installer with `--eval`, or anyone without a
  license file (the file does not exist → `load_license_status()` returns
  the eval default).
- **Commercial use:** ❌ blocked by `require_feature` (returns
  `LicenseError::EvaluationRestricted(feature)`).
- **Daemon mark:** `/v1/health` reports `tier: "evaluation"` (Spec 112 §4.5).
- **Rate limit:** 100 req/min on the daemon (Spec 112 §4.5).
- **Install:** allowed via `--eval` only. `focusa install` without
  `--license-key` or `--eval` returns an error.
- **Features:** none. `features[]` is empty.
- **What you can do:**
  - `focusa start` / `stop` / `doctor`
  - `focusa workpoint` / `trajectory` / `recall`
  - `focusa-tui` (full Mission Deck, observation + learning)
  - `focusa license status` / `doctor` / `check-feature`
  - All read paths in `/v1/*` API.
- **What you cannot do** (gated):
  - `focusa install` without `--eval`
  - `focusa release prove` (`official_release_bundle`)
  - `focusa export` (`commercial_export`)
  - `focusa binary` (`packaged_installer`)
  - `focusa device pair-qr` (`qr_pwa_handoff`)
  - `feature_enabled("public_stream")` returns `false` → API blocks
    `/v1/awareness/publish` for non-loopback consumers
- **Pricing:** Free. No key required.

### 2.2 Operator

- **Source:** `tier: "operator"` in license.json, returned by the registry.
- **Who:** Single operator / individual pro license. Lifetime.
- **Commercial use:** ✅.
- **Features (registry-controlled):** at least `["daemon", "tui"]` per
  Spec 112 §4.4. The actual `features[]` array is the registry's contract;
  if a feature is missing, the daemon refuses to start that feature
  (Spec 112 §18.7).
- **Install:** `bash focusa-install.sh --license-key focusa_live_xxxxx`.
- **Pricing:** $697 lifetime, per install.focusa.dev/license.
- **Daemon mark:** `tier: "operator"` in `/v1/health`.
- **Use case:** Personal commercial work. Local machine, no team seats,
  no hosted service.

### 2.3 FoundersForge

- **Source:** `tier: "founders-forge"` (or `"founders_forge"`) in
  license.json. The mode parser accepts both spellings.
- **Who:** Paid cohort members of the Founders Forge program.
- **Commercial use:** ✅.
- **Features:** registry-controlled; typically Operator-tier features
  plus cohort-specific extras (early-access builds, dedicated support
  channel, cohort-only proof artifacts).
- **Pricing:** $7,500 ($1,500 deposit + remainder) per
  install.focusa.dev/license.
- **Why it's a separate mode:** the cohort program has its own
  `commercial_use` semantics, support contract, and refund terms
  (see the Founders Forge program page); the license JSON reflects
  this via `tier: "founders-forge"`.
- **Daemon mark:** `tier: "founders-forge"` in `/v1/health`.

### 2.4 Team

- **Source:** `tier: "team"` in license.json.
- **Who:** Multi-seat team license. Seats come from `max_users` in the
  registry response (Spec 112 §4.4).
- **Commercial use:** ✅.
- **Features:** registry-controlled; at least all Operator features.
- **Pricing:** Sales-contact (per install.focusa.dev/license: "contact sales"
  for multi-seat / team scopes).
- **Seat enforcement:** when `max_users > 0`, the daemon should track
  active sessions per `customer_email` from the license file and refuse
  new sessions when the cap is exceeded. (Not yet enforced in code; tracked
  as `focusa-team-seat-cap-enforcement` if needed.)
- **Daemon mark:** `tier: "team"` in `/v1/health`.

### 2.5 Enterprise

- **Source:** `tier: "enterprise"` in license.json.
- **Who:** Hosted-service, multi-team, or white-label deployments.
- **Commercial use:** ✅.
- **Features:** all Operator features plus `menubar`, custom features
  per Spec 112 §18.7.
- **Pricing:** Sales-contact.
- **Daemon mark:** `tier: "enterprise"` in `/v1/health`. All surfaces
  available; no rate-limit cap.

---

## 3. Command gate matrix (authoritative)

Generated by `license_gate_matrix()` in
`crates/focusa-cli/src/commands/license.rs:586`. Run
`focusa license doctor` to see live.

| Command | Side effect | Required gate | Gate status | Evidence |
|---|---|---|---|---|
| `focusa install` | install / replace binaries + service | `registry_validate_or_eval_mode` | **gated** | `install.rs::phase_license` |
| `focusa upgrade` | atomic binary swap | `delegates_to_focusa_install_license_gate` | **gated** | `upgrade.rs` (delegates) |
| `focusa release prove` | official release bundle proof | `official_release_bundle` | **gated** | `release.rs::require_feature` |
| `focusa export` | commercial export artifact | `commercial_export` | **gated** | `export.rs::require_feature` |
| `focusa binary` | packaged installer generation | `packaged_installer` | **gated** | `binary.rs::require_feature` |
| `focusa device pair-qr` | QR PWA device handoff | `qr_pwa_handoff` | **gated** | `device_pairing.rs::require_feature` |
| `focusa license activate/deactivate` | local license state admin | `not_required_license_administration` | **not_required** | `license.rs` |

Eval users get blocked at rows 1, 3, 4, 5, 6 with the structured
error shape:

```json
{
  "error_code": "evaluation_restricted",
  "feature": "commercial_export",
  "message": "evaluation mode — feature 'commercial_export' not permitted",
  "upgrade": "https://focusa.dev/buy"
}
```

For tier-restricted (non-eval) features, the error is
`LicenseError::FeatureRequiresLicense` with the same shape and
`error_code: "feature_requires_license"`.

---

## 4. Install-time flow (Spec 112 §4.2)

```
[1] User runs: curl ... | bash -s -- --license-key focusa_live_xxxxx
[2] Installer extracts LICENSE_KEY from args
[3] Installer POSTs to /license/validate with the key
[4] WordPress (registry) responds with {valid: true, tier, ...}
[5] Installer writes license.json to ~/.config/focusa/license.json
[6] focusa-daemon reads license.json on startup, validates against registry
[7] Daemon reports tier in /v1/health; gates run per §3
```

For eval installs, `[3]` is skipped, `[4]` is skipped, `[5]` writes
`{ eval: true, tier: "evaluation", features: [] }`.

---

## 5. Re-validation + revocation

Spec 112 §4.6:

- Daemon re-validates against the registry every 24 hours.
- 7-day offline grace period after each successful validation
  (`offline_valid_until` field).
- If registry returns `valid: false` (revoked / refunded / expired):
  daemon enters `license_expired` mode, refuses new mutations, allows
  reads, emits alert via `/v1/doctor` and Focus Slice.
- `focusa license status` reports `expires_at` and
  `offline_valid_until` to operators.

---

## 6. API surface (daemon `/v1/*`)

| Endpoint | License impact |
|---|---|
| `GET /v1/health` | Always returns; includes `tier` field |
| `GET /v1/license/status` | Returns full `LicenseStatus` (mode, tier, features, expiry) |
| `GET /v1/license/doctor` | Returns gate matrix + missing gates + recovery hint |
| `GET /v1/awareness/publish` | Gated by `feature_enabled("public_stream")` |
| All read endpoints (`/v1/state/*`, `/v1/clt/*`, etc.) | Always allowed in any tier |
| All write endpoints that mutate canonical state | Subject to per-endpoint gate (none defined yet outside the gates in §3) |

---

## 7. What is **not** enforced yet

These are tracked as deferred or roadmap items per `PRE_MVP_LAUNCH_READINESS_2026-07-06.md`:

- **Team seat cap enforcement** (`max_users` from registry).
  Currently the field is parsed but not enforced. Eval/Operator
  installs ignore `max_users`.
- **License presence on daemon write paths.** Today only the
  6 commands in §3 are gated. Future work: extend
  `require_feature("...")` to daemon write paths when the team
  asks for stricter enforcement.
- **TUI license plane display.** The TUI does not yet surface the
  current `LicenseMode` in the footer. Roadmap:
  `focusa-tui: gate commercial-only features behind LicenseGuard`.

---

## 8. Quick reference — what tier do I need?

| Want to… | Need |
|---|---|
| Run Focusa locally and try it | **Evaluation** (`--eval`) |
| Use Focusa on real client work, single seat | **Operator** ($697 lifetime) |
| Join the paid cohort program | **FoundersForge** |
| Run Focusa across a team (seats + max_users) | **Team** (sales-contact) |
| Run Focusa as a hosted service or embedded product | **Enterprise** (sales-contact) |
| Use the Mac menubar | **Enterprise** (or higher; feature is gated under `menubar`) |
| Pair a Mac via QR | **Operator+** (`qr_pwa_handoff` is gated) |
| Publish awareness streams to non-loopback consumers | **Operator+** with `public_stream` in `features[]` |
| Export commercial artifacts | **Operator+** with `commercial_export` in `features[]` |
| Build packaged installers | **Operator+** with `packaged_installer` in `features[]` |
| Cut an official release bundle | **Operator+** with `official_release_bundle` in `features[]` |

---

## 9. Where to look next

- File: `docs/PRE_MVP_LAUNCH_READINESS_2026-07-06.md` — pre-MVP sweep that captured this gap.
- File: `docs/evidence/SPEC112-LICENSE-JSON-PARITY-AUDIT.md` — writer/reader parity audit.
- File: `tests/spec96_license_boundary_static_test.sh` — static guard for LICENSE.md / Cargo license-file metadata.
- File: `tests/spec_license_doctor_gate_matrix_static_test.sh` — static guard for license gate matrix.
- File: `tests/spec_sbom_license_gate_static_test.sh` — SBOM license gate.
- URL: `https://install.focusa.dev/license` — operator-facing commercial explanation.
- URL: `https://install.focusa.dev` — install tracks (`--eval` vs licensed).
- CLI: `focusa license status` / `focusa license doctor` / `focusa license check-feature <name>`.