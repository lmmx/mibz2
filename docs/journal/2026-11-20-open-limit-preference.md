# The open kernel's thin-run limit: no exploitable per-block preference

*20 November 2026. A bounded research measurement for the Coordinator.*

- **The question:** does the shipped open kernel's thin-run limit (`LIMIT = 16`, the arm
  `Open<16, INDUCE=true, QUEUE=true>`) have a real per-block preference for a smaller or
  larger limit on Wikidata blocks? If so, could counts the kernel already holds after its run
  pass predict it?
- **What it decides:** nothing in production. It tests whether a per-block limit is worth a
  predictor. The rule was fixed in the brief, before any data.

**The finding: no.** `LIMIT = 16` stands.
- *A per-block preference exists, but it is small.* Choosing each block's best limit on half
  the rounds and scoring it on the other half saves 0.8% of the time (total-weighted ratio
  0.992). The rule's line is 0.98. Even the in-sample oracle, which cheats, reaches only 0.990.
  So no predictor could reach the line, and Step 3 did not run.
- *No fixed limit beats 16.* Against 16, limit 8 is 7.2% slower, 12 is 1.8% slower and 24 is
  0.6% slower, each in total and each with its 95% interval clear of 1.
- *The earlier "i8 beats i16 on 70 blocks" was noise, or a property of the unqueued arms.* On
  the shipped queued arm, limit 8 is faster than 16 on none of the 228 blocks, and
  significantly slower on 181. A 2-round sign, like the earlier 2-lap medians, correlates only
  0.24–0.38 with the same block's other 38 rounds.

## What we believed

`2026-11-18-open-return/evidence/phases-wikidata.csv` (2-lap medians, 233 large blocks, the
non-queued arms) has `i8` beating `i16` on 70 blocks. We doubted it:
- i8/i16 correlated 0.46 with qi16/i16, an arm at the same limit, which points to noise in
  the shared denominator;
- r/n didn't predict the preference;
- only 34 of the 70 i8 wins were also i4 wins.

## Method

- **The harness:** `ab limit-sweep ROUNDS FILE`, added here to `examples/ab.rs` (feature
  `open-return`), on `bench/open-admission`.
  - *The blocks:* a collector (`open_gate::SweepCollect`) decodes the file through LazyCells
    as shipped. It keeps every run-route block LazyCells admits with at least 65,536 rows: its
    capped run records, `n`, the primary and LazyCells' output.
  - *The arms:* `Kernel::run::<INDUCE=true, QUEUE=true, SEGMENTS=false>` at limits 8, 12, 16
    and 24, the research kernel's `Open<L, true, true>`. Each limit has its own kernel,
    reused across blocks, as a production worker's is.
- **The research kernel is the shipped one.** Comments stripped, master's `src/open.rs` and
  `src/open/endpoint.rs` differ from the journal's `open_return.rs` and `endpoint.rs` only in:
  - their wrappers (`OpenCells` versus `Open`, the research `SEGMENTS` and `price` paths);
  - one extra `assert!(width > 0)` in the shipped `endpoint.rs`.

  `construct` and `emit` are identical. The limit is a runtime argument there, so one build
  times every limit.
- **The timing:**
  - one thread, `taskset -c 2`, in the cloud container (4 cores, shared, otherwise idle at
    the time: load about 0.6);
  - a release build (`cargo build --release --features open-return --example ab`);
  - 40 rounds over all 228 blocks;
  - within each round, each block's four limits are run back to back, in an order rotated by
    round and block (`(slot + round + block) mod 4`).
  - Every timing is recorded. The arms are paired within a round: per-round totals at 16
    ranged from 360 to 505 ms across the run, which is the shared machine's drift.
- **Correctness:** every output, in the warm-up and in all 40 rounds at all four limits, is
  byte-exact against LazyCells' output for that block (an `assert!`). A whole-file decode
  through the collector matches `mibz2::decode`.
- **The input:** the pinned Wikidata sample, `wikidata.bz2` (SHA-256 `82f65ace…`, as in
  `2026-11-19-open-admission/inputs.sha256`).
- **The admission:** the collector admits 228 blocks. They are exactly the blocks
  `2026-11-19-open-admission/blocks.jsonl` records as `lazy_admitted` with `n >= 65,536`:
  the same indices, record counts and row counts. Every admitted Wikidata block has at least
  65,536 rows, so the floor removes none. The indices are the `block` column of
  `features.csv`.

## Results

`analyse.py` reads the rounds and prints `results.txt`. Ratios are `t_L / t_16`.

**1. Split-half reliability.** This is the correlation, across blocks, of the mean
log(t_L/t_16) over the odd rounds against the even rounds.

