# Exact dispatch counts for intermediate horizons

On each marked walk, split the n bytes at every nonmodal predecessor. Each exceptional byte requires its explicit unit transition. A maximal positive modal streak of length r costs floor(r/8) + r mod 8 dispatches in the measured emitter. With all power-of-two horizons through eight, the minimum becomes floor(r/8) + popcount(r mod 8). Greedy is optimal for these dyadic lengths by pairing equal short jumps until the binary expansion remains. With a free arbitrary-length shortcut up to h, the exact minimum is ceil(r/h). These formulas permit direct replay from the true marked path, independently of the PDEP implementation. The first and last streaks are truncated to the output horizon.

| Across ten marked outputs | Fixed depth 8 | Variable depth ≤16 |
|---|---:|---:|
| Exceptional unit steps | 409,432 | 818,537 |
| Positive modal streaks | 329,843 | 350,888 |
| Measured 8/1 dispatches | 2,593,760 | 2,666,959 |
| Exact 8/4/1 dispatches | 2,064,794 | 2,354,848 |
| Exact 8/4/2/1 dispatches | 1,872,758 | 2,197,347 |
| Ideal arbitrary length 1–8 | 1,620,259 | 2,051,500 |
| Ideal arbitrary length 1–16 | 1,109,202 | 1,570,394 |
| Ideal arbitrary length 1–32 | 881,737 | 1,336,130 |
| Ideal unbounded single dispatch per streak | 739,275 | 1,169,425 |

Retaining both intermediate tables would reduce dispatch count by 27.8% fixed and 17.6% variable, before counting additional failed eligibility tests, rank queries, table size, output stores and setup. The actual fixed and variable emitters immediately after construction need 37.7% and 50.0% less CPU respectively just to match the entire scalar inverse. Dispatch counts give no proportional wall-time guarantee. Context discovery and index setup alone each already exceed the complete scalar's time for both modes. The measured candidate therefore has no credible path through this isolated schedule change. Arbitrary-length shortcuts or exception-crossing words need a distinct construction and measured full cost. [Paid comparison](bench/README.md)

The census replays dense true LF on the real cap inputs as an offline falsifier. Every one of the twenty rows agrees with the independently gated native counts of eight-byte jumps, total dispatches and exceptional transitions. [Raw per-block counts](horizons.csv)

```sh
d=$(dirname "$(rg --files | rg '/exceptions/jumps/native/jump.hpp$')")
python3 "$d/generate.py" /tmp/astra-physical-jumps
g++ -std=c++20 -Wall -Wextra -Werror -O3 -mbmi2 -mpopcnt -I/tmp/astra-physical-jumps "$d/horizons.cpp" -o /tmp/astra-physical-jumps/horizons
python3 "$d/horizons.py" /tmp/astra-physical-jumps/horizons /tmp/astra-cut-caps | diff - "$d/horizons.csv"
```
