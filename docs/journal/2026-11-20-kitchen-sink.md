# The kitchen sink: heavy frameworks, each with a named bridge

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

## The lesson we take

Not "use many fields". Three narrower points:
1. **The cost of importing heavy machinery has collapsed.** A theorem that once took a specialist
   a year to bring in can now be proved inside the project. Timidity about distant fields is no
   longer an economy.
2. **Every import has a named job.** Each framework sits in the stack because one step needs
   exactly it. A kitchen sink with no bridge is a reading list.
3. **A non-standard route through standard results** can be where the novelty is: here, Galois
   representations derived via theta lifts rather than as in the literature.

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

## Priority

Two rows are untouched by anyone here and speak directly to the corrected toy-transfer question
(cheap global coordinates):
1. **Rauzy fractals.** The natural toy above Sturmian: substitutive words with a torus coordinate.
2. **Veech groups.** Our block is already an origami, so its affine images are computable,
   alternative coordinates on the same object.

The decisive test in each case is acquisition: can the coordinate be read off the compressed
block for less than the walk it replaces?
