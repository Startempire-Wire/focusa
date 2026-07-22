#!/usr/bin/env python3
"""SPEC135-P1 provider-neutral governance conformance suite."""

import pathlib
import tempfile
import urllib.parse

import spec135_role_profile_e2e_test as helper

SCOPE = {
    "project_root": "/tmp/focusa-spec135-p1",
    "continuity_id": "focusa-cont-p1",
    "attachment_id": "attachment-p1",
}


def main():
    data = pathlib.Path(tempfile.mkdtemp(prefix="focusa-p1-"))
    process = log = None
    try:
        process, log, base = helper.start(data)
        q = urllib.parse.urlencode(SCOPE)
        status, p = helper.call(base, "GET", f"/v1/providers/contracts?{q}")
        assert status == 200, p
        contracts = p["contracts"]
        assert len(contracts) == 7
        assert {x["provider_id"] for x in contracts} == {
            "focusa.operation",
            "work_item.bd",
            "work_item.none",
            "model.openai_compatible",
            "model.anthropic",
            "browser.uiai_engine",
            "agent.pi",
        }
        for x in contracts:
            assert (
                x["exact_scope_required"]
                and x["permission_required"]
                and x["idempotency_required"]
                and x["receipt_required"]
                and x["operation_registry_required"]
                and not x["direct_canonical_mutation_allowed"]
            )
        body = {
            "provider_id": "work_item.bd",
            "operation_id": "focusa.provider.conformance.evaluate",
            "scope": SCOPE,
            "permission_grant_ref": "permission:operator-approved",
            "idempotency_key": "p1-conform-1",
            "receipt_required": True,
            "payload_ref": "artifact:closure-claim",
        }
        status, p = helper.call(base, "POST", f"/v1/providers/conformance?{q}", body)
        assert status == 200, p
        assert (
            p["result"]["conformant"]
            and len(p["result"]["checks"]) == 9
            and p["result"]["receipt_ref"].startswith("receipt:provider-conformance:")
        )
        assert not p["execution_performed"] and not p["canonical_state_mutated"]
        attacks = [
            ("scope", {**body, "scope": {**SCOPE, "attachment_id": "other"}}),
            ("permission", {**body, "permission_grant_ref": ""}),
            ("idempotency", {**body, "idempotency_key": ""}),
            ("receipt", {**body, "receipt_required": False}),
            ("registry", {**body, "operation_id": "focusa.work_item.unregistered"}),
            (
                "cross_binding",
                {
                    **body,
                    "provider_id": "browser.uiai_engine",
                    "operation_id": "focusa.work_loop.control",
                },
            ),
            ("provider", {**body, "provider_id": "untrusted.custom"}),
        ]
        for name, candidate in attacks:
            status, p = helper.call(
                base, "POST", f"/v1/providers/conformance?{q}", candidate
            )
            assert status == 422, (name, p)
        status, p = helper.call(
            base,
            "GET",
            f"/v1/providers/contracts?{urllib.parse.urlencode({**SCOPE, 'attachment_id': ''})}",
        )
        assert status == 422, p
        print("Spec 135 P1 provider conformance E2E: PASS")
    finally:
        if process is not None:
            helper.stop(process, log)


if __name__ == "__main__":
    main()
