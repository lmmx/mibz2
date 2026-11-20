# The kitchen sink: use everything

*20 November 2026. Louis's direction, after a relayed overview of the claimed RH/GRH Lean
development. The overview was posted by Chris Birkbeck, summarising another model's reading of the
code, and is unverified here. A pending lead, not an assignment.*

## What the overview says the argument uses

As relayed, the argument's dependency stack runs as follows:
- **The reduction:** RH from the non-vanishing of Dirichlet L-functions for Re s > 1/2.
- **What that non-vanishing rests on:**
  - a growth bound on Tate–Shafarevich-like Galois cohomology over cyclotomic subfields;
  - an arithmetic-geometry covolume bound;
  - theta integrals on unitary groups, with local Langlands for GL_n.
- **The automorphy lifting it uses:** Galois representations for polarised cuspidal
  representations over CM fields, base change, Jacquet–Shalika, Clozel's purity,
  Bernstein–Zelevinsky, CM theory and Deligne–Ribet.
- **Not the literature's route:** those Galois representations are *derived*, from a geometric
  construction for unitary groups plus theta lifts between them.
- **The arithmetic geometry, proved inside the project with no axioms:**
  - resolution of singularities;
  - arithmetic Hilbert–Samuel;
  - Moriwaki bigness;
  - toroidal compactifications;
  - Kawamata–Viehweg and Nadel vanishing;
  - Grothendieck–Messing;
  - prismatic Dieudonné theory.

## The lesson we take (corrected)

Louis: "no no, seriously: kitchen sink approach." Alex Kontorovich on the claimed proof: "RH is
proved by using *everything*!" Igor Carron: "Throwing the whole kitchen sink at solving things, has
never been easier."

The first draft of this note said every import needs a named job before it comes in. That
reverses the order of events. In the relayed stack, the jobs were found by bringing the machinery
in. The non-standard route (Galois representations via theta lifts) is the kind of thing nobody
would have proposed in advance. So:
1. **Everything comes in first.** The cost of importing heavy machinery has collapsed, so breadth is
   no longer a luxury. Each cycle should try to build bridges from many frameworks, including ones
   that look irrelevant.
2. **A named job is the output of the attempt, not its entry ticket.** Each attempt is recorded:
   - the bridge found, or why none exists;
   - what it would buy;
   - its acquisition cost.
3. **The filter comes after, and stays unforgiving.** The register and native timing judge what
   survives. Breadth on the way in, rigour on the way out.

## Status of this list

