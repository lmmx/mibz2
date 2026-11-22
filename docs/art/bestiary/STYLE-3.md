# Bestiary Style Sheet 3: Textured Mineral Gouache & Monochrome Woodcut Standard

This style sheet is the definitive visual and technical specification for Streams 2–6 under `docs/art/bestiary/BRIEF-3.md`. The flagship reference implementation is [`draco-bipes.svg`](draco-bipes.svg), verified against Louis's target manuscripts on [`draco-bipes-sheet.png`](draco-bipes-sheet.png).

---

## 1. Core Aesthetic: Textured Illuminated Miniature + Woodcut Bestiary

**Primary visual anchors (study all four before drawing a single path):**
1. **`refs/louis/3-dragon-border-drollery.jpg`** — late-medieval border dragon sitting on a margin: compact sculptural poise, arched cinnabar-orange S-neck painted with fine parallel contour brushwork, smoky olive-charcoal hindquarters, sturdy gripping clawed feet, long flame-curved swept horns, taut sharply tapered upturned tail.
2. **`refs/louis/1-coiled-dragon-roundel.jpg`** — Romanesque coiled dragon on gold: layered longitudinal gouache shading bands (lit outer rim, rich core, dark shadow seam), a single crisp row of tiny lead-white dots running along the dark dorsal seam, earnest hound-dragon head with fine radiating facial brushwork, weighty gripping claws.
3. **`refs/louis/2-winged-dragon-initial-interlace.jpg`** — interlaced Gothic dragon initial: rich deep mineral blue ribbons modelled with pale highlights and tiny white scale dots, hairy expressive head with flicking flames, folded ribbed wings, busy living interlace.
4. **`IMG_7329.jpeg` (`DE DRACONE.`)** — monochrome bestiary woodcuts: dense granular crescent dorsal scales, fine stipple dots, pale crescent ventral scutes along the belly and lower tail, decisive radial wing-rib shading.

### What "Bold" Means Here
- **Decisively shaped and stroked** — strong sculptural silhouettes, modulated dark iron-gall contours, and **rich surface texture** built from layered mineral gouache bands and fine directional brushwork/woodcut marks.
- **Mix of monochrome and rich mineral colour:** Pair rich, tactile **monochrome charcoal/slate/woodcut ink textures** with **deep mineral pigments** (cinnabar, madder wine, mineral lapis, verdigris, burnished ochre-gold). Richness comes from *texture, tonal layering, and brushwork* — never from flat plastic RGB blocks.
- **Explicitly forbidden:**
  - **No crude flat block colours or primary-colour clipart:** never leave large surfaces as flat unmodelled `#ff0000` / `#00b078` / `#1d42d8` vector fills.
  - **No cartoonishly thick uniform marker outlines** (`2.5–3.2 px` uniform tubes make creatures look like stickers).
  - **No childish or goofy tropes:** no bobblehead proportions, no big round googly eyes, no clipart party crowns, no cartoon tulip flowers, no stubby sausage legs, no slapstick grins.
  - **No pink or pastel:** read any rose/mauve in historical references as cinnabar, madder wine, ochre, or parchment ivory.

---

## 2. Linework & Rich Surface Texture Formula

Every form combines a **decisively shaped, subtly modulated iron-gall outer contour** with **layered longitudinal gouache bands** and **fine directional brushwork / woodcut textures**:

