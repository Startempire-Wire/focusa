#!/usr/bin/env python3
"""SPEC135-M5 browser context ownership, isolation, sharing, and restart proof."""

import pathlib
import tempfile

import spec135_m4_surface_bindings_e2e_test as m4
import spec135_role_profile_e2e_test as h


def reject_binding(base, surface_id, key, **extra):
    current = m4.listed(base, surface_id)
    body = {
        **m4.SCOPE,
        "work_surface_id": surface_id,
        "idempotency_key": key,
        "expected_state_version": current["state_version"],
        "expected_binding_revision": 0,
        "action": "bind",
        **extra,
    }
    status, payload = h.call(base, "POST", m4.MUTATE, body)
    assert status in (409, 422), payload
    return payload


def bind_session(base, surface_id, suffix):
    return m4.mutate(
        base,
        "bind",
        f"m5-session-{suffix}",
        surface_id,
        binding_kind="session",
        target_ref=f"uiai-session:{suffix}",
        access_mode="invoke",
    )["binding"]


def bind_context(base, surface_id, suffix, context_ref, isolation, sharing):
    return m4.mutate(
        base,
        "bind",
        f"m5-context-{suffix}",
        surface_id,
        binding_kind="browser_context",
        target_ref=context_ref,
        access_mode="invoke",
        browser_isolation_class=isolation,
        authentication_sharing=sharing,
        retention_policy="persistent",
    )["binding"]


def main():
    data = pathlib.Path(tempfile.mkdtemp(prefix="focusa-m5-"))
    process = log = None
    try:
        process, log, base = h.start(data)
        for index, surface_id in enumerate(("m5-a", "m5-b", "m5-c", "m5-d")):
            m4.create_surface(base, surface_id, f"m5-surface-{surface_id}", index)

        reject_binding(
            base,
            "m5-a",
            "m5-context-without-session",
            binding_kind="browser_context",
            target_ref="browser-context:unowned",
            access_mode="invoke",
            browser_isolation_class="isolated_authenticated",
            authentication_sharing="isolated",
            retention_policy="persistent",
        )
        reject_binding(
            base,
            "m5-a",
            "m5-target-without-context",
            binding_kind="browser_target",
            target_ref="browser-target:unowned",
            access_mode="invoke",
        )

        bind_session(base, "m5-a", "a")
        isolated = bind_context(
            base,
            "m5-a",
            "a",
            "browser-context:isolated",
            "isolated_authenticated",
            "isolated",
        )
        assert isolated["browser_isolation_class"] == "isolated_authenticated"
        assert isolated["authentication_sharing"] == "isolated"
        assert isolated["retention_policy"] == "persistent"

        bind_session(base, "m5-b", "b")
        reject_binding(
            base,
            "m5-b",
            "m5-isolated-reuse-denied",
            binding_kind="browser_context",
            target_ref="browser-context:isolated",
            access_mode="invoke",
            browser_isolation_class="isolated_authenticated",
            authentication_sharing="isolated",
            retention_policy="persistent",
        )

        bind_session(base, "m5-c", "c")
        shared_c = bind_context(
            base,
            "m5-c",
            "c",
            "browser-context:shared",
            "shared_authenticated",
            "shared_explicit",
        )
        bind_session(base, "m5-d", "d")
        shared_d = bind_context(
            base,
            "m5-d",
            "d",
            "browser-context:shared",
            "shared_authenticated",
            "shared_explicit",
        )
        assert shared_c["target_ref"] == shared_d["target_ref"]

        target = m4.mutate(
            base,
            "bind",
            "m5-target-a",
            "m5-a",
            binding_kind="browser_target",
            target_ref="browser-target:a",
            access_mode="invoke",
        )["binding"]
        assert target["binding_kind"] == "browser_target"

        h.stop(process, log)
        process = log = None
        process, log, base = h.start(data)
        persisted = m4.listed(base, "m5-a", isolated["binding_id"])["bindings"]
        assert persisted == [isolated]
        print("Spec 135 M5 browser context isolation and UIAI ownership E2E: PASS")
    finally:
        if process is not None:
            h.stop(process, log)


if __name__ == "__main__":
    main()
