#!/usr/bin/env python3
"""Static contract for REL.2 native Windows x64/ARM64 lifecycle proof."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
workflow = (ROOT / ".github/workflows/windows-ota-e2e.yml").read_text()
installer = (ROOT / "crates/focusa-cli/src/commands/install.rs").read_text()
updater = (ROOT / "crates/focusa-cli/src/commands/update.rs").read_text()

required = (
    "runner: windows-latest",
    "runner: windows-11-arm",
    "install_target: windows-x64",
    "install_target: windows-arm64",
    "Windows native dependency preflight (${{ matrix.architecture }})",
    "Windows native OTA (${{ matrix.architecture }})",
    "RuntimeInformation]::OSArchitecture",
    "@earendil-works/pi-coding-agent@0.81.1",
    "native dependency preflight reported missing dependencies",
    "historical_fixture_authority = 'non_authoritative_migration_fixture'",
    "production_entitlement_claimed = $false",
    "dependency_preflight = 'passed_without_dependency_install_or_executable_shim'",
    "clean_install = 'v0.9.116-dev'",
    "apply = 'v0.9.117-dev'",
    "rollback = 'v0.9.116-dev'",
    "reapply = 'v0.9.117-dev'",
    "user_data_preserved = $true",
    "windows-native-ota-${{ matrix.architecture }}.json",
    "actions/upload-artifact@v4",
    "WINDOWS_OTA_E2E=PASS architecture=${{ matrix.architecture }}",
)
for marker in required:
    assert marker in workflow, f"Windows native OTA workflow missing: {marker}"

for prohibited in (
    "Copy-Item $candidate (Join-Path $shimDir 'pi.exe')",
    "Copy-Item $candidate (Join-Path $shimDir 'uiai-engine.exe')",
    "--install-dependencies --assume-yes",
):
    assert prohibited not in workflow, (
        f"false dependency availability retained: {prohibited}"
    )

windows_candidates = installer[
    installer.index("fn find_command") : installer.index("fn is_root")
]
assert windows_candidates.index('format!("{name}.exe")') < windows_candidates.rindex(
    "name.to_string()"
)
assert "extensionless POSIX shims" in windows_candidates
assert "extensionless shim must not win" in installer
assert '("windows", "x86_64") => "x86_64-pc-windows-msvc"' in updater
assert '("windows", "aarch64") => "aarch64-pc-windows-msvc"' in updater
assert 'triple.ends_with("-pc-windows-msvc")' in updater
assert "release_binary_asset_names_cover_native_windows_targets_once" in updater
assert "exact_release_version_normalizes_to_tag_endpoint_identity" in updater
assert "releases/tags/{tag}" in updater
assert (
    'stop_daemon_before_promotion().context("stop promoted daemon before rollback")?'
    in updater
)
assert "fn rename_pi_extension_path" in installer
assert "error.raw_os_error() == Some(5)" in installer
assert "activate verified Pi extension package" in updater
assert "update transaction phase {failed_phase}" in updater
assert '.join("extensions/focusa-runtime/package.json")' in updater
assert "$env:PI_CODING_AGENT_DIR = Join-Path $testHome '.pi\\agent'" in workflow
assert workflow.count("'--latest-version', 'v0.9.117-dev'") == 3
assert workflow.count("runner: windows-11-arm") >= 2
assert workflow.count("native runner architecture mismatch") >= 2
assert "Windows release build (${{ matrix.target }})" in workflow
assert "target: aarch64-pc-windows-msvc" in workflow
print("REL.2 native Windows x64/ARM64 OTA workflow contract: PASS")