| Layer | Specification (at ~`340 × 220` viewBox) | Purpose & Reference |
| --- | --- | --- |
| **Weighted lower/gesture outer contour** | `stroke="#1c100a" stroke-width="1.85"–"2.15"`, `stroke-linecap="round" stroke-linejoin="round"` | Authoritative quill weight along belly, lower tail curve, jaw underside, nape shadow, and grounded feet. |
| **Crisp upper/lit outer contour** | `stroke="#1c100a" stroke-width="1.5"–"1.75"`, round caps & joins | Clean, decisive silhouette without chunky cartoon bulk. |
| **Layered longitudinal gouache bands** | 3 flat clipped tonal layers over `var(--wash)` / `var(--wash2)`:<br>1. Lit warm parchment/ivory rim (`#f4e5c8` or `#ece0c8`, `fill-opacity="0.28"–"0.36"`)<br>2. Mid mineral core (`var(--wash)` / `var(--wash2)`)<br>3. Deep shadow band (`#1c100a`, `fill-opacity="0.18"–"0.28"`) | Reproduces real layered manuscript tempera modelling (`1-coiled-dragon-roundel.jpg`) with zero SVG gradients, harmonizing with any CSS `--wash` override. |
| **Directional neck & body brushwork** | `stroke="#1c100a" stroke-opacity="0.44"–"0.58" stroke-width="0.68"–"0.85"` + lit flecks `#f6efe2` (`0.55` opacity) | Fine parallel curved fur/feather brushwork following the exact tangent of the S-neck, haunches, and feathered pigment seams (`3-dragon-border-drollery.jpg`). |
| **Granular woodcut crescent scales & stipples** | Staggered curved crescent arcs `stroke="#1c100a" stroke-opacity="0.72"–"0.78" stroke-width="0.78"–"0.86"` + tiny round-capped stipple dots `stroke-width="1.2"` | Dense, authentic bestiary woodcut tactility on charcoal/monochrome flanks and tails (`DE DRACONE.`). |
| **Pale crescent ventral scutes** | Clipped `#ece0c8` (`0.78–0.82` opacity) underside band bounded by `1.15–1.25u` seam + curved scute bars (`0.82–0.88u`, `0.74` opacity) | Crisp pale belly and tail plates hugging the lower contour (`DE DRACONE.`). |
| **Signature lead-white dorsal dot row** | Single compact `<path d="M...h.1 ...">` with `stroke="#f6efe2" stroke-width="1.45"–"2.2" stroke-linecap="round"` | One crisp, jewel-like pearl row tracing the dark upper dorsal seam (`1-coiled-dragon-roundel.jpg`). |
| **Radiating facial brushwork** | Fine short strokes `0.68–0.75u` (`0.55–0.60` opacity) radiating around cheek, brow, and muzzle | Replaces empty flat cartoon cheeks with authentic miniature brushwork (`1-coiled-dragon-roundel.jpg` & `3-dragon-border-drollery.jpg`). |

> **Compactness tip:** Quantize generated spline/texture coordinates to `0.5` units (`f1`), combine all scales/scutes/dots of the same stroke weight into single multi-subpath `<path d="M...Q... M...Q..."/>` elements, and use zero-length round-capped strokes (`M x,y h.1`) for stipples and white dot rows. This delivers dense hand-painted miniature + woodcut richness in **26–35 KB** (well under the 40 KB cap).

---

## 3. Mineral & Monochrome Palette (Rich Texture, Zero Flat Primaries, Zero Pink)

Every creature pairs **monochrome parchment/charcoal/iron-gall woodcut tones** with **rich mineral manuscript pigments**:

| Pigment | Hex | Role |
| --- | --- | --- |
| **Iron-Gall Ink** | `#1c100a` | Modulated outer contours, crescent scales, brushwork, stipples, pupils, shadow bands |
| **Cinnabar Mineral** | `#c84618` | Flagship `--wash` head/neck/breast pigment (`3-dragon-border-drollery.jpg`) |
| **Smoky Olive-Charcoal** | `#464c42` | Flagship `--wash2` monochrome torso, hindquarters, legs, and tail (`3-dragon-border-drollery.jpg`) |
| **Deep Madder Wine** | `#862428` | Romanesque coiled-dragon mineral body wash (`1-coiled-dragon-roundel.jpg`) |
| **Iron Slate** | `#3b403c` | Cool monochrome woodcut/wing slate |
| **Mineral Lapis** | `#244c94` | Interlaced Gothic dragon ribbons & heraldic borders (`2-winged-dragon-initial-interlace.jpg`) |
| **Deep Verdigris** | `#1f6e52` | Hairy Gothic dragon heads, folded wing mantles, serpent washes (`1-coiled`, `2-winged`) |
| **Burnished Ochre / Gold** | `#c28a22` (highlight `#e8c468`) | Monochrome-plus-gold woodcut accents, subtle gilded crests, manuscript highlights |
| **Parchment Ivory** | `#ece0c8` / `#f4e5c8` | Ventral belly scutes, lit gouache highlight bands, curved talons |
| **Lead White** | `#f6efe2` | Single crisp dorsal dot row, eye sclera, tiny articulated teeth, pinpoint eye glint |
| **Mouth Interior** | `#3a0f08` | Narrow open-jaw depth |
| **Claude Terracotta-Orange** | `#d85224` (accent `#F2683A`) | Headline-frame Claude serpents (layered with ochre highlight, madder shadow, and fine brushwork — never flat!) |

