# GitBook Sync Setup

This repository is structured as the Git source for DEVINES CHRONICLES.

1. In GitBook, create or select the public DEVINES space.
2. Connect **Git Sync → GitHub**.
3. Select `Devineverse/DEVINES-CHRONICLES` and branch `main`.
4. Use the repository root as the content root.
5. Use `README.md` as the landing page.
6. Import navigation from `SUMMARY.md`.
7. Keep Git Sync two-way only if GitBook edits are also intended to become repository history.

## Canonical Site-Wide Appearance

In GitBook **Customization → site-wide**, apply these values to the entire DEVINES site rather than to a single section:

- Theme: **Clean**
- Default mode: **Dark**
- Tint color (dark): **#000000**
- Primary color (dark): **#CFAEEE**
- Sidebar: **Default** background, not Filled
- Links/active state: AUM Lavender / DEVINES Violet only
- Corners: minimal
- Shadows/depth: subtle or none
- Do not introduce gray page backgrounds or blue GitBook accents

This site-wide tint is required so the black inside AUM and Being circular identity assets merges visually with the surrounding page instead of appearing as a second black rectangle.

GitBook currently stores these values as site customization settings; Git Sync controls manuscript content but does not by itself apply the hosted site's customization panel.
