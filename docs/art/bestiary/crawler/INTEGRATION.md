# Page Integration Guide (Brief 3)

Drop-in integration calls for `docs/art/bestiary/crawler/crawler.js` across the conversation page. Every creature travels on a single synchronized SMIL timeline and pauses automatically when scrolled off-screen (`autoPause: true` by default via `observeVisibility`).

Include once before initializing the rigs:

```html
<script src="art/bestiary/crawler/crawler.js"></script>
```

---

## 1. Headline Frame ("managing Claudes all day is knackering")

Two Claude-ruffed serpents in neon `#ff5a22` / `#F2683A` race around the headline bubble border, meet head-to-head at bottom center to snap & recoil, and stay tightly bound to the frame edge so the burnished-gold caption `"Of Man's Labours & their Eternal Return"` below stays 100% uncovered at desktop and `390px` phone widths.

```js
BestiaryCrawler.makeHeadlineChase(headlineBubbleEl, {
  scale: window.innerWidth <= 420 ? 0.72 : 0.86,
  dur: 11
});
```

Keep at least `20px–24px` vertical gap between the headline bubble's bottom border and the gold caption element below it.

---

## 2. Text-Weaving Burrowers (`o` counter and `i` dot)

Measure the exact glyph bounding box via `textEl.getExtentOfChar(idx)` and pass `{ x, y, r }` as `hole` (never draw a fake hole circle). Optionally pass `startHole` so the creature emerges cleanly from an earlier letter, and clip a foreground text copy over chosen word spans so the burrower visibly weaves OVER and UNDER words:

```js
const crawler = BestiaryCrawler.makeCrawler({
  path: weavePathD,                 // ends at (hole.x, hole.y)
  hole: { x: hx, y: hy, r: hr },    // exact counter of 'o' or dot of 'i'
  startHole: startHoleOrNull,       // optional clean masked emergence
  creature: 'aspis',                // or 'spotted-serpent' | 'amphisbaena' | 'draco-bipes'
  wash: '#e82914',
  scale: fontSize / 16,
  dur: 12.5
});
overlaySvg.appendChild(crawler);
BestiaryCrawler.observeVisibility(overlaySvg);
```

---

## 3. The Moonwalker (`MOONWALK` bubble)

Full clockwise circuit round the bubble perimeter with sliding stance claw, heel roll, poised top-right corner pause & sideways glance, and textured border-drollery colouring (cinnabar head/neck stroked with golden-ochre brushwork transitioning into smoky olive-charcoal finely ribbed body & tail). Stays attached to the bubble edge at every width.

```js
BestiaryCrawler.makeMoonwalker(moonwalkBubbleEl, {
  creature: 'draco-bipes',
  wash: '#3e453b',
  headWash: '#c84416',
  wash2: '#e6c068',
  scale: window.innerWidth <= 420 ? 0.80 : 0.96,
  dur: 14
});
```

---

## 4. Winged Dragons & 3Blue1Brown Swarm

Finely ribbed medieval winged wyverns launching off the bubble corner into a figure-eight swoop with apex banking glance and wingbeat perch back onto the bubble. Use `count: 1` for a solo wyvern or `count: 3` for the staggered swarm mixing mineral ultramarine (`#2b5ea8`), deep chestnut (`#7a411e`), and monochrome woodcut (`#efe4cc` / `#1c100a`):

```js
BestiaryCrawler.makeFlightLoop(threeB1bBubbleEl, {
  count: 3,
  scale: window.innerWidth <= 420 ? 0.74 : 0.90,
  dur: 11.5
});
```

---

## 5. Insular Knotted Pair

Twin deep mineral ultramarine (`#244594`) & Romanesque violet-wine (`#5e2250`) serpents modelled in 3-band gouache with tiny lead-white seam dots, attached directly to the bubble's bottom border. The over-under interlace knot visibly cinches tight and loosens as both serpents bite at each other's tails, then slithers smoothly along the bottom edge and back:

```js
BestiaryCrawler.makeKnottedSlider(knottedBubbleEl, {
  wash: '#244594',
  wash2: '#5e2250',
  scale: window.innerWidth <= 420 ? 0.80 : 0.96,
  dur: 13
});
```

---

## 6. "yuck" — The Deliberate Cute Odd-One-Out

Small, round, endearing wind-up creature with antique brass key and coiled spring-tail bouncing round the red blackletter `"yuck"` bubble without obscuring the letters:

```js
BestiaryCrawler.makeYuckBounce(yuckBubbleEl, {
  scale: window.innerWidth <= 420 ? 0.84 : 0.94,
  dur: 8.5
});
```

---

## 7. Mobile (`390px`), Palette & Performance Checklist

- **Zero horizontal scroll at 390px:** set `overflow-x: clip` on `html, body` and the conversation column, and give message containers at least `24px–34px` lateral gutter padding.
- **On-screen-only execution:** every bubble rig automatically registers with `BestiaryCrawler.observeVisibility(svg)` (`IntersectionObserver`), calling `pauseAnimations()` off-screen and `unpauseAnimations()` on-screen. Call `BestiaryCrawler.observeVisibility(svg)` on any custom inline `<svg>` hosting `makeCrawler()`.
- **Palette rules:** rich historical mineral tempera & monochrome woodcut (`#1c100a` iron-gall, `#c84416` cinnabar, `#244594` mineral ultramarine, `#5e2250` violet-wine, `#3d705a` verdigris-sage, `#3e453b` smoky olive-charcoal, `#cf941c` ochre-gold, `#efe4cc` parchment ivory), **zero pink**, antique verdigris fleurons (`#2f6e50`), blue links.
- **Proofs before integration:** verify every cycle visually in `docs/art/bestiary/proofs/index.html` and run `python3 docs/art/bestiary/crawler/render_proof.py draco-bipes` to confirm `PASS (0 stray px after tail entry)`.
