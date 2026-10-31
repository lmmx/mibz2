# The unrolled room walk's full-decoder trial: amendment 1 (before any timing)

**Status:**
- Authorised by the maintainer after reviewing the frozen preparation (head `c647c8d`,
  pin `30018de`; `freeze.md`; the dry-run record on `docs/walk-unroll-decoder-records`
  `b36fb03`).
- Committed before any timing.
- The registered protocol, `freeze.md` and the original frozen records stay as they were.
  This amendment adds to them.
- **Unchanged:** the kernels, corpora, corpus hashes, thresholds, correctness checks,
  same-binary A/B/A′ comparisons, schedule and stop rules.

## 1. W4 (c): accepted as a documented integration-level code-generation difference

**What is accepted:** this exact signature, recorded in `frozen.json` as `w4_review`.
The runner compares it with each timed binary's audit, and refuses timing if they differ.

```
inlining_differences: [
  "<mibz2::bwt::SegScratch>::assemble",
  "<mibz2::bwt::Segmented as mibz2::bwt::InverseBwt>::invert",
  "mibz2::decode_block::<mibz2::bwt::Segmented>"
]
```

**What it means:**
- **A (`Segmented`):** the worker calls `decode_block::<Segmented>` out of line. The
  kernel's `invert` is inlined into it, and `SegScratch::assemble` is called out of line.
- **B (`SegmentedUnrolled`):** `decode_block` is inlined into the worker, and its `invert`
  is out of line with the assembly inlined.
- W1–W4 (b) pass. The source-level stages, and the build, MTF, RLE1 and CRC functions
  each worker reaches, are the same.

**Disclosure (the maintainer's, and binding on how results are read):**
- The difference isn't assumed to cost only a few calls a block. Inlining also changes
  register allocation, optimisation across function boundaries, and instruction-cache
  and front-end behaviour.
- So it is a performance confounder, though not a correctness problem.
- A B/A result measures the integrated decoder as compiled, walk and layout together. It
  is not attributed to the walk algorithm alone.
- The kernels and other performance-critical code are **not** changed to make the
  disassembly match.

## 2. The toolchain: rustc 1.97.0, accepted

This is the toolchain of the registered stage PASS (CI run 41222735026), chosen before
any build for this trial.

## 3. The audit-tool corrections, accepted, with their limits

- **W4 (b)** compares the call-graph closure from each arm's worker thread body, not
  same-named functions. The first version reported inlining as a false stage mismatch.
- **`SegScratch::assemble` is counted with the walk** (the change under test), because
  `walk_segments_room` and its twin both end with it. It stays in the W4 (c) signature.
- **Limits:**
  - The call graph is read from direct `call` instructions in `objdump` output.
  - Indirect calls (through registers or the GOT, such as `memcpy`) and tail jumps aren't
    followed.
  - Inlined code is credited to the function it is inlined into.
  - So W4 (b) shows that the same out-of-line stage functions are reached. It doesn't
    show that inlined code is identical.

## 4. `zeros`: kept, its description corrected

- The corpus and its hash are unchanged (`07dca4a334909c5f…`), and its role is as frozen.
- **Description, corrected:** it is a **short-block fallback control**. bzip2's initial
  run-length coding shrinks its block below `MIN_SEGMENTED`, so both arms run the plain
  `walk`. It doesn't exercise degenerate room-walk segments, as protocol §5 item 4 said.
- `frozen.json` records the corrected description in `corpus_notes`.

## 5. Environment snapshots: numbers only

- The runner's `ps` snapshot now records `pid,psr,pcpu`, with no process names or
  command lines.
- CPU affinity and workload evidence are unchanged: per-thread `Cpus_allowed_list`,
  `/proc/stat`, pressure and the allocation.
- The earlier dry-run record's process name is left in its history, as the maintainer
  directed.

## 6. What this authorises

One three-job exploratory CI run at w = 1, 2:
- 3 jobs on `ubuntu-24.04`, at this amendment's pinned commit;
- the frozen thresholds and rules.

It is reported, then stops for review.

**Not authorised:**
- the i9 trial;
- reruns;
- production defaults, merging, or combining with P74-C.

**Reading CI** (per the maintainer):
- CI is exploratory. Its 2 physical cores are a different configuration from the i9's.
- A CI PASS does not establish a four-core i9 improvement.
- A CI negative does not overrule the i9 primary gate.
