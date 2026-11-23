# Component-Extracted Historical Vector Tracing Pipeline

This document describes the reproducible pipeline used to convert high-resolution medieval manuscript illuminations into pixel-locked, multi-layer, reusable SVG anatomical components and living-ornament animations.

Validated reference implementation: [extract_roundel.py](extract_roundel.py) on `../refs/louis/1-coiled-dragon-roundel.jpg` (**95.01% exact silhouette IoU**).

---

## Why Per-Component Extraction Beats Freehand SVG Tracing

Direct hand-plotted or coordinate-gridded Bézier tracing introduces 2–12 unit contour drift, smooths away irregular iron-gall brush pressure, and homogenizes medieval silhouettes into modern geometric curves.

Extracting each anatomical part directly from manuscript pixels first—and vectorizing each pigment and ink layer independently with `potrace`—preserves:

- Exact 12th–15th century contour proportions, swells, ripples, and tapers.
- Authentic variable-width iron-gall linework, claw hooks, ear clefts, and broken painterly brush marks.
- True open voids (e.g., transparent gold-through open mouths rather than solid black fills).
- Clean reusable anatomical parts (`head.svg`, `wing.svg`, `foot-right.svg`, `foot-bottom.svg`, `inner-coil.svg`, `outer-coil.svg`) that can be overlaid 1:1 on the source, animated around true joints, or recombined onto other creatures.

---

## Prerequisites

- **`potrace` 1.16+**: `sudo apt-get install -y potrace`
- **Python 3** with `Pillow` (`PIL`)
- **Playwright + Chromium** (optional, for automated verification sheets and pixel-level Silhouette IoU scoring)

---

## Five-Step Pipeline

### Step 1 — Chromatic Pigment Separation & Ground Removal

Work at `900×900` (`3 px = 1 SVG unit` on a `300×300` `viewBox`): high enough for `< 0.35` SVG-unit edge accuracy while naturally suppressing single-pixel parchment grain.

1. **Spatial border exclusion**: Mask out framing borders outside the inner illumination field (e.g., `r > 116.5u` from roundel center), keeping tight allowance polygons only where limbs step over the frame (`foot-right`, `foot-bottom`) and rejecting terracotta-red / lapis-blue border paint inside those boxes.
2. **Gold-leaf / ground rejection**: Reject burnished gold leaf and dark tarnished gold shadows via their high yellow chroma (`R > G + 7`, `G - B > 33`, `B < 116`, `R - G < 65`). Medieval body pigments (ivory, claret-wine, mauve-plum, crimson, sage green, iron-gall ink) sit cleanly outside this gold region.
3. **Six-class pigment classification**:
   - **Iron-gall ink**: `lum < 52` or (`lum < 68` and `R - G < 15`) — separated from deep claret-wine shadows by neutral chroma.
   - **Sage covert**: cool green (`G >= R - 11`, `G > B + 4`, `lum > 68`) localized to the wing covert.
   - **Ivory highlight + lead-white pearl dots**: `lum > 146`, `B > 114`, `R - B < 54`.
   - **Crimson mask / whiskers / crown hatch / pinion ribs**: `R - G > 34`, `R - B > 46`, `R > 94`, `G < 112` in head/wing boxes.
   - **Deep claret-wine dorsal band (`--wash2`)**: `lum < 104`, `R > G + 8`, `G - B < 24`.
   - **Mid mauve-plum body wash (`--wash`)**: remaining non-gold cool/neutral body tones.
4. **Connected-component cleanup**: Apply a 3px morphological close, retain only the single largest connected dragon mask (dropping 100% of stray dust flecks), and fill tiny internal pinholes (`<= 420 px²`) without filling real anatomical voids such as the open mouth gap. Gate every pigment mask against this clean dragon mask.

### Step 2 — Anatomical Component Segmentation

Split the verified dragon silhouette along true medieval anatomical joints using region-of-interest polygons:

| Component | Role / Articulation | Z-Order |
| :--- | :--- | :---: |
| `outer-coil.svg` | Static outer spiral tail (ground-truth anchor) | 1 (back) |
| `foot-right.svg` | Clutching right foot + white ankle bead | 2 |
| `foot-bottom.svg` | Emerging lower leg, rounded heel knob, 3 claws | 2 |
| `inner-coil.svg` | Neck and swollen torso coil | 3 |
| `wing.svg` | Diagonal pinion fan, crimson ribs, white bone, sage covert | 4 |
| `head.svg` | Sub-segmented into `head-jaw`, `head-skull`, `head-ears` | 5 (front) |

