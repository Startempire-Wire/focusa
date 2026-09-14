#!/usr/bin/env python3
"""Shared static contract for architecture-aware Linux installer assets."""

from __future__ import annotations


def assert_linux_install_target_contract(install_source: str) -> None:
    """Require ARM64 GNU and x64 musl selection in the canonical Rust owner."""

    function_start = install_source.find("fn triple_for(target: InstallTarget) -> String {")
    if function_start < 0:
        raise AssertionError("installer target contract missing triple_for owner")
    linux_start = install_source.find("InstallTarget::Linux => {", function_start)
    darwin_start = install_source.find("InstallTarget::Darwin =>", linux_start)
    if linux_start < 0 or darwin_start < 0:
        raise AssertionError("installer target contract missing bounded Linux match arm")
    linux_arm = install_source[linux_start:darwin_start]

    required = {
        'cfg!(target_arch = "aarch64")': "architecture branch",
        '"aarch64-unknown-linux-gnu".to_string()': "ARM64 GNU release target",
        '"x86_64-unknown-linux-musl".to_string()': "x64 musl release target",
    }
    for source, name in required.items():
        if source not in linux_arm:
            raise AssertionError(f"installer Linux target contract missing {name}: {source}")
