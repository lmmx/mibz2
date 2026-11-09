# 2026-11-09: B5 evidence for the run-fed stage (#269, Guardian D044)

The evidence the Guardian's D044 HOLD names as missing for #269, gathered at the frozen
head and without changing it. D044 (inbox comment 6729932417; decision
`team-state/guardian@24bf0264:state/decisions.md`, section D044) asked for:

1. Miri on the new emitter's scalar and AVX2 record stores, with reachability and every exit;
2. reproducible fuzz and differential results through emit → sort → fill and the fallback;
3. the mutation-kill and negative-state evidence kept, with exact mutants and results.

No timing was run or needed.

## Source pin

- **Code under test:** `research/run-fed-stage` at
  `fd60fffada25d475bbcc2b54165b96cf5fd7a92f`. `src/`, `examples/`, the Cargo files and
  README are byte-identical to that commit on this branch
  (`git diff --quiet fd60fffa -- src examples Cargo.toml Cargo.lock README.md`). Blobs:
  `src/mtf.rs` 2610fa7e, `src/run_stage.rs` 07a540a5, the same ones D044 reviewed.
- **Added here:**
  - `tests/run_fed_b5.rs` (blob 99ad4e78), compiled only with `--features run-fed`;
  - `miri.yml`: three steps (emitter Miri scalar, emitter Miri AVX2, nightly doc-test error
    codes), and `src/mtf.rs` and the new test added to its paths;
  - `test.yml`: the new test, in debug and release;
  - this note.
- **Toolchains:** stable cargo 1.97.0; nightly cargo 1.101.0-nightly (2026-10-23), miri
  0.1.0 (16d1ffac17 2026-10-26). The host has AVX2 and BMI1.

Everything here lives outside `src`. No test needed private access: `mtf::mtf_runs`,
`run_stage::{sort, fill, fill_rows, build, takes_runs, Scratch}` and `Symbols`' fields are
public under the feature.

## How a case is checked (`one` in the test)

Each `Symbols` value runs on every kernel: scalar, and AVX2 where `Kernels::detect()` has
it. A run does three things:

1. **Production:** `mtf::mtf` (same kernel) → `bwt::build`, into dirty buffers.
2. **The emitter alone:** `mtf::mtf_runs` into a dirty buffer.
3. **The stage:** `run_stage::build`, with its own dirty buffers.

Each part runs under `catch_unwind`. Asserted:

- **Panics:** production and emitter panic together or not at all, and the stage panics with
  them.
- **Errors:** an error is the same kind from `mtf_runs`, `mtf` and `build`.
- **The record bound:** `recs.len() <= syms.len() + 1` on every exit, errors included. The
  raw stores rely on it.
- **On success:**
  - result and counts equal production's;
  - `sort` is tried directly. If it accepts, `fill` on both kernels and `fill_rows` in 1–3
    shares each equal production's table word for word;
  - if it refuses, the records must not be the column's maximal runs. The test also asserts
    the one malformation seen: a leading run with no end of block, `[front, 0]` for n = 0.
    The only other allowed refusal is n ≥ 2^24.
- **`build`:**
  - it returns the production length and table;
  - its path is the records' exactly when `takes_runs` is true and `sort` accepts;
  - it may panic only through its `debug_assert` on refused records, and only in a debug
    build.

Each kernel run is classified:

| Class | What happened |
|---|---|
| **accepted** | `build` took the records' path |
| **fallback** | `takes_runs` was false |
| **refused** | `takes_runs` was true and `sort` refused: the debug_assert panic in debug, the column's path in release |
| **error** | RunTooLong or BlockTooLong |
| **panicked** | Symbols outside the scanner's contract |

Two more counters are kept:

- **non_maximal:** records `sort` accepted that are not maximal runs (the right table even
  so);
- **direct_fills** and **avx2_fills.**

## 1. Emitter Miri, scalar and AVX2

**Commands**, from the worktree, `CARGO_TARGET_DIR` private:

```
cargo +nightly miri test --features run-fed --test run_fed_b5 -- miri_ --nocapture
RUSTFLAGS="-C target-feature=+avx2,+bmi1" MIBZ2_EXPECT_AVX2=1 \
  MIRIFLAGS="-Zmiri-env-forward=MIBZ2_EXPECT_AVX2" \
  cargo +nightly miri test --features run-fed --test run_fed_b5 -- miri_ --nocapture
```

