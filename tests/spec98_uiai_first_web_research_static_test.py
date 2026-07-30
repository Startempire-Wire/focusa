#!/usr/bin/env python3
"""Static guard: web/browser/research asks must surface UIAI-first routing before generic web tools."""

from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
TURNS = ROOT / "apps/pi-extension/src/turns.ts"
BROWSER_SKILL = ROOT / ".pi/skills/focusa-browser-uiai/SKILL.md"


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
    require(turns, '"uiai_first_web_research"', "turns.ts")
    require(turns, "buildSliceSection(", "turns.ts")

    skill = BROWSER_SKILL.read_text()
    require(skill, "UIAI-first browser research/action", "Focusa browser skill")
    require(skill, "URL or website task", "Focusa browser skill")
    require(skill, "generic web fallback before UIAI health", "Focusa browser skill")
    require(skill, "Required sequence", "Focusa browser skill")

    print("✓ PASS: UIAI-first web/research routing guard ok")


if __name__ == "__main__":
    main()
