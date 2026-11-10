# The run-record front in production: no column, and the LF table filled in arrival order (registration)

> **Status (added when #324 closed): closed, failed under its frozen rule.**
> - **What failed:** the benefit rule was met in all three runs: `wikidata` −12.8% to −15.8%
>   and `text` −23.3% to −26.2% wall at 1 thread. The no-harm rule failed: `random` and
>   `records` at 1 thread were flagged slower in runs 2 and 3. See §6.
> - **The code is not on master.** It is the head of #324,
>   `e88c063155c3e734e748e99ccda025393302720f`, kept on branch `perf/run-front-records`.
> - **This construction is closed. Its advantage is not disproved.** It was measured large on
>   its target data, and the harm appears on blocks where the new code doesn't run.
> - **Below:** the registration, frozen when #324 opened (§1–§5), and the result (§6).

10 November 2026. A production-path change on the run-fed line (PROTOCOL §11). It comes from
two of the Pathfinder's leads on #313:
- the run-record emitter, priced in REVIEW 6755498073 ("For the Optimiser");
- the backward table, design A of `docs/journal/2026-11-10-backward-table.md` (#313,
  `0926a832`), which the Coordinator asked to be priced against the sort-and-sweep fill.

This entry follows STEWARDSHIP's order:
1. the price, from figures already on master (§2);
2. the change (§1), its equivalence and B5 evidence (§3), and its instruction counts (§4);
3. the rule, frozen before the pull request opens (§5);
4. the result, recorded after the runs (§6).

**This rule is frozen:** it is not amended after the pull request is opened. A result that
fails it is recorded here, never re-read.

## 1. What changes

The records' path is taken by a block of at most one symbol per 2 rows of its bound, below
2^24 rows (`runs::takes_runs`), under a kernel that can take it (`InverseBwt::reads_runs`,
true only for the default `bwt::Segmented`). Every Wikidata block (0.05–0.22 symbols a row)
and every `text` block (0.47) qualifies; `records` and `random` (1.0) don't. The cut was set
from instruction counts before any timing (§4), and widened from #269's one-in-8 at the
Coordinator's request (the Pathfinder's text finding, #145 6756520552), before this
registration froze. There, `decode_block` takes the records' path:

| | today (the column's path) | this change (the records' path) |
|---|---|---|
| phase 2 | `mtf::mtf`: the n-byte last column, written | `runs::front`: the runs of L, `byte << 8 \| len` cut at 127 rows, 2 bytes a run; no column |
| phase 3, the table | `bwt::build`: T = L \| Ψ << 8, the column read back and scattered into 256 buckets | `runs::fill`: P = L \| LF << 8, one counter a byte, the records written in arrival order, 8 entries a store |
| phase 4, the walk | the room walk from Ψ(orig_ptr), segments joined forward | the room walk's own code (`room_segments::<32, PF_L1, true>`) from orig_ptr, segments joined reversed (`assemble_reversed`) |
| RLE1, CRC | unchanged | unchanged |

**Every other block and kernel is unchanged.** That covers:
- `Cooperative<k>`, the teams' kernel, which reads checkpoints;
- `Plain`;
- the pipelined plan, and the parallel decoder's own staging.

They all take the column's path exactly as before. CI's 4-CPU runners plan `Workers(n)` with
`Segmented` at every thread count, so every bench row runs `decode_block` and sees the change.

- **The front** is the Pathfinder's `runs_from_symbols` (`examples/run_front.rs`, #313),
  ported verbatim but for its timers.
  - **The AVX2 kernel:** two AVX2 bitmaps of the symbols (not RUNA/RUNB; not RUNA), then one
    step of production's two-level MTF list a literal, with the count after it read from the
    bitmap as one mask and shift.
  - **Its scalar twin:** `mtf::mtf_scalar`'s loop, with a record in place of the column.
  - **Runtime dispatch:** `Kernels::avx2` (AVX2 and BMI1) and BMI2, LZCNT and POPCNT
    detected.
- **The fill.** In the LF direction a run of L at rows p..p+ℓ, of byte c with k copies of c
  before it, has LF(p+j) = C(c)+k+j. So its entries are consecutive values at consecutive
  slots: `next[c]` gives the first, and then `next[c] += ℓ`.
  - **The checks** take the place of `build_one_pass`'s per-write bucket check and end check:
    the counts sum to n; every record is at least a row; no record passes the block's end or
    its byte's bucket's end; and the lengths reach exactly n.
  - **What they guarantee:** every slot of [0, n) is written once, in order, and every link is
    below n. That is the T-vector invariant the walks' unchecked reads rely on.
  - **On failure** (a bug in the front, never seen) the table is left empty, and
    `decode_block` takes the column's path, after a debug assertion.
- **The walk.**
  - **Following LF from orig_ptr** gives the block's bytes last to first. It is the same
    `room_segments`, monomorphised with `BACK = true`, which only changes its first row.
  - **The forward walk is not touched:** its instance is `BACK = false`, its code the same.
  - **The reversed join** writes each segment reversed from the output's end.
  - **Exactness on any table:** a forward walk gives n bytes of Ψ from orig_ptr's cycle. When
    that cycle's length m doesn't divide n (impossible in a real BWT; possible in a corrupt
    block), the backward walk's n bytes would differ. So the join stops where the chain comes
    back to its first segment, and `repeat_cycle` makes the forward walk's bytes from the m
    walked. A corrupt block then fails with the same error, the same `computed` CRC included.
    The tests check this on made-up tables.

## 2. The price, before building

All figures are nominal c/B on Wikidata, from registered CI runs already on master, except
the two local ones, which are labelled.

**The inputs:**

| quantity | value | source |
|---|--:|---|
| MTF (`mtf::mtf`), accepted blocks | 1.290 (Intel 6973P-C), 1.085 (AMD 9V74) | #269, #277 phase tables |
| the old build, accepted blocks | 1.432, 1.357 | the same |
| #269's sort, and its F-order sweep (fill) | 0.317 and 1.332; 0.319 and 1.010 | the same |
| master's build after #312, against the old | ×0.913 (Xeon 8573C), ×0.716 (EPYC 7763) | #312's stage rows, runs 2 and 1 |
| the new front against `mtf::mtf` | ×0.409 (0.173 against 0.423 ns a row) | the Pathfinder, **local**, one host |
| design A's LF fill against `bwt::build` | ×0.64 (0.293 against 0.458 ns a row) | the Pathfinder's `lf_fill`, **local** |
| accepted blocks' share of the stage | 0.74 | #269: (1.290 + 1.432) / 3.676 |
| MTF + build in the whole 1-thread decode | 3.37 of 10.83 c/B | #312 run 2's stage table |

**The model:** on an accepted block, today's stage is M + B_m (MTF plus master's build). Two
candidates were priced:
- **S, sort-and-sweep:** ρM + sort + sweep, with ρ the front's cost against MTF's.
- **A, the LF table:** ρM + φB_m, with φ the LF fill's cost against the build's. The walk
  is assumed unchanged in cost: the same code on a table of the same size and kind.

