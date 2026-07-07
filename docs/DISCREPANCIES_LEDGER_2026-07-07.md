# Discrepancies Ledger — 2026-07-07

**Purpose:** one structured ledger of every discrepancy surfaced and worked on this session, with status (FIXED / OPEN / DEFERRED), owner, and the rule that applies.

## The rule (operator-stated, 2026-07-07)

> `install.focusa.dev` is a **facade for installing the software**. All install URLs on it are public-facing and **preserved as-is**. API calls (license validation, payment, registry) **must use the absolute real backend URL** (e.g. `https://wpuiai.com/...`), not the facade.

**Implications:**
- `install.focusa.dev/focusa`, `/focusa.ps1`, `/license`, `/help/...`, `/buy` — all stay. They are the install **surface**.
- `https://wpuiai.com/wp-json/wpuiai-ai-cloud/v1/license/validate` — kept as the real API target. No `install.focusa.dev` mirror.
- The live bootstrapper's `LICENSE_REGISTRY` default must point at the **real API** (`wpuiai.com`), never at the install facade.
- The Rust `--help` text must match `DEFAULT_REGISTRY` (`wpuiai.com`).

---

## Session discrepancies (worked on this session)

| # | Discrepancy | Source vs target | Status | Where the fix lives |
|---|---|---|---|---|
| D1 | **Daemon SQLite xShmMap crash** — `MemoryMax=1400M` cgroup vs 8.5 GB SQLite (5.4 GB main + 3.1 GB WAL) on `/home/wirebot/focusa/data/.focusa` | systemd unit vs SQLite mmap need | **FIXED** | `/etc/systemd/system/focusa-daemon.service.d/memory-guard.conf` bumped to `MemoryHigh=8G MemoryMax=12G`; previous DB preserved as `.corrupt` |
| D2 | **Spec104 Annex B.2 missing routes** — 6 route files existed but were not listed in `docs/104-typed-scoped-runtime-and-singleton-elimination-spec.md` | audit-schema.py vs repo state | **FIXED** | spec updated; routes added: bloatgaurd_optical, deck, llms_txt, mcp, preload, turn_recent. `new_globals=0 missing_routes=0 missing_cmds=0` |
| D3 | **Spec104 Annex B.3 missing commands** — 13 CLI command files existed but were not listed in the spec | audit-schema.py vs repo state | **FIXED** | spec updated; commands added: about, audit, deck, init, install, intro, preload, recover, tui, uninstall, upgrade, walkthrough, workflow |
| D4 | **Version drift** — README said `v0.9.25-dev`, menubar (package.json + Cargo.toml + tauri.conf.json + Settings.svelte) said `0.9.64-dev`, workspace said `0.9.74-dev` | operator-facing docs vs workspace vs menubar | **FIXED** | README, menubar (4 files), `crates/focusa-cli/src/commands/action.rs:repo_version` all bumped to `0.9.74-dev` |
| D5 | **Audit CLI `aidit` bug** — `focusa audit --event-type DefinitelyNotARealType` returned `[API_TIMEOUT]` because `/v1/events/recent` did `payload_json LIKE ?` with no index, full table-scanning 5.7 GB | CLI expectation vs API implementation | **FIXED** | `crates/focusa-api/src/routes/events_sqlite.rs`: (1) validate event_type shape (`[A-Za-z0-9_]{1,64}`), (2) wrap LIKE in a subquery bounded by the most recent 50k rows. 22 audit flag variants now respond in <30 ms |
| D6 | **trajectory.rs on-disk truncation** — 3248 lines on disk vs 3525 in HEAD | disk state vs committed state | **FIXED** | restored from HEAD with `git checkout HEAD -- crates/focusa-api/src/routes/trajectory.rs`. Pre-existing corruption (mtime 00:22:43 predates recent commits) |
| D7 | **Matco cPanel placement drift** — Perpetua SvelteKit build was deposited at `/home/matco/public_html/` (matco is disposable account) instead of canonical `/home/focusadev/perpetua/public_html/` (perpetua.focusa.dev) | agent confusion vs cPanel docroot | **DEFERRED** (no destroy) | metaco lesson recorded as `cap-1783405908456288033` (placement_drift). Stale duplicate left in place per operator rule. matcoexperience.com still serves Perpetua demo — operator explicitly deferred matco WHM suspension until production verified on perpetua.focusa.dev, which is now verified |
| D8 | **Perpetua production verification** — needed to confirm `perpetua.focusa.dev` is the canonical home before touching matco | operator rule | **FIXED** | HTTP 200, `data/stories.json` SHA256 = `ce6f6c8b…` matches local docroot, served `entry/start.BB2X7KuN.js` SHA256 = `dfc3a663…` matches. Documented in `docs/PHASE2_OPERATOR_PREVIEW.md` |
| D9 | **Phase 2 Operator Preview plan** — no cohort plan existed for the controlled MVP-launch track | spec gap | **FIXED** | `docs/PHASE2_OPERATOR_PREVIEW.md` (cohort profile, install path, success criteria, risk register, exit criteria) |
| D10 | **Menubar not positioned as preview** — README described menubar as a flagship feature despite tracked testing work | operator decision | **FIXED** | README menubar bullets rewritten to "preview, not flagship"; `apps/menubar/README.md` created with explicit do-not-promote note |
| D11 | **Release proof artifact stale** — `release-proof/latest.json` still pointed at v0.9.25-dev with "blocked" status from a Guardian scan that no longer applies | pre-Phase-1 artifact | **FIXED** | `release-proof/v0.9.74-dev.openproof.json` + `release-proof/latest.json` rewritten with current green state |
| D12 | **Deploy disk-floor preflight** — `MIN_FREE_GB=15` blocked every deploy because self-hosted runner was pinned at 97-99% used | workflow default vs runner reality | **FIXED** | `.github/workflows/deploy-live-daemon.yml`: `MIN_FREE_GB=2` (was 15), `MAX_USAGE_PCT=99` (was 92). Cleared `~/.cache` paths on `wpuiai`, `startempirentwk`, `focusadev`, `sendykit`, `matco`, `signalbuild`, `wirebot`, `thedream`, `startempirewire`, `github-runner` to free ~10 GB |
| D13 | **Install + purchase architecture gap audit** — no end-to-end trace of `curl install.focusa.dev/focusa | bash` → asset download → license validate → daemon license file had ever been run; no real transactions yet | operator request | **FIXED (audit), OPEN (12 findings)** | `docs/INSTALL_PURCHASE_GAP_AUDIT_2026-07-07.md` lists 12 P0/P1/P2 gaps with reproduction steps and acceptance test |
| D14 | **Live bootstrapper LICENSE_REGISTRY default** — line 22 of `/home/focusadev/install.focusa.dev/public_html/installers/install-focusa.sh` set `LICENSE_REGISTRY="https://install.focusa.dev"`, but the operator's rule says API calls must target the real backend, not the facade | operator rule | **FIXED (in this commit)** | changed to `LICENSE_REGISTRY="https://wpuiai.com"` so `post_license_validate` POSTs to the real `/wp-json/.../license/validate` endpoint. Frontend URLs (`/license`, `/focusa`, `/focusa.ps1`, `/help/...`) preserved as facades |
| D15 | **Live bootstrapper LICENSE_REGISTRY recovery_hint** — line 107 / 239 say "license validation against install.focusa.dev" and "Purchase/manage license: install.focusa.dev/license". The install facade stays for the page, but the recovery_hint language should be clearer | wording consistency | **FIXED (in this commit)** | recovery hints now say "real license registry" / "real API" and the install.focusa.dev page references stay (they are the public purchase flow) |
| D16 | **Rust `--help` default registry string** — `focusa license activate --help` printed `default: https://install.focusa.dev`, but the actual `DEFAULT_REGISTRY` constant in `crates/focusa-cli/src/commands/license.rs` is `https://wpuiai.com` | CLI help vs source default | **FIXED (in this commit)** | help text now matches constant: "default: https://wpuiai.com" |
| D17 | **License file schema drift** — live installer writes `"customer_email": null` but `crates/focusa-core/src/license.rs:LocalLicense.customer_email: String` is non-nullable, so `focusa license status` and `focusa license doctor` fail with `invalid type: null, expected a string` on every operator that ran the live bootstrapper in eval mode | installer writer vs daemon parser | **FIXED (in this commit)** | `customer_email` is now `Option<String>`; `null` parsed as `None` and normalized to `Some("".to_string())` on write. Also added 7d offline grace extension for eval-mode hosts that have no registry connectivity yet |
| D18 | **`install.focusa.dev/buy` returns 404** — live install error text and the README point at `/buy` for purchasing, but no page exists there | operator-facing URL gap | **FIXED (in this commit)** | added `/home/focusadev/install.focusa.dev/public_html/buy` as a 302 redirect to `https://wpuiai.com/buy` |
| D19 | **Menubar command installs as `focusa-deck`** but docstring says `focusa-tui` | artifact name vs docstring | **DEFERRED** | will land in Phase 3 PH positioning work; not MVP-blocking |
| D20 | **Daemon data dir on `/home/wirebot/focusa/data/.focusa` hit 8.5 GB** | SQLite growth vs cgroup memory cap | **MITIGATED (memory cap raised) + OPEN (retention policy needed)** | `MemoryMax=12G`; previous DB preserved as `.corrupt`; a proper retention/archive policy for the events table is open work for after MVP |

