#!/usr/bin/env python3
"""Protect the existing #307 entitlement at the installer boundary."""
from pathlib import Path

root = Path(__file__).resolve().parents[1]
source = (root / 'crates/focusa-cli/src/commands/install.rs').read_text()
phase = source.split('async fn phase_license(', 1)[1].split('\nfn resolve_installer_entitlement(', 1)[0]
resolver = 'focusa_license::developer_full_projection()'
assert phase.count(resolver) == 1, 'reuse the one canonical developer resolver'
developer = phase.index(resolver)
for boundary in ('std::env::var_os("HOME")', 'resolve_installer_entitlement(', 'begin_legacy_license_migration(', 'authorize_installer_activation_flow(', 'acquire_installer_entitlement('):
    assert developer < phase.index(boundary), f'developer access must precede {boundary}'
assert 'return Ok(format!("developer_full_{}", developer.authority_source));' in phase
assert 'E_AUTHORITY_EXISTING_UNUSABLE' in phase
assert 'E_AUTHORITY_PAID_MIGRATION_REQUIRED' in phase
assert 'E_AUTHORITY_RAW_KEY_FORBIDDEN' in phase
assert 'E_AUTHORITY_LEASE_UNUSABLE' in phase
library = (root / 'crates/focusa-license/src/lib.rs').read_text()
assert 'developer_full_projection_from(developer_origin::developer_origin_source())' in library
origin = (root / 'crates/focusa-license/src/developer_origin.rs').read_text()
assert 'fn probe_disabled() -> bool' in origin
assert 'DEFAULT_TAILNET_SUFFIX' in origin
print('PASS: installer reuses verified developer origin before customer state/activation; ordinary signed authority checks remain')
