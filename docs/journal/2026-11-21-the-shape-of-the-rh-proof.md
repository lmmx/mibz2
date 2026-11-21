# What the shape of the Riemann proof shows us

*21 November 2026. Coordinator. Non-canon: the proof's meta-structure as a general principle, then its consequences for mibz2.*

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

## Meta-strategy: how a proof of this kind is engineered

This reads Birkbeck's overview as strategy. Where it names the classical patterns behind its steps
(Wiles's numerical criterion, Deligne's Weil II, Bloch–Kato), that is our inference.

### I. Choosing the statement

1. **Strengthen the target until your operations preserve it.** RH alone is not attacked. The
   development proves nonvanishing for every Dirichlet L-function across cyclotomic families. The
   stronger statement is easier, because the family is closed under the operations the tools need:
   twisting by characters, base change up a tower, lifts between groups. A single object gives an
   induction nothing to hold. This is the inventor's paradox, used on purpose: choose the statement
   by what your operations preserve, not by what you were asked.
2. **Prove a growth rate, not a value.** The bounds work in towers, where a hypothetical zero would
   make the obstruction grow at the rate of a volume. One side's growth exponent is then compared
   with the other's. Exponents survive the error terms and constants that would swamp an exact
   comparison at a single level.
3. **Use inequalities only, in both directions.** Hilbert–Samuel, bigness, Kawamata–Viehweg and
   Nadel vanishing are all robust inequalities. Nothing needs an exact formula. A chain of
   inequalities is long but local: each link can be checked alone. That is why 17 million lines
   are checkable while no one holds the whole argument.

### II. Choosing the invariant

4. **Choose an invariant with two faces.** The quantity carrying the proof (the size of a Selmer- or
   Tate–Shafarevich-like group) has two faces:
   - an algebraic face, bounded above by Galois deformation theory and automorphy lifting;
   - a geometric or analytic face, tied to L-values and covolumes.

   Every load-bearing tool is an identity equating counts of two different kinds of thing: trace
   formulas, Lefschetz, Eichler–Shimura, arithmetic Riemann–Roch. The art is finding an invariant
   that two unrelated theories can each grip. One theory gripping it gives one bound. Two give a
   squeeze.
5. **Amplify before measuring.** One zero is invisible to coarse tools. Pushed through a family or
   up a tower, it produces many obstructions, enough to violate a bound. This descends from
   Deligne's tensor-power and Rankin-squaring arguments: bound |α|^{2k} for every k, and you
   control |α|. Find a construction under which a defect multiplies.
6. **Look for positivity.** Theta integrals and Petersson norms are manifestly nonnegative
   quantities, and so are volumes and Hodge-index forms. A zero is ruled out by showing it would
   force such a quantity negative, or below a volume. Much of the change of setting is aimed at
   reaching a place where positivity exists.

### III. Choosing the route

7. **Move to where the tools work, then move back.** The original objects have no geometry. Unitary
   groups have Shimura varieties: they are proper and are moduli spaces of abelian varieties, so
   they carry Frobenius, cohomology, positivity and p-adic Hodge theory. The route is planned
   backwards, from where the bounds exist to a chain of functors (base change, theta lifts) that
   reaches it. The functors are the cost, and the destination is the point.
8. **Make each hard step local, where classifications are complete.** Local Langlands, Clozel's
   purity and Bernstein–Zelevinsky classify everything at one place. The global step is then a
   product or a count over places. Difficulty is pushed to where it has already been solved
   completely.
