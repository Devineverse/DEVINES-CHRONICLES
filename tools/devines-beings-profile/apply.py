#!/usr/bin/env python3
"""DEVINES BEINGS PROFILE local application helper.

Usage:
  python3 tools/devines-beings-profile/apply.py <DEVINES_ID> <image-file> [repo-root]

The helper preserves the approved image bytes, wires the Being profile,
updates identity manifests/reference mapping, extends Rust hash validation,
and runs the Chronicle validator. It does not commit or push.
"""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys


def die(msg: str) -> None:
    raise SystemExit(f"DEVINES BEINGS PROFILE: {msg}")


def detect_ext(data: bytes) -> str:
    if data.startswith(b"\xff\xd8\xff"):
        return ".jpg"
    if data.startswith(b"\x89PNG\r\n\x1a\n"):
        return ".png"
    if len(data) >= 12 and data[:4] == b"RIFF" and data[8:12] == b"WEBP":
        return ".webp"
    die("unsupported image bytes; expected JPEG, PNG, or WEBP")


def load_json(path: Path):
    with path.open("r", encoding="utf-8") as f:
        return json.load(f)


def save_json(path: Path, value) -> None:
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")


def update_rust(path: Path, devines_id: str, hero_path: str, sha256: str) -> None:
    text = path.read_text(encoding="utf-8")
    arm_body = f'"{devines_id}" => ("{hero_path}".to_string(), Some("{sha256}")),'

    # Existing direct arm: replace in place while preserving its indentation.
    pattern = re.compile(
        rf'^(?P<indent>[ \t]*)"{re.escape(devines_id)}"\s*=>\s*\([^\n]+\),[ \t]*$',
        re.MULTILINE,
    )
    match = pattern.search(text)
    if match:
        arm = match.group("indent") + arm_body
        text = text[: match.start()] + arm + text[match.end() :]
    else:
        fallback = '            _ => (format!(".gitbook/assets/beings/empty/{id}.svg"), None),'
        if fallback not in text:
            # main.rs currently uses deeper indentation.
            fallback = '                        _ => (format!(".gitbook/assets/beings/empty/{id}.svg"), None),'
        if fallback not in text:
            die(f"cannot find direct-hero match fallback in {path}")
        indent = fallback[: len(fallback) - len(fallback.lstrip())]
        arm = indent + arm_body
        text = text.replace(fallback, arm + "\n" + fallback, 1)

    if "Command::new(" in text and "use std::process::Command;" not in text:
        insert_after = "use std::path::{Path, PathBuf};"
        if insert_after in text:
            text = text.replace(insert_after, insert_after + "\nuse std::process::Command;", 1)
        else:
            die(f"cannot add Command import in {path}")

    path.write_text(text, encoding="utf-8")

