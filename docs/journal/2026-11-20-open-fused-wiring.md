# Open's wiring: fused (F) and queue compaction (Q) (negative result)

**Question.** The phase split (`2026-11-20-open-phase-split.md`, #653) found the queued wiring,
counted whole, is 23-28% of the shipped open kernel's time (`Open<16, INDUCE = true, QUEUE =
true>`), mostly the per-run `Vec::extend` of the queue fill. Two rewirings were proposed, each
measured against shipped Open on its own:

- **Arm F, fusion (Neon):** the induced band loop writes `edges` (and `extra`) directly. There
  is no `bands`, no `cursors` and no `if QUEUE` loop. The codomain prefix sum goes in the dead
  `Node.prev[1]` instead of `Endpoint::bottom`. `wide` is decided by a lazy spill.
- **Arm Q, queue compaction (the phase split's finding; Neon proposed the same):** one queue
  entry per thin run instead of one per row, expanded in the scatter. No new pass.

**Answer. Both arms FAIL the decision rule.** Neither is at most 0.98x shipped Open on any
block, and both are above 1.01x on all six.

| block | F | Q (Neon's packing) | Qi (pre-sized, by index) | F+Q (information only) |
|---|---|---|---|---|
| 0 | 1.023 | 1.046 | 1.089 | 1.047 |
| 2 | 1.043 | 1.053 | 1.085 | 1.074 |
| 85 | 1.044 | 1.093 | 1.119 | 1.075 |
| 143 | 1.033 | 1.058 | 1.100 | 1.061 |
| 207 | 1.038 | 1.070 | 1.109 | 1.071 |
| 210 | 1.040 | 1.072 | 1.110 | 1.089 |

The table gives the median of the 41 per-round ratios of whole time to shipped Open's, single
thread. The p10-p90 of the ratios is in the CSV: F's p10 is 0.971-1.006, and no arm's p10 is
below 0.966.

## The decision rule (fixed before the data)

An arm PASSES if its median total time is at most 0.98x shipped Open's on at least 4 of the 6
blocks, and on no block above 1.01x. Otherwise it FAILS, recorded as a negative result with no
rescue rounds. The arms are judged independently; F+Q is reported for information.

Q was measured in two forms, as the brief allowed: Neon's packed encoding (the coordinator's
choice for Q's representation) and pre-sized per-row queues written by index (`Qi`). Q's verdict
uses the better of the two, the packed form. Both fail.

## Why

- **F removes almost nothing.** The bands are 2.4k-6.8k entries a block (12 B each). Fusing them
  away moves the scatter into the band loop, but the scatter itself is the same work: every
  retained row is still read from its queue and written to `edges`. Phased, the fused loop
  (0.118-0.278 ms) costs what shipped Open's bands and scatter cost together (0.116-0.267 ms),
  and 2-8% more. Whole, F executes 0.29-0.89 M more instructions a block than shipped Open
  (cachegrind). The likely cause is the per-band spill test and the per-row queue slice taken
  inside the larger loop, where the compiler hoists less. Interleaving the spelling's appends
  with the `edges` writes may also cost something whole that the phased form, with the spelling
  deferred, doesn't show. F's whole-time ratio (1.02-1.04) is above its phased total's (0.99-1.03
  of the phase split's clone of shipped Open, as a ratio of medians).
- **Q moves the cost from the fill to the scatter, and adds to it.**
  - The run pass halves, as Neon predicted: `a_runs` falls 0.45-0.98 ms to 0.24-0.51 ms
    (-46% to -54%).
  - The scatter (`e_wire`) rises 4.5-6.4x: 0.078-0.175 ms to 0.36-0.87 ms.
  - The thin runs average 1.6-2.2 rows (retained rows over queue entries), so each band expands
    many short runs. The per-row zip copy that a per-row queue allows becomes a branchy loop
    over runs of unpredictable length (1-16).
  - Net: +1.2-3.5 M instructions a block, and +4.6-9.3% time.
- **Qi (pre-sized, by index) is worse than `Vec::extend`.** The run pass rises 0.45-0.98 ms to
  0.58-1.28 ms: the bounds check and the slice per run cost more than `extend` on a `TrustedLen`
  range. So the queue fill isn't slow because of `extend`. It is slow because it writes every
  retained row (90k-204k `u32`s a block) in short scattered runs, into 256 growing vectors.

Neon's prediction for Q was an 8-15% total reduction a block, with the run pass and queue fill
falling 40-55%. The second half held (-46% to -54%). The first didn't: the saving went into the
scatter, and more.

## Stage split (phased forms, medians, ms)

Shipped Open's wiring counted whole is the queue fill (inside `a_runs`) plus `c3_bands` and
`e_wire`. F folds the scatter into `c3_bands`, so its `e_wire` is 0. The full split, with
shares, is in `2026-11-20-open-fused-wiring-stages.csv`.

| block | shipped a / c3+e | F a / c3 (fused) | Q a / c3+e | Qi a / c3+e |
|---|---|---|---|---|
| 0 | 0.454 / 0.118 | 0.451 / 0.120 | 0.237 / 0.404 | 0.579 / 0.115 |
| 2 | 0.491 / 0.150 | 0.497 / 0.163 | 0.261 / 0.494 | 0.628 / 0.157 |
| 85 | 0.674 / 0.116 | 0.663 / 0.118 | 0.313 / 0.582 | 0.877 / 0.115 |
| 143 | 0.982 / 0.267 | 1.011 / 0.278 | 0.507 / 0.962 | 1.285 / 0.266 |
| 207 | 0.673 / 0.167 | 0.669 / 0.173 | 0.365 / 0.603 | 0.875 / 0.168 |
| 210 | 0.935 / 0.179 | 0.928 / 0.189 | 0.509 / 0.795 | 1.225 / 0.183 |

The spelling (`d_spell`) and `emit` are unchanged within noise across the arms.

## Instructions (cachegrind)

The harness ran each arm twice per process, a warm-up and one run, under `FUSED_ONLY`, and the
counts were halved. The absolute figure includes reading the file and building the records, the
same for every arm, so only the deltas are the arm's own. Per block, against shipped Open:

| arm | delta (M instructions a block) |
|---|---|
| F | +0.29 to +0.89 |
| Q | +1.18 to +3.48 |
| F+Q | +1.06 to +3.26 |
| Qi | +1.51 to +3.93 |

## Scratch capacity (a hypothesis-level figure, not RSS)

These are every buffer's capacity after the block, as the kernel keeps them live between blocks.
No RSS change is claimed from them.

- **Queues:** Q's packed queues hold 284-680 KB, against shipped Open's 546-1,228 KB: -42% to
  -52%. Qi's high-water queues (grown by doubling) hold 2.5-20% more than shipped Open's.
- **All live scratch:**
  - shipped Open: 1.56-3.23 MB;
  - F: -2.8% to -5.9% (no `bands`, no `bottom`);
  - Q: -14% to -21%;
  - F+Q: -20% to -24%.

## Four workers in parallel

Four copies of the harness ran at once, one per core (`taskset -c 0..3`), each with its own
replica of the six blocks, 21 rounds each. The ratios to shipped Open on the same core keep the
single-thread ordering:

| arm | ratio to shipped Open |
|---|---|
| F | 1.02-1.06 |
| Q | 1.04-1.09 |
| Qi | 1.05-1.16 |

Contention doesn't rescue either arm. Q's smaller queues don't turn into a gain with four cores
sharing the L3.

## Correctness

- **Internal state:** on all six blocks, every research arm's construction equals shipped Open's
  (`Kernel::construct::<true, true>`) exactly: `edges`, `extra`, `spelling`, `offsets`, `mark`
  and `bits`. This covers F, Q, F+Q, Qi and F+Qi, in both the whole and the phased form (the
  harness's `check_state`, run before timing).
- **Output:** every output of every timed round (12 slots, 43 rounds, 6 blocks) is byte-exact
  against a dense-LF walk.
- **Arrow audit:** `checks.rs`'s audit is split into `audit` (construct, then check) and
  `arrows` (the check, given any construction's tables).
  - `fused::tests::arms_equal_shipped_and_audit` runs `arrows` on eight research
    configurations (F, Q, F+Q, the phased F and Q, an all-off copy, Qi, F+Qi).
  - It covers every binary column to length 8 and ternary to 5, at every mark and limits
    {0, 1, 2, 4, 16}, the limits `all_port_arrows` uses.
  - It also asserts each configuration's state equal to shipped Open's.
  - Q's 8-bit width field is enforced by a release `assert!(limit <= 256)`, tested by
    `compact_refuses_limits_above_256`. Every limit the audit uses is at most 16, and
    production's is 16.
- **Forced spill:** `fused::tests::forced_wide_spill` lowers the packing cap (a test-only
  `Spill::cap`) to `255 + words / 2`, so the spill happens partway through the band loop.
  - F and F+Q give `edges` and `extra` identical to a run wide from the start (`Spill::wide`)
    and to Q's unfused wiring at the same cap.
  - Their arrows audit clean, and `emit` gives the plain inverse's output.
  - A mutation that skips the spill's unpacking fails this test.
- **Other variants:** the default build's tests (`cargo test --lib open::`) pass, with the
  refactored `audit` still auditing all four shipped configurations. `cargo clippy
  --all-targets` is clean with and without the feature.

## Method

- **The arms:** research copies in `src/open/endpoint/fused.rs`, behind `open-phases`.
  - The construction is generic over `FUSE`, `COMPACT`, `INDEX` and `PHASED`. `PHASED`
    defers the spelling and stamps the stages, as the phase split's clone does.
  - With all four off, it is shipped Open's construction, and the tests check that too.
  - Production's `Kernel::construct` and `run` are unchanged. `checks.rs` (test-only) is
    refactored, and its audits are unchanged.
- **Harness:** `examples/open_fused.rs DIR 41 OUT`, run as `taskset -c 3`.
  - Twelve slots a round: shipped, F, Q, F+Q, Qi and F+Qi, each whole and phased. Shipped
    Open's phased form is the phase split's clone.
  - The order is rotated each round, with 2 warm-up rounds, and buffers are kept between
    rounds.
  - Ratios are per round, against shipped Open's whole time in the same round.
- **Machine:** the shared 4-core cloud VM (Xeon @ 2.1 GHz). Local timings are indicative only.
- **Blocks:** the six admitted Wikidata columns (0, 2, 85, 143, 207, 210).

## Files

- `2026-11-20-open-fused-wiring.csv`: per block and arm, the whole-time median and per-round
  ratio spread; the stage medians (`a_runs`, `c3_bands`, `d_spell`, `e_wire`, wiring whole,
  `f_emit`); instructions and their delta; queue and live bytes; and the four-worker ratio by
  core.
- `2026-11-20-open-fused-wiring-stages.csv`: the full stage split in the phase split's columns.

## Rejected

- **F**, fused wiring: about 1.02-1.04x, with no gain to find. The scatter is the work, and
  the bands it removes are a few thousand entries.
- **Q**, queue compaction: about 1.05-1.09x. The run pass halves, and the scatter grows 4.5-6.4x.
- **Qi**, pre-sized queues written by index: about 1.09-1.12x, slower than `Vec::extend`.
