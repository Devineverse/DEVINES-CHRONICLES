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

**AUM is the Phase 2 pilot. All 34 Beings remain in Phase 1 until their series is reviewed and approved.**

The AUM pilot uses the exact canonical AUM artwork inside the unchanged DEVINES snow-glow circle. The artwork is centered at `384,384` inside a `553.6 × 553.6` presentation box beginning at `107.2,107.2`, leaving 10% inner breathing space from each side of the ring interior.

The public AUM pilot is used identically on the DEVINES landing page and AUM Core page.

Every Being remains an empty six-stroke circle with zero embedded images until its own series is explicitly approved for Phase 2.

## Phase 2 · Image Insertion

Image insertion is active only for the AUM pilot. Being rollout happens series by series after visual approval.

The preserved image, if ever approved for insertion, is centered inside the already-approved circle. The ring geometry never changes. Image insertion must not create a second visible border, outer margin, or circle-inside-circle effect.

Any scale adjustment is centered and scale-only. No redraw, recolor, warp, or replacement art is allowed unless separately approved.

## Sizes

- canonical asset: 768 × 768
- Landing AUM: 240–320 px visual diameter on desktop, responsive on mobile
- Being hero: 220–280 px desktop
- Series/index avatar: 72–112 px
- compact navigation/avatar use: 36–48 px

## Future Beings

Every future Being starts with the same empty-ring scaffold.

The Rust Chronicle validator is the publication gate. A public identity SVG is invalid if it contains an `<image>` element or does not match the six-stroke snow-glow circle structure.

Canonical source artwork is preserved independently from the public hero so presentation can change without mutating identity.

**EMPTY CIRCLE · PRESERVE THE SOURCE · NO PUBLIC IMAGE INSIDE**