def main() -> None:
    if len(sys.argv) not in (3, 4):
        die("usage: apply.py <DEVINES_ID> <image-file> [repo-root]")

    devines_id = sys.argv[1].strip().upper()
    image_src = Path(sys.argv[2]).expanduser().resolve()
    root = Path(sys.argv[3]).expanduser().resolve() if len(sys.argv) == 4 else Path.cwd().resolve()

    if not image_src.is_file():
        die(f"image not found: {image_src}")

    identity_path = root / ".gitbook/assets/BEING_IDENTITY_MANIFEST.json"
    asset_path = root / ".gitbook/assets/ASSET_MANIFEST.json"
    ref_path = root / "DESIGN/IDENTITY_IMAGE_REFERENCE_MAP.json"
    for p in (identity_path, asset_path, ref_path):
        if not p.is_file():
            die(f"repo file missing: {p}")

    identity = load_json(identity_path)
    beings = identity.get("beings", {})
    if devines_id not in beings:
        die(f"{devines_id} is not registered in BEING_IDENTITY_MANIFEST.json")

    pages = list((root / "BOOKS/BOOK-II-BEINGS").glob(f"**/{devines_id}.md"))
    if len(pages) != 1:
        die(f"expected exactly one profile page for {devines_id}, found {len(pages)}")
    page = pages[0]

    data = image_src.read_bytes()
    ext = detect_ext(data)
    sha256 = hashlib.sha256(data).hexdigest()
    rel_hero = f".gitbook/assets/beings-direct/{devines_id}{ext}"
    hero = root / rel_hero
    hero.parent.mkdir(parents=True, exist_ok=True)
    hero.write_bytes(data)

    # Wire the profile hero directly.
    rel_from_page = os.path.relpath(hero, page.parent).replace(os.sep, "/")
    page_text = page.read_text(encoding="utf-8")
    hero_line = f"![{devines_id}]({rel_from_page})"
    if re.search(r"^!\[[^\]]*\]\([^)]+\)", page_text, re.MULTILINE):
        page_text = re.sub(r"^!\[[^\]]*\]\([^)]+\)", hero_line, page_text, count=1, flags=re.MULTILINE)
    else:
        page_text = hero_line + "\n\n" + page_text
    page.write_text(page_text, encoding="utf-8")

    # Identity manifest.
    beings[devines_id]["hero_asset"] = rel_hero
    save_json(identity_path, identity)

    # Asset manifest.
    assets = load_json(asset_path)
    record = next((x for x in assets.get("beings", []) if x.get("id") == devines_id), None)
    if record is None:
        die(f"{devines_id} missing from ASSET_MANIFEST.json")
    record["hero_path"] = rel_hero
    record["hero_sha256"] = sha256
    save_json(asset_path, assets)

    # Reference map.
    refs = load_json(ref_path)
    ref_root = refs.get("identities") or refs.get("beings") or refs
    if devines_id not in ref_root:
        die(f"{devines_id} missing from IDENTITY_IMAGE_REFERENCE_MAP.json")
    ref_root[devines_id]["hero_asset"] = rel_hero
    ref_root[devines_id]["hero_path"] = rel_hero
    ref_root[devines_id]["hero_sha256"] = sha256
    save_json(ref_path, refs)

    # Rust publication gates.
    update_rust(root / "tools/devines-chronicles/src/main.rs", devines_id, rel_hero, sha256)
    update_rust(root / "tools/devines-chronicles/src/review.rs", devines_id, rel_hero, sha256)

    # Ensure copied bytes are exact.
    if hashlib.sha256(hero.read_bytes()).hexdigest() != sha256:
        die("post-copy hash mismatch")

    print(f"DEVINES BEINGS PROFILE: {devines_id}")
    print(f"hero={rel_hero}")
    print(f"sha256={sha256}")
    print(f"profile={page.relative_to(root)}")

    subprocess.run(
        [
            "cargo", "run",
            "--manifest-path", "tools/devines-chronicles/Cargo.toml",
            "--release", "--", "validate", ".",
        ],
        cwd=root,
        check=True,
    )
    print("DEVINES BEINGS PROFILE: VALIDATION PASS")
    print("Next: review git diff, commit, merge, then validate merged main on DEVHub.")


if __name__ == "__main__":
    main()
,
        re.MULTILINE,
    )
    match = pattern.search(text)
    if match:
        arm = match.group("indent") + arm_body
        text = text[: match.start()] + arm + text[match.end() :]
    else:
        fallback = '            _ => (format!(".gitbook/assets/beings/empty/{id}.svg"), None),'
        if fallback not in text:
            # main.rs currently uses deeper indentation.
            fallback = '                        _ => (format!(".gitbook/assets/beings/empty/{id}.svg"), None),'
        if fallback not in text:
            die(f"cannot find direct-hero match fallback in {path}")
        indent = fallback[: len(fallback) - len(fallback.lstrip())]
        arm = indent + f'"{devines_id}" => ("{hero_path}".to_string(), Some("{sha256}")),'
        text = text.replace(fallback, arm + "\n" + fallback, 1)

    if "Command::new(" in text and "use std::process::Command;" not in text:
        insert_after = "use std::path::{Path, PathBuf};"
        if insert_after in text:
            text = text.replace(insert_after, insert_after + "\nuse std::process::Command;", 1)
        else:
            die(f"cannot add Command import in {path}")

    path.write_text(text, encoding="utf-8")