These are suggestions, not a canon. A researcher's own candidates have equal standing, and may
replace, extend, combine or ignore these (Louis, 20 November 2026: "if Carbon is the expert then
surely Carbon should be seeding this"). The number of fields used is never a measure of success.

## The kitchen for inverse BWT

Each entry gives a framework, its bridge to our problem, and what it would buy if the bridge holds.
"Studied" means the repository already has work on it.

| Framework | Bridge | What it would buy | Here before? |
|---|---|---|---|
| **Rauzy fractals and the Pisot conjecture** (Rauzy 1982; Arnoux–Ito 2001; Barge, Akiyama et al.) | For an irreducible Pisot substitution, the symbolic system is (conjecturally in general, provably in many cases) measurably conjugate to a translation on a torus. The Rauzy fractal is the coordinate domain | **Global coordinates beyond two letters:** the direct generalisation of the Sturmian toy, where position k sits at x + kβ on a torus and its letter is read from which fractal piece holds it | no |
| **Veech groups and the SL(2,ℤ) action on origamis** (Schmithüsen; Herrlich) | Our (LF, +1) block *is* an origami. SL(2,ℤ) acts on origamis by affine maps. Its orbit and stabiliser (the Veech group) are computable from the two permutations | Other directions on the same surface: an affine image whose horizontal permutation has far fewer cycles or runs. This is "directions and transport", made algebraic | origami yes; Veech groups no |
| **Krohn–Rhodes decomposition and semigroup scans** | Emission is a product in a transformation monoid (Carbon's "monoid contraction"). Krohn–Rhodes factors any finite automaton into a cascade of simple groups and resets. Parallel prefix computes products in any monoid | A principled way to split the walk into composable pieces, with the reset parts free and the group parts small | no |
| **Dimension groups and K₀ of AF algebras** (Effros–Handelman–Shen; Giordano–Putnam–Skau) | Rauzy induction yields a Bratteli diagram, whose ordered K₀ group is a complete invariant of the Cantor system it presents | Invariants of a block's hierarchy cheap enough to read without building it: a test for which zoo member applies | barely (Bratteli: 1 file) |
| **Numeration systems** (Ostrowski; Bratteli–Vershik; Dumont–Thomas for substitutions) | A position's path down the induction hierarchy is a digit expansion | **Random access:** the letter at position k in O(depth). The question is whether the digits can be read without the whole build | Ostrowski: 2 files |
| **Random permutation statistics** (Poisson–Dirichlet; Shepp–Lloyd) | Unique random strings (GUIDs, 54–67% of one-row runs) act like a random permutation segment | Either a provable floor for that class, or the exact law of how many independent chains it contains, which sizes any lanes or splitters | no |
| **Wheeler graphs and co-lex orders** (Gagie–Manzini–Sirén) | The BWT as the simplest Wheeler graph; Carbon proved Myhill–Nerode minimisation collapses nothing on primitive blocks | Mostly a closed door now (Carbon cycle 2), kept for completeness | yes |
| **Move structures and run-length BWT tricks** (Nishimoto–Tabei; Brown–Gagie–Rossi) | LF in O(1) per step with O(r) space by balancing run intervals | Practitioner prior art; already studied (32 files) | yes |
| **Gessel–Reutenauer, Foata's transformation** | The BWT as a bijection between words and multisets of necklaces; the cycle structure of the standard permutation | Enumerative control of the cycle structure; studied | yes (8 files) |
| **Kontsevich–Zorich cocycle and Lyapunov exponents** (Avila–Forni; Zorich) | The growth rates of word lengths under Rauzy induction | How deep a hierarchy must be, so the cost of reading coordinates | yes (22 files) |

## More of the kitchen (not yet bridged; each is an attempt to make)

**Algebra and representation theory.**
- **Representation theory of the symmetric group:** LF is a permutation. Fourier analysis on S_n
  (Diaconis), characters and cycle types.
- **RSK:** the correspondence from permutations to tableau pairs, applied to LF or to the
  standard permutation. What do its shapes say about runs?
- **Coxeter groups:** LF as a product of interval transpositions. Reduced words, and the
  Bruhat order.
- **Cluster algebras and snake graphs** (Çanakçı–Schiffler): continued fractions as cluster
  expansions. A bridge from the Sturmian toy to an algebra with mutation.
- **Christoffel words and Markoff numbers** (Reutenauer's book): Cohn matrices encode Christoffel
  words. A matrix coordinate for the toy that might generalise.

**Geometry and topology.**
- **Braids and knot theory:** a permutation with its run structure as a braid closure. Our
  origami's monodromy as a braid.
- **Tropical geometry:** Rauzy induction and continued fractions are piecewise linear;
  tropicalisation may turn the induction into linear algebra.

**Analysis and probability.**
- **Additive combinatorics and discrepancy:** the three-distance theorem and Fourier methods
  (as in the claimed proof's toy setting). Bounds on how runs distribute.
- **Free probability and random matrices:** the spectral statistics of LF's permutation matrix
  restricted to thin rows.
- **Optimal transport:** LF as a transport map between two orders of the same multiset.

**Computation and learning.**
- **Learned indexes** (Kraska et al.; PGM-index, Ferragina–Vinciguerra): LF is piecewise linear
  with r pieces. Learned piecewise-linear models predict positions with bounded error, and a
  learned global coordinate with exact correction is a practitioner's version of the toy's kα.
- **SAT/SMT and program synthesis:** search for the smallest exact construction on tiny blocks
  (the charter's small-instance generator), at scale.
- **Category theory and operads:** emission as composition. Operads describe which compositions
  can be done in parallel.
- **Compressed sensing:** recover a sparse object from few measurements. Is a block's
  irregular part sparse in some basis?

## Priority

Everything above should be attempted. If an order is needed, two rows are untouched by anyone here
and speak directly to the corrected toy-transfer question (cheap global coordinates):
1. **Rauzy fractals.** The natural toy above Sturmian: substitutive words with a torus coordinate.
2. **Veech groups.** Our block is already an origami, so its affine images are computable,
   alternative coordinates on the same object.

The decisive test in each case is acquisition: can the coordinate be read off the compressed
block for less than the walk it replaces?

## Composition of worlds (Louis's discussion with ChatGPT, relayed)

As relayed, the claimed proof's theories each discharge a different obligation: a representation,
a lifting mechanism, a bound, a structural property. The result is their composition. For us, the
question is whether some obligation disappears only by passing through several worlds, for
example:
1. the interval exchange exposes piecewise translations;
2. symbolic dynamics groups the orbit segments that share a representation;
3. an algebraic composition law resolves them collectively;
4. a compact executable form writes the output.

This chain is hypothetical. METHOD.md §1 now carries the rule ("Mathematical maximalism").

## The message for Carbon (relayed from ChatGPT, for its next cycle boundary)

```text
We are explicitly expanding the mathematical ambition of this programme. Do not restrict yourself to
compression algorithms, interval exchanges, graph algorithms, or the mathematical doorways we have
already identified.

Explore the possibility that exact inverse-BWT reconstruction requires a genuinely
interdisciplinary construction involving many mathematical theories, potentially far removed from
algorithms and data compression.

Levent Alpöge's claimed RH/GRH argument is a useful provocation: its reported architecture draws on
automorphic forms, theta lifts, Galois cohomology, arithmetic geometry, representation theory and
numerous deep structural theorems. The lesson is not to imitate those exact subjects. It is to
recognise that combining a very large mathematical toolkit may enable a result inaccessible to any
one toolkit.

You are authorised to investigate elaborate chains of mathematical reductions, to develop auxiliary
theorems that do not immediately improve a benchmark, and to search outside the fields represented
in our current research literature.

Do not impose an artificial limit on mathematical distance or complexity. The final executable
decoder must remain economical, but the mathematics required to discover it need not be.

Treat our current computational obligations as the final economic test, not a boundary on what
mathematics you're allowed to invent.

The charter now says this (METHOD.md §1, "Mathematical maximalism"); the kitchen-sink journal lists
about 25 frameworks with the bridge each might attempt, as a starting supply, not a menu.
```
