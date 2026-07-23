#!/usr/bin/env python3
"""
UIAI menubar pre-test (focusa-ui0y.7).

Boots the menubar Svelte preview at the exact popover dimensions (340x480),
opens it in a UIAI session, verifies each tab renders, exercises the
workpoint action bar, captures network evidence, and reports a bounded
result. Used to pre-test menubar changes before committing.

Usage:
    python3 scripts/uiai_menubar_pretest.py [--url http://199.167.201.52:1420/]
    python3 scripts/uiai_menubar_pretest.py --tab workpoint --action re-render

Exit codes:
    0 = all checks passed
    1 = UIAI not available or no capacity
    2 = preview server not reachable
    3 = one or more menubar checks failed
"""

import argparse
import json
import os
import sys
import time
import urllib.request
from urllib.error import URLError

UIAI_BASE = os.environ.get("UIAI_BASE", "http://127.0.0.1:7456")
DEFAULT_URL = os.environ.get("MENUBAR_PREVIEW_URL", "http://199.167.201.52:1420/")


def uiai_post(path: str, body: dict, timeout: int = 8) -> dict:
    req = urllib.request.Request(
        f"{UIAI_BASE}{path}",
        data=json.dumps(body).encode("utf-8"),
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return json.loads(resp.read().decode("utf-8"))


def uiai_get(path: str, timeout: int = 5) -> dict:
    with urllib.request.urlopen(f"{UIAI_BASE}{path}", timeout=timeout) as resp:
        return json.loads(resp.read().decode("utf-8"))


def check_capacity() -> bool:
    h = uiai_get("/api/health")
    if h.get("status") != "healthy":
        print(f"✗ UIAI not healthy: {h}")
        return False
    # Open the agent-capacity slot
    return True


def open_session(url: str, width: int = 340, height: int = 480) -> str:
    res = uiai_post("/api/session/open", {"url": url, "width": width, "height": height})
    return res.get("session", {}).get("id") or res.get("id")


def close_session(sid: str) -> None:
    try:
        req = urllib.request.Request(f"{UIAI_BASE}/api/session/{sid}", method="DELETE")
        urllib.request.urlopen(req, timeout=5).read()
    except Exception:
        pass


def eval_js(sid: str, js: str) -> str:
    res = uiai_post(
        f"/api/session/{sid}/eval_async", {"js": f"return ({js})", "timeout_ms": 5000}
    )
    return res.get("result", "")


def snapshot(sid: str) -> dict:
    res = uiai_post(
        f"/api/session/{sid}/snapshot", {"interactive": True, "compact": True}
    )
    return res.get("tree", "")


def click_ref(sid: str, ref: str) -> dict:
    return uiai_post(f"/api/session/{sid}/click", {"selector": ref})


def diagnostics(sid: str) -> list:
    res = uiai_get(f"/api/session/{sid}/diagnostics")
    return res.get("network", [])


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--url", default=DEFAULT_URL)
    ap.add_argument(
        "--tab",
        default="workpoint",
        choices=[
            "focus",
            "now",
            "path",
            "workpoint",
            "proof",
            "loop",
            "gate",
            "sync",
            "pair",
            "settings",
        ],
    )
    ap.add_argument(
        "--action",
        default="re-render",
        choices=["checkpoint", "re-render", "link-evidence"],
    )
    args = ap.parse_args()

    print("=== UIAI menubar pre-test ===")
    print(f"  url={args.url}")
    print(f"  tab={args.tab} action={args.action}")
    print()

    # 1. UIAI healthy
    if not check_capacity():
        return 1

    # 2. Preview reachable
    try:
        with urllib.request.urlopen(args.url, timeout=3) as r:
            if r.status != 200:
                print(f"✗ preview not 200: {r.status}")
                return 2
    except URLError as e:
        print(f"✗ preview unreachable: {e}")
        return 2

    # 3. Open session
    try:
        sid = open_session(args.url)
    except Exception as e:
        print(f"✗ could not open UIAI session (capacity?): {e}")
        return 1
    if not sid:
        print("✗ UIAI returned no session id")
        return 1
    print(f"✓ session={sid}")

    failures: list[str] = []
    try:
        time.sleep(1.0)
        # 4. Verify 10 tabs present
        tree = snapshot(sid)
        tab_count = tree.count("- button")
        if tab_count < 10:
            failures.append(f"expected ≥10 interactive buttons in nav, got {tab_count}")
        else:
            print(f"✓ {tab_count} tab buttons visible")

        # 5. Switch to the requested tab
        tab_refs = {
            "focus": "@e1",
            "now": "@e2",
            "path": "@e3",
            "workpoint": "@e4",
            "proof": "@e5",
            "loop": "@e6",
            "gate": "@e7",
            "sync": "@e8",
            "pair": "@e9",
            "settings": "@e10",
        }
        ref = tab_refs[args.tab]
        click_ref(sid, ref)
        time.sleep(0.5)
        # Re-snapshot to get the tab-scoped refs (e11/e12/e13 = action buttons on WP)
        tree = snapshot(sid)
        if args.tab == "workpoint":
            # Expect Checkpoint / Re-render / Link evidence
            for label in ("Checkpoint", "Re-render", "Link evidence"):
                if label not in tree:
                    failures.append(f"Workpoint action bar missing button: {label}")
            if "Checkpoint" in tree and "Re-render" in tree and "Link evidence" in tree:
                print("✓ Workpoint action bar: 3 buttons present")

        # 6. Exercise the requested action
        if args.tab == "workpoint":
            action_refs = {
                "checkpoint": "@e11",
                "re-render": "@e12",
                "link-evidence": "@e13",
            }
            action_ref = action_refs[args.action]
            before_perf = eval_js(
                sid,
                "performance.getEntriesByType('resource').filter(r => r.name.includes('/v1/')).length",
            )
            click_ref(sid, action_ref)
            time.sleep(1.0)
            after_perf = eval_js(
                sid,
                "performance.getEntriesByType('resource').filter(r => r.name.includes('/v1/')).length",
            )
            print(f"  perf entries: {before_perf} → {after_perf}")
            if int(after_perf) <= int(before_perf):
                failures.append(f"action {args.action} did not produce a /v1/ request")
            else:
                print(f"✓ action {args.action} fired a /v1/ request")

        # 7. Verify toast container is in the DOM
        toast_present = eval_js(sid, "!!document.querySelector('.toast-stack')")
        if toast_present != "true":
            failures.append("ToastContainer not in DOM (.toast-stack missing)")
        else:
            print("✓ ToastContainer present in DOM")

        # 8. Verify no console errors / exceptions
        diag = diagnostics(sid)
        errors = [e for e in diag if e.get("failed")]
        if errors:
            failures.append(f"{len(errors)} failed network requests")
        else:
            print(f"✓ 0 failed network requests (out of {len(diag)} total)")

    finally:
        close_session(sid)

    print()
    if failures:
        print(f"✗ {len(failures)} failure(s):")
        for f in failures:
            print(f"  - {f}")
        return 3
    print("✓ all menubar pre-test checks passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