**Joint-cut rule (prevents floating hairline seams):**
When subtracting an upper component from a lower component (e.g., removing `head` from `inner-coil`), subtract a **dilated** mask of the upper component everywhere *except* a narrow overlap rectangle right at the true physical joint (neck base, wing root, ankle roots). This eliminates 1–2px halo rings around separated components while ensuring subtle rotation in animation never opens a visible gap at the joint.

### Step 3 — Per-Component Multi-Layer `potrace` Vectorization

For each component, trace its layers bottom-to-top using `potrace -s --flat -t <turdsize> -a 1.0 -O <opttolerance>` and convert coordinates cleanly into `0..300` space (1 decimal place):

1. **Base silhouette** (`turdsize=22, opttolerance=0.65`) — filled with `var(--wash, #7c3a4c)` on coils, ivory on head/wing/feet.
2. **Deep tonal wash** (morphological `open(3) → close(5)`, `turdsize=45, opttolerance=1.2`) — filled with `var(--wash2, #4c1624)` on coils for smooth painterly gouache bands without halftone speckle noise.
3. **Feature pigments**:
   - Coils: ivory rim + hand-dabbed white pearl dots (`turdsize=5, opttolerance=0.75`).
   - Wing: crimson pinion ribs, hole-filled solid sage covert (`#97a988`), white leading bone.
   - Head: crimson mask + whiskers + skull-crown hatching (`turdsize=7, opttolerance=0.6`), pale ivory muzzle/cheek/sclera/ears (`turdsize=5, opttolerance=0.55`).
4. **Iron-gall ink linework** (`turdsize=7` on head/wing, `14` on coils; `opttolerance=0.55–0.70`) — traced directly from the ink mask dilated 3px into the component boundary so every variable-width stroke, claw, ear cleft, and charcoal feather mark sits crisp on top.

Avoid `<use href="#id"/>` inside embeddable SVGs so multiple instances (colour variants, pure-black silhouette previews, and animated copies) never collide on DOM IDs in the same HTML document.

### Step 4 — Standalone Library & Stage A / B / C Assembly

- **Standalone components** (`traced/components/roundel/*.svg`): Each component is written with a tight cropped `viewBox` around its own bounding box so it can be inspected or composited independently.
- **Stage A — Static Master** (`romanesque-roundel-master.svg`): All 6 components re-assembled in exact `viewBox="0 0 300 300"` alignment with zero transforms—the visual ground truth.
- **Stage C — Living-Ornament Animated Derivative** (`romanesque-roundel-animated.svg`): Wraps articulated component groups in `translate(px,py) → animateTransform → translate(-px,-py)` pivots with prime-number SMIL durations (`4.7s`–`7.3s`) for non-repeating, restrained motion (head turn, open-gap jaw flex, ear twitch, blink lid, wing ruffle, foot grip, inner-coil breath) while keeping the outer spiral tail static.
- **Metadata** (`romanesque-roundel.meta.json`): Records source reference, tracing method, measured Silhouette IoU, component manifest, assemblies, and CSS palette tokens.

### Step 5 — Quantitative & Visual Verification Gate

Every traced asset must pass four automated checks rendered in headless Chromium:

1. **Component 1:1 Overlay Sheet** (`romanesque-roundel-components.png`): Side-by-side `Extracted Pixels | Traced SVG | 50% Overlay` for every individual component at matched scale.
2. **Full & Zoomed 50% Overlay Sheet** (`romanesque-roundel-sheet.png`): Source, full 50% overlay, zoomed head 50% overlay, and zoomed wing/feet 50% overlay.
3. **Silhouette & Recolour Tests**: Pure-black silhouette test (verifying historical authenticity without colour), gilded roundel context, and CSS variable `--wash` / `--wash2` recolouring (e.g., mineral lapis).
4. **Silhouette IoU Metric**: Rasterize the composited SVG silhouette at `600×600` and measure pixel Intersection-over-Union against the extracted manuscript dragon mask (target `>= 94%`; achieved **95.01%**).

---

## Detail Pass: Trace the Detail at Full Resolution

The broad fills and the detail are separate layers, composited: the fills (base, wash, wash2) carry
the shape and the colour, and the ink and highlight layers on top carry everything that makes the
creature legible: pupils and eye highlights, brows, nostrils, ear and feather lines, scale hatching,
fur strokes. **Trace the detail layers at full resolution, never at the fills' resolution.**