| | Intel model | AMD model |
|---|--:|--:|
| today: M + B_m | 1.290 + 1.307 = 2.597 | 1.085 + 0.972 = 2.057 |
| S: ρM + sort + sweep (ρ = 0.409) | 0.528 + 0.317 + 1.332 = 2.177 (**−16%**) | 0.444 + 0.319 + 1.010 = 1.773 (**−14%**) |
| A: ρM + φB_m (φ = 0.64) | 0.528 + 0.836 = 1.364 (**−47%**) | 0.444 + 0.622 = 1.066 (**−48%**) |
| A at φ = 1 (no fill gain at all) | 1.835 (−29%) | 1.416 (−31%) |
| ρ at break-even, S / A (φ = 0.64) | 0.73 / 1.37 | 0.67 / 1.32 |

**The choice: A.** The bound favours it by 0.7–0.8 c/B on accepted blocks over S, even if
its fill gains nothing on the build (φ = 1). It also removes S's whole F-order apparatus:
- the sort, and its 8-byte-a-run F array;
- the merge of two iterators;
- the cells' tiling proof.

What A adds is a different walk direction. Its cost on hardware is the one thing the bound
can't price from figures on master. The instruction counts (§4) show the walk's instructions
unchanged (+0.02% to +1.6%). Its loads are as random in either direction. So the risk is in
hardware behaviour the counts can't see, and the falsifier below covers it.

**The bound says gain at the stage, and through to the decode.** Taking A's −29% to −48% on
0.74 of 3.37 c/B gives 0.72–1.19 c/B: **−7% to −11% of the 1-thread decode** on Wikidata.

**Discounts:**
- **The fill is store-bound.** #269 measured the build's stores alone at a third of its
  cost (R_floor 0.321), so φ well below 0.64 is unlikely.
- **#312 already cut the build** by 9–28% on CI's runners.
- **The prediction below is set lower than the model, at −4% to −10%,** because both
  local ratios (ρ, φ) were measured on one shared VM.

**With the wider cut** (§4), Wikidata's other 47 blocks (20% of its rows, 0.19–0.22 symbols
a row) take records too, at −10% to −12% of their instructions against −13% for the rest. So
the whole stage, not 0.74 of it, is in play. The model's figures on the whole 3.37 c/B give
0.98–1.62 c/B, or −9% to −15% of the 1-thread decode. The prediction below keeps the lower
range for the reasons above.

**Text (added before freezing, at the Coordinator's request):**
- **The Pathfinder's figures** (`docs/journal/2026-11-10-by-data-class.md`, #313 at
  2e67726b): the emitter's stage runs locally 3.0× faster than MTF on text, and the front is
  39% of a text decode. Their model gives −21% to −30% of text's decode wall.
- **The counts here** (§4): text's blocks fall 15–16% in instructions. MTF goes 13.7 M →
  7.8 M. Build and join go 10.2 M → 9.7 M fill + 0.8 M join, since text's ~0.45 records a row
  make the fill nearly as dear as the build. The walk is unchanged.
- **Text's cells:** 12 blocks, all by records, at 0.47 symbols a row of the bound, against
  the cut at 0.5. That margin is small, and a text-like block with a few more symbols would
  keep the column: its effect would be nil, not a loss.

**The other corpora:**
- **`records` and `random`** take no block by records (1.0 symbol a row). Their decode adds
  only the selector: two compares and a branch, about 60 instructions a block.
- **`runs`, `pattern` and `onebyte`** take every block by records.

### Prediction and falsifier (written before any CI result)

- **`wikidata` wall, 1 thread:** −4% to −12%.
- **`wikidata` CPU at 4 threads:** −3% to −10%.
- **`text` wall, 1 thread:** −8% to −25%. **`text` CPU at 4 threads:** −6% to −25%. (The
  low end is the instruction count, −15%, discounted for the fill's stores; the high end is
  the Pathfinder's model, which credits the front's shorter dependent chain.)
