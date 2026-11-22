# Bestiary Crawler (`crawler/crawler.js`)

Zero-dependency SMIL SVG rigging library for medieval serpents, wyverns, and basilisks:
- **`makeCrawler(opts)`**: single-timeline crawling along any SVG path with a width-aware hole-mask burrow contract.
- **`drape(bubbleEl, opts)`**: smooth border draping around any rounded rectangle with tangent-aligned scales, ornament, and optional manuscript fleurons.
- **`makeMoonwalker(bubbleEl, opts)`**: border-hugging moonwalk circuit with sliding stance foot, lifting/rolling heel, body bob, corner head-turns, and a corner pause.
- **`buildHead(type, opts)`**: reusable SVG head builder (`wyvern`, `basilisk`, `serpent`, `claude-ruffed`, `cute`).

Include via `<script src="crawler/crawler.js"></script>` (exposes `window.BestiaryCrawler`) or CommonJS `require('./crawler.js')`.

---

## API

### `makeCrawler(opts) -> SVGGElement`

```js
const g = BestiaryCrawler.makeCrawler({
  path,        // SVG path `d` string ending at (or pointing into) the hole
  hole,        // { x, y, r } counter or dot centre & radius; omit for open routes
  creature,    // species slug (table below)
  register,    // 'engraved' (default) | 'illuminated'
  headVariant, // optional override: 'claude-ruffed' | 'wyvern' | 'basilisk' | 'serpent' | 'cute'
  wash,        // primary colour (translucent wash in engraved, saturated pigment in illuminated)
  wash2,       // secondary colour (knotted-pair second body, amphisbaena rear head, shadow accent)
  scale,       // 1 = default size for 16px text
  storyboard,  // [{ t: 0..1, u: 0..1 }] unified timeline keyframes (u=1 clears tail past hole)
  looks,       // [{ t: 0..1, deg }] inquisitive head turns
  flicks,      // [{ t0, t1 }] forked tongue flick intervals
  shed,        // boolean or { u } optional synced shed-skin exuvia (clears before tail entry)
  motion,      // 'auto' | 'undulate' | 'concertina' | 'gait'
  dur          // cycle duration in seconds (default 14)
});
```

#### Burrow contract
1. **One timeline:** every body band, dorsal hatch/dot layer, head `animateMotion`, leg pair, wing, rear head, and shed exuvia reads the same normalized storyboard `u(t)` and shared `dur`.
2. **Explicit hole mask:** a `<mask maskUnits="userSpaceOnUse">` carries a white full-plane rect and a black disc of radius `R_mask = max(hole.r, maxHalfExtent + 4.1)` shifted forward along the entry tangent by `R_mask - hole.r`. Its near rim coincides with the exact rim of `{x, y, r}` (no premature clipping), while the post-hole path spirals tightly inside `R_mask` for `>= 1.65 × bodyLen`.
3. **Zero residue:** once the tail crosses `(hole.x, hole.y)`, every pixel of the creature (including wide wings, ruffs, and claws) lies inside the black mask disc. No fade, scale-to-zero, or rectangle clip is used.

Verify any species with:

```bash
python3 docs/art/bestiary/crawler/render_proof.py draco-bipes engraved
```

---

### `drape(bubbleEl, opts) -> SVGSVGElement`

```js
BestiaryCrawler.drape(bubbleEl, {
  creature: 'aspis',
  register: 'illuminated',     // 'engraved' | 'illuminated'
  side: 'top',                 // 'top' | 'bottom' | 'left' | 'right' | 'loop' | 'frame'
  wash: '#F2683A',
  wash2: '#d8342a',
  headVariant: 'claude-ruffed',
  fleurons: true,              // adds green/blue manuscript leaf fleurons at tail ends
  texture: true,               // lightweight vellum/pigment mottle filter in illuminated register
  scale: 1,
  attach: true                 // appends overlay SVG inside bubbleEl (sets position:relative if static)
});
```

`bubbleEl` may be a live DOM element or a plain `{ width, height, radius }` object. Sharp rounded-rect corners are sampled and smoothed into cubic splines so offset ribbons never kink.

---

### `makeMoonwalker(bubbleEl, opts) -> SVGSVGElement`

