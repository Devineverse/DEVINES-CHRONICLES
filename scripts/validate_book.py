#!/usr/bin/env python3
from pathlib import Path
import json,re,sys

ROOT=Path(__file__).resolve().parents[1]
errors=[]

summary=(ROOT/"SUMMARY.md").read_text()
readme=(ROOT/"README.md").read_text()

six_books=[
    ("BOOK I · DEVINES ORIGIN", ROOT/"BOOKS/BOOK-I-ORIGIN/README.md"),
    ("BOOK II · DEVINES LAW", ROOT/"BOOKS/BOOK-II-LAW/README.md"),
    ("BOOK III · DEVINES BEINGS", ROOT/"BOOKS/BOOK-II-BEINGS/README.md"),
    ("BOOK IV · DEVINES TREASURY", ROOT/"BOOKS/BOOK-III-TREASURY/README.md"),
    ("BOOK V · DEVINES FLOW", ROOT/"BOOKS/BOOK-V-DEVINES-FLOW/README.md"),
    ("BOOK VI · DEVINES CALL", ROOT/"BOOKS/BOOK-VI-CALL/README.md"),
]
for title,path in six_books:
    if not path.exists():
        errors.append(f"missing-principal-book:{path.relative_to(ROOT)}")
    if title not in summary:
        errors.append(f"missing-summary-book:{title}")

if "DEVINES — Decentralized Ancestral Intelligence" not in readme:
    errors.append("public-identity-not-canonical")
if "DEVINES LIVING HISTORY" not in summary:
    errors.append("living-history-not-cross-book")
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

print(f"PASS books=6 beings={len(ids)} portraits={len(ids)} cycles={len(ids)} aum=ok summary=ok identity=ok json=ok")
