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
- **What it shows, and what it doesn't:** in this harness, the overhead of segmenting and joining
  outweighs any latency benefit. The small gain from one lane to eight suggests that the
  dependent load is not the main cost: about ten bytes are appended per retained step (n/m ≈ 10).
  It does not establish the hardware bottleneck; that would need counters.
- **Scope:** lanes are traction, not transfer: they follow the same edges concurrently. This
  kills them as a cheap speed-up of Open's emit. It says nothing about
  surrogates aimed at the construction phases.
- **Measured** (C harness, this VM, not the native Rust decoder on the i9).

## Additions from Louis's discussion with ChatGPT (relayed)

- **Transfer the effect, not the algorithm.** Rather than generalise the toy's algorithm, ask
  what different mechanism in the hard setting produces the same effect.
- **Damage the toy deliberately.** Find the theorem that makes the toy easy. Relax its assumptions
  one at a time until it fails, and record which assumption carried the effect.
- **Amplification by composition.** Look for an exact operation whose accumulated benefit grows
  faster than its complete cost when it is repeated.
- **Discovery complexity is not execution complexity.** A long proof can justify a cheap
  operation, and a short theorem can justify an expensive one.

**The Coordinator's reading of amplification.**
- **Cells already amplifies.** Rauzy induction composes return words, so word lengths grow
  geometrically while the interval count stays r: about n/r bytes per step.
- **The cost that remains is r itself.** The working-backwards measurement says r's hardest part
  is unique random strings.
- **What any new amplification must do:** act on those runs *collectively*. Under induction they
  never merge, because they share no continuation.
- **What they do share:** format, not content. In a version-4 UUID the dashes sit at offsets 8,
  13, 18 and 23, the version nibble is always 4, and the variant nibble is one of 8, 9, A and B.
  Every GUID also uses the same 16-symbol alphabet.
- **The question:** can a shared format, rather than shared content, be the native surrogate for
  width? This is a conjecture, unexamined.

## The lead as relayed, for Carbon at its next handoff

```text
A mathematical lead for a future foundational cycle, inspired by Levent Alpöge's November 2026
account of his claimed RH/GRH proof with Claude (https://alpo.ge/remark.pdf).

The interesting methodology is not the specific number theory. It is the reported transfer of a
powerful operation from a tractable toy setting into a much harder setting by discovering a
completely different mechanism with the same mathematical effect.

I want you to investigate that pattern for exact inverse-BWT reconstruction.

Find a tractable family of BWTs in which an unusually powerful collective reconstruction operation
is available. Establish exactly what makes it work. Then relax the assumptions until that
capability disappears.

Rather than merely generalising the original algorithm, ask whether mathematics from another
setting could restore the missing capability through a different primitive.

Pay particular attention to effects that can be amplified by composition: an exact operation that
resolves progressively more ordering relationships without comparable growth in construction or
consumption costs.

You are free to choose the toy family, the mathematical effect and the replacement theory. The
examples in our research history are starting points, not restrictions.

The desired result is a new exact computational principle, or a precise obstruction explaining why
the proposed transfer cannot work.

Keep the existing verification and complete-cost standards. No charter changes or production
integration. Treat Alpöge's claimed RH proof as unverified pending independent scrutiny; we're
studying the reported discovery process, not assuming its final theorem is established.

Read docs/journal/2026-11-20-toy-transfer.md (including its correction: the effect to transfer is cheap
global coordinate evaluation, not run width) and docs/journal/2026-11-20-working-backwards.md first:
they name a candidate toy (perfectly clustering BWTs), the measured missing ingredient (run width,
absent in unique random strings), and one surrogate already refuted (independent lanes for Open's
emit).
```

## Correction: the effect to transfer is global coordinates, not width

Louis, relaying ChatGPT, corrected step 2 above.
- **What the toy really has:** run width is what today's algorithms exploit. The toy's stronger
  capability is *cheap global coordinate evaluation*: the symbol at any output position follows
  from a compact arithmetic coordinate (kα mod 1), so the orbit never has to be followed.
- **The two can separate.** A block could have mostly singleton runs and still have an LF given by
  a simple arithmetic map. A block with wide runs need have no closed form.
- **So the surrogate to seek** is for global coordinates, or for cheap composition of long orbit
  segments, not for width.
