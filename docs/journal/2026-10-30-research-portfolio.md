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
- `2026-10-30-external-report-4.md` to `-23.md`: the inputs this round, verbatim.
  - Report 5 holds the researcher's and the guardian's corrections to the coordinator's
    first reading.
  - Report 6 is the researcher's study session on the algebra of inverse MTF, and their
    reading of the bzip2gpu source.
  - Report 7 is their reading session II, on recency and order statistics.
  - Report 8 is their proposal of batched small-rank MTF tables (N2b, §3.2b).
  - Report 9 returns to the BWT itself. It includes their multi-byte record theorem,
    which converges with §3.1(c); see §3.1(g).
  - Reports 10–11 hold the researcher's review of this PR and burn-down rounds 1–5:
    - the counterexample that corrected §3.1(a);
    - the traversal-gap identity and the stable-Ψ proof;
    - the run-native propagation of the boundaries;
    - the interval-join construction.
  - Reports 12–14 hold burn-down rounds 6–9 and the researcher's summary of the day:
    - two-view construction, with the coordinator's correction to round 7's destination
      offsets;
    - exactness of the join;
    - segment caps, with the invariant checked;
    - executed lookups.

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

### 3.1 Composing Ψ: the record partition, its exact size, and how to build it

**Erratum (corrected after the researcher's review, external reports 10–11).** The
first version of (a) claimed that Ψ^k's own breakpoints equal the union of the
preimages. That is **false**. The researcher's counterexample is the cyclic BWT of
`aabc`:
- F = [a, a, b, c] and Ψ = [1, 2, 3, 0];
- Ψ²'s translation breaks plus the bucket boundaries are {1, 2}, but the union is
  {0, 1, 2};
- Ψ² is a translation on rows 0–1, yet their two-byte words are `aa` and `ab`.

The flawed step was the ⊇ direction: a separation at step t need not survive as a break
of Ψ^k. Brute force confirms the failure is common: 6,829 of 22,940 random primitive
cases.

The quantity the records need was never Ψ^k's breakpoints. It is the partition on which
both the successor and the word are constant, and for that the union is exact. The
corrected statement follows.

**Notation.**
- Ψ maps row j to the row of the next suffix.
- The walk at row j emits F[j], the first byte of row j's rotation.
- Let B be the set of j such that Ψ(j+1) ≠ Ψ(j) + 1, **together with every F-bucket
  boundary**. "j ∈ B" means "a break between rows j and j+1".
- **Round 4's identity:** for the stable mapping, |B| = r − 1. The one-step intervals are
  L's runs in F-bucket order. This was measured on all three checked blocks, for
  example B = 51,183 with r = 51,184.
- Let C_k = ⋃_{t=0}^{k−1} Ψ^{−t}(B), the union of preimages.
- Let M_k be the **record partition**: the breaks j (j < n − 1) where the k-byte word
  changes, or where Ψ^k(j+1) ≠ Ψ^k(j) + 1.
- The records' intervals are M_k's intervals.

**(a) The record partition equals the union, for a stable Ψ.** For every j < n − 1,
j ∈ M_k exactly when j ∈ C_k. The only exception is a meaningless label on the last row,
which has no row after it.

*Proof.*
1. **M ⊆ C (contrapositive).** Suppose j ∉ C_k. By induction on t < k, the trajectories
   of j and j+1 stay adjacent and share a bucket:
   - adjacent at t, and Ψ^t(j) ∉ B, gives the same byte at t and adjacency at t+1;
   - so the words agree and Ψ^k(j+1) = Ψ^k(j) + 1, and j ∉ M_k.
2. **C ⊆ M.** Suppose Ψ^t(j) ∈ B for some t < k, and the words of j and j+1 agree.
   - Ψ is order-preserving within a bucket, so Ψ(y) − Ψ(x) ≥ y − x for x < y in one
     bucket. The researcher states this in round 3.
   - While the words agree, the two trajectories share a bucket at every step, so their
     separation never shrinks.
   - The separation becomes at least 2 at the first step where they are not adjacent,
     and stays so through step k. So Ψ^k(j+1) ≠ Ψ^k(j) + 1.
   - If the words differ, j ∈ M_k directly. ∎

The coordinator's form of step 2 counts the rows between the two trajectories: they
share the trajectories' byte, so Ψ carries them along. The researcher's form counts the
separation. They are the same argument, reached independently.

**Checks:**
- **Brute force:** C_k = M_k in all 22,940 random primitive cases here, and in the
  researcher's exhaustive 58,212 (every primitive string over three symbols up to
  length 8).
- **Real blocks:** explicitly, for k = 1…6, on three full bench blocks.
- **Stability:** the decoder's Ψ is bucket-monotone by construction, since the T-vector
  is a stable counting scatter in L order. The researcher's periodic counterexample
  (`aaaa` with an unstable tie order) cannot arise from the decoder's Ψ. For a periodic
  block, Ψ has several equal cycles, and the gap sum below runs over each cycle that
  holds breakpoints.

