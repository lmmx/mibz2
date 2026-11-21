# What the shape of the Riemann proof shows us

*21 November 2026. Coordinator. Non-canon: a reading of method, not a result about mibz2.*

## The source

On 20 November, the Lean Zulip thread "Discussion: Alpöge Proves the Riemann Hypothesis" reported
that three independent proof kernels had accepted the Lean development: nanoda, con-leche (in
`--verified` mode, justifying `riemannHypothesis` and `LFunction_ne_zero_of_half_lt_re` among five
theorems) and pestka. The development is Lean v4.33.1, about 17 million lines in 175,190 files. It
proves RH and GRH for Dirichlet L-functions, not for the Selberg class. The toy proof in Alpöge's
remark is not part of it. Chris Birkbeck posted an overview on the thread, written with Fable's
help. Paraphrased:

1. RH follows from "no Dirichlet L-function vanishes for Re(s) > 1/2". This step is short and
   standard.
2. That nonvanishing follows from three things:
   - a growth bound on certain Galois cohomology groups (Tate–Shafarevich-like) over a family of
     subfields of cyclotomic fields;
   - a covolume bound from arithmetic geometry;
   - statements about theta integrals on unitary groups, together with local Langlands for GL_n.

   Birkbeck: "I have not yet understood how these combine to exclude zeros."
3. The cohomology bound is an automorphy-lifting argument. It uses Galois representations attached
   to regular algebraic, polarized, cuspidal automorphic representations over CM fields.
4. Those Galois representations are not assumed. The development builds them from a geometric
   construction for unitary groups (Eichler–Shimura, crystalline above ℓ), with theta lifts between
   unitary groups. As far as Birkbeck can tell, this is not the route in the literature.
5. It uses no axioms, so the infrastructure is proved inside the development too: resolution of
   singularities in characteristic zero, arithmetic Hilbert–Samuel, Moriwaki's bigness theorem,
   toroidal compactifications of Siegel varieties, Kawamata–Viehweg and Nadel vanishing,
   Grothendieck–Messing, and prismatic Dieudonné theory.

We have not checked any of this ourselves. What follows uses only the proof's reported *shape*.

## The shape, as six moves

| | Move in the proof | What it is, abstractly |
|---|---|---|
| A | RH ⇐ nonvanishing in a half-plane, for every Dirichlet L-function | Turn the target into an **exclusion statement over a family**: no bad event in a region, for every member |
| B | Nonvanishing ⇐ three bounds of different kinds | A **squeeze**: an algebraic count, a geometric volume and a local harmonic classification, each from a different field, together leave no room |
| C | Local Langlands for GL_n | **Local–global**: classify the local pieces completely, so the global argument only glues |
| D | Galois representations built, not assumed | When the standard route needs an object you can't import, **build an equivalent object by another construction**, then transfer it (theta lifts) |
| E | No axioms, 17M lines | **Infrastructure is the bulk.** The insight is small next to what it stands on, and an agent can now afford to build that |
| F | Three independent kernels, before anyone understands it | **Trust comes from independent checks of the same artefact**, not from understanding. Understanding comes later |

Two smaller points also bear on us:
- **The headline is wider than the theorem:** "RH, and its generalisation" means GRH for Dirichlet
  L-functions.
- **The toy that guided the discovery is not in the artefact.**

## What each move means for mibz2

### A. Our target as an exclusion statement over a family

We state goals as "kernel X beats Open by y%". The proof's first move suggests a statement of a
different kind: no costly event happens outside an irreducible set, for every block in a family.

**For us, a "zero" is a dependent memory access on the critical path:** a load whose address
depends on the previous load, missing cache. Open already has this shape. Thick strips are
eliminated without dependent loads, and only the retained rows (about 0.1n) are walked. Depth's
lower bounds play the role of the zeros on the critical line: what can't be excluded.

**Move:** restate the obligation register's central quantity as a family statement:

  N_dep(B) ≤ f(r(B), profile(B)) for every block B in the family,

where N_dep counts dependent accesses. Ask each research line which bound pushes f down. Then a
candidate is judged by the bound it proves on N_dep, as well as by the time it measures.

### B. A squeeze from three bounds of different kinds

The obligation register counts only one kind of thing. The proof combines three different kinds of
bound. We have one candidate of each, from separate lines that have never been put together:

- **The algebraic count:** Kitchen Sink's directed feedback vertex set. Only 0.3% of runs break every
  cycle of the run-overlap graph (cycle 9). This counts the obstruction, the way the cohomology
  bound does.
- **The geometric volume:** the memory hierarchy's capacity. Workers × per-worker working set must
  fit the shared L3 (20 MB on the i9), and DRAM bytes per output byte must stay under the bandwidth
  each worker gets. The fidelity ledger's crowd losses (lanes: +13% alone, −5% at 8 threads) are
  this bound binding.
