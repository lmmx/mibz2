# The origami lift: a BWT block as a branched cover of the torus

*20 November 2026. The Coordinator, from a discussion with Louis. A candidate change of basis,
recorded for the foundational track. No bets yet: nothing here is yet a cheaper decoder.*

**Status, by claim:**

**MONODROMY's verdict (record 86 at 32d32f74; REVIEW 7040376454 on #145).** Its null rule was frozen
first, at bee415be.
1. **Support of k, corrected:** supp k = C̄ \ X, the codomain starts minus runs that share a
   translation mod n. The smallest counterexamples are `ba` and `bca`. X is empty on the six
   blocks, which is why the count came out as exactly r.
2. **Cone-point counts:** verified.
3. **Not exceptional.** All 233 Wikidata blocks match run-shuffled columns (median 5 cone points).
   Random permutations of the support give about 10. The few cone points are a general property of
   (LF, +1), not a feature of Wikidata.
4. **The φ form:** proved, as conjugation by SA.
5. **The genus formula:** proved, by Riemann–Hurwitz.
6. **Saddle connections:** the horizontal ones are the arcs, the vertical ones are the runs (Open's
   strips), and those of slope 1 are all short.
7. **The investigation.**
   - *Proved:* the cone structure is the commutator of (LF_w, +1) for the run-symbol word w alone.
     So it is O(r) from the entry state.
   - *Obstruction:* the cone structure ignores run lengths, so it cannot determine the text. The
     counterexample is `cbaa` against `cbbaa`.

**What this leaves:**
- *Closed:* the singular locus alone as a decoding object, and "near-minimal strata are special to
  Wikidata".
- *Still open:* directions and transport (§6.2), which also depend on the lengths.

| Claim | Status |
|---|---|
| The construction (§1) | proved, elementary |
| The singular locus is the run boundaries (§2) | proved, and measured on six blocks |
| Cone-point counts and genus (§3) | measured, exact, on six blocks |
| The commutator equals φ's breakpoint dynamics (§4) | argued; MONODROMY to verify |
| The questions in §6 | open |

## Why record it

The project's first shift came from recognising LF as a discrete interval exchange. That opened
Rauzy–Veech induction, a whole body of mathematics that row-by-row traversal could not reach.

This note records a second lift of the same kind. It takes the exchange's suspension one step
further, to an exact square-tiled surface whose **monodromy** is LF. The term "monodromy" is used
in the classical topological sense: transport around loops in a covering, as in Riemann–Hurwitz
and the singularity-theory tradition of Thom, Picard–Lefschetz and Milnor. It is not a homonym.

Whether the lift yields a cheaper primitive is unknown.

## 1. The construction

The ingredients:
- **L:** a BWT column of n rows.
- **r = LF:** stable LF, one step back in the text.
- **u:** u(i) = i + 1 mod n, the lexicographically next row.

**The surface.** Any pair of permutations (r, u) of n things defines an **origami**:
- one unit square per row;
- the right edge of square x is glued to the left edge of square r(x);
- the top edge of square x is glued to the bottom edge of square u(x).

**Equivalently**, a branched cover of the torus, unbranched outside one point. Its monodromy
representation F₂ → Sₙ sends one generator to r and the other to u. Label square x with the byte
L[x].

**The text in this language.** The text is the reverse of the label sequence read along the
horizontal leaf through the primary square p. A primitive block has one LF cycle, so the
horizontal direction is a single cylinder of circumference n and width 1.

## 2. The singular locus is the run boundaries (proved)

**The proof.** Inside a run, LF(i+1) = LF(i) + 1, so r∘u and u∘r agree there. Consequently:
- the commutator k = r u r⁻¹ u⁻¹, the monodromy around the puncture, fixes every square except
  those at run boundaries;
- the surface is flat everywhere else;
- its cone points are the cycles of k on the r boundary squares;
- the cone angle at each point is 2π times its cycle length;
- the genus follows from Riemann–Hurwitz: 2g − 2 = Σ (cycle length − 1).

**Reading Open in this language:** the flat regions are bundles of adjacent rows that travel
together. These are Open's convoys, and its thick strips are wide flat strips. Open has been
exploiting the flat part of this surface.

## 3. The measurements (exact, six blocks)

[`origami.py`](2026-11-20-origami-lift/origami.py) is standard-library Python. Run it with
`BLOCKS=<dir with manifest.json>/`.

**Inputs.** The six authenticated columns on branch `data/admitted-bwt` (`admitted-bwt/`). Their
SHA-256 digests are in that branch's `manifest.json`, and their provenance is the bench's pinned
Wikidata corpus (`82f65ace…`). The script prints the commutator's moved squares, its non-trivial
cycles with their lengths, the genus, and whether LF is a single cycle.

| Block | r | Squares moved by k | Cone points | Genus | LF one cycle |
|---|---|---|---|---|---|
| 0 | 51,184 | 51,184 | 6 | 25,590 | yes |
| 2 | 54,532 | 54,532 | 8 | 27,263 | yes |
| 85 | 79,375 | 79,375 | 5 | 39,686 | yes |
| 143 | 108,335 | 108,335 | 3 | 54,167 | yes |
| 207 | 87,111 | 87,111 | **1** | 43,556 | yes |
| 210 | 128,732 | 128,732 | 6 | 64,364 | yes |

**What they show:**
- **k moves exactly r squares on every block,** confirming §2.
- **The r boundary squares fall into only 1–8 cycles.** Real Wikidata blocks are surfaces of genus
  about r/2 with a handful of enormous cone points.
