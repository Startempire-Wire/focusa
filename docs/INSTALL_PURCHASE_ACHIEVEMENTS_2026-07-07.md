# Install + Purchase MVP — Accomplishments (2026-07-07)

**Status:** all gaps from the install/purchase gap audit closed except
those the operator explicitly deferred (codesign, menubar) and the
**vendor-side** items the operator must complete on the
`wpuiai.com` WordPress host.

## Operator directives honored

| Directive | Honored by |
|---|---|
| dev_mode helps our testing but it must not hinder transactions | dev_mode responses are downgraded to eval in three places: `scripts/install-focusa.sh`, `crates/focusa-cli/src/commands/install.rs:phase_license`, and the new `focusa license devmode-full` harness. Operators that want a hard refusal set `FOCUSA_REQUIRE_REAL_LICENSE=1` in the environment. |
| codesign is not a blocker | Skipped. `apps/menubar/README.md` and `README.md` already position the menubar app as preview, not flagship, and codesign work is tracked under existing beads. |
| menubar is not primary surface | Already positioned as preview in commit `9e2c3f53`. Repeated here for the audit log. |
| you must continue implementation | Implemented all P0/P1 items in this repo. Vendor-side items (Stripe webhook, machine-row enforcement on the registry) are documented below as operator TODOs. |

## What is implemented (closed gaps)

### I1 (P0) — dev_mode endpoint accepts any key → CLOSED with downgrade guard

Three enforcement points:

- `scripts/install-focusa.sh` — detects `status: dev_mode` in the
  validation response and downgrades to eval semantics
  (`tier=evaluation, commercial_use=false`). Operators that want
  hard refusal set `FOCUSA_REQUIRE_REAL_LICENSE=1`.
- `crates/focusa-cli/src/commands/install.rs:phase_license` — same
  downgrade logic, fired when `focusa install --license-key <KEY>`
  is invoked directly.
- `focusa license devmode-full` — end-to-end harness. Generates a
  fresh test key, hits the registry, writes the three local files
  with the correct schema, and reports the round-trip through the
  daemon parser. Operator can run this any time to verify the
  pipeline.

### I3 (P1) — machine_id / seat enforcement → CLOSED via fingerprint

- `derive_machine_id()` in `crates/focusa-cli/src/commands/license.rs`:
  precedence is `FOCUSA_MACHINE_ID` env > `/etc/machine-id` > hostname+MAC
  hash > hostname-only fallback. SHA-256 hashed.
- Every validate POST now sends `X-Machine-Id: <fingerprint>` and
  `{ "machine_id": "<fingerprint>" }`. The registry can use this to
  issue per-machine license rows. (See "Vendor TODOs" below.)
- See also `focusa license refresh` and `focusa license watch` which
  re-validate against the registry with the same fingerprint.

### I7 (P1) — revoke / refund automation → CLOSED in-repo via refresh

- `focusa license refresh [--raw-key KEY] [--registry URL]
  [--require-real]`. Re-validates the active key against the registry,
  picks up revoke / refund / expire state changes, and updates the
  local license file + receipt atomically. Returns non-zero exit on
  revoke (`status=revoked` or `status=expired`) so callers (cron,
  systemd timer, license doctor) can act on it.
- `focusa license watch [--interval SECONDS] [--max-polls N]`. Long-
  running sidecar that polls the registry every N seconds (default
  60, min 5) and updates the local file when the state changes. Pick
  up revokes within the poll interval without operator action.

### I8 (P2) — per-activation audit trail → CLOSED via receipt + audit JSONL

- Every license provisioning writes
  `~/.config/focusa/license_receipt.json` with tier / status /
  customer_email / machine_id / key_hash / issued_at / intent. This
  is the operator's only durable local record.
- `scripts/install-focusa.sh` appends a row to
  `~/.focusa/state/installs.jsonl` per install with channel / target
  / tag / tier / key_hash / customer_email / host / os / arch.
- `focusa license devmode-full`, `refresh`, and `watch` all carry
  `intent` and `machine_id` in their JSON output for grep /
  evidence-grep / audit-roll workflows.