**(b) The exact size, from traversal gaps** (the researcher's identity, round 2).
- **The identity:** follow Ψ around its cycle, and let g_1, …, g_m be the distances
  between successive rows of B. Then |C_k| = Σ_i min(k, g_i), so the records number
  1 + Σ_i min(k, g_i).
- **Why:** a row lies in C_k exactly when the next breakpoint along the traversal is
  fewer than k steps ahead.
- **Checked** explicitly on three real blocks.
- **Slope:** |C_{k+1}| − |C_k| = #{i : g_i > k}.
- **Bounds:** d_k − 1 ≤ |C_k| ≤ k·|B|. The rows are sorted, so equal k-words are
  contiguous, and the word changes exactly d_k − 1 times.
- **Decomposition:** |C_k| = (d_k − 1) + #{j : equal words but non-adjacent successors}.
  The second term is the navigation excess behind the researcher's fragmentation factor
  R_k / d_k.

**(c) The bytes are free to locate, on M_k's intervals.** On each interval I of M_k,
Ψ^k is one translation j ↦ j + D_I and the k-byte word is constant, by definition and
by (a). A record (D_I, k bytes) recovers all k bytes with no per-row information:
k·(|C_k| + 1) bytes of words in all.

This holds for the record partition, not for Ψ^k's translation intervals. The original
error was to conflate the two.

**(c′) Building the records without a T-vector or a search per record.** This answers
round 4's open problem: the destinations and the words.

1. **Level 1 comes from L's runs.** Each run (c, start p, length ℓ) is one record:
   - its source rows are [C[c] + q_c, + ℓ) in F order;
   - its displacement is D = p − (C[c] + q_c);
   - its word is (c).

   This is one pass over the run list with 256 counters. Checked: the level-1 count
   equals L's runs in every brute-force case.
2. **Level a + b from levels a and b, by image splitting.**
   - Ψ^a translates each level-a interval I = [s, e) onto the contiguous range
     [s + D_I, e + D_I).
   - Split I wherever that image crosses a level-b boundary.
   - Each piece's record is (D_I + D_J, word_I ‖ word_J), where J is the level-b
     interval it lands in.
   - Checked: built for k = 2, 3, 4, 8, this reproduces the directly computed minimal
     records and words exactly, with 0 mismatches in 21,970 random cases.
3. **Destination resolution needs no search.**
   - The images of a level's intervals tile the rows: Ψ^a is a permutation that
     translates each interval. Checked: no tiling failures in any case.
   - So the level-a intervals, sorted by image start, merge with the level-b partition in
     one sequential pass, which finds every J.
   - At level 1, that image order is L's run order, which is the run stream itself. No
     sort at all.
   - At higher levels, the image order propagates. The pieces landing in one level-b
     interval J appear together in the merge, and their images under Ψ^{a+b} follow J's
     place in level b's image order. So the next level's image order is a permutation of
     contiguous groups: a gather, not a comparison sort.
   - The cost per level: one merge over |C_a| + |C_b| boundaries, plus a group gather of
     about |C_a| groups. Each group is a contiguous copy of a record or two, reading
     arrays of about 0.5–1.4 MB with independent accesses. There is no chain of dependent
     loads.
4. **Round 4's boundary propagation** (LF from runs, 256 sorted per-character streams,
   and a merge) is the same machinery seen from the boundary side. Both reach C_k with
   no T-vector.

**The model** (operation counts, not timings):
- **Building P⁸** by 1 → 2 → 4 → 8: three merges and three group gathers, over at most
  |C_1| + |C_2| + |C_4| ≈ 51–55k + 62–68k + 76–85k ≈ 190–210k groups a block. That is
  about 0.2–0.25 group moves an output byte, all independent.
- **The T-vector build:** n = 900,000 scatter writes into 3.6 MB.
- Whether the merge-and-gather costs less than 2.16 c/B is the measurement that would
  decide it. It can't be shown from counts alone.

