# Focusa Pairing — Revoke + Re-pair Spec (v0.9.35-dev)

**Status:** Canonical for the revoke + re-pair cycle.
**Architecture overview:** `docs/55-focusa-self-host-architecture.md` §7.

---

## 1. Purpose

Defines how to revoke a paired device, re-pair after revoke, support multiple devices per host, and handle token expiry. The PairingStore is the durable backing for all of this; the JSONL ledger is the audit trail.

## 2. Revoke

### 2.1 CLI

```
focusa device pair-list                                   # list all paired devices
focusa device pair-list --host <hostname>                 # filter by host
focusa device pair-revoke <device_id>                     # revoke by device_id
focusa device pair-revoke <device_id> --reason "lost laptop"
```

### 2.2 API

```
POST /v1/device/pair/revoke
Content-Type: application/json

{
  "device_id": "019f0794-de58-7752-94f9-70664a18d776",
  "host": "focusa-vps",
  "reason": "lost laptop"           # optional, stored in ledger
}
```

Returns `200` with the appended ledger entry. Idempotent: revoking an already-revoked device returns `200` with the original ledger entry (no duplicate append).

### 2.3 Effects

- JSONL ledger gets a new entry with `revoked=true` for the device_id.
- The in-memory token cache evicts the device.
- PairingStore SQLite row updates `revoked_at` column.
- Mac, on next API call with the revoked token, receives `401 pairing_revoked`.
- FirstRunWizard (Mac side) detects `401` and flips to `welcome` step.

### 2.4 Idempotency

Revoking the same device twice is safe. The ledger is append-only; the second revoke call finds the existing `revoked=true` entry and returns it without appending a duplicate.

## 3. Re-pair

### 3.1 Operator flow

After revoke (or after token expiry), the operator re-runs the wizard:

```
$ focusa pairing wizard
```

A new room is created with a fresh `room_id`. The wizard prints a new terminal QR. The operator scans + approves with the phone. The Mac, on relaunch or after `401`, auto-enters the wizard and joins the new room.

### 3.2 Mac flow

When the Mac app receives a `401 pairing_revoked` or `401 token_expired`:

1. Clears the local Keychain entry for the revoked/expired device.
2. Reads `WizardState` from SQLite (created on first launch).
3. If state is `connected`, transitions to `welcome`.
4. FirstRunWizard shows step 1 of the wizard.
5. Mac discovers VPS via Tailscale MagicDNS (or Bonjour).
6. Mac polls for active rooms via `GET /v1/connect/rooms?status=waiting_for_phone` (or the room_id it has from prior session).
7. Joins the new room.
8. Status flips to `mac_seen` → `completed` once phone approves.
9. New token stored in Keychain. Wizard flips to `connected`.

### 3.3 What about the old device_id?

The old device_id is replaced in the PairingStore by the new one. The ledger keeps the old revocation entry for audit. PairingPanel reflects only the active (non-revoked) device.

## 4. Multi-device pairing

### 4.1 Per host

Each paired device is identified by `device_id` (UUIDv7). One host can have N paired devices. PairingPanel lists them all.

### 4.2 Pairing multiple devices

Re-running `focusa pairing wizard` on the VPS creates a new room each time. Each room results in one paired device. There is no "shared room" — each device gets its own.

```
$ focusa pairing wizard    # pairs mac-A
$ focusa pairing wizard    # pairs mac-B
$ focusa pairing wizard    # pairs mac-C
$ focusa device pair-list
device_id                             name                created_at              revoked
019f0794-de58-7752-94f9-70664a18d776  Verious MacBook     2026-06-26T22:34:00Z    false
019f07d3-2633-7a30-a38c-8f43d5a6d312  Workstation         2026-06-27T06:45:00Z    false
019f07d5-1234-7a30-a38c-9abcdef01234  Test Mac            2026-06-27T07:00:00Z    false
```

### 4.3 Revoking one device

Revoking `019f07d5-...` does not affect the other two. The remaining devices continue to function normally.

## 5. Token expiry

### 5.1 TTL

Tokens expire after 30 days. The expiry is recorded in the PairingStore `expires_at` column and returned in the pair-completion response.

### 5.2 Detection

On any API call with an expired token, the daemon returns `401 token_expired`:

```json
{
  "status": "error",
  "error": "token_expired",
  "device_id": "019f0794-de58-7752-94f9-70664a18d776",
  "expired_at": "2026-07-26T22:34:00Z",
  "recovery_hint": "Re-run focusa pairing wizard on the VPS"
}
```

### 5.3 Mac handling

Mac detects `401 token_expired` and:
1. Clears the local Keychain entry.
2. Transitions FirstRunWizard from `connected` to `welcome`.
3. Shows the wizard. Operator re-runs `focusa pairing wizard` and the flow restarts.

There is no silent auto-re-pair. The operator must explicitly run the wizard.

## 6. Revoke + re-pair test cycle

The test cycle runs the full flow N times to verify idempotency and persistence:

```bash
# tests/spec_focusa_ui0y_pairing_revoke_repair_test.sh
#!/usr/bin/env bash
set -euo pipefail

ROUNDS="${ROUNDS:-10}"
DAEMON_URL="${FOCUSA_DAEMON_URL:-http://127.0.0.1:8787}"

for i in $(seq 1 "$ROUNDS"); do
    echo "=== round $i of $ROUNDS ==="

    # 1. Create room via wizard (demo mode auto-approves)
    ROOM_JSON="$(FOCUSA_WIZARD_DEMO=1 focusa pairing create-room)"

    # 2. Verify room exists
    ROOM_ID="$(echo "$ROOM_JSON" | jq -r .room_id)"
    STATUS="$(curl -fsS "$DAEMON_URL/v1/connect/room/$ROOM_ID/status" | jq -r .status)"
    [[ "$STATUS" == "completed" ]] || { echo "FAIL: round $i: expected completed, got $STATUS"; exit 1; }

    # 3. Extract device_id from completed room
    DEVICE_ID="$(curl -fsS "$DAEMON_URL/v1/connect/room/$ROOM_ID/status" | jq -r .device_id)"
    [[ -n "$DEVICE_ID" && "$DEVICE_ID" != "null" ]] || { echo "FAIL: round $i: no device_id"; exit 1; }

    # 4. Revoke
    REVOKE_RESP="$(curl -fsS -X POST "$DAEMON_URL/v1/device/pair/revoke" \
        -H 'content-type: application/json' \
        -d "$(jq -nc --arg d "$DEVICE_ID" '{device_id: $d, host: "test-host", reason: "test cycle"}')")"
    REVOKED="$(echo "$REVOKE_RESP" | jq -r .revoked)"
    [[ "$REVOKED" == "true" ]] || { echo "FAIL: round $i: revoke did not return revoked=true"; exit 1; }

    # 5. Verify idempotency — second revoke returns same ledger entry
    REVOKE_RESP2="$(curl -fsS -X POST "$DAEMON_URL/v1/device/pair/revoke" \
        -H 'content-type: application/json' \
        -d "$(jq -nc --arg d "$DEVICE_ID" '{device_id: $d, host: "test-host"}')")"
    REVOKED2="$(echo "$REVOKE_RESP2" | jq -r .revoked)"
    [[ "$REVOKED2" == "true" ]] || { echo "FAIL: round $i: second revoke not idempotent"; exit 1; }

    # 6. Verify list shows the revoked device
    LIST="$(curl -fsS "$DAEMON_URL/v1/device/pair/list?host=test-host")"
    FOUND="$(echo "$LIST" | jq -r --arg d "$DEVICE_ID" '.devices[] | select(.device_id == $d) | .revoked')"
    [[ "$FOUND" == "true" ]] || { echo "FAIL: round $i: list did not show device_id as revoked"; exit 1; }

    echo "round $i: PASS"
done

echo ""
echo "ALL $ROUNDS rounds passed."
```

This test exercises the full cycle: create room → phone approves → token minted → revoke → idempotent re-revoke → list reflects revoked state. Running it after every daemon change catches regressions.

## 7. What gets persisted

| Data | Storage | Survives daemon restart | Survives OS reboot |
|---|---|---|---|
| Room state | PairingStore SQLite (`connect_sessions` table) + in-memory cache | yes | yes |
| Paired devices | JSONL ledger (append-only) + PairingStore SQLite (`devices` table) | yes | yes |
| Revocations | JSONL ledger (append-only) | yes | yes |
| Token | Mac Keychain (macOS) / Secret Service (Linux) | n/a (Mac-side) | yes |
| Wizard state | PairingStore SQLite (`wizard_state` table) | yes | yes |

## 8. Operator runbook — revoke + re-pair

```
# 1. List devices
$ focusa device pair-list
device_id                             name                created_at              revoked
019f0794-de58-7752-94f9-70664a18d776  Verious MacBook     2026-06-26T22:34:00Z    false
019f07d5-1234-7a30-a38c-9abcdef01234  Test Mac            2026-06-27T07:00:00Z    false

# 2. Revoke the test Mac
$ focusa device pair-revoke 019f07d5-1234-7a30-a38c-9abcdef01234 --reason "decommissioned"
Revoked 019f07d5-1234-7a30-a38c-9abcdef01234 at 2026-06-27T08:00:00Z. Reason: decommissioned.
The device will receive 401 on its next API call.

# 3. Verify
$ focusa device pair-list
device_id                             name                created_at              revoked
019f0794-de58-7752-94f9-70664a18d776  Verious MacBook     2026-06-26T22:34:00Z    false
019f07d5-1234-7a30-a38c-9abcdef01234  Test Mac            2026-06-27T07:00:00Z    true

# 4. Re-pair a new Mac
$ focusa pairing wizard
…
✓  Pairing complete.

# 5. Confirm
$ focusa device pair-list
device_id                             name                created_at              revoked
019f0794-de58-7752-94f9-70664a18d776  Verious MacBook     2026-06-26T22:34:00Z    false
019f07d5-1234-7a30-a38c-9abcdef01234  Test Mac            2026-06-27T07:00:00Z    true
019f07e0-5678-7a30-a38c-fedcba987654  New MacBook         2026-06-27T08:05:00Z    false
```

## 9. Versioning

This spec ships with v0.9.35-dev. The revoke + re-pair flow is unchanged from v0.9.34-dev (the API and CLI exist); this spec formalizes the semantics, idempotency guarantees, multi-device model, token expiry handling, and the automated test cycle.