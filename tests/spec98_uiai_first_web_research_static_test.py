#!/usr/bin/env python3
"""Static guard: web/browser/research asks must surface UIAI-first routing before generic web tools."""

from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
TURNS = ROOT / "apps/pi-extension/src/turns.ts"
ROOT_AGENTS = Path("/root/AGENTS.md")
VISION_SKILL = Path("/root/.pi/skills/vision/SKILL.md")


def fail(msg: str) -> None:
    print(f"✗ FAIL: {msg}")
    sys.exit(1)


def require(text: str, phrase: str, label: str) -> None:
    if phrase not in text:
        fail(f"{label} missing required phrase: {phrase}")


def main() -> None:
    turns = TURNS.read_text()
    require(turns, "currentAskLooksLikeWebResearch", "turns.ts")
    require(turns, "getUiaiFirstFocusSliceLines", "turns.ts")
    require(turns, "UIAI_FIRST_WEB_RESEARCH: required=true", "turns.ts")
    require(
        turns, "pi_uiai_agent_card → uiai_health → uiai_browser_open/read", "turns.ts"
    )
    require(turns, "web_search/fetch_content only after UIAI unavailable", "turns.ts")
    require(turns, "close unused UIAI sessions before generic web fallback", "turns.ts")
    require(turns, 'buildSliceSection("uiai_first_web_research"', "turns.ts")

    agents = ROOT_AGENTS.read_text()
    require(agents, "UIAI-FIRST WEB/RESEARCH RULE", "/root/AGENTS.md")
    require(agents, "pi_uiai_agent_card` → `uiai_health`", "/root/AGENTS.md")
    require(
        agents,
        "Generic `web_search` / `fetch_content` are fallbacks only",
        "/root/AGENTS.md",
    )

    skill = VISION_SKILL.read_text()
    require(skill, "UIAI-first web/browser/research workflow", "vision skill")
    require(
        skill,
        "For any URL, website, browser, documentation, or web-research task, use UIAI first",
        "vision skill",
    )
    require(skill, "Operator shorthands", "vision skill")

    print("✓ PASS: UIAI-first web/research routing guard ok")


if __name__ == "__main__":
    main()
