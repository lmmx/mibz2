# 2026-11-09: The pair table built from run intersections (opt-in), and a diagnostic registered (DRAFT, NOT FROZEN)

**Status: a draft registration, NOT FROZEN, NOT RUN on CI.** The coordinator freezes it, revises
it or drops it. Nothing here changes the decoder, with or without the new feature. **Revised
2026-11-09 (§10)** for Guardian D046 (the safe boundary and B5's evidence) and the Researcher's
REVISE (6730270019): the registration's text, VALID, fairness and wording.

**The input.** The Researcher's REVIEW, #145 comment 6729693845 (report
`state/reports/run-constructions-6729486092/report.md` at `1db0acdb`, §2), under Louis's
direction to make the run-structure ideas pay. It proposes building #34's dense pair (ψ², two
bytes a step) table from the last column's run intersections instead of by a per-row bigram
counting sort, then comparing it first against the old builder with the old consumer unchanged,
then against production's one-hop path.

**In short:**
- **#34's pair path is recovered** from `perf/pair-walk` (head `a3687d8f`): its table, its
  per-row builder `build_pairs`, `pair_start` and its flagged 32-lane walk, in
  `src/pair_runs.rs` behind the non-default `pair-runs` feature.
- **The run-intersection builder, `build_pairs_runs`, gives #34's words exactly:** checked
  exhaustively on small columns, on random ones, by a differential fuzz against a stable-sort ψ²
  oracle, and on all 431 bench-corpus blocks. Its unsafe code is under Miri, behind checked safe
  entries (§10).
- **It removes the per-row bigram histogram and scatter.** It reads the column and writes the
  table in order; its random accesses are one gather a run into r-entry arrays. By
  instruction count (callgrind, counts, not timing) it is **12.4 instructions a row**, against
  #34's **15.0** and production's one-hop build's **6.2**.
- **The spare-bit idea is not built. Its census:** a 3,516-byte directory against the old
  lookup's 28,125 bytes, and 0.34% of Wikidata's rows flagged, statically and dynamically.
  But the old lookup's correction loop runs only **0.002 times a step**, so the old path is
  already one load and one compare. Its prize is small.
- **Registered (draft):** Q1, the build cost ratio run / row; Q2, the inverse-BWT entry point
  (not the whole decoder) against production's in-place one-hop path, the R candidate with its
  admission rule. Each has positive and negative stops, run once through `sweep.yml`
  (`tool: pair_runs`).
- **What #34 suggests:** the consumer, more than the builder, is the likely cost. #34's walk
  cost about twice a one-hop step for each pair step, and production's walk has since become
  the faster room walk. Counts alone do not decide it (§4). **The drafter's prior is Q1
  INCONCLUSIVE and Q2 NEGATIVE STOP.**

Labels, as in #266 and #269:
- **proved**, a short argument given here;
- **checked**, by tests or by counts;
- **model**, an estimate with its assumptions;
- **local, indicative**, a timing on the shared machine, never evidence.

## 1. #34, recovered

`git log --all -S pair` leads to `perf/pair-walk` (PR #34, closed unmerged; journal
`2026-10-27-pair-walk.md`). On CI it lost by 21–52% at one thread and by 21–30% at four,
against 0.1.4. Locally its build was 7.1–7.6 c/B against the one-hop build's 2.5. Its walk took
half the steps at about twice the cost each (≈10 c/B against ≈10.4).

