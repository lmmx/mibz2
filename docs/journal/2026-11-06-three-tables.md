# Three tables: confirming the team-size screen's best plans

Registered before any run, on 2026-11-06.

## The question

In the team-size screen (`2026-11-06-team-size.md`), neither registered primary raised the
8-thread ceiling. More threads on the same four tables gave 1.017 and 1.003. The grid showed
that throughput follows the number of live tables, and its two best cells had three:
- 3 × 4 at 12 threads: 1.097 (1.083–1.111) against 4 × 2 at 8;
- 3 × 3 at 10 threads: 1.075 (1.058–1.093).

Both used 18% less peak memory. They were picked after seeing the data, so those figures are
inflated by the selection and qualify nothing.

This run tests them as registered primaries on fresh data. If either passes and confirms, the
policy's cap moves from 4 teams to 3 on this machine.

## The run

`just bench-three-tables` on the i9, at this branch's commit:
- **The grid:** the same as the screen. `mibz2`, `mibz2-team2`, `-team3` and `-team4` run at
  8, 10, 12 and 16 threads, at the default window, for 20 rounds, paired by round against
  `mibz2@8`.
- **The counters:** the same separate pass, on seven cells including both primaries. It
  runs only if perf can count.
- **The occupancy pass:** this time the 3-table plans, 4 × 2 at 8, 3 × 3 at 10 and 3 × 4 at
  12, each pinned spread against packed.
- **The output:** one directory per attempt, its console included, packed into one tarball.

## The rule

The team-size rule, unchanged in every predicate, threshold and limit
(`2026-11-06-team-size.md`, "The rule", as frozen after the Guardian's D029). Only the
primaries change:
- **Primaries:** `mibz2-team4@12` (3 × 4) and `mibz2-team3@10` (3 × 3). The plan predicate
  wants every run of each to show exactly 3 leaders, with 9 and 6 helpers.
- **Thread counts:** a cell's thread count is what it asks for. `mibz2-team3@10` asks for 10
  and runs 10 // 3 = 3 teams of 3, which is 9 decoding threads; `mibz2-team4@12` runs 3 × 4 =
  12. The scanner, the feeder and main run beside them in every cell, the reference's too.
  The report counts each run's leaders and helpers from the thread names, so the plan is
  checked, not inferred from the request.
- **The reference** stays `mibz2@8` (4 × 2), and **the control** `mibz2-team2@8`.
- **A pass** needs all of these, as before:
  - the bytes, grid, reference and control predicates;
  - a mean of at least 1.03 and a lower bound above 1.00;
  - the adoption limits (peak RSS ≤ 1.15×, CPU-seconds ≤ 1.5× the reference's) in this run
    and in its confirmation.
- **Confirmation:** the same cell must pass in one predeclared second run of
  `just bench-three-tables`. No switching cells, no pooling, no third run.
- **Invalidation:** one replacement run in the whole experiment, as before.
- **The screen's runs are not reused,** for either the test or the confirmation.

## The prediction

The screen's figures are the best of 14 cells, so a fresh run should regress toward the
middle:
- **3 × 4 at 12 threads:** 1.04–1.09;
- **3 × 3 at 10 threads:** 1.03–1.07.

Both should keep their peak RSS about 18% below the reference's. 3 × 3 should use less CPU
than the reference; 3 × 4 about the same.

## What each outcome would mean

- **Either passes and confirms:** a PR changes the policy's cap on this machine from 4 teams
  to 3, and k from the cores per table. It is measured again, end to end, as a release-shaped
  comparison.
- **Both fail:** the screen's 3-table result was selection or noise. The cap stays at 4, and
  the next lever is placement (one thread per core).
- **The occupancy pass (diagnostic):** it shows whether a block decodes faster with 3 tables
  than with 4 at the same k, which is the mechanism the screen suggests. The occupancy limits
  of the team-size entry apply unchanged.

## Result

Not yet run.
