# Bestiary Crawler (`crawler/crawler.js`)

Zero-dependency SMIL SVG motion rig combining rich historical mineral tempera with monochrome woodcut detail (after 12th–14th century illuminated manuscripts and *De Dracone*):
- **Decisively stroked iron-gall contours (`#1c100a`, ~`1.2–1.8px`)** enclosing layered mineral pigments (`#c84416` cinnabar, `#244594` mineral ultramarine, `#5e2250` Romanesque wine-violet, `#3d705a` verdigris-sage, `#3e453b` smoky olive-charcoal, `#cf941c` ochre-gold, `#efe4cc` parchment ivory — zero neon primaries, zero pink).
- **Rich texture instead of flat blocks:** longitudinal multi-band gouache modelling (ivory outer edge + mineral core + deep inner shadow + single row of tiny lead-white seam dots), fine parallel brushwork along necks, flanks, and wings, and monochrome woodcut scutes/ocelli (`register: 'woodcut'`).
- **Six unified single-timeline travelling rigs** (no static sitting poses) plus automatic **`IntersectionObserver` on-screen-only pausing** for mobile performance.
- **Exact 0-stray-pixel burrow contract** into letter counters (`o`) and the dot of `i`.

Include via `<script src="crawler/crawler.js"></script>` (exposes `window.BestiaryCrawler`) or CommonJS `require('./crawler.js')`.

---

## Travelling Motion Engines

### 1. `makeMoonwalker(bubbleEl, opts) -> SVGSVGElement`
Full clockwise rounded-rect border circuit around `bubbleEl`, never leaving contact at any width (including `390px`). Features a textured border-drollery dragon (cinnabar head & S-neck stroked with golden-ochre brushwork transitioning into smoky olive-charcoal finely ribbed body & tail), sliding stance claw, lifting heel roll, and a poised top-right corner pause with sideways glance & forked tongue flick.

```js
BestiaryCrawler.makeMoonwalker(bubbleEl, {
  creature: 'draco-bipes',
  wash: '#3e453b',     // smoky olive-charcoal body
  headWash: '#c84416', // cinnabar head & neck
  wash2: '#e6c068',    // golden-ochre brushwork highlight
  scale: 0.94,
  dur: 14,
  autoPause: true      // default true: pauses SMIL when scrolled off-screen
});
```

### 2. `makeFlightLoop(bubbleEl, opts) -> SVGSVGElement`
Winged wyvern (or staggered multi-dragon swarm via `count: 3`) with tri-colour banded Romanesque feathered wings launching off the bubble's corner, swooping in a figure-eight around the message with steady wingbeats, performing a graceful banking roll & alert glance at the reader at the apex, and landing back touching the bubble.

```js
BestiaryCrawler.makeFlightLoop(bubbleEl, {
  count: 3,            // 1 = solo wyvern, 3 = staggered 3b1b swarm
  scale: 0.86,
  dur: 11.5,
  showTrail: false,
  autoPause: true
});
```

### 3. `makeCrawler(opts) -> SVGGElement`
Single-timeline text-weaving burrower along any SVG path `d` with optional `startHole` (seamless emergence), poised pause & alert look, courtly bow before diving, sinuous tail curl flourish, and the **0-stray-pixel shifted circular hole `<mask>`** (`hole: { x, y, r }`).

```js
const g = BestiaryCrawler.makeCrawler({
  path: phrasePathD,                // ends at (hole.x, hole.y)
  hole: { x: hx, y: hy, r: hr },    // exact letter counter or i-dot
  startHole: null,                  // optional { x, y, r } for masked emergence
  creature: 'aspis',                // 'draco-bipes' | 'draco-alatus' | 'aspis' | 'spotted-serpent' | 'amphisbaena' | ...
  wash: '#e82914',
  headWash: '#e82914',
  scale: fontSize / 16,
  dur: 12.5
});
```

