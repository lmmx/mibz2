# Research portfolio: the search for a full-decoder breakthrough on large-block Wikidata bz2

**Status:** a planning document for review. Nothing in it is authorised for
implementation, benchmarking or workstation (i9) use. Each item names the cheapest test
that would decide it, and the approval it would need.

**Objective (the maintainer's brief):**
- an algorithmic gain that substantially raises full-decoder throughput on existing -9
  Wikidata `.bz2`;
- preferably at equal or lower CPU-seconds per GB, and with no material rise in peak RSS;
- the baseline is the shipped cooperative decoder, mibz2 0.2.1 (`c217676`).

**Companion files:**
- `2026-10-30-research-portfolio-register.md`: the deduplicated register. It holds every
  hypothesis and experiment since 0.1.0, with its status, measurement boundary,
  condition, failure class and sources, plus the recorded question lists and the merged
  contradictions.
- `2026-10-30-external-report-4.md`, `-5.md`, `-6.md` and `-7.md`: the inputs this
  round, verbatim.
  - Report 5 holds the researcher's and the guardian's corrections to the coordinator's
    first reading.
  - Report 6 is the researcher's study session on the algebra of inverse MTF, and their
    reading of the bzip2gpu source.
  - Report 7 is their reading session II, on recency and order statistics.

**Boundaries.** Four boundaries are kept apart throughout, and a figure is never carried
from one to another:
- **kernel:** the inverse BWT's build and walk alone;
- **decoder stage:** `examples/phases.rs`, one core;
- **full decoder:** `ab run`, or the i9's `bench/variants.py`;
- **application:** wikidata-pq's split or its whole run.

Each item is also labelled *measured* or *model*. A model says what it assumes.

## 1. Corrections taken in

The researcher and the guardian corrected the coordinator's first reading of external
report 4. Both corrections stand, and the register carries them.

1. **Interval composition is not settled by M2 (#42).**
   - The bound |B_k| ≤ k·|B| on Ψ^k's breakpoints is a worst case. It is not a prediction
     for Wikidata.
   - M2's ~41 instructions a step is a cost of M2's lookup, a move structure with
     fast-forwards that advances one byte per step. It is not a lower bound for a
     shortcut that advances k bytes per lookup.
   - **Classification:** related prior work; an open structural hypothesis.
     Implementation is conditional on a construction, a byte-recovery method and a
     hot-path cost model that differ materially from M2's (§3.1).
2. **Composing reverse MTF is not refuted by #25.**
   - #25 rejected a register-held implementation of the ordinary MTF update. It did not
     test chunk-level composition of MTF's state transformations.
   - **Classification:** open structural hypothesis, with no failure class assigned. It
     comes with a census and a strict test of total work (§3.2).
   - The coordinator's "probably experimentally inferior" is withdrawn.
3. **General rule, adopted for the register:** a failed implementation is evidence
   against that implementation. It closes a mathematical direction only when the failure
   is shown to hold for every representation of it: failure class I, or class II with
   the budget argument made.

## 2. Reconciliations done this round (read-only)

### 2.1 #93 against #114: acceptance-criteria drift, then a trade-off accepted knowingly

Both are the same design: `bwt::build` writes each entry whole, with no init pass.
#114 adds a check on every write, a two-pass fallback for foreign counts, and a Miri
test. Read from the PRs, their bench comments and `2026-10-29-one-pass-build.md`:

| | #93 (open, never merged) | #114 (merged as `6a341f6`, shipped in 0.2.1) |
|---|---|---|
| Registered pass | single-core build ≥ 1.10× on CI; fail < 1.03× | the build stage faster, its interval above 1; no thread count slower; the same bytes; plus an i9 arm |
| Merge rule stated | "an end-to-end figure improves beyond noise" | the registered criteria; a whole-decoder gain "hoped for, not required" |
| CI build stage, one thread | −2.6% (−2.9%, −2.2%), i.e. 1.027×: below the fail line | −6.0% (−8.5%, −3.5%) |
| CI whole decoder, Wikidata | +0.2% / +0.0% / +0.0% at 1 / 2 / 4 threads (noise) | +0.8% at 1, +0.8% at 4 (noise) |
| CI CPU-seconds | assessment "Better (contention sensitivity)"; amplification −1.1% at 2 threads | assessment "Worse (CPU cost)": Wikidata +1.1% (+0.2%, +2.1%) at 4 threads; records +1.0% at 2, +2.0% at 4 |
| i9, full decoder, against v0.2.0 | not run | speed 1.048 at 1 thread, 1.035 at 4; CPU 0.954 at 1, 0.971 at 4, both below 1; 6–10 threads (teams) within noise as the control |
| Decision | held: no end-to-end figure moved | "a separate engineering decision, made by the user after both results", weighing the i9's gain at lower CPU against CI's flat wall and small CPU rise |

**Verdict.** Two things happened. Neither is a hidden regression.
1. **The acceptance rule drifted.** #93's single-core floor (≥ 1.03×, and an end-to-end
   gain to merge) was replaced in #114 by "faster with its interval above 1", and the
   end-to-end gain became optional. The journal states the change openly, but no record
   says why the floor was dropped.
2. **The trade-off was accepted knowingly.** The two machines disagree:
   - the i9 shows a whole-decoder gain of 3.5–4.8% at less CPU, on physical cores;
   - CI shows CPU +1–2% where the 4-thread runs share SMT siblings.

   Both results stay in the record. The mechanism (L's extra loads competing with the
   SMT sibling) is a hypothesis.

**Against the breakthrough objective** (equal or lower CPU per GB): on the i9, #114
lowers CPU per GB. On a 2-core SMT machine it raises CPU per GB by about 1–2%, at
unchanged wall. The 4-core release target has 4 physical cores, so it is closer to the
i9's case, but it is unmeasured.

**Open housekeeping:** #93 is still open, although its change is in master. Closing it as
superseded by #114 needs the maintainer.

**A finding about the record itself.** The two runs' bases differ:
- #93's base: walk 5.05 c/B, total 11.29, 272 MB/s at 1 thread;
- #114's base: walk 6.64 c/B, total 12.98, 238 MB/s at 1 thread.

Both ran on "EPYC 7763, 4 cores". Their walks differ by 31% in absolute cost, so absolute
c/B are not comparable from one CI run to another. Only paired ratios within a run are.
The cause is unknown: a code change between bases `0e6ab25` and the 0.2.0-era base, or
tenants on the runner's shared L3. The memory-bound stage moving most points to the
second. Every CI figure below is read within its own run.

### 2.2 The super-alphabet (Ψ²) construction: the pair-count pathway was tested

The guardian asked whether mibz2's failed Ψ² work tested Kärkkäinen–Kempa–Puglisi's
optimised construction, or only the explanatory gather. The record shows both, with
distinct results:

| Attempt | Construction | Result |
|---|---|---|
| #29 | `LF[LF[i]]` gather pass, 8-byte pair table | CI full decoder +9.3% / +20.7% at 1 / 4 threads. The literature reading recorded "#29 tested constructions their papers do not use" |
| Earlier ψ² port | sequential bigram histogram | build 16.6 → 12.5 c/B (stage, local) |
| H1 cheap test | libsais-style ψ² build | 18.7 c/B unchecked against `build`'s 6.1 (local kernel); with a sequential bigram histogram, 12.46 against 5.68 |
| #34 | a faithful port of libsais 2.12.1: one bigram counting sort (65,536 `bucket2` counters), bytes recovered from `fastbits` + `bucket2` | build 7.1–7.6 against 2.5 c/B; walk ≈10 against ≈10.4–10.8; total 17.4–17.6 against 13.1 (stage, local VM). CI full decoder +40.6% at 1 thread, +24.3% at 4 |
| E1 (i9) | the pair walk in the 0.2.x harness | 0.691× at 1 thread, 1.107–1.117× at 8–10 workers against its own control; best against best 0.856× |

So the counts-based pathway was tested, in libsais's implementation. Two things remain
untested:
1. KKP's own implementation, as distinct from libsais's. A diff of the two is a
   paper-and-code task (theory track T4).
2. The decomposition the guardian asked for: builder against representation. #34
   answers it in the 0.1.4 flagged walk:
   - the builder cost about +5 c/B;
   - the pair walk saved only 0.4–0.8 c/B.

   **Consequence:** a free builder would have left Ψ² at about −4–7% of the walk under
   that walk. That is a gain, but nowhere near a breakthrough. The representation's
   per-step economy is the binding constraint ("half the steps, but each costs about
   twice as much"), not the builder alone.

**Status:**
- rejected for the shipped design;
- conditional past the i9's knee (E1);
- reopening (as recorded): a pair step cheaper than two single steps, or a walk that
  becomes latency-bound.
- **Failure class:** III (experimentally inferior, on the per-step instruction economy
  under MLP), with an architecture-specific component (IV) past the knee. The register
  marks the class as judgement.

### 2.3 bzip2gpu (Weißenberger and Schmidt, ICPP 2024): no total-work reduction

This is prior art only. The work's basis is unchanged: ordinary CPUs (four-core laptops,
with the i9 for controlled experiments), Rust, existing `.bz2` files, and baseline mibz2
0.2.1. The researcher confirms this in external report 6.

The source was read at `66d263c`, from a shallow anonymous clone; nothing was built or
run. The researcher read the same source independently (external report 6). The two
readings agree, and the details below include theirs, checked against the code.

**Inverse MTF** (`src/imtf.cu`):
- **Per-chunk summaries:** each chunk is 10,000 ranks (`PRIVATE_PERM_SIZE`; the 64 is
  the CUDA thread-block size). Each chunk's summary is a full 256-entry permutation.
  - `private_perm` (Algorithm 4, the one launched) simulates ordinary MTF on an identity
    list in shared memory. That is the same shift work as the sequential decoder.
  - `private_perm_2` (Algorithm 5) is commented out at its launch site. It would track
    each of 256 list positions through the chunk with one thread per position,
    `if (idx < next) ++idx; else if (idx == next) idx = 0`. That is O(256 × chunk) work.
- **Composition:** the summaries are composed by a Blelloch scan, `concat`, at 256
  operations per composition.
- **Output:** a second full pass (`private_perm_write` / `_2`) re-runs MTF from the right
  incoming list to emit the symbols.
- **Total work:** at least twice the sequential MTF, plus O(256 · chunks · log chunks).
- **Not used:** the rank-prefix property the researcher states (§3.2). Every summary is
  256 wide.

**Inverse BWT** (`src/ibwt.cu`):
- **The permutation:** built by a general sort,
  `cub::DeviceSegmentedSort::StableSortPairs`, not by a counting scatter.
- **Method:** Helman–JáJá list ranking with splitters every `LIST_DISTANCE = 32` rows,
  about 28,125 sublists a 900,000-row block. Random splitters are a README TODO.
  - local walks from the splitters, each element's sublist id and local rank written
    (2 × 4 B an element);
  - a serial reduce over the splitters;
  - a full permute pass, `add_permute`.
- **Total work:** every element is still reached by a dependent pointer chase, plus two
  n-sized side arrays and a scatter.
- **Relation to mibz2:** the shape is the segmented walk plus assembly that mibz2 already
  runs (32 lanes, 512 segments, assembly 0.23 c/B on CI).

**Huffman and RLE2** (`src/decoder.cu`): `decode_block<<<num_blocks, 1, …>>>` runs one
GPU thread per bzip2 block. So the front end is parallel across blocks only, as
mibz2's scanner is not.

**Verdict:**
- both stages are class II in external report 4's sense: they reduce depth, not work;
- neither carries a work-reducing component a 4-core CPU could take;
- its value to the portfolio is as the implementation-backed precedent for the
  scan-composition of reverse MTF, and as evidence that the obvious version of it costs
  at least twice the work.

### 2.4 The "~30 open questions"

No such list exists in any branch, issue, PR or journal. The succinct review §0
(`research/psi-census`) records that the research assistant's original log had "30 open
questions", but only its conclusions were kept. The register's §3 merges every recorded
question list instead, with each question's status.

## 3. Theory: new results and refinements

### 3.1 Composing Ψ: an exact identity, two bounds, and free byte recovery

**Notation.**
- Ψ maps row j to the row of the next suffix.
- The walk at row j emits F[j], the first byte of row j's rotation.
- Let B be the set of j such that Ψ(j+1) ≠ Ψ(j) + 1, **together with every F-bucket
  boundary** (j the last row of its byte's bucket). The second part adds at most 255
  positions.
- "j ∈ B" means "a break between rows j and j+1".
- Let B_k be the same set for Ψ^k.

**(a) An exact identity.** B_k = ⋃_{t=0}^{k−1} Ψ^{−t}(B). The researcher's "⊆" is an
equality.

*Proof.*
- **⊆:** Suppose j ∈ B_k. Take the least s ≤ k with Ψ^s(j+1) ≠ Ψ^s(j) + 1. Then
  Ψ^{s−1}(j) and Ψ^{s−1}(j+1) are adjacent and Ψ separates them, so
  Ψ^{s−1}(j) ∈ B.
- **⊇:** Suppose Ψ^t(j) ∈ B. Either some earlier step already separated j and j+1, or
  they are adjacent at step t and Ψ separates them there. Either way j ∈ B_k. ∎

So the census quantity is exactly the overlap of the preimages of the breakpoints:
|B_k| = |⋃_{t<k} Ψ^{−t}(B)|. This is the researcher's open question, made precise.

**(b) Two bounds:**

d_k − 1 ≤ |B_k| ≤ k·|B|

- d_k is the number of distinct cyclic k-grams in the block.
- **Upper bound:** each preimage set has |B| elements.
- **Lower bound:** suppose rows j and j+1 share fewer than k leading bytes, LCP < k.
  Then within k steps they fall into different F-buckets, and a bucket boundary lies in
  B, so j ∈ B_k. The rows with LCP(j, j+1) < k number d_k − 1.
- The lower bound needs no BWT. It can falsify cheaply from k-gram counts alone: if d_k
  is close to n for small k, Ψ^k has close to one interval a row, and the composed
  representation is a per-row table. That is H1's representation-budget failure,
  generalised.

**(c) The bytes are free to locate.** This refines the researcher's "an interval
representation of Ψ^k does not automatically recover the k output bytes".
- Let I be an interval of Ψ^k. For every t < k, Ψ^t(I) is contiguous and lies inside one
  interval of Ψ. Because B includes the bucket boundaries, that is inside one F-bucket.
- So **the k bytes emitted from any row of I are the same string**, and Ψ^k on I is a
  single translation j ↦ j + D_I.
- A record (D_I, k bytes) recovers all k bytes with no per-row information. The
  information cost the researcher names is real, but bounded: k bytes per interval,
  k·|B_k| in all.
- This is the interval analogue of how KKP and libsais recover a pair from its row's
  `bucket2` position.

**(d) What makes it materially different from M2.** This is the guardian's
novelty condition:

| | M2 (#42, rejected locally) | Composed table P^k (proposed for the census only) |
|---|---|---|
| Bytes per lookup | 1 | k (2–8) |
| Lookup | move structure: interval index carried, fast-forward scan (the select arm was −60% for one chain) | rank over a bitvector of B_k: one 16-byte interleaved {count, 64 bits} word, then one record |
| Record | (interval, offset), 8 bytes | (D_I: u32, k bytes): 8 bytes for k ≤ 4, 12 for k ≤ 8 |
| Size, bound (r ≈ 56,000) | 0.64–1.07 B/row | rank words ≈ 225 KB, plus records ≤ k·r·(4 + k) B: k=2 ≤ 0.9 MB, k=4 ≤ 1.8 MB, k=8 ≤ 5.4 MB, against the T-vector's 3.6 MB |
| Build | from the run list: 7.8–9.2 c/B untuned, against 2.7 | from L's runs: B_1 in O(r). Then B_{t+1} = B_t ∪ Ψ^{−1}(B_t \ B_{t−1}): only new points need an LF, at most |B_k| LF evaluations in all. Records by doubling: (D, bytes) for Ψ^{2a} from two Ψ^a records, about 2|B_k| lookups for k a power of 2. No T-vector, and no n-sized scatter |

**Precedent check owed before any novelty claim** (T4):
- k = 1 with a rank lookup is the classical run-length LF of the r-index literature;
- M2's `examples/movest.rs` select arm may already be it, so check before claiming a
  difference;
- k-step composition is not in what the corpus has read (Nishimoto–Tabei; Brown–Langmead
  SEA 2026; libsais; KKP), but Move-r and Movi have not been read for it.

**(e) A cost model.** All of it is model: instruction counts are estimates, and no
timing is claimed.
- **Walk:**
  - today's room walk: ≈16.9 instructions a byte (i9 profile, r-arms);
  - P^k: ≈10–14 instructions a lookup (shift, a 16-byte load, popcount, add, a record
    load, add, a store of up to 8 bytes, lane bookkeeping), so ≈2.5–3.5 a byte at k = 4;
  - misses past L2: 1/k a byte against 1.
- **Build:**
  - today: 6.65 instructions a byte;
  - P^4: at most ≈3|B_4| operations of ≈20 instructions each over n rows. That is
    ≈7 a byte when overlap is heavy (|B_4| ≈ 100,000) and ≈15 at the bound
    (|B_4| = 4r ≈ 224,000).
  - **The build is the weak point.** At the upper bound it costs twice today's.
- **Inversion, net:** today ≈23.6 instructions a byte; P^4 ≈10–18.5.

**Stop rule** (pre-implementation, model-based): if, for every k ∈ {2, 3, 4}, the
census's measured |B_k| gives a modelled net of more than 0.67 × today's inversion
instructions a byte on the median block, stop. Record it as a class II failure (the
budget, measured), and keep it open only as a data-structure problem, as the guardian
asked.

**(f) What it would enable: the run-native pipeline (N3, a model).**
- **What the run list removes:** P^k is built from L's runs. MTF+RLE2 already produce
  those runs: each L-run is one nonzero rank followed by its RUNA/RUNB zero-run. So the
  decoder need never write the n-byte L:
  - the MTF stage's per-byte expansion goes;
  - the T-vector build (2.16 c/B) goes;
  - the walk's per-byte loads go.
- **What stays per byte:** the output's writes, RLE1 (0.29) and CRC (0.26).
- **Model:** one-core c/B from 12.93 to ~5–7, a speedup of 1.8–2.5×. Its assumptions:
  - the census lands mid-bound;
  - the MTF stage is mostly per-token (§3.2);
  - construction ≈ today's build.
- **Fallback:** a block with high r (random-like data; Wikidata blocks reach
  r ≈ 180,000) falls back to the T-vector, chosen per block from r, which is known
  after MTF.
- This is the most structural candidate in the portfolio: it removes two passes and a
  3.6 MB materialisation a block.

### 3.2 Reverse MTF: what bounds composition's value on Wikidata

**The researcher's two exact results** (external reports 5 and 6):
- **The prefix lemma:** a chunk with maximum rank m acts as the identity outside the
  first m + 1 positions.
- **The stable move-to-front characterisation:** L_out = D ‖ (L_in \ D).
  - D is the chunk's distinct selected items, in decreasing order of last selection.
    Every other item keeps its incoming order.
  - So a chunk's state change needs d = |D| identities (as positions in L_in), not a
    256-entry permutation. It can be small even when m is large.
  - The summaries are closed under composition. Composing two of them means translating
    the second's identities through the first, then removing duplicates.

**The researcher's split of the hypothesis, adopted:**
- **(i) compact boundary-state composition:** useful for parallelism, and mathematically
  interesting;
- **(ii) work-reducing symbol reconstruction:** the only route to a breakthrough at equal
  or lower CPU.

**A refinement of (ii) by the coordinator, to be checked.** Symbolically, a chunk of q
operations emits:
- d *first touches,* each a reference into L_in;
- q − d *re-touches,* each a reference into the chunk's own recency list, which holds at
  most d items.

The re-touches are themselves MTF over a list of at most d items. So the emitted
sequence still costs one operation per token, on a smaller list. Shrinking the list
reduces the cost per operation, not the number of operations. For (ii) to reduce work,
whole token sequences must go untouched (§3.2 point 2 below).

**Reading session II** (external report 7) reaches the same point independently.
- **What it sets aside:**
  - timestamp and order-statistic trees (SwiftShader), as an alternative implementation
    rather than less work;
  - Move-From-Front (Niemi–Teuhola 2020), which needs the final MTF list, and that is
    not in a `.bz2`;
  - the hot-prefix register hybrid, which stays closed by #25 unless a new mechanism
    batches several ranks for less total work.
- **The condition it keeps:** a representation S(R) yielding both T_R (the state change)
  and O_R (the emitted symbols) for less work than interpreting R. The next reading
  shelf is small-state transducers, Four-Russians tabulation and the composition of
  frequent rank subsequences.
- **How it maps here:** this is the memoisation route of point 2 below, and T2's
  rank-n-gram repetition statistic is its cheapest falsifier.

The coordinator adds three facts that bound what composition can save. None of them
closes the hypothesis.

1. **The list changes once per L-run.** Rank 0 leaves the list unchanged, and a nonzero
   rank occurs exactly at an L-run's first byte. So a block does r − 1 nonzero MTF
   operations:
   - 47,935–179,971 a full Wikidata block;
   - 0.072 a byte overall (n/r = 13.8).

   The MTF+RLE2 stage costs 2.29 c/B on CI (#114's head, one thread). If all of that
   were list updates, it would be ≈32 cycles an update. The stage also:
   - fetches the scanner's tokens (one per run, plus ≈log₂ ℓ RUNA/RUNB tokens per run);
   - takes data-dependent branches per token;
   - expands the runs into L;
   - counts for the build;
   - writes checkpoints under teams.

   Composition can remove only the list-update share.
2. **Symbolic decoding does not reduce work.** Decoding a chunk against an unknown
   incoming list costs what the ordinary decode costs: bzip2gpu's second pass. A
   work-reducing variant needs one of two things:
   - (i) **memoisation:** the same rank sequence recurring often enough that a cached
     (prefix permutation, output positions) summary replaces its re-execution;
   - (ii) a representation where most tokens are never touched individually.

   For (ii), the run list of §3.1(f) already removes the per-byte part. What remains is
   the per-token cost.
3. **The teams case is better served by overlap than by parallel MTF.** Under teams, a
   helper idles during its block's MTF (≤ 0.11 of a 4×2 team's core-time). Splitting
   that MTF across the team by composition at least doubles its work. Overlapping one
   block's MTF with the previous block's inversion (the "batched team", C2) fills the
   same idle time with no added work.

**Ceiling:**
- **Algorithmic:** at most the list-update share of MTF+RLE2, which is itself ≤ 17.7% of
  a one-core decode.
- **Prior:** small on Wikidata, because updates occur once per ~14 bytes. That is a
  prior, not a measurement.
- **Decoder:** ≤ 1.2×, even at the share's maximum.

**Decisive tests (both read-only, T2):**
- **(a) A token census per block:**
  - nonzero-op count, checked against r − 1;
  - the rank histogram;
  - for chunks of q = 8…1,024 operations: max rank, distinct positions affected, and
    prefix-summary size;
  - the repetition rate of rank n-grams, which bounds memoisation's potential.
- **(b) Attribution of the stage's 2.29 c/B:** list updates against token dispatch,
  expansion and counting. This uses a local profile of the existing code (indicative
  only), or an instruction-count model from its disassembly.

**Stop rules:**
- if list updates are < 0.5 c/B, composition's ceiling is under 4% of a one-core decode;
  deprioritise it as low-ceiling, not refuted;
- if typical chunks of ≥ 16 operations reach a max rank > 32, compact summaries fail on
  size (class II, measured).

### 3.3 Ceilings, from the current stage table

All of these are models, from #114's CI run (EPYC 7763, one thread, 12.93 c/B in all).
Only in-run proportions are used (§2.1).

| Remove entirely | Share | One-core ceiling |
|---|--:|--:|
| walk | 51.8% | 2.07× |
| walk + build (the inversion) | 68.5% | 3.17× |
| MTF + RLE2 | 17.7% | 1.22× |
| scanner | 9.5% | 1.10× |

**The serial scanner caps every worker-side idea in the parallel decoder.** #114's model
gives:
- the scanner at 9.7 ns a bz2 byte, so ≈0.55 ns an output byte at 17.69×: about
  1.8 GB/s of output at CI's clock;
- the workers at 5.44 ns an output byte at 4 threads (CPU, summed), so 1.36 ns of wall a
  byte over 4.

**Headroom before the scanner binds:**
- **4-core target:** ≈2.5× (1.36 / 0.55);
- **i9:** ≈2.1×, from its decoder peak of ≈1.26 GB/s (register, Part G) against a
  scanner cap scaled by clock to ≈2.7 GB/s.

**Consequences:**
- **Above the headroom:** a worker-side breakthrough beyond ~2× on the i9 needs the
  scanner parallelised too: #10, speculative markers. #10 is therefore a prerequisite
  for any result above the headroom, not a standalone candidate.
- **The application:** the scanner is unmodelled there, and the next point bounds the
  value of all bz2 work.

**The application's ceiling is unknown, and it bounds everything.** In the i9's
release-shaped run (wpq #215, auto = 4), the split took 25 s of 103 s of wall. A perfect
decoder can save at most part of the split. Model: the decoder is ~11 GB of output at
≈1 GB/s, so ≈11 s, under 11% of that run's wall. The split's own parsing also takes time.

A decoder-free arm, N0, measures this bound directly. It is the highest-value
measurement in the portfolio for the maintainer's question, "if the bz2 work gives ANY
wikidata-pq speedup".

### 3.4 The 8-thread efficiency hypothesis: attribution, not policy

These are the measured inputs (wpq #215; i9; 8 paired rounds). Against auto (4), 8
threads (4×2 teams) gave:
- split wall −1.04 s (−4.2%);
- split CPU +40.6 s (+16.9%);
- `other_cpu_s` unchanged.

So the whole extra cost is inside the split.

That extra cost is the same order as the decoder's own CPU in the split. A model puts
that CPU at ~30–45 CPU-seconds for ~11 GB, so this is not a small spin overhead. A
per-handoff spin of 2,000 iterations over ~12,400 blocks is well under a second.

**Ranked hypotheses for where the 40 s go** (all "contention suspected, not
demonstrated"):
1. **SMT oversubscription with the split's own threads.** There are 18 parse workers, 18
   writer threads and 9 decoder threads on 20 logical CPUs. Parse workers' stalled
   cycles count as CPU. This is avoidable by a thread budget.
2. **Teams' shared-table traffic and L3 pressure,** at 4 blocks in flight in teams
   against 4 single-worker blocks. This is partly inherent latency-hiding cost.
3. **Helpers spinning or waking** under back-pressure from the consumer. This is
   avoidable.

**Decisive test** (application, i9 queue Q2): per-thread CPU by thread role (decoder
workers, helpers, scanner, parse workers, writers), from `/proc/<pid>/task/*/schedstat`
at the split's end, in both arms. Add the #116 occupancy trace under teams for the
helpers' busy, spin and park time.
- If the parse workers carry the extra CPU, the 8-thread wall gain may be had at lower
  CPU by trading parse workers for decoder threads. That would be a thread-budget
  experiment, not a decoder change.
- If the helpers carry it, the spin/park policy is the lever.

## 4. Ranked candidates

**Tracks:**
- **A:** fewer dependent transitions per byte;
- **B:** cheaper transitions;
- **C:** less non-navigation work;
- **D:** scheduling, MLP and contention;
- **N0:** application.

**Classes:**
- **I:** reduces work;
- **II:** restructures dependence (reduces depth, not work).

**How the ranks work:**
- **Science:** information gain per unit of cost.
- **Engineering:** potential at the target. The 4-core release target plans
  `Plan::Workers` (teams start at 6 physical cores), so per-worker work dominates there.
  The i9 plans teams from 6 threads.

**Ceilings:**
- **Algorithmic ceiling:** of the target stage.
- **Decoder ceiling:** Amdahl, with the scanner cap of §3.3.

| ID | Candidate | Track / class | Status | Science rank | Eng. rank, 4-core | Eng. rank, i9 | Algorithmic ceiling | Decoder ceiling | Cheapest decisive test | Stop rule |
|---|---|---|---|--:|--:|--:|---|---|---|---|
| N1 | Composed Ψ^k table, rank-indexed (§3.1) | A / I | open (structural) | 1 | 1 | 1 | inversion 23.6 → 10–18.5 instr/B (model) | ≤ 2.5× (4-core) / ≤ 2.1× (i9), scanner-capped | T1: census of exact \|B_k\| and d_k, k = 1…16, 233 blocks | §3.1(e): net > 0.67× on the median block for all k ≤ 4 |
| N3 | Run-native pipeline: MTF → runs → P^k, no L, no T-vector | A + C / I | open, depends on N1 | 3 | 2 | 2 | one-core 12.93 → ~5–7 c/B (model) | same cap | T6: paper cost model after T1 | N1 stops, or modelled construction > 2.16 c/B + walk saving |
| N0 | wikidata-pq decoder-free arm (the split fed decoded lines) | application | not built | 2 | — (measurement) | — | n/a | bounds every bz2 item | a bench arm in wikidata-pq (measurement-only PR) | none: it is the bound |
| N2 | Reverse-MTF chunk composition (prefix lemma, memoisation) | C / I if memoised, II otherwise | open (structural) | 4 | 5 | 6 | ≤ list-update share of 2.29 c/B | ≤ 1.2× | T2: token census + stage attribution | §3.2 |
| D1 | 8-thread CPU attribution in the split | D | open | 5 | 4 (thread budget) | 3 | split −4.2% wall now | application | Q2 per-role CPU | — |
| E8 | M2 under four copies (pre-registered in #42) | A / I | conditional, never run | 6 | 7 | 5 | per #42 | per #42 | Q3 (code exists on `scratch/move-structure`) | ≥ 10% per copy at 4 copies, or closed for good |
| B1 | Compact Ψ (24-bit link-only, #112) on a low-L3 machine | B | conditional: reopens on less L3 per core | 7 | 3 (if laptops sit past the knee at 4 workers) | 7 | i9: 1.25× inversion | i9: 1.1–1.15× | Q5: knee under restricted L3 (`/sys/fs/resctrl` CAT, if the i9 has it) or a laptop | i9-like L3 per core: closed |
| C2 | Batched team (MTF of block b+1 under b's inversion) | D / II, no added work | designed, never built | 8 | n/a (no teams at 4 cores) | 4 | helper idle ≤ 0.11 | ≤ ~1.12× at teams | occupancy trace (Q2) sizes it first | idle < 0.05 measured |
| #10 | Parallel scanner (speculative markers) | C / II | open, inferred | 9 | 8 | 8 now, 1 after N1 | lifts the scanner cap | prerequisite above ~2× | T5: marker statistics in the bitstream (read-only) | — |
| H1/E1 | Per-row Ψ² (KKP / libsais) | A / I | rejected; conditional past the knee | 10 | 9 | 9 | walk −4–7% with a free builder (§2.2) | small | none new; T4 (KKP against libsais construction diff) | as recorded |
| SI | Streaming inversion / generic list ranking / fragment assembly | A / II | open (scientific) | 11 | 10 | 10 | none shown: bzip2gpu and FGM do not reduce work | — | paper bound on passes × bytes moved | > 3 n-sized passes |

**Why N1 leads both columns.**
- It is the only class-I candidate whose mechanism is shown to exist (§3.1(a)–(c)).
- It cuts per-byte instructions and misses, so CPU per GB falls. That serves the
  4-core target, where every worker runs alone.
- It shrinks the table, which moves the i9's knee as a side effect (B1's effect, for
  free).
- Its decisive test is read-only and cheap.

**Why it might fail:** the build. At the upper bound of |B_k|, construction costs twice
today's build (§3.1(e)). The census decides that before any code is written.

## 5. The parallel CI and theory plan

None of these needs the i9. Items marked *read-only* use existing data or code, and need
only the maintainer's go-ahead to run an analysis script. Items marked *PR* change code
and need approval.

| ID | Item | Kind | Cost | Output |
|---|---|---|---|---|
| T1 | Ψ^k census: exact \|B_k\| by the preimage iteration of §3.1(f), d_k from k-gram counts, the ratio \|B_k\| / (k·\|B\|), record bytes by k, best k per block, and the heuristic S of external report 4 §9 for hybrids | read-only analysis, 233 Wikidata blocks (the existing census inputs) | CPU minutes on CI or the VM | a journal entry and the N1 go or stop |
| T2 | MTF token census + stage attribution (§3.2) | read-only analysis; a local profile is indicative | CPU minutes | a journal entry and the N2 go or stop |
| T3 | Read the #116 occupancy artifacts on CI (teams at 4 threads, k = 2): helpers' spin, park and busy shares | read-only | minutes | sizes C2 and part of D1 at k = 2 |
| T4 | Precedent checks: k-step composition in Move-r, Movi and SEA 2026; KKP 2012's implementation against libsais's construction; M2's `movest.rs` select arm against a rank-indexed k = 1 | papers and code | hours | N1's novelty conditions; closes §2.2's residue |
| T5 | #10 marker statistics: block-header magic false positives per MB in real dumps | read-only | minutes | sizes #10 |
| T6 | Run-native pipeline cost model, with T1's numbers | paper | hours | N3 go or stop |
| P1 | wikidata-pq N0 arm: the split fed pre-decoded lines (a measurement-only bench arm, full tier) | PR | one PR | the application bound |

**Order:**
1. T1, T2, T3 and T5 in parallel: one at a time on the shared machine, each light.
2. Then T4 and T6.
3. P1 whenever approved; it is independent.

## 6. The i9 queue (serialised; none launched)

**The frozen baseline for every campaign:**
- **commit:** mibz2 `v0.2.1` (`c217676`) and wikidata-pq master at the campaign's start,
  recorded;
- **binary:** built by the repository's `just` recipe, release profile, with its hash
  recorded;
- **corpus:** `bench/wikidata.json`'s windows (SHA-256 `82f65ace0731962d…`) for the
  decoder, and `--release 20261019 --windows 24 --window-mb 25` for the application;
- **harness:** bench 3.2.0 / `bench/variants.py`, with its version recorded;
- **topology:** i9-10900K, 10C/20T, 20 MiB L3, quiet (no browser: the 16 GiB/s DRAM
  contamination of #39). The `machine.json` is captured;
- **concurrency:** stated per arm;
- **recorded per run:** CPU-s, peak RSS and parity.

**The rule:** an arm runs against its own control in the same session, paired and with
the order rotated. Cache-sensitive ideas are judged on the i9 or the target machine, not
rejected on CI's small differences.

| Order | Campaign | Needs first | Arms | Pre-registration |
|---|---|---|---|---|
| Q1 | N0: the decoder-free bound in the release-shaped run | P1 merged | auto (4) against decoder-free | report the bound; no pass or fail |
| Q2 | D1: per-role CPU at 4 against 8 decoder threads, plus the occupancy trace under teams | none. The occupancy tool's fix (#116) is on master after the v0.2.1 tag, but `src/` has not changed since the tag, so the decoder measured is still 0.2.1's | auto, 8 | attribute ≥ 80% of the +40.6 CPU-s to roles |
| Q3 | E8: M2 under four copies | none (`scratch/move-structure`) | as #42 registered | ≥ 10% per copy, or close |
| Q4 | N1 kernel: P^k against the T-vector, at 1–10 workers | T1 passes, T4 done, implementation approved | k ∈ {2, 4}; control the T-vector | written after T1, from its numbers |
| Q5 | B1 on restricted L3 | resctrl available, or a laptop | as #112, with the L3 ways restricted | the knee moves at 4 workers |
| Q6 | C2 batched team | Q2 shows idle ≥ 0.05; implementation approved | teams with and without overlap | ≥ 1.05× at 8–10 threads, CPU not higher |

## 7. Contradictions

The merged list (about 60 points from the four audits, deduplicated) is in the register,
§4. Three were resolved this round:
1. **#93 against #114:** acceptance-criteria drift, then a trade-off accepted knowingly
   (§2.1). It was not a hidden regression.
2. **"#29/#34 didn't test the published construction":** #29 didn't; #34 did, in
   libsais's implementation (§2.2).
3. **"wikidata-pq has never run 0.2.x":** stale. It pins 0.2.1 (`d09edd91`). The
   4-against-8 A/B ran on it, and the cap stays 4 (wpq #215–#217).

One was added: the absolute CI walk cost differs by 31% between two runs of nominally
identical runners (§2.1). Every cross-run comparison of absolute CI figures in the
register is therefore suspect, and is marked so.

## 8. What is asked of the maintainer

1. **Approve or decline** the read-only analyses T1, T2, T3 and T5 (scripts over existing
   blocks and CI artifacts; no decoder change).
2. **Approve or decline** P1, the wikidata-pq decoder-free bench arm (measurement only).
3. **Close #93** as superseded by #114, or keep it as the record.
4. **The i9 queue** stays unlaunched until reviewed. Q2 and Q3 need no new code.
