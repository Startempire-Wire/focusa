#!/usr/bin/env python3
from pathlib import Path

source = Path("crates/focusa-license/src/license_migration.rs").read_text()
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
print("Spec152 license migration journal gate: PASS")