#### Burrow Contract Verification
1. **One timeline:** every body band, dorsal dot/ring layer, head `animateMotion`, legs, wings, rear head, and tail curl share the same normalized storyboard `u(t)` and `dur`.
2. **Explicit hole mask:** `<mask maskUnits="userSpaceOnUse">` carries a full-plane white rect and a black disc of radius `R_mask = max(hole.r, maxHalfExtent + 5.6)` shifted forward along the entry tangent by `R_mask - hole.r`. The near rim coincides with `{x, y, r}`, while the post-hole path spirals tightly inside `R_mask` for `>= 1.85 × bodyLen`.
3. **Zero residue:** zero drawn hole, zero fade, zero stray pixels after tail entry. Verify any species with:

```bash
python3 docs/art/bestiary/crawler/render_proof.py draco-bipes
```

### 4. `makeKnottedSlider(bubbleEl, opts) -> SVGSVGElement`
Twin vermilion & lapis serpents in an over-under Insular zoomorphic interlace knot touching `bubbleEl`'s bottom edge. The knot visibly tightens and loosens as both serpents slide through snapping at each other's tails, then glides smoothly along the bottom edge and back.

```js
BestiaryCrawler.makeKnottedSlider(bubbleEl, {
  wash: '#e82914',
  wash2: '#1e4cff',
  scale: 0.92,
  dur: 13
});
```

### 5. `makeHeadlineChase(bubbleEl, opts) -> SVGSVGElement`
Two neon Claude-orange (`#ff5a22` / `#F2683A`) Claude-ruffed serpents chasing each other round the headline bubble border, meeting head-to-head at bottom center to snap & recoil with an 8-ray illuminated gold star flash, while hugging the frame tightly so the burnished-gold caption below stays uncovered at desktop and `390px` phone widths.

```js
BestiaryCrawler.makeHeadlineChase(headlineBubbleEl, {
  scale: 0.86,
  dur: 11
});
```

### 6. `makeYuckBounce(bubbleEl, opts) -> SVGSVGElement`
The deliberate cute odd-one-out: small, round, endearing wind-up creature with brass wind-up key and coiled spring-tail hopping merrily round the small red blackletter `"yuck"` bubble without obscuring the lettering.

```js
BestiaryCrawler.makeYuckBounce(yuckBubbleEl, {
  scale: 0.90,
  dur: 8.5
});
```

### 7. `observeVisibility(svgEl, opts)` & `drape(bubbleEl, opts)`
- **`observeVisibility(svgEl)`**: attaches an `IntersectionObserver` calling `svgEl.pauseAnimations()` whenever the SVG leaves the viewport and `svgEl.unpauseAnimations()` on re-entry (respecting `svgEl.dataset.bestiaryManualPause === '1'`). Enabled by default (`autoPause: true`) on all bubble rig builders; pass `autoPause: false` for headless screenshot scripts.
- **`drape(bubbleEl, opts)`**: compatibility router forwarding cleanly to `makeHeadlineChase`, `makeYuckBounce`, `makeKnottedSlider`, `makeFlightLoop`, or `makeMoonwalker`.

---

## Animated Motion Proofs

Open `docs/art/bestiary/proofs/index.html` to inspect every motion cycle interactively with Play/Pause, timeline scrubbing, speed control, phase jump buttons, and a `390px` phone preview toggle:
- `proofs/proof-draco-bipes.html`
- `proofs/proof-draco-alatus.html`
- `proofs/proof-burrowers.html`
- `proofs/proof-knotted-pair.html`
- `proofs/proof-headline-yuck.html`

---

## Mobile & Phone Performance

All motion runs purely via native SMIL `<animate>`, `<animateTransform>`, and `<animateMotion>` with zero `requestAnimationFrame` loops, zero SVG filters on animated nodes, and automatic `IntersectionObserver` pausing off-screen. Keep conversation containers at `overflow-x: clip` and `scale <= 0.82` on viewports `<= 420px` for guaranteed zero horizontal scroll at `390px`.
