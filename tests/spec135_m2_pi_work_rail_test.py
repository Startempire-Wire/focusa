#!/usr/bin/env python3
import json
from pathlib import Path

R = Path(__file__).resolve().parents[1]


def main():
    widget = (R / "apps/pi-extension/src/work-rail-widget.ts").read_text()
    turns = (R / "apps/pi-extension/src/turns.ts").read_text()
    index = (R / "apps/pi-extension/src/index.ts").read_text()
    page = (R / "apps/menubar/src/routes/+page.svelte").read_text()
    fixture = json.loads(
        (R / "packages/a2ui-renderer/fixtures/work-rail-tui-snapshots.json").read_text()
    )
    for marker in [
        "providerItemId",
        "workpointId",
        "proofCount",
        "nextAction",
        "width < 48",
        "width >= 76",
        "ascii",
        "FOCUSA_ASCII_UI",
    ]:
        assert marker in widget + turns
    assert (
        "renderWorkRailWidget" in turns
        and 'setWidget("focusa"' in turns
        and "if (ctx.hasUI)" in turns
    )
    assert (
        "ctrl+shift+r" in index
        and "Inspect active Work Rail row" in index
        and "ctx.ui.notify" in index
    )
    assert "activeTab === 'mission-canvas'" in page and (
        "on:click" in page or "onclick" in page
    )
    assert [x["name"] for x in fixture["cases"]] == [
        "narrow-ascii",
        "standard-unicode",
        "wide-badges",
    ]
    assert all(x["required_fragments"] for x in fixture["cases"])
    print("Spec 135 M2 Pi Work Rail responsive/keyboard/mouse/ASCII static proof: PASS")


if __name__ == "__main__":
    main()