- **`pattern`:** faster at 1 thread (−15% of its block's instructions).
- **`runs` and `onebyte`:** −3% to −4% of their instructions; their rows may not resolve it.
- **`records` and `random`:** no change, since no block takes the path.
- **The stage table** (`examples/phases.rs`) times the library's `mtf::mtf`, `bwt::build` and
  forward walk directly, so it can't see this change. Its rows should not move beyond their
  usual noise.
- **Falsifier of the price:** `wikidata`'s 1-thread wall point estimate at or above −2% in
  two runs. For text, its 1-thread wall point estimate at or above −3% in two runs. That would say:
  - the backward walk costs on hardware what its instructions don't show;
  - or the front or the fill cost on CI's runners what the local figures don't show.
- **Falsifier of the mechanism on high-entropy blocks:** a `records` or `random` wall row
  flagged slower at 1 thread in two runs. Nothing they run changes but the selector.

## 3. Equivalence and safety (B5)

### The safety argument (proof scope: `src/runs.rs` and the touched parts of `src/bwt.rs`)

| unsafe | what it relies on | where it is established |
|---|---|---|
| `front_avx2` (target feature) | AVX2, BMI1, BMI2, LZCNT, POPCNT | `front_fast`: `Kernels::avx2` (AVX2, BMI1) and the other three detected |
| `front_avx2`'s bitmap writes (`lit32`, `one32`) | ci < 2·words, words = m/64 + 3 reserved | the loop bound ci·32 + 32 ≤ m, the tail ci·32 ≤ m, the zero fill to 2·words |
| its symbol loads | a set bit of `lit` is below `end` < m; the 32-symbol loads are below m | the bitmap loop's bound; `lit`'s bits are cleared past `end` |
| its 8-byte read of `one` (`count!`) | from ≤ end + 1, so bytes from/8 .. from/8 + 8 lie in `words` words | words = m/64 + 3 ≥ end/64 + 3, the last two zero |
| its record writes | one per literal (< m), plus the cuts of runs over 127 rows (≤ max_len/127 + 1, `slack`), within m + slack + 2 reserved | `long!` takes each cut from `slack` and refuses past it; `big` refuses rows past max_len |
| its second-level rows (`second`) | t < 224, `tail` 64-aligned and 224 bytes | `(s − 1).min(255) − 32` with s ≥ 33; `Rows` is `align(64)` |
| `set_len` of records | every record up to `out` written | each write advances `out` by one |
| `fill_avx2`'s stores | w < end ≤ n, so w + 8 ≤ n + 7 < n + 8 reserved | each record is refused unless q + ℓ ≤ n and ℓ ≥ 1 |
| `fill_avx2`'s `set_len(n)` | every slot below n written | records' lengths tile [0, n) in order (q == n), each record's rows written by its own stores (slack rewritten by the next) |
| the table as a `TVector` (the walks' unchecked reads) | every link below n | links are `next[c] + j < lim[c] ≤ n`, the buckets from counts that sum to n |
| `assemble_reversed`'s `set_len(n)` | every byte of [0, n) written | the join writes [at, n), with at < n since the first segment holds a byte; `repeat_cycle` copies those n − at bytes to the front and repeats them to n |
| `room_segments::<_, _, true>` | its first row below n | `assert!(orig_ptr < n)`; the rest is the room walk's existing argument (its contract comment), unchanged |

### The tests (`src/runs/tests.rs`)

- **`miri_runs_synthetic_blocks`** checks 12,000 made-up blocks, each also at three or four
  bounds around its own length (48,000 cases, 43,484 tables, at the last count). Each case
  checks both kernels:
  - `front`'s result, its length or error, equals `mtf_scalar`'s;
  - its records are the column's runs cut at 127, its row and record counts the column's;
  - both fills give the column's LF table word for word;
  - on every block below 2^16 rows, the plain backward walk and the backward room walk (1, 4
    and 32 lanes; 2, 8 and 64 segments) give `bwt::walk`'s bytes on the column's T-vector,
    and leave the table's flags clear.

  The generator is the Pathfinder's, with two shapes added:
  - **runs coded to exact lengths:** 125–129, 253–257, multiples of 127 plus 0–2, up to 2000,
    and 2^20–2^21 and 2^21−1 and up, which is 21 and 22 run symbols;
  - **literal indices to the list's end,** past 31.

  Seen in one run: 21,742 blocks decoded, 11,169 `BlockTooLong`, 9,275 `RunTooLong`, 1.6 G
  rows, 12.5 M records cut at 127, and 2.36 M literals past index 31.
- **`miri_runs_front_edges`:** runs of exactly 1, 2, 126, 127, 128, 254, 255, 256, 381 and
  1000 rows at the block's start, between literals and at its end. Literals of indices 1,
  31, 32, 33, 200 and 255. 21 run symbols together (decodes) and 22 (`RunTooLong`), at the
  start and later.
- **`miri_runs_broken_tilings_refused`:** both kernels refuse each of 12 broken inputs, and
  leave the table empty:
  - a record a row longer or shorter, of length 0, or of another byte;
  - a count one more, or moved to another byte;
  - n one more or one less;
  - a record dropped, or added (of length 0 or 1).
- **`miri_runs_decode_block_equals_the_column`:** 3,000 made-up blocks through
  `decode_block` with `Segmented` (by records where `takes_runs`) and with `Plain` (the
  column always). The same result on both kernel sets: bytes, CRC, and the error with its
  values.
- **Every corpus:** `bzcat` (this branch) against `bzip2 -dc`, the same SHA-256 on all seven
  bench corpora. `bench/wikidata.py build` decodes all 431 Wikidata blocks identically in
  libbzip2, lbzip2 and mibz2 scalar, SIMD, 3-thread and `decode_blocks`.
- **The crate's whole suite** passes (`cargo test --release`, every target): 63 library tests
  and every integration test, with the corpus decode, the error tests and the in-place tests.

### 3.1 Miri, fuzz and lints

All local runs were on this branch's final source, with nightly Miri 2026-10-26 and rustc
1.99.1.

- **Miri, the default target:** `cargo +nightly miri test --lib -- runs::tests::miri_
  bwt::tests::miri_` gives 7/7 in 322 s.
  - **What it covers:** the scalar front, the scalar fill, the backward room walk and its
    reversed join, `decode_block` by records, and the room walk's three existing tests.
  - **How the selection works:** under Miri with no target features, `Kernels::detect()`
    is scalar.
- **Miri, AVX2:** the same tests with `RUSTFLAGS="-C
  target-feature=+avx2,+bmi1,+bmi2,+lzcnt,+popcnt,+sse4.1,+ssse3,+pclmulqdq"` give 5/5 in
  293 s.
  - **`miri_runs_avx2_kernels_selected`** asserts that this build selects `front_avx2` and
    `fill_avx2`, and that the tests run both kernel sets.
  - **CI:** its Miri job (`--lib -- miri_`, the default target) now runs on changes to
    `src/runs.rs` and `src/runs/**` too.
- **The fuzz smoke** (`examples/fuzz.rs`, profile `fuzz`: debug assertions and overflow
  checks on, so a refused tiling's `debug_assert` would count as a failure):
  - classic, 300 s, seed 20261111: 503,208 cases, **0 failures**, 2,188 decoded the same as
    libbzip2;
  - classic, 300 s, seed 20261110, an earlier run: 450,684 cases, 0 failures;
  - mixed, 120 s, seed 20261112: 15,736 cases, 0 failures.

  The harness decodes with this crate's sequential decoder on both kernel sets, so every
  mutated block of few symbols took the records' path.
- **Clippy** (all targets, release) and `cargo fmt --check`: clean.
- **Not done here (B5's remaining items):**
  - **Mutation testing:** the 12 tiling breaks in `miri_runs_broken_tilings_refused` are
    hand-made mutants of the inputs, not of the code.
  - **The adversarial review** by an agent other than the implementer is the Coordinator's to
    commission.

### 3.2 Mutation testing (addendum, 10 November 2026, after the owner's adoption decision)

Added after §6, when Louis adopted #324 as a production default (#145, LOG 6764294229). It
completes the first of §3.1's two outstanding B5 items. The adversarial review remains the
Guardian's. §2, §4 and §5 are unchanged.

**The method.**
- **The tool:** cargo-mutants 27.1.0 (`--in-place`, 240 s timeout), on the branch merged with
  master at `f91f2302`. It mutated:
  - all of `src/runs.rs`;
  - in `src/bwt.rs`: `reads_runs`, `invert_runs`, `repeat_cycle`, `assemble_reversed`,
    `walk_back`, `walk_back_room`, and `room_segments`' first-row lines (the `BACK` choice);
  - in `src/lib.rs`: `decode_block` and `finish`.
- **The tests run against each mutant:** the library's tests, except
  `room_walk_crafted_permutations` (124 s, and on the forward walk only).
- **What cargo-mutants skips:** `unsafe fn`s. So `front_avx2`, `second` and `fill_avx2` had
  a hand pass of 34 mutants instead. These were bounds, the cap of 127, span limits of 21
  and 22 run symbols, the slack, the counts' halves, the shuffle tables' rows, and each of
  `fill_avx2`'s checks. Each was run against `runs::` on this AVX2/BMI2 machine, where
  `kernels()` checks both kernel sets.

**The results.**

| pass | mutants | caught | missed | timeouts | unviable |
|---|--:|--:|--:|--:|--:|
| cargo-mutants, before the new tests | 192 | 151 | 22 | 3 | 16 |
| cargo-mutants, the 22 missed, after the new tests | 22 | 12 more | 10 (below) | 0 | 0 |
| hand pass: `front_avx2`, `second` | 24 | 16 | 8 (below) | 0 | 0 |
| hand pass: `fill_avx2` | 10 | 8 | 2 (below) | 0 | 0 |

- **The three timeouts** are infinite loops in `repeat_cycle`: `have <= n`, and `have -= take`
  or `*= take`. A hang is a detection.
- **The 16 unviable mutants** did not compile.
- **A few mutants that grow a buffer without bound** were ended by the kernel's OOM killer.
  cargo-mutants counts these as caught.

**Killed by new tests (12).** In each of these mutants the bytes stay right, but the wrong
path or kernel runs, so the equivalence tests could not see them. The new tests are in
`src/runs/tests.rs`:

| mutant | killed by |
|---|---|
| `decode_block`: `!cooperative` → `cooperative` (the records' path never taken) | `runs_path_taken_exactly_where_selected` |
| `Segmented::reads_runs` → false | the same |
| `walk_back_room`: `< MIN_SEGMENTED` → `==`, `>`, `<=` (3) | `runs_walk_back_room_segments_from_its_bound` |
| `takes_runs`: `max_len < 1 << 24` → `<=` | `runs_takes_runs_bounds` |
| `Runs::buffers` → `vec![]` | `runs_buffers_listed` |
| `front_fast` → true, → false, and each `&&` → `\|\|` (5) | `runs_front_kernel_selected` |

The new tests check:
- **`runs_path_taken_exactly_where_selected`:** across 400 synthetic blocks, a sentinel
  record shows that `front` ran with `Segmented` exactly where `takes_runs` holds, and never
  with `Plain`.
- **`runs_front_kernel_selected`:** only the AVX2 front writes the bitmaps. So scalar
  kernels must leave them empty, and `Kernels::detect()` must fill them exactly when AVX2,
  BMI2, LZCNT and POPCNT are detected. Both kernels give the same records.
- **`runs_walk_back_room_segments_from_its_bound`:** at `MIN_SEGMENTED − 1` and
  `MIN_SEGMENTED` rows, it checks `walk`'s bytes, and checks whether the scratch recorded a
  room walk.

**Missed and equivalent (19).** None of these changes the output on any input:

| mutant | why it is equivalent |
|---|---|
| default `InverseBwt::invert_runs` → true, → false (2) | unreachable: `decode_block` calls it only when `reads_runs`, and every kernel that reads runs overrides it |
| `repeat_cycle`: `m == 0 \|\| m >= n` → `&&` | m ≤ n at both callers. At m = n the copy is of [0, n) onto itself and the loop doesn't run. m = 0 only when n = 0 |
| `assemble_reversed`: `while at > 0` → `>= 0` | `at` is unsigned. At 0 each further step copies nothing, and the join still stops on returning to its first segment |
| `MAX_RUN_WEIGHT`: `2 * 1024 * 1024` → `2 + 1024 * 1024` | `weight` is a power of two, and the first one at or above either bound is 2^21 |
| `close` (`front_scalar`) and `fill_scalar`: `\|` → `^` (2) | disjoint bits: a length below 128 beside `c << 8`, a byte beside `next << 8` |
| `steps`: `(nn − 16) as u8` → `(nn + 16)` | `pshufb` reads a control byte's low 4 bits and bit 7. nn ± 16 for nn in 16..32 agree in both |
| `fill_scalar`'s `len > n − q`; `fill_avx2`'s check removed (F3) (2) | implied by the bucket check: each byte's records stay within its bucket, and the buckets sum to n, so q + len ≤ n always. Removing the bucket check (F1) is caught |
| `fill_avx2`: `w >= end` → `w > end` (F5) | one more store at w = end ≤ n, inside the n + 8 reserved, rewritten by the next record or past n |
| `front_avx2`: the bitmap loop `ci·32 + 32 <= m` → `<` (A1) | a last whole 32 symbols goes to the tail loop, which takes up to 32 |
| `front_avx2`: `total > 127` → `>=` (A8) | a run of exactly 127 rows through `long!` is the same single record and count |
| `front_avx2`: `long!`'s guard `more > slack \|\| big > max_len` → `&&` (A10) | the slack guard is implied: Σ more ≤ big / 127 ≤ max_len / 127 < slack while big ≤ max_len |
| `front_avx2`: a later word's span `> 22` → `> 23` (A15) | a span of 23 is 22 run symbols, at least 2^22 − 1 rows, so `emit!`'s own span check refuses it |
| `front_avx2` refusing more often: slack per record (A9), no +1 slack (A17), final length `>=` (A16), in-word span + 1 (A20) (4) | a refusal is `None`, and `front_scalar` then decides the block. Only the time changes |

**Out of scope (1).** `decode_blocks`' `Range { end: start_bit, .. }` field deleted
(`src/lib.rs:504`) is October code, not #324's. It got past the file filter and is not
judged here.

**Two findings from the pass.**
1. **Two of the AVX2 kernels' safety guards are implied by others** (A10, F3). Neither can
   fail alone:
   - `long!`'s slack check is implied by the length check;
   - `fill_avx2`'s block-end check is implied by the bucket check.

   They stay as defence in depth. The bucket check (F1), on which `fill_avx2`'s stores rely,
   is caught by `miri_runs_broken_tilings_refused`.
2. **Every refusal mutant of `front_avx2` is masked by the scalar fallback.** A wrong `None`
   costs time, never bytes. The cases where `front_avx2` returned `Some` when it should not
   have were caught: A2–A7, A11–A14, A18, A19, A21–A24.

## 4. Instructions: whole `decode_block`, and by stage

**The method:** inclusive Ir of the driver's `measured`, one `decode_block` on a fresh
`Scratch` (`driver_main.rs` in this entry's directory: the method of
`2026-11-05-deferred-flag-walk/` and `2026-11-09-build-window-runs-v2/`). Both arms are built
the same way: rustc 1.99.1, fat LTO, `codegen-units = 1`, on this machine's AVX2 kernels.
- **master** is 3e1a7ec3 (`base-*`); **this branch** is `head-*`. The profiles are in `cg/`.
- **Every output is identical** across the two (`cg/*.stdout`: length and CRC).
- **The corpora** are the CI bench's, byte-identical: `ab gen` at scale 4, and the Wikidata
  corpus built from `bench/wikidata.json` (SHA-256 `82f65ace…`).

**The census of the cut** (`drv census`):

| corpus | blocks | by records | symbols a byte of output |
|---|--:|--:|--:|
| `wikidata` | 431 | 431 | 0.119 |
| `text` | 12 | 12 | 0.417 |
| `runs`, `pattern`, `onebyte` | 1, 22, 1 | all | 0.006, 0.0001, 0.0000 |
| `records`, `random` | 11, 9 | 0 | 1.000, 1.000 |

**Per block, by stage:**
- **front:** `mtf_avx2` against `front_avx2`;
- **fill:** `runs::fill_avx2`;
- **walk:** `room_segments`, the same code either way;
- **the rest:** the residual of build plus join (master) against the join (this branch),
  after RLE1 and the CRC, which are unchanged.

A "0" front means below callgrind_annotate's threshold: a block of a few hundred symbols.

| block | output bytes | base total | head total | change | front: MTF → emitter | fill (head) | walk: base → head | build + join (base) → join (head) |
|---|--:|--:|--:|--:|--:|--:|--:|--:|
| wikidata-b1 | 900,000 | 18,362,224 | 16,000,507 | -12.86% | 3,100,361 → 1,621,663 | 2,667,445 | 9,434,920 → 9,587,778 | 4,543,979 → 840,745 |
| wikidata-b233 | 899,995 | 18,238,504 | 15,779,759 | -13.48% | 3,055,687 → 1,595,007 | 2,647,709 | 9,412,698 → 9,414,993 | 4,488,914 → 840,928 |
| wikidata-b38 | 899,975 | 22,292,620 | 20,015,061 | -10.22% | 5,952,682 → 3,548,501 | 5,028,232 | 9,439,738 → 9,316,032 | 5,618,067 → 840,265 |
| wikidata-b41 | 899,971 | 23,909,711 | 21,156,867 | -11.51% | 6,874,441 → 3,997,451 | 5,513,090 | 9,423,741 → 9,520,256 | 6,326,435 → 841,059 |
| wikidata-b100 | 24 | 5,621 | 7,224 | +28.52% | 1,573 → 2,754 | 1,409 | 0 → 0 | 3,306 → 2,319 |
| text-b0 | 1,010,950 | 36,506,959 | 30,700,957 | -15.90% | 13,709,543 → 7,766,789 | 9,655,406 | 9,614,959 → 9,420,415 | 10,164,762 → 840,735 |
| text-b5 | 1,011,172 | 36,339,026 | 30,829,570 | -15.16% | 13,667,240 → 7,764,282 | 9,657,820 | 9,518,044 → 9,540,058 | 10,127,064 → 840,825 |
| runs-b0 | 10,000,235 | 17,691,814 | 17,152,139 | -3.05% | 3,176,665 → 2,357,668 | 1,858,503 | 2,990,074 → 2,971,226 | 1,834,972 → 274,722 |
| pattern-b0 | 899,981 | 14,790,002 | 12,580,643 | -14.94% | 902,483 → 0 | 900,615 | 9,448,501 → 9,499,319 | 3,158,359 → 900,138 |
| onebyte-b0 | 20,000,000 | 23,392,998 | 22,469,462 | -3.95% | 393,396 → 0 | 392,710 | 4,255,731 → 4,278,611 | 1,368,046 → 422,399 |
| records-b0 (column) | 899,980 | 95,404,355 | 95,404,416 | +0.00% | 71,102,362 → 71,102,362 | 0 | 9,392,629 → 9,392,629 | 14,909,364 → 14,909,425 |
| random-b0 (column) | 899,981 | 84,474,095 | 84,474,156 | +0.00% | 60,252,556 → 60,252,556 | 0 | 9,253,915 → 9,253,915 | 14,967,624 → 14,967,685 |

- **Wikidata, full blocks: −10.2% to −13.5% of the decode.**
  - **The front** is 0.52–0.60 of MTF's instructions.
  - **Fill plus join** is 3.5 M against build plus join's 4.5 M on low-symbol blocks.
    On the 0.2-symbol blocks it is 5.9–6.4 M against 5.6–6.3 M: there the fill is about level
    with the build, and the gain is the front's.
  - **The walk:** −1.3% to +1.6% on the same code, a different table and starting row.
- **`text`: −15.2% to −15.9%.** The front saves 5.9 M; the fill (9.7 M, about 9.5 a row at
  0.45 records a row) plus the join is level with the build plus join (10.1 M).
- **`pattern` −14.9%, `onebyte` −4.0%, `runs` −3.1%.**
- **`records` and `random`:** +61 instructions a block, the selector, against 84–95 M:
  +0.00007%.
- **Short blocks** (a stream's last, 24 bytes here): +1,600 instructions, the front's and
  fill's fixed set-up (the bitmaps, the 224-byte second level, 2 KiB of counts). They are one
  a stream.
- **Wider admission is not free.** Taking `records`' and `random`'s blocks too would cost
  them **+18.3% and +19.2%**: a record a row, and the fill's ~37 instructions a record against
  the build's ~16 a row. So they keep the column. Blocks between 0.5 and 1.0 symbols a row
  are not counted here, and stay on the column.
- **No branch-simulation or cache-simulation claim is made.** The walk's backward direction
  is the change counts can't price (§2).

## 5. The decision rule (frozen)

The rule is #312's (`2026-11-09-build-window-runs-v2.md`), with its inherited no-harm policy
from `2026-11-05-deferred-flag-walk.md` written out in full.

- **Correctness (required):**
  - CI green, Miri included (the runs tests and the room walk's);
  - the same bytes on every corpus in every bench run;
  - the records' tests pass (§3).
- **Benefit (required):** on `wikidata` at 1 or 4 threads, the CPU change or the wall change
  excludes zero on the faster side in at least one run. The same figure's point estimate is
  not slower in the other run (in every other run, if a third is made).
- **No harm (required), each run read on its own:**
  - a wall row (any corpus × thread count, all seven corpora) flagged **slower** is a harm
    signal;
  - one more run on the same head settles it: flagged again, it is a harm, and the rule
    fails;
  - not flagged again, the row is reported as unresolved, with all its runs;
  - at most three runs in all.

  "Not flagged" means no slowdown detected, not equivalence. A harm smaller than a row's
  half-width (±1–6% in recent runs) can't be excluded by these runs, and this PR doesn't claim
  to exclude it.
- **The repeat policy:**
  - **Run 1** is the bench run that opening the pull request starts, on this head against its
    merge base with master.
  - **Run 2** is one re-run of that same workflow run (the same head and base), started once
    run 1 has posted its table. Both runs are planned: run 2 is the benefit rule's "other
    run", and it settles any wall row flagged slower in run 1.
  - **Run 3** is made only to settle a wall row flagged slower in run 2 and not in run 1. No
    run is made for any other reason, and none after three.
  - A run whose job fails before the bench produces its table (a setup failure,
    `docs/team/CI-INCIDENTS.md` class B) is re-run as the same run; it is not a run.
  - Each run's whole table is preserved before the next run starts. The bench edits one
    comment in place.
- **The whole grid is reported** in every run, whatever it shows:
  - every corpus × thread count: wall, CPU and peak RSS;
  - the resources table and the stage table.
- **No retries to pass, and no fallback:** if the rule fails, the PR is closed and this entry
  records the negative result.
- **Not changed after the data:**
  - the corpora, thread counts and rounds;
  - the cut (`2·syms ≤ max_len`) and this rule.
- **Peak RSS:** the records' buffers per worker are 2 bytes a record (at most a symbol), two
  bitmaps of a bit a symbol, and nothing for the F order. With no column written, `ll` holds
  the output only. Expected ≤ +1 MB a worker; a flagged peak-RSS row is reported, not judged.

## 6. Result: the registered rule fails (two confirmed harms on high-entropy corpora)

Recorded after the runs; the rule above is unchanged.
- **The runs:** three, on the benched head **e88c063155c3e734e748e99ccda025393302720f**
  against its merge base 5e7b99547. They are run 42747668401, attempts 1, 2 and 3, at bench
  3.4.0 with 10 paired rounds each.
- **The runners:**
  - run 1: Intel Xeon Platinum 8370C;
  - run 2: AMD EPYC 7763;
  - run 3: AMD EPYC 9V45.
- **The tables are preserved on #324:** run 1 in comment 6758044068, run 2 in 6758194980 and
  run 3 in 6758350884. The bench comment 6757883083 holds run 3.
- **Run 3 was made under the repeat policy,** only to settle the three rows run 2 flagged and
  run 1 had not.
- **Every run gave the same bytes on every corpus.** CI was green on the head, Miri and the
  fuzz smoke included.

**The whole grid, wall change (CPU change), head over base, 95% intervals:**

| corpus | threads | run 1: wall (CPU) | run 2: wall (CPU) | run 3: wall (CPU) |
|---|--:|--:|--:|--:|
| `wikidata` | 1 | −12.8% (−13.4%, −12.2%) **faster** (−12.8% faster) | −14.5% (−15.5%, −13.6%) **faster** (−14.5% faster) | −15.8% (−17.3%, −14.2%) **faster** (−15.7% faster) |
| `wikidata` | 2 | −10.7% (−11.7%, −9.7%) **faster** (−11.1% faster) | −12.7% (−14.6%, −10.7%) **faster** (−12.9% faster) | −12.3% (−17.2%, −7.1%) **faster** (−13.0% faster) |
| `wikidata` | 4 | −7.7% (−8.4%, −7.0%) **faster** (−8.4% faster) | −9.7% (−11.2%, −8.2%) **faster** (−13.0% faster) | −10.3% (−12.5%, −8.1%) **faster** (−13.2% faster) |
| `text` | 1 | −23.3% (−24.0%, −22.5%) **faster** (−22.8% faster) | −24.5% (−25.5%, −23.4%) **faster** (−23.9% faster) | −26.2% (−28.5%, −23.9%) **faster** (−25.4% faster) |
| `text` | 2 | −17.6% (−19.7%, −15.5%) **faster** (−18.2% faster) | −18.7% (−21.3%, −16.0%) **faster** (−18.4% faster) | −19.4% (−23.8%, −14.8%) **faster** (−21.0% faster) |
| `text` | 4 | −12.4% (−14.5%, −10.2%) **faster** (−15.5% faster) | −10.7% (−13.8%, −7.5%) **faster** (−16.4% faster) | −14.1% (−16.7%, −11.5%) **faster** (−18.4% faster) |
| `pattern` | 1 | −4.9% (−6.3%, −3.6%) **faster** (−4.6% faster) | −5.2% (−6.9%, −3.5%) **faster** (−4.9% faster) | −6.0% (−6.7%, −5.3%) **faster** (−5.6% faster) |
| `pattern` | 2 | −1.3% (−4.8%, +2.3%) (−3.8% faster) | −5.3% (−10.8%, +0.5%) (−4.5%) | −8.2% (−14.9%, −0.9%) **faster** (−3.8%) |
| `pattern` | 4 | −8.7% (−11.7%, −5.7%) **faster** (−6.8% faster) | −3.0% (−7.7%, +1.8%) (−4.3%) | +0.1% (−7.0%, +7.6%) (−4.3% faster) |
| `onebyte` | 1 | +0.4% (−1.6%, +2.5%) (+0.4%) | −0.6% (−2.5%, +1.2%) (−0.4%) | +1.1% (−2.0%, +4.2%) (+1.2%) |
| `onebyte` | 2 | −0.2% (−2.1%, +1.6%) (−0.0%) | +1.6% (−2.4%, +5.8%) (+2.0%) | +0.5% (−1.2%, +2.2%) (+0.2%) |
| `onebyte` | 4 | −0.9% (−2.5%, +0.6%) (−0.5%) | +0.6% (−2.5%, +3.7%) (+0.8%) | +0.7% (−2.1%, +3.5%) (+0.6%) |
| `runs` | 1 | −1.5% (−2.9%, −0.1%) **faster** (−1.8% faster) | +0.3% (−1.4%, +2.2%) (+0.2%) | +5.3% (−12.0%, +26.1%) (+5.5%) |
| `runs` | 2 | −1.9% (−4.1%, +0.3%) (−2.3% faster) | −0.4% (−2.9%, +2.2%) (−0.5%) | +0.2% (−1.5%, +1.9%) (+0.4%) |
| `runs` | 4 | −0.3% (−4.0%, +3.5%) (−0.4%) | −0.7% (−5.2%, +4.1%) (−0.5%) | −1.3% (−4.2%, +1.7%) (−1.5%) |
| `random` | 1 | +1.3% (−1.1%, +3.8%) (+1.4%) | +3.4% (+2.2%, +4.6%) **slower** (+3.2% slower) | +5.3% (+4.5%, +6.1%) **slower** (+5.2% slower) |
| `random` | 2 | +0.1% (−3.9%, +4.3%) (+1.0%) | +2.9% (−0.0%, +5.8%) (+3.6% slower) | +0.8% (−2.1%, +3.8%) (+3.5% slower) |
| `random` | 4 | −0.5% (−4.1%, +3.3%) (+1.7%) | +1.7% (−1.1%, +4.6%) (+4.2% slower) | −1.5% (−16.9%, +16.8%) (−4.7%) |
| `records` | 1 | −0.6% (−2.7%, +1.5%) (−0.6%) | +1.5% (+0.2%, +2.9%) **slower** (+1.6% slower) | +2.1% (+1.0%, +3.3%) **slower** (+2.1% slower) |
| `records` | 2 | +4.7% (+0.2%, +9.4%) **slower** (+3.4% slower) | +1.7% (−1.3%, +4.9%) (+1.9% slower) | −2.6% (−7.2%, +2.2%) (−1.4%) |
| `records` | 4 | +1.2% (−1.2%, +3.7%) (+1.2%) | +4.0% (+1.6%, +6.5%) **slower** (+3.2% slower) | −1.2% (−6.5%, +4.4%) (+2.7%) |

Peak RSS, head over base (run 1 / run 2 / run 3):
- `wikidata`: 1 threads +1.3% more / +1.2% more / +1.1% more; 2 threads +7.1% more / +6.4% more / +5.0% more; 4 threads +3.6% more / +4.4% more / +4.1% more
- `text`: 1 threads +3.5% more / +3.5% more / +3.3% more; 2 threads +8.1% more / +7.4% more / +8.0% more; 4 threads +5.6% more / +10.4% more / +7.0% more
- `pattern`: 1 threads −0.3% / −0.0% / +0.1%; 2 threads −0.2% / +0.4% / −1.1%; 4 threads −2.1% / +2.4% / +1.3%
- `onebyte`: 1 threads +0.1% / +0.0% / +0.0%; 2 threads +0.0% / +0.2% / +0.2%; 4 threads +0.1% / −0.2% / −0.0%
- `runs`: 1 threads +0.7% more / +0.6% more / +0.6% more; 2 threads +0.9% more / +0.7% more / +0.9% more; 4 threads +0.6% more / +0.7% more / +0.6% more
- `random`: 1 threads −0.1% / +0.2% more / −0.2%; 2 threads −2.9% / +0.4% / +1.4%; 4 threads +1.0% / −1.0% / −1.9%
- `records`: 1 threads −0.1% / −0.0% / +0.1%; 2 threads −1.9% / −1.2% / −1.9%; 4 threads +0.8% / +0.1% / −0.4%

**The stage table** (`examples/phases.rs`, which can't see this change: §2): MTF + RLE2 ran
−3.0%, −4.4% and −7.7%, and the build +1.0%, +0.0% and +0.0%. The walk was not flagged.

**Resources (`wikidata`):** each run reads "a trade-off: better speed, CPU cost; worse
contention sensitivity, memory" (run 3: "worse memory").
- **Amplification:** +1.9% at 2 threads (runs 1 and 2) and +5.0% at 4 threads (run 1),
  beyond noise. Run 3's is not flagged.

### Against the rule
- **Correctness:** met.
- **Benefit:** met, in every run.
  - **`wikidata` at 1 thread:** wall −12.8%, −14.5% and −15.8%, CPU likewise.
  - **`wikidata` CPU at 4 threads:** −8.4%, −13.0% and −13.2%.
- **No harm: fails.**
  - **`random` at 1 thread:** flagged slower in run 2 (+3.4%) and again in run 3 (+5.3%).
    **A confirmed harm.**
  - **`records` at 1 thread:** flagged in run 2 (+1.5%) and again in run 3 (+2.1%).
    **A confirmed harm.**
  - **`records` at 2 threads:** flagged in run 1 (+4.7%), not in run 2. Unresolved.
  - **`records` at 4 threads:** flagged in run 2 (+4.0%), not in run 3. Unresolved.
- **Verdict:** the registered rule **fails**. Under §5 the PR is closed, unmerged (the
  Coordinator closed #324), and this head is not re-benched.

### Against the predictions
- **`wikidata` wall at 1 thread:** predicted −4% to −12%; measured −12.8%, −14.5% and
  −15.8%, above the range. **CPU at 4 threads:** predicted −3% to −10%; measured −8.4%,
  −13.0% and −13.2%.
- **`text` wall at 1 thread:** predicted −8% to −25%; measured −23.3%, −24.5% and −26.2%. That
  is the Pathfinder's model's range (−21% to −30%), not the instruction count's (−15%).
- **`pattern`:** faster at 1 thread in every run (−4.9% to −6.0%), as predicted.
  **`runs` and `onebyte`:** unresolved, as predicted.
- **`records` and `random`: the prediction failed.** "No change" was predicted, since no
  block of theirs takes the path and their decode adds 61 instructions a block (+0.00007%).
  They ran +1.5% to +5.3% slower at 1 thread.
  - **The mechanism falsifier fired:** a `records` or `random` 1-thread wall row flagged slower
    in two runs.
  - **What changed for them:** not the work. Their blocks run the column's path, the same
    `mtf_avx2` and `bwt::build`, so the change must be in where things sit: the code's layout,
    and the data's.
  - **The data's layout:** `Scratch` (on each worker's stack, in the decoder and the bench)
    grew by `runs::Runs`, about 2.1 KiB inline: two `[u32; 256]` and four `Vec`s. That moves
    `counts` against `mtf_avx2`'s stack frame.
  - **That matches the pattern #277's codegen entry found**
    (`2026-11-09-run-fed-fallback-codegen.md`). There, a fallback path ran 6–37% slower with
    identical kernels where `counts`, read-modify-written once a literal, fell under
    `mtf_avx2`'s frame modulo 4 KiB.
  - **Why only these corpora:** high-entropy blocks have a literal a row, so they are the
    most sensitive to it, as here. On Wikidata and text, every block now takes the records'
    path and never runs `mtf_avx2`.
  - **This is a hypothesis, not a finding:** no listing or address probe was made for this
    head.
- **The walk's direction cost nothing visible.** On Wikidata, the wall gain matches the
  model's high end. The stage table's walk is the forward walk's, so it can't show this.

### What it shows, and the next experiment
1. **The run-record front and the backward LF table work on the target data.** Wikidata
   (−13% to −16%) and text (−23% to −26%) gain across three CPU families, with the walk
   reversed. This is the largest single-thread gain on the run-fed line, and it closes the
   line's named gaps:
   - the emitter's cost;
   - the sort;
   - the fallback's cost on blocks that take records.
2. **The harm is on blocks that run none of the new code.** A possible successor is the same
   change with `Runs` out of `Scratch`'s inline layout (boxed, so `Scratch` grows by one
   pointer), and the column's path's stack and data positions as master's.
   - **Its admissibility under C2** is the Guardian's ruling (#161, 6758376066).
   - **If admitted, it needs:**
     - its own registration, frozen before data;
     - a static check first: `counts`' offset from `mtf_avx2`'s frame modulo 4 KiB, in each
       build, read as #277's entry read it.
3. **The extra peak RSS** (+4–10% on Wikidata and text at 2–4 threads, about 3 MB) is the
   records' buffers and the walk's output in `ll`, both kept a worker. It is reported, not
   judged.
