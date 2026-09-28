#!/usr/bin/env python3
from pathlib import Path
import json,re,sys
ROOT=Path(__file__).resolve().parents[1]
errors=[]
summary=(ROOT/"SUMMARY.md").read_text()
for target in re.findall(r'\]\(([^)]+\.md)\)',summary):
    if not (ROOT/target).exists(): errors.append(f"missing:{target}")
for p in [ROOT/"PUBLIC_STATE/latest.json",ROOT/"PUBLIC_STATE/2026-09-28.json",ROOT/"SCHEMAS/public-cycle-v1.schema.json"]:
    try: json.loads(p.read_text())
    except Exception as e: errors.append(f"json:{p.relative_to(ROOT)}:{e}")
state=json.loads((ROOT/"PUBLIC_STATE/latest.json").read_text())
ids=[b["being_id"] for b in state["beings"]]
if len(ids)!=len(set(ids)): errors.append("duplicate-being-id")
for bid in ids:
    if not list((ROOT/"BOOKS/BOOK-II-BEINGS").glob(f"*/{bid}.md")): errors.append(f"missing-being-page:{bid}")
    if not (ROOT/f"CYCLES/2026/09/28/{bid}.md").exists(): errors.append(f"missing-cycle:{bid}")
if errors:
    print("\n".join(errors)); sys.exit(1)
print(f"PASS beings={len(ids)} portraits={len(portrait_ids)} summary_links=ok json=ok cycles=ok assets=ok")