### Two-Tone CSS Variable Discipline (`--wash` & `--wash2`)
- Use `var(--wash, <mineral>)` for the primary coloured pigment (e.g. cinnabar head/neck/breast on `draco-bipes`) and `var(--wash2, <monochrome-or-contrast>)` for the contrasting monochrome charcoal/slate or secondary ribbon.
- Never hardcode tinted highlight/shadow colours that clash when `--wash` is overridden: model ALL highlights with translucent `#ece0c8` / `#f4e5c8` / `#f6efe2` and ALL shadows/brushwork/scales with translucent `#1c100a`.

---

## 4. Proportions, Anatomy & Character

- **Sculptural medieval poise:** Compact, weighty hindquarters sloping cleanly into a taut, sharply tapered tail; proud arched S-neck rising into a small, intense canine/equine beast head (head length ~1/5.5 of body sweep).
- **The Earnest Medieval Beast Head:**
  - Elongated, slightly blunt tapered snout with subtle upturned nose tip and crisp curved nostril.
  - **Two long, graceful flame-like S-curved swept-back horns/ears** flaring subtly apart at the tips, modelled with pale ochre-ivory inner ridges and dark longitudinal strokes (`3-dragon-border-drollery.jpg`).
  - **Small intense almond eye** with `#f6efe2` sclera, warm amber-umber iris rim (`#7c3810`), dark pupil (`#1c100a`), single pinpoint white glint, firm arched brow, and smoothly clipped upper blink lid (`<clipPath>`).
  - **Narrow parted jaws** with tiny articulated ivory teeth and a slim forked crimson ribbon tongue.
  - **Fine radiating cheek & brow brushwork** giving genuine painted-miniature expression.
- **Sturdy, Expressive Medieval Clawed Feet:**
  - Muscular thighs blending seamlessly at their upper-back edge into the scaly haunch (leave upper-back hip outline open so the leg grows organically out of the torso!).
  - Compact scaled hocks/tarsi and weighty splayed gripping toes with knuckle creases and curved pale parchment talons (`#ebe0ca`).
  - **Moonwalk IK discipline:** Grounded walkers pivot cleanly at hip, knee, and ankle (`upper + lower + foot ≈ 0°` on flat backward slide; `≈ +20°` on arched heel-pop), never lifting off the bubble edge.

---

## 5. Distinct Creature Roster (Streams 1–5)

Each species blends monochrome woodcut texture with rich mineral pigment and carries a distinct medieval temperament:

| Stream | Asset(s) | Default `--wash` / `--wash2` | Texture & Reference Anchor | Attitude & Motion Beat |
| --- | --- | --- | --- | --- |
| **1 (Lead)** | `draco-bipes.svg` | Cinnabar `#c84618` / Olive-Charcoal `#464c42` | `3-dragon-border-drollery.jpg` + `1-coiled` white dorsal dots + `DE DRACONE.` crescent scales & ventral scutes | **Aristocratic border showman** — dignified moonwalk heel-roll around the bubble edge, corner pause, alert sideways glance and slim tongue flick. |
| **2** | `draco-alatus.svg` | Mineral Lapis `#244c94` / Iron Slate `#3b403c` (or Cinnabar + Charcoal bat-wings) | `3-dragon-border-drollery.jpg` ribbed charcoal bat-wings + `DE DRACONE.` radial wing-rib shading + white spine dots | **Watchful aerial sentinel** — lifts off bubble corner, sweeps a poised figure-eight with ribbed wingbeats, glances sideways at the reader at the apex, lands cleanly touching the bubble. |
| **3** | `aspis.svg` | Deep Madder Wine `#862428` / Charcoal `#3b403c` | `1-coiled-dragon-roundel.jpg` layered longitudinal bands + crisp lead-white dorsal dot row + `DE DRACONE.` ventral scutes | **Sly & sinuous** — weaves over and under text lines, pauses with narrowed almond eye, bows cleanly into `"o"` / `"i"` (masked vanish, zero fade, zero drawn hole). |
| **3** | `spotted-serpent.svg` | Monochrome Charcoal `#464c42` / Burnished Ochre `#c28a22` | `DE DRACONE.` granular crescent scales, dense stipple spots, pale crescent scutes + `manuscript-1.jpg` alert ear & eye | **Alert & coiled** — taut poise, crisp look-around before a swift masked burrow with a final tail-hook flick. |
| **3** | `amphisbaena.svg` | Cinnabar `#c84618` / Deep Verdigris `#1f6e52` | Opposing sculpted canine-serpent heads on a densely scaled monochrome-charcoal spindle torso with pale scutes | **Contending rivals** — concertina crawl as both heads pull opposite ways, snap, glare sideways, and yield in turns. |
| **4** | `knotted-pair.svg` | Mineral Lapis `#244c94` & Deep Madder `#862428` | `2-winged-dragon-initial-interlace.jpg` living Gothic interlace ribbons with pale highlight edges, white scale dots & monochrome shaded crossings | **Entangled & relentless** — knot tightens and loosens as each serpent slides through to nip the other's tail, then glides along the bubble bottom edge and back. |
| **5** | Headline frame serpents (`claude-ruffed`) | Claude Terracotta `#d85224` / Olive-Charcoal `#464c42` (burnished ochre-gold highlights) | `2-winged-dragon-initial-interlace.jpg` hairy bearded dragon heads with fine flickering flames + textured scaly bodies | **Fierce & tireless** — chase around the headline frame, meeting at the bottom to snap and recoil; gold caption below stays completely uncovered at every viewport width. |
| **5** | `"yuck"` variant | Deep Verdigris `#1f6e52` / Monochrome Charcoal `#464c42` | Compact rounded marginalia beastlet with authentic fine brushwork, tiny white spine dots, and curled tail (deliberate odd one out via compact silhouette, still textured miniature craft — never plastic clipart!) | **Plucky wind-up odd-one-out** — briskly hops around the small bubble framing the red blackletter `"yuck"`. |

