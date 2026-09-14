#!/usr/bin/env python3
from pathlib import Path

source = Path("crates/focusa-license/src/license_migration.rs").read_text()
installer = Path("crates/focusa-cli/src/commands/install.rs").read_text()
for marker in [
    "LegacyLicenseSourceClass",
    "LegacyLicenseInventoryItem",
    "LicenseMigrationJournalEntry",
    "source_digest",
    "previous_entry_hash",
    "compute_entry_hash",
    "IdempotentReplay",
    "file.sync_all()",
    "SoftwareRolledBack",
    "AuthorityRollbackForbidden",
    "candidate.authority_lease_id != previous.authority_lease_id",
]:
    assert marker in source, marker
for forbidden in ["license_key: String", "customer_email", "raw_key", "access_token"]:
    assert forbidden not in source, forbidden
assert source.index("validate_transition") < source.index("options.create(true).append(true)")
for marker in [
    "begin_legacy_license_migration",
    "complete_legacy_license_migration",
    "PaidKeyRecord",
    "EvaluationRecord",
    'config_dir.join("license-migration.jsonl")',
    '"node_identity".into()',
    '"device_pairing".into()',
    '"projects".into()',
    '"workpoints".into()',
    '"evidence".into()',
]:
    assert marker in installer, marker
phase = installer[installer.index("async fn phase_license"):installer.index("struct PendingLegacyMigration")]
assert phase.index("begin_legacy_license_migration") < phase.index("acquire_installer_entitlement")
assert phase.index("resolve_installer_entitlement") < phase.index("complete_legacy_license_migration")
print("Spec152 license migration journal gate: PASS")
