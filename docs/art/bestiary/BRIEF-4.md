# Bestiary brief 4: from traced specimens to creatures living on the post

Tracing was the means, not the goal. The goal is a set of **traced manuscript creatures that
move about the live blog post**, "Decompression is the harder problem", each in its place:
- perched;
- burrowing into letters;
- chasing round the headline frame;
- moonwalking round a bubble;
- knotting and slithering;
- bouncing round "yuck".

The traced specimens (`traced/*-master.svg`, `traced/components/`) are the raw material. Stop
when each **slot** on the page is done, not when each specimen is traced. BRIEF-3's look, motion,
flair and page rules all still hold.

## What "done" means for a slot

1. **The creature:** a cut-out of just that creature, separated from its source. That means no
   letter frame, no rubric text, no pillar or scroll unless they are part of the joke.
2. **The gaps:** gilding and ground that the tracer dropped are filled, never left as white holes.
3. **Colour:** set through `--wash` and `--wash2`, in neon pigment with no pink.
4. **Weight:** about 25 KB gzipped at most per creature.
5. **Movement:** it travels or acts in its slot as BRIEF-3 §"The motion" describes. It never
   just sits.
6. **The proof:** an animated proof page that plays the full cycle in a mock chat bubble at
   desktop width and at 390 px. Every frame shows the creature touching its bubble and vanishing
   cleanly into its hole, with nothing scrolling sideways.
7. **The handover:** listed in `HANDOFF-4.md` with its slot, file paths, sizes and proof page, so
   the writer can put it into the post without guessing.

## The slots, and what to build them from

| Slot on the post | Build from | How |
|---|---|---|
| **Perched dragon** (on "(i.e. you might be able to design a compression format…)") | `romanesque-roundel` | Done: in the post through PR #776. Add a small hop or turn of the head so it isn't static. |
| **Knotted pair** (under "this angle got a lot of attention", and two more places) | `gothic-initial-interlace` | Cut out the two winged dragons and their braid only. Drop the G, the rubric and the gold-ground holes. Animate the knot tightening and loosening, then slithering along the bubble's lower edge. |
| **Burrowers** (four, crawling along text into an "o" or "i") | heads, bodies and tails from `aberdeen-serpent-pillar`, `calendar-september-serpent` and `bestiary-vipers-miniature` | Graft the traced heads and tails onto `crawler.js`'s travelling spine and masked burrow, matching the body's stroke, fill and dot rail to the traced palette. Keep the zero-stray-pixel burrow. |
| **Headline frame** (round "managing Claudes all day is knackering") | the `calendar-september-serpent` body, with `romanesque-s-initial`'s horned head | Two serpents chase each other round the frame on `drape()`. Their heads meet, snap and recoil at the bottom. The gold "Of Man's Labours" caption sits below, uncovered. |
| **Moonwalker** (one bubble) | `border-drollery-dragon` | The traced torso rides `makeMoonwalker`'s path, with the traced head, wing and legs articulated at their real joints. |
| **Swarm** (gold, round the "agentic development" line) | small vipers from `bestiary-vipers-miniature` | They orbit and dart round the bubble. |
| **"yuck"** | one small, wide-mouthed viper from `bestiary-vipers-miniature`, or the `romanesque-roundel` head on a short coil | Small, round and cute, bouncing round the bubble like a wind-up toy. |

Skip batches 3 and 4 (the woodcuts, engravings and the stone Visconti relief) unless a slot
needs them.

## The integration in `crawler.js`

`crawler.js` is yours. Add what the slots need, each driven by one timeline:
- graft points for traced head, wing, leg and tail components;
- a flight loop;
- a frame chase;
- a yuck bounce.

Run each creature only while it is on screen, so phones stay light. The writer will not edit
`crawler.js`; it will place what you hand over.

## Splitting the work (for /teamwork)

One stream per slot, each owning its own files:
1. Knotted pair.
2. Burrowers.
3. Headline frame.
4. Moonwalker.
5. Swarm and "yuck".
6. **Rig:** the `crawler.js` graft points and the new motions that streams 2–5 need, plus the
   shared proof-page harness. The other streams ask this stream for rig changes.

Finish slot by slot, and push each to `origin/art/bestiary-serpents` as soon as its proof
passes, adding it to `HANDOFF-4.md`. The writer integrates one slot per PR as each lands.

Work only under `docs/art/bestiary/`. Commit as Louis Maddox <louismmx@gmail.com>, with no AI
attribution.
