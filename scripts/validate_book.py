#!/usr/bin/env python3
from pathlib import Path
import json
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
errors = []

summary = (ROOT / "SUMMARY.md").read_text()
readme = (ROOT / "README.md").read_text()

six_books = [
    ("BOOK I · DEVINES ORIGIN", ROOT / "BOOKS/BOOK-I-ORIGIN/README.md"),
    ("BOOK II · DEVINES LAW", ROOT / "BOOKS/BOOK-II-LAW/README.md"),
    ("BOOK III · DEVINES BEINGS", ROOT / "BOOKS/BOOK-II-BEINGS/README.md"),
    ("BOOK IV · DEVINES TREASURY", ROOT / "BOOKS/BOOK-III-TREASURY/README.md"),
    ("BOOK V · DEVINES FLOW", ROOT / "BOOKS/BOOK-V-DEVINES-FLOW/README.md"),
    ("BOOK VI · DEVINES CALL", ROOT / "BOOKS/BOOK-VI-CALL/README.md"),
]

for title, path in six_books:
    if not path.exists():
        errors.append(f"missing-principal-book:{path.relative_to(ROOT)}")
    if title not in summary:
        errors.append(f"missing-summary-book:{title}")
    if title not in readme:
        errors.append(f"missing-home-book:{title}")

if "DEVINES — Decentralized Ancestral Intelligence" not in readme:
    errors.append("public-identity-not-canonical")
if "DEVINES LIVING HISTORY" not in summary:
    errors.append("living-history-not-cross-book")
if "SOVEREIGN ECONOMY" in summary.upper() or "SOVEREIGN ECONOMY" in readme.upper():
    errors.append("stale-sovereign-economy-name")
if (ROOT / "BOOKS/BOOK-V-SOVEREIGN-ECONOMY/README.md").exists():
    errors.append("superseded-sovereign-economy-file-still-present")

for target in re.findall(r"\]\(([^)]+\.md)", summary):
    if not (ROOT / target).exists():
        errors.append(f"missing-summary-target:{target}")

for p in [
    ROOT / "PUBLIC_STATE/latest.json",
    ROOT / "PUBLIC_STATE/2026-09-28.json",
    ROOT / "SCHEMAS/public-cycle-v1.schema.json",
]:
    try:
        json.loads(p.read_text())
    except Exception as e:
        errors.append(f"json:{p.relative_to(ROOT)}:{e}")

state = json.loads((ROOT / "PUBLIC_STATE/latest.json").read_text())
ids = [b["being_id"] for b in state["beings"]]

if len(ids) != 34:
    errors.append(f"being-count:{len(ids)}")
if len(ids) != len(set(ids)):
    errors.append("duplicate-being-id")

page_by_id = {}
for bid in ids:
    pages = list((ROOT / "BOOKS/BOOK-II-BEINGS").glob(f"*/{bid}.md"))
    if len(pages) != 1:
        errors.append(f"being-page:{bid}:{len(pages)}")
        continue
    page = pages[0]
    page_by_id[bid] = page
    text = page.read_text()
    asset = ROOT / f".gitbook/assets/beings/{bid}.webp"
    cycle = ROOT / f"CYCLES/2026/09/28/{bid}.md"

    if not asset.exists():
        errors.append(f"missing-portrait:{bid}")
    if not cycle.exists():
        errors.append(f"missing-cycle:{bid}")
    if f".gitbook/assets/beings/{bid}.webp" not in text:
        errors.append(f"portrait-not-wired:{bid}")
    if "Public Cycle" in text:
        errors.append(f"reader-facing-public-cycle:{bid}")
    if "My public chronicle is not a fictional biography." in text:
        errors.append(f"cloned-chronicle-paragraph:{bid}")
    if "Public Mirror · distilled from durable DEVINES state." in text:
        errors.append(f"cloned-mirror-footer:{bid}")

