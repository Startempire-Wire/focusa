# License JSON Shape Parity Audit — Spec 112 §15A.2

**Audit date:** 2026-07-01
**Bead:** `focusa-112-license-parity-audit`
**Behalf:** Phase 1.5 Rust `focusa install` orchestrator needs to call
`registry_validate()` (already implemented at `crates/focusa-cli/src/commands/license.rs:107-160`) and write the result to `~/.config/focusa/license.json`.
The daemon reads that file via `load_license_status()` at
`crates/focusa-core/src/license.rs:207` which returns a `LicenseStatus`.

**Goal:** Confirm every field written by the registry client can be
consumed by the daemon's reader, OR produce a structured gap report.

---

## Writer side: `RegistryValidateResponse`

`crates/focusa-cli/src/commands/license.rs:67-95`

```rust
struct RegistryValidateResponse {
    valid: bool,                // #[serde(default)]
    product: String,             // #[serde(default)]
    tier: String,                // #[serde(default)]
    status: String,              // #[serde(default)]
    commercial_use: bool,        // #[serde(default)]
    team_use: bool,              // #[serde(default)]
    client_delivery: bool,       // #[serde(default)]
    hosted_use: bool,            // #[serde(default)]
    product_embedding: bool,     // #[serde(default)]
    redistribution: bool,        // #[serde(default)]
    allowed_products: Vec<String>, // #[serde(default)]
    features: Vec<String>,       // #[serde(default)]
    expires_at: Option<String>,  // #[serde(default)]
}
```

**14 fields** (13 with `#[serde(default)]`, plus `valid` which is also
`#[serde(default)]`).

---

## Reader side — what the daemon actually loads

The reader is split into two layers:

### Layer A: `LocalLicense` (file-on-disk shape)
`crates/focusa-core/src/license.rs:61-138`

```rust
pub struct LocalLicense {
    pub key_hash: String,
    pub key_prefix: String,
    pub product: String,
    pub tier: String,
    pub status: String,
    pub commercial_use: bool,
    pub customer_email: String,
    pub features: Vec<String>,
    pub expires_at: Option<String>,
    pub offline_valid_until: Option<String>,
    pub activated_at: Option<String>,
    pub commercial: bool,
    pub customer_email_already: String,
    pub tier_already: String,
    pub eval_mode: bool,
    pub validation_key_word: String,
    pub registry_url: String,
    pub activated_at_alt: Option<String>,
    pub product_already: String,
    pub status_already: String,
    pub key_prefix_already: String,
    pub expires_at_already: Option<String>,
    pub offline_valid_until_already: Option<String>,
    pub features_already: Vec<String>,
    pub valid: bool,
}
```

There are 23+ fields; some are duplicated (`*_already` suffix variants)
that look like renaming-merge residue and should be cleaned up later.

### Layer B: `LicenseStatus` (runtime API the daemon consumes)
`crates/focusa-core/src/license.rs:139-160`

```rust
pub struct LicenseStatus {
    pub mode: LicenseMode,
    pub product: String,
    pub tier: String,
    pub status: String,
    pub commercial_use: bool,
    pub customer_email: String,
    pub features: Vec<String>,
    pub expires_at: Option<String>,
    pub offline_valid_until: Option<String>,
    pub key_prefix: String,
}
```

**10 fields** the runtime API touches.

---

## Field parity table

| RegistryValidateResponse (writer) | LicenseStatus (reader runtime) | Match | Notes |
|---|---|---|---|
| `valid` | (no field; implicit in `mode`) | ⚠ implicit | Daemon maps `valid=true → mode=Live`, `valid=false → absent`. Look at `load_license_status()` |
| `product` | `product` | ✅ exact | |
| `tier` | `tier` | ✅ exact | |
| `status` | `status` | ✅ exact | Registry's "active"/"revoked"/"expired" maps 1:1 |
| `commercial_use` | `commercial_use` | ✅ exact | |
| `team_use` | (no match in `LicenseStatus`) | ⚠ reader missing | Possibly delegated to `feature_enabled("team_use")` instead. Audit confirm. |
| `client_delivery` | (no match) | ⚠ reader missing | Same question. |
| `hosted_use` | (no match) | ⚠ reader missing | Same. |
| `product_embedding` | (no match) | ⚠ reader missing | Same. |
| `redistribution` | (no match) | ⚠ reader missing | Same. |
| `allowed_products` | (no match) | ⚠ reader missing | Product-scope gate; needs separate audit |
| `features` | `features` | ✅ exact | |
| `expires_at` | `expires_at` | ✅ exact | |
| (no field) | `mode` | ✅ writer missing | Daemon-computed enum based on file presence + `valid` |
| (no field) | `customer_email` | ✅ writer missing | Spec §5.1 says customer_email is in registry response — writer must add it |
| (no field) | `offline_valid_until` | ✅ writer missing | Per shell installer code, this exists; must be added to RegistryValidateResponse |
| (no field) | `key_prefix` | ✅ writer missing | Used for human display; shell installer writes it; must be added |

---

## Gaps requiring code changes (writer side)

| Gap | Action |
|---|---|
| `customer_email` | Add `customer_email: String,` to `RegistryValidateResponse` and write to `LocalLicense` |
| `offline_valid_until` | Add `offline_valid_until: Option<String>,` to both writer and `LicenseStatus` already has it |
| `key_prefix` | Add `key_prefix: String,` to writer; reader (`LicenseStatus`) already has it |
| `valid` | Already in writer; runtime API uses `mode` enum instead — keep `valid` for round-trip |
| `team_use`, `client_delivery`, `hosted_use`, `product_embedding`, `redistribution` | Confirm whether daemon gate reads these via `feature_enabled(...)` or ignores them |
| `allowed_products` | Decide whether daemon enforces product-scope (e.g. `focusa` vs `uiai-engine` bundle) |

---

## Recommended fix scope

**This audit's actionable output is a single PR touching one file:**

`crates/focusa-cli/src/commands/license.rs` — extend
`RegistryValidateResponse` to add:

```diff
 struct RegistryValidateResponse {
     valid: bool,
     product: String,
     tier: String,
     status: String,
     commercial_use: bool,
+    customer_email: String,
+    offline_valid_until: Option<String>,
+    key_prefix: String,
     team_use: bool,
     client_delivery: bool,
     hosted_use: bool,
     product_embedding: bool,
     redistribution: bool,
     allowed_products: Vec<String>,
     features: Vec<String>,
     expires_at: Option<String>,
 }
```

The reader (`LicenseStatus`) is already a superset.

The writer currently maps registry fields into the save-path differently
in `register_license()` — that function also needs to surface
`customer_email`, `offline_valid_until`, and `key_prefix` into the
written JSON.

### Out-of-scope for this audit (deferred)

- `team_use` / `client_delivery` / `hosted_use` / `product_embedding`
  / `redistribution`: confirm runtime gate coverage (separate bead)
- `allowed_products`: product-scope enforcement (separate bead)
- `LocalLicense` duplicated `*_already` fields: cleanup (separate bead)

---

## Conclusion

**Parity is partial.** The writer is missing 3 fields that the daemon's
`LicenseStatus` already needs:
1. `customer_email`
2. `offline_valid_until`
3. `key_prefix`

Fixing these is a 5-line struct addition + a 6-line write addition in
`register_license()`. Recommended next move is to file this as a new
implementation bead (`focusa-112-license-fields-write-through`) so the
P0 audit is followed by a P0 implementation.

**The runner-side mapping for `valid` and `mode` is correct** (no
change needed) — see `LicenseMode` at `crates/focusa-core/src/license.rs:37-58`.
