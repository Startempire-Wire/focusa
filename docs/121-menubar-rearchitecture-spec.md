# Spec 121 — Focusa Menubar Rearchitecture

**Status:** draft, iterable, NOT FINAL — operator has not yet signed off.
**Owner:** Focusa / Verious Smith
**Created:** 2026-07-07
**Scope:** apps/menubar (SvelteKit + Tauri shell) and the Focusa-runtime-cockpit surface in the focusa project.
**Inherits:** Spec 103 (call stack), Spec 104 (typed scoped runtime), Spec 105 (DX/UX envelope), Spec 107 (spec-first lifecycle), Spec 108 (Pi plugin awareness), Spec 110 (Pi tool-layer reminder), Spec 115 (cloud control plane), Spec 116 (provider-neutral closure), Spec 117 (mission deck + PWA), Spec 118 (license tiers), Spec 119 (receipts + governance ledger), Spec 120 (adversarial spec workbench — this spec was authored *through* the workbench).
**Out of scope:** focusa core daemon API changes, menubar macOS Tauri packaging, menubar Linux/Windows builds.

---

## 0. One-line definition

The Focusa menubar should pull live, typed, envelope-normalized, receipt-bearing data structures from the daemon, surface only the operator-facing slices of those structures, and support the documented focusa surface (project, trajectory, workpoint, workloop, proof, gate, sync, pair, settings, plus the new 121 surface: receipts, license, cloud, reminders), without code duplication and without assuming the daemon's old flat-response shape.

## 1. Normative basis

1.1. **No code until spec is approved.** Per Spec 120 §11, this spec is iterable. No implementation is staged, no file is edited under apps/menubar/, no bead is closed as a 121 deliverable until operator signs off.
1.2. **Spec is not final.** Operator retains final say. Subsequent revisions are tracked as new drafts with explicit diff sections at the bottom.
1.3. **All surface decisions cite one or more of Spec 103–120.** Each §3 surface must include a `spec_refs` line that points to the authoritative normative basis. A surface with no `spec_refs` is invalid.
1.4. **Menubar reads, never invents.** When a menubar surface disagrees with the daemon's data, the menubar defers to the daemon. The fix is a daemon bug, not a menubar override.
1.5. **Everything typed.** Per Spec 104 §6, the menubar's runtime snapshot replaces `any` with proper interfaces. No `any` outside the legacy `as any` escape hatch (which itself must be tracked as a 121 bead).
1.6. **Everything enveloped.** Per Spec 105 §6, the menubar uses a single `normalize()` that handles both envelope-wrapped and flat responses, exposing `details.tool_result_v1` consistently. The two-track implicit handling is removed.
1.7. **Every POST has a receipt.** Per Spec 119 §6, the 4 POSTs the menubar issues (`workpoint.checkpoint`, `workpoint.resume`, `workpoint.evidence/link`, `device.pair/start`) display the receipt on success and on recoverable failure.
1.8. **Closed under Spec 116.** The menubar's WorkpointPeek shows closure-authority badges per the provider-neutral work-item closure model.
1.9. **Reception honour per Spec 115.** The menubar's header shows a cloud/remote indicator (KH vs OVH), tied to the daemon's Tailscale topology.
1.10. **Tier badge per Spec 118.** The menubar's header shows license tier (free/pro/team). Tier-gated surfaces are explicitly disabled at the menubar level.
1.11. **Reminder state per Spec 110.** Each peek shows whether it has pending reminders; the user can dismiss or act.
1.12. **Onboard per Spec 117.** FirstRunWizard becomes a guided onboarding, not a fix-on-error. Recall surfaces the recent actions.

## 2. Status of the menubar at the time this spec is drafted

| Field | Value |
|---|---|
| `RuntimeSnapshot` interface | all `any \| null` (no types) |
| Envelope handling | two implicit tracks (envelope-wrapped vs flat); `normalizeToolResult` falls back across both |
| Surfaces with data fetched but never displayed | 5 (`predictionsRecent`, `metacogStatus`, `metacogEvaluations`, `snapshotsRecent`, `lineageHead`) — fixed in commit `21d5ed5d` |
| Component name conflict | `CockpitView` renamed to `RuntimeView` because `cockpit` is owned by UIAI Engine desktop browser |
| Tests | `e2e-endpoints.mjs` (33 GET), `e2e-post.mjs` (4 POST). Both pass 31/31 and 4/4. |
| Active tabs | focus / cockpit / trajectory / workpoint / proof / workloop / gate / sync / pair / settings |
| Push state | clean as of this spec draft. Local and remote main at `bcf8dc1a`. |
| Mac Tauri shell | not packaged (focusa-gfwh, focusa-l3t8) — out of 121 scope |

## 3. New surface (121.0) — Receipts Pane

**spec_refs:** Spec 119 §6, §7, §9

**3.1.** A new `ReceiptsPane` component appears in the menubar's tab bar between `proof` and `workloop`. This is the first 121 surface and is the gate for everything else.