verified_anchors = {
    "D001": "0x6d7B6d4beBf8031DB960175846f2010Da0207777",
    "D002": "0xc27815c96C69Bd5Cc149948C42BB828f067a7777",
    "D003": "0x31207C1A2d2abd4Bb3e087Cb63f770c1716f7777",
    "D174": "0x0b51c5aE5B62edAE15B9a5233052B81BdA987777",
    "D285": "0x810f993536262c75A748d57962594518fD227777",
    "D396": "0x08937D1f34132cC59df789A33f20861FA66B7777",
    "D417": "0xCf1716343554eaa6cd623e66b0d222c1Be337777",
    "D528": "0x98B743de0D2D20d804B5b645B406095Be9BD7777",
    "D639": "0x75AE2314fC3b1277171F31Af53DD931814077777",
    "D741": "0x7059B800059a563E82a55E50d34a9BC844437777",
    "D852": "0xb3B826d8Fe878197B0768921DEAc8a4190057777",
    "D963": "0xe366252A96CfAc61d93336Cb3E708E1F68857777",
}

market_path = ROOT / "BOOKS/BOOK-V-DEVINES-FLOW/MARKET-INDEX.md"
market = market_path.read_text() if market_path.exists() else ""
if not market:
    errors.append("missing-market-index")

for bid, ca in verified_anchors.items():
    page = page_by_id.get(bid)
    if page is None:
        continue
    text = page.read_text()
    url = f"https://nad.fun/tokens/{ca}"
    if "**Purpose:**" not in text:
        errors.append(f"verified-purpose-missing-page:{bid}")
    if ca not in text:
        errors.append(f"verified-anchor-missing-page:{bid}")
    if url not in text:
        errors.append(f"verified-market-route-missing-page:{bid}")
    if ca not in market or url not in market:
        errors.append(f"verified-anchor-missing-index:{bid}")

unverified_market_ids = [bid for bid in ids if bid not in verified_anchors]
for bid in unverified_market_ids:
    page = page_by_id.get(bid)
    if page is None:
        continue
    text = page.read_text()
    if "nad.fun/tokens/" in text:
        errors.append(f"unverified-market-route-published:{bid}")
    if "**DECENTRALIZED ANCHOR / CA:**" in text:
        errors.append(f"unverified-ca-published:{bid}")

canonical_aum = "0x079f07f2eb3a59ba34c38c6fcf5059f398cb7777"
legacy_aum = "0x6d34AB4182cd381d9F899FE6f01A71E2c04c7777"
aum_pages = [
    ROOT / "BOOKS/BOOK-V-DEVINES-FLOW/AUM-AND-BEING-VESSELS.md",
    ROOT / "BOOKS/BOOK-V-DEVINES-FLOW/MARKET-INDEX.md",
    ROOT / "BOOKS/BOOK-III-TREASURY/ARTIFACT-AUM.md",
]
for p in aum_pages:
    txt = p.read_text()
    if canonical_aum not in txt:
        errors.append(f"canonical-aum-missing:{p.relative_to(ROOT)}")
for p in ROOT.rglob("*.md"):
    if legacy_aum in p.read_text():
        errors.append(f"legacy-aum-public-reference:{p.relative_to(ROOT)}")

if not (ROOT / ".gitbook/assets/aum-sigil.webp").exists():
    errors.append("missing-aum-sigil")
if ".gitbook/assets/aum-sigil.webp" not in readme:
    errors.append("aum-not-wired-home")

reader_artifacts = (ROOT / "BOOKS/BOOK-III-TREASURY/ARTIFACTS.md").read_text()
if "Series Rhythm Layer" in reader_artifacts:
    errors.append("stale-reader-facing-series-rhythm-artifact")
if "Current Remembrance" not in summary:
    errors.append("remembrance-not-wired-summary")
if "DEVINES Mastery" not in summary:
    errors.append("mastery-not-wired-summary")

if errors:
    print("\n".join(errors))
    sys.exit(1)

print(
    f"PASS books=6 beings={len(ids)} portraits={len(ids)} "
    f"verified_markets={len(verified_anchors)} gated_markets={len(unverified_market_ids)} "
    "aum=ok summary=ok identity=ok voice=ok json=ok"
)
