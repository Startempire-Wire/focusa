# Spec 185 — Screenshot Evidence Share Settings Authority (Menubar-Connected Preferences)

**Status:** IMPLEMENTED, 2026-09-09. Canonical authority for Evidence Share presentation preferences consumed by the Focusa menubar (PR #441, merged — `apps/menubar/src/lib/components/Settings.svelte`, "EVIDENCE SHARING" section).
**Implementation owner:** Focusa daemon (`crates/focusa-api/src/routes/screenshot_settings.rs`).
**Parent feature authority:** WPUIAI/uiai-engine#126 — Screenshot Evidence Share Packets. Focusa issues #438/#440 (menubar settings portion) and #441 are satisfied by this authority plus the merged menubar consumer; the Chrome-extension (#437/#439) and FocusCanvas object-model portions remain deferred until those surfaces are scheduled.

---

## 0. One-line definition

> The Focusa daemon is the canonical authority for Screenshot Evidence Share presentation settings; the menubar consumes them over one versioned endpoint with optimistic-concurrency writes and stores nothing canonical locally.

---

## 1. Endpoint contract

Base: the Focusa daemon (`http://127.0.0.1:8787` for the local menubar). Auth: the standard daemon auth layer (local-first loopback default; device-pairing bearer tokens supported with their bound scopes).

| Method | Path | Purpose |
|---|---|---|
| GET | `/api/screenshot/settings` | Read the current effective settings. |
| PUT | `/api/screenshot/settings` | Write settings with optimistic concurrency. |

### 1.1 Shapes (field names are pinned by a consumer contract test)

GET 200:

```json
{
  "revision": 7,
  "values": {
    "enablement": { "auto_screenshot": false },
    "image": { "quality": 80 },
    "presentation": { "theme": "system" }
  }
}
```

PUT request body:

```json
{
  "expected_revision": 7,
  "values": {
    "enablement": { "auto_screenshot": true },
    "image": { "quality": 95 },
    "presentation": { "theme": "dark" }
  }
}
```

PUT responses:

- `200` — the same shape as GET with the incremented revision.
- `409` — `{ "error": "...", "current_revision": <u64> }` when `expected_revision` does not match the current revision. The client re-reads and reapplies.
- `400` — `{ "error": "..." }` for malformed values: `quality` outside `40..=100`, `theme` outside `system|light|dark`, or a malformed body.

### 1.2 Defaults

`auto_screenshot = false` (evidence capture is strictly opt-in), `quality = 80`, `theme = "system"`, initial `revision = 1`.

---

## 2. Storage and consistency

- One JSON document per daemon: `<data_dir>/screenshot-settings.json`, written atomically (temp file + rename) so a crash can never leave a torn state.
- A single writer lock serializes read-modify-write; the revision increments by one per accepted write.
- Consumers never store canonical values: the menubar keeps only in-memory state and the pairing/connection identity. Re-reading GET is always authoritative.

---

## 3. Consumer binding (menubar)

`apps/menubar/src/lib/components/Settings.svelte`:

- `GET` populates the section; a failure renders the explicit `unavailable` state with the error surfaced (never a silent empty form).
- `PUT` sends `expected_revision` from the loaded state (CAS); on `409` the client surfaces the conflict error and refreshes.
- Controls: presentation theme (`system|light|dark`), image quality slider (`40..100`), auto-create screenshot Evidence Share Packets toggle.

A unit test in `screenshot_settings.rs` includes the merged menubar source and fails if any pinned field name drifts — the client-consumer contract cannot silently rot.

---

## 4. Governance

- The settings authority is the daemon (the operator's governance plane). It does not widen any scope: it stores three presentation preferences and nothing else — no secrets, no customer data, no credentials.
- Automatic suspension/revocation policy is untouched: these preferences never gate license truth.
- Change path: contract change here → daemon release via the canonical release pipeline → consumers consume on next refresh. No hand-edited state.

---

## 5. Acceptance (proved by tests in the route module)

1. GET on a fresh daemon returns the defaults with `revision = 1`.
2. PUT with the current revision updates values, bumps the revision, and persists across reload.
3. PUT with a stale `expected_revision` fails `409` and reports `current_revision`.
4. PUT with out-of-range quality or unknown theme fails `400` and persists nothing.
5. The merged menubar client binds exactly the fields this contract serves.
