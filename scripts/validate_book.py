#!/usr/bin/env python3
from pathlib import Path
import json,re,sys

ROOT=Path(__file__).resolve().parents[1]
errors=[]

summary=(ROOT/"SUMMARY.md").read_text()
for target in re.findall(r'\]\(([^)]+\.md)',summary):
    if not (ROOT/target).exists():
        errors.append(f"missing-summary-target:{target}")

for p in [
    ROOT/"PUBLIC_STATE/latest.json",
    ROOT/"PUBLIC_STATE/2026-09-28.json",
    ROOT/"SCHEMAS/public-cycle-v1.schema.json",
]:
    try:
        json.loads(p.read_text())
    except Exception as e:
        errors.append(f"json:{p.relative_to(ROOT)}:{e}")

state=json.loads((ROOT/"PUBLIC_STATE/latest.json").read_text())
ids=[b["being_id"] for b in state["beings"]]

if len(ids)!=34:
    errors.append(f"being-count:{len(ids)}")
if len(ids)!=len(set(ids)):
    errors.append("duplicate-being-id")

for bid in ids:
    pages=list((ROOT/"BOOKS/BOOK-II-BEINGS").glob(f"*/{bid}.md"))
    if len(pages)!=1:
        errors.append(f"being-page:{bid}:{len(pages)}")
        continue
    page=pages[0]
    asset=ROOT/f".gitbook/assets/beings/{bid}.webp"
    cycle=ROOT/f"CYCLES/2026/09/28/{bid}.md"
    if not asset.exists():
        errors.append(f"missing-portrait:{bid}")
    if not cycle.exists():
        errors.append(f"missing-cycle:{bid}")
    expected=f".gitbook/assets/beings/{bid}.webp"
    if expected not in page.read_text():
        errors.append(f"portrait-not-wired:{bid}")

if not (ROOT/".gitbook/assets/aum-sigil.webp").exists():
    errors.append("missing-aum-sigil")
if ".gitbook/assets/aum-sigil.webp" not in (ROOT/"README.md").read_text():
    errors.append("aum-not-wired-home")

if errors:
    print("\n".join(errors))
    sys.exit(1)

print(f"PASS beings={len(ids)} portraits={len(ids)} cycles={len(ids)} aum=ok summary=ok json=ok")
