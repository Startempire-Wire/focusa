#!/usr/bin/env python3
"""Validate redacted, replayable Spec 152E deployed authority inventory."""

import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PATH = ROOT / "docs/contracts/spec152e-deployed-surface-inventory.v1.json"
raw = PATH.read_text(encoding="utf-8")
data = json.loads(raw)

assert len(raw.splitlines()) < 500
assert data["schema"] == "focusa.spec152e.deployed_surface_inventory.v1"
assert data["inventory_id"] == "focusa-vbcqu.20.13.2"
assert data["authority"] == {
    "canonical": "WPUIAI.com EDD",
    "rule": "customer commerce human-license refund and entitlement truth are singular",
    "install_site_target_role": "registered branded facade and bounded proxy",
    "spec158": "excluded",
}
assert all(value is False for value in data["privacy"].values() if isinstance(value, bool))

sites = {site["id"]: site for site in data["deployments"]}
assert set(sites) == {"wpuiai_com", "install_focusa_dev"}
assert sites["wpuiai_com"]["source_control"]["repository_present"] is True
assert re.fullmatch(r"[0-9a-f]{40}", sites["wpuiai_com"]["source_control"]["commit"])
assert sites["wpuiai_com"]["source_control"]["parity"] == "dirty_deployment_not_reconstructable_from_commit_alone"
assert set(sites["wpuiai_com"]["source_control"]["dirty_paths"]) == {
    "includes/class-settings.php",
    "wpuiai-ai-cloud-admin.php",
    "includes/class-focusa-license-production.php",
}
assert sites["install_focusa_dev"]["source_control"]["repository_present"] is False
assert sites["install_focusa_dev"]["source_control"]["parity"] == "deployed_only_no_durable_source_commit"
assert sites["wpuiai_com"]["php_lint"] == {"checked": 7, "passed": 7}
assert sites["install_focusa_dev"]["php_lint"] == {"checked": 5, "passed": 5}
for site in sites.values():
    for digest in site["tree_digests"].values():
        assert re.fullmatch(r"[0-9a-f]{64}", digest)

files = data["files"]
assert len(files) == 16
assert len({row["id"] for row in files}) == len(files)
for row in files:
    assert row["deployment"] in sites
    assert row["owner"] and row["path"] and row["classification"] and row["migration"]
    assert re.fullmatch(r"[0-9a-f]{64}", row["sha256"])
    if "repository_sha256" in row and row["repository_sha256"] is not None:
        assert re.fullmatch(r"[0-9a-f]{64}", row["repository_sha256"])
assert {row["id"] for row in files if row.get("parity") == "diverged"} == {
    "installer.unix",
    "installer.windows",
}
assert {row["id"] for row in files if row.get("parity") == "deployed_only"} == {
    "installer.engine",
    "installer.bundle",
}

wpuiai_tables = data["database"]["wpuiai_com"]
install_tables = data["database"]["install_focusa_dev"]
assert len(wpuiai_tables) == 14
assert len(install_tables) == 2
for row in wpuiai_tables + install_tables:
    assert row["owner"] and row["classification"] and row["migration"]
    assert isinstance(row["rows"], int) and row["rows"] >= 0
    if row["present"]:
        assert re.fullmatch(r"[0-9a-f]{64}", row["schema_sha256"])
    else:
        assert row["schema_sha256"] is None and row["rows"] == 0
required_missing = {row["table"] for row in wpuiai_tables if row["classification"] == "required_missing"}
assert required_missing == {
    "wpuiai_authority_accounts",
    "wpuiai_email_identities",
    "wpuiai_activation_registrations",
    "wpuiai_authority_nodes",
    "wpuiai_authority_leases",
    "wpuiai_authority_outbox",
}
assert {row["table"]: row["rows"] for row in install_tables} == {
    "wpuiai_licenses": 83,
    "wpuiai_license_audit": 393,
}