- **The acquisition condition holds:** the coordinates must be derived cheaply from the compressed
  input, and evaluating them must cost less than following LF.

**A family that separates the two (the Coordinator's suggestion, unexamined).**
- **The toy can't tell them apart:** in the two-letter case, wide runs and global coordinates
  coincide.
- **The separating family:** codings of a *rotation* by an r-piece partition. The orbit
  coordinate stays x + kα, but these words are not perfectly clustering (Ferenczi–Zamboni), so
  their BWTs have more runs.
- **The first damage step:** measure how runs grow with r while the arithmetic coordinate
  survives. That is "damage the toy" with the coordinate kept and the width destroyed.

## The damaged toy, measured

`toy.py` codes a rotation by the golden-ratio slope, with the circle cut into p pieces, on 5,000
letters. It counts the runs of the BWT's last column.

| Pieces p | Runs | Mean width (rows) |
|---:|---:|---:|
| 2 | 10 | 500 |
| 8 | 46 | 109 |
| 64 | 101 | 50 |
| 256 | 276 | 18 |
| 1,024 | 1,036 | 4.8 |

**Measured.** The 2-piece row is a Sturmian prefix, and its finite length adds runs to the ideal
two. The 3/8 Christoffel word `aabaabab` has the BWT `bbbaaaaa`.

**What it shows.**
- **A few pieces barely damage the toy.** The "first damage step" proposed above is too weak.
- **About a thousand pieces destroy the width.** Runs average about 5 rows, close to a random
  string's.
- **The coordinate survives.** Letter k is still one binary search on (x + kα) mod 1.
- **So the two separate:** a block can be almost all narrow runs and still have a global
  coordinate.

## Correction: lanes do pay, once the append is branchless (Carbon cycle 2)

The lanes negative above was too broad.
- **Carbon's finding:** cycle 2 (`research/carbon-word-transport`, 8b54b024) found that lanes and
  branchless emission interact.
- **The mechanism:** each code's word is stored as one pre-aligned 16-byte vector, so 92–94% of
  steps are a single unconditional 16-byte store.
- **The interaction:** with that change, 8 lanes cut Open's emit to 0.55–0.78×, though either
  change alone barely moves it.
- **Why my test missed it:** it used branchy, variable-length appends, so it measured the
  appends, not the lanes.

**The Coordinator's rerun of `g5_bvsmc.c`.** Run on the six blocks on this machine (output in
`bvsmc-rerun.txt`), it was byte-exact with canaries passing.
- **Emit:** BVSMC-1P is 0.61–0.77× Open.
- **Whole kernel:** 0.66–0.78× on five blocks. Block 207 came out at 0.92×, a noisy round on a
  shared machine.
- **The strengthened single-lane arm** (U16Q, a fixed 16-wide queue store plus the 16-byte emit):
  0.77–0.92× whole kernel, with no extra memory.
- **The verdict:** Carbon's pre-registered G5 gate fails on its strict all-blocks emit criterion,
  here as in Carbon's own run.
- **Scope:** measured in a C harness, not the native Rust decoder on the i9.

## Correction: the damaged-toy table was an alphabet artefact

Carbon's cycle 4 (`research/carbon-word-transport`, da079dde) pointed out that `toy.py` gave each
piece its own letter. At p = 1,024 that is a 1,024-letter alphabet, so r ≈ σ, and the table's run
count mostly counted letters.

`toy2.py` reruns it with two letters, each piece labelled a or b at random, on 5,000 letters:

| Pieces p | Runs | Mean width |
|---:|---:|---:|
| 2 | 26 | 192 |
| 8 | 44 | 114 |
| 64 | 218 | 23 |
| 256 | 802 | 6.2 |
| 1,024 | 2,018 | 2.5 |

**Measured.** The conclusion survives, and is stronger: width collapses while the coordinate survives.

**Carbon's limit, which stands.** A coded rotation has zero topological entropy. A positive-entropy
block admits no global coordinate shorter than its entropy, so a coordinate can at best cover the
structured part.

*Update, 21 November 2026:* on 20 November the Lean Zulip thread "Discussion: Alpöge Proves the
Riemann Hypothesis" reported that three independent kernels (nanoda, con-leche `--verified`, pestka)
accepted the Lean development, which proves RH and GRH for Dirichlet L-functions. We have not checked
it ourselves. The remark's toy proof is not part of the Lean development. Nothing above depends on it.
