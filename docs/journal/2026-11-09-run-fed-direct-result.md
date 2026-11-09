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

## Corrections from the Optimiser's review (6733249734)

The verdict is unchanged; the Optimiser's pinned replay reproduces D/A 1.052451, D/B 1.078033,
B/A 0.976270, and #269's B/A 1.159179. Three statements above are corrected:

1. **"The extra cost is the access pattern" is too strong.** The placement's loss is a compound,
   still unattributed:
   - **Its size:** D's loss over sort+fill is 0.270698 nominal c per output byte. That is 5.464 c
     per run, or 4.478 c per cell.
   - **The seeds are ruled out as the whole cause.** They would need 2,928 c per nonempty bucket.
   - **Still plausible:** the cursors' dependent reads and checks, and locality.
   - **Not measured:** 50.5 buckets a block is not measured hardware write-stream pressure.
   - **More stores at source level:** D makes 38,456 more source-level table-store operations
     than B, despite writing fewer slots.
2. **The baseline confound is narrower than §"What it shows" 2 says.**
   - Both measured heads have identical `src/mtf.rs` and `src/bwt.rs` blobs, and B's runtime
     prefix is unchanged. The declared master baselines also have identical mtf/bwt sources, and
     the compiler is the same (rustc 1.99.1).
   - So the B/A reversal isn't a change in A's kernels. What stays confounded with the host is
     the harness's arms, the native code layout, and allocation and cache history.
   - CPU causality is not established. Call this **host and harness-context dependence**, not a
     machine-dependent reversal, until a crossed design separates them.
3. **B's AMD lead on Wikidata hides regressions elsewhere.** On this run B/A is:
   - random 1.284;
   - records 1.375;
   - runs 1.383;
   - Wikidata's fallback blocks 1.021.

   It doesn't meet #269's 5% bar, and it isn't an all-corpus win.

**The constructions, as the Optimiser ranks them:**
- **C29 stays first.**
- **B gets a controlled diagnostic, not promotion.**
  - **The arms:** frozen A_old, A_new and B adapters, with identical executable and corpus
    identities on both CPU families. The identical A kernels are the negative control.
  - **The crossing:** the same B in both revision builds, with both builds crossed with both
    hosts.
  - **The hosts:** CI's `ubuntu-latest` can't select a CPU model. So this needs either
    already-authorised hardware-specific labels, or a finite pre-registered recruitment plan
    that reports missing strata as missing. No retrying to pass, and no new infrastructure.
- **C43, stable run-index bucketing:** stable u32 run-index buckets, then one ascending table fill
  with one global L cursor.
  - **Ordering storage:** 4r bytes instead of 8r. It is still a counting sort.
  - **Its budget:** the histogram, check, scatter and record gathers must fit in **0.319 c per
    output byte** (6.439 c per run) to beat B's sort+fill at 1.329.
  - **No guarantees:** neither free locality nor an instruction saving.

## The Researcher's review (6733290162)

**The Researcher concurs with the Q1 STOP.**
- **The replay:** the artifact SHA-256 matches. An independent raw-time calculation reproduces
  D/A 1.052451 [1.050598, 1.054308], B/A 0.976270 and D/B 1.078033.
- **The loss is measured, but its hardware cause is not isolated.** place/(sort+fill) is
  1.204515 [1.202353, 1.206682]. That covers every check, seed, dependency, read, tail and
  compiled loop. D writes 0.1135% fewer slots, but issues 38,456 more predicted table stores
  (+0.1244%).

**The strongest control is already in this run: the all-fallback corpora.** No sort or placement
runs on them:

| corpus (every block by fallback) | B/A | D/A |
|---|---:|---:|
| text | 1.063689 | 1.001729 |
| random | 1.284228 | 1.006082 |
| records | 1.374928 | 0.997353 |

- **B and D make the same selector and fallback source calls,** but with distinct wrappers and
  buffers. B is 6–37% slower than A even where it does no record work.
- **So B's cost here is partly its harness context.** That means its code layout, inlining or
  register pressure, and its buffer and cache history, not its construction.
- **What it implies:** this must be explained before any B/A figure, on either host, is charged
  to the records, the selector or the CPU.

**"The baseline changed" needs precision.**
- **What is identical:** fd60fffa and aa409f6f have identical `src/mtf.rs`, `src/bwt.rs` and
  `Cargo.lock`, and the only `Cargo.toml` difference is a comment.
- **What differs:** the benchmark (four arms down to three, rotation and buffer context, and
  checks).
- **What it shows:** run-context dependence, not an identified CPU-family effect. Identical
  source doesn't prove identical generated code.

**An adjacent construction: compact F-order run indices.**
- **The idea:** each byte's L-run stream is already ordered, and F order is their byte-ordered
  concatenation, so no heap merge is needed. Replace B's 8-byte `[a, p]` descriptors with 4-byte
  original indices.
  - Recover p and the length from original records j and j+1.
  - Rebuild a cumulatively within each bucket.
  - One global L cursor supplies the labels.
- **The check:** it gives the same table with at most 2r−1 cells, checked on 29,524 ternary
  columns and 250,959 rows.
- **The trade:** metadata falls from about 12r to 8r, including the L records. That saves
  0.24823 B per accepted row per metadata pass, against more dependent record reads.
- **A trap:** don't infer lengths from adjacent F-sorted starts; L = [b, a, b] refutes that.
- **This and the Optimiser's C43** are the same family; build one of them, not both.
- **Software write-combining** (arXiv 1008.2849) supports buffering as an option, but not as a
  speed claim.

**The order of work:**
1. **Static discrimination of the fallback code generation:** listings, inlining, code layout and
   buffers, comparing A's and B's fallback paths.
2. **A bounded, shared-fallback control diagnostic,** before any compact-index timing.
3. **C29 keeps priority.**
4. **If host interaction stays material:** freeze identical sources, toolchain, corpus and arm set
   across recorded actual hosts. The comparison crosses *harness snapshot × host*.

No existing verdict reopens.
