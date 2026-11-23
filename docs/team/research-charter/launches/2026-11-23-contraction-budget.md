# Launch record: Contraction Budget, 23 November 2026 (prepared)

- **Model:** Gemini Carbon, High, in a fresh Antigravity conversation and its own checkout.
- **Branch:** `research/contraction-budget`. Label WIP commits `wip(contraction-budget)`.
- **Role:** one falsifiable engineering hypothesis. Kitchen Sink stays the breadth engine. Wormhole,
  Mechanisms and the teamwork programme have finished.
- **Why:** the maintainer's analysis of 23 November. The next large gain may come from deciding,
  block by block, how much of the Open kernel's construction to buy, rather than from making that
  construction or the walk faster.
- **Prepared** by the Coordinator at Louis's request. Paste everything inside the fence.

````text
# Contraction Budget: how much construction should each block buy?

You are starting a new, independent research line for lmmx/mibz2, a high-performance Rust bzip2
decoder. Its hardest stage is the exact inverse Burrows–Wheeler transform (BWT). Earlier Gemini
sessions have worked on it, and their published work is yours to read; don't touch their branches:
- Mechanisms (`research/carbon-word-transport`) built and timed reconstruction mechanisms;
- Wormhole (`research/carbon-wormhole`) studied the parallel transport and the plateau;
- Kitchen Sink (`research/carbon-kitchen-sink`) explores broadly, and is still running;
- the teamwork programme (`research/neon-teamwork-paradigm`) proved bounds on contraction, and
  stated four laws for why materialising contractions lose at bzip2's block size.

## The idea

The inverse BWT must write n output bytes, but it need not make n *dependent* memory accesses.
The shipped Open kernel shows this. It first builds structure from the block's runs: an
elimination, edges, and "convoys" of rows that travel together. Then it walks only the rows it
kept, about a tenth of them, with each step a dependent load plus a 16-byte copy. Construction is
bought to shorten the dependency chain.

That makes the decoder's cost a trade, not a single quantity. Let k be the fraction of a block's n
rows kept for walking. Then, to first order,

    T(k) = T_construct(k) + n · k · t_walk(k),

where construction cost rises as more dependent steps are eliminated. The teamwork programme cites,
as an i9-10900K calibration from `bench-emit-cost` (#145), construction at 0.72–0.90 ns per block
row and the walk at 4.8–5.8 ns per kept row. Check those figures against their evidence before you
rely on them: the coordinator could not confirm that the run was made.

**The hypothesis:** the k that minimises T differs from block to block. A block whose runs give
the contraction plenty to work with can justify elaborate construction. A block dominated by
random identifiers can't: Wikidata's UUIDv4 core leaves little structure for *this* contraction
model to exploit. That doesn't prove the walk irreducible. It says further contraction of this
kind stops paying for itself. So choosing, per block, how much construction to buy may beat any
fixed strategy.

**The decoder already adapts, crudely.** Read `src/open.rs` and the dispatch around it, plus:
- `docs/journal/2026-11-19-open-admission.md`, the admission band;
- `docs/journal/2026-11-09-target-gated-route-decision.md`;
- `docs/site/blog/2026-11-22-correctness-is-uniform-cost-is-not.md`.

A block's run density r/n and other cheap statistics already route it to the Open kernel or the
Cells kernel. The Open kernel picks between two-ended and one-ended emission by size. A change
being ported now adds a cheaper construction path for blocks whose edges fit in 384 KiB. That is a
two- or three-point budget with hand-set thresholds. Your question is whether a principled,
finer-grained budget does better.

## What to establish, in order

1. **The trade's shape, per block.** For each of a fixed set of real blocks, measure
   T_construct, the kept rows K, and the walk cost under at least two construction strategies A
   and B. The strategies already in the code are the first candidates: Open's full construction,
   Open's lean small-block path, and the Cells kernel. Test the marginal condition
   
       T_construct,A − T_construct,B  <  t_walk · (K_B − K_A).
   
   Does the extra contraction eliminate enough walking to pay for itself? Treat t_walk as a
   measured function, not a constant: locality, memory traffic and the worker count all change it.
2. **The deciding statistics, and when they exist.** A choice must be made before the work it
   avoids. Use only what the decoder has before construction: the run count r, the run-length
   distribution, the symbol histogram, and anything the first pass (`runs::front`) already
   computes. Estimate the cost of computing each statistic.
3. **A simple rule first.** Fit the simplest rule that might work, such as one threshold on one
   statistic. Only then consider a small classifier. Count the rule's own overhead in its cost.
4. **Block-local before stream-wide.** Blocks in one stream can differ, so decide per block. Note
   any stream-level signal separately. A worker count chosen by the input is a related open
   question: today's recalibration showed 8 workers 4–7% faster on Wikidata but 6–11% slower on
   text.

## The falsifier (registered here, before any data)

The adaptive hypothesis **fails** if a simple per-block rule, with its own overhead counted, does
not beat the best *fixed* strategy on **held-out** blocks, by a margin outside noise. It must win
at both 1 worker and the highest worker count you can measure, on both Wikidata-like and structured
corpora. Choose the held-out blocks before you fit anything, and say how you chose them. A rule
fitted and scored on the same blocks proves nothing.

## Data and tools

- **Real blocks:** the authenticated admitted blocks on branch `data/admitted-bwt`, and
  `corpora/wikidata.bz2` (431 blocks, 12 windows). Also the repository's generated corpora from
  `ab gen`: text, records, random, pattern, runs and onebyte.
- **Per-block timing:** Open's phase split (feature `open-phases`, research branch
  `research/open-phase-split`, #653) times construction and emission separately.
- **The scorer:** `bench/moonshot` on master, a correctness gate and paired timing.
- **Your machine** is not the i9. Report what transfers and what doesn't. The team's i9 runs are
  done by the maintainer, from frozen recipes; you never time on it.

## Rules

- **Correctness never bends.** Every strategy must give byte-identical output on every block.
- **No changes to `src/` on master.** Prototype on your branch, under `docs/journal/` or
  `research/`.
- **Name things by what they do,** never after a team, model or round.
- **Label every claim:** proved, verified, measured (with the machine), or modelled.
- **Hand off** a journal with the trade's measured shape, the rule, the falsifier's outcome, and,
  if the rule wins, a production design ready to port. Interim handoffs are welcome.
````
