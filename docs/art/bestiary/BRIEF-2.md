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

## 4. The owner's further direction: basilisks, heraldry, gold

The owner's words: the animations are "simplistic"; the creatures are "often not well proportioned
for snakes"; the heads "look a bit like dogs and a bit crude"; it should be "quite proper and
illuminated manuscript medieval vibe", "less simply slithering around in a silly way and more
crestlike coat of arms like astride", with "gold patterning and medieval colouring and texture".

New references in `refs/`:
- `basilisk-aldrovandi.jpg`: Aldrovandi's basilisk woodcut: a crowned cockerel head with a hooked
  beak, a scaled humped body, six legs, a tail ending in a spiral curl.
- `basilisk-gheeraerts-weasel.jpg`: Marcus Gheeraerts's basilisk and the weasel: a cockerel's comb,
  wattles and beak, feathered breast, wing, a long scaled serpent tail, a bird's clawed feet.
- `visconti-biscione-milan.jpg`: the Visconti biscione in relief: a heavy serpent in tight,
  regular S-coils filling a shield, crowned head, a man in its jaws. Coiling as *composition*.
- `ouroboros-engraving.jpg`: the winged ouroboros (Lambsprinck): a closed ring of scaled body,
  small horned head biting the tail, membranous wings, clawed forelegs.
- `marlborough-arms-wyvern-supporters.png`: wyverns as heraldic supporters: rampant, astride a
  scroll, wings displayed, tails knotted, flanking a central device.
- `aberdeen-bestiary-initial-gold.jpg`: a bestiary initial: a two-tone serpent wound round a
  column on burnished gold, in lapis, vermilion and verdigris, framed by bands of white-dot and
  zigzag ornament.
- Read also: bestiary.ca, beast source 264 (https://bestiary.ca/beasts/beastsource264.htm), and
  https://en.wikipedia.org/wiki/Ouroboros (the alchemical and gnostic images on that page).

What follows from them:

1. **Heads are basilisk and wyvern heads, never dog heads.** No muzzle with a round nose and jowls.
   Build heads from: a hooked or straight beak, a cockerel's comb or a small crown, a wattle, a
   brow ridge over a small round eye, a long split tongue, and (for the drakes) swept-back horns
   and a finned crest down the neck. Give at least three distinct head types: basilisk (cockerel,
   crowned), wyvern (horned, finned, open jaw with a barbed tongue), serpent (biscione: a narrow
   wedge, crowned).
2. **Snake proportions.** A serpent's body is long: at least 12 to 20 head-lengths, tapering
   continuously from the neck to a fine point, with the widest point a third of the way back. No
   stubby or uniform bodies.
3. **Keep the diversity of the engraving.** Different species, not one creature recoloured:
   - a legless serpent (biscione);
   - a two-legged wyvern, wings displayed;
   - a basilisk: cockerel forequarters, serpent tail;
   - a six-legged crawling basilisk (Aldrovandi);
   - a winged ouroboros, which bites its own tail;
   - an amphisbaena (a head at each end) if it suits.
4. **Heraldic poses, not cartoon slithering.** Besides crawlers, give poses that sit still and
   *hold* a place on the page, as charges and supporters do:
   - **rampant / segreant:** rearing on its hind legs, wings displayed, as a supporter beside a
     bubble or caption;
   - **nowed:** the body tied in a knot;
   - **ondoyant (the biscione):** the body in regular vertical S-coils;
   - **ouroboros:** a closed ring, which can circle an avatar or a letter's counter;
   - **astride:** standing on a scroll, a rule or the top of a bubble.

   These "rest" poses breathe: a slow sway of the neck, the tongue flicking, the wings shifting,
   the tail curling. They don't run around.
5. **Illumination, gold and texture.** The page wants illuminated-manuscript colour:
   - **gold:** burnished gold leaf as ground or pattern. A gradient with a bright specular band,
     and a fine punched or stippled diaper over it;
   - **pigments:** ultramarine/lapis #1d3f9e, vermilion #d8342a, verdigris #3f9a7a, lead white,
     ochre. Flat, saturated, with a black outline (here the manuscript's outline is right; the
     engraving's hatching is the other register), and white highlight strokes along the body's
     curve;
   - **ornament:** white-dot rows, zigzag lines and scale patterns painted in white or gold over
     the colour, as in the Aberdeen initial;
   - **texture:** a subtle vellum grain or pigment mottle (an feTurbulence-based filter, cheap)
     so the colour isn't flat digital fill.

   Offer two registers, selectable per creature: `engraved` (ink line, hatching and wash, section
   1) and `illuminated` (pigment, gold, white ornament, this section). Both keep modulated line
   and good proportion.

## 5. Arbitrary paths: drape it like vinery

The rig in section 2 must take **any** path, not just a run to a hole:
- a path round a bubble's rounded rectangle (a closed loop, or an open arc over its top or down
  one side), so a serpent can *drape* a bubble the way the page's fleurons and vines do;
- the body must follow sharp-ish corners without kinking: sample the path, smooth it, and build the
  body as a ribbon of offset curves from a width profile (or as a stroke with dashes, where that
  is enough);
- markings, scales and ornament follow the path's tangent and the body's width at each point;
- an API for the page, e.g. `drape(bubbleEl, {creature, register, side: 'top'|'loop'|'left',
  fleurons: true})`, which reads the bubble's box and builds the path. It can combine with fleurons
  (❧ ☙) set at the ends or knots;
- a still pose along a path (the vine), and a crawling one (section 2), share the same rig.

## 6. Motion that isn't simplistic

Replace the rigid "slide along the path" with motion from the animals:
- **lateral undulation:** a travelling sine wave down the body, amplitude growing towards the tail,
  phase locked to the forward speed so the body doesn't skate (each point of the body passes
  through the same points on the ground);
