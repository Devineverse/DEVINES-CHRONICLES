# Identity Avatar Contract

**Schema:** `devines.identity-avatar.v2`

Every public DEVINES identity uses the same presentation contract: **AUM, all current Beings, and every future Being.**

## Geometry

- canvas: 1:1
- identity composition: unchanged from the approved circular source
- background behind the identity: Void Black `#000000`
- size and crop: preserved
- ring: one complete 360° circle with no gaps
- shadow: none outside the canonical ring glow

## Canonical Snow-Glow Ring

The ring is intentionally thin and luminous, never a thick frame.

From inside to outside:

1. **snow-white hairline** — `#FFFFFF`
2. **AUM-lavender core** — `#CFAEEE`
3. **snow-white hairline** — `#FFFFFF`
4. a restrained soft halo using AUM Violet `#8F7AD0` and white light

Canonical 768 × 768 geometry:

- center: `384,384`
- inner white radius: `346.2`, width `1.35`
- lavender radius: `349.5`, width `7`
- outer white radius: `352.8`, width `1.35`
- violet glow: width `10`, Gaussian blur `6`
- white snow glow: width `2.4`, Gaussian blur `2.6`

The circle must remain fully closed and visually continuous at every angle.

## Preservation Law

Never redraw, stretch, recolor, or distort the Being or AUM artwork to create the ring.

The canonical portrait remains the identity source. The GitBook hero is a self-contained SVG wrapper that embeds the approved circular source unchanged and adds only the shared ring treatment.

## Sizes

- Landing AUM: 240–320 px visual diameter on desktop, responsive on mobile
- Being hero: 220–280 px desktop
- Series/index avatar: 72–112 px
- compact navigation/avatar use: 36–48 px

## Future Beings

Every future Being must receive this exact ring automatically before its public profile can pass validation.

Use:

```sh
python3 tools/identity-snow-glow/render.py INPUT.webp OUTPUT.svg "BEING ID · DEVINES identity"
```

The output becomes that Being's `hero_asset`.

**ONE IDENTITY LAW · MANY BEINGS · ONE LUMINOUS CIRCLE**