aggregates = data["bounded_aggregates"]
assert aggregates["wpuiai_edd_licenses"]["total"] == 11
assert aggregates["wpuiai_edd_licenses"]["focusa_live_prefix"] == 4
assert sum(aggregates["wpuiai_edd_licenses"]["by_download_id"].values()) == 11
assert aggregates["download_453"]["title"] == "WPUIAI Pro Lifetime"
assert aggregates["download_453"]["disposition"] == "not_accepted_as_implicit_focusa_mapping"
assert sum(row["count"] for row in aggregates["install_registry"]) == 83
assert {row["source"] for row in aggregates["install_registry"]} == {"api", "stripe"}

routes = data["routes"]
assert len(routes["wpuiai_com"]) == 6
assert len(routes["install_focusa_dev"]) == 11
for deployment, rows in routes.items():
    assert len({row["route"] for row in rows}) == len(rows)
    for row in rows:
        assert row["route"].startswith("/wpuiai-ai-cloud/v1/")
        assert row["owner"] and row["classification"] and row["migration"]
assert "/wpuiai-ai-cloud/v1/focusa/license/issue" in {row["route"] for row in routes["wpuiai_com"]}
assert "/wpuiai-ai-cloud/v1/license/create" in {row["route"] for row in routes["install_focusa_dev"]}
assert "/wpuiai-ai-cloud/v1/stripe-webhook" in {row["route"] for row in routes["install_focusa_dev"]}

assert data["configured_presence_only"]["wpuiai_com"] == {"edd_settings": True}
assert all(data["configured_presence_only"]["install_focusa_dev"].values())
facades = {row["url"]: row for row in data["facade_urls"]}
assert facades["https://install.focusa.dev/focusa"]["status"] == 404
assert facades["https://install.focusa.dev/bundle"]["status"] == 404
assert facades["https://install.focusa.dev/installers/install-focusa.sh"]["status"] == 200
assert all(row["uiai_evidence_ref"].startswith("uiai-diagnostics:session=") for row in facades.values())

bridge = data["bridge_assessment"]
assert bridge["result"] == "split_authority_no_durable_unifying_bridge"
assert bridge["one_way_installer_validation_to_wpuiai"] is True
assert not any(bridge[key] for key in (
    "shared_database",
    "trigger_or_view_bridge",
    "scheduled_reconciliation_bridge",
    "durable_edd_sync",
))
nonconformities = "\n".join(data["known_nonconformities"])
for phrase in (
    "verified mailbox",
    "six required authority tables",
    "direct Stripe",
    "explicit frozen product registry",
    "local Evaluation",
    "convenience /focusa and /bundle URLs return 404",
    "privacy remediation",
):
    assert phrase in nonconformities

assert data["replay"]["mode"] == "read_only_as_account_owner"
assert data["replay"]["php_lint_entrypoints"] == 12
assert data["replay"]["verification"] == "python3 tests/spec152e_deployed_surface_inventory_test.py"
assert set(data["replay"]["forbidden"]) == {
    "option values", "customer rows", "raw email", "raw keys", "secrets", "card data", "production mutation"
}

# Reject accidentally captured customer identity, credential material, or secret-shaped payloads.
assert not re.search(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}", raw)
assert not re.search(r"(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+", raw)
assert not re.search(r"focusa_live_[0-9]+_[0-9a-f]+", raw)
assert "license_key\"" not in raw
assert "option_value" not in raw

print(json.dumps({
    "schema": "focusa.spec152e.deployed_surface_inventory_validation.v1",
    "deployments": len(sites),
    "files": len(files),
    "database_surfaces": len(wpuiai_tables) + len(install_tables),
    "routes": sum(len(rows) for rows in routes.values()),
    "required_missing_tables": len(required_missing),
    "install_registry_rows": sum(row["count"] for row in aggregates["install_registry"]),
    "inventory_sha256": hashlib.sha256(raw.encode()).hexdigest(),
    "result": "passed",
}, sort_keys=True))
