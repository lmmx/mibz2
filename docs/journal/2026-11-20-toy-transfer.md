# Toy, missing ingredient, native surrogate: a method note, and one quick negative

*20 November 2026. Prompted by Levent Alpöge's announcement note (alpo.ge/remark.pdf) of a
claimed, Lean-formalised proof of the (Generalized) Riemann Hypothesis found with Claude. The claim
is his and is not checked here. This note takes only the method he describes.*

## The method, as the note describes it

1. **A toy setting where the theorem is already known.** The Riemann Hypothesis over function
   fields (Weil, Deligne; Stepanov's elementary method). He re-proved it with a new, "sign-only"
   argument whose ingredients were chosen so that each might exist over the integers.
2. **Name the ingredient the hard setting lacks.** Stepanov's auxiliary polynomial uses
   multiplicity, which over the integers is "something like differentiating whole numbers".
3. **Find a native surrogate for it.** Chains of theta lifts, each lift mimicking one order of
   multiplicity, kept "natively automorphic".
4. **Size the surrogate from what it must mimic.** A chain of length √X for a sum of length X:
   "what else could it be?"
5. **A formal arbiter rather than prose.** Only the Lean development is published.
6. **Division of labour.** The human held the analogy and the direction; Claude supplied the
   technical mastery of higher-rank automorphic forms that the human lacked.

## Its translation for mibz2

1. **Our function field.** Perfectly clustering words: a BWT with one run per letter. These are
   exactly the codings of symmetric discrete interval exchanges (Ferenczi and Zamboni, *Clustering
   words and interval exchanges*, 2013). In the two-letter case (Christoffel and Sturmian words;
   Mantaci, Restivo and Sciortino, 2003), the text is mechanical: letter k is
   ⌊(k+1)α + ρ⌋ − ⌊kα + ρ⌋. So every output position is *computed*, with no walk and no
   dependence between positions.

   The task is to re-derive that toy inversion with ingredients chosen to transfer. One
   candidate is Ostrowski numeration, generalised to Rauzy–Veech and Bratteli–Vershik
   coordinates, which gives random access to the symbol at position k.
2. **The ingredient real blocks lack.** The working-backwards measurement says what it is: run
   width. The JSON skeleton has width to spare. Unique random strings (statement GUIDs) have
   none: they hold 54–67% of the one-row runs.
3. **A native surrogate for width.** This is the open question. One obvious surrogate was tried
   tonight (below) and failed.

## The quick negative: independent chains as a surrogate for width

**Hypothesis.** The plain inverse already walks its LF cycle as 32 independent lanes (512
segments), which converts dependent-load latency into memory-level parallelism (`src/bwt.rs`,
`walk_segments_room`). Open's `emit` walks its retained rows as one serial chain (#656 tested only
software pipelining of that one chain). If Open's emission were latency-bound, K lanes over
segments of the retained cycle would act as width for the one-row runs.

**Test.** `g5_lanes.c`: Neon's cycle-5 C harness with a segmented K-lane emit added.
- **The walk:** about m/2048 + 64 pseudo-random segment starts plus the mark. Each segment is
  walked into a lane arena, and the segments are joined from the mark's.
- **The comparison:** the serial emit, best of 5.
- **The output:** byte-exact on all six blocks.

**Results** (ms on this VM, best of 5; one run per block):

| block | m | serial | K=1 | K=4 | K=8 | K=16 | K=32 |
|---|---:|---:|---:|---:|---:|---:|---:|
| 0 | 89,874 | 0.46 | 0.85 | 0.82 | 0.82 | 0.86 | 0.95 |
| 2 | 112,070 | 0.65 | 1.11 | 1.08 | 1.11 | 1.15 | 1.29 |
| 85 | 156,404 | 0.87 | 1.44 | 1.25 | 1.23 | 1.28 | 1.32 |
| 143 | 196,841 | 1.32 | 2.12 | 2.15 | 2.11 | 2.17 | 2.22 |
| 207 | 135,921 | 0.85 | 1.39 | 1.32 | 1.43 | 1.41 | 1.54 |
| 210 | 204,281 | 1.28 | 2.00 | 1.92 | 1.89 | 2.00 | 1.94 |

**Verdict: no.**
- **Lanes buy at most 5–14% over one lane,** and the segmenting and joining cost about 1.6–1.8×
  the serial emit.
- **Open's emission is not latency-bound** in this harness. About ten bytes are appended per
  retained step (n/m ≈ 10), and the appends dominate, not the edge load.
- **Scope:** this kills lanes as a surrogate for width in Open's emit. It says nothing about
  surrogates aimed at the construction phases.
- **Measured** (C harness, this VM, not the native Rust decoder on the i9).