**3.2.** On each of the 4 POSTs the menubar issues, the response is run through `receiptFromResponse()`. The resulting receipt is shown in a toast AND persisted in the runtime store as `lastReceipt` so the ReceiptsPane always shows the most recent.

**3.3.** The receipt is a 7-tuple:
   - `receipt_id` (string, monotonic by `created_at`)
   - `created_at` (string, ISO8601)
   - `action` (string, e.g. `workpoint.checkpoint`)
   - `canonical` (bool)
   - `evidence_refs` (array of strings)
   - `rehydratable` (string, an endpoint URL the menubar can fetch to read the receipt back)
   - `next_tools` (array of strings, the focusa CLI commands the operator can run from terminal)

**3.4.** A receipt that returns `canonical: false` is shown with a `watch` tone. A receipt with `failure_class` set is shown with a `bad` tone. The toast disappears in 8s; the persisted receipt in the pane stays until 50 entries deep.

**3.5.** A receipt's `rehydratable` URL is what the menubar GETs to read the receipt back. The menubar must NOT cache the receipt; it must refetch by `rehydratable` when the pane is opened.

**3.6.** This pane replaces the existing single `toastStore.ok` on POSTs. The toast path is kept for in-flight feedback but the persistence is the source of truth.

## 4. New surface (121.1) — License Tier Badge in Header

**spec_refs:** Spec 118 §0, §3, §4

**4.1.** The menubar reads `/v1/license/tier` (a new daemon endpoint; see §11.1) and shows a small badge in the header. Tiers: `free`, `pro`, `team`. The badge is an inline pill; click opens a `LicenseDialog`.

**4.2.** A `pro` tier is required for: cloud sync, evidence link to a non-KH ledger, live daemon over Tailscale. A `team` tier is required for: shared receipts, shared license, multi-user work items. The menubar greys out tier-locked surfaces and surfaces the upgrade CTA.

**4.3.** License-gated surfaces are NOT removed when the user is on a lower tier; they are kept but disabled. This preserves the surface count for the operator to discover what the next tier unlocks.

**4.4.** A `LicenseDialog` is a new Svelte component that shows: current tier, what's enabled, what's locked, a "View on web" deep-link to the focusa license page. The dialog is accessible from the header badge and from `Settings`.

## 5. New surface (121.2) — Cloud Indicator in Header

**spec_refs:** Spec 115 §4, §5

**5.1.** A small cloud icon in the header shows which daemon the menubar is bound to. Three states:
   - `local-only` (no remote, daemon on 127.0.0.1)
   - `kh-direct` (daemon on KH, Tailscale relay via the W4b letta path)
   - `ovh-via-relay` (daemon on OVH, reached via the W7 openclaw path)

**5.2.** The icon colour follows the same `ok / watch / bad / neutral` envelope tones. Click opens a `CloudDialog` that shows: current bind, last successful Tailscale ping, current remote endpoint, "reconnect" CTA.

**5.3.** A `bad` cloud state (last Tailscale ping failed) triggers a `CloudHealthBanner` across the menubar header. This is a hard-warn, not a soft hint.

## 6. Existing surface (121.3) — RuntimeView refactor

**spec_refs:** Spec 103 §6, Spec 104 §6, Spec 105 §6

**6.1.** The current `RuntimeView.svelte` (renamed from `CockpitView.svelte` in 2026-07-07 commit `21d5ed5d`) is preserved as a layout but its 13 card bindings are typed.

**6.2.** The 22 runtime fields (see §2) become a `RuntimeSnapshot` interface. `any` is replaced with specific types per endpoint contract (see Spec 105 §3 for the canonical envelope).

**6.3.** The 4 cards I added in 2026-07-07 (PREDICTIONS, METACOG, SNAPSHOTS, LINEAGE) keep their current rendering. No regression.

**6.4.** A new `TopologyCard` is added between `PROJECT` and `TRAJECTORY`. It shows daemon name, daemon version, work-loop status, and the new cloud indicator (see §5).

**6.5.** A new `ReceiptsCard` is added between `RELEASE` and `RECOVERY` (where the 4 dark-data cards were inserted in `21d5ed5d`). It shows the most recent receipt's `action`, `canonical` state, and `next_tools`. The full receipts list is in the ReceiptsPane (§3).

## 7. Existing surface (121.4) — TrajectoryPeek refactor

**spec_refs:** Spec 105 §6, Spec 110

**7.1.** `TrajectoryPeek` shows the L/M/S ladder from `/v1/trajectory/view`. Per Spec 110, each ladder rung is also a "reminder source" — the peek shows the reminder count next to each rung.

**7.2.** A new `TrajectoryPeek` button `Adopt this HLT` lets the operator commit the HLT to a Spec 120 workbench entry. This is a 121 future-looking affordance, not a runtime action. The button is greyed unless license ≥ pro (see §4).

## 8. Existing surface (121.5) — WorkpointPeek refactor