```js
BestiaryCrawler.makeMoonwalker(bubbleEl, {
  creature: 'draco-bipes',
  register: 'engraved',
  wash: '#8b6d3f',
  scale: 0.85,
  dur: 16,
  attach: true
});
```

Computes the exact perimeter and corner arc fractions of `bubbleEl` so the creature stays tangent to the border on any bubble width or height, pauses briefly at the top-right corner to look around and flick its tongue, and drives both legs with a sliding-stance / lifted-heel moonwalk cycle synchronized with a subtle step bob.

---

## Creatures & Options

| Slug | Head archetype | Limbs / Wings | Default motion | Default `engraved` wash | Default `illuminated` pigment |
|---|---|---|---|---|---|
| `draco-bipes` | `wyvern` | 2 clawed legs | `gait` | `#8b6d3f` sepia | `#d8342a` vermilion |
| `draco-alatus` (`wyvern`) | `wyvern` | 2 legs + bat wing | `gait` | `#3b83d6` blue | `#1d3f9e` lapis |
| `aspis` (`biscione`) | `serpent` (crowned) | legless | `undulate` | `#a3141c` carmine | `#1d3f9e` lapis |
| `spotted-serpent` (`basilisk`) | `basilisk` (cockerel) | 2 bird claws + wing | `gait` | `#9c4c1e` ochre | `#3f9a7a` verdigris |
| `basilisk-6` | `basilisk` (cockerel) | 6 legs | `gait` | `#7a5a2c` sepia | `#d8342a` vermilion |
| `knotted-pair` | `serpent` (twin) | legless (2 bodies) | `undulate` | `#d22f1c` / `#1d3f9e` | `#d8342a` / `#1d3f9e` |
| `ouroboros` | `wyvern` | 2 forelegs + wing | `concertina` | `#7a5c32` bronze | `#3f9a7a` verdigris |
| `amphisbaena` | `serpent` (both ends) | legless | `concertina` | `#8c3b28` madder | `#d8342a` / `#3f9a7a` |
| `cute` | `cute` (wendling) | legless | `undulate` | `#d8342a` red | `#d8342a` red |

**Head variants:** pass `headVariant: 'claude-ruffed'` on any creature to mount an 18-petal uneven Claude-orange (`#F2683A`) sunburst ruff with modulated iron-gall ink outlines and a tiny hidden `^ω^` motif in negative space at the nape.

---

## Mobile & iPhone Safari Performance Cost

All motion runs purely on SMIL `<animate>`, `<animateTransform>`, and `<animateMotion>` — zero `requestAnimationFrame` loops and zero per-frame path `d` morphing.

| Feature / Motion | Animated primitives | Relative GPU/CPU cost on iPhone Safari | Notes |
|---|---|---|---|
| `undulate` (spatial zero-skate wave) | ~38 `stroke-dashoffset` + 1 head `animateMotion` | **Low** | Sine track is baked into the spine curve once at build time; zero path-morph cost per frame. |
| `concertina` (bunch & stretch) | ~38 `stroke-dashoffset` + 1–2 `animateMotion` | **Low** | Segment spacing shifts smoothly in precomputed `stroke-dashoffset` keyframes. |
| `gait` (2-legged / 6-legged) | Body + 2–3 leg `animateMotion` + 4–6 joint `rotate` | **Low–Moderate** | Transforms only; pauses rest statically. |
| `wings` | +1 wing `animateMotion` + 1 shoulder `rotate` | **Very low** | Single nested group transform. |
| `drape()` resting pose | 1 slow breathing `scale` + 1 head `rotate` + tongue | **Minimal** | Static vector body with 3 tiny transforms; safe to place on many bubbles. |
| `makeMoonwalker()` | 1 `animateMotion` + 1 bob `translate` + 6 leg/head transforms | **Low** | Compact single-actor rig; ~12 SVG paths total. |
| `illuminated` `texture: true` (`feTurbulence`) | Static filter evaluated on static `drape()` only | **Low** (pass `texture: false` on small phones if many bubbles use it) | Never animated; `makeCrawler()` keeps filters off animated groups for 60 fps scrolling. |
| Burrow `<mask>` | 1 static `<circle>` mask per crawler | **Low** | Mask geometry never animates; only masked children move. |
