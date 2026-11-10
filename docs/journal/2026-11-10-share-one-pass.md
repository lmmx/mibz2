# 2026-11-10: `build_one_pass`'s loop in `FusedBuild::share`, counted

**Status: counted, not timed. Condition 6 is UNMET, so the count is not read as registered
(FAIL-MEASUREMENT under the registration's own rule). The Guardian disposes.**
- **Registered** at `bd7a6b75f` (`2026-11-10-share-one-pass-registration.md`), before the
  kernel.
- **Kernel and tests:** `eac71660c`.
- **Stacked on port 1,** #335 at `efbdb53af`.
- **Every figure** below is computed from the archive (`cg/*.gz`) by `count.py.txt`. Its full
  output, with every profile's SHA-256, is `count.txt`.

**What was counted:**
- **`share` inclusive:** 6.786 → **5.403** Ir/D (−1.383). That is below the falsifier's 5.5,
  but above the predicted band of 5.16 to 5.25.
- **Engagement:** conditions 3 to 5 are met by instruction identity, exactly where an exact
  count was registered.
- **Whole process:** 36.006 (the same-session control) → 34.498. The control missed port 1's
  archived 35.800 by 0.206, against a registered tolerance of 0.1. That is condition 6.

## 1. The change

`FusedBuild::share`'s scatter is now `coop::scatter`, a twin of `bwt::build_one_pass`'s loop
(#312), on one range:
- **The classifier:** `run_joins` classifies each window.
- **One run:** `ALL_JOINED` takes `run16`, as port 1 does.
- **At most `RUNS` runs:** one cursor step and one `run16` a run, when `s + 16 ≤ lim[c]`.
  Otherwise the run's rows are written one at a time.
- **Every other window** takes 16 one-row steps, then the tail.
- **`lim`** is the share's window end.

**`bwt.rs` changes only visibility:** `run_joins`, `few_runs`, `row_byte`, `ALL_JOINED` and
`RUNS` become `pub(crate)`. The loop is copied, not shared, so `bwt::build`'s source is
unchanged.

**`read_u64`'s import** moves into the test module. The production code no longer uses it.

**Safety** (written as `scatter`'s `# Safety` contract, and at each `unsafe`):
- **The contract:** for every c, `ftab[c]..lim[c]` lies in `[0, n)` and is this thread's
  alone.
- **How `share` meets it:**
  - `new` checked every checkpoint to be within the counts, and the counts to sum to n;
  - the windows are disjoint because the checkpoints rise;
  - a share's index is claimed once;
  - the table is held until `finish`.
- **Every write is inside its window:**
  - the one-row step writes slot s < lim[c];
  - each 16-wide write has `s + 16 ≤ lim[c]`;
  - so every `run16` reads `ll[s..s + 16]` and writes slots `s..s + 16` inside the window.
- **Provisional entries:** `s + len..s + 16` lie above the new cursor in this share's own
  window. The same call's later step that reaches each of them overwrites it, before the end
  check can pass.
- **On refusal:** a refused share leaves them, but `finish` never sets the table's length.
- **Wrapping links:** a provisional entry's link may wrap (`run16`'s add). It is overwritten.
- **Proof scope:** the argument covers what `new` accepts. It does not assume the
  checkpoints are the block's own.
- **B5:** the implementer does not discharge it. An independent adversarial review is still
  required.

## 2. Tests (before the count)

**New tests:**

| test | what |
|---|---|
| `share_one_pass_is_the_share` | 600 columns of short runs (2 to 5 bytes, runs of 1 to 24) plus every window of 2 to 7 runs at every offset 0 to 15, at k 1 to 4, strides 1, 7, 16, 64: port 2 against `bwt::build`, the scalar share (port 0) and **port 1's share** (a verbatim reference copy, `port1_share_reference`), entry for entry, refusals included |
| `share_one_pass_refuses_as_port_1` | 1,500 columns with lying counts and checkpoints: port 2 accepts and refuses exactly what port 1 does, and what it accepts is `bwt::build`'s table. It requires more than 100 of each |
| `miri_share_one_pass` | the run-by-run windows near shares' window ends, the provisional entries and the shares' disjoint slots under Miri's race detector, at k = 2 and 3 |

**Kept passing:**
- port 1's tests;
- the independent B5 reviewer's three tests (`b5_guard_top_bucket_sentinel`,
  `b5_top_byte_and_largest_block`, `b5_lying_counts_fuzz`);
- all of `cargo test --release`.

**Results:**
- `cargo test --release --lib coop::` passes 30 of 30.
- The debug-mode `coop::` tests pass 30 of 30.
- Miri, `cargo +nightly miri test --lib -- miri_share`, passes 3 of 3 in 409 s.
- fmt and clippy (`-D warnings`) are clean.

**Mutants of the new logic** (`cargo test --release --lib coop::`):

| mutant | result |
|---|---|
| M1 run guard laxer, `s + 15 <= lim` | caught: `b5_lying_counts_fuzz`, `b5_top_byte_and_largest_block`, `build_fused_is_build`, `miri_share_one_pass` |
| M2 cursor `s + len + 1` | caught (8 or more tests) |
| M3 last run's length from bit 15, not 16 | caught |
| M4 a run's base the window's row, not the run's | caught |
| M5 fallback loop one row short | caught |
| M6 run guard stricter, `s + 17 <= lim` | **survives, equivalent:** more runs take the one-row steps, which write the same final table. Its cost is in Ir only |

## 3. Inventory (archived profiles; `count.txt` has each file's full SHA-256)

**Binaries:**
- port 1: `ab` at `efbdb53af`, SHA-256 `9a35efe6…`;
- port 2: `ab` at `eac71660c`, `a898ef13…`.

Both are built by `cargo build --release --example ab`, rustc 1.99.1.

| arm | files | threads | share calls by helper thread (7–12) |
|---|---|---|---|
| port 1 team | `port1-team.30932-01.gz` … `-12.gz` | 1–12, each once | 76, 76, 75, 75, 82, 82 |
| port 2 team | `port2-team.31118-01.gz` … `-12.gz` | 1–12, each once | 81, 80, 81, 80, 72, 72 |
| port 2 one worker | `port2-one.31604-01.gz` | 1 | |

- **How the profiles were archived:** callgrind wrote them into a directory that only ever
  held raw files. They were gzipped there without `-k` and moved into `cg/`.
- **The loader now refuses doubled input:** `bench/team_gap_count.py`'s `load` exits on a
  raw-and-gzip pair or on a repeated thread. The count ran through it.
- **D** = 209,700,196, each arm's `"out"`. All three hash to `a8830caee5c511a6`.

## 4. The count (Ir per decoded byte)

| figure | port 1 (archived, #335) | port 1 (this session, control) | port 2 | port 2 − control |
|---|--:|--:|--:|--:|
| **`share` inclusive** | 6.7863 | **6.7863** | **5.4032** | **−1.3831** |
| (self + arcs) | 1,422,933,571 + 148,188 | 1,422,933,571 + 148,188 | 1,132,900,032 + 148,188 | |
| build stage | 6.9545 | 6.9545 | 5.5714 | −1.3831 |
| MTF | 4.7148 | 4.7148 | 4.7148 | 0 |
| walk | 17.7466 | 17.8024 | 17.7633 | −0.0391 |
| coordination | 0.0739 | 0.0662 | 0.0662 | 0 |
| harness | 1.4833 | 1.5706 | 1.5259 | −0.0447 |
| other | 0.0248 | 0.0899 | 0.0580 | −0.0319 |
| **whole process** | **35.8005** | **36.0055** | **34.4980** | **−1.5075** |
| one worker | 25.7917 | | 25.7917 | |

- **The `share` and build figures are exact:** the control reproduces port 1's archived
  `share` to the instruction.
- **The whole process is not exact.** Thread 2, `ab`'s sampler, executed 36,978,340 Ir in
  the control, against 5,041,849 in port 1's archived count. The shared machine was loaded:
  the control's run queue was 30.5 s, against 8.7 s.
- **What moved:** the sampler's extra work, and the walk's split between the helpers, moved
  the harness, the "other" row and the walk. Together that is 0.205 Ir/D.

**Supplementary, not registered:** the decoder alone (the whole process less the harness and
"other") is 34.292, 34.345 and 32.914 for the three team columns. Port 2 against the control
is −1.431.

**Where port 2's `share` still exceeds one worker's `bwt::build`** (5.403 against 5.160 Ir/D;
by source line, the archive):
- the bulk of the kernel matches line for line: the `run16` arithmetic (`macros.rs:173` 0.781
  and 0.782, `sse2.rs:111` 0.583 and 0.584), the stores (0.291 and 0.292) and the classifier
  (`sse2.rs:1569` 0.125 and 0.125);
- the excess sits in the loop's control: `uint_macros.rs:1092` 0.25 (absent in `build`),
  `cmp.rs:2144` 0.16 and `range.rs:1100` 0.087.

That attribution is read off the lines and not analysed further. It is the next question, and
it puts the predicted band's miss in the loop's control, not in the windows.

## 5. Conditions

| # | condition | result |
|---|---|---|
| 1 | same bytes | **met**, all three arms |
| 2 | 466 `share` calls, none on an MTF thread | **met**, both team arms |
| 3 | port 2's `share`: `run_joins`'s `pmovmskb` exactly 13,106,081, `rol` 0; port 1's: `pmovmskb` 0, `rol` 13,106,081 | **met:** `pmovmskb` at `0xd697a` ×13,106,081 and no `rol` executed; port 1's `rol` at `0xd69f1` ×13,106,081 and no `pmovmskb` executed |
| 4 | the `ALL_JOINED` site's four `movdqu` stores exactly 10,740,083 each | **met:** `0xd6a1b`, `0xd6a25`, `0xd6a30`, `0xd6a3b`, ×10,740,083 each. The site is named by the next executed `coop.rs` line, 618 (`ftab[c] = (s + 16)`) |
| 5 | the `few_runs` site's four stores between 3,670,695 and 4,568,175 | **met:** `0xd6add` … `0xd6afd` ×4,536,329 each, next line 636 (`ftab[c] = (s + len)`). That is 31,846 fewer than one worker's 4,568,175 (its counted site, next line `bwt.rs:355`), within the bound. No such site in port 1 |
| 6 | control within 0.01 of 6.786 (`share`) and within 0.1 of 35.800 (process) | **`share` met** (6.7863); **process UNMET:** 36.0055, off by 0.205 (§4) |
| 7 | one worker within 0.01 of 25.792 | **met:** 25.7917 |

- **Prediction:** `share` about 5.2, in the band 5.16 to 5.25; whole process about 34.2.
  **Counted:** 5.403 and 34.498. The band is missed by 0.15.
- **Falsifier:** `share` not below 5.5. **Not triggered** (5.403).
- **Classification:** none here. Condition 6 is unmet, and the registration requires every
  condition. The Guardian disposes.

## 6. Deviations, disclosed

1. **Condition 6's process tolerance (0.1) was tighter than the harness's own variation** on
   a loaded machine. The sampler thread's Ir varies with the run queue. Unmet, reported as
   unmet, nothing re-run. The `share` half of condition 6 holds exactly.
2. **The predicted band was missed.** The 0.24 Ir/D above one worker's per-row cost sits in
   the loop's control (§4).
3. **A test-generator overflow** (`u8 * 85` in `run_columns`) was found by the debug build
   and Miri before any count, and fixed. No kernel change.
4. **Miri's size:** `miri_share_one_pass` was first written with columns of up to 1,500 rows.
   It ran past ten minutes under Miri, was stopped, and was cut to four columns of up to 240
   rows. The three `miri_share*` tests take 409 s together, so CI's Miri job grows by about
   5 minutes.
5. **`bench/team_gap_count.py` changed** (the loader's refusal). That touches `bench/**`,
   which the CI bench's paths include. No measured figure changes meaning.
6. **The sites were identified by adjacent source lines.** With `line-tables-only`, the
   stores carry `core::arch` lines. Each store site was named by the first executed
   `coop.rs` line after it. Conditions 3 and 4 needed nothing beyond mnemonics and exact
   counts.

## 7. Dispatch paths (G3)

| path | changes? | evidence |
|---|---|---|
| pipelined team helpers (`invert_pipelined`) | **yes** | this count, the unit tests |
| ordinary teams (`build_fused` on the pool) | **yes**, the same function | the unit tests; not counted |
| plain worker `bwt::build` / `build_one_pass` | **no** (visibility only) | condition 7: 25.7917, 3,273 Ir of 5.41 G from port 1's binary |
| team solo paths (`parallel::solo`) | **no** | read: they call `bwt::build` |

## 8. What it does not say

- **Not time.**
- **One corpus.**
- **CI's A/B never runs a team,** so its neutral reading says nothing of this.
- **On #334's base (model):** 30.21 − 2.15 − 1.38 ≈ 26.7.
- **The team's build is now 5.571 Ir/D** against one worker's 5.160: 0.41, of which
  `FusedBuild::new` is 0.16.

## Reproduce

```text
python3 docs/journal/2026-11-10-share-one-pass/count.py.txt . docs/journal/2026-11-10-share-one-pass AB_PORT1 AB_PORT2
```

The binaries are needed only to name instructions (objdump). Every Ir comes from `cg/*.gz`.

## 9. The independent B5 reviewer's tests, adopted

The independent B5 review of #337 found no material objection (posted on #337). Its
tests-only diff (SHA-256 `903d5fb7…effd`, against `3797905e`) is applied unchanged; `cargo fmt`
changed nothing. **These are the reviewer's tests, not the implementer's.**

| test | what |
|---|---|
| `b5_scatter_stays_in_its_windows` | lying checkpoints under which a guard weakened to `ll.len()` (the reviewer's M7 and M8) lets a share write into another share's window. It catches those weakenings, which the implementer's suite did not. M8's weakness was reachable in port 1 too |
| `b5_scatter_largest_block_short_runs` | the largest block `new` accepts, in short runs, through the run-by-run path |

**Result:** `cargo test --release --lib coop::` passes 32 of 32. fmt and clippy
(`-D warnings`) are clean. The kernel, the count and the dispositions are unchanged:
condition 6 is still unmet (§5).