- **The local classification:** Open's LIMIT 16 split into thin and thick runs, and Vec16's 16-byte
  emit table, which is a complete answer for one local shape.

**Move:** accept a candidate only when all three bounds hold for it at once. It must:
- shrink the obstruction (FVS-like) set;
- stay within the volume at 6–10 workers;
- resolve every local configuration it claims from a table or a fixed formula.

The Wormhole prompt now asks for chains that end in such a squeeze.

### C. Local–global: a catalogue of local factors

Local Langlands classifies the local pieces completely, so the global argument only has to glue.
Vec16 is a one-entry instance: one local shape (a thin run of up to 16 rows), one table.

**Generalisation:** index retained rows by their local configuration (the next k runs' symbols and
lengths, relative to the row), and look up a precomputed fragment of the output, a "local factor",
for each configuration. The global walk then steps from one factor to the next.

**The catch, which decides it:** LF targets depend on global rank offsets (C[c] plus the rank
within c), not only on local shape. A local factor is useful only if the output fragment, as
distinct from the addresses, depends on local shape alone. That is a measurable question:

- **Census:** on the six authenticated blocks, for k = 1…4, count the distinct local configurations
  among the retained rows. Measure the share of retained rows covered by the most common
  configurations, and whether each configuration determines its emitted bytes.
- **Kill rule** (registered here, before the data): drop the lead if fewer than 50% of retained rows
  fall in configurations that determine their output, using a catalogue that fits in 256 KiB per
  worker (an L2 share).

### D. Build the object we keep importing, by another construction

Every decoder we run imports one object, LF, the rank map, built the standard way: counts and then
a walk. The proof's lesson is that the expensive object can be rebuilt by a construction from a
different field, then transferred. The Galois representations in step 4 were built geometrically,
then moved between groups by theta lifts. Candidate constructions of the text order that avoid LF:

- **Φ and toehold sampling** (the r-index). Φ steps through the suffix array in text order from r
  samples. The cost is in getting the samples. Is there a chain that yields them in O(r) from the
  run structure? This is the first link the Wormhole line should test.
- **The bijective BWT and Lyndon factorisation** (Gil–Scott; Kufleitner): text order through the
  Lyndon structure, not through LF.
- **Induced sorting run backwards** (SA-IS's L/S types): the forward algorithm's induction from a
  small seed set, inverted.

### E. Infrastructure is now affordable

17 million lines, with no axioms. Routes we have dismissed as "too much engineering" deserve a
second look, because the engineering is now the cheap part. Three of them, in order of leverage:

1. **A crowd harness on CI.** Run the lazy route at 4 workers on CI's 4 CPUs, with the shared-cache
   pressure that production has, so candidates are screened for crowd behaviour before an i9 run.
   The fidelity ledger shows one-thread wins reversing at 6–10 workers (lanes, R4), and CI's
   `mibz2` arm at 1–4 threads never runs Open at all (stage attribution, 20 November).
2. **Per-block kernel selection from a family of kernels.** Admission already chooses between Cells
   and Open per block. A larger family, chosen by measured profile, is the natural extension.
3. **Machine-checked invariants for Open:** elimination preserves LF on retained rows. The parity,
   Miri and fuzz checks already constrain this. A proof would cover what fuzzing can't reach.

### F. Independent checks of the same artefact

The three kernels checked one artefact, the same files with the same hash. Our three measurements
(the C harness, native CI and the i9) did not. The C harness timed a re-implementation, so its wins
were a claim about a different program, and the fidelity ledger shows where they parted.

**Move:** research lines hand over Rust patches against master, not re-implementations in C. Then
CI and the i9 check the same artefact the line measured. This is "experimenting in Rust, not C",
stated as a principle.

Adopting before understanding is legitimate when the checks are independent. That is our policy
already: we adopt on a registered measurement, and mechanism hypotheses go under the ledger's
*Open* heading.

### Scope, and the toy

- **Scope:** 0.5.9 was released for one routing change: unpinned, 4 threads, on the i9. Its
  release-scaling run pins its threads (CPUs 0–3 at 4 threads), so it can't see the change. Both
  releases ran identical code in every cell it measured (#685). Release scaling should add an
  unpinned leg at the cells a routing change touches, or each release note should name the scope
  that was measured.
- **The toy:** the toy-transfer method (FRONTIER) guides discovery. The artefact must stand without
  it, as Alpöge's Lean development stands without his remark's toy proof.

## What happens next

- FRONTIER gains this lead, with the census in C as its first test.
- The Wormhole prompt gains the proof's shape: a squeeze, building instead of importing, and Φ
  sampling as the first link to test.
- The Coordinator runs the local-factor census (C) now, and proposes the CI crowd harness (E1) as
  bench work.