- **The word (#34's):** row r holds `L[r] | ψ²(r) << 8`. The step's second byte, L[ψ(r)] =
  F(r), comes from a first-byte lookup: a 32 KB table indexed by `r >> fshift` (`fshift` makes
  n >> fshift < 2^15, so 5 at 900k), then a correction loop over the bucket ends. Bit 31 marks
  a segment start; bit 30 marks a start's LF row.
- **What is #34's code, unchanged:** `build_pairs`, `pair_steps`, `pair_start`, and the walk's
  lanes, `take`, `finish`, `grow`, `step` and the joins.
- **What changed, and why:**
  - **The walk is cut in two, `setup` and `walk`.** The column is read only in `setup`, so the
    output can go over it, as production's in-place path does.
  - **The prefetch** takes today's `prefetch::<PF_L1>` instead of #34's boolean form. The hint
    is the same.
  - **`Pairs` is a kernel with `invert_in_place`.** It uses production's `Segmented` below
    `MIN_SEGMENTED`, as #34 did with `Segmented` of its day.

## 2. The construction

Let L run i be (c_i, p_i, ℓ_i). Its F interval is I_i = [a_i, a_i + ℓ_i), where a_i = C(c_i)
plus the copies of c_i in L before it. Ψ maps I_i onto J_i = [p_i, p_i + ℓ_i) by
Ψ(q) = p_i + q − a_i. Here Ψ = ψ, the T-vector's link.

**The cells (proved).** Cut [0, n) in x at the L runs' starts and at the F intervals' starts.
On a cell [x, y) ⊂ J_i ∩ I_j, at q = a_i + x − p_i:

ψ(q + t) = x + t and ψ²(q + t) = p_j + x − a_j + t.

So **D[q + t] = L[q + t] | (p_j + x − a_j + t) << 8.** These cells are line A's, at most
2r − 1 (#266). Equivalently, D[LF(x)] = (T[x] & !0xff) | L[LF(x)]: the pair table is the
one-hop table's links, moved by LF, with the column's bytes.

**The low byte is L[q], #34's, not the Researcher's F(Ψ(q)).**
- The REVIEW's D[q] = (Ψ²(q) << 8) | c_j fits a walker in F coordinates, which emits F(q),
  then the low byte.
- #34's walker emits L[r], then F(r). Only #34's word keeps its consumer unchanged.
- **The cost of keeping it:** L[q] is not constant on a cell, so the fill ORs in the column's
  bytes, which are contiguous at q.

**The build, `build_pairs_runs`:**
1. **The runs**, in one pass over L. A run starts wherever a byte differs from the one before.
   The pass finds them 8 bytes at a time, with a SWAR byte-difference mask. Each run's a
   comes from the counts and the copies of its byte so far.
2. **The F order:** a stable counting sort of the r runs by byte, into `[a, p]` records with
   a closing `[n, 0]`, and each record's L index.
3. **A merge in L order:** the F interval each run's image J_i starts in. The L starts and
   the F starts both rise, so this is O(r).
4. **The fill, in F order, so in rising q.**
   - Per F interval, one gather: its run's merge result.
   - Then its cells, 8 words a store: AVX2 where present, else SSE2.
   - A store runs past its cell's end into rows that later cells (in rising q) write again,
     or into 8 words of slack. Near the block's end the stores are exact.

**What the REVIEW said to charge, and where it went:**
- **The F ordering:** step 2.
- **The non-monotone destination writes:** the fill is put in rising q instead, so the table
  and the column are streamed. Their non-monotone order is paid as step 3 and one gather a run
  into the r-entry arrays, about 0.07n words on Wikidata.
- **The first version filled in x order, with q jumping run to run.** It cost more
  instructions and was replaced before any CI run. Both versions are word-identical.

**Safety (proved; Miri):**
- **The record writes in step 1:** at most n, since the starts rise strictly below n.
- **`set_len(n)`:** the I_i tile [0, n), given two checks: the runs' lengths per byte equal
  `counts`, and the counts sum to n (in u64). Otherwise the function returns an error, and
  nothing falls back to `build_pairs` (§10). Each interval's cells cover it in rising q, so
  every slot is written. A store past a cell only goes upward, into later cells' rows.
- **Serial only:** a store past a cell writes words a later cell writes again. Adjacent cells
  filled in parallel would race on them, which no later store repairs, so the fill must never
  be parallelised across cells as it stands (D046; `fill`'s contract).
- **The fill's raw reads and writes** are each behind a check that q + len + 8 ≤ n, computed
  without overflow. Its interval index is asserted below r, so a wrong input panics or writes
  wrong words in bounds.
- **The AVX2 fill** is an `unsafe fn` reached only through `is_x86_feature_detected!`.

**The admission (part of the R candidate; see §6 and §8):** `Pairs` on the run builder samples
the column's run density, the run starts in one 8-byte word of every 16 (the comparisons at
offsets 1-8 of every 128 bytes). A block above `ADMIT_DENSITY` = 0.15 runs a row (strictly
above) goes to production's `Segmented`. The need for it followed the local pilot's losses on
records and random, and 0.15 was picked from the corpora's observed run densities (§8). **The
sampler is a heuristic, not a density bound:** the Researcher's witness, a 16,384-byte column
that samples to 0 at a true density of 0.9375, is admitted (kept as the test
`admission_heuristic_aliases`). It is frozen literally for the registration.

## 3. Correctness (checked)

- **`pair_runs::tests::exhaustive_small`:** every column over 3 bytes of lengths 1–8 and over
  2 bytes of lengths 9–12, 17,520 columns (the earlier 17,511 was an arithmetic slip; the test
  asserts the count). At every primary index, it checks the same words,
  the same LF rows (every row), the same first row as `pair_start`, and the words against ψ²
  by definition (from `bwt::build`). The cells are below 2r.
- **`random_columns`:** uniform, full-alphabet, periodic, constant and run-structured columns,
  at sizes from 1 to 200,000 (the 16-wide paths of `build_pairs` included).
- **`pair_walk_equals_plain`:** #34's test, on both builders, in place as well, against
  `bwt::walk`: 4 column shapes, 3 primary indices, 2–4,096 segments.
- **`miri_pair_runs`, under Miri:** both builders, and the walk's raw-pointer lanes on the
  run-built table. It passed with the SSE2 fill (default target) and with
  `-C target-feature=+avx2` (the AVX2 fill). About 2 minutes locally; `miri.yml` now also
  runs on changes to `src/pair_runs.rs`.
- **All 431 bench-corpus blocks** (`examples/pair_runs.rs`, its validation, run with
  `ROUNDS` 0). On every segmented block (233), it checks:
  - the same words, LF rows (512 rows) and first row from both builders;
  - the words are ψ²;
  - the cells are below 2r.

  On every block it also checks:
  - P, R and O give the same output;
  - the run-built walk, with no admission, gives that output too;
  - RLE1 and the block's CRC.

  The whole output is `mibz2::decode`'s, and the corpus's size and decoded size are
  `bench/wikidata.json`'s. The synthetic corpora pass too.

## 4. Census (checked, counts)

| corpus | r/n | cells/n | cells/r |
|---|---|---|---|
| Wikidata (233 segmented blocks) | 0.0722 | 0.0881 | 1.220 |
| synthetic text | 0.247 | 0.367 | 1.48 |
| synthetic random | 0.992 | 1.000 | 1.01 |
| synthetic records | 0.997 | 1.000 | 1.00 |
| synthetic one byte, pattern | ≈0 | ≈0 | — |

Wikidata's cells/n matches #266's line-A figure, 0.0881.

**Instructions (callgrind; counts, not timing).** The first 4 segmented Wikidata blocks,
3,599,923 rows; each builder called once, in its own buffers:

| builder | instructions | a row |
|---|---|---|
| #34's `build_pairs` + `pair_start` | 53.9 M | 15.0 |
| `build_pairs_runs` (AVX2 fill) | 44.8 M | 12.4 |
| production's one-hop `bwt::build` | 22.2 M | 6.2 |

So the run builder does 17% fewer instructions than #34's, with no per-row random RMW (#34's
scatter writes `p[x]` and `b2[w]` at random). It does twice production's one-hop build. The
archive's reopening bound for ψ² was "a build at no more than today's one-pass build plus
1 c/B" (`2026-11-07-left-field-archive-check.md`). **Counts alone do not decide it.** The cycle
difference is I_run / IPC_run − I_one / IPC_one, not the instruction difference at one IPC:
12.4 / 4 − 6.2 / 2 = 0 cycles with both IPCs below 6 (the Researcher's counterexample). M1
estimates it, in nominal clock-scaled units (§6).

**The spare-bit census (the REVIEW's second idea, not built).** The directory is one byte per
256 F rows; a block is flagged where its first and last rows' bytes differ in their high five
bits.

| | Wikidata | text | random | records |
|---|---|---|---|---|
| directory, B a 900k-class block | 3,516 | 2,819 | 2,605 | 3,125 |
| old shift-five lookup, B a block | 28,125 | 26,912 | 27,085 | 25,001 |
| flagged blocks a block (max; bound 31) | 12.0 (13) | 18 (19) | 31 (31) | 31 (31) |
| distinct flagged rows | 0.342% | 0.620% | 1.190% | 0.992% |
| one-chain walk, steps on flagged rows | 0.343% | 0.612% | 1.196% | 0.993% |
| old lookup's correction loop, iterations a step | 0.002 | 0.004 | 0.004 | 0.005 |

- **The dynamic rate here** is one chain of n/2 pair steps from ψ(orig_ptr): the rows whose
  first byte a step looks up. It is not the segmented walk's exact visit set. The two match
  the static fraction closely on these corpora.
- **The deciding figure is the last row.** The old lookup is already, on almost every step,
  one load from a table that fits L1 and one compare. The spare bits would replace that with
  a tag extraction, an OR, a flag test and a pointer mask, and save about 24.6 KB of the
  lookup's footprint.
- **Expected:** no speed prize worth a new consumer at 32 lanes, unless that footprint
  matters under cache pressure. Not built, and not proposed ahead of Q2.

## 5. The diagnostic: `examples/pair_runs.rs`

```text
cargo build --release --features pair-runs --example pair_runs
pair_runs FILE.bz2 [ROUNDS] [GHZ]      (ROUNDS 2-31, default 15; 0 validates only)
env PAIR_RUNS_RAW (raw paired timings, CSV), PAIR_RUNS_HEAD, PAIR_RUNS_SPEC
```

Each block's last column is made once by `mtf::mtf`, untimed. Every arm runs over every block
in decode order, on one thread. The arms' order rotates by round, and each ratio is paired by
round: the geometric mean and its 95% t-interval.

- **(a), the build alone,** on the segmented blocks, each arm in its own buffers (its own
  table and its own LF rows; until §10 the row and run arms shared the LF rows' vector):
  - `row`: `build_pairs` + `pair_start`;
  - `run`: `build_pairs_runs`, giving the same table, LF rows and first row;
  - `one`: production's `bwt::build`, a reference only.
- **(b), the whole entry point,** `InverseBwt::invert_in_place` over a fresh copy of the
  column (the copy untimed), then RLE1 and the CRC, untimed and checked:
  - `P`: production's `Segmented` (`build_one_pass`, then the 32-lane room walk), the real
    in-place path;
  - `R`: the run-built pair table and #34's consumer, with the admission;
  - `O`: #34 as it was (row builder, same consumer, no admission), a reference.

  (b) is the inverse-BWT entry point, not the whole decoder: MTF, Huffman, scheduling,
  concurrency and the later stages are outside it. The four-copy run is a contention
  diagnostic, not a multithreaded decode.

The files: Wikidata (the corpus `bench.yml` uses), then the test corpora's synthetic inputs at
level 9.

**On CI.** Dispatch `sweep.yml` with `tool: pair_runs`, `rounds: 15` and `frozen` the frozen
head, at that head, once.
- **Before any copy starts,** the workflow writes `plan.json`: the rounds, the clock read, the
  CPUs and the copies they imply (`1`, then `N-cpuC` for each), the head, the frozen head, and
  the corpus's path, spec and SHA-256.
- It runs one copy pinned to CPU 1, then one copy per CPU at once.
- Each copy's exit status goes in `status-*.txt`, captured by `if` so no errexit skips it; its
  raw paired timings in `raw-*.csv` (file, round, arm, seconds, and the file's paired blocks,
  paired rows and output bytes).
- **The `Validity` step, `bench/pair_runs_validity.py`, writes `validity.txt`.** It fails the
  job unless all of these hold (each has a fixture test in `bench/test_pair_runs_validity.py`):
  - the plan: rounds 2–31, a finite positive clock, the copy set the CPUs imply, the head the
    frozen head, the corpus's SHA-256 the spec's and the plan's;
  - the inventory: every planned copy's three files, and no unplanned copy's (a wholly
    missing child is found from the plan); every status 0; every VALID line;
  - every raw CSV: its header and the exact corpus × round × six-arm matrix, nothing missing,
    duplicated or extra; every total finite and positive (a build arm's exactly 0 only on a
    file with no paired block); each file's counts the same throughout; Wikidata paired;
  - from the single copy's raw timings: every registered interval and M1's difference
    present, with finite, ordered endpoints.

  Only then does it print the readings, recomputed from the raw timings. A run that is not
  VALID prints its problems and no reading.
- **A copy's VALID line needs:**
  - a finite, positive GHZ;
  - every registered interval and M1's difference (nominal c/B and ns a row) present, with
    finite endpoints;
  - every arm's every round total finite and positive;
  - the head, 40 hex digits;
  - the corpus's size and decoded size.

  Its readings are printed only after it; a copy that is not VALID prints none and exits 3.

## 6. The registration (DRAFT, NOT FROZEN)

Judged on the **pinned single copy, on Wikidata**. The four-copy copies are exploratory. The
readings are printed mechanically, labelled UNJUDGED until the coordinator names the
registered run, and only once the run is VALID.

**The candidate, and what was seen before.** The Q1/Q2 thresholds below are those written
into the harness before its first timed run, kept as they were. The local pilot rounds (§8)
are development data: excluded from every registered interval and verdict. The candidate was
revised after profiling (§8), and the admission rule was added after the pilot's losses; the
registered run tests the final, fixed candidate on its registered runner, not a pristine
discovery. **The R candidate is the deployable hybrid:** the run builder, #34's consumer, and
the admission (sampler, constant, strict `>`, fallback to `Segmented`, and its per-corpus
taken/declined counts), all charged inside the timed entry point.

**Q1: does building from run intersections remove the old build's cost?**
R_build = run / row, from (a): the two builders alone, over every segmented block, with no
admission. It is the only isolated builder comparison; R / O, below, includes the admission
and so is a policy contrast.
- **POSITIVE** if the geometric mean is ≤ 0.50 and its upper bound < 0.60. Most of #34's
  build penalty is removed.
- **NEGATIVE STOP** if its lower bound > 0.80. The construction saves under 20%: this
  registered candidate and configuration is closed. (Not every run-built pair table: a
  materially cheaper builder is a new registration under C3.)
- **INCONCLUSIVE** otherwise. No re-run without a new registration.

**Q2: does the run-built pair path beat production?**
R_whole = R / P, from (b): the inverse-BWT entry point, admitted R against production's P.
- **POSITIVE** if all three hold:
  - the geometric mean is ≤ 0.95;
  - its upper bound is < 1.0;
  - the worst synthetic corpus's geometric mean is ≤ 1.05 (a point-estimate screen, not an
    upper confidence bound on regression).

  A production prototype may then be registered, for `bench.yml`'s paired A/B to decide.
- **NEGATIVE STOP** if the lower bound is > 1.0. The run-built pair with #34's consumer is
  slower than production, and this candidate pauses. Reopening does not need a different
  consumer: a materially cheaper builder, a changed admission, a different target or another
  justified mechanism can warrant a new registration (PROTOCOL §11, C3).
- **STOP (synthetic)** if Wikidata passes but a synthetic corpus's point estimate is above
  1.05 (the screen above).
- **INCONCLUSIVE** otherwise. No re-run without a new registration.

**M1 (exploratory): the archive's reopening bound, timed.** The run build less the one-hop
build, in **nominal c/B** of the segmented rows (wall seconds × the GHZ read once from
`/proc/cpuinfo`: clock-scaled wall time, not measured cycles; frequency changes and
descheduling are not integrated), as a paired mean and 95% interval; the raw nanoseconds a row
are printed beside it. Its difference is required for VALID.
- **MET** if the upper bound is ≤ 1 c/B.
- **NOT MET** if the lower bound is > 1 c/B.

O/P and R/O are printed as references and are not gates.

## 7. What #34 implies, and what is expected

- **#34 lost on both halves.** It was closed on the build's 3× cost and on a walk with no
  per-byte saving: half the steps, each about twice the work. Its loss was +40.6% at one
  thread on Wikidata, and the build accounted for about 5 c/B of it.
- **What this line can fix:** the run-intersection build removes the bigram histogram and the
  per-row scatter (Q1 measures what that saves).
- **What it does not change:** the consumer. Since #34, production's walk has become the room walk.
  It is the flagged walk in passes with a shared offset (#37: Wikidata −10.3% at one thread on
  CI, walk stage 4.37 → 3.81 c/B on the Xeon runner), then #157 and #171. #34's pair walk is
  the flagged walk #34 was built on, two bytes a step, plus the first-byte lookup.
- **So the expectation is that even a free build** leaves R/P near #34's walk against today's
  room walk. The binding census (`2026-11-05-binding-census/nogo.md`) found #34's port at least
  10% slower at four threads, with about 5 c/B of build. This is a prior, not a necessity.
- **Expectation, plainly:**
  - **Q1: INCONCLUSIVE, perhaps POSITIVE on very repetitive data,** but not at 0.50 on
    Wikidata. The run builder's instructions are 0.83 of #34's. Its cycles should fall
    further, since #34's are partly random RMW misses, but not to half.
  - **Q2: NEGATIVE STOP.** The pair consumer, not the build, is the larger cost, and nothing
    here changes it.

  A Q2 POSITIVE would challenge a prediction transferred from #34 (the builder, baseline,
  hardware, allocation and admission all differ), not #34's own measurements, which stand; a
  mechanism would be found before any adoption.

## 8. Disclosures

- **Local, indicative timings were seen while developing** (the shared machine, 2–3 rounds,
  load 1–4.6), to form the hypothesis and to find the builder's costs. **Never evidence.**
  - Wikidata, (a): row about 5.3–6.0 c/B, run 4.1–5.2, one-hop 1.6–2.4. R_build was
    0.73–0.87 across builder versions.
  - Wikidata, (b): P 6.0–8.8 c/B, R 8.2–11.0, O 10.1–11.6.
  - The synthetic records and random corpora: the run builder was 4× #34's on its own, and
    up to 6.5× production's whole path before the admission.
- **What was fixed when:**
  - The Q1/Q2 thresholds were written into the harness before its first timed run.
  - **The admission (`ADMIT_DENSITY` = 0.15) was added after that local reading** on
    records and random: the need for it followed those losses, a serious observed problem
    on the pilot (not a proof about any CI result).
  - **0.15 was picked from the corpora's observed run densities** (Wikidata 0.07, text 0.25,
    records and random about 1) to separate those families, before the registered run, but
    not from a timing sweep of thresholds. That is corpus-informed selection.
- **The admission is in the registered R candidate** (Researcher, 6730270019), frozen
  literally with the sampler; it is the shape any production use would need, as #269's
  fallback is. It is not tuned after the registered result.
- **The pilot samples are excluded** from every registered interval and verdict.
- **The builder was revised after local profiling** (x-order fill → q-order fill; SWAR run
  scan; AVX2 fill). Each version passed the same word-identity checks. Its revision is part
  of building a fair candidate, not a fitted threshold.
- **No CI timing has been run.**

## 9. Not done

- The spare-bit consumer (§4: census only).
- Any change to #34's walk, or a port of the room walk's protocol to pairs. The REVIEW notes
  that this needs rephased segment starts, two-row marker tests, odd tails and periodic orbits.
- Admission tuning, multi-threaded decode, and the workstation.

## 10. Revision, 2026-11-09: Guardian D046 and the Researcher's REVISE

**Still a DRAFT, NOT FROZEN.** This revision answers Guardian D046 (6730359769: the STOP on the
exposed pair path's unsafe public state, and B5's evidence) and the Researcher's REVISE
(6730363373, report 6730270019). It does not lift the STOP, freeze the registration, or
pre-empt either reviewer: each decides on its own evidence. No timing was run.

### D046: the safe boundary

| D046 obligation | Fix | Test |
|---|---|---|
| 1. `pair_start(&[], &PairTable::default(), 0)` reaches `first.get_unchecked(0)`; an out-of-range `orig_ptr` on a built table likewise | The builders check `1 <= n <= MAX_N` and `orig_ptr < n` before anything unchecked, and on success alone return a `Built` token (private fields) borrowing the table and the column immutably. `pair_start` takes a `Built` and checks its row. `first_byte` is the private `unsafe fn first_byte_unchecked` with a `# Safety` contract | `miri_witness_default_table_and_bad_row`; doc tests: a `Built` forged (`compile_fail,E0451`), `pair_start` on a bare table (`E0308`), the table rebuilt under a `Built` (`E0499`) |
| 2. `ll=[1]`, `counts[0]=1`: equal sum, valid buckets, `pair_steps` reads `ll[1]` | `build_pairs` checks each byte's count against the column, from the per-bucket histogram it makes anyway (summed over the buckets), before the scatter. The counts' sum is taken in u64. `pair_steps` is a private `unsafe fn` with a `# Safety` contract; each call site states why it holds | `miri_witness_wrong_histogram_and_no_fallback`: the witness, a u32 sum that wraps to n (`counts[0]=u32::MAX`, `counts[1]=2`), equal-sum moves near and far, a sum one more; each refused by both builders and by `setup` on both. The fuzz refuses an equal-sum wrong histogram on every case |
| 3. `setup(Build::Runs)` falls back to the unchecked row builder on failed validation | The fallback is gone. Both builders return `Result<Built, PairError>`, and `setup` returns the error. A failed check is never a reason to try the other builder | The same test: `setup(Build::Runs)` returns `Counts`, and the scratch then builds and walks a good block |
| 4. Replacing the public `scratch.t` leaves stale starts; `walk` reads the empty table | `PairScratch`'s fields are private. `setup` returns a `Prepared` token (private fields) borrowing the table, starts and LF marks immutably and the walk's buffers mutably. `walk` takes it by value. The walk's lanes are the private `unsafe fn walk_lanes`, whose contract is the prepared state | `miri_witness_stale_state` (a refused `setup` clears the state, and no `Prepared` exists for it); doc tests: the table replaced (`E0616`), a `Prepared` forged (`E0451`), the scratch prepared again under one (`E0499`) |
| Size and overflow limits | `MAX_N` = 2^22 (links below 2^22, words below 2^30, bits 30 and 31 free; the table comment's "2^20" corrected); segments 1 to `MAX_SEGMENTS`; the LF rows strictly rising below n; a walk of 0 lanes is a compile error | `too_long_refused`, `miri_rows_refused`, `miri_witness_stale_state` (segments 0, `MAX_SEGMENTS + 1`, `usize::MAX`) |
| Never parallelise adjacent cells with the spilling stores | A serial-only contract on `fill` and in the module's doc. Nothing runs the fill in parallel | (a contract: there is no parallel variant to test) |
| A valid prepared fast path kept | The checks are per block, outside the loops: the bigram histogram's 256 × 256 totals, and an O(256 + rows) pass. The hot loops are unchanged | the existing word-for-word and walk tests |

**The compile_fail doc tests' error codes** are checked on nightly (`miri.yml`'s doc-test step,
`cargo test --features pair-runs --doc pair_runs`); all 8 doc tests pass locally on nightly.

### D046: B5's evidence on the repaired code (local; CI's Miri run is the record)

**Miri**, at source commit `709613e541a455b1754300972bff4ebf36c9e9ce` (the code as pushed; the
later commit changes only this journal), `rustc 1.101.0-nightly (16d1ffac1 2026-10-26)`,
`miri 0.1.0 (16d1ffac17 2026-10-26)`, `cargo +nightly miri test --lib -- pair_runs::tests::miri_`:

| Run | Result | Reached |
|---|---|---|
| default target, `MIBZ2_EXPECT_SSE2=1` (forwarded) | 6 passed, 408 s | the SSE2 fill, asserted in every build the tests check (`expect_fill`) |
| `-C target-feature=+avx2,+bmi1`, `MIBZ2_EXPECT_AVX2=1` (forwarded) | 6 passed, 407 s | the AVX2 fill, asserted likewise |
| negative control: default target, `MIBZ2_EXPECT_AVX2=1`, `miri_fill_reached` | **failed, as it must**: "AVX2 fill expected, and the SSE2 one ran" | the assertion is not vacuous |

The six tests cover both builders on short and long cells, vector spills and scalar tails
(asserted against the oracle's classification, including a 13-row end cell in an exact
allocation), buffer reuse, the setup/flagged walk in place and out of place, start and LF-row
ends and buffer growth (asserted), more segments than rows, and the four witnesses' refusals.
`miri.yml` runs the same three steps on CI, plus its existing `miri_*` set and the nightly doc
tests. The scalar fill off x86_64 is not exercised (no such runner): a named limitation.

**The differential fuzz** (`pair_runs::tests::fuzz_long`, seed `0xb5d046`, 300 s, the `fuzz`
profile: release speed with debug assertions and overflow checks; source as above):
`MIBZ2_PAIR_FUZZ_SECS=300 cargo test --profile fuzz --lib pair_runs::tests::fuzz_long --
--ignored --nocapture`. Each case draws one of 11 column kinds (constant, alternating, periodic,
runs around 8/16/64, random runs, small and full alphabets, the sampler-aliasing shape, a ramp,
sparse insertions, the extreme bytes) at a boundary length (1-20, 23-25, 31-33, 47-49, 63-66,
71-73, 127-129, 255-257, 511-513, 1023-1025) or a random one up to 6,000, with three primary
indices and random LF rows. Both builders (the run builder on a reused table poisoned by another
build) are compared with a stable-sort ψ² oracle: the words, the LF rows, the first row, and the
fill's cells, vector cells, spills and scalar cells, cell by cell; equal-sum wrong counts and an
out-of-range primary index are refused; the walk (1, 7 or 32 lanes, 1 to n + 8 segments, in place
or not, either builder) is compared with the oracle's text; and every 64th case `Pairs` takes a
dense and a sparse segmented block (admission declined and taken) and a short one.

| | |
|---|---|
| cases | 583,803 (each kind 52,744-53,488), 914.9 M rows |
| builds | 3,502,818 (the run builder's 1,751,409 all on the AVX2 fill: this machine has it) |
| cells / vector cells / spills / scalar tails | 1,272,969,639 / 1,265,356,020 / 1,255,874,934 / 7,613,619 |
| walks (in place) | 1,751,409 (875,508) |
| segment ends at a start / at an LF row; buffer grows | 140,992,903 / 76,066,193; 317 |
| more segments than rows | 342,388 |
| refused invalid inputs | 2,335,212 |
| `Pairs`: taken / declined / short | 9,266 / 9,266 / 9,266 |
| failures | none |

`fuzz_smoke`, a fixed-seed 400-case run of the same harness, is in every `cargo test`. The fuzz's
fills ran AVX2 natively; the SSE2 fill's coverage is Miri's and the shared scalar classification.

### D046: the mutation kills

Each mutant was made on a throwaway local branch from the repaired commit, its commands run, and
the branch deleted; the restored baseline then passed `cargo test --lib pair_runs` (13 passed)
and the nightly doc tests (8 passed). Native is `cargo test --lib pair_runs` (the test profile:
opt-level 3 with debug assertions); Miri is `cargo +nightly miri test --lib --
pair_runs::tests::NAME --exact` with `MIBZ2_EXPECT_SSE2=1`; doc is the nightly doc tests.

| Obligation | Mutant | Killed by |
|---|---|---|
| the physical read/write guard | `len <= room - 8` → `len <= room` | native: `fuzz_smoke` (the fill's cells against the oracle's classification, kind 3, n 1356) and `miri_fill_reached` (a 13-row end cell must be scalar); Miri `miri_fill_reached`: **UB**, an 8-byte read 5 bytes from the end of the column's allocation |
| the spill repair | the F intervals filled in falling order (spills land on cells already written) | native: `exhaustive_small`, `fuzz_smoke`, `miri_fill_reached`, `miri_pair_runs` and others (the words) |
| the final cell / initialisation | the last F interval never filled (`set_len` exposes it) | native: `exhaustive_small`, `fuzz_smoke` (its poisoned, reused table), `miri_fill_reached`, `pair_walk_equals_plain` and others; Miri `miri_fill_reached`: **UB**, uninitialised memory read |
| the merge | `fo[j + 1].start <= p` → `< p` | native: `exhaustive_small`, `fuzz_smoke` and others (the words, or the fill's interval assertion) |
| stable order | the runs' counting sort taken in reverse | native: `exhaustive_small`, `fuzz_smoke` and others |
| the cell arithmetic | a cell's link base + 1 | native: `exhaustive_small`, `fuzz_smoke` and others |
| input validation: the row builder's counts | its per-byte count check removed | native: `miri_witness_wrong_histogram_and_no_fallback` (the standard library's unchecked-index precondition, in `pair_steps`, on the D046 witness) and `fuzz_smoke`; Miri: the same, the precondition abort on the witness |
| input validation: the run builder's counts | its per-byte count check removed | native: `miri_witness_wrong_histogram_and_no_fallback` (the fill's interval assertion), `fuzz_smoke` |
| input validation: the primary index | the `orig_ptr < n` check removed | native: `fuzz_smoke` (the unchecked-index precondition in `first_byte_unchecked`); Miri `miri_witness_default_table_and_bad_row`: the same abort |
| flag bit 31 | `take` keeps START in a start's link (`& !PAIR_LF`) | native: `take`'s link assertion (row below n) in `pair_walk_equals_plain`, `miri_pair_runs`, `fuzz_smoke` and others |
| flag bit 30 | a start's LF row marked START only | native: the walk's join (no successor recorded) in `pair_walk_equals_plain`, `miri_pair_runs`, `fuzz_smoke` and others |
| the stale-pointer refresh | `grow` does not refresh the lane's end pointer | native: the standard library's `offset_from` precondition in `admission_heuristic_aliases`; Miri `miri_pair_runs` (which asserts it grew): **UB**, `ptr_offset_from` across two allocations |
| the state boundary: the table | `PairScratch.t` public again | doc: the `E0616` test compiles, so fails |
| the state boundary: the token | `Prepared`'s fields public | doc: the `E0451` test compiles, so fails |

### The Researcher's REVISE (6730270019)

| Item | What changed | Where, and its check |
|---|---|---|
| Keep Q1/Q2's thresholds; exclude the pilot; disclose the revision | Thresholds unchanged. The pilot samples are excluded from every registered interval and verdict, and the candidate's revisions are disclosed | §6 "The candidate", §8 |
| The admission in the fixed R candidate | R is the deployable hybrid (sampler, constant, strict `>`, fallback, taken/declined counts). The need for admission followed the records/random losses; 0.15 was picked from observed corpus counts. "Fixed before any timing" is removed from `ADMIT_DENSITY`'s doc, and "fixed before CI" from §2 | §2, §6, §8; `src/pair_runs.rs` |
| Q1 a straight builder comparison; R/O not the builder | Q1 is (a), both builders over every segmented block with no admission. R/O is a policy contrast | §6; the harness's doc |
| 17,511 | 17,520; `exhaustive_small` asserts the count | §3 |
| The selector a heuristic | The sampler frozen as it is, documented as a heuristic, not a density bound. The witness is kept: 16,384 bytes sampling to 0 at a true 0.9375, admitted; at 65,536 bytes `Pairs` takes it and decodes it correctly | `admission_heuristic_aliases`; `run_density`'s doc |
| VALID: M1's difference required, finite endpoints | Required in the harness and in the workflow's validity; `diff` returns no interval with a non-finite or unordered endpoint | `examples/pair_runs.rs`; `bench/pair_runs_validity.py`; fixtures `test_missing_m1`, `test_m1_difference_nonfinite_endpoints` |
| VALID: a finite, positive clock | NaN or 0 GHz is NOT VALID (harness and plan) | fixtures `test_bad_clock_nan`, `test_bad_clock_zero` |
| VALID: the planned copies and the exact inventory | `plan.json`, written before any copy, holds the copy set its CPUs imply; each planned copy needs exactly its status, output and raw files, and no unplanned copy may appear. No header-only raw CSV passes | fixtures `test_wholly_missing_child`, `test_all_concurrent_copies_missing`, `test_unplanned_copy`, `test_plan_copies_must_match_the_cpus`, `test_missing_plan`, `test_missing_status`, `test_failed_child`, `test_no_valid_line`, `test_empty_raw`, `test_header_only_raw` |
| VALID: the complete corpus × round × arm matrix | Parsed and checked: every registered file, every round once, the six arms; no missing, duplicated or extra row; totals finite and positive (a build arm's exactly 0 only with no paired block); each file's counts constant. The single copy's intervals and readings are recomputed from it | fixtures `test_truncated_raw`, `test_truncated_row`, `test_duplicate_round`, `test_round_outside_the_matrix`, `test_unregistered_file`, `test_nonfinite_total`, `test_zero_total_on_a_paired_file`, `test_zero_build_arms_only_with_no_paired_block`, `test_counts_change_between_rows` |
| Verdicts only after validity | The harness prints its readings only after its own VALID line, and none when NOT VALID; the workflow's validity prints recomputed readings only when VALID | every fixture asserts no reading when NOT VALID; `test_complete_run_is_valid_with_readings` |
| Evidence bound to the frozen source and the corpus | The dispatch names the frozen head (`frozen`); VALID needs HEAD to be it. The corpus's SHA-256 must be the spec's and the plan's | fixtures `test_head_not_frozen`, `test_no_frozen_head`, `test_corpus_identity`, `test_plan_corpus_identity` |
| A wholly missing child | Found from the plan, not from the files that survived | `test_wholly_missing_child` |
| Statuses captured without errexit skipping the write | Each copy's status is taken by `if`, inside the group, and written whatever it did; a failed copy fails the Sweep step only after every file and the summary are written | `sweep.yml` |
| Fairness: LF scratch | The row and run arms each have their own LF rows' vector (`lf_row`, `lf_run`) | `examples/pair_runs.rs` |
| Fairness: Q2's scope | Q2 measures the inverse-BWT entry point, not the full decoder | §5, §6; the harness's doc |
| Fairness: the 1.05 rule | A point-estimate screen, not an upper confidence bound | §6; the verdict's wording |
| Fairness: M1 | Nominal clock-scaled wall time (seconds × one GHz reading), not measured cycles; the raw nanoseconds a row are printed beside it | §6; the harness |
| Wording | The "IPC above 6" necessity is replaced: counts alone do not decide (12.4/4 − 6.2/2 = 0). A Q1 NEGATIVE STOP closes this registered candidate; a Q2 NEGATIVE STOP pauses this candidate, and reopening needs no different consumer (PROTOCOL §11, C3). A Q2 POSITIVE would challenge a transferred prediction, not #34's measurements | §1 summary, §4, §6, §7 |

The fixture tests run in `test.yml` (`python3 bench/test_pair_runs_validity.py`): 28 tests, all
passing locally.

