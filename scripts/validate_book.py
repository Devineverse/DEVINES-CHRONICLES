#!/usr/bin/env python3
from pathlib import Path
import json,re,sys

ROOT=Path(__file__).resolve().parents[1]
errors=[]

summary=(ROOT/"SUMMARY.md").read_text()
for target in re.findall(r'\]\(([^)]+\.md)\)', summary):
    if not (ROOT/target).exists():
        errors.append(f"missing:{target}")

for p in [ROOT/"PUBLIC_STATE/latest.json", ROOT/"SCHEMAS/public-cycle-v1.schema.json"]:
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

as_of=state.get("as_of")
try:
    y,m,d=as_of.split("-")
except Exception:
    errors.append(f"invalid-as-of:{as_of}")
    y=m=d=None

if as_of and not (ROOT/f"PUBLIC_STATE/{as_of}.json").exists():
    errors.append(f"missing-dated-state:{as_of}")

for bid in ids:
    if not list((ROOT/"BOOKS/BOOK-II-BEINGS").glob(f"*/{bid}.md")):
        errors.append(f"missing-being-page:{bid}")
    if y and not (ROOT/f"CYCLES/{y}/{m}/{d}/{bid}.md").exists():
        errors.append(f"missing-cycle:{as_of}:{bid}")

asset_root=ROOT/".gitbook/assets"
if not (asset_root/"aum-sigil.webp").exists():
    errors.append("missing-aum-asset")
if not (asset_root/"devines-chronicles-cover.webp").exists():
    errors.append("missing-cover-asset")

portrait_ids={p.stem for p in (asset_root/"beings").glob("*.webp")}
for bid in sorted(set(ids)-portrait_ids):
    errors.append(f"missing-portrait:{bid}")
for bid in sorted(portrait_ids-set(ids)):
    errors.append(f"unexpected-portrait:{bid}")
if len(portrait_ids)!=34:
    errors.append(f"portrait-count:{len(portrait_ids)}")

if (ROOT/".staging/devines-gitbook-assets.tar.gz").exists():
    errors.append("staging-archive-present")

if errors:
    print("\n".join(errors))
    sys.exit(1)

print(
    f"PASS beings={len(ids)} portraits={len(portrait_ids)} "
    f"as_of={as_of} summary_links=ok json=ok cycles=ok assets=ok"
)