**(h) Gate A: the census on all 233 full bench blocks** (measured; read-only Python over
the bench's own windows, rebuilt from the `bench-data` branch; 1,073 s, one core).

| Quantity (min / p10 / median / p90 / max) | Value |
|---|---|
| n | 899,809 / 899,971 / 900,000 / 900,000 / 900,000 |
| r (= m₁) | 47,936 / 51,108 / 56,032 / 99,380 / 179,972 |
| \|B\| − (r − 1) | 0 on every block (round 4's identity) |
| m₂ | 57,676 / 62,125 / 68,029 / 121,772 / 203,914 |
| m₄ | 70,271 / 76,737 / 84,287 / 149,854 / 227,058 |
| m₈ | 89,048 / 100,416 / 109,726 / 195,995 / 268,255 |
| m₈ / (8\|B\|), the overlap the worst case ignores | 0.18 / 0.24 / 0.25 / 0.26 / 0.30 |
| m₈ / d₈ (round 1's fragmentation factor) | 1.27 / 1.60 / 1.67 / 1.76 / 2.16 |
| P⁸ resident, 12 B records + 225 KB directory, MB | 1.29 / 1.43 / 1.54 / 2.58 / 3.44 |
| P⁴ resident, MB | 0.79 / 0.84 / 0.90 / 1.42 / 2.04 |
| P¹⁶ resident (20 B records), MB | 2.62 / 2.97 / 3.28 / 5.50 / 7.49 |

**The gap distribution along the traversal,** pooled over all blocks (mean gap 13.8):
- 84.8% of gaps are ≤ 2 steps, 90.0% ≤ 8, 95.8% ≤ 64;
- 93.4% of rows lie in gaps longer than 2, 91.6% in gaps longer than 8, 79.9% in gaps
  longer than 64.

So breakpoints come in tight bursts, and most rows lie in long, breakpoint-free
stretches. That is why m₈ is about 2|B| rather than 8|B|.

**Storage, the researcher's round-3 threshold:** every block fits under the T-vector's
3.6 MB at k = 8, including the worst block at 3.44 MB. k = 16 does not fit on the
high-r blocks.

**Negative controls** (one 899,000-byte block each, built from the bench's own corpus
definitions in `tests/common/mod.rs`):

| Corpus | r | n/r | Pure 32-row groups | m₁ / m₂ / m₄ / m₈ | P⁸ MB |
|---|--:|--:|--:|---|--:|
| records | 896,176 | 1.0 | 0.000 | 896,176 / 898,996 / 899,001 / 899,001 | 11.0 |
| random | 895,424 | 1.0 | 0.000 | 895,424 / 898,985 / 899,001 / 899,001 | 11.0 |
| text (the crate's sources) | 135,801 | 6.6 | 0.292 | 135,801 / 200,487 / 277,841 / 352,169 | 4.45 |

All three fail the storage gate. A P⁸ decoder needs a per-block fallback to the
T-vector, chosen before building:
- **The rule:** r is known after MTF, and m₈ ≤ 8r, so blocks with
  8r·12 + 225 KB ≤ 3.6 MB pass outright.
- **Otherwise:** a cheap bound or a partial build decides.

This is the guard against #25-style regressions on random and records data.

**Children per parent on doubling** (40 sampled blocks; min / p10 / median / p90 / max):

| Level | Unsplit parents | Round 8's lower bound 2m_a − m_2a | 2 children | ≥ 3 children | Most children |
|---|---|---|---|---|--:|
| 1 → 2 | 0.899 / 0.904 / 0.906 / 0.913 / 0.936 | 0.753 / 0.767 / 0.779 / 0.798 / 0.867 | 0.055–0.073 | 0.008–0.030 | 3,161 |
| 2 → 4 | 0.929 / 0.936 / 0.939 / 0.950 / 0.983 | 0.729 / 0.747 / 0.763 / 0.788 / 0.887 | 0.012–0.044 | 0.005–0.028 | 3,439 |
| 4 → 8 | 0.934 / 0.936 / 0.942 / 0.950 / 0.987 | 0.668 / 0.671 / 0.689 / 0.713 / 0.838 | 0.007–0.035 | 0.005–0.032 | 2,093 |

About 94% of Ψ⁴ parents pass to Ψ⁸ whole, far above round 8's 69% bound. The splitting
concentrates in about 3% of parents, some of which split into thousands of children:
the long intervals whose images cross a burst. So a fast unsplit path is justified, and
the split loop's cost is dominated by a few large parents.

**The construction-traffic model** (Gate B's first input; a model, not a measurement):
- **Assumptions:**
  - the corrected two-pass counting construction, with both views at levels 1–4 and
    the source view only at level 8;
  - 12 B construction records (16 B at level 8);
  - u32 counts, plus the 225 KB directory.
- **Results, per block:**

| | min / p10 / median / p90 / max |
|---|---|
| Bytes moved per output byte, P⁸ build | 19.5 / 21.0 / 23.1 / 40.6 / 66.3 |
| Bytes moved per output byte, T-vector build (L read, 4n scattered) | 5.0 |
| Peak live MB during the P⁸ build | 3.9 / 4.3 / 4.7 / 8.2 / 12.2 |
| Resident MB during the walk | 1.3 / 1.4 / 1.5 / 2.6 / 3.4 (T-vector 3.6, plus L's 0.9 if kept) |

- **The plain reading:** the P⁸ builder moves about 4.6× the bytes of the T-vector
  build on the median block, and about 8× on the 90th percentile.
  - Its accesses are mostly sequential, over arrays of at most about 2.6 MB. The
    T-vector's 4n bytes are scattered into a 3.6 MB array.
  - Which costs fewer cycles is exactly Gate B. Bytes alone don't decide it, but they
    make Gate B the most likely place for N1 to fail.
- **Peak build memory** matches the T-vector's (4.7 against 4.5 MB) on the median
  block, and is about twice that on high-r blocks. That is a laptop concern at four
  blocks in flight.
- **Cheaper variants,** to model before any code:
  - fold each level's counting pass into the previous level's scatter, which removes a
    read of both views;
  - the researcher's indexed views, from round 6;
  - stop at k = 4 on high-r blocks.

**(i) Builder layouts, peak memory, and the per-block choice under both constraints**
(rounds 17–19; a model over the measured m_a, with no measured cycles).

| Layout | Bytes moved per output byte | of which random gathers | Peak live MB | Notes |
|---|---|---|---|---|
| A: two full views, two merge passes | 19.5 / 21.0 / **23.1** / 40.6 / 66.3 | 0 | 3.9 / 4.3 / **4.7** / 8.2 / 12.2 | the original model |
| B: one descriptor array + u32 image permutation, two passes | 19.5 / 21.0 / **23.1** / 40.6 / 66.3 | 4.7 / 5.1 / **5.5** / 9.9 / 16.3 | 3.4 / 3.7 / **4.0** / 7.0 / 9.9 | the same bytes as A, because the gathers replace the second view's reads; it lowers memory, not traffic |
| C: B, with children counted from two n-bit start bitvectors, one merge pass | 14.7 / 15.7 / **17.0** / 28.1 / 44.1 | 2.3 / 2.5 / **2.8** / 4.9 / 8.1 | 3.6 / 3.9 / **4.2** / 7.2 / 10.1 | e_i and f_j are popcounts over each image range, because the images tile the rows. 26% less traffic than A |

Values are min / p10 / **median** / p90 / max over the 233 blocks. The T-vector build
moves 5.0 bytes per output byte and peaks at 4.5 MB, with L included.

- **Layout B answers round 19's question.** One descriptor store plus a permutation
  brings the median peak under the T-vector's (4.0 against 4.5 MB), and cuts the worst
  from 12.2 to 9.9 MB, at the cost of indexed gathers.
- **No layout gets the upper tail under the T-vector's peak.** Those blocks are the
  high-r ones, p90 r ≈ 99k.
- **Round 19's split-excess identity,** Σ(e_i − 1) = m_2a − m_a, measured at the last
  doubling: m₈ − m₄ = 17,957 / 23,551 / 26,607 / 43,212 / 67,627.

**The per-block choice under resident and peak constraints together** (layout B; resident
≤ 3.6 MB; peak ≤ the budget):

| Peak budget | P⁸ | P⁴ | P² | T-vector |
|---|--:|--:|--:|--:|
| 4.5 MB (the T-vector's own peak) | 161 | 40 | 16 | 16 |
| 5.0 MB | 189 | 22 | 15 | 7 |
| 6.0 MB | 199 | 25 | 8 | 1 |

So "all 233 choose P⁸" holds only under the resident-size rule. Round 19 is right that
the complete objective selects differently. At the T-vector's own peak, 69% of blocks
take P⁸. The third constraint, build plus walk below the baseline, is Gate B/C and is
unmeasured. It may move the choice further toward P⁴.

**(j) Per-level mixed layouts, and folding the counting pass** (round 20; a model over
the measured m_a).

**Folding the counting pass is possible without a second join.**
- Parent i's child count is one plus the number of level-a source starts strictly inside
  its image J_i: e_i = 1 + #{starts ∈ (start J_i, end J_i)}.
- The images tile the rows, so one sequential popcount sweep over an n-bit bitvector of
  level-a starts, taken in image order, yields every e_i.
- Likewise f_j comes from a bitvector of image starts, swept in source order.
- Both bitvectors are marked during the previous level's scatter (112.5 KB each).

That is layout C of (i).

**All eight layout mixes,** each with both counting methods. "dup" keeps a full
image-order view at that level; "idx" keeps a u32 permutation.

| Image view at levels 1/2/4 | Counting | B/B, median | Gathers, B/B | Peak MB (median / p90 / max) |
|---|---|--:|--:|---|
| idx/idx/idx | bitvector | 17.0 | 2.8 | 4.23 / 7.19 / 10.12 |
| dup/idx/idx | bitvector | 17.2 | 2.0 | 4.23 / 7.19 / 10.12 |
| **dup/dup/idx** | **bitvector** | **17.5** | **1.1** | **4.23 / 7.19 / 11.10** |
| dup/dup/dup | bitvector | 17.9 | 0.0 | 4.91 / 8.38 / 12.42 |
| any mix | two-pass | 23.1 | 0.0–5.5 | 4.01–4.68 / 6.96–8.16 / 9.90–12.20 |

**The reading:**
- **The counting method sets the traffic.** Bitvector counting moves 17.0–17.9 B/B
  against 23.1 for the two-pass, a 23–26% cut.
- **Under two passes,** every mix moves exactly 23.1 B/B. This confirms round 20: an
  index's saved writes are spent again on its gathers.
- **The layout only trades gathers against peak.** Indexing the level-4 image view alone
  takes about 0.7 MB off the median peak (4.91 → 4.23) for 1.1 B/B of gathers.
- **The upper tail stays above the T-vector's 4.5 MB peak in every mix.** The graded
  selector, (i), is what handles those blocks.
- **None of this is cycles.** The comparison to make is still 17.5 bytes of mostly
  sequential traffic plus 1.1 B/B of gathers, against the T-vector's 5 B/B of scattered
  writes, and that is Gate B.

**(k) The counting gate (round 22): passed.**
- **What was checked:** children counts taken from strict-interior popcounts:
  - e_i = 1 + #{level-a source starts u : start J_i < u < end J_i};
  - f_j = 1 + #{level-a image starts u : start I_j < u < end I_j}.
- **The result:** on 8,160 random doubling levels (levels 2, 4 and 8 over random
  run-structured permutations), with 0 failures in each of three assertions:
  - the counts equal the merge-counted ones;
  - Σe_i = Σf_j = m_2a;
  - the source and image views built from them match the two-pass builder exactly,
    endpoints, displacements and words included.
- **Caps:** these counts are over the uncapped partition, as round 22 notes. The
  segment caps are applied as a post-split of the final level only: at most
  2(k − 1) records per segment start, the scheme already checked in (g)2. So no cap
  boundary needs to enter the doubling counts.

**The no-second-join construction is settled mathematically.** What remains is cycles,
which is Gate B.

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

**(g) External report 9 converges on (c), and adds three questions.**

The researcher's "output-word theorem" is (c), derived independently. Their bound,
R_k ≤ 1 + k(r + σ − 2), is (b)'s upper bound with the bucket boundaries counted in B:
|B| ≤ (r − 1) + (σ − 1). This portfolio adds two things to the bound:
- the exact identity (a), so the census measures |B_k| itself rather than bounding it;
- the lower bound d_k − 1, which needs no BWT. Their table shows k = 16 reaching about
  one row a record at the upper bound. The lower bound tells us cheaply whether the
  real block is near that.

Their three additions, answered where the theory allows:

1. **Variable-length records need a budget, or the oracle is trivial.**
   - Every row lies in some interval of Ψ^k, even a singleton one. With no limit on
     records, a k-step record exists everywhere, and an oracle removes exactly 1 − 1/k
     of the dependent steps. The real question is lookups per byte at a given number of
     record bytes.
   - The structure helps: B_{k−1} ⊆ B_k, by (a). So the intervals form a refinement
     tree: level k's intervals subdivide level k−1's.
   - A variable-length representation is therefore a cut of that tree. Each region keeps
     the deepest level whose subdivision stays within a record budget.
   - **The budgeted oracle:** for candidate cuts (for example, "keep level k under a
     parent if it splits into at most s children"), count the record bytes and simulate
     the walk along the actual cycle. The lookups counted are exact, because each jump's
     landing row determines the next record. This is read-only, about n steps per
     candidate per block, and now part of T1.
2. **The cooperative walk's segment ends are handled by construction.**
   - The walk's start rows are fixed before it runs, so they are known before the
     records are built.
   - A jump must not pass over a segment's end row e. So each row j with Ψ^t(j) = e for
     some 0 < t < k gets a record of length t at most.
   - That costs at most (k − 1) extra breakpoints per segment end. With 512 segments,
     that is ≤ 512(k − 1), against k·r ≈ 56,000k. It is negligible, and it needs no
     check in the hot loop.
   - **Why caps, rather than masked tail stores (rounds 8–9).** The shipped room walk
     doesn't know a segment's length: a lane finds the end by reaching a flagged row (the
     marker audit, #100: "lengths are unknowable without walking"). A k-step jump would
     skip that row. A capped record stores its own length, t ≤ k. On any row whose
     trajectory reaches a segment start within 1…k−1 steps, it emits exactly t bytes and
     lands on the flagged row. The researcher accepted this in round 9's follow-up.
   - **The segment-terminal invariant, as the researcher stated it.** From any segment
     start, the capped records alone must emit exactly that segment's bytes, reach its
     terminal marker exactly once, and never go past it.
     - **Checked by brute force:** 10,188 segments over random primitive strings with
       random flagged starts, 0 failures.
     - **Extra records:** at most 5 per start were observed, against a bound of
       2(k − 1) = 14.
     - **The fallback:** none. Ψ¹ and the T-vector are not retained.
   - **Executed lookups are exact.** A lane reaches a capped row only on its final hop,
     because the capped rows are those fewer than k steps before the next start. So a
     segment of length ℓ_s takes exactly ⌈ℓ_s/k⌉ lookups, and
     H = Σ_s ⌈ℓ_s/k⌉ < n/k + 512.
     - This is the researcher's round-9 count, now holding under caps.
     - At k = 8 and n = 900,000, n/H > 7.96 bytes a lookup.
     - Checked: every one of the 10,188 walked segments took exactly ⌈ℓ/8⌉ lookups.
3. **The gap between row-space and traversal-space regularity can be measured.**
   - This is the researcher's obstruction, deduction 3. Identity (a) turns traversal-space
     regularity into a row-space count: |B_k| is the number of adjacent row pairs whose
     k-step trajectories separate.
   - The lower bound d_k − 1 then caps how much traversal regularity the text can have,
     by its k-gram diversity alone.
   - So the obstruction is not hidden. T1 measures it.

**Novelty** is still owed on both sides. The researcher states they have not established
it, and T4 is unchanged.

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

### 3.2b N2b: batched small-rank MTF tables (external report 8), checked against `src/mtf.rs`

**The proposal.** Rank-limited batches of MTF instructions act only on the list's first
few positions. One table entry gives both:
- the emitted bytes' selectors;
- the prefix's new order.

They would be applied with `pshufb`. This is a concrete instance of route (ii): it
produces the output, not only the state. The researcher checked the algebra
exhaustively for four-instruction batches with prefix sizes two to four.

**What the code shows** (`mtf_avx2`, master `ae942bd`):

1. **Explicit tokens are never rank 0.**
   - The scanner's tokens are RUNA (0), RUNB (1), and s ≥ 2 for rank s − 1.
   - So the explicit ranks are 1…255. Rank 0 exists only as RUNA/RUNB runs, which
     `write_bytes` expands (a memset of the front byte, plus one count update).
   - A run does not change the list. So a batch is defined over consecutive explicit
     tokens, whatever the runs between them.
   - The emitted bytes interleave: after explicit symbol x_i, the next run repeats x_i,
     because x_i is the new front.
   - **Consequences:**
     - the {0, 1} parity case reduces, on explicit tokens, to rank-1-only stretches
       (a, b, a, b, …, with runs between);
     - the 0–3 table indexes ranks {1, 2, 3}: 3⁴ = 81 entries for a batch of four, and
       3⁸ = 6,561 for eight;
     - the runs between explicit tokens still need their memsets and counts, as now.
2. **The update is already constant-time for ranks below 32.**
   - The step is one aligned 32-byte load, one unaligned load at `base − 1`, a compare,
     a blend and a store.
   - So the table does not save shifting work. What it can save is the loop-carried
     chain.
   - The code's own comment, citing coop journal §15, records that the next symbol's
     32-byte list load overlaps the previous store, cannot be forwarded from it, and
     waits for it to commit. That is a store-forwarding stall on every explicit symbol.
   - **The register already holds this lead:** P74, "MTF store-forwarding chain; front
     32 B in a register", open and "recorded and not started", with `mtf_cp2p`/`mtf_cp2`
     at 1.0582 ±0.0093 on the i9 from moving a store off the chain.
   - **N2b is one specific way of taking P74:** keep the front in a register, and take
     several explicit tokens per dependent step.

**The novelty conditions:**
- **Against #25 (P73, rejected):** #25 held the MTF head in registers and regressed on
  high-rank data (random +11.1% / +8.2%, records +9.7% at 4 threads). N2b must keep
  today's path for any batch containing a rank above the limit. It must show its
  recognition test costs less than the chain it removes, on random and records as well
  as Wikidata.
- **Against #8 / P78 (front-end fusion):** P78 aims at memory and the in-place T-vector.
  N2b changes neither, so they do not overlap.
- **Against the shipped two-symbol Huffman:** that is the scanner's stage, and N2b the
  worker's. They are complementary.

**The ceiling** (model):
- Explicit tokens are r − 1 a block, 0.072 a byte. The stage costs ≈32 cycles per run
  (2.29 c/B × 13.8).
- If the store-forwarding chain were half of that, removing it entirely would save
  ≈1.1 c/B. That is ≤ 9% of a one-core decode (12.93 c/B), and ≤ 1.09× on one core.
- That is useful, but not a breakthrough on its own. It is a per-worker CPU reduction,
  so it serves the 4-core target.

**The census** (read-only, added to T2), over each block's explicit tokens:
- the share of explicit tokens with rank ≤ 1, ≤ 3 and ≤ 7;
- the share coverable by non-overlapping batches of 4 and 8 consecutive explicit tokens
  under each limit;
- for comparison, the same statistics on the random and records corpora (#25's
  regressions).

**The stop rule:** if fewer than half the explicit tokens on Wikidata fall in eligible
batches of four under the ≤ 3 limit, the ceiling falls below ≈4%. Then fold N2b back
into P74, as a design option rather than a candidate.

### 3.2c T2, the MTF token census: N2 and N2b stop; P74 remains (measured)

**The method:** for 20 sampled blocks, the explicit MTF ranks bzip2's encoder produces,
over the used-symbol list, with rank 0 carried by RUNA/RUNB. They come from each block's
own BWT, read-only.

- **Explicit tokens:** median 54,795 a block, 0.061 an output byte. This matches r − 1:
  an L-run's first byte is the only explicit token.
  - At 2.29 c/B for the stage, that is about 38 cycles per explicit token, run expansion
    and counting included.
- **Rank distribution,** as a share of explicit tokens:

  | Rank limit | ≤ 1 | ≤ 3 | ≤ 7 | ≤ 15 | ≤ 31 |
  |---|--:|--:|--:|--:|--:|
  | Share | 14.5% | 29.8% | 52.4% | 84.9% | 98.4% |

  This agrees with the register's 15.2% at rank ≥ 16 and 1.52% at rank ≥ 32.
- **N2b, small-rank batch tables:** the share of explicit tokens in aligned batches that
  are wholly within the limit.

  | Limit | Batches of 4 | Batches of 8 |
  |---|--:|--:|
  | ≤ 3 | 7.1% (6.0–9.0) | 5.0% |
  | ≤ 7 | 15.6% | 9.0% |
  | ≤ 15 | 58.5% | 41.3% |
  | ≤ 31 | 94.7% | 91.4% |

  - **§3.2b's pre-registered stop rule,** "< 50% of explicit tokens in ≤ 3 batches of
    four", **is met: 7.1%. N2b stops.**
  - Only the ≤ 31 limit covers most tokens. A table over four ranks of up to 31 needs
    32⁴ ≈ 1M entries, which is no longer a table. It becomes a 32-byte list held in a
    register and updated per token: P74 itself.
- **N2, memoised chunk summaries** (route (ii) of §3.2):
  - 74.8% of rank 4-grams are unique within their block (8-grams: 95.7%);
  - the 1,000 most common 4-grams cover 14.7% of positions (8-grams: 6.1%);
  - chunk prefixes are not small either: the max rank in a 16-token chunk has median 18
    and p90 37, and in a 64-token chunk median 25 and p90 62.
  - **N2 stops on Wikidata** (low ceiling, §3.2), as a measured negative. The algebra
    stands: rank sequences repeat too little for memoisation to remove work.
- **What remains is P74:** keep the 32-byte list front in a register across tokens,
  breaking the store-forwarding chain, with the existing path for the 1.6% of tokens
  above rank 31.
  - It must differ materially from #25 and answer #25's regression on random data.
  - Its ceiling is the chain's share of about 38 cycles a token, unmeasured.

### 3.2d P74: the structural test (round 21) passes

**The register-held update for rank q ≤ 31** keeps the list's first 32 bytes in one ymm
register, V, across tokens. It takes three instructions on the state's dependency chain:
1. S = `vperm2i128`(V), which swaps the 128-bit lanes;
2. A = `vpshufb`(V, idx_q) and B = `vpshufb`(S, idx_q), the same index used on both;
3. V′ = `vpblendvb`(A, B, sel_q).

idx_q and sel_q come from a 2 KB table indexed by the token's rank. They depend only on
the token stream, which the scanner has already decoded, so they stay off the chain.

**Checked:** an emulation of AVX2's byte semantics (within-lane `vpshufb`, lane swap,
blend) is exact against plain MTF on 20,000 random states, for every q in 0–31.

**Chain length** (published instruction latencies, being confirmed from primary tables by
the literature agent):
- **P74, about 5–6 cycles a token:** `vperm2i128` (≈ 3), then `vpshufb` (≈ 1), then
  `vpblendvb` (≈ 1–2);
- **the shipped `mtf_avx2`:** a 32-byte store followed by an unaligned 32-byte load that
  overlaps it. The source's own comment says it "can't be forwarded from it and waits for
  it to commit", a store-forwarding stall on every explicit token;
- **AVX-512 VBMI** (`vpermb`) would make it one instruction. Neither the i9-10900K nor
  CI's EPYC 7763 has it.

**How P74 differs from #25:**
- **Coverage:** 32 entries (98.4% of explicit tokens) against 16 (84.8%).
- **The state never goes through memory** for the fast path.
- **The cross-lane permutation is table-driven,** so no per-rank branching.

**The fallback for rank ≥ 32** (1.6% of tokens on Wikidata):
- **The head:** V′ = [tail[q−32], V[0…30]]. That is a cross-lane one-byte shift
  (`vperm2i128` + `vpalignr`) plus an insert.
- **The tail:** a memmove in memory, off the head's chain.

**The full state machine is correct (rounds 23–24).**
- **The state:** the head register H (32) plus the memory tail T (the used-symbol list's
  remainder).
- **For q ≥ 32:**
  - H′ = [T[q−32], H[0…30]];
  - T′ = [H[31]] followed by T with its element q−32 removed.
- **Checked:** H′‖T′ matches plain MTF with 0 mismatches on:
  - Wikidata-like, uniform 0–255, all-high and 30–33-boundary rank streams (60 × 4,000
    tokens each);
  - two real bench blocks' explicit-rank streams, with bzip2's own used-symbol mapping
    (89 and 86 symbols; 51,184 and 57,506 tokens; ranks ≥ 32 at 1.51% and 0.94%), with
    every emitted symbol and the full list compared after each token.

What remains for P74 is its economics only: the confirmed chain latency, the fallback's
cost, and the random-data guard.

**Random-data guard** (#25's regression): on random data, ranks are near-uniform and only
about 12.5% are ≤ 31.
- P74 then needs a per-block switch, for example on the block's running share of
  rank > 31.
- Any block whose used alphabet has ≤ 32 symbols is 100% fast path by construction.
- Wikidata blocks use 79–102 symbols (the census's d₁), so they rely on rank locality.

**The ceiling (model):** about 38 cycles an explicit token today, 0.061 tokens a byte.
- **If the ~15–20-cycle forwarding chain binds:** cutting it to about 6 saves about
  10–14 cycles a token, that is 0.6–0.85 c/B, or 5–7% of a one-core decode.
- **If the loop is otherwise throughput-bound:** less.

It is a real but modest candidate, not a breakthrough. It needs a CI-measurable microbenchmark,
with no PMU required.

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
| N2 | Reverse-MTF chunk composition (prefix lemma, memoisation) | C / I if memoised, II otherwise | **stopped (T2, §3.2c):** rank n-grams 75–96% unique, so memoisation can't remove work | 4 | 5 | 6 | ≤ list-update share of 2.29 c/B | ≤ 1.2× | T2: token census + stage attribution | §3.2 |
| N2b | Batched small-rank MTF tables (§3.2b), a route to P74 | C / I (shortens the chain; no added work) | **stopped by its own rule (T2, §3.2c):** 7.1% < 50%; folded into P74 | 4= (shares T2 with N2) | 3= | 6= | ≤ ≈1.1 c/B of MTF+RLE2 (model) | ≤ 1.09× one core | T2: explicit-token census by rank limit and batch length | < 50% of explicit tokens in ≤ 3 batches of four |
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
| T1 | Ψ^k census: exact \|B_k\| by the preimage iteration of §3.1(f), d_k from k-gram counts, the ratio \|B_k\| / (k·\|B\|), record bytes by k, best k per block, the heuristic S of external report 4 §9, and the budgeted variable-length oracle of §3.1(g) (record bytes against exact lookups along the cycle, for candidate cuts of the refinement tree) | read-only analysis, 233 Wikidata blocks (the existing census inputs) | CPU minutes on CI or the VM | a journal entry and the N1 go or stop |
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

## 9. The burn-down ledger (30 October)

This is the register's 47 open questions (`2026-10-30-research-portfolio-register.md`
§3), worked through with the researcher in reports 10–16. No question is closed by
algebra alone where its criterion is a performance figure.

**Marks:**
- **A:** answered at the question's own level;
- **N:** narrowed. The mechanism or mathematics is settled, and one named measurement
  remains;
- **O:** untouched so far.

**Sources:**
- *meas.:* the coordinator's read-only census of the bench blocks, §3.1(h);
- *R10–R12:* the researcher's rounds, external report 16.

| # | Question (short) | Mark | Evidence | What remains |
|---|---|---|---|---|
| 11 | H4: fused front end / in-place T-vector | N | R13: buffer reuse (P78) is not work elimination (N3), and an in-place scatter must prove an alias-safe schedule. R14: N3 must carry the fallback's cost, with options A/B/C | lifetime and RSS analysis; N3's mixed-corpus cost |
| 15 | E3: why 3x3 wins; where libsais's time goes | N | R13: a gap between throughputs (1,707 against 1,256 MB/s, P134) is not a component cost. The coop half was already answered | same-boundary phase profiling of libsais |
| 19 | E7: 20-bit ψ at 6–10 workers | N | R13: 20 bits suffice for every row index (900,000 < 2²⁰); the raw footprint would be 2.25 MB, against 3.6 | the registered i9 run (never made) |
| 21 | E9: walk output in L's freed buffer | N | R13: it cannot shrink the walk's hot table, so the strong form is disposed; RSS and contention benefit remain | an RSS/contention measurement |
| 45 | FOR + EF + exception hybrid, or one scheme (per-block format choice) | N | R14 with meas.: a per-block choice is necessary (the controls fail). The graded rule: build level by level, accept the next level from m_2k ≤ 2m_k − 1, and reject from exact m_k. records and random → T-vector at level 1 (zero composition); text → P⁴ (2.45 MB); Wikidata → P⁸ on 233/233, 203 accepted before level 8 is built | the cost of the fallback path itself (N3 option A/B) |
| 2 | Intel DDP/AOP prefetch interplay | N | R12: not attributable from timings; no design depends on it | an on/off comparison, only if a design comes to depend on it |
| 3 | Room walk's window headroom (AMAC) | N | R12: plausible; window and memory bounds separated | PMU attribution |
| 4 | 32 lanes = 32 misses? | A | R12: lanes bound outstanding successor requests and are not the miss count | exact occupancy (PMU), if needed |
| 25 | Gagie 2026, Reportable Shortcuts | A | R1: the citation exists (arXiv:2609.06596, revised 12 Oct 2026); its speedups are for search, not inversion. Not independently re-read here | whether any part transfers to inversion (low) |
| 37 | Redundancy above nH₀ | N | R11: boundary information is log₂ C(n−1, r−1) ≈ 38 KB, against N1's 225 KB directory, so the cost is in location, not storage | an access-time model per representation |
| 38 | Run records, ψ = base + r | N | R11 with N1: the successor is arithmetic once the interval is known | lookup cycles (N1 Gate C) |
| 39 | Alphabet-partitioned select | N | R11: Ψ(j) = select_c(L, j − C[c] + 1), with its run form | predecessor/select cost |
| 44 | Next pointer by arithmetic | N | N1: one 8-byte jump per lookup, with n/H > 7.96 bytes a lookup under caps (§3.1(g)) | Gate C |
| 46 | Direct select using F's intervals | N | R11: the exact select formula | whether direct indexing avoids select |
| 53 | Ψ groups in one streaming pass | N (mathematics settled: §3.1(k)) | N1: run-native, 1 → 2 → 4 → 8, no sort, no search, no T-vector (§3.1(c′)); brute-force exact | Gate B cycles. Modelled at 23 bytes moved per output byte, against the T-vector build's 5 (§3.1(h)) |
| 58 | Other inputs; worst-case exceptions | A (structure) | meas.: the bench's random and records blocks have 0.000 pure 32-row groups, so 100% impure is reached by valid BWTs. R11's alternating column is not a valid BWT (its Ψ has unequal cycles) | each encoding's own worst-case exception rate |
| 63 | Run length → group occupancy | A | R10: the exact occupancy formula. meas.: corr(n/r, pure fraction) = 0.78 over 40 blocks, so a strong but imperfect predictor | none for the question |
| 64 | Symbol-specific boundary patterns | A | meas.: breaks per row by F-bucket range from `"` 0.007 and `p` 0.006 to `0` 0.154. The digit buckets carry about 20× the break density of the structural symbols | none |
| 65 | Breaks concentrated near mod 32? | A (no) | meas.: 3.1% of breaks fall on a group edge (uniform: 1/32 = 3.1%), and the mod-32 histogram is flat to within 1.1% | none |
| 66 | Repetitiveness clusters breaks? | A (yes, in both spaces) | meas.: in row space, 77.9% pure groups against 15.1% if breaks were uniform, with 9.0 breaks per impure group. In traversal space, 84.8% of gaps are ≤ 2 steps. R10: neither summary determines the other | cross-block correlation of the two (optional) |
| 67 | Choose segment starts to cheapen navigation | N | R10: a critical-path question, not total work. N1 makes starts a construction input (caps) | a gain measurement, if a scheduler change is proposed |
| 68 | Split intervals to cut long paths | N | R10: total work separated from path length | the cost of a particular layout |
| 69 | Amortised bound under 512 segments | A | R10: segmented work ≤ an + S·Δ_Φ. N1 counts its work directly (H = Σ⌈ℓ/8⌉) | Δ_Φ for move-structure designs only |
| 71 | Low and high parts load independently? | N | R10: decided by the address dependencies, which can be read before benchmarking | the payoff (measurement) |
| 74 | Slow records bunch across lanes? | A (no) | meas.: the shipped scheduler simulated on 12 blocks (32 lanes in lockstep; r₀ plus 511 xorshift starts; next unwalked segment on hand-off), against phase-shifted lanes and iid flags (round 17's design). **Breakpoint rows:** mean 2.03 exceptional lanes a full round, variance 1.88 against binomial 1.91, P(X ≥ 8) = 0.07%. **Compact-Ψ slow rows** (impure groups): mean 7.2, variance 5.39 against 5.56. All three schedules match. Bursts lie along each lane, and the pseudo-random starts decorrelate the lanes. 77% of rounds are full width | the stall cost per exceptional lookup is a separate, hardware question |
| 77 | CI 2-spread amplification predicts the 10-core knee? | N | R12: A(2) does not determine A(10); no validated predictor | paired multi-concurrency curves |
| 78 | lbzip2's 1→4 amplification a stable baseline? | A (no) | R12: the recorded spread is 1.11–1.42× across runners | none |
| 80 | 2-core fingerprint preserves rankings? | N | R12: not guaranteed by construction | matched ranking comparisons |
| 81 | Fingerprint: more MLP or less compute? | A (method) | R12: timings alone cannot identify the mechanism | per-machine resource counters, if needed |
| 82 | lbzip2/pbz2 controls after #69 | N | R12: unmatched cross-run controls are not comparisons | a matched run |
| P132 | USL as an extrapolator | rejected | R12: three points can't validate a knee; descriptive use only | — |

**The tally of the 47 open questions** (after rounds 10–14; corrected: #84 is one of the
register's 6 conditional rows, not an open one):
- **11 answered:** #4, #25, #58, #63, #64, #65, #66, #69, #74, #78, #81;
- **19 narrowed to a named measurement:** #2, #3, #11, #15, #19, #21, #37, #38, #39,
  #44, #45, #46, #53, #67, #68, #71, #77, #80, #82;
- **17 worked below:**
  - the ten in the next table;
  - the literature rows, #28–#33 and #36, which are with the literature agent.

**The completion standard** (the researcher's, in their burn-down status report). Every
row ends with one of three dispositions:
- **answered:** a proof, a counterexample, a primary source or a measurement;
- **conditionally unresolved:** the missing evidence is named, with the reason it can't
  be inferred from existing work;
- **superseded:** another result subsumes it, leaving no independent uncertainty.

On that standard, "narrowed" above means conditionally unresolved, with the
evidence named.

**The remaining open rows:**

| # | Question (short) | Disposition | The missing evidence, and why it can't be inferred |
|---|---|---|---|
| 20 | E8: M2 under four copies | conditionally unresolved | The pre-registered four-copy run (the code is on `scratch/move-structure`). M2 lost at one thread because its walk was instruction-bound. Under four copies the T-vector walk's cost doubles (5.0 → 10.4 c/B, measured), so the result depends on the balance of misses against instructions at four copies, which is machine-specific. Low priority: N1 is the same family with k = 8 and a cheaper lookup |
| 43 | 64-byte fixed record ≈ a `u32` load? | conditionally unresolved | A dependent-load microbenchmark: chained lookups at 32 lanes, with the line and extraction cost against a direct `u32`. It depends on extraction instructions against miss latency, which is not inferable. It could run on CI, since it needs no PMU |
| 47 | A different organisation enables more MLP at more bits? | superseded for composed records; conditionally unresolved otherwise | N1 is such an organisation: about 1.5 MB, one lookup per 8 bytes. Its Gate C answers #47 for it. No other organisation is proposed |
| 48 | The space–throughput frontier across machines | conditionally unresolved | The frontier runs of #102. The frontier is a measured map per microarchitecture, so it can't be derived |
| 51 | Cost of non-translation payload loads | moot for N1; conditionally unresolved for compact Ψ | N1's records cover every row uniformly, so it has no exceptional payload path. For #112's formats, the #43 microbenchmark decides it |
| 54 | Does any representation improve best-concurrency throughput? | answered negatively for compact Ψ as built; conditionally unresolved for N1 | #112: the knee moved from 4 to 6 blocks, but the best arm reached 0.687× of the shipped 4×2 teams. N1 is untested (Gates B and C) |
| 72 | What the compiler emits for in-register select and extraction | conditionally unresolved | It needs a prototype of the format; none exists. It is cheap once code exists (read the disassembly) |
| 73 | CPU-specific alternatives to PDEP | pending a primary-source check (with the literature agent) | instruction tables for Intel Haswell+, Zen 1/2 and Zen 3, plus PDEP-free select references |
| 75 | Which structure predicts expensive extra loads under 32 lanes | narrowed | meas.: exceptional rows arrive in bursts along trajectories (84.8% of gaps ≤ 2), so the predictor is the burst structure rather than the mean rate. The cost under 32 lanes needs a lane simulation or a measurement |
| 76 | How much of Ψ can be rebuilt from compact information near flat-table speed? | conditionally unresolved, now quantified on the space side | N1 rebuilds all of Ψ⁸ from 1.5 MB (median) with exact semantics. The speed side is Gates B and C, with the builder's traffic modelled as adverse (§3.1(h)) |

**The register's 6 conditional rows:**

| # | Question (short) | Disposition | Reason |
|---|---|---|---|
| 5 | Best hint or lane count, 1 against 4 threads | answered (boundary-specific) | yes on CI's four-copy walk stage, no on the i9's full decoder (P25). Both are recorded at their boundaries; no independent question remains |
| 12 | H5: parallel scanner (speculative markers) | conditionally unresolved, with a trigger | R13: the pipeline ceiling, min(scanner, workers). The trigger is workers reaching the scanner's rate, with headroom of about 2.5× (4-core) and 2.1× (i9) (§3.3). Its value can't be measured before then |
| 18 | E6: libsais's walk parameters in the Ψ² port | superseded | E1 failed its registered pass, so E6's precondition doesn't hold. It reopens only with E1 |
| 62 | Can compact ψ compete with the shipped decoder on the i9? | answered (bounded) on the i9; conditionally unresolved on low-L3 machines | Modelled ≤ 1.25× for the inversion and ≤ 1.1–1.15× for the decoder (feasibility review). It reopens on a machine with less L3 per core, such as a laptop (B1) |
| 83 | A twin that loops inside one process (harness) | conditionally unresolved | a harness design (P130). It is measurement infrastructure, so there's no scientific uncertainty beyond whether to build it |
| 84 | E3's fill-buffer headroom at 4–10 workers | conditionally unresolved | R12: the lane sweeps narrow it (lanes 8 and 16 measured worse). The counter verdict was never posted; it needs PMU attribution on the i9 |

**After the literature rows return,** every one of the 84 questions will have a
disposition. Answered rows stay answered. The open and conditional rows end as follows:
- **answered (including negative and boundary-specific):** 11 open + #5 + #62 (i9);
- **superseded:** #18, and #47 for composed records;
- **conditionally unresolved, each with its evidence named:** the rest;
- **the literature rows:** pending.

**#53's evidence is now adverse as well as positive.** The algorithm exists, but its
modelled traffic is about 23 bytes per output byte, against the T-vector build's 5
(§3.1(h)). Confidence in N3's 1.8–2.5× one-core projection (§3.1(f)) is lowered
accordingly, until Gate B reconciles the model with measured cycles.

The register's own status column is updated only when the maintainer accepts these
dispositions.

**Next, in order:**
1. **The front end and whole-decoder cluster** (#11, #15, #19–#21; the researcher's
   round-12 pointer). Most of these are recorded experiments whose registered runs were
   never made.
2. **The literature cluster:** primary-source reading, with no runs.
3. **The implementation-bound questions,** each given a pre-registered test against
   Gates B and C, so that none stays open without a named decider.
