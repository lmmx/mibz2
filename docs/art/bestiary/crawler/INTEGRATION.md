# Page Integration Guide

This note maps each creature and ornament slot on the conversation page to the exact drop-in calls from `docs/art/bestiary/crawler/crawler.js`, the standalone creature SVGs in `docs/art/bestiary/*.svg`, and the manuscript ornament symbols in `docs/art/bestiary/ornaments.svg`.

Include `crawler.js` before `vine.js` (or inline it in `build_html.py`):

```html
<script src="art/bestiary/crawler/crawler.js"></script>
```

---

## 1. Headline bubble frame ("Of Man's Labours & their Eternal Return")

Heraldic dual-serpent border in saturated Claude orange (`#F2683A`) with `claude-ruffed` heads. Using `side: 'frame'` wraps the left, top, and right edges tightly against the bubble box while leaving the bottom edge open so the illuminated gold caption stays uncovered.

```js
BestiaryCrawler.drape(headlineBubbleEl, {
  creature: 'aspis',
  register: 'illuminated',
  side: 'frame',
  wash: '#F2683A',
  wash2: '#d8342a',
  headVariant: 'claude-ruffed',
  fleurons: true,
  scale: 0.9
});
```

---

## 2. The four burrowers (crawling along phrases into `o` / `i` holes)

Replace the old clip/fade burrowers in `vine.js` with `makeCrawler`. Measure the exact glyph box of the target letter (`o` counter or `i` dot) and pass `{ x, y, r }` as `hole` — do not draw a fake hole circle.

```js
// Standard horizontal / curved phrase burrower
const crawler = BestiaryCrawler.makeCrawler({
  path: phrasePathD,                // ends at (hole.x, hole.y)
  hole: { x: hx, y: hy, r: hr },    // counter of 'o' or dot of 'i'
  creature: 'draco-bipes',          // or 'aspis' | 'spotted-serpent' | 'amphisbaena'
  register: 'engraved',             // or 'illuminated'
  wash: '#8b6d3f',
  scale: fontSize / 16,
  dur: 14
});
overlaySvg.appendChild(crawler);
```

For the **upward burrower behind the text**, place its container `<svg>` at `z-index: 0` behind the text span (`position: relative; z-index: 1`) and pass an upward-curving `path` terminating at the upper letter hole:

```js
const upward = BestiaryCrawler.makeCrawler({
  path: upwardPathD,
  hole: { x: hx, y: hy, r: hr },
  creature: 'aspis',
  register: 'engraved',
  wash: '#1d3f9e',
  scale: fontSize / 16,
  motion: 'undulate',
  dur: 15.3
});
behindTextSvg.appendChild(upward);
```

---

## 3. The moonwalker (`MOONWALK` bubble)

Replaces the rigid border glide with a full-circuit moonwalk (sliding stance foot, lifting/rolling heel, step bob, corner head-turns, and top-right pause). Automatically adapts to any bubble width/height and stays attached at `390px` phone width without horizontal overflow (`overflow: visible` inside a clipped parent wrapper or `pad <= page-gutter`).

```js
BestiaryCrawler.makeMoonwalker(moonwalkBubbleEl, {
  creature: 'draco-bipes',
  register: 'engraved',
  wash: '#8b6d3f',
  scale: 0.82,
  dur: 16
});
```

---

## 4. The hanging dragons (3Blue1Brown blue & brown)

Two winged wyverns (`draco-alatus.svg`, or `drape()` with `side: 'bottom'`) suspended directly beneath their bubble in 3Blue1Brown's blue (`#3b83d6`) and brown (`#7a4a26`). When inlining `draco-alatus.svg`, set `--wash` on the wrapper element (never on the root `<svg>`):

```html
<div class="hanging-dragons">
  <span class="draco-slot" style="--wash:#3b83d6"><!-- inline draco-alatus.svg --></span>
  <span class="draco-slot mirrored" style="--wash:#7a4a26"><!-- inline draco-alatus.svg (flipped X) --></span>
</div>
```

Keep the container flush against the bubble's bottom border (`margin-top: -6px`) so both wyverns touch their bubble and never float free.

---

## 5. The knotted pair (under the same line)

Inline `docs/art/bestiary/knotted-pair.svg` beneath the line with CSS variables controlling both intertwined bodies:

```html
<div class="knotted-slot" style="--wash:#d8342a; --wash2:#1d3f9e;">
  <!-- inline knotted-pair.svg -->
</div>
```

---

## 6. "yuck" — the deliberate cute odd one out

Keep the final framed bubble small, round, and endearing (`creature: 'cute'`), framing the red blackletter `"yuck"` text without overcrowding it:

```js
BestiaryCrawler.drape(yuckBubbleEl, {
  creature: 'cute',
  register: 'illuminated',
  side: 'loop',
  wash: '#d8342a',
  texture: false,
  scale: 0.78
});
```

---

## 7. Ornaments: flourishes and fleurons (`docs/art/bestiary/ornaments.svg`)

Include `docs/art/bestiary/ornaments.svg` once (hidden `<svg style="display:none">`) in `build_html.py`, then reference its green-and-blue manuscript symbols anywhere on the page:

- **Heading flourishes either side of "Compendium of Links" (`svg.flo` replacement):**
  ```html
  <svg class="flo flo-left" viewBox="0 0 140 36" aria-hidden="true">
    <use href="#ornament-rinceau-left"/>
  </svg>
  <h2>Compendium of Links</h2>
  <svg class="flo flo-right" viewBox="0 0 140 36" aria-hidden="true">
    <use href="#ornament-rinceau-right"/>
  </svg>
  ```
- **Rule centerpiece (`❦` replacement):**
  ```html
  <svg class="rule-ornament" viewBox="0 0 44 36" width="24" height="20" aria-hidden="true">
    <use href="#ornament-aldus-leaf"/>
  </svg>
  ```
- **Fleurons flanking linked terms (`❧` and `☙` replacements):**
  ```html
  <svg class="fleuron" viewBox="0 0 36 22" width="18" height="11" aria-hidden="true">
    <use href="#ornament-fleuron-left"/>
  </svg>
  <a href="...">Linked Term</a>
  <svg class="fleuron" viewBox="0 0 36 22" width="18" height="11" aria-hidden="true">
    <use href="#ornament-fleuron-right"/>
  </svg>
  ```

---

## 8. Mobile (`390px`) & palette checklist

- **No horizontal scroll at 390px:** give the outer conversation column `overflow-x: clip` (preferred over `hidden` so sticky headers keep working) and keep `drape`/`moonwalker` `scale <= 0.85` on viewports `< 480px`.
- **Palette:** saturated medieval pigments & neon accents (`#F2683A`, `#3b83d6`, `#7a4a26`, `#d8342a`, `#1d3f9e`, `#2f7d4a`), never pastel, oxidised, or pink. Fleurons stay green (`#2f7d4a` / `#3f9a7a`), links stay blue (`#1d3f9e` / `#2563eb`).
- **Contact:** keep every `drape()` and `makeMoonwalker()` overlay attached inside its bubble (`attach: true`) so creatures hug the border on every screen width.