These are `miri.yml`'s two new steps. In both, every case also goes through `sort`, `fill`
(both kernels where AVX2 is enabled), `fill_rows` and `build`. Miri therefore checked the
AVX2 fill again on records the emitter wrote, not only on hand-made ones.

**The tests:**

- **`miri_emitter_exits`** covers every exit and limit:
  - the empty input, the end of block alone, a literal then the end;
  - a leading run, then the end, or then literals and runs;
  - alternating literals, and symbols after the end;
  - no end of block after a run, after a literal, and after a leading run alone (the
    refused records);
  - each limit met exactly and one short, at a literal and at a run;
  - a leading run past the limit, and a limit of 0;
  - RunTooLong after a literal and in a leading run;
  - a symbol above the end but inside the list;
  - an end of 0 and of 1 (never met);
  - a symbol past the list (300, and u16::MAX): the scalar kernel panics in `mtf` as in
    `mtf_runs`, and the AVX2 one clamps in both;
  - a byte twice in the map (records that are not maximal runs, accepted, the right
    table);
  - all 256 bytes, each index up and down, then the second level's row edges;
  - the longest run a weight allows (21 RUNB, 4,194,302 rows), led or after a literal, at
    its limit (accepted, records checked against the analytic ones, sorted) and one past
    (BlockTooLong). These run the emitter and `sort` only: a 4M-row fill is too large to
    interpret.
- **`miri_emitter_kept_buffers`:** one record buffer kept across seven blocks, starting from
  no capacity. It goes through a larger block then smaller ones, and an error then a
  success.
- **`miri_emitter_seeded`:** seeds 1, 2 and 3, 14 streams each of the fuzz's seven kinds,
  short.

**Results**, as kernel runs per test (`Paths` printed by each test):

| Step | Test | Runs (AVX2) | Accepted | Fallback | Refused | RunTooLong | BlockTooLong | Panicked | Non-maximal | AVX2 fills | Result |
|---|---|---|---|---|---|---|---|---|---|---|---|
| scalar | exits | 31 (0) | 13 | 6 | 1 | 2 | 3 | 2 | 1 | 0 | ok |
| scalar | kept buffers | 7 (0) | 3 | 2 | 0 | 1 | 1 | 0 | 0 | 0 | ok |
| scalar | seeded | 42 (0) | 22 | 7 | 0 | 0 | 13 | 0 | 0 | 0 | ok |
| AVX2 | exits | 62 (31) | 28 | 12 | 2 | 4 | 6 | 2 | 2 | 53 | ok |
| AVX2 | kept buffers | 14 (7) | 6 | 4 | 0 | 2 | 2 | 0 | 0 | 13 | ok |
| AVX2 | seeded | 84 (42) | 44 | 14 | 0 | 0 | 26 | 0 | 0 | 80 | ok |

Miri reports 3 passed in both steps, with no undefined behaviour. Its clocks are
deterministic: 74.9 s scalar and 132.8 s AVX2 by Miri's clock, 46 s and 87 s wall. In the
AVX2 step the scalar runs are counted too, which is why the totals are twice the AVX2 ones.

**Reachability:**

- **AVX2 detected.** With `MIBZ2_EXPECT_AVX2` set, `kernels()` asserts that AVX2 was detected
  and is among the kernels run. That is the same pattern as `run_stage::tests`. The negative
  control fails as it must: the variable forwarded without the target features gives
  "AVX2 expected, and not detected" (1 failed).