- **The genus is not an independent finding.** Let R be the number of squares k moves and c the
  number of its non-trivial cycles. Riemann–Hurwitz gives 2g − 2 = Σ (cycle length − 1) = R − c,
  so g = 1 + (R − c)/2. For block 207: 1 + (87,111 − 1)/2 = 43,556. A high genus follows from
  R ≈ r. What is informative is how small c is.
- **Block 207 lies in the minimal stratum:** one cone point, of angle 2π × 87,111. One cycle of
  the puncture monodromy visits every run boundary.
- **The largest cycles.** Block 0's largest cone point has angle 2π × 39,737, and block 210's
  2π × 118,964.

**An open question about this:** a generic exchange with r pieces need not be near-minimal.
Whether the BWT forces it, and whether non-Wikidata text differs, is not yet known. It needs a
proper null model before anyone calls it a special Wikidata property: for example, BWTs of shuffled
or randomly generated text with matched r/n.

**Scope:** "k moves exactly r squares" is measured on six blocks. It is not yet a theorem for every
BWT column. Cyclic wraparound at row 0, and distinct runs that happen to share a translation, may
give exceptions. A proof needs stated hypotheses.

## 4. The singular locus is φ's breakpoint structure (argued; to verify)

**The translation.** In text coordinates, k becomes j ↦ φ⁻¹(φ(j) + 1) − 1, where φ is the r-index's
predecessor function, φ(SA[i]) = SA[i − 1].
- It fixes j exactly where φ shifts by +1.
- It moves only φ's r breakpoints.

**The consequence:** the monodromy around the cone points is the dynamics of φ's breakpoints.

**The link to earlier work:** Neon's cycle 1 measured the Φ-overlap structure as small (1.25–1.37r
transitions), and found it expensive to acquire. In this language that structure is the surface's
singular locus, whose cyclic structure is very simple (§3).

## 4a. A constraint the lift exposes immediately

For a primitive block, **both** gluing generators are single n-cycles: LF by assumption, and u by
definition. So the horizontal and vertical directions each already form one cylinder, the minimum
possible. A search for directions with *fewer* cylinders therefore cannot be the goal. Any
advantage must come from something richer:
- cylinder widths and heights;
- the complexity of the symbolic itinerary;
- the structure of the transformed generators;
- the cost of transporting their words back.

## 5. What it is not yet

These are recorded so the lift isn't oversold.
- **Familiar objects reappear as special cases.**
  - Greedy Rauzy–Veech induction is one path in the family of §6.2.
  - Its substitutions form the S-adic expansion Cells builds.
  - The flat strips are Open's convoys.
- **The global invariants are too coarse to decode with.** Genus, the stratum and the
  Kontsevich–Zorich cocycle (its action on homology) give only how many times each symbol occurs,
  not their order.
- **The cone-point coincidence was overstated earlier.** An earlier chat equated D̄∩C̄ with
  saddle connections. It counts only one-step boundary coincidences. The full saddle-connection
  structure is a separate computation, not yet done.

## 6. Open questions at the level of the surface

1. **Walk the singular locus.**
   - *Cost:* each step of k from a boundary square uses only rows next to known run boundaries.
     LF there is O(1), and Ψ is one select, about O(log r). Walking every cone-point cycle
     therefore costs r dependent steps. Open makes about n/10 dependent loads; Cells takes 1.7–1.9r
     induction steps.
   - *What it yields:* the cyclic order of boundaries around each singularity, not their text
     positions directly.
   - *The question:* do this order and the flat structure determine the text cheaply? That would
     reconstruct from the singularities, with the flat part handled collectively.
2. **Move through the family.**
   - *The action:* SL(2, ℤ) / Out(F₂) acts on the monodromy pair by Nielsen moves, for example
     (r, u) ↦ (r, u r⁻¹). This is the same surface seen in other directions. An origami's orbit is
     finite and closed (a Teichmüller curve).
   - *Where Cells sits:* greedy Rauzy–Veech induction is one path in this space.
   - *The question:* are there directions in which the surface decomposes into few, wide
     cylinders, and does transporting the reading back to the horizontal direction (an explicit
     substitution) cost less than it saves? This would mean deforming to an easier member of the
     family and transporting back, made exact.
3. **Why so few cone points?**
   - *What to look for:* a theorem that the BWT forces near-minimal strata, or a measurement
     showing where it fails (prose, code, low-r/n blocks).
   - *Why it matters:* either would give an O(r) structural handle on a block before any
     construction.
4. **The full saddle connections.**
   - *What to compute:* the saddle connections on the six blocks: their number, their lengths, and
     how they relate to D̄∩C̄ and to Open's thick strips.
   - *How it is decided:* this one is a matter of exact computation.

## A lead for Neon (not an assignment)

The object is the pair (LF, u) ∈ Sₙ × Sₙ, not LF alone. It has operations that don't exist for
LF by itself:
- Nielsen moves, and the SL(2, ℤ) / Out(F₂) action on the pair;
- the monodromy representation, and its subgroups and covers;
- the cylinder decomposition in every direction;
- the singular locus.

**The question:** is there a presentation or direction of this pair in which the marked symbolic
orbit has a cheaper exact description, with an affordable law for transporting it back?

**What would answer it:**
- a new primitive, or a sharply scoped obstruction, is the result wanted;
- computing the invariants listed in §5 is not.

**What not to do:**
- don't force the answer back into Open's formulation;
- don't assume it involves the singular locus. It might come from the group action, from a
  decomposition of the cover, or from a relation between the generators that is simple in another
  presentation.

## Routing

- **Neon:** take this as one lead for the foundational track, at a cycle boundary. FRONTIER.md
  points here.
- **MONODROMY:** verify §2–§4. The proof of §2 is short; §4's identification needs checking.
- **No charter change** follows from this note.