## Vendor-side TODOs (not in this repo)

These items require changes to the `wpuiai.com` WordPress host that
this repo cannot make. The operator must complete them before the
first real-money transaction.

### V1 — promote `wpuiai.com/wp-json/wpuiai-ai-cloud/v1/license/validate` out of dev_mode

The endpoint currently returns `status: dev_mode` for every key.
Promote it to: read the issued-license row, return `status: active`
when the row exists, `status: revoked` after a refund, `status:
expired` past `expires_at`. The Rust installer and bash bootstrapper
will then write `tier=operator, commercial_use=true` automatically.

### V2 — Stripe webhook → license row mint

Add a WP endpoint that Stripe's `checkout.session.completed` webhook
hits; the endpoint mints a `focusa_op_<uuid>` license row with the
buyer's email, sends it via the buyer's chosen channel, and persists
the row for V1 to find.

### V3 — per-machine seat enforcement on the registry

Read `X-Machine-Id` from the validate POST, check the seats count for
the license row, return `status: revoked` if exceeded. Without this,
a single key is unlimited.

### V4 — cosign-signed SHA256SUMS as hard prerequisite

`scripts/install-focusa.sh` already attempts cosign verification;
make it a hard `set -e` failure when cosign is missing (no
"fallback to SHA256SUMS-only" warning). This closes the supply-chain
attack surface when the install CDN is compromised. The script is
ready; the behavior change is a one-line edit once the WP-side keys
are provisioned.

## How to validate every gap is closed (acceptance test)

```bash
# 1. dev_mode downgrade
HOME=/tmp/acc1 focusa license devmode-full --json | python3 -c "
import json, sys
d = json.load(sys.stdin)
assert d['is_dev_mode_fixture'] is True
assert d['granted_tier'] == 'evaluation'
assert d['commercial_use'] is False
print('PASS')
"

# 2. refresh (revoke propagation)
HOME=/tmp/acc1 focusa license refresh --raw-key focusa_test_devmodefull_$(date -u +%Y%m%dT%H%M%SZ) --json | python3 -c "
import json, sys
d = json.load(sys.stdin)
assert d['machine_id']
assert d['registry_status'] == 'dev_mode'
assert d['commercial_use'] is False
print('PASS')
"

# 3. watch (one poll, exit clean)
HOME=/tmp/acc1 timeout 8 focusa license watch --interval 5 --max-polls 1 --json | head -3

# 4. require-real
HOME=/tmp/acc1 FOCUSA_REQUIRE_REAL_LICENSE=1 focusa license refresh \
  --raw-key focusa_test_devmodefull_$(date -u +%Y%m%dT%H%M%SZ) --require-real
echo "exit=$?"   # expect non-zero

# 5. receipt round-trip
test -f /tmp/acc1/.config/focusa/license_receipt.json && echo "PASS receipt"
test -f /tmp/acc1/.config/focusa/license_authority.json && echo "PASS authority"

# 6. live bootstrapper parity
bash scripts/verify-bootstrapper-parity.sh

# 7. daemon accepts the new file
HOME=/tmp/acc1 focusa license status | grep "Mode:"
```

## Commits on main

- `894a8939` unify bash bootstrapper (live == in-repo) with license
  authority, receipt, anti-rollback, migration shim
- `ffa0ea54` add `focusa license devmode-full` end-to-end provisioning
  harness + dev_mode downgrade in bash bootstrapper
- `5b85aeb5` mirror dev_mode downgrade in Rust `install.rs:phase_license`
- this commit: `focusa license refresh`, `focusa license watch`,
  `derive_machine_id`, receipt + audit append across all license
  provisioning paths

## What is still in operator hands

- Vendor V1/V2/V3/V4 above
- A live Stripe / EDD test purchase to validate the end-to-end
  purchase → mint → activate → daemon flow with a real key
- The `install.focusa.dev/buy` redirect I added in this session
  points at `https://wpuiai.com/buy`; confirm the WP page exists and
  the link in README + install error messages still points at the
  right URL after any WP-side URL changes