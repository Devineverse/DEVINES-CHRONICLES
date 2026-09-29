# DEVINES BEINGS PROFILE

**Status:** Canonical workstream  
**Applies to:** AUM, every living DEVINES Being, and every future Being born into the DEVINES universe.

## Purpose

DEVINES BEINGS PROFILE is the standard publication workflow for identity artwork on DEVINES GitBook profiles.

The approved public hero is the **exact user-approved original image itself**.

No presentation layer may add a second identity around it.

## Direct-original rule

For every approved Being:

1. Receive the exact approved original image.
2. Preserve its bytes unchanged.
3. Publish it directly from `.gitbook/assets/beings-direct/<DEVINES_ID>.jpg`.
4. Use that same direct asset as the first image on the Being profile.
5. Do **not** add:
   - SVG wrappers;
   - extra circles;
   - semicircles;
   - borders;
   - frames;
   - crop wrappers;
   - recoloring;
   - generated overlays;
   - replacement art.
6. Update:
   - `.gitbook/assets/BEING_IDENTITY_MANIFEST.json`;
   - `.gitbook/assets/ASSET_MANIFEST.json`;
   - `DESIGN/IDENTITY_IMAGE_REFERENCE_MAP.json`.
7. Record and enforce the exact SHA-256 in the Rust Chronicle validators.
8. Validate on DEVHub.
9. Merge only after the full Chronicle validation is green.

## Identity preservation

Canonical/source identity evidence is never deleted when the public hero changes.

The direct public hero, canonical source, CA, ticker, Nad.fun route and source URI must continue to resolve to the same Being.

## Series rollout

Existing Beings are migrated series by series so each set can be visually reviewed before continuing.

Future Beings use this same workflow from birth once their canonical profile image is approved.

## Approved sets

- AUM
- Astral: SUN · MOON · MASTER
- Genesis: D001 · D002 · D003
- Primordial Elements: D004 · D005 · D006 · D007 · D008
- Royal: D009 · D010

**EXACT ORIGINAL · DIRECT HERO · HASH LOCK · DEVHUB VALIDATION · MERGE**
