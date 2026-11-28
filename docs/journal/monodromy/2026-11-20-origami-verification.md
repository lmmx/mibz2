# 2026-11-20 · MONODROMY 86 · The origami lift verified. k moves exactly the codomain starts C̄, except for runs that share a translation mod n; that is r on the six blocks, but not in general. The cone-point counts and genus check, and the genus formula is Riemann–Hurwitz. φ's form is SA-conjugation. The horizontal saddle connections are the persistence arcs, and the vertical ones are the runs. Few cone points is not a Wikidata property: run-shuffled columns have as few

**The request.** The Coordinator's CHECKPOINT 7039964966: `docs/journal/2026-11-20-origami-lift.md`
at master 7aa152c0.

**Labels:** **proved** · **verified** (exhaustive on the stated domain) · **measured**.

**Files** (in `2026-11-20-origami-verification/`):
- `ori.py.txt`: the checks;
- `nulls.py.txt`: the null models;
- `null-rule.md`: claim 3's rule, committed at bee415be before any null data existed.

**Conventions.** Composition is right to left, so k(x) = r(u(r⁻¹(u⁻¹ x))) = LF(Ψ(x − 1) + 1),
indices mod n. Going once around the bottom-left corner of square x, counter-clockwise, passes
u⁻¹x, then r⁻¹u⁻¹x, then u r⁻¹u⁻¹x, and returns at k(x). So the vertices are the cycles of k,
and a vertex of cycle length ℓ has cone angle 2πℓ.

**Small domain:** every column over 2 letters to length 12, 3 letters to length 8 and 4 letters to
length 6. That is 23,490 columns, 3,029 of them one LF cycle. Any column is allowed, so several
cycles and periodic columns are included.

## Verdicts

| # | claim | verdict |
|---|---|---|
| 1 | k moves exactly the run-boundary squares, \|supp k\| = r | **Corrected.** supp k = C̄ \ X, where X = {LF(d) : d ∈ D̄, LF(d) ≡ LF(d − 1) + 1 (mod n)}. So \|supp k\| = r − \|X\|. This holds on all 23,490 columns, but \|X\| > 0 on 12,069 of them (2,434 of 3,029 single-cycle). On the six blocks X = ∅, so R = r |
| 2 | cone points 6, 8, 5, 3, 1 and 6; block 207 minimal | **Verified.** All counts and the two largest cycles match exactly |
| 3 | Is the small cone-point count exceptional? | **Not exceptional, by the rule fixed in advance** (§3). Run-shuffled columns have as few |
| 4 | k = j ↦ φ⁻¹(φ(j) + 1) − 1 in text coordinates | **Verified, and proved:** it is k conjugated by SA. It holds on 23,093 (column, mark) pairs and the six blocks. The breakpoints are the SA images of C̄ \ X |
| 5 | g = 1 + (R − c)/2 | **Proved** (Riemann–Hurwitz; the surface is connected because u is an n-cycle). It equals V − E + F on every column and block |
| 6 | Saddle connections | **Computed exactly.** In the horizontal, vertical and slope-1 directions, the horizontal ones are record 80's persistence arcs. \|D̄∩C̄\| is the number of length-1 horizontal ones. The vertical ones are the runs, so Open's thick strips are the long vertical saddle connections |

## 1. The support of k (proved, corrected)

**The proof.**
- *Write z = Ψ(x − 1),* so x − 1 = LF(z).
- *If z + 1 (mod n) is not a run head,* z and z + 1 lie in one run, so LF(z + 1) = LF(z) + 1 = x,
  and k(x) = x.
- *If z + 1 = d is a run head,* with d = 0 when z = n − 1, then x = LF(d − 1) + 1. That is the
  start of the codomain interval that follows run(d − 1)'s, in cyclic order, so x ∈ C̄. Then
  k(x) = LF(d), which differs from x unless LF(d) ≡ LF(d − 1) + 1 (mod n).
- *Why every point of C̄ is reached:* the codomain intervals tile ℤ/n, so d ↦ LF(d − 1) + 1 is a
  bijection D̄ → C̄. □

**So the moved squares are the codomain starts,** not the run starts, unless a pair of
cyclically adjacent runs share a translation mod n. That is the hypothesis the note anticipated,
made exact.

**The exceptions come in three forms.** Across the 729 single-cycle columns over at most three
letters to length 10 or 7:

| form | columns | smallest example |
|---|--:|---|
| row-0 wrap: L[0]'s bucket starts where L[n − 1]'s ends, cyclically | 571 | `ba` |
| image wrap: LF(d − 1) = n − 1 and LF(d) = 0 | 208 | `ba` |
| plain interior: d − 1 holds the last a, d the first b, and b is a's successor | 104 | `bca` |

**The smallest counterexamples.**
- *`ba`,* the BWT of `ab`: LF and u are both the swap, so k = id, r = 2, R = 0, and the surface is
  a torus.
- *`bca`:* LF = +1 = u, so again k = id, with r = 3 and R = 0.

