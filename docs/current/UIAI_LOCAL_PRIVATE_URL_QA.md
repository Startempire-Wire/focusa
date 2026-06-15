# UIAI Local Private URL QA Procedure

Use this only for local/dev product QA when UIAI must open loopback or private URLs such as `http://127.0.0.1:1420` or `http://127.0.0.1:8787`.

## When allowed

- Local Focusa/menubar browser QA on the operator-controlled VPS.
- Short-lived validation runs with evidence capture.
- Never as a production default or public browser setting.

## Enable private URL access

```bash
as-user wpuiai 'cd /home/wpuiai/uiai-engine && perl -0pi -e "s/allow_private_urls:\s*false/allow_private_urls: true/" config.yaml && grep -n "allow_private_urls" config.yaml'
systemctl restart uiai-engine
sleep 5
curl -fsS http://127.0.0.1:7456/api/health/browser | jq -r '{status,active_pages,capacity:.agent_pressure.browser.current_capacity.status}'
```

Expected health: `capacity` is `available` and `active_pages` is within the configured limit.

## QA workflow

1. Open only the local endpoint under test with UIAI.
2. Capture bounded proof handles: browser read/snapshot/diagnostics/session id.
3. Close unused sessions when possible:

```bash
# via Pi tool: uiai_browser_close(session_id)
```

4. If sessions are stuck and capacity is exhausted, restart UIAI Engine and verify health again:

```bash
systemctl restart uiai-engine
sleep 5
curl -fsS http://127.0.0.1:7456/api/health/browser | jq -r '{status,active_pages,capacity:.agent_pressure.browser.current_capacity.status}'
```

## Rollback to hardened default

Run this immediately after local/private URL QA completes:

```bash
as-user wpuiai 'cd /home/wpuiai/uiai-engine && perl -0pi -e "s/allow_private_urls:\s*true/allow_private_urls: false/" config.yaml && grep -n "allow_private_urls" config.yaml'
systemctl restart uiai-engine
sleep 5
curl -fsS http://127.0.0.1:7456/api/health/browser | jq -r '{status,active_pages,capacity:.agent_pressure.browser.current_capacity.status}'
```

Expected config line after rollback: `allow_private_urls: false`.

## Evidence checklist

- UIAI session id or diagnostics ref.
- Endpoint(s) tested.
- Health output before/after restart when a restart was required.
- Confirmation that rollback returned `allow_private_urls` to `false`.