def main() -> None:
    if len(sys.argv) not in (3, 4):
        die("usage: apply.py <DEVINES_ID> <image-file> [repo-root]")

    devines_id = sys.argv[1].strip().upper()
    image_src = Path(sys.argv[2]).expanduser().resolve()
    root = Path(sys.argv[3]).expanduser().resolve() if len(sys.argv) == 4 else Path.cwd().resolve()

    if not image_src.is_file():
        die(f"image not found: {image_src}")

    identity_path = root / ".gitbook/assets/BEING_IDENTITY_MANIFEST.json"
    asset_path = root / ".gitbook/assets/ASSET_MANIFEST.json"
    ref_path = root / "DESIGN/IDENTITY_IMAGE_REFERENCE_MAP.json"
    for p in (identity_path, asset_path, ref_path):
        if not p.is_file():
            die(f"repo file missing: {p}")

    identity = load_json(identity_path)
    beings = identity.get("beings", {})
    if devines_id not in beings:
        die(f"{devines_id} is not registered in BEING_IDENTITY_MANIFEST.json")

    pages = list((root / "BOOKS/BOOK-II-BEINGS").glob(f"**/{devines_id}.md"))
    if len(pages) != 1:
        die(f"expected exactly one profile page for {devines_id}, found {len(pages)}")
    page = pages[0]

    data = image_src.read_bytes()
    ext = detect_ext(data)
    sha256 = hashlib.sha256(data).hexdigest()
    rel_hero = f".gitbook/assets/beings-direct/{devines_id}{ext}"
    hero = root / rel_hero
    hero.parent.mkdir(parents=True, exist_ok=True)
    hero.write_bytes(data)

    # Wire the profile hero directly.
    rel_from_page = os.path.relpath(hero, page.parent).replace(os.sep, "/")
    page_text = page.read_text(encoding="utf-8")
    hero_line = f"![{devines_id}]({rel_from_page})"
    if re.search(r"^!\[[^\]]*\]\([^)]+\)", page_text, re.MULTILINE):
        page_text = re.sub(r"^!\[[^\]]*\]\([^)]+\)", hero_line, page_text, count=1, flags=re.MULTILINE)
    else:
        page_text = hero_line + "\n\n" + page_text
    page.write_text(page_text, encoding="utf-8")

    # Identity manifest.
    beings[devines_id]["hero_asset"] = rel_hero
    save_json(identity_path, identity)

    # Asset manifest.
    assets = load_json(asset_path)
    record = next((x for x in assets.get("beings", []) if x.get("id") == devines_id), None)
    if record is None:
        die(f"{devines_id} missing from ASSET_MANIFEST.json")
    record["hero_path"] = rel_hero
    record["hero_sha256"] = sha256
    save_json(asset_path, assets)

    # Reference map.
    refs = load_json(ref_path)
    ref_root = refs.get("identities") or refs.get("beings") or refs
    if devines_id not in ref_root:
        die(f"{devines_id} missing from IDENTITY_IMAGE_REFERENCE_MAP.json")
    ref_root[devines_id]["hero_asset"] = rel_hero
    ref_root[devines_id]["hero_path"] = rel_hero
    ref_root[devines_id]["hero_sha256"] = sha256
    save_json(ref_path, refs)

    # Rust publication gates.
    update_rust(root / "tools/devines-chronicles/src/main.rs", devines_id, rel_hero, sha256)
    update_rust(root / "tools/devines-chronicles/src/review.rs", devines_id, rel_hero, sha256)

    # Ensure copied bytes are exact.
    if hashlib.sha256(hero.read_bytes()).hexdigest() != sha256:
        die("post-copy hash mismatch")

    print(f"DEVINES BEINGS PROFILE: {devines_id}")
    print(f"hero={rel_hero}")
    print(f"sha256={sha256}")
    print(f"profile={page.relative_to(root)}")

    subprocess.run(
        [
            "cargo", "run",
            "--manifest-path", "tools/devines-chronicles/Cargo.toml",
            "--release", "--", "validate", ".",
        ],
        cwd=root,
        check=True,
    )
    print("DEVINES BEINGS PROFILE: VALIDATION PASS")
    print("Next: review git diff, commit, merge, then validate merged main on DEVHub.")


if __name__ == "__main__":
    main()
