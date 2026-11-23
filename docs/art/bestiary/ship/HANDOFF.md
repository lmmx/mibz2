# Bestiary Ship Handoff (Brief 5)

Drop-in handover for the post **"Decompression is the harder problem"**.
Open [`demo.html`](./demo.html) to preview all seven slots together at desktop width and `390 px`, in light and dark mode.

## Shared Engine

Include [`crawler.js`](./crawler.js) once before any slot script:

```html
<script src="art/bestiary/ship/crawler.js"></script>
```

- **File:** `docs/art/bestiary/ship/crawler.js` (`120.0 KB` raw, `27.4 KB` gzipped — shared engine for all 6 travelling slots)
- **Recolouring:** every creature reads `--wash` and `--wash2` CSS custom properties (neon, saturated, zero pink).
- **On-screen only:** every rig attaches an `IntersectionObserver` (`autoPause: true` by default) calling `pauseAnimations()` off-screen and `unpauseAnimations()` on-screen.
- **Mobile (`390 px`):** keep `html, body` and the conversation column at `overflow-x: clip` with at least `24 px` horizontal padding.

---

## Slots

| Slot | File | Size (raw / gz) | Target message | Exact call |
|---|---|---|---|---|
| **1. Headline frame** | `docs/art/bestiary/ship/headline-frame.js` | `1.0 KB` / `0.59 KB` | *"managing Claudes all day is knackering"* (with gold caption *"Of Man's Labours & their Eternal Return"* below, uncovered) | `BestiarySlots.mountHeadlineFrame(headlineBubbleEl)` |
| **2. Burrowers (×4)** | `docs/art/bestiary/ship/burrowers.js` | `4.6 KB` / `1.70 KB` | Four inline phrases weaving into an `"o"` counter or `"i"` dot (2 Aberdeen heads, 2 Calendar heads; 0-stray-pixel masked dive + re-emergence) | `BestiarySlots.mountBurrower({ svgEl, phrase, startCharIndex, charIndex, kind: 'o' \| 'idot', presetIndex: 0..3 })` |
| **3. Knotted pair (×3)** | `docs/art/bestiary/ship/knotted-pair.js` | `1.4 KB` / `0.72 KB` | *"this angle got a lot of attention"* plus two more conversation bubbles (Gothic interlace dragon heads; knot tightens/loosens and slithers along lower edge) | `BestiarySlots.mountKnottedPair(bubbleEl, 0)` (presets `0`, `1`, `2`) |
| **4. Moonwalker** | `docs/art/bestiary/ship/moonwalker.js` | `1.0 KB` / `0.55 KB` | `MOONWALK` bubble (border-drollery head + beating wing on redrawn knackering body & legs; full perimeter circuit) | `BestiarySlots.mountMoonwalker(moonwalkBubbleEl)` |
| **5. Swarm (gold)** | `docs/art/bestiary/ship/swarm.js` | `0.85 KB` / `0.49 KB` | *"agentic development"* line bubble (3 small gold vipers orbiting & darting round the bubble) | `BestiarySlots.mountSwarm(agenticBubbleEl)` |
| **6. "yuck"** | `docs/art/bestiary/ship/yuck.js` | `0.78 KB` / `0.45 KB` | Red blackletter *"yuck"* bubble (one small, round, cute wind-up viper bouncing round the bubble) | `BestiarySlots.mountYuck(yuckBubbleEl)` |
| **7. Perched roundel** | `docs/art/bestiary/traced/romanesque-roundel-animated.svg` | `55.1 KB` / `20.5 KB` | *"(i.e. you might be able to design a compression format…)"* — already merged into the post, no change | `<object data="art/bestiary/traced/romanesque-roundel-animated.svg" type="image/svg+xml"></object>` |

---

## Copy-Paste Wiring (Matches `demo.html`)

```html
<script src="art/bestiary/ship/crawler.js"></script>
<script src="art/bestiary/ship/headline-frame.js"></script>
<script src="art/bestiary/ship/burrowers.js"></script>
<script src="art/bestiary/ship/knotted-pair.js"></script>
<script src="art/bestiary/ship/moonwalker.js"></script>
<script src="art/bestiary/ship/swarm.js"></script>
<script src="art/bestiary/ship/yuck.js"></script>
<script>
  // 1. Headline frame
  BestiarySlots.mountHeadlineFrame(document.getElementById('slot-headline'));

  // 2. Burrowers (four: 0 & 2 = Aberdeen head -> 'o', 1 & 3 = Calendar head -> 'i' dot)
  BestiarySlots.mountBurrower({ svgEl: el1, phrase: 'decompression loops', startCharIndex: 3, charIndex: 15, kind: 'o', presetIndex: 0 });
  BestiarySlots.mountBurrower({ svgEl: el2, phrase: 'bitstream indexing',  startCharIndex: 1, charIndex: 15, kind: 'idot', presetIndex: 1 });
  BestiarySlots.mountBurrower({ svgEl: el3, phrase: 'blocks on the floor', startCharIndex: 2, charIndex: 16, kind: 'o', presetIndex: 2 });
  BestiarySlots.mountBurrower({ svgEl: el4, phrase: 'writing inline code', startCharIndex: 2, charIndex: 11, kind: 'idot', presetIndex: 3 });

  // 3. Knotted pair (three places)
  BestiarySlots.mountKnottedPair(document.getElementById('slot-knotted-1'), 0);
  BestiarySlots.mountKnottedPair(document.getElementById('slot-knotted-2'), 1);
  BestiarySlots.mountKnottedPair(document.getElementById('slot-knotted-3'), 2);

  // 4. Moonwalker
  BestiarySlots.mountMoonwalker(document.getElementById('slot-moonwalker'));

  // 5. Gold Swarm
  BestiarySlots.mountSwarm(document.getElementById('slot-swarm'));

  // 6. "yuck"
  BestiarySlots.mountYuck(document.getElementById('slot-yuck'));
</script>
```
