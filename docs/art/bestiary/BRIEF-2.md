# Second brief: make the bestiary sophisticated, and build the vanishing in

Read this whole file, and look at every image in `refs/`, before you change anything.

- `refs/current-set.png`: our six creatures as they stand.
- `refs/engraving-four-dracones.jpg`: the standard to reach. These are the *Draco bipes apteros*,
  *Draco alatus*, *Figura ex Pareo* and *Draco Aethiopicus*.
- `refs/manuscript-1.jpg` to `-3.jpg`: the earlier manuscript references.
- `refs/thebes-claude-spark-character.jpg`: the Claude imagery to riff on (section 3).

The owner's verdict on the current set: next to the engraving, ours look "crude and cartoonish, vulgar
even".

## 1. What is wrong with ours, against the engraving

Fix each point in every creature.

1. **Line.** Ours use one heavy, uniform outline everywhere, like a cartoon sticker. The engraving
   never does this:
   - its lines swell and taper with the pen's pressure: thick on the shadow side, hairline on the lit
     side;
   - contours break off where light hits;
   - there is no closed, even outline round a flat fill.
2. **Form by hatching, not by flat fill.** The engraving models every body with hatching that follows
   the form:
   - parallel strokes curving round the cylinder of the body;
   - cross-hatching in the shadows;
   - stipple in the scales.

   Ours are flat colour fields with a few stripes and dots laid on top.
3. **Curves.** The engraving's bodies are long, continuous S-curves whose curvature changes
   smoothly, like a clothoid: a neck rising in one sweep, a tail ending in a whip-thin flourish.
   Ours:
   - kink and wobble;
   - coil in lumpy loops;
   - are constant-width tubes, like sausages.

   Every spine should be a smooth spline whose curvature varies gradually, with no corners and no
   wobble.
4. **Taper and proportion.** In the engraving:
   - the body is thickest at the shoulder or belly and tapers over a long length to a hair-fine tail;
   - the necks are long and thin;
   - the heads are small, about the width of the neck at its thickest, not three times it.

   Ours have oversized heads on stubby, uniform bodies.
5. **Anatomy.** In the engraving:
   - **scales:** dorsal scales are drawn as overlapping arcs and rows;
   - **belly:** a banded belly of ventral plates runs the length of the body;
   - **legs:** they have musculature, with a shoulder, an elbow and splayed clawed toes gripping the
     ground;
   - **wings:** they spring from a shoulder, with ribs radiating from a wrist and a scalloped, hatched
     membrane between them.

   Ours are simplified to shapes.
6. **Heads.** The engraving's heads are restrained and specific: a brow ridge, a modelled jaw, small
   eyes set under the brow, teeth and a tongue drawn finely. Ours are goggle-eyed and rubbery.
7. **Colour.** The engraving is iron-gall ink on parchment. Its colour, where there is any, is a thin
   wash laid over the hatching, never a saturated flat fill. Keep the hatching as the main carrier of
   form:
   - render each creature as ink line-work with hatching;
   - lay one translucent wash of its colour (the page recolours it: ochre, sepia, verdigris,
     vermilion, Claude orange #F2683A, 3Blue1Brown's blue #3b83d6 and brown #7a4a26) over it;
   - expose the wash as a single CSS variable, `--wash`, plus a second, `--wash2`, for a second body.

**Elegance is the test.** Put each creature beside the engraving at the same size. If it looks like a
sticker, it isn't done.

## 2. Build the vanishing in: the burrow contract

On the page, serpents crawl along a phrase and go into the hole of a letter: the counter of an o, the
dot of an i. So far that has been bolted on afterwards: a clip region, a scale to zero, an opacity
cut. The result is glitchy, with glimpses, sliced heads and parts left behind. Design it into the
rig instead.

Deliver `crawler/crawler.js`: a small library with no dependencies that builds an SMIL-animated SVG
crawler from:

```
makeCrawler({ path,            // the route, an SVG path d string, ending at the hole
              hole,            // {x, y, r}: the hole's centre and radius (the letter's counter)
              creature,        // 'aspis' | 'spotted' | 'draco-bipes' | ... (a body profile and a head)
              wash, wash2,     // colours
              scale,           // 1 = the default size for 16px text
              storyboard,      // [{t, u}] keyframes: time 0..1 against distance along the path 0..1+
              looks, flicks,   // head turns [{t, deg}], tongue flicks [{t0, t1}]
              dur })           // seconds per cycle
  -> SVGGElement
```

Its rules, all of which must hold:

1. **One timeline.** Every moving part reads the same storyboard `u(t)`, the distance travelled. None
   has a separate timeline that can drift. That includes:
   - the body's dash offset, its markings and hatching, and its wash;
   - the head's `animateMotion`;
   - the legs and wings;
   - the shed skin.
2. **The hole is a hole.** Model it explicitly:
   - **a mask** whose black region is the hole's disc;
   - **anything inside the disc**, at the point where the crawler goes in, is masked, so it is under
     the page;
   - **the path continues** past the hole's centre, into the page, for at least one body length.

   The crawler therefore goes in by travelling, never by fading, scaling or clipping to a rectangle.
3. **Nothing is left behind.** At the moment the tail passes the hole's centre:
   - every part of the creature is past the hole and masked;
   - nothing exists outside the disc: legs, ruffs, wings and husk flakes included.

   Then, and only then, the cycle may reset, with everything off the page.
4. **Width-aware.** The hole's mask disc must be at least as wide as the widest part that passes
   through it, including the head, a ruff or a wing.
5. **Prove it, with a contact sheet.** Supply `crawler/test.html`, which renders each creature
   crawling into:
   - an `o` at 16px;
   - an `o` at 48px;
   - the dot of an `i`.

   Add a script that screenshots 12 frames across the last 30% of the cycle. Commit the sheet as
   `crawler/proof-<creature>.png`. Every frame after the tail's entry must show no pixels of the
   creature outside the disc.

## 3. The Claude imagery, by thebes

The reference is `refs/thebes-claude-spark-character.jpg`. It is a character whose head is Claude's
spark rendered as a sunflower: about 18 long, rounded, slightly uneven petals in Claude orange, round
a white face with a kaomoji `^ω^`. It has a confident black outline and flat colour, and an
ASCII-art cat beside it.

We riff on it; we don't copy it. Where a creature belongs to Claude, its ruff or crest is a riff on
that petal burst:
- long, rounded rays of unequal length, wobbling slightly;
- orange `#F2683A`, with an ink outline modulated like the engraving's, not a uniform cartoon one.

Make one variant head, `claude-ruffed`, for any creature. Optionally, add a small kaomoji face
(`^ω^`) in negative space somewhere a sharp eye would find it.

## Order and method

1. Rework one creature (the *Draco bipes*, against its engraving), and commit it and a contact sheet
   beside the engraving.
2. Build `crawler/` with the *Draco bipes* as its first creature, and commit its proof sheets.
3. Then the other five creatures, one commit each, each with its contact sheet.

Same rules as before:
- push after each commit;
- commit as Louis Maddox <louismmx@gmail.com>, with no AI attribution;
- touch nothing outside this folder.
