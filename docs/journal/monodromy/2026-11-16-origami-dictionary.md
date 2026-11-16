# 2026-11-16 · MONODROMY 31 · The origami's dictionary, completed: vertical saddle connections are the stretches of LF's cycle, and the quarter-turned surface is Φ (next suffix in sorted order), whose induction spells the BWT column: the forward transform. Every surface object maps onto one we already have; none changes what the decoder computes

Record 30's two next items, on the surface (r, u) = (x ↦ x + 1, LF).

**Labels:** **P** proved · **C** counted (exact for its input) · **Conj** conjecture.

## 1. Vertical saddle connections are the stretches (P)

**Where the cone points are.** At the corners of run ends: the right side of square x where x and
x + 1 lie in different runs (record 30).

**Where a vertical line goes.** The line up the right side of square x continues up the right side
of u(x) = LF(x), because the top of x is glued to the bottom of u(x).

**When it meets a cone point.** Exactly at the first k ≥ 1 with u^k(x) a run end. Below that,
u^j(x) and u^j(x) + 1 lie in one run, so the corner is regular.

**So:**
- *There are r vertical saddle connections,* one up from each run end's corner.
- *Each runs to the next run end along LF's cycle,* and its length is the gap between them.
- *They are the stretches of LF's cycle,* cut at the run ends, which the Rauzy–Veech entry and
  records 11 and 12 studied.

**What follows, already known.**
- *The equal steps,* as a tree, are the Cartesian tree of the stretches' minima in cycle order
  (Theorem 3 of the Rauzy–Veech entry).
- *Which stretches the free closure closes* has no local rule in a stretch's shape or its place in
  that tree (record 12, §1).

So "S − floor counts saddle connections" says nothing that record 12 does not.

## 2. The quarter turn is the forward transform (P; C)

**The turn.** It sends (r, u) to (u⁻¹, r). Number the squares along u's cycle by text position p,
so that the new horizontal u⁻¹ is the shift. Then the new vertical r, in text coordinates, is

  Φ(p) = the text position of the row after p's row in sorted order,

the next suffix in sorted order. That is the map behind PLCP and the r-index's toehold.

**Pieces.** Φ is a piecewise translation with as many pieces as the surface has run ends.
- *Proved:* the dual's commutator is the same corner structure, read the other way.
- *Counted (`phi_breaks`):* r − 1 or r breaks on all 30 blocks checked.

**What its induction would spell.** Read up the new vertical cylinder, the squares' labels are the
rows in sorted order, which is the BWT column. Its Rauzy–Veech induction would build that column
from the text: the forward transform. It does not decode.

**So the decoder's problem and the compressor's are the two quarter turns of one surface.** Both
are exchanges of r pieces in one stratum. The decoder can only use the turn whose vertical spells
the text, which is the one Cells already induces.

## 3. The dictionary, and the verdict on this line

| surface | the decoder's object | record |
|---|---|---|
| squares; u = LF; r = shift | rows; the walk; the sorted order | 30 |
| the vertical cylinder | the text (one cycle) | 30 |
| cone points | cycles of σ on runs; 1 to 11 a block | 30 |
| vertical saddle connections | the stretches of LF's cycle | here |
| zippered rectangles of a stage | the letters' columns; runs | 26, 28 |
| Rauzy–Veech induction | Cells | the Rauzy–Veech entry |
| the quarter turn | Φ, the forward transform | here |
| genus | r/2 | 30 |

**Verdict.**
- *An exact dictionary:* every object of the surface maps onto an object the programme already has.
- *No predictor:* none of its coarse invariants predicts S (record 30).
- *No new computation:* none of its operations gives a computation the decoder does not already
  do.
- *Where the cost lies (Conj):* in the induction's path, how lengths fall, which is the
  schedule question. Orders give up to −32% of the word store with random access, and −15% at the
  stream fronts (record 29).

**The origami line closes here as a dictionary, not a lever.**

## Reproduce

`origami.rs.txt` (in `2026-11-16-origami/`), field `phi_breaks`, every 8th block:

```
cargo run --release --example origami -- corpora/wikidata.bz2 8
```