| L | split-half r | Spearman-Brown, 40 rounds | sd across blocks |
|--:|--:|--:|--:|
| 8 | 0.464 | 0.634 | 0.030 |
| 12 | 0.320 | 0.484 | 0.025 |
| 24 | **0.583** | 0.737 | 0.035 |

At 24 the preference is reliable enough to measure: it is over the rule's 0.5. Its spread is
small, though: a standard deviation of 3.5% in the log ratio, around a mean slightly above 0.

**2. Blocks with a significant preference.** A block counts when some limit other than 16
has a 95% paired-t interval on the mean log ratio over 40 rounds that lies wholly below 0.
- **Any limit:** 25 of 228 blocks (11.0%). Under Bonferroni over the three limits: 14 (6.1%).
- **Limit 8:** faster on 0 blocks, slower on 181.
- **Limit 12:** faster on 8, slower on 60.
- **Limit 24:** faster on 17, slower on 56.

With three tests at 2.5% one-sided each, up to about 7.5% would pass by chance.

**3. The honest oracle ceiling.** Each block's limit is chosen by its median on one half of
the rounds, then scored by the medians on the other half.

| choose on | score on | mean ratio | total-weighted | picks (12 / 16 / 24) |
|---|---|--:|--:|---|
| odd rounds | even rounds | 0.9921 | **0.9918** | 23 / 100 / 105 |
| even rounds | odd rounds | 0.9925 | 0.9923 | 22 / 101 / 105 |
| (in-sample, all rounds: not honest) | | | 0.9898 | |

Limit 8 is never picked.

**4. Each fixed limit against 16, total-time weighted.** The first figure is from per-block
medians over all rounds. The second is from per-round paired totals, with its 95% interval
across the 40 rounds.

| L | total ratio (medians) | per-round totals | 95% interval |
|--:|--:|--:|---|
| 8 | 1.0719 | 1.0623 | 1.0585 – 1.0662 |
| 12 | 1.0181 | 1.0157 | 1.0117 – 1.0198 |
| 16 | 1 | 1 | |
| 24 | 1.0056 | 1.0088 | 1.0054 – 1.0122 |

No fixed limit reaches 0.98. 16 is the best of the four, in total.

## The rule, applied

These are the brief's terms, fixed before the data:
- **"The split-half correlation is below 0.5 for every L":** no. It is 0.583 at 24.
- **"The honest oracle ceiling, total-time weighted, is above 0.98":** yes, at 0.992 (and
  0.992 the other way round).

So: **STOP.** No exploitable per-block preference; `LIMIT = 16` stands. No fixed limit is at
or below 0.98, so there is no separate finding. Step 3, the predictor, did not run. A perfect
predictor would be worth at most about 1% (the in-sample oracle's 0.990), against the
rule's 2%.

## Why the earlier sign misled

- **It was a different arm.** Its i8/i16 comparison was between non-queued arms. On the queued
  arm that ships, limit 8 is 6–7% slower in total and never faster on any block.
- **Two laps are mostly noise at this effect size.** For each limit, a 2-round mean log ratio
  correlates 0.24–0.38 with the same block's other 38 rounds. With a between-block spread of
  about 3%, 2-lap medians on a shared machine sort blocks by noise more than by preference.

## What remains open

- **A preference for 24 on some blocks is real but small:** a split-half r of 0.58, and 105
  blocks picked 24 on half the rounds. It is worth under 1% even if perfectly predicted, so
  it is not pursued.
- **One machine:** the cloud container's Xeon, one thread. The i9's cache sizes could shift
  the optimum. No more than 1% is in play here, so a confirmatory i9 run isn't proposed.

## Files

- `2026-11-20-open-limit-preference/rounds.csv`: every timing, one line each, 36,480 lines
  (`file, block, round, slot, limit, ns`; `slot` is the position within the block's rotation).
- `2026-11-20-open-limit-preference/features.csv`: the kernel's counts per block and limit,
  from the warm-up decode (`n`, records, runs, retained thin rows, thick strips, bands, words,
  table bytes): the features Step 3 would have used.
- `2026-11-20-open-limit-preference/analyse.py` and `results.txt`: the analysis, standard
  library only, and its output.

## Reproducing

```sh
cargo build --release --features open-return --example ab
LIMIT_SWEEP_FEATURES=features.csv taskset -c 2 \
  target/release/examples/ab limit-sweep 40 wikidata.bz2 > rounds.csv
python3 docs/journal/2026-11-20-open-limit-preference/analyse.py rounds.csv features.csv
```

The run took 78 s.
