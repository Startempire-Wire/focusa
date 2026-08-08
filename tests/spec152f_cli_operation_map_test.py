#!/usr/bin/env python3
"""Validate Spec152F CLI operation-map contract for top-level command surfaces."""

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

CLI_MAP_PATH = ROOT / "docs/contracts/spec152f-cli-operation-map.v1.json"
CLI_RECON_PATH = ROOT / "docs/contracts/spec152f-surface-reconciliation/cli.v1.json"
OP_REGISTRY_PATH = ROOT / "docs/contracts/spec135/generated-contract-v1/operation-registry.json"

ALLOWED_COMMAND_CLASS = {
    "canonical",
    "legacy_read",
    "local_display",
    "local_support",
    "local_control",
    "recovery",
    "destructive",
}

CORE_GUARDED_COMMANDS = {
    "Binary": "packaged_installer",
    "Device": "qr_pwa_handoff",
    "Export": "commercial_export",
    "Release": "official_release_bundle",
}

REQUIRED_LOCAL_HELP = {
    "About",
    "Help",
    "Explain",
    "PairingTransport",
    "PairingWizard",
    "Preflight",
    "Stack",
    "Start",
    "Stop",
    "Status",
}

REQUIRED_KEYS = {
    "command",
    "command_path",
    "operation_refs",
    "unmapped_routes",
    "command_class",
    "local_display",
    "recovery",
    "destructive",
    "requires_confirmation",
    "direct_core_call",
    "core_guard",
    "route_reference_count",
    "notes",
    "source",
}


def _load_json(path: Path):
    raw = path.read_text(encoding="utf-8")
    return raw, json.loads(raw)


def test_cli_map_exists_and_shape():
    raw, payload = _load_json(CLI_MAP_PATH)
    assert payload["schema"] == "focusa.spec152f.cli_operation_map.v1"
    assert payload["row_count"] == len(payload["rows"]) == 86
    assert len(payload["rows"]) == 86
    assert len(raw) > 0
    assert len(hashlib.sha256(raw.encode()).hexdigest()) == 64


def test_cli_map_matches_reconciliation_shard():
    _, map_payload = _load_json(CLI_MAP_PATH)
    _, cli_shard = _load_json(CLI_RECON_PATH)

    cli_commands = [row["symbol_or_route"] for row in cli_shard["rows"]]
    mapped = [row["command"] for row in map_payload["rows"]]

    assert sorted(cli_commands) == sorted(mapped)
    assert len(mapped) == 86
    assert map_payload["row_count"] == 86


def test_cli_map_keys_and_classes_are_valid():
    _, map_payload = _load_json(CLI_MAP_PATH)
    _, op_payload = _load_json(OP_REGISTRY_PATH)
    op_ids = {row["operation_id"] for row in op_payload["operations"]}

    seen = set()
    for row in map_payload["rows"]:
        assert set(row.keys()) >= REQUIRED_KEYS
        cmd = row["command"]
        assert cmd not in seen
        seen.add(cmd)

        assert row["command_class"] in ALLOWED_COMMAND_CLASS
        assert isinstance(row["operation_refs"], list)
        assert isinstance(row["unmapped_routes"], list)
        assert isinstance(row["local_display"], bool)
        assert isinstance(row["recovery"], bool)
        assert isinstance(row["destructive"], bool)
        assert isinstance(row["requires_confirmation"], bool)
        assert isinstance(row["direct_core_call"], bool)
        assert isinstance(row["route_reference_count"], int)
        assert row["route_reference_count"] >= 0

        assert all(op in op_ids for op in row["operation_refs"])

        if cmd in REQUIRED_LOCAL_HELP:
            assert row["command_class"] in {"local_display", "local_support", "local_control"}
            assert not row["operation_refs"]
            assert row["requires_confirmation"] is False

        if row["requires_confirmation"]:
            assert row["destructive"] is True
            assert cmd == "Uninstall"

        if cmd in CORE_GUARDED_COMMANDS:
            assert row["direct_core_call"] is True
            assert row["core_guard"] == CORE_GUARDED_COMMANDS[cmd]
        elif row["direct_core_call"]:
            assert row["core_guard"] is None

        if cmd in {"Binary", "Device", "Export", "Release"}:
            assert row["direct_core_call"] is True

        if cmd == "Pairing":
            assert row["command_path"] == "focusa pairing"

        if cmd == "WorkLoop":
            raise AssertionError("WorkLoop is intentionally not represented in top-level CLI mapping")

        if cmd == "Continue":
            assert row["recovery"] is True
            assert row["command_class"] == "recovery"

        if row["command_class"] == "canonical":
            assert row["operation_refs"], f"canonical entry missing operation_refs: {cmd}"


def test_recovery_and_destruction_marked_explicitly():
    _, map_payload = _load_json(CLI_MAP_PATH)
    recovery_commands = {row["command"] for row in map_payload["rows"] if row["recovery"]}
    destructive_commands = {row["command"] for row in map_payload["rows"] if row["destructive"]}

    assert recovery_commands == {"Recover", "Continue"}
    assert destructive_commands == {"Uninstall"}

    for cmd in {"Claim", "ContextCognition", "License", "WorkItem", "Workpoint"}:
        direct_rows = [row for row in map_payload["rows"] if row["command"] == cmd]
        assert direct_rows and direct_rows[0]["direct_core_call"]


def main() -> None:
    test_cli_map_exists_and_shape()
    test_cli_map_matches_reconciliation_shard()
    test_cli_map_keys_and_classes_are_valid()
    test_recovery_and_destruction_marked_explicitly()

    map_payload = json.loads(CLI_MAP_PATH.read_text(encoding="utf-8"))["rows"]
    print(
        json.dumps(
            {
                "schema": "focusa.spec152f.cli_operation_map_validation.v1",
                "result": "passed",
                "rows": len(map_payload),
                "direct_core_rows": sum(row["direct_core_call"] for row in map_payload),
                "local_display_rows": sum(row["local_display"] for row in map_payload),
                "recovery_rows": sum(row["recovery"] for row in map_payload),
            },
            sort_keys=True,
        )
    )


if __name__ == "__main__":
    main()
