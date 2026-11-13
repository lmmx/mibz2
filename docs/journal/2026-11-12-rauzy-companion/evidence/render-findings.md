# Rendering findings and corrections

Actual Chromium screenshots were inspected at1280×900 and390×844, light and dark.
The page checker saves every H2 section, splitting long sections into1600px slices.
A fresh reader independently read the prose twice; this file records the separate visual
inspection. Exact-head rerun and the complete screenshot archive/manifest are linked in
the completion report and PR, after the content commit exists.

Corrections made from actual renders:

1. The final four-stage boxed chain required horizontal scrolling on a phone. It now uses
   the identical terms/arrows vertically. The kernel-buffer inequality also uses two lines,
   preserving every coefficient, variable, inequality and byte unit.
2. The mapsto glyph in the SVG was missing in the available text font. The branch labels
   now write `F′(x)=x+2` / `F′(x)=x−3` explicitly. KaTeX equations retain their math fonts.
3. The crossing C translation struck through the bottom-row label. That label now sits
   below the strip; partition geometry and translations are unchanged.
4. The house table rule keeps the first column on one line. Page-local id-scoped wrapping
   now keeps four prose tables inside the phone's reading area. All entries are preserved.
5. The structural pseudocode wraps on phones. Its words, indentation, operations and
   caveat remain; no code or algorithm was changed.
6. The slot-layout cross-order arrow also crossed its lower row label. Moving that
   label below the lower strip preserves the cell coordinates and offset.
7. Caption/accessible-description numerical labels were spaced for readability.

Automated checks did not catch every visual defect: a scrolling math/table region passes
its overflow check, and it cannot identify a missing font glyph or a label crossed by a
line. Those findings came from looking at the screenshots, not from claiming a green
checker alone constitutes inspection.

The initial raw log is preserved as look-initial.log.gz (including trailing spaces in
unrendered math diagnostics). The initial direct check failed on blocked Google Fonts and the Chromium proxy-CA issue.
The retained asset fixture documents the exact KaTeX/font inputs used for local checks;
public dependency delivery still requires ordinary exact-head CI. The strict build's
Python inventory warning and empty local benchmark history are documented separately.

The social card was inspected at its actual1200×630 size: complete title/dek/footer, no
line truncation. The render helper checks unclamped line extent, rather than treating
font ink descent (`scrollHeight`) as an extra clipped text line. It uses the existing
house template and identical hero, embedded to avoid blocked `file:` navigation.

Final pre-commit inspection: 132 actual section screenshots of the article and index,
at desktop 1280×900 and phone 390×844, both themes. All were inspected; after the last
slot-label and phone-table corrections, 90 images were byte-identical and all 42 changed
images were inspected again. No remaining visual defect was identified in the changed
article. The history table now exposes all three columns on the phone without scrolling.
The earlier look-rebuild-race.log records a local 404 caused by rebuilding the served tree
while inspection was still running; the subsequent sequential build and look passed.
