# The run network, measured: separators, first-return pieces and the chain of 41

*20 November 2026. Measurement only, following [2026-11-20-run-network.md](2026-11-20-run-network.md)
(open questions 1–3). No decoder, no charter change. Scripts:
[`census.py`](2026-11-20-run-network-census/census.py) and
[`baseline.py`](2026-11-20-run-network-census/baseline.py), run on the six admitted blocks
(numpy 2.5.5, scipy 1.18.1, pymetis 2025.2.2).*

**Notation:**
- n is the block's rows, and r its maximal runs.
- m is Open's retained rows: the rows in runs of width ≤ 16.
- The core is the series-reduced run network. Its vertices are the runs with in-degree or
  out-degree ≥ 2. Its arcs are the chains of pass-through runs between core vertices.

## The rules, fixed before the data

**Q1. Separators of the core.** "promising" if the total port count is at most 0.05n on at least
four blocks; "not promising" if it is at least 0.15n on four or more. Heuristic separators give
upper bounds; say so.

**Q2. Where the ordering information lives.** "the graph adds leverage" if the count is at most
0.5·m on four or more blocks; "no leverage" if it is at least m.

## Q1: a separator hierarchy of the core

**Method:**
- *The graph:* the core, undirected. Self-loops are dropped (38–197 per block), and parallel arcs
  are kept as edge weights.
- *The bisection:* recursive METIS bisection (pymetis, default balance), to pieces of at most 64
  vertices (7–8 levels).
- *Ports:* a port is a core arc cut at some level. Each arc is cut at most once, so the total
  equals the number of core arcs that cross some separator. Counting both ends of each cut arc
  doubles it, and that column is given too.
- *Vertex separators:* the minimum vertex cover of each level's cut (König), summed.
- *What the numbers are:* METIS is a heuristic, so every figure is an upper bound on the optimum.

**Edge cut per level (arcs):**

| Block | Core vertices | L0 | L1 | L2 | L3 | L4 | L5 | L6 | L7 |
|---|---|---|---|---|---|---|---|---|---|
| 0 | 5,771 | 813 | 1,119 | 1,512 | 914 | 1,005 | 1,091 | 952 | – |
| 2 | 6,558 | 697 | 1,071 | 1,243 | 1,448 | 1,376 | 1,565 | 1,438 | – |
| 85 | 8,531 | 837 | 1,993 | 1,483 | 1,813 | 1,578 | 1,533 | 1,638 | 1,407 |
| 143 | 13,230 | 1,176 | 2,434 | 2,213 | 3,053 | 3,550 | 2,399 | 2,776 | 2,437 |
| 207 | 10,176 | 1,092 | 1,922 | 1,917 | 2,389 | 2,441 | 2,032 | 1,836 | 1,904 |
| 210 | 14,069 | 1,154 | 2,962 | 2,875 | 2,198 | 2,675 | 2,521 | 2,381 | 2,309 |

**Totals:**

| Block | Core arcs | Ports (cut arcs) | Share of core arcs cut | Ports / n | Both ends / n | Ports / r | Ports / m | Rows on cut arcs | Vertex-separator total |
|---|---|---|---|---|---|---|---|---|---|
| 0 | 16,725 | 7,406 | 0.44 | 0.0082 | 0.0165 | 0.145 | 0.082 | 0.267n | 2,801 (0.49 of core) |
| 2 | 19,781 | 8,838 | 0.45 | 0.0098 | 0.0196 | 0.162 | 0.079 | 0.267n | 3,127 (0.48) |
| 85 | 25,650 | 12,282 | 0.48 | 0.0136 | 0.0273 | 0.155 | 0.079 | 0.277n | 5,032 (0.59) |
| 143 | 40,194 | 20,038 | 0.50 | 0.0223 | 0.0445 | 0.185 | 0.102 | 0.315n | 6,673 (0.50) |
| 207 | 30,979 | 15,533 | 0.50 | 0.0173 | 0.0345 | 0.178 | 0.114 | 0.311n | 5,382 (0.53) |
| 210 | 40,817 | 19,075 | 0.47 | 0.0212 | 0.0424 | 0.148 | 0.093 | 0.283n | 8,255 (0.59) |

**Verdict under the rule: "promising".** The port total is at most 0.05n on all six blocks,
whether ports are counted once (0.008–0.022n) or at both ends (0.017–0.045n). These are upper
bounds.

**Why the verdict is weak:**
- *The rule is nearly vacuous.* The whole core has only 0.019–0.045n arcs, so even cutting
  every arc stays under 0.05n.
- *The structure doesn't show small separators.* About half the core arcs (0.44–0.50) end up as
  ports.
- *The cut doesn't shrink with depth.* The per-level cut stays flat. At level 0 it is about
  0.14 cut arcs per vertex (block 0: 813 for 5,771). At level 6 it is about 0.16 (952 for 64
  pieces of 92 vertices). A cut proportional to piece size at every scale means the separators
  are linear in size. A planar-like or tree-like hierarchy would give sublinear ones.
- *It is still far from random.* A degree-preserving random rewiring of the same core bisects
  4–8× worse (block 0: 3,608 against 813; block 210: 8,874 against 1,154). The core has locality,
  but not a hierarchy that compresses.
- *Port maps would carry a lot of rows.* The cut arcs carry 0.27–0.32n rows.

## Q2: pieces of the first-return map to the branching rows

