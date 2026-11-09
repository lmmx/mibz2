# C31, the direct placement: the registered CI run

9 November 2026. This is the result of the run registered in
`docs/journal/2026-11-09-run-fed-direct.md` (§5), on #277.
- **The frozen head** is `aa409f6f2ebce9f41dc7902789f0ab8d6958b020`. The tested source is `d8a4c6f0`.
- **The freeze:** the Researcher's PROCEED (6732984192).
- **The safety review:** Guardian D058 lifted D055's timing HOLD on the freeze, green CI, and
  completed CI Miri of the empty-block test in both selections (job 128994104024: default 19/19
  and AVX2 4/4).
- **CI on the head:** all green. The aarch64 wheel's class B maturin download passed on one
  re-run.

**The run:** `sweep.yml` run [42567752634](https://github.com/lmmx/mibz2/actions/runs/42567752634),
`tool: run_fed_stage`, 15 rounds, `registered_head` aa409f6f.
- **The artifact:** `sweep-aa409f6f…`; its zip's SHA-256 is `00b0e008…3e94`.
- **The runner:** AMD EPYC 9V74, 4 CPUs, 3.6937 GHz reported; rustc 1.99.1. No PMU was exposed.
- **The judged copy:** `run-fed-1.json`, pinned to one CPU. The four contended copies are reported
  only.

**Validity:** `bench/run_fed_stage.py verify` gives **VALID; readings JUDGED**.

## The judged reading

- **Q1:** R_direct, D over A at the entry point on Wikidata, is **1.0525 (1.0506, 1.0543)**.
  - **STOP:** no saving of 5% or more on this runner. This is an operational pause for this
    implementation, not proof of no effect (PROTOCOL §11).
- **The screens:**
  - **The synthetic whole stage fails:** the worst is `runs`, at 1.2826 (1.2070, 1.3629).
  - **The fallback passes:** the worst is `random`, at 1.0061.
- **D against B** (exploratory): R_DB is 1.0780 (1.0727, 1.0834). The verifier reads it as
  **D NOT CHEAPER THAN B**.

## Where the cycles go (Wikidata, the pinned copy, nominal c/B)

| phase | A: column (master 6145b3a9) | B: records + sort (#269) | D: direct placement |
|---|---|---|---|
| front: MTF or emitter | 1.085 | 1.020 | 1.020 |
| sort | — | 0.319 | — |
| build, fill or place | 1.357 | 1.010 | **1.600** |
| fallback | 0.835 | 0.861 | 0.834 |
| entry point | 3.280 | 3.196 | 3.453 |

There were 384 blocks by records (79.8% of rows, r/n 0.0621 there) and 47 fallback blocks.
- D's predicted table stores (from the source's store loops, not machine code) are 0.1849 a row,
  writing 1.4761 slots a row against B's 1.4778.
- It touches 50.5 nonempty destination buckets a block.

**What it shows:**
1. **The direct placement costs more than the sort and the sequential fill it replaces.**
   - D's place is 1.600 c/B, against B's sort plus fill at 0.319 + 1.010 = 1.329: place/(sort+fill)
     is 1.2045 (1.2024, 1.2067).
   - The stores are about the same in number (1.476 against 1.478 slots a row).
   - So the extra cost is not store volume. It is the access pattern: writes spread over about 50
     buckets a block, and per-bucket cursors read back through the records. B's sort pays for
     itself by making the fill one sequential stream.
   - The exploratory budget agrees: δ_direct is +0.590 c/B.
     - On #269's budget, breaking even needed δ_direct ≤ −0.218.
     - On this run's own phases (K = −0.385, since B already beats A here), D would have broken
       even with δ_direct up to +0.385. +0.590 exceeds that too.
2. **B beat A on this runner.** R_stage, B over A, is **0.9763 (0.9713, 0.9813)** here.
   - On #269's registered run it was **1.1592 (1.1387, 1.1801)**, on an Intel Xeon 6973P-C at
     4.2 GHz. This run is on an AMD EPYC 9V74.
   - B's fill is 26% cheaper than A's build here (1.010 against 1.357), and its emitter is 6%
     cheaper than A's MTF (1.020 against 1.085).
   - **Caution:** A here is master 6145b3a9, not #269's base, and the runner differs. Both
     changed, so this run can't separate the two. This contrast is exploratory and does not
     reopen #269's verdict.
   - It does say that **the run-fed construction's cost relative to production depends strongly
     on the machine**, more than any arm effect measured on the line so far. The target-machine
     run (§11, C5) is the evidence that counts.
3. **The run-heavy blocks still win:** D/A is 0.3519 on one byte and 0.2805 on pattern.
   - `runs` (r/n 0.18) loses at 1.2826. There, place/fill is 1.5415.
   - So the placement's cost grows with the number of runs a bucket receives.

## What this means (PROTOCOL §11)

This arm, D, pauses. The line stays open. The constructions this points to:
1. **Keep the sort, and make the fill cheaper, not the sort's absence.** On this runner B already
   beats A. A sorted fill is the better shape; the direct placement's scattered writes are not.
2. **The runner dependence first.** One registered CI diagnostic that runs A and B on both CPU
   families in one dispatch would say whether B's lead here is the machine or the baseline. That
   is A at #269's base against A at master, on each family. Without it, no CI reading of this line
   transfers.
3. **Bucket-batched placement** (a sketch): a two-pass placement that groups each bucket's runs
   before writing, so it keeps the sort's locality without its full F array. Its predicted cost
   needs the Optimiser's model first.

Nothing here changes production. These go to the Optimiser and the Researcher.

**Record correction (the Researcher, 6732984192):** the registration's §11 says cases 35–36 break
the coupling alone. That is wrong for case 35, which also breaks `8·d_vector + d_scalar ≤ slots`.
Case 36 isolates the new lower bound. Both tests stay.
