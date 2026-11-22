# Review of the first two serpents (aspis.svg, spotted-serpent.svg)

Read this before the next creature. Reviewed at commit 85d4631ea, rendering each file in headless
Chromium at t = 0.4, 1.6, 2.9 and 4.3 s.

## Keep doing

- The look: flat colour in a strong iron-gall outline, cross-bands, lead-white back stroke and dots,
  the palette. The beast heads (ear, almond eye under a lid, teeth, red forked tongue) are right.
- The rigging: ids prefixed per file, `<path class="spine">` with `data-widths`, the head as a
  `<symbol>` with jaw, tongue, eye and lid groups. This is exactly what the page needs.
- One creature per commit, pushed each time.

## Fix, in these two and in every creature after them

1. **Necks pinch.** The Aspis goes from a fat coil to a thin, stringy neck over a few units, which
   reads as tubing, not anatomy. Taper the body into the neck gradually over one and a half to two
   head-lengths. Where it meets the head, the neck should be about 70% of the head's height. The
   calendar-page serpent's thick, muscular neck is the model.
2. **The idle motion is nearly invisible.** At t = 0.4, 1.6, 2.9 and 4.3 s the only clear change is
   the tongue. At 200 to 300 px on a phone it must be visible without staring:
   - the head looks round: ±12 to 18°, with holds of a second or two between turns;
   - the jaw opens 15 to 20° around each tongue flick;
   - the tail tip curls and uncurls by 15 to 25°;
   - the body breathes: about a 3 to 4% swell, eased;
   - the eye blinks occasionally (the lid closes for about 150 ms).
   Keep the rule on what may animate: transforms and opacity, plus `d` on small parts such as a
   tail tip or a tongue, never on the whole body.
3. **Heads are small for their bodies,** above all on the spotted serpent. The head should be about
   1.3 to 1.5 times the neck's width, and read clearly at 200 px.
4. **The spotted serpent's loop should be a knot,** as on the calendar page: the body passes over,
   then under, itself, with a break in the ink outline where it goes under.

## Order

Fix 1 to 4 in aspis.svg and spotted-serpent.svg first (one commit), then carry on: knotted pair,
draco bipes, draco alatus, amphisbaena. Render each one at four or five moments and check the
motion is visible before you commit it.

# Second review (at 939afae26: revised pair, knotted-pair.svg, draco-bipes.svg)

The revisions landed well: the spotted serpent's knot now goes over and under, the heads are larger,
and the head turns, jaws and tongues read at a glance. Two more, then carry on with draco alatus and
the amphisbaena:

5. **knotted-pair.svg reads as a chain of three loops,** like links or the letters "OOC", not two
   creatures. Make it two serpents knotted round each other in one large diagonal knot, as in the
   blue-ground reference: each body crosses the other several times with over-under breaks, unevenly,
   and each head bites at or turns toward the other's tail. Asymmetry is the point.
6. **draco-bipes.svg's legs bunch into one cluster under the belly.** Set the two legs apart (fore
   and hind, as on the engraving) with a clear stride, the claws splayed on the ground line, and let
   the walk cycle swing them visibly in alternation.
