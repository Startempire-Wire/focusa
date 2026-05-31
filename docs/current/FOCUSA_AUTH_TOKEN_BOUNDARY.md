# Focusa Auth Token Boundary

Status: current local-first auth boundary and Pi startup-noise suppression rationale.

## Short version

Normal local Pi sessions do **not** need `FOCUSA_AUTH_TOKEN`.

Focusa uses two different token names for two different jobs:

| Name | Owner | Purpose | Required when |
| --- | --- | --- | --- |
| `FOCUSA_AUTH_TOKEN` | `focusa-daemon` | Server-side bearer token that enables/enforces API auth. | The daemon binds outside loopback, or an operator intentionally enables daemon auth. |
| `FOCUSA_TOKEN` | Pi extension/client | Client-side bearer token sent to an auth-enabled Focusa daemon. | A Pi client talks to a daemon that has `FOCUSA_AUTH_TOKEN` enabled. |

Loopback-only Focusa (`127.0.0.1:8787` or `[::1]:8787`) remains local-first and can run without either token.

## Why `FOCUSA_AUTH_TOKEN` exists

Focusa exposes HTTP mutation routes for Workpoints, Trajectory, Focus State, metacognition, predictions, ECS, and work-loop controls. If the daemon is reachable beyond loopback without auth, another local/network user could read or mutate cognitive state. The daemon therefore fails closed when `FOCUSA_BIND` is non-loopback and no enforced `FOCUSA_AUTH_TOKEN` is present.

This is a deployment/API safety boundary only. It is **not** a product license key, plan entitlement, billing token, seat-control primitive, commercial-license term, or pricing mechanism.

## Why Pi startup should stay quiet

The Focusa Pi extension has two separate integration modes:

1. **Default bridge mode** — tools, Workpoint/Trajectory injection, evidence links, compaction support. This is the normal mode and does not require a proxy provider or token for loopback Focusa.
2. **Optional proxy-provider mode** — registers a selectable `focusa` model provider that sends model traffic through the Focusa proxy API.

Proxy-provider mode is now explicit opt-in only. Auto-registering it during every Pi startup caused confusing auth-token noise even when the operator only wanted default bridge mode. The extension should not ask for, warn about, or imply `FOCUSA_AUTH_TOKEN` during ordinary loopback sessions.

## Enabling proxy-provider mode intentionally

Use proxy-provider mode only when you intend to select the `focusa` model provider in Pi.

Example project/user Pi settings:

```json
{
  "focusaPiBridge": {
    "registerProxyProvider": true,
    "focusaToken": "use-secret-storage-or-env-instead"
  }
}
```

Preferred env form:

```bash
export FOCUSA_PI_REGISTER_PROVIDER=true
export FOCUSA_TOKEN="same-value-as-daemon-token"
```

If the daemon is exposed beyond loopback, set the daemon token separately in the daemon environment:

```bash
export FOCUSA_AUTH_TOKEN="same-secret-value"
```

## Suppression rule

The Pi extension must not register the `focusa` proxy provider unless both conditions are true:

1. `registerProxyProvider` / `FOCUSA_PI_REGISTER_PROVIDER` is true.
2. `focusaToken` / `FOCUSA_TOKEN` is non-empty.

This keeps default local sessions silent while preserving an explicit authenticated proxy path for operators who request it.
