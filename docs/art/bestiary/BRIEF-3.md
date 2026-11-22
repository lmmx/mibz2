# Bestiary brief 3: medieval cartoons that travel

This brief supersedes BRIEF-2.md wherever they conflict. It is Louis's direction on round 2.

## What went wrong in round 2

The round-2 creatures went the wrong way. They read as engraved copperplate, drafted penmanship
or pen sketches: hatching, pressure-varied hairlines, broken contours. They also mostly sit still,
a fixed pose with small idle twitches.

Louis wants the opposite on both counts:
- **The look:** the bold, confident, cartoon life of the first attempt (the creatures live on the
  post now), made specifically medieval. Not generic fantasy, not "arcana", nothing that looks
  like amateur fantasy art.
- **The motion:** creatures that travel along paths through and around the chat in interesting
  ways.

What round 2 got right, and should be kept: the burrow into the "i" (the `crawler.js` rig and its
masked-hole vanishing).

## The look

**Style target:** English and Flemish manuscript drolleries and bestiary illumination, about
1200–1350:
- the Aberdeen Bestiary, for dragons and serpents;
- the Luttrell, Gorleston and Macclesfield Psalters, for marginal grotesques with comic timing;
- Insular zoomorphic interlace (Kells, Lindisfarne), for the knotted pair.

**Line:**
- One confident, continuous, near-uniform dark outline in iron-gall brown-black, about 2.5–3 px
  at display size.
- No hatching, no cross-hatching, no sketchy doubled strokes, no hairline taper.

**Colour:**
- Flat fills of saturated pigment, pushed to neon for the page: vermilion, ultramarine or lapis,
  verdigris green, orpiment yellow, and burnished gold for crowns.
- No pink. Nothing pastel, oxidised or dull.
- Interior detail is a few decisive marks only: belly bands, a row of scale arcs, lead-white
  highlight dots.

**Cute and alive.** The first attempts are the reference for character. Give each creature:
- an oversized head, about a third of the body;
- big round eyes with a white glint;
- a toothy grin or a curled tongue;
- stubby legs;
- a pose that tells a joke, as the drolleries do.

## The motion: every creature travels, none just sits

Each creature moves along a path through and around the chat, not a fixed pose with idle
twitches. Drive all of it with `crawler.js`, one timeline per creature.

- **The winged dragons and the swarm:** take off from a bubble's corner, loop or swoop in a
  figure-eight around the message, and land back touching the bubble. Wings beat in time with the
  flight, and the head turns to look at the reader at the top of each loop.
- **The moonwalker:** a full circuit of its bubble's edge, with a sliding foot, a heel roll and a
  pause at a corner. It never leaves contact with the bubble.
- **The knotted pairs:** the knot tightens and loosens as each serpent slides through it and
  bites at the other's tail. Then the pair slithers along the bottom edge of its bubble and back.
- **The burrowers:** crawl along the line of text, weaving over and under words, pause to look
  around, then dive into the "o" or "i" with the clean masked vanish. Then they re-emerge, so the
  scene loops.
- **The headline frame serpents** (round "managing Claudes all day is knackering"): chase each
  other round the frame. Their heads meet at the bottom, snap and recoil. The gold caption, "Of
  Man's Labours & their Eternal Return", sits below the frame, uncovered, at every width.
- **"yuck":** the deliberate odd one out, small, round and cute. Its creature bounces round the
  small bubble like a wind-up toy, still framing the red blackletter "yuck".
- **Timing:** vary speeds and add pauses, so the page feels alive, not mechanical.

## The page's rules (unchanged)

- Colour neon and saturated; no pink.
- Fleurons ❧ ☙ green, links blue.
- Creatures touch their bubbles.
- Vanishing is clean: no fade and no drawn hole.
- Nothing scrolls sideways at 390 px.
- The page stays light on phones: run each creature only while it is on screen.

## Where the page is now

The post is a normal page of the site now, and the private source (`chat.md`,
`build_html.py`) is retired.
- The conversation's source is `docs/site/blog/conversations/2026-11-22-thomas-murphy.html` on
  master.
- `docs/scripts/inline_conversation.py` turns it into the fragment the post includes,
  `docs/conversations/2026-11-22-thomas-murphy.inline.html`, with its assets under
  `docs/site/assets/conversations/conversation-thomas-murphy/`.
- The writer integrates the art into the page. Work only under `docs/art/bestiary/`.

## Deliverables

- The six creatures, and the cute "yuck" variant, as SVGs driven by `crawler.js`.
- For each creature, a sheet placing the new drawing beside its first-attempt version and a
  manuscript reference.
- For each creature, an animated proof of a full cycle of its motion (an HTML page that plays
  it), so the motion can be judged before anything goes into the post.
- Changes only under `docs/art/bestiary/`, committed as Louis Maddox with no AI attribution.
