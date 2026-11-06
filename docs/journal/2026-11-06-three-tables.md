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

- **Either passes and confirms:** a policy candidate, not an adoption. A PR changing the cap
  on this machine from 4 teams to 3 still needs its own measured-head CI, parity, safety and
  regression evidence, as any decoder change does.
- **Both fail:** no qualification. The report names the predicate that failed, and the
  effect's interval is kept. A failure does not show that the screen's result was selection
  or noise.
- **The occupancy pass (diagnostic):** it traces 4 × 2, 3 × 3 and 3 × 4. No pair shares a k,
  and the total window changes from 16 to 12 blocks, so it cannot isolate a table-count
  mechanism or compare block times at the same k. That claim is deferred (the Researcher
  and the Guardian, D030). The trace reads waits and spans per plan, under the team-size
  entry's occupancy limits.
- **Error rate:** the nominal fixed-run bound is 2 × 0.025 = 0.05 under the paired-log model.
  It is not campaign-wide, and not calibrated for the replacement run.

## Frozen

The Guardian carried the D029 rule over exactly (D030, #145 comment 6683039047). The
Researcher kept the screen and rule (#145 comment 6683028366). The predicates are fixed from
here. The two primaries were frozen before collection, and no screen observation is reused.

## Result

### Run 1 of 2 (the test): both primaries pass

Louis handed over the bundle in the Coordinator's session on 2026-11-06 and named it run 1 of
2. That act is the run's authorisation (RULES.md C5).

**Provenance.**
- **The bundle:** `20261106T105317Z.tar.gz`, SHA-256 `15ee5617cc9d32b2…`.
- **When and where:** started 10:53:17 UTC on 2026-11-06, at `639c3df1`. That commit is on this
  branch, 4½ minutes after the freeze at `656d2794`, and changes only the journal. The recipe,
  `bench/`, the decoder and `Cargo.lock` are identical at the freeze, at `639c3df1` and at
  `838d765c`.
- **The machine:** the i9-10900K, 10 cores and 20 threads, with no CPU quota or throttling.
- **The data:** the corpus SHA-256 matched. All 431 blocks decoded byte-identically in
  libbzip2, lbzip2 and every mibz2 path.
- **The counters:** none. perf couldn't count on the machine (`perf_event_paranoid`), and the
  registration makes the counter pass conditional on that. The occupancy pass ran for all
  three plans.

**The report.**
- The Coordinator reran `bench/team_size.py report` from the raw `runs.jsonl` with this
  branch's script, and got the bundle's report back byte for byte, as JSON and as Markdown.
- The two primaries' ratios, intervals, plans and adoption ratios were also recomputed
  independently from the raw runs. They agree to four decimal places.

**The predicates.**
- **Bytes:** one output hash and one length across all 320 runs, and no failed run.
- **The reference** `mibz2@8`: rounds 0–19 once each, 4 × 2 in every run.
- **The grid:** 16 cells × 20 rounds, each round once.
- **The control** `mibz2-team2@8`: 1.004 (0.992–1.016), within ±2%.

| primary | plan in every run | 20 unique pairs | mean ≥ 1.03 | lower bound > 1.00 | peak RSS / ref (≤ 1.15) | CPU / ref (≤ 1.5) | registered prediction |
|---|---|---|---|---|---|---|---|
| `mibz2-team4@12` (3 × 4) | yes: 3 leaders, 9 helpers | yes | **1.090** | **1.072** | 0.82 | 1.04 | 1.04–1.09 |
| `mibz2-team3@10` (3 × 3) | yes: 3 leaders, 6 helpers | yes | **1.076** | **1.058** | 0.82 | 0.90 | 1.03–1.07 |

Both primaries pass every predicate in run 1. **This is not a qualification.** The same cell
must pass again in one predeclared second run of `just bench-three-tables`, with no switching
cells and no pooling. Until then, nothing changes in the policy.

**Against the predictions.**
- **3 × 4:** at the top of its predicted range.
- **3 × 3:** just above its predicted range.
- **Peak memory:** 18% below the reference's for both, as predicted.
- **CPU:** 3 × 3 used less than the reference and 3 × 4 about the same, both as predicted.

The screen's figures were 1.097 and 1.075. The predicted regression toward the middle didn't
appear in this run. One run can't say why.

**The occupancy pass (diagnostic only, under the limits the registration set).**
- **The traces are uninterpreted-safe:** for every plan and placement, neither traced mode
  perturbed throughput (each interval includes 0).
- **The workers were busy:** in all three plans they spent about 96% of their time decoding and
  about 3% idle, so no plan was starved for blocks.
- **SMT siblings:** packing threads onto SMT siblings cost throughput in every plan:
  - 4 × 2 at 8: −20.7% (−21.9 to −19.5);
  - 3 × 3 at 10: −19.4% (−20.9 to −17.9);
  - 3 × 4 at 12: −12.4% (−14.3 to −10.4).
- **No table-count mechanism is claimed.** No two plans share a team size, and the window
  differs, so the registration defers that claim (D030).
