#!/usr/bin/env python3
"""SPEC135-M6 exact Mission Canvas topology persistence and restart proof."""

import pathlib
import tempfile
import time
import urllib.parse

import spec135_m4_surface_bindings_e2e_test as m4
import spec135_mission_canvas_surfaces_e2e_test as m3
import spec135_role_profile_e2e_test as h

GET_STATE = "/v1/mission-canvas/state"
MUTATE_STATE = "/v1/mission-canvas/state/mutate"
IDENTITY = {
    "project_root": m4.SCOPE["project_root"],
    "continuity_id": m4.SCOPE["continuity_id"],
    "client_instance_id": "client-portable-local",
    "user_id": "local-user",
    "device_id": "device-local",
}


def get_state(base):
    status, payload = h.call(
        base,
        "GET",
        f"{GET_STATE}?{urllib.parse.urlencode(IDENTITY)}",
    )
    return status, payload


def persist(base, key, expected_canvas_revision, **topology):
    for _ in range(30):
        current = m3.listed(base)
        body = {
            **IDENTITY,
            "idempotency_key": key,
            "expected_state_version": current["state_version"],
            "expected_canvas_revision": expected_canvas_revision,
            "session_projection_revision": 7,
            **topology,
        }
        status, payload = h.call(base, "POST", MUTATE_STATE, body)
        if status == 200:
            return payload
        assert status == 409, payload
        time.sleep(0.05)
    raise RuntimeError("Mission Canvas state writer busy")


def main():
    data = pathlib.Path(tempfile.mkdtemp(prefix="focusa-m6-"))
    process = log = None
    try:
        process, log, base = h.start(data)
        status, payload = get_state(base)
        assert status == 404, payload
        assert "refusing to manufacture" in str(payload)

        surface_a = m4.create_surface(base, "m6-a", "m6-create-a", 0)
        surface_b = m4.create_surface(base, "m6-b", "m6-create-b", 1)
        topology = {
            "open_work_surface_ids": ["m6-a", "m6-b"],
            "focused_work_surface_id": "m6-a",
            "secondary_focused_surface_id": "m6-b",
            "split_layout_ref": "split:horizontal:50-50",
            "group_order": ["group:primary", "group:secondary"],
            "aggregate_project_roots": [IDENTITY["project_root"]],
            "aggregate_continuity_ids": [IDENTITY["continuity_id"]],
            "aggregate_surface_kinds": ["pi", "uiai_browser"],
            "aggregate_surface_states": ["active", "suspended"],
            "selected_context_refs": ["context:document:alpha", "evidence:m6"],
            "unread_event_cursor": 42,
        }
        saved = persist(base, "m6-persist-topology", 0, **topology)
        canvas = saved["canvas"]
        assert canvas["state_revision"] == 1
        assert canvas["open_work_surface_ids"] == ["m6-a", "m6-b"]
        assert canvas["selected_context_refs"] == topology["selected_context_refs"]
        replayed = persist(base, "m6-persist-topology", 0, **topology)
        assert replayed["replayed"] is True
        assert replayed["canvas"] == canvas

        suspended = m3.mutate(
            base,
            "suspend",
            "m6-suspend-a",
            surface_a,
        )["surface"]
        closed = m3.mutate(
            base,
            "close_view",
            "m6-close-b",
            surface_b,
        )["surface"]
        assert suspended["status"] == "suspended"
        assert closed["status"] == "view_closed"

        h.stop(process, log)
        process = log = None
        process, log, base = h.start(data)
        status, restored = get_state(base)
        assert status == 200, restored
        assert restored["canvas"] == canvas
        assert {surface["work_surface_id"] for surface in restored["surfaces"]} == {
            "m6-a",
            "m6-b",
        }
        assert "resume_surface:m6-a" in restored["recovery_actions"]
        assert "reopen_view:m6-b" in restored["recovery_actions"]

        current = m3.listed(base)
        invalid = {
            **IDENTITY,
            "idempotency_key": "m6-cross-scope-denial",
            "expected_state_version": current["state_version"],
            "expected_canvas_revision": 1,
            "session_projection_revision": 8,
            "open_work_surface_ids": ["surface-not-in-scope"],
        }
        status, payload = h.call(base, "POST", MUTATE_STATE, invalid)
        assert status == 422, payload
        assert "outside exact project and continuity scope" in str(payload)

        print("Spec 135 M6 exact Mission Canvas persistence and restart E2E: PASS")
    finally:
        if process is not None:
            h.stop(process, log)


if __name__ == "__main__":
    main()