**spec_refs:** Spec 116 §3, §4

**8.1.** `WorkpointPeek` shows the active workpoint with closure-authority badges. Each bead type (epic, task, bug, story, etc.) has a closure-authority label from Spec 116 — the menubar shows this label next to the workpoint title.

**8.2.** The peek's "checkpoint" action uses Spec 119's receipt flow (see §3) instead of the current single-toast.

## 9. Existing surface (121.6) — FirstRunWizard upgrade

**spec_refs:** Spec 117 §3, §4

**9.1.** `FirstRunWizard` becomes a 3-step guided onboarding:
   - step 1: connect (paste daemon URL, hit `/v1/health`)
   - step 2: scope (select or create a project — reads `/v1/project/identity?project_root=<guess>` and lets the operator confirm)
   - step 3: seed (shows recent issues from `/v1/state/dump` so the operator sees context before connecting)

**9.2.** Per Spec 117 §5, recall is a 4th tab `Recall` showing the last 20 user actions in this menubar session.

## 10. Existing surface (121.7) — ProofPeek refactor

**spec_refs:** Spec 119

**10.1.** `ProofPeek` shows the most recent receipt (§3) plus evidence peek plus the 5 P2 cards I added in 2026-07-07 (predictions, metacog, snapshots, lineage, evaluations).

**10.2.** A `Rehydrate` button on each evidence card uses the receipt's `rehydratable` URL to refresh from the daemon. This replaces the current "show evidence" inline.

## 11. New daemon endpoints required (cross-spec dependencies)

This spec requires 3 new daemon endpoints. The menubar is the consumer; the daemon must implement them. These are not 121 deliverables — they're separate work items filed under the relevant spec.

**11.1. `GET /v1/license/tier`** (Spec 118) — returns `{ tier, features_enabled, features_locked, upgrade_url }`. Filed as `focusa-spec118-license-tier-endpoint`.

**11.2. `GET /v1/topology/bind`** (Spec 115) — returns `{ bind, daemon_host, tailscale_ping_ms, last_ping_at }`. Filed as `focusa-spec115-bind-endpoint`.

**11.3. `GET /v1/receipt/{receipt_id}`** (Spec 119) — returns the full receipt object. Filed as `focusa-spec119-receipt-read-endpoint`.

These endpoints are 121 dependencies, not 121 deliverables. They block the new surfaces above until the daemon implements them.

## 12. Hard design laws

**12.1.** No code until spec approved.
**12.2.** No new component without a `spec_refs` line that cites one of Spec 103–120.
**12.3.** No "any" outside the legacy escape hatch. The escape hatch itself must be tracked as a 121 bead.
**12.4.** No two-track envelope handling. The single `normalize()` is the only path.
**12.5.** No toast-as-source-of-truth. Toast is feedback; pane is persistence.
**12.6.** No force-push on shared branches. The 2026-07-07 incident taught us: prefer clean pull+rebase+push; if local diverged, cherry-pick to a new commit; do not overwrite remote history.
**12.7.** No file under apps/menubar/ is edited by a 121 deliverable without first confirming no other agent is working on it.
**12.8.** The mac Tauri shell is out of scope for 121. It picks up 121 once the SvelteKit side is done.

## 13. Implementation phases (when approved)

**Phase 0 — Spec sign-off.** Operator signs off this spec. The Call Stack Design (`019f3fae-...`) is finalized with all the surfaces above as explicit handler chains.

**Phase 1 — Foundation.**
- 121.3 RuntimeView refactor (typed)
- 121.1 envelope normalization (single track)
- 121.2 license badge
- 121.2 cloud indicator

**Phase 2 — Receipts.**
- 121.0 ReceiptsPane
- 121.5 WorkpointPeek closure-authority badges
- 121.7 ProofPeek rehydrate button

**Phase 3 — Onboard + recall.**
- 121.6 FirstRunWizard upgrade
- 121.7 Recall tab

**Phase 4 — Tauri shell pickup.**
- mac menubar Tauri adapter consumes the same SvelteKit output
- Linux/Windows build support

## 14. Open operator questions

1. Is the LicenseDialog tier-bumping flow in scope, or does it just deep-link to a web URL?
2. Is the Recall tab a menubar concern or a PWA concern (Spec 117 §3 keeps it ambiguous)?
3. Should the menubar's "operator steering wins" rule (per focusa project_context authority) override the daemon's degraded=true state, or vice versa?
4. Should the e2e tests be moved from `apps/menubar/tests/` to a top-level `tests/` directory once the menubar is full SvelteKit, or stay co-located with the menubar?

## 15. Diff against prior drafts

This is the first draft. No prior diffs. The document is iterable — operator-revised versions will appear as `121-...-v2.md` etc., with explicit diff sections at the bottom of each.

---

**Reminder:** Specs are NOT final until operator says so. This spec is the first of potentially many iterations. The 2026-07-07 commit-history lesson applies here too: rebase, don't overwrite.