9. **Re-optimise the route when the cost function changes.** The literature's route is optimal for
   human cost: short, with large black boxes cited (Harris–Taylor, Scholze's torsion classes). With
   no axioms, a black box costs its full formalisation. So the cheapest route changes: more pieces,
   each simpler to build. "Not the route in the literature" is what to expect when the cost of a
   proof is measured differently, not an oddity.

### IV. Running the search

10. **Make checking cheap and certain, then search wildly.** A kernel check is certain and costs
    little next to discovery. So the search can afford high variance: unintuitive routes, long
    compositions, steps no one has understood. Accepting a step doesn't depend on judgement.
    Whether a generate-and-check strategy pays off depends entirely on what the check costs.
11. **Use a toy to find the missing ingredient, then drop the toy.** The remark's toy proof found
    the ingredient, but the artefact doesn't contain it. The toy exists to say which capability has
    to transfer, not to be scaled up.
12. **Decouple trust from understanding.** Three independent kernels accepted the development before
    anyone could say how the three bounds combine. Understanding becomes a later, separate project,
    not something acceptance has to wait for.

## What this demands of our programme

The demand is the same strategy run on our problem, with each lever put as a question.

| Lever | The question for mibz2 | Where it stands |
|---|---|---|
| 1. Strengthen until closed | What family of inputs is closed under the operations our tools use? Splitting a block into segments, rotation (primary-index changes), concatenating blocks, run refinement. A decoder statement for the whole family may admit induction (parallel splitting) where one block does not | Never posed. Our statements are per-block and per-kernel |
| 2. Growth rates | Measure how N_dep, the number of dependent accesses, scales with block size, r and the run profile, as exponents across block sizes from 100k to 900k, against cache capacity. A design whose exponent is below the volume's wins whatever the constants | Only single-size measurements exist |
| 4. Two-faced invariant | String attractors γ, LZ77's z and the BWT's r are compressibility measures on two different sides (text and BWT), tied by r = O(z log² n) (Kempa–Prezza). Text-side theory could give lower bounds on decoding, and BWT-side constructions give upper bounds. This is the natural invariant for a squeeze | Depth worked in a fixed cost model without text-side measures |
| 5. Amplify | Which construction multiplies a cost defect? Powers of LF (LF^k, pointer jumping) and block concatenation are candidates, so that a small structural advantage invisible at k = 1 becomes decisive | Unexplored |
| 6. Positivity | Is there a nonnegative quantity whose vanishing is "no dependent miss"? A cut or flow quantity on the run-overlap graph, whose feedback set is 0.3% of runs | Kitchen Sink's FVS is a count, not yet a bound |
| 7. Move and return | Plan backwards from settings where both bounds exist (IET and Teichmüller, discrete Morse, compressed indexes), then ask which transfer reaches them in O(r) | Wormhole's charge, now stated this way |
| 9. Re-optimise the route | LF walking was optimal under the cost model of its time, a single core with cheap random access. Under 6–10 workers sharing 20 MB of L3, the optimal route may be a different one, not a tuned LF | Our kernels are tuned LF descendants |
| 10. Cheap checking | Our correctness check is cheap and certain (parity, Miri, fuzz). Our performance check is the opposite: the i9 is scarce, noisy and human-gated, and C proxies were unfaithful. Wild search pays only once the performance check is cheap: a CI crowd harness, and cost models validated against the evidence branch | The binding constraint. It is investment priority one, ahead of new kernels |
| 12. Trust without understanding | Adopt on registered measurement and keep mechanisms as hypotheses | Already our policy |

**The single strategic conclusion:** levers 10 and 4 decide the rest. Until performance checking
is as cheap as correctness checking, a broad search over compositions can't be afforded. Until we
hold a two-faced invariant (γ, z and r is the candidate), there is nothing to squeeze, only
mechanisms to race.

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

## Consequences for mibz2, move by move

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

**Result (#687, the same day): dropped.** The best determining share within a 256 KiB catalogue
was 10.5% (block 0, k = 2, m = 4). Configurations of at least two rows that determine their output
cover at most 3.3% of retained rows, and a catalogue pooled over the six blocks covers 2.8%. The
reason: the bytes a row emits next are its own sorted prefix, fixed by its position in the global
order, not by the shape of the runs around it. Local classification fails here because the
invariant is global. This is lever 8's precondition failing: complete local classification must
exist before difficulty can be pushed to the local step.

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
