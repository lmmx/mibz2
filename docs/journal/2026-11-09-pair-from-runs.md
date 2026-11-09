# 2026-11-09: The pair table built from run intersections (opt-in), and a diagnostic registered (DRAFT, NOT FROZEN)

**Status: a draft registration, NOT FROZEN, NOT RUN on CI.** The coordinator freezes it, revises
it or drops it. Nothing here changes the decoder, with or without the new feature.

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
  exhaustively on small columns, on random ones and on all 431 bench-corpus blocks. Its unsafe
  code is under Miri.
- **It removes the per-row bigram histogram and scatter.** It reads the column and writes the
  table in order; its random accesses are one gather a run into r-entry arrays. By
  instruction count (callgrind, counts, not timing) it is **12.4 instructions a row**, against
  #34's **15.0** and production's one-hop build's **6.2**.
- **The spare-bit idea is not built. Its census:** a 3,516-byte directory against the old
  lookup's 28,125 bytes, and 0.34% of Wikidata's rows flagged, statically and dynamically.
  But the old lookup's correction loop runs only **0.002 times a step**, so the old path is
  already one load and one compare. Its prize is small.
- **Registered (draft):** Q1, the build cost ratio run / row; Q2, the whole entry point
  against production's in-place one-hop path. Each has positive and negative stops, run once
  through `sweep.yml` (`tool: pair_runs`).
- **What #34 implies:** a cheaper builder cannot make up for the consumer. #34's walk cost
  about twice a one-hop step for each pair step, and production's walk has since become the
  faster room walk. **The drafter's prior is Q1 INCONCLUSIVE and Q2 NEGATIVE STOP.**

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
  `counts`, and the counts sum to n. Otherwise the function returns None and the caller uses
  `build_pairs`. Each interval's cells cover it in rising q, so every slot is written. A store
  past a cell only goes upward, into later cells' rows.
- **The fill's raw reads and writes** are each behind a check that q + len + 8 ≤ n, computed
  without overflow. Its interval index is asserted below r, so a wrong input panics or writes
  wrong words in bounds.
- **The AVX2 fill** is an `unsafe fn` reached only through `is_x86_feature_detected!`.

**The admission (fixed before CI; see §6):** `Pairs` on the run builder samples the column's
run density, the run starts in one 8-byte word of every 16. A block above `ADMIT_DENSITY` =
0.15 runs a row goes to production's `Segmented`.

## 3. Correctness (checked)

- **`pair_runs::tests::exhaustive_small`:** every column over 3 bytes of lengths 1–8 and over
  2 bytes of lengths 9–12, 17,511 columns. At every primary index, it checks the same words,
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
1 c/B" (`2026-11-07-left-field-archive-check.md`). On counts that would need the extra 6.2
instructions a row to retire in 1 cycle. **Model: not met** unless its IPC is above 6. M1
times it.

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

- **(a), the build alone,** on the segmented blocks, each arm in its own buffers:
  - `row`: `build_pairs` + `pair_start`;
  - `run`: `build_pairs_runs`, giving the same table, LF rows and first row;
  - `one`: production's `bwt::build`, a reference only.
- **(b), the whole entry point,** `InverseBwt::invert_in_place` over a fresh copy of the
  column (the copy untimed), then RLE1 and the CRC, untimed and checked:
  - `P`: production's `Segmented` (`build_one_pass`, then the 32-lane room walk), the real
    in-place path;
  - `R`: the run-built pair table and #34's consumer, with the admission;
  - `O`: #34 as it was (row builder, same consumer), a reference.

The files: Wikidata (the corpus `bench.yml` uses), then the test corpora's synthetic inputs at
level 9.

**On CI.** Dispatch `sweep.yml` with `tool: pair_runs` and `rounds: 15`, at the frozen head,
once.
- It runs one copy pinned to CPU 1, then one copy per CPU at once.
- Each copy's exit status goes in `status-*.txt`; its raw paired timings in `raw-*.csv`.
- **The `Validity` step writes `validity.txt`.** It fails the job unless all of these hold:
  - the corpus's SHA-256 is `bench/wikidata.json`'s;
  - every copy exited 0;
  - every copy printed its VALID line;
  - every copy left non-empty raw timings.
