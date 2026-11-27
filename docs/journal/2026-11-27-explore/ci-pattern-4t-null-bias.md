# CI's paired bench at `pattern`, four threads: a head-arm bias and a high false-flag rate

*Explore, 27 November 2026. Primary evidence only: the `bench-pr-*` artifacts (`runs.jsonl`, `result.json`, `comment.md`) of 20 recent successful `Benchmark` runs on pull requests, 25–27 November. Script and per-run table: [`ci-pattern-4t-null/`](ci-pattern-4t-null/).*

## Why look

The open flag [`2026-11-27-scanner-coalesced-ci-acceptance`](../../../frontier/flags/2026-11-27-scanner-coalesced-ci-acceptance.md) rests on one cell of the exact-head run [45947633032](https://github.com/lmmx/mibz2/actions/runs/45947633032): `pattern` at four threads, wall +7.3% (95% CI +2.6%, +12.3%), flagged **slower**. The probe claim `scanner-per-block-fixed-cost` asks whether the scanner causes it. This note asks the prior question: what does that cell do when the decoder doesn't change?

## The pool

20 runs, all at bench 3.9.0 on the same runner class (Xeon Platinum 8573C, 4 vCPUs on 2 cores, 10 paired rounds, `SCALE 4`):

- **Scanner:** 2 runs whose diff is #1034's scanner (`782f4feb`, the exact head; `d8b47e0c`, an earlier head).
- **Strict null:** 5 runs whose head and base are identical in `src/`, `Cargo.*` and `build.rs`, or differ only in `src/python.rs`, which the `ab` binary doesn't build (`02e492a7`, `1de36285`, `77adf04e`, `3fb0937e`, `a0370c77`).
- **Non-scanner:** those 5 plus 13 runs that change other decoder code (Open kernel, runs fold, MTF, parallel/trace, pooled scan). None touches `src/bits.rs`, `src/block.rs` or `src/huff.rs`. Several, such as the Open kernel's, shouldn't run at ≤4 threads at all.

The head arm is always `mibz2` (the head's `ab`), the base arm `mibz2@base/ab`, and their order alternates each round.

## What the null pool shows

Over the 18 non-scanner runs (180 paired rounds a cell), the geometric-mean head/base wall ratio and a 95% cluster-bootstrap interval over runs:

| cell | head/base wall | 95% CI (runs) | median head/base peak RSS | flagged slower / faster |
|---|--:|--:|--:|--:|
| `pattern` 4T | **1.0186** | **1.0027–1.0338** | **1.0169** | **3 / 0** |
| `pattern` 2T | 1.0039 | 0.9851–1.0235 | 0.9989 | 1 / 0 |
| `pattern` 1T | 0.9981 | 0.9927–1.0061 | 0.9999 | 1 / 2 |
| every other corpus × threads (18 cells) | 0.9943–1.0097 | all include 1 | 0.989–1.012 | 0–2 / 0–2 |

- **One biased cell.** Only `pattern` at 4T has an interval excluding no change. Its head arm is about 1.9% slower and uses 1.7% more peak RSS than the base, though both arms run identical decoder code in the strict-null runs. That pool alone gives a mean of +1.5%, and 2 of its 5 runs are flagged **slower** (`77adf04e` +6.1%, `a0370c77` +6.0%).
- **Not the order.** Head-first rounds give a median ratio of 1.0125, base-first rounds 1.0146.
- **A high flag rate.** The 10-round interval flags this cell slower in 3 of 18 null runs (17%) and faster in none. If its interval held its stated coverage, the rate would be about 2.5% a direction. Between-run spread of the point estimate is σ ≈ 3.3%.
- **A near-control on the same base.** `faa9df70` (Open-kernel endpoint, base `f9848a4`, the same base as `782f4feb`, run three minutes earlier) gives +5.4% unflagged.

## Placing #1034's figure

| run | `pattern` 4T head/base | wall CI (report) | CPU change | RSS change |
|---|--:|--:|--:|--:|
| `782f4feb` (exact head) | 1.0734 | +2.6%, +12.3% | −2.2% | +7.9% |
| `d8b47e0c` (earlier head) | 1.0158 | −1.7%, +5.0% | −0.6% | +0.4% |
| non-scanner pool, 18 runs | mean +1.9%, σ 3.3%, max 1.0683 | | | |

`782f4feb`'s ratio is the highest of the 20, just above the null maximum (1.0683, `12222999`, a runs-fold change). It sits about 1.6σ above the null mean, or about the 95th percentile of an 18-run empirical null. With the null bias removed, it's about +5.3%. On the same cell its CPU fell 2.2%: more wall for less work, the signature of latency or scheduling rather than added computation. The earlier head of the same scanner, `d8b47e0c`, falls well inside the null.

## What this does and doesn't show

- **It shows:** CI's paired bench has a systematic, unexplained head-arm penalty of about +1.9% wall and +1.7% RSS on `pattern` at 4T, absent from every other cell. In that cell the 10-round interval undercovers: it raises false "slower" flags several times more often than its nominal rate. A single flag there is weak evidence of a regression.
- **It does not show** that #1034 is free on `pattern` at 4T. Its exact-head figure is at the edge of the null, not inside its bulk, and remains about 5% after the bias is removed. The per-block fixed-cost probe (`scanner-per-block-fixed-cost`) is still the way to test a mechanism. Its CONFIRMED threshold is half the CI loss, so it should read the null-corrected loss (≈ +5.3%) beside the raw +7.3%.
- **The flag** is not resolved here. Policy §2 rule 3 weighs a secondary loss without a rerun, and how much a measured null bias may discount a flag is a policy question for whoever resolves the flag, not something an exploration decides. This is evidence for that agent.
- **Mechanism of the bias:** unknown. `pattern` at 4T is a ~20 ms run of about 22 tiny blocks on 2 SMT-shared cores, so the pipeline-fill and scheduling share is large. A difference between the two arms' binaries (the head's `ab` versus the base's, including harness changes) or their process start-up could show here and nowhere else. The idea claim `ci-pattern-4t-arm-bias` asks.

Artifacts expire in February 2027. `ci-pattern-4t-null/per-run.tsv` keeps each run's per-cell geometric-mean ratio, and `analyse.py` recomputes the table from re-downloaded artifacts.