**Method:**
- *Which rows:* B is the set of rows in branching runs (in-degree or out-degree ≥ 2). That is
  0.51–0.56n rows.
- *The walk:* the script walks the LF cycle densely once. LF is one cycle on every block. For
  each row of B, it records its first return T(x) to B and a hash of the run sequence it passes
  through.
- *Two counts:*
  - (a) the maximal intervals of consecutive rows in the same run that follow the same arc
    sequence, which is the brief's definition;
  - (b) the maximal intervals of consecutive rows of B on which T is a translation,
    T(x+1) = T(x)+1.

| Block | m (Open) | m / r | (a) pieces | (a) / n | (a) / r | (a) / m | (b) translation pieces | (b) / m |
|---|---|---|---|---|---|---|---|---|
| 0 | 89,874 | 1.76 | 16,725 | 0.0186 | 0.327 | 0.186 | 16,617 | 0.185 |
| 2 | 112,070 | 2.06 | 19,781 | 0.0220 | 0.363 | 0.177 | 19,659 | 0.175 |
| 85 | 156,404 | 1.97 | 25,650 | 0.0285 | 0.323 | 0.164 | 25,435 | 0.163 |
| 143 | 196,841 | 1.82 | 40,194 | 0.0447 | 0.371 | 0.204 | 39,869 | 0.203 |
| 207 | 135,921 | 1.56 | 30,979 | 0.0344 | 0.356 | 0.228 | 30,777 | 0.226 |
| 210 | 204,281 | 1.59 | 40,817 | 0.0454 | 0.317 | 0.200 | 40,469 | 0.198 |

**Verdict under the rule: "the graph adds leverage".** The count is 0.16–0.23m on all six
blocks, well under 0.5m.

**Two facts that bound what the verdict means:**
1. **The count is the core arc count, exactly.**
   - *The measurement:* (a) equals the series-reduced arc count on every block, to the unit.
   - *Why that is forced:* two adjacent rows of a core run that land in the same run stay
     adjacent through every pass-through run, since a pass-through run maps whole into one run.
     So they return to the same core run, still adjacent.
   - *What remains:* the bundles of the marked circuit are therefore the core arcs. Only about
     0.6–0.9% of adjacent piece boundaries are spurious, where T happens to stay a translation
     across two different arc sequences ((b) < (a)).
   - *What it adds:* Q2 needs no row walk: it is the census's own 0.32–0.37r. The walk confirms
     the identity; it finds no new structure.
2. **Much of the margin against m is run-length compression, not the graph.**
   - *Runs alone:* r is already 0.49–0.64m, since m = 1.56–2.06r. On block 2 the run count
     alone meets the 0.5m threshold.
   - *What the network adds:* the factor from r to the core arcs, about 3× (0.32–0.37r).

**The comparison is also of unlike things.** m counts rows that Open handles individually. The
pieces are intervals, each needing a start, a width and a target. The interval exchange on B has
0.32–0.37r pieces against LF's r. That is the measured size of the "collective primitive" the
run-network note asked about, if one exists. Nothing here shows it can be acquired or consumed
more cheaply than Open's rows.

## The chain of 41

**On block 0** (`census.py --chain` prints the chain for every block):
- *The length:* the longest series chain has 41 pass-through runs, each of width 2.
- *The core arc's ends:* it runs from a `"` run of width 722 to a `:` run of width 36.
- *The text it spells:* read in text order (LF runs backwards), the symbols are
  `:"d86f0e4609ff92b6b693a41cf49c0baa72160a8c"`. The 41 runs are the opening quote and the 40
  hex digits of a SHA-1 `"hash"` value from a Wikidata reference or qualifier, a hash that occurs
  twice in the block (hence width 2).

**Blocks 85, 143, 207 and 210:** the longest chain is again a SHA-1 hash, 40 hex digits plus
the quote. It has width 9 on block 85 and width 1 on the others.

**Block 2 is the exception.**
- *The chain:* its longest chain is **37**, not 41. It is a statement GUID: `$` plus
  `B1C80647-2187-4623-A594-C787BB3FFE84`, an 8-4-4-4-12 GUID of 36 characters, after the digit `3` that ends the entity id.
- *The earlier note:* runnet.py itself prints `max 37` for block 2. So "41 on every block" in
  [2026-11-20-run-network.md](2026-11-20-run-network.md) is a transcription slip, on five
  blocks of six.

**Why a hash, and why exactly 41: structural.**
- *How a chain grows:* a run of width 1 always has in-degree and out-degree 1, and a run of a
  string repeated w times keeps them at 1. So a series chain lasts as long as each preceding
  symbol is unpredictable from its right-context, and the context is shared by no other
  occurrence.
- *Inside a random hex string:* both hold for every position, so each character is its own run.
- *In natural text:* preceding symbols are predictable from shared contexts, and runs of
  different origin merge within a few characters.
- *Where the chain ends:* at the delimiters, since `":"` and `","` precede thousands of rows.
- *The bound:* the longest high-entropy tokens in Wikidata JSON are therefore the reference and
  qualifier hashes (40 hex digits) and the statement GUIDs (36 characters). The maximum chain is
  their length plus one delimiter: 41, or 37 when no hash in the block survives whole.
- *What it is not:* not the 127 cap, a record width or JSON nesting.

## Scratch

The venv and the outputs were deleted after the run. To reproduce, set `BLOCKS` to the
admitted-bwt directory (manifest.json and block-*.L). The full run takes about 2 minutes on one
core.