---

## 6. Technical SVG, SMIL & `crawler.js` Contract (Mandatory for All Streams)

Every SVG under `docs/art/bestiary/` MUST satisfy every check below and pass `render_svg.py` (`validate_svg`) with zero errors:

1. **Size & XML validity:**
   - Strictly **under 40 KB** (target 26–35 KB).
   - Valid XML (`xml.etree.ElementTree.fromstring` succeeds). **Never place `--` inside XML comments.**
   - Root `<svg>` MUST declare `xmlns="http://www.w3.org/2000/svg"`, explicit `viewBox`, `width`, `height`, `role="img"`, and `aria-label`.
   - **NEVER put inline `style="--wash:..."` on root `<svg>`** — rely strictly on `var(--wash, ...)` and `var(--wash2, ...)`.
2. **ID Scoping:**
   - Every `id` MUST start with `<file-stem>-`. Zero duplicate IDs.
3. **Required `<defs>` Exports for `crawler.js`:**
   - `<path id="<stem>-spine" class="spine" d="..." data-widths="..."/>`: centerline from neck-head junction through body to tail tip with ~15–21 body widths.
   - `<symbol id="<stem>-head-symbol" overflow="visible">...</symbol>`: facing `+x`, neck at `(0, 0)`, ~`20` units long, matching the main creature's sculpted textured head with proportional stroke weights (`0.55–0.65`).
4. **Pure SMIL Animation Only:**
   - Use `<animateTransform>` and `<animate>` only — **NO** `<script>`, **NO** CSS `@keyframes`/`animation`, **NO** `<filter>`, **NO** `<linearGradient>`/`<radialGradient>`, **NO** `<feTurbulence>`, **NO** `<animate attributeName="d">`.
   - Every `keyTimes` starts at `0`, ends at `1`, matches `values` count, and has `len(keyTimes) - 1` `keySplines` entries when `calcMode="spline"`.
   - Use prime/incommensurate durations (`2.3s`, `4.1s`, `4.3s`, `4.7s`, `5.3s`, `6.7s`, `7.1s`).
5. **Comparison Sheet (`<stem>-sheet.png`):**
   - Render in headless Chromium (Playwright) and visually inspect against the target references in `refs/louis/` and `IMG_7329.jpeg` before sign-off.
   - Include: target manuscript/woodcut references, `v1` baseline, new SVG, 4 mineral+monochrome `--wash`/`--wash2` variants, and 4 SMIL motion timestamps.
6. **Page & Attribution Rules:**
   - Work only within your owned files under `docs/art/bestiary/`.
   - Creatures touch their bubbles; burrowing vanishes cleanly with zero fade and zero drawn hole; nothing scrolls sideways at `390 px`; animations run only while on screen (`crawler.js`).
   - **Zero AI attribution** anywhere in any file or commit.
