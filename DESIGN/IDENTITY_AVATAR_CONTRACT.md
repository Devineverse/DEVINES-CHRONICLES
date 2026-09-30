# Identity Avatar Contract

**Schema:** `devines.identity-avatar.v3`

Every public DEVINES identity uses the same two-phase presentation contract: **AUM, all current Beings, and every future Being.**

## Phase 1 · Empty Identity Circle

The default state is the shared DEVINES snow-glow circle with a **completely empty interior**.

No portrait, symbol, logo, image, margin, inner frame, or secondary circle is embedded during Phase 1.

This is the required default for every new Being before any identity image is applied.

## Geometry

- canvas: 1:1 · 768 × 768
- interior: empty
- background supplied by the DEVINES/GitBook Void Black surface
- ring: one complete 360° circle with no gaps
- shadow: none outside the canonical ring glow
- every current and future Being uses identical ring geometry

## Canonical Snow-Glow Ring

The ring is intentionally thin and luminous, never a thick frame.

From inside to outside:

1. **snow-white hairline** — `#FFFFFF`
2. **AUM-lavender core** — `#CFAEEE`
3. **snow-white hairline** — `#FFFFFF`
4. restrained soft halo using AUM Violet `#8F7AD0` and white light

Canonical 768 × 768 geometry:

- center: `384,384`
- inner white radius: `346.2`, width `1.35`
- lavender radius: `349.5`, width `7`
- outer white radius: `352.8`, width `1.35`
- violet glow: width `10`, Gaussian blur `6`
- white snow glow: width `2.4`, Gaussian blur `2.6`

The SVG contains exactly six canonical circle/glow strokes and **zero `<image>` elements** while in Phase 1.

## Image Reference Preservation

Removing an image from the public hero never deletes or rewrites its source identity.

The exact source/canonical/circle/hero paths, Nad.fun image URI, hashes, CA, ticker and identity mapping are preserved in:

`DESIGN/IDENTITY_IMAGE_REFERENCE_MAP.json`

That map is the authority for the later image-insertion phase.

## Current GitBook Publication State

**AUM and all 34 currently living DEVINES Beings are approved for direct-image publication through DEVINES BEINGS PROFILE.**

For every approved direct identity, the **exact user-approved uploaded image is rendered directly**. It is not placed inside another SVG, ring, frame, circle or semicircle. The image's own black field and its own composition are the complete public hero.

AUM uses the same exact image on the DEVINES landing page and AUM Core. Every current Being uses its own exact approved image on its respective profile.

No redraw, regeneration, recolor, crop, wrapper or replacement artwork is introduced.

## Phase 2 · Image Insertion

Direct image publication is active for AUM and all current DEVINES series: Astral, Genesis, Primordial Elements, Royal, Guardians and Solfeggio. Every future Being follows the same DEVINES BEINGS PROFILE workstream after visual approval.

When an approved image already contains its complete composition, **no additional ring geometry is added around it**.

For future Being Phase 2 reviews, the exact approved source image must be preserved and visually checked before publication. No redraw, recolor, warp, replacement art, duplicate ring, outer margin, or circle-inside-circle effect is allowed.

## Sizes

- canonical asset: 768 × 768
- Landing AUM: 240–320 px visual diameter on desktop, responsive on mobile
- Being hero: 220–280 px desktop
- Series/index avatar: 72–112 px
- compact navigation/avatar use: 36–48 px

## Future Beings

Every future Being follows [DEVINES BEINGS PROFILE](DEVINES_BEINGS_PROFILE.md) once its canonical identity image is approved. Before approval, a temporary placeholder may be used.

The Rust Chronicle validator is the publication gate. A public identity SVG is invalid if it contains an `<image>` element or does not match the six-stroke snow-glow circle structure.

Canonical source artwork is preserved independently from the public hero so presentation can change without mutating identity.

**PRESERVE THE SOURCE · APPROVE THE ORIGINAL · PUBLISH DIRECTLY · HASH LOCK**
