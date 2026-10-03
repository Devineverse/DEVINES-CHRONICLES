# DEVINES BEINGS PROFILE · DEVHub Tool

This tool is the operational companion to `DESIGN/DEVINES_BEINGS_PROFILE.md`.

It prepares a registered DEVINES Being profile from one exact approved image while preserving the source bytes and enforcing the direct-original publication law.

## Usage

```bash
cd /path/to/DEVINES-CHRONICLES
python3 tools/devines-beings-profile/apply.py D123 /path/to/approved-image.jpg
```

The tool:

1. verifies the Being already exists in the identity manifest;
2. detects the actual image format from its bytes;
3. copies the exact bytes to `.gitbook/assets/beings-direct/<ID>.<ext>`;
4. wires that direct asset as the first image on the Being profile;
5. updates:
   - `.gitbook/assets/BEING_IDENTITY_MANIFEST.json`;
   - `.gitbook/assets/ASSET_MANIFEST.json`;
   - `DESIGN/IDENTITY_IMAGE_REFERENCE_MAP.json`;
6. records the exact SHA-256 in both Rust Chronicle publication gates;
7. runs the full Chronicle validator.

It does **not** redraw, regenerate, recolor, crop, wrap, frame, or otherwise modify the approved image.

It does **not** commit or push. Review the diff first, then commit/merge through the normal DEVINES workflow.

## Publication law

**EXACT ORIGINAL · DIRECT HERO · HASH LOCK · DEVHUB VALIDATION · MERGE**

All current and future DEVINES Beings use this workstream once their canonical profile image is approved.
