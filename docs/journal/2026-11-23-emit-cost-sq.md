# The squared retained map (T²) in the Open kernel's emit: KILL

*23 November 2026. A C1a diagnostic, decided by its pre-registered rule. Spec: MONODROMY record 93
(`docs/journal/monodromy/2026-11-23-squared-trace-arm.md` at c3dab1ef, addendum b36ec511); review:
record 94 (1b188ada); rule: #145 LOG 7097052169, order correction LOG 7097922091. Recipe and
registration: `bench-emit-cost-sq` on `bench/emit-cost-sq` at `92a09f14` (#792, not merged:
research instrumentation). Evidence: `emit-cost-sq/run-20261123T110350Z` on `evidence` at
`a01d1f9c`, SHA-256 `cc1d465722c5cfbc80f34f04e8630a73103ac092f174a7e20ed7bf02f718a64d`. Verdict LOG
7099262911.*

## The idea, and the pre-screen that kept it alive

The Open kernel emits its retained rows by walking the retained map T from both ends. Record 92
proposed walking T² (word pairs) instead, halving the dependent rounds, after one square pass. Its
cheapest falsifier, MONODROMY's C pre-screen on synthetic Haar cycles, ran on the i9
(`squared-trace-prescreen/20261123T093340Z`, evidence `5218e05b`): T²/chase 0.726–0.760 at
m = 89,874 to 204,281, ALIVE under its rule (DEAD iff ≥ 0.95 at every m). Record 93 then predicted,
for the real emit at 10 workers, E = 0.65 [0.55, 0.78].

## The run

Base, `sq` (serial square pass, then a two-ended T²/T⁻² walk) and `sq-shadow` (the square pass
interleaved into the T walk, 4 iterations a step) in one binary, switched per block; 1, 6 and 10
workers; 20 rounds; the arms in all six orders. wikidata ×4: 932 open blocks a decode, 928
two-ended. E is arm emit over base emit per retained row; K the net change per row (construct +
square + emit), ns.

| workers | base emit, ns | `sq` E | `sq` K | `sq-shadow` E | `sq-shadow` K |
|--:|--:|--:|--:|--:|--:|
| 1 | 4.94 | 0.990 [0.982, 0.997] | +1.73 | 1.555 | +2.93 |
| 6 | 5.27 | 1.021 [1.016, 1.025] | +2.29 | 1.656 | +3.74 |
| **10 (judged)** | 6.18 | **1.219** [1.178, 1.261] | +4.78 | 2.141 | +7.61 |

Square pass 2.6 ns a row at 10 workers; scratch +8m bytes a kernel (1.85 MB at the largest m).
Decode wall at 10 workers: base 0.209 s, `sq` 0.272 s, `sq-shadow` 0.303 s.

**Verdict: KILL** (`sq`'s E > 0.80 at 10 workers). The line ends; `sq-shadow` decides nothing.

## What it shows

- **The synthetic chase did not transfer.** Halving the dependent chain bought 0.53–0.55 of the pure
  chase on the i9, but nothing on the real emit even at one worker (E 0.99), where there is no
  contention. Record 93's identification of Open's emit with the pure chase (b(1) 4.82 against 4.92
  ns) matched the totals but not what bounds the emit. Why is for MONODROMY's review (CHECKPOINT
  7099277283).
- **It gets worse with workers** (E 1.02 at 6, 1.22 at 10), consistent with the extra 8m bytes a
  kernel competing for shared cache and memory bandwidth; no counters were taken.
- **The lesson for pre-screens:** a synthetic kernel that reproduces a stage's total time does not
  show that it reproduces what bounds that stage. A cheaper next falsifier would have been the T²
  walk on the real tables at one worker, before the full three-arm harness.