**Counted:** R = r − |X| and supp k = C̄ \ X on all 23,490 columns. The six blocks have X = ∅.

## 2. Cone points and genus (verified; claim 5 proved)

**The six blocks:** R = r, and c, the largest cycle and g all match the note's table exactly.

| block | R | c | largest cycle | g |
|--:|--:|--:|--:|--:|
| 0 | 51,184 | 6 | 39,737 | 25,590 |
| 2 | 54,532 | 8 | 20,976 | 27,263 |
| 85 | 79,375 | 5 | 63,897 | 39,686 |
| 143 | 108,335 | 3 | 54,408 | 54,167 |
| 207 | 87,111 | **1** | 87,111 | 43,556 |
| 210 | 128,732 | 6 | 118,964 | 64,364 |

**The genus formula.**
- *Riemann–Hurwitz:* for a degree-n cover of the torus branched over one point,
  2g − 2 = Σ over the cycles of k of (length − 1) = R − c.
- *Why the cover is connected:* ⟨r, u⟩ is transitive, since u is an n-cycle.
- *Parity:* k is a commutator, so it is even, and R − c is even.
- *A second count:* Euler's formula with V = #cycles of k (fixed points included), E = 2n and
  F = n gives the same g on every column and block.

## 3. Is the small cone-point count exceptional? (measured, under the frozen rule)

**The rule** (`null-rule.md`, fixed at bee415be before the data): Wikidata's few cone points are
exceptional only if more than 25% of its 233 large blocks have c below the 5th percentile of both
N1 and N2.

| null | what it is | c |
|---|---|---|
| N1 | uniform even derangements, R = 51,184 (2,000 samples) | 5th percentile 6, median 10, 95th 16 |
| N1 | the same, R = 128,732 | 5th percentile 6, median 10, 95th 16 |
| N2 | the six blocks' runs shuffled (20 samples each, 120 in all) | 5th percentile 2, median 5, 95th 8 |
| N3 | all 233 Wikidata blocks | 5th percentile 2, median 5, 95th 8; min 1 (4 blocks), max 11 |
| N4 | prose (4 blocks; r/n 0.29–0.41, **not matched**) | 6, 4, 11, 5 |
| N5 | this repository's Rust source (2 blocks; r/n 0.26–0.31, not matched) | 6, 5 |

**Verdict: not exceptional.** Only 4 of the 233 blocks (1.7%) fall below both 5th percentiles,
far under the 25% the rule requires. N3's distribution matches N2's: median 5 in both, and the
same 5th and 95th percentiles. Block 207's single cycle is one of the 4 blocks at c = 1.

**BWT columns do differ from random permutations.** N3's median of 5 is below N1's 5th
percentile (6). Prose and code show the same, at 4–11.

**What it means.** A BWT's k is far from a random permutation of its support. Shuffling the runs
keeps r/n and the run lengths, and already gives cycle counts as low as Wikidata's. So the small
count comes from how runs glue in the stable sort, not from Wikidata's content. A theorem about
why remains open.

## 4. φ's breakpoints (verified; proved as conjugation)

**The proof.** Give rows text positions SA, cyclically, with SA[LF(i)] = SA[i] − 1. Then:
- r becomes j ↦ j − 1;
- u becomes φ⁻¹, where φ(SA[i]) = SA[i − 1].

Conjugating gives SA∘k∘SA⁻¹ (j) = φ⁻¹(φ(j) + 1) − 1, exactly as the note states.

**Verified:** on every single-cycle small column at every mark (23,093 checks) and on all six
blocks.

**What it adds: no new information.** The fixed points are where φ shifts by +1, and the
breakpoints are SA(C̄ \ X). The singular locus is φ's breakpoint set by relabelling, so its cycle
structure is §2's.

## 5. Saddle connections (computed exactly)

**Which directions.** In the direction (p, q), saddle connections are the segments between
singular corners along the orbits of r^p u^q. I computed every saddle connection in the
horizontal (r), vertical (u) and slope-1 (u∘r) directions. The full set over all directions is
infinite.

**Proved** (with X = ∅; verified on every small column with X = ∅ and all six blocks):
- **Horizontal = the persistence arcs.**
  - *Why:* along LF, the singular corners are C̄. The next point of C̄ after c is LF(d), where d
    is c's first run head, so the horizontal saddle connections are exactly the arcs of record 80,
    of lengths ℓ(c) + 1.
  - *Their count:* R = r, of mean n/r.
  - ***D̄ ∩ C̄*** *is the number of length-1 horizontal saddle connections* (c ∈ D̄, ℓ = 0). It is
    the shortest class of a full spectrum.
- **Vertical = the runs.**
  - *Why:* in row order the singular corners are C̄, and consecutive codomain starts are one
    codomain interval apart, which is one run's length.
  - *So Open's thick strips* (runs longer than τ) *are the vertical saddle connections longer
    than τ:* 3,448, 3,827, 1,565, 5,047, 3,474 and 2,809 at τ = 16. These are DOE's m from
    record 85.

**Measured, on the six blocks:**