- **The AVX2 kernel ran:** 80 AVX2 emitter runs in the AVX2 step, plus 4 longest-run runs.
- **The record stores were reached.** `mtf_runs_avx2` with `reserve(syms.len())`, one short,
  is mutant M3 below. Miri stopped it at the closing record's store (`src/mtf.rs:461`,
  `*out.add(m) = (n as u32) << 8`): "attempting to access 4 bytes, but got alloc…+0xc which
  is at or beyond the end of the allocation of size 12 bytes". The case was `build` → `mtf_runs`
  in `miri_emitter_exits`. The native suites pass that mutant (the existing ones) or abort
  (this one, through `set_len`'s debug precondition).
- **Exits reached** (by the classes above), each on both kernels in the AVX2 step:
  - the normal end, and the end missing;
  - RunTooLong and BlockTooLong, at both of BlockTooLong's checks;
  - the empty input, a leading run, the limits, and the longest runs.
- **What this does not cover:**
  - the unsupported-CPU case: `Kernels { avx2: true }` forged on a CPU without AVX2, the
    pre-existing forgeable-`Kernels` issue, out of scope below;
  - a full 2^24-row fill under Miri (too large; natively in §2).

`miri.yml`'s existing passes stay, and were re-run here on the restored code:
`cargo miri test --lib -- run_stage::tests::miri_` with the AVX2 flags gives 2 passed
(`miri_sort_and_floor`, `miri_unchecked_states_refused`).

## 2. Seeded fuzz and differential: emit → sort → fill → fallback

**`fuzz_emit_sort_fill_fallback`:**

- **Seeds:** `0xb50001, 0xb50002, 0x5eedd044, 0x0269fd60` (fixed).
- **Cases:** 2048 per seed, on both kernels, so 8,192 cases and 16,384 kernel runs.
- **Stream size:** the longest stream doubles every 64 cases, from 64 symbols to 4096.

Seven kinds, taken in turn:

| Kind | Streams |
|---|---|
| 0, run-heavy | mostly RUNA/RUNB runs up to 4096 rows |
| 1, alternating | index 1 each time (two bytes by turns), with or without runs between |
| 2, single-symbol | one byte in use, one run |
| 3, max-alphabet | 256 in use: uniform, far and row-edge indices, runs |
| 4, mixed | a random run share |
| 5, sparse literals | long runs between literals |
| 6, corrupt | one of the above broken, below |

Kind 6 breaks a stream in one of ten ways:

- the end removed, or moved early;
- a symbol above the end;
- a symbol past the list (257 to 65535);
- a weight past its bound;
- an end of 0 or 1;
- a symbol replaced at random;
- the limit 0;
- a byte twice in the map;
- a leading run with no end.

Each stream's limit is set around its length: n, n − 1, n + 1, n − k, 900,000, or around
`takes_runs`' 8·syms boundary. Every check therefore refuses somewhere, and both sides of
`takes_runs` are reached.

**Asserted reached:** accepted, fallback, refused, RunTooLong, BlockTooLong, direct fills,
non-maximal records and panics; AVX2 emitter and AVX2 fills whenever AVX2 is detected.

**Fixed seeds (debug):** `cargo test --features run-fed --test run_fed_b5 -- --nocapture`

| Seed | Kernel runs | Accepted | Fallback | Refused | RunTooLong | BlockTooLong | Panicked | Non-maximal | Direct fills | AVX2 fills |
|---|---|---|---|---|---|---|---|---|---|---|
| 0xb50001 | 4096 | 1342 | 935 | 68 | 498 | 1235 | 18 | 0 | 2275 | 2949 |
| 0xb50002 | 4096 | 1426 | 998 | 46 | 540 | 1066 | 20 | 10 | 2422 | 3138 |
| 0x5eedd044 | 4096 | 1387 | 939 | 46 | 526 | 1175 | 23 | 0 | 2326 | 3024 |
| 0x0269fd60 | 4096 | 1352 | 1006 | 54 | 497 | 1153 | 34 | 0 | 2358 | 3040 |
| total | 16384 | 5507 | 3878 | 214 | 2061 | 4629 | 95 | 10 | 9381 | 12151 |

Half of the kernel runs are AVX2 (8,192). Other error kinds: 0. All five tests passed, in
3.5 s.

In debug, "refused" is `build`'s debug_assert panic, as the test expects. In release
(`cargo test --release …`, also in `test.yml`) the same 214 are `build`'s column path, with
the production table. Both builds pass.

**Bounded run (release):**
`MIBZ2_B5_FUZZ_SECS=300 cargo test --release --features run-fed --test run_fed_b5 fuzz -- --nocapture`.
It ran the 4 fixed seeds, then 471 further seeds (`0x1000..0x11d7`), 2048 cases each, for
300.5 s:

- **Volume:** 972,800 cases, 1,945,600 kernel runs, 972,800 of them AVX2.
- **Paths:**
  - 655,778 accepted, 457,727 fallback, 27,902 refused;
  - 238,086 RunTooLong, 553,516 BlockTooLong, 0 other errors;
  - 12,591 panicked;
  - 1,994 non-maximal;
  - 1,113,421 direct fills and 1,443,093 AVX2 fills.
- **Result:** passed.

**`representation_limit`** (native):

- **Under 2^24:** four longest runs and three literals, n = 16,777,211 = 2^24 − 5,
  `max_len` 2^24 − 1. `takes_runs` is true, both kernels are accepted, and the production
  table matches.
- **Over 2^24:** five longest runs, n = 20,971,514, `max_len` 2^25. `mtf_runs` succeeds, but
  `sort` refuses the records fed to it directly (n ≥ 2^24). `takes_runs` is false, so
  `build` takes the column's path, with the production table.

**Findings:**

- **No differential mismatch, and no accepted wrong table, in any run.**
- **Malformed `Symbols` and the scalar/AVX2 difference.** The Guardian asked for this
  explicitly. For a symbol past the list (≥ 257, not the end), the scalar `mtf` and
  `mtf_runs` both panic on the list index; the AVX2 `mtf` and `mtf_runs` both clamp, to the
  same result. The two kernels do differ on such out-of-contract input. The emitter matches
  the production kernel it replaces on each, and the stage panics exactly when production
  does. In-contract input (the scanner's symbols end with `eob ≤ 257`) never reaches it.
- **The one refused malformation.** A hand-made `Symbols` that begins with a run and has no
  end of block leaves its run unclosed. Both kernels then write `[front, 0]` with n = 0, a
  zero-length run that `sort` refuses. If `takes_runs` holds, `build` trips its
  `debug_assert` in a debug build; a release build takes the column's path with the right
  (empty) table. That is the documented behaviour, not unsoundness: the records are never
  read unchecked. The scanner's symbols always end with the end of block, so it is outside
  the contract. It is the only refusal seen across the million cases besides n ≥ 2^24, and
  the test asserts that.
- **The map.** A byte twice in `seq_to_unseq` (outside the scanner's contract) gives records
  that are not maximal runs. `sort` accepts them, as a tiling, and the table is still the
  production one.

## 3. The existing evidence, kept and re-run

### Mutation kills

Each mutant was applied alone with `sed` to the frozen source, then restored with
`git checkout -- src`. Suites:

- **KT:** `cargo test --features run-fed --test mtf_kernels`;
- **RS:** `cargo test --features run-fed --lib run_stage`;
- **B5:** `cargo test --features run-fed --test run_fed_b5`;
- **Miri:** B5's `miri_emitter_exits`, AVX2 step.

| Mutant | Change | KT | RS | B5 | Miri | First failing checks |
|---|---|---|---|---|---|---|
| M1 (the journal's) | AVX2 emitter, `src/mtf.rs:456`: `(n as u32) << 8 \| c` → `(n as u32 + (n == 40) as u32) << 8 \| c` | caught, 3 of 4 (`corrupt_runs_and_limits` passes) | passes (doesn't call the emitter) | caught, 4 of 5 (`representation_limit` passes) | not run | KT "index 1 repeated, max 900000, avx2: the records"; B5 "seed 0xb50001 case 1 kind 1, max 407, avx2: refused records [1, 597, 1281, …]" (`sort` refused, and they are no permitted malformation) |
| M1s (added) | the same in the scalar emitter, `recs.push((n as u32 + (n == 40) as u32) << 8 \| c)` | caught, 3 of 4 | passes | caught, 4 of 5 | not run | B5 "miri seed 1 case 3, max 338, scalar: refused records […]" |
| M2 (the journal's) | AVX2 fill, `src/run_stage.rs:252`: step `2048` → `2304` | caught, 4 of 4 | caught, 2 of 4 (`tables_from_records_random`, `miri_sort_and_floor`) | caught, 4 of 5 (`miri_emitter_exits` passes: none of its cells exceeds 8 rows) | not run | RS "n 9, avx2: the table"; KT "random 1, max 900000, avx2: the records' table"; B5 "seed 0xb50001 case 4 kind 4, max 7490, scalar: the records' table, fill avx2" |
| M3 (added) | AVX2 emitter capacity, `src/mtf.rs:388`: `reserve(syms.len() + 1)` → `reserve(syms.len())` | **passes**, 4 of 4 | passes | aborts (SIGABRT: `set_len`'s debug precondition) | **caught: UB** | Miri: out-of-bounds 4-byte store at `src/mtf.rs:461` (the closing record), allocation of 12 bytes |
| M4 (added) | AVX2 emitter length, `src/mtf.rs:465`: `set_len(m)` → `set_len(m + 1)` | not run | not run | not run | caught | "the empty input, max 100, avx2: 2 records for 0 symbols" (the record bound) |

**Restored baseline** (`src` byte-identical to fd60fffa, confirmed by `git diff --quiet`):

| Suite | Result |
|---|---|
| KT | 4 passed |
| RS | 4 passed |
| B5 | 5 passed |
| doc tests, stable and nightly | 5 passed |
| Miri AVX2, B5 `miri_` | 3 passed |
| Miri AVX2, `--lib run_stage::tests::miri_` | 2 passed |

M3 is why the emitter needed Miri. The existing native differential suite does not see a
capacity one short: the heap's slack absorbs the store. Only Miri, or a debug
precondition, does.

### Negative-state tests

`run_stage::tests::miri_unchecked_states_refused` passes:

- natively (RS, above);
- under Miri with AVX2 (re-run here).

The states it refuses:

- no records, for n 0 and 1, and with a count;
- the closing record alone;
- the empty block's records with n = 1;
- another block's n (0, 1, 3, 5, 2^24);
- records out of order;
- a first run past row 0;
- two runs at one row;
- an uncounted byte;
- a scratch re-sorted for the next block's counts (left empty);
- `fill_rows` past n (panics).

The empty block fills an empty table on both kernels.

### `compile_fail` doc tests

`cargo test --features run-fed --doc run_stage` gives 5 passed, on stable and on nightly:

| Doc test | Line | Error code |
|---|---|---|
| a hand-made `Sorted` | 48 | E0451 |
| a forged `Scratch.fo` | 52 | E0616 |
| records changed under a live `Sorted` | 57 | E0502 |
| the scratch re-sorted under one | 67 | E0499 |
| a checked block fills | 77 | runs |

**The codes are checked on nightly only.** With all four codes changed (E0452, E0615, E0501,
E0498):

- **stable** still passes 5 of 5: rustdoc ignores the codes there;
- **nightly** fails all four: "Some expected error codes were not found".

`test.yml` runs the doc tests on stable, so CI was not checking the codes. `miri.yml`'s
nightly job now runs them: the step "The run-fed doc tests' error codes (nightly)".

## Out of scope, and for #270

- **The forgeable `Kernels`.** `Kernels`' public fields let safe code claim AVX2 on a CPU
  without it. It is pre-existing and crate-wide, and not repaired or widened here, as D044
  records. Every pass above runs `Kernels::scalar()` or `Kernels::detect()` on a CPU that
  has the feature, or under Miri with it enabled.
- **For #270** (no change there). D044's requirement stands: the whole-table AVX2 fill's
  unmasked overstores must not transfer to parallel shares. A one-row share issuing an
  8-word store writes into its neighbour's rows. Writing the same value, or overwriting it
  later, does not make concurrent writes safe. C30/C31 or any shared-table fill needs its
  own physical-write-footprint argument, such as bounded tails or private scratch then a
  safe copy. A row partition alone does not establish exact ownership at share edges. This
  evidence covers only `fill`'s single-owner sweep and the safe scalar `fill_rows`.

## Reproduction

```
git fetch origin safety/run-fed-b5 && git checkout safety/run-fed-b5
cargo test --features run-fed --test run_fed_b5 -- --nocapture
cargo test --release --features run-fed --test run_fed_b5 -- --nocapture
MIBZ2_B5_FUZZ_SECS=300 cargo test --release --features run-fed --test run_fed_b5 fuzz -- --nocapture
cargo +nightly miri test --features run-fed --test run_fed_b5 -- miri_ --nocapture
RUSTFLAGS="-C target-feature=+avx2,+bmi1" MIBZ2_EXPECT_AVX2=1 \
  MIRIFLAGS="-Zmiri-env-forward=MIBZ2_EXPECT_AVX2" \
  cargo +nightly miri test --features run-fed --test run_fed_b5 -- miri_ --nocapture
cargo +nightly test --features run-fed --doc run_stage
```

Mutants: apply one `sed` from the table to the frozen file, run the suites, then
`git checkout -- src`.

These runs were made on a shared 4-core host. Their pass/fail and case counts are
deterministic for the seeds. The bounded run's seed count depends on the host's speed.
