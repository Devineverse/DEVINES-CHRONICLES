#!/usr/bin/env python3
"""Verify the canonical DEVINES snow-glow ring in empty or image-inserted phase."""
from __future__ import annotations

import argparse
import base64
import re
from pathlib import Path

REQUIRED = (
    'r="346.2"',
    'r="349.5"',
    'r="352.8"',
    'stroke="#FFFFFF"',
    'stroke="#CFAEEE"',
    'stroke="#8F7AD0"',
    'stroke-width="1.35"',
    'stroke-width="7"',
    'stroke-width="10"',
    'stdDeviation="2.6"',
    'stdDeviation="6"',
)

def main() -> None:
    p = argparse.ArgumentParser()
    p.add_argument("paths", nargs="+")
    p.add_argument("--empty", action="store_true")
    p.add_argument("--scale", type=float, default=None)
    a = p.parse_args()

    if a.empty:
        if len(a.paths) != 1:
            raise SystemExit("empty mode: verify.py SVG --empty")
        svg = Path(a.paths[0])
        source = None
    else:
        if len(a.paths) != 2:
            raise SystemExit("image mode: verify.py SOURCE.webp SVG")
        source = Path(a.paths[0]).read_bytes()
        svg = Path(a.paths[1])

    text = svg.read_text(encoding="utf-8")
    for marker in REQUIRED:
        if marker not in text:
            raise SystemExit(f"missing ring marker: {marker}")

    match = re.search(
        r'<image\s+href="data:image/webp;base64,(.*?)"\s+x="([^"]+)"\s+y="([^"]+)"\s+width="([^"]+)"\s+height="([^"]+)"',
        text,
        flags=re.S,
    )

    if a.empty:
        if match:
            raise SystemExit("empty identity circle contains an embedded image")
        if text.count("<circle") != 6:
            raise SystemExit("empty identity circle must contain exactly six canonical ring circles")
        print("PASS empty=true image=none ring=canonical")
        return

    if not match:
        raise SystemExit("missing embedded WebP identity")

    payload = re.sub(r"\s+", "", match.group(1))
    embedded = base64.b64decode(payload, validate=True)
    if embedded != source:
        raise SystemExit("embedded identity bytes differ from preserved source")

    if a.scale is not None:
        expected_size = 768.0 * a.scale
        expected_offset = (768.0 - expected_size) / 2.0
        x, y, width, height = map(float, match.groups()[1:])
        if (
            abs(x - expected_offset) > 0.01
            or abs(y - expected_offset) > 0.01
            or abs(width - expected_size) > 0.01
            or abs(height - expected_size) > 0.01
        ):
            raise SystemExit("identity fit scale does not match expected geometry")

    print("PASS source_bytes=unchanged ring=canonical fit=verified")

if __name__ == "__main__":
    main()
