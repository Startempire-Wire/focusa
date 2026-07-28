#!/usr/bin/env python3
"""SPEC135-M4 exact attachment scope, denial, persistence, and evidence proof."""

import pathlib
import tempfile
import time
import urllib.parse

import spec135_mission_canvas_surfaces_e2e_test as m3
import spec135_role_profile_e2e_test as h

LIST = "/v1/mission-canvas/surface-bindings"
MUTATE = "/v1/mission-canvas/surface-bindings/mutate"
SCOPE = m3.S


def listed(base, surface_id, binding_id=None):
    query = {**SCOPE, "work_surface_id": surface_id}
    if binding_id:
        query["binding_id"] = binding_id
    for attempt in range(20):
        status, payload = h.call(base, "GET", f"{LIST}?{urllib.parse.urlencode(query)}")
        assert status == 200, payload
        if not binding_id or payload.get("bindings") or attempt == 19:
            return payload
        time.sleep(0.05)
    raise AssertionError("unreachable bounded binding read loop")


def mutate(base, action, key, surface_id, binding=None, **extra):
    for _ in range(30):
        current = listed(base, surface_id)
        body = {
            **SCOPE,
            "work_surface_id": surface_id,
            "idempotency_key": key,
            "expected_state_version": current["state_version"],
            "expected_binding_revision": binding["state_revision"] if binding else 0,
            "action": action,
            **extra,
        }
        if binding:
            body["binding_id"] = binding["binding_id"]
        status, payload = h.call(base, "POST", MUTATE, body)
        if status == 200:
            return payload
        assert status == 409, payload
        time.sleep(0.05)
    raise RuntimeError("surface binding writer busy")


def create_surface(base, surface_id, key, tab_index):
    return m3.mutate(
        base,
        "create",
        key,
        work_surface_id=surface_id,
        instance_id=f"instance-{surface_id}",
        session_id=f"session-{surface_id}",
        workpoint_id=f"workpoint-{surface_id}",
        mission_ref="mission:m4",
        title=surface_id,
        surface_kind="pi",
        pane_id="primary",
        tab_index=tab_index,
        pinned=False,
        canonical_state_refs=[f"attachment:{SCOPE['attachment_id']}"],
    )["surface"]


def main():
    data = pathlib.Path(tempfile.mkdtemp(prefix="focusa-m4-"))
    process = log = None
    try:
        process, log, base = h.start(data)
        surface_a = create_surface(base, "surface-a", "m4-surface-a", 0)
        create_surface(base, "surface-b", "m4-surface-b", 1)
        result = mutate(
            base,
            "bind",
            "m4-bind-a",
            surface_a["work_surface_id"],
            binding_kind="evidence",
            target_ref="evidence:m4-a",
            access_mode="read",
        )
        binding = result["binding"]
        assert result["evidence_ref"] and result["receipt_ref"]
        assert listed(base, "surface-a", binding["binding_id"])["bindings"] == [binding]
        assert listed(base, "surface-b")["bindings"] == []

        current = listed(base, "surface-b")
        cross_scope = {
            **SCOPE,
            "work_surface_id": "surface-b",
            "binding_id": binding["binding_id"],
            "idempotency_key": "m4-cross-surface-denial",
            "expected_state_version": current["state_version"],
            "expected_binding_revision": binding["state_revision"],
            "action": "unbind",
        }
        status, _ = h.call(base, "POST", MUTATE, cross_scope)
        assert status in (404, 409, 422)
        assert listed(base, "surface-a", binding["binding_id"])["bindings"] == [binding]

        h.stop(process, log)
        process = log = None
        process, log, base = h.start(data)
        assert listed(base, "surface-a", binding["binding_id"])["bindings"] == [binding]
        unbound = mutate(base, "unbind", "m4-unbind-a", "surface-a", binding)["binding"]
        assert unbound["active"] is False
        print("Spec 135 M4 exact attachment-scoped Work Surface bindings E2E: PASS")
    finally:
        if process is not None:
            h.stop(process, log)


if __name__ == "__main__":
    main()