- **concertina** or **rectilinear** creep for the heavy serpents: segments bunch and stretch;
- **legged gait:** the legs step in a diagonal sequence, timed to the body's progress, with a
  claw that plants and lifts, not a pendulum swing;
- **the head leads:** it pauses, rears, looks about and tastes the air, then moves; the body
  catches up;
- **the wings beat or settle** with the gait, slowly;
- ease every move, and give the creatures moments of stillness.
- **the moonwalker:** one creature walks a circuit round the edge of a bubble, feet on its
  border, along an SVG `animateMotion` path (`MOONWALK` in the page's `build_html.py`). Today it
  just glides round, body rigid. Make it a real walk: the feet plant and slide back against the
  direction of travel (the moonwalk illusion: the planted foot slides, the other heel lifts and
  rolls); the body bobs a little with each step; the head turns at the corners; it pauses
  briefly at one corner. It must stay attached to the bubble's edge at every frame, on any
  bubble size.

Keep it cheap: SMIL or CSS transforms on a modest number of points (this page runs on phones).
Say in the README what each motion costs.

## 7. Deliverables, revised

1. A plate, `plate.html` + `plate.png`: every species in both registers and at least two heraldic
   poses, beside the references, on one sheet.
2. `crawler/` as in section 2, with `drape()` from section 5 and the motion from section 6, plus
   its proof sheets.
3. A README for the page integrator: the API, each creature's options, what each motion costs.

Push after each step, so the owner can follow along.

## Order and method

1. Rework one creature (the *Draco bipes*, against its engraving), and commit it and a contact sheet
   beside the engraving.
2. Build `crawler/` with the *Draco bipes* as its first creature, and commit its proof sheets.
3. Then the other five creatures, one commit each, each with its contact sheet.

Same rules as before:
- push after each commit;
- commit as Louis Maddox <louismmx@gmail.com>, with no AI attribution;
- touch nothing outside this folder.