| block | horizontal: length 1 = \|D̄∩C̄\| / max | vertical: length 1 / max | slope 1: length 1 / max |
|--:|--:|--:|--:|
| 0 | 40,230 / 4,714 | 35,116 / 26,407 | 31,416 / 1,215 |
| 2 | 41,309 / 10,435 | 34,262 / 26,319 | 30,135 / 890 |
| 85 | 62,256 / 848 | 55,504 / 25,630 | 49,245 / 647 |
| 143 | 81,371 / 17,545 | 72,578 / 25,134 | 64,403 / 419 |
| 207 | 66,308 / 4,305 | 62,407 / 25,780 | 54,905 / 1,048 |
| 210 | 101,984 / 6,845 | 97,054 / 27,234 | 88,041 / 346 |

**Each direction has R saddle connections of mean length n/R.**
- *The longest:* about 26,000 vertically (the longest run), up to 17,545 horizontally (the
  longest arc), but at most 1,215 at slope 1.
- *Interpretation, not measured:* the slope-1 direction (u∘r, one LF step then the next row) has
  no long saddle connection on any block. Whether a direction with uniformly short saddle
  connections gives a cheaper transport is the note's §6.2 question. These numbers are its first
  data.

## 6. Item 7, the investigation: what the lift gives for reconstruction

The three kinds of result are kept apart below: what is proved, what is observed, and what is
only speculation.

### Proved correspondences

1. **The singular structure is an O(r) function of the run-symbol word w alone.**
   - *The run-level form:* by §1, k(LF(d − 1) + 1) = LF(d). On runs, that is
     k̂ = D ∘ C⁻¹, where D is the next run in row order and C the next run in codomain order.
   - *Why it depends on w alone:* the codomain order of runs is the stable sort of the run
     symbols, whatever the lengths.
   - *What k̂ is:* the commutator of w's own pair (LF_w, +1).
   - *So:* with X = ∅, the block's cone points, their angles and the genus are those of the
     r-letter run word's origami. Computing them costs one sequential pass over the records.
   - *Verified:* the cycle type agrees on all 11,421 small columns with X = ∅, and on all six
     blocks (6, 8, 5, 3, 6 and 1 cycles, with identical largest cycles).
2. **An obstruction follows: the singular structure cannot determine the text.**
   - *Why:* anything computed from it alone is unchanged when the run lengths change and the run
     symbols don't. The text is not.
   - *The smallest witness:* `cbaa` and `cbbaa`. Both are one LF cycle with X = ∅ and the same run
     word `cba`, so both have c = 1 and R = 3. But their texts differ.
   - *So walking the cone-point cycles (r steps, as the note's §6.1 proposes) gives only w's
     combinatorics.* The positions the text needs, the arc lengths, are exactly what it loses.
3. **The two axis directions are the dIET's own objects** (§5): the horizontal saddle connections
   are the arcs, and the vertical ones are the runs.
4. **Every other direction is a word in the generators.**
   - *What a direction is:* the orbit of a word g in (r, u). Its singular corners are the same C̄
     \ X, because the cone points are intrinsic.
   - *Nielsen moves:* (r, u) ↦ (r, u r^{±1}) or (r u^{±1}, u) re-express r as a word in the new
     generators. So reading the marked orbit through them costs at least one generator step per
     byte.
   - *What can be batched:* only the flat strips between saddle connections in that direction.
     That is the same piecewise-translation machinery (Rauzy–Veech, Open) applied to a different
     exchange, not a new operation.

### Observed phenomena

- **The singular cycles are few in every BWT-structured column:** N2, N3, prose and code. A
  random permutation of the support is not like this (§3). By item 1 this is a property of run
  words under the stable sort. Why that permutation has so few cycles is open.
- **The slope-1 direction (u ∘ r) has only short saddle connections:** at most 346–1,215, against
  about 26,000 vertically and up to 17,545 horizontally (§5).
- **One identity failed, so it is withdrawn.** I tested whether the slope-1 orbit reads
  (u r)^k(x) = LF^k(x + k), the anti-diagonals of the rotation matrix, until it meets a singular
  corner. It failed in 5,522 of 25,021 steps: the commutations needed happen at points the orbit
  never visits.

### Speculative (unmeasured; no cost account)

- **Inducing in the slope-1 direction.** Its uniformly short saddle connections suggest a
  better-balanced exchange than the horizontal one, whose arcs are badly unbalanced (record 84).
  Its orbits are not the text, though. Transport back would have to cost less than the balance
  saves, and nothing here shows that.
- **The strongest opportunity in the lift:** item 1. It gives an exact O(r) structural invariant
  of each block, a cheap classifier of the stratum, before any construction. It cannot
  reconstruct the text, by item 2.

## What this decides

Nothing in production.
- *What the lift reproduces:* the objects already measured. C̄ is the singular set, the arcs are
  horizontal, the runs (Open's strips) are vertical, and D̄ ∩ C̄ is the shortest horizontal
  class.
- *Where it needs correcting:* "exactly r" needs X = ∅, and the small cone-point count is
  generic to BWTs.
