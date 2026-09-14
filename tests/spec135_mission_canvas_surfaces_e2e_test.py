#!/usr/bin/env python3
"""SPEC135-M3 durable multiplexed Mission Canvas Work Surface lifecycle proof."""

import pathlib
import shutil
import tempfile
import time
import urllib.parse
import spec135_role_profile_e2e_test as h

S = {
    "project_root": "/tmp/focusa-spec135-m3",
    "continuity_id": "focusa-cont-m3",
    "attachment_id": "attachment-m3",
}
LIST = "/v1/mission-canvas/surfaces"
MUTATE = "/v1/mission-canvas/surfaces/mutate"


def listed(base, sid=None, scope=S):
    q = urllib.parse.urlencode({**scope, **({"work_surface_id": sid} if sid else {})})
    status, p = h.call(base, "GET", f"{LIST}?{q}")
    assert status == 200, p
    return p


def mutate(base, action, key, surface=None, **extra):
    for _ in range(30):
        current = listed(base, surface["work_surface_id"] if surface else None)
        body = {
            **S,
            "idempotency_key": key,
            "expected_state_version": current["state_version"],
            "expected_surface_revision": surface["state_revision"] if surface else 0,
            "action": action,
            **extra,
        }
        if surface:
            body["work_surface_id"] = surface["work_surface_id"]
        status, p = h.call(base, "POST", MUTATE, body)
        if status == 200:
            return p
        assert status == 409, p
        time.sleep(0.05)
    raise RuntimeError("surface writer busy")


def main():
    data = pathlib.Path(tempfile.mkdtemp(prefix="focusa-m3-"))
    process = log = None
    try:
        process, log, base = h.start(data)
        a = mutate(
            base,
            "create",
            "m3-create-a",
            work_surface_id="surface-a",
            instance_id="instance-pi",
            session_id="session-a",
            workpoint_id="workpoint-a",
            mission_ref="mission:alpha5",
            title="Implementation",
            surface_kind="pi",
            pane_id="primary",
            tab_index=0,
            pinned=True,
            canonical_state_refs=["workpoint:workpoint-a", "attachment:attachment-m3"],
        )["surface"]
        b = mutate(
            base,
            "create",
            "m3-create-b",
            work_surface_id="surface-b",
            instance_id="instance-uiai",
            session_id="session-b",
            mission_ref="mission:alpha5",
            title="Browser proof",
            surface_kind="uiai",
            pane_id="primary",
            tab_index=1,
            pinned=False,
            canonical_state_refs=["uiai-session:session-b", "attachment:attachment-m3"],
        )["surface"]
        assert len(listed(base)["surfaces"]) == 2
        a = mutate(
            base,
            "arrange",
            "m3-arrange-a",
            a,
            pane_id="left",
            tab_index=0,
            pinned=True,
            unread=False,
        )["surface"]
        b = mutate(
            base,
            "arrange",
            "m3-arrange-b",
            b,
            pane_id="right",
            tab_index=0,
            pinned=False,
            unread=True,
        )["surface"]
        b = mutate(base, "suspend", "m3-suspend-b", b)["surface"]
        assert b["status"] == "suspended" and b["session_id"] == "session-b"
        time.sleep(1.0)
        h.stop(process, log)
        process = log = None
        time.sleep(0.5)
        process, log, base = h.start(data)
        for _ in range(60):
            rows = listed(base)["surfaces"]
            uniq = {r["work_surface_id"] for r in rows}
            if uniq == {"surface-a", "surface-b"}:
                break
            time.sleep(0.1)
            if _ == 29:
                # log diagnostic after 3s if still empty
                print(f"retry {_:02d} got {uniq} rows={rows}")
        uniq = {r["work_surface_id"] for r in rows}
        assert uniq == {"surface-a", "surface-b"}, f"expected 2 distinct surfaces after restart, got {uniq}: {rows}"
        assert any(r == b for r in rows) and any(
            x["work_surface_id"] == "surface-a" and x["pane_id"] == "left" for x in rows
        )
        b = mutate(base, "resume", "m3-resume-b", b)["surface"]
        b = mutate(base, "close_view", "m3-close-view-b", b)["surface"]
        assert b["status"] == "view_closed" and b["session_id"] == "session-b"
        assert listed(base, scope={**S, "attachment_id": "unrelated"})["surfaces"] == []
        print("Spec 135 M3 multiplexed Mission Canvas Work Surfaces E2E: PASS")
    finally:
        if process is not None:
            h.stop(process, log)
        shutil.rmtree(data, ignore_errors=True)


if __name__ == "__main__":
    main()
