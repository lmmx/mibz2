# Bestiary brief 6: traced creatures, rigged

This replaces BRIEF-5. Louis rejected BRIEF-5's procedural redraws: ribbons, little eels and a
bounce nobody asked for.

The target is every creature in the **traced** style of the perched roundel
(`traced/romanesque-roundel-animated.svg`), fully rigged so that it moves as well as the old
cartoons did.

## What to keep, unchanged

- **On the live post:** the headline frame (the big writhing orange serpents), the perched
  roundel, and the current creatures in every slot, until a traced replacement is approved.
- **From BRIEF-5:** the winged burrower Louis liked. Use it as the reference for how a burrower
  should move.
- **From `crawler.js`:**
  - the spine and timeline engine;
  - the masked-hole burrow, with zero stray pixels;
  - running only while on screen.

## What to drop

- **No procedural bodies at all:** no tubes, ribbons or eels.
- **No bounce:** nothing wind-up.
- **Not the BRIEF-5 versions:** the knotted pair, the swarm and the headline chase from BRIEF-5
  are out.

## The job

1. **Trace the missing specimens and components** with the existing pipeline (the one that made
   the roundel). That covers batch 2 and anything else a slot needs:
   - border-drollery dragon;
   - calendar serpent;
   - vipers miniature;
   - a clean cut-out of the two Gothic interlace dragons.

   For each creature, trace as separate components:
   - head, with the jaw separate;
   - neck;
   - body;
   - tail;
   - wing(s);
   - legs and feet.
2. **Rig each traced creature for movement.**
   - **The body:** cut its traced body into short overlapping segments, 12 to 30 of them, each
     pinned at its joint on a spine.
   - **The motion:** as the spine moves, each segment follows and rotates, so the traced body
     bends, coils, writhes and travels.
   - **Head, wings and feet:** ride their joints, each with its own small motion (jaw, blink,
     turn, wing beat).
   - **The look:** must stay the traced look at every frame, with no visible seams between
     segments (overlap them).
3. **Fill each slot with a traced, rigged creature.**
   - **Burrowers:** the Aberdeen serpent and the calendar serpent crawl along the text and dive
     into "o"/"i" through the masked hole.
   - **Knotted pair:** the two interlace dragons, coiling round each other.
   - **Moonwalker:** the drollery dragon walks the bubble's edge.
   - **Swarm:** small traced vipers swooping round the bubble.
   - **Headline frame:** traced serpents writhing round the frame, at least as big and lively as
     the current frame. Only replace it if it's better.
   - **"yuck":** a small traced viper or the roundel head, coiled round the bubble, cute, with
     no bounce.

## Rules

- Neon colour through `--wash` and `--wash2`, no pink.
- Creatures touch their bubbles.
- Vanishing is clean.
- No sideways scrolling at 390 px.
- About 25 KB gzipped per creature.

## Working

- **One agent:** no /teamwork, /swarm or subagents.
- **Order:** finish one slot at a time, starting with the knotted pair.
- **Each slot ships with:**
  - a proof page in `ship/proof-<slot>.html`, showing the creature in its bubble at desktop width
    and at 390 px, at the same size as the slot on the post today;
  - a row in `ship/HANDOFF.md`.
- **Pushing:** push each slot as it's done.
- **Scope and commits:** work only under `docs/art/bestiary/`. Commit as Louis Maddox
  <louismmx@gmail.com>, with no AI attribution.
