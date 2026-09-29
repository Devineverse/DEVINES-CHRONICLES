#!/usr/bin/env python3
"""Create the canonical DEVINES snow-glow SVG wrapper without altering source pixels."""
from __future__ import annotations

import argparse
import base64
import textwrap
from pathlib import Path

TEMPLATE = """<svg xmlns="http://www.w3.org/2000/svg" width="768" height="768" viewBox="0 0 768 768" role="img" aria-label="{label}">
<defs>
  <filter id="violetGlow" x="-12%" y="-12%" width="124%" height="124%">
    <feGaussianBlur stdDeviation="6"/>
  </filter>
  <filter id="snowGlow" x="-12%" y="-12%" width="124%" height="124%">
    <feGaussianBlur stdDeviation="2.6"/>
  </filter>
</defs>
<image href="data:image/webp;base64,{payload}
" x="{offset}" y="{offset}" width="{size}" height="{size}" preserveAspectRatio="xMidYMid meet"/>
<circle cx="384" cy="384" r="349.5" fill="none" stroke="#8F7AD0" stroke-width="10" opacity=".34" filter="url(#violetGlow)"/>
<circle cx="384" cy="384" r="346.2" fill="none" stroke="#FFFFFF" stroke-width="2.4" opacity=".66" filter="url(#snowGlow)"/>
<circle cx="384" cy="384" r="352.8" fill="none" stroke="#FFFFFF" stroke-width="2.4" opacity=".66" filter="url(#snowGlow)"/>
<circle cx="384" cy="384" r="349.5" fill="none" stroke="#CFAEEE" stroke-width="7" opacity=".96"/>
<circle cx="384" cy="384" r="346.2" fill="none" stroke="#FFFFFF" stroke-width="1.35" opacity=".98"/>
<circle cx="384" cy="384" r="352.8" fill="none" stroke="#FFFFFF" stroke-width="1.35" opacity=".98"/>
</svg>
"""

def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("input")
    parser.add_argument("output")
    parser.add_argument("label")
    parser.add_argument(
        "--scale",
        type=float,
        default=1.0,
        help="Centered scale-only fit for identities that already contain an intrinsic circle.",
    )
    args = parser.parse_args()

    if not 0.8 <= args.scale <= 1.25:
        raise SystemExit("--scale must be between 0.8 and 1.25")

    src = Path(args.input)
    dst = Path(args.output)

    raw = base64.b64encode(src.read_bytes()).decode("ascii")
    payload = "\n".join(textwrap.wrap(raw, 76))
    size = 768.0 * args.scale
    offset = (768.0 - size) / 2.0

    def number(value: float) -> str:
        return f"{value:.3f}".rstrip("0").rstrip(".")

    dst.parent.mkdir(parents=True, exist_ok=True)
    dst.write_text(
        TEMPLATE.format(
            label=args.label.replace('"', "'"),
            payload=payload,
            size=number(size),
            offset=number(offset),
        ),
        encoding="utf-8",
    )

if __name__ == "__main__":
    main()
