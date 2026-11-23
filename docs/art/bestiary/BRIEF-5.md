# Bestiary brief 5: finish and ship

Ship the creatures for the post "Decompression is the harder problem" now. This brief replaces
BRIEF-4's open-ended plan with a finite job: a fixed list of slots, each finished to the same
standard, all in one handover. No more tracing of new specimens.

**Be economical, and ship as fast as possible.**
- **One agent, no teams:** work alone, with no `/teamwork`, no `/swarm` and no subagents.
- **Reuse:** start from the existing knackering-frame code and `crawler.js`, not from scratch.
- **No polishing:** don't build pipelines, metrics or sheets beyond what a slot needs, and don't
  optimise past "good and working".
- **Push early:** push each slot as it works, then the demo.

## The look

**Bodies:** the bodies that move are drawn with the original procedural approach, the one that
made the serpents in the "managing Claudes all day is knackering" frame.
- It gives thick tubes with a bold dark outline, a lighter belly band, a lead-white dot rail and
  rib bands.
- Those bodies bend and writhe well, so redraw bodies this way wherever a creature has to move.

**Heads, wings, feet:** taken from the traced specimens in `traced/components/`, cleaned up into
bold, flat, confident shapes, and redrawn where the trace is noisy.
- Each sits on its body at the anatomical joint, with its own small motion: jaw, blink, turn,
  wing beat.

**Colour:**
- Neon, saturated, no pink.
- Recolourable through `--wash` and `--wash2`.

**Outline:** one continuous dark outline round each creature. No hatching, no specks, no white
cracks.

## The slots

Every slot must travel or act, never just sit.

| Slot | Creature | Motion |
|---|---|---|
| **Headline frame**, round "managing Claudes all day is knackering" | the existing frame serpents' bodies, with traced heads (the Romanesque S-initial's horned head, the roundel's head) | They chase round the frame, then meet, snap and recoil at the bottom. The gold caption "Of Man's Labours & their Eternal Return" stays below the frame, uncovered. |
| **Burrowers** (four) | knackering-style bodies, with the Aberdeen serpent's and the calendar serpent's traced heads | They crawl along the text, weaving, pause to look around, then dive into the "o" or "i" through the masked hole, with no stray pixels. Then they re-emerge. |
| **Knotted pair** (three places) | two knackering-style bodies, with the Gothic interlace's dragon heads | The knot tightens and loosens, then slithers along the bubble's lower edge. |
| **Moonwalker** | the border-drollery dragon: traced head and wing on a redrawn body and legs | A full circuit of its bubble on `makeMoonwalker`. |
| **Swarm** (gold, by the "agentic development" line) | small vipers: redrawn bodies with traced viper heads | They orbit and dart round the bubble. |
| **"yuck"** | one small, round, cute viper | It bounces round the bubble like a wind-up toy. |
| **Perched roundel** | done (`traced/romanesque-roundel-animated.svg`, merged into the post) | No change. |

## The rules

- Every creature touches its bubble.
- Vanishing is clean: no fade and no drawn hole.
- Nothing scrolls sideways at 390 px.
- Each creature is 25 KB gzipped or less.
- Each creature runs only while it's on screen.

## The handover (one push, all slots)

1. **The files:** for each slot, one self-contained file under `docs/art/bestiary/ship/`, either:
   - an SVG with its SMIL, for static slots; or
   - a call to `crawler.js` with its options, for travelling slots.
   Plus the final `crawler.js`.
2. **The demo:** `docs/art/bestiary/ship/demo.html`, a mock of the conversation with every slot in
   place, working at desktop width and at 390 px, in light and dark mode. The writer copies from
   this into the post.
3. **The list:** `docs/art/bestiary/ship/HANDOFF.md`. For each slot it gives the file, its size,
   which message it attaches to, and the exact call.

Push to `origin/art/bestiary-serpents` as soon as `demo.html` works. Work only under
`docs/art/bestiary/`. Commit as Louis Maddox <louismmx@gmail.com>, with no AI attribution.