- **A copy's VALID line needs:**
  - every registered interval present and finite;
  - every arm's every round total finite and positive;
  - the head, 40 hex digits;
  - the corpus's size and decoded size.

  A copy that is not VALID exits 3.

## 6. The registration (DRAFT, NOT FROZEN)

Judged on the **pinned single copy, on Wikidata**. The four-copy copies are exploratory. The
readings are printed mechanically, labelled UNJUDGED until the coordinator names the
registered run.

**Q1: does building from run intersections remove the old build's cost?**
R_build = run / row, from (a).
- **POSITIVE** if the geometric mean is ≤ 0.50 and its upper bound < 0.60. Most of #34's
  build penalty is removed.
- **NEGATIVE STOP** if its lower bound > 0.80. The construction saves under 20%, and the
  run-built pair table is closed.
- **INCONCLUSIVE** otherwise. No re-run without a new registration.

**Q2: does the run-built pair path beat production?**
R_whole = R / P, from (b).
- **POSITIVE** if all three hold:
  - the geometric mean is ≤ 0.95;
  - its upper bound is < 1.0;
  - the worst synthetic corpus's geometric mean is ≤ 1.05.

  A production prototype may then be registered, for `bench.yml`'s paired A/B to decide.
- **NEGATIVE STOP** if the lower bound is > 1.0. The run-built pair with #34's consumer is
  slower than production. The line reopens only with a different consumer (for instance the
  spare-bit one), registered on its own.
- **STOP (synthetic)** if Wikidata passes but a synthetic corpus regresses by more than 5%.
- **INCONCLUSIVE** otherwise. No re-run without a new registration.

**M1 (exploratory): the archive's reopening bound, timed.** The run build less the one-hop
build, in c/B of the segmented rows, as a paired mean and 95% interval.
- **MET** if the upper bound is ≤ 1 c/B.
- **NOT MET** if the lower bound is > 1 c/B.

O/P and R/O are printed as references and are not gates.

## 7. What #34 implies, and what is expected

- **#34 lost on both halves.** It was closed on the build's 3× cost and on a walk with no
  per-byte saving: half the steps, each about twice the work. Its loss was +40.6% at one
  thread on Wikidata, and the build accounted for about 5 c/B of it.
- **What this line can fix:** the run-intersection build removes the bigram histogram and the
  per-row scatter (Q1 measures what that saves).
- **What it cannot fix:** the consumer. Since #34, production's walk has become the room walk.
  It is the flagged walk in passes with a shared offset (#37: Wikidata −10.3% at one thread on
  CI, walk stage 4.37 → 3.81 c/B on the Xeon runner), then #157 and #171. #34's pair walk is
  the flagged walk #34 was built on, two bytes a step, plus the first-byte lookup.
- **So even a free build** leaves R/P near #34's walk against today's room walk. The binding
  census (`2026-11-05-binding-census/nogo.md`) found #34's port at least 10% slower at four
  threads, with about 5 c/B of build.
- **Expectation, plainly:**
  - **Q1: INCONCLUSIVE, perhaps POSITIVE on very repetitive data,** but not at 0.50 on
    Wikidata. The run builder's instructions are 0.83 of #34's. Its cycles should fall
    further, since #34's are partly random RMW misses, but not to half.
  - **Q2: NEGATIVE STOP.** The pair consumer, not the build, is the larger cost, and nothing
    here changes it.

  A Q2 POSITIVE would contradict #34's walk measurements and the binding census, and would
  need a mechanism found before any adoption.

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
    records and random. Without it, Q2's synthetic condition could not pass on any CI
    result.
- **The coordinator should decide at freeze** whether the admission belongs in the
  registered candidate. It is the shape any production use would need, as #269's fallback
  is. The constant was chosen from the census (Wikidata 0.07, text 0.25, records and random
  about 1), not from a timing of it.
- **The builder was revised after local profiling** (x-order fill → q-order fill; SWAR run
  scan; AVX2 fill). Each version passed the same word-identity checks. Its revision is part
  of building a fair candidate, not a fitted threshold.
- **No CI timing has been run.**

## 9. Not done

- The spare-bit consumer (§4: census only).
- Any change to #34's walk, or a port of the room walk's protocol to pairs. The REVIEW notes
  that this needs rephased segment starts, two-row marker tests, odd tails and periodic orbits.
- Admission tuning, multi-threaded decode, and the workstation.
