#!/usr/bin/env python3
"""Create the canonical DEVINES snow-glow SVG wrapper without altering source pixels."""
from __future__ import annotations
import argparse
import base64
from pathlib import Path

TEMPLATE = """<svg xmlns="http://www.w3.org/2000/svg" width="768" height="768" viewBox="0 0 768 768" role="img" aria-label="{label}">
<defs>
  <filter id="violetGlow" x="-12%" y="-12%" width="124%" height="124%"><feGaussianBlur stdDeviation="6"/></filter>
  <filter id="snowGlow" x="-12%" y="-12%" width="124%" height="124%"><feGaussianBlur stdDeviation="2.6"/></filter>
</defs>
<image href="data:image/webp;base64,{payload}" x="0" y="0" width="768" height="768" preserveAspectRatio="xMidYMid meet"/>
<circle cx="384" cy="384" r="349.5" fill="none" stroke="#8F7AD0" stroke-width="10" opacity=".34" filter="url(#violetGlow)"/>
<circle cx="384" cy="384" r="346.2" fill="none" stroke="#FFFFFF" stroke-width="2.4" opacity=".66" filter="url(#snowGlow)"/>
<circle cx="384" cy="384" r="352.8" fill="none" stroke="#FFFFFF" stroke-width="2.4" opacity=".66" filter="url(#snowGlow)"/>
<circle cx="384" cy="384" r="349.5" fill="none" stroke="#CFAEEE" stroke-width="7" opacity=".96"/>
<circle cx="384" cy="384" r="346.2" fill="none" stroke="#FFFFFF" stroke-width="1.35" opacity=".98"/>
<circle cx="384" cy="384" r="352.8" fill="none" stroke="#FFFFFF" stroke-width="1.35" opacity=".98"/>
</svg>
"""

def main() -> None:
    p = argparse.ArgumentParser()
    p.add_argument("input")
    p.add_argument("output")
    p.add_argument("label")
    a = p.parse_args()
    src = Path(a.input)
    dst = Path(a.output)
    payload = base64.b64encode(src.read_bytes()).decode("ascii")
    dst.parent.mkdir(parents=True, exist_ok=True)
    dst.write_text(TEMPLATE.format(label=a.label.replace('"', "'"), payload=payload), encoding="utf-8")

if __name__ == "__main__":
    main()