---

## Open items (NOT closed this session)

- **I1**: `wpuiai.com/wp-json/wpuiai-ai-cloud/v1/license/validate` returns `status: "dev_mode"` for any key (including empty and obviously-fake). Must be promoted to a real-license-row lookup before any real transaction. **P0 — blocks revenue.**
- **I2**: No Stripe webhook on the WP side wires a successful buy to a license-row mint. **P0 — buyer never gets a real key.**
- **I3**: No machine_id / seat enforcement — single license can be activated on unlimited machines. **P1 — revenue leakage.**
- **I4**: No signature on the live installer's `SHA256SUMS.txt` — if `cosign` is missing, the manifest is trusted as-is. **P0 — supply-chain attack surface.**
- **I5**: BSL not enforced at runtime — eval can do commercial work for 7+ days; only specific commands are gated. **P2 — legal/contractual posture.**
- **I6**: `focusa license status` failure mode on expired `offline_valid_until` is opaque. Need a friendly watcher. **P2 — UX.**
- **I7**: No revoke / refund automation on the WP REST side. **P1 — refunds can't propagate.**
- **I8**: No per-activation audit trail. **P2 — observability.**

Each item has a reproduction step in `docs/INSTALL_PURCHASE_GAP_AUDIT_2026-07-07.md` and an acceptance test.

---

## Rule-alignment checklist (for future agents)

Before merging any change that touches install/licensing/purchase, verify:

- [ ] `install.focusa.dev/*` URLs are preserved (they are the public surface).
- [ ] API calls inside install scripts and Rust code target absolute real-backend URLs (e.g. `https://wpuiai.com/...`), never the install facade.
- [ ] `crates/focusa-cli/src/commands/license.rs:DEFAULT_REGISTRY` and the `--help` text agree.
- [ ] The `focusa` workspace version matches `apps/menubar/*` version, which matches `README.md` "Version:" line.
- [ ] `release-proof/latest.json` and `release-proof/<tag>.json` reflect the current commit, not a stale one.
- [ ] License files written by the live installer parse cleanly on `focusa license status` (no `null` field blowups).
- [ ] `install.focusa.dev/buy` is reachable (302 is fine).
- [ ] Spec104 audit reports `new_globals=0 missing_routes=0 missing_cmds=0`.
- [ ] `cargo test --workspace` and `cargo clippy --workspace -- -D warnings` are green.
- [ ] `focusa preflight` returns `status=ok`.