### Why the first traces looked impressionistic

On the first amphisbaena traces (November 2026) the outline matched the source at 96–98%, but the
ink only at 50–70%: the creatures read as painterly washes with the line work smudged away. The
detail layers were being treated like fills:
- **Resolution:** the ink layer was traced at 750 px (`trace_ink.scale_size`) from a 900 px working
  image, so hairline strokes merged or vanished.
- **Smoothing:** a blur near 1 px and a speckle filter of 5–8 px, right for a fill, removed short
  strokes, dots and the small shapes of an eye.
- **Ink test:** ink was taken as near-black (`ink_l_max` 28, `ink_c_max` 20). Medieval iron-gall and
  sepia ink is brown and often faded, so most of the line work fell into the fills.

### The fix: the `-fine` presets

`lib/families/illuminated.py` registers a `-fine` variant of each preset
(`parchment_pale_subject_fine`, `parchment_drollery_fine`, `gilded_romanesque_fine`,
`gilded_interlace_compact_fine`, `monochrome_woodcut_fine`). Each keeps its base preset's ground
removal and fills unchanged, and changes only the detail:

| Setting | Base presets | `-fine` |
| --- | --- | --- |
| Working image (`work_size`) | 900 px | 1600 px |
| Ink and highlight trace (`scale_size`) | 750–900 px | 2000 px |
| Blur (`blur_radius`) | 0.85–1.1 | 0.35 |
| Speckle filter (`filter_speckle`) | 5–8 | 2 |
| Shortest stroke kept (`min_subpath_u`) | 0.35 u | 0.12 u |
| Ink lightness / chroma limits | 28 / 20 | 40 / 30 |
| Ink valley test (`ink_valley_delta`, `ink_valley_l_max`) | 12–13, 56–58 | 9, 68 |

Measured on the winged amphisbaena (`8-winged-amphisbaena-linear.jpg`): ink IoU **67.9% → 86.2%**,
silhouette unchanged (98.3%), 24 KB gzipped. The eyes keep their pupils and highlights; the brows,
nostrils and ear lines come back.

### Using it

- **Use a `-fine` preset for every painted specimen** from now on. Run the base preset only to
  compare.
- **Check the result by eye,** on the zoomed head crops of the sheet, not by the metric: the silhouette
  IoU is measured against the pipeline's own mask and can stay high while a head is lost entirely.
  Record the ink IoU before and after.
- **Pen and pencil drawings:** the `-fine` detail can turn grain into speckle. Compare
  `monochrome_woodcut` with `monochrome_woodcut_fine` and keep whichever reads cleaner; a faint drawing
  may first need the contrast-raised copy from `derived/contrast_outline.py`.
- **Size:** the detail roughly doubles a creature's path data. That is the price of legibility; if a
  page needs it smaller, simplify the fills, not the ink.

## Running the Extractor

```bash
python3 docs/art/bestiary/traced/extract_roundel.py
```

Outputs updated:
- `docs/art/bestiary/traced/components/roundel/*.svg`
- `docs/art/bestiary/traced/romanesque-roundel-master.svg`
- `docs/art/bestiary/traced/romanesque-roundel-animated.svg`
- `docs/art/bestiary/traced/romanesque-roundel.meta.json`
- `docs/art/bestiary/traced/romanesque-roundel-components.png`
- `docs/art/bestiary/traced/romanesque-roundel-sheet.png`
- `docs/art/bestiary/proofs/proof-traced-roundel.html`

---

## Adapting to Other Historical References

- **Border drolleries (`refs/louis/3-dragon-border-drollery.jpg`)**: Swap gold-leaf rejection for cream-vellum rejection (`lum > 185` and low chroma), separate burnt-orange/vermilion body pigment from charcoal wings/legs and surrounding blue/gold acanthus vines, then trace head, wing, legs, torso, and foliate tail separately.
- **Gothic inhabited initials (`refs/louis/2-winged-dragon-initial-interlace.jpg`)**: Isolate each intertwined dragon by its distinct mineral hue (green bearded dragon vs orange/ochre winged serpents vs blue/gold letter frame) into separate over/under crossing layers before `potrace` vectorization.
- **Early modern woodcuts (`DE DRACONE.`)**: Single-channel luminance thresholding + adaptive local ink valley extraction cleanly isolates crisp monochrome hatching, scales, and anatomical contours in 2 layers (parchment fill + iron-gall/woodblock ink).
