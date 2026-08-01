#!/usr/bin/env python3
"""Render deterministic Mission Canvas ANSI snapshots to a PNG contact sheet."""
from __future__ import annotations

import json
import re
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "docs/evidence/spec135-pi-native-reference-renders.v1.json"
OUTPUT = ROOT / "docs/evidence/spec135-pi-native-reference-renders.png"
LIVE_SOURCE = ROOT / "docs/evidence/spec135-pi-native-live-capture.ansi"
LIVE_OUTPUT = ROOT / "docs/evidence/spec135-pi-native-live-capture.png"
ANSI = re.compile(r"\x1b\[([0-9;]*)m")
FONT_PATHS = [
    Path("/System/Library/Fonts/Menlo.ttc"),
    Path("/System/Library/Fonts/Monaco.ttf"),
]


def font(size: int = 14):
    for path in FONT_PATHS:
        if path.exists():
            return ImageFont.truetype(str(path), size=size)
    return ImageFont.load_default()


def render_line(draw: ImageDraw.ImageDraw, line: str, x: int, y: int, cell_w: int, cell_h: int, face) -> None:
    cursor = 0
    fg = (231, 237, 245)
    bg = (8, 13, 20)
    position = 0
    for match in ANSI.finditer(line):
        segment = line[position:match.start()]
        if segment:
            draw.rectangle((x + cursor * cell_w, y, x + (cursor + len(segment)) * cell_w, y + cell_h), fill=bg)
            draw.text((x + cursor * cell_w, y + 1), segment, font=face, fill=fg)
            cursor += len(segment)
        codes = [int(code) for code in match.group(1).split(";") if code]
        if not codes or 0 in codes:
            fg, bg = (231, 237, 245), (8, 13, 20)
        for index, code in enumerate(codes):
            if code == 38 and index + 4 < len(codes) and codes[index + 1] == 2:
                fg = tuple(codes[index + 2:index + 5])
            if code == 48 and index + 4 < len(codes) and codes[index + 1] == 2:
                bg = tuple(codes[index + 2:index + 5])
        position = match.end()
    segment = line[position:]
    if segment:
        draw.rectangle((x + cursor * cell_w, y, x + (cursor + len(segment)) * cell_w, y + cell_h), fill=bg)
        draw.text((x + cursor * cell_w, y + 1), segment, font=face, fill=fg)


def main() -> None:
    packet = json.loads(SOURCE.read_text())
    captures = packet["captures"]
    face = font()
    cell_w, cell_h = 9, 19
    cols = int(packet["width"])
    panel_width = cols * cell_w
    title_h = 32
    gap = 24
    max_lines = max(len(lines) for lines in captures.values())
    panel_height = title_h + max_lines * cell_h
    names = list(captures)
    sheet = Image.new("RGB", (panel_width * 2 + gap * 3, panel_height * 3 + gap * 4), (5, 8, 13))
    draw = ImageDraw.Draw(sheet)
    for ordinal, name in enumerate(names):
        col, row = ordinal % 2, ordinal // 2
        ox = gap + col * (panel_width + gap)
        oy = gap + row * (panel_height + gap)
        draw.text((ox, oy), name.upper(), font=face, fill=(231, 237, 245))
        for line_no, line in enumerate(captures[name]):
            render_line(draw, line, ox, oy + title_h + line_no * cell_h, cell_w, cell_h, face)
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    sheet.save(OUTPUT)
    print(f"Rendered {OUTPUT.relative_to(ROOT)} ({sheet.width}x{sheet.height})")

    if LIVE_SOURCE.exists():
        lines = LIVE_SOURCE.read_text(errors="replace").splitlines()
        live_width = 160 * cell_w
        live = Image.new("RGB", (live_width, max(1, len(lines)) * cell_h), (8, 13, 20))
        live_draw = ImageDraw.Draw(live)
        for line_no, line in enumerate(lines):
            render_line(live_draw, line, 0, line_no * cell_h, cell_w, cell_h, face)
        live.save(LIVE_OUTPUT)
        print(f"Rendered {LIVE_OUTPUT.relative_to(ROOT)} ({live.width}x{live.height})")


if __name__ == "__main__":
    main()
