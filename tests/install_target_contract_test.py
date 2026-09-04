#!/usr/bin/env python3
"""Regression coverage for the shared Linux installer static contract."""

from __future__ import annotations

from pathlib import Path

from install_target_contract import assert_linux_install_target_contract

ROOT = Path(__file__).resolve().parents[1]


def main() -> None:
    source = (ROOT / "crates/focusa-cli/src/commands/install.rs").read_text(encoding="utf-8")
    assert_linux_install_target_contract(source)

    for missing in (
        'cfg!(target_arch = "aarch64")',
        '"aarch64-unknown-linux-gnu".to_string()',
        '"x86_64-unknown-linux-musl".to_string()',
    ):
        weakened = source.replace(missing, "removed", 1)
        try:
            assert_linux_install_target_contract(weakened)
        except AssertionError as error:
            assert "installer Linux target contract missing" in str(error)
        else:
            raise AssertionError(f"weakened installer contract passed without {missing}")

    print("installer Linux target static contract: PASS")


if __name__ == "__main__":
    main()
