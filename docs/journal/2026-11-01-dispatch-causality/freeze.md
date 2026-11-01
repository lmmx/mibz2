# Dispatch causality: the freeze, and the preparation VM's dry run (untimed)

**Status:** frozen and dry-run. The real i9 capture waits for the maintainer's
authorisation. Nothing here is a finding.

## The freeze

| | |
|---|---|
| Protocol | `docs/journal/2026-11-01-dispatch-causality-protocol.md` |
| Branch | `research/dispatch-causality` |
| Pin (`bench/dispatch-pin`) | `382fde1cfb0bfb1f2145e35cfb11fbc3e68fbcb6` |
| Source base (C6) | `45b366d1455a487f1655def689ebd8c87a31f135` |
| Toolchain | rustc 1.97.0 (2d8144b78 2026-07-07), `--locked`, release, empty RUSTFLAGS |
| Frozen sources | 25 files, hashed in `frozen.json` (`src/*.rs`, the two examples, the dispatch and attribution scripts, the two trace tests, `.just/bench.just`, `Cargo.toml`, `Cargo.lock`) |
| Corpora | the walk trial's frozen files, byte-identical; `wikidata-20261026` primary (`cf13f439…`, 207,000,624 decoded bytes), `records` and `random` for correctness |
| Schedule | 3 w = 4 pairs, 3 w = 1 pairs, expansion to 10 w = 4 pairs on the quality checks alone |
| Thresholds | materiality 5% of aggregate worker time, repeatability 80% of runs, coverage 90%, perturbation 3%, alignment 20 µs, scheduler state known 99% |
| Mark buffers | 32,768 marks a thread, preallocated |

**The runner refuses to start** in any of these cases:
- the tree is dirty;
- HEAD differs from the pin outside `bench/dispatch-pin` and this directory;
- a frozen source's hash differs;
- a corpus's hash differs;
- the label has been used before.

**The commits:**
- `21cc812`: the implementation;
- `0f7e3ac`: two fixes the first dry run found (below);
- `382fde1`: the dry run forces the expansion;
- the freeze commits after each.

## What the first dry run found, and the fixes

1. **The analysis globbed its run files as `w4-r*.json`.** That also matches the marks'
   sidecars (`w4-r00.tsv.meta.json`), so the quality step crashed with
   `KeyError: 'complete'`.
   - **Fix:** the globs are exact, `w[14]-r[0-9][0-9].json`.
2. **The runner read a crashed quality step as a failed check.** It then moved to tier B
   and stopped the expansion: it failed open, into a smaller capture.
   - **Fix:** exit 0 means every check holds and 3 means a check fails. Any other exit
     stops the run.
3. **A dry run whose quality checks fail never ran the expansion.** The VM's noise makes
   that the likely case.
   - **Fix:** a dry run now forces the expansion, logged as forced: "a real run stops
     here".
   - A real run never forces it. The code path is `[[ $dry == 1 ]]` only.

## The second dry run (`preparation-vm`, untimed; `dry-run-preparation-vm/`)

On this 4-vCPU VM, as root, the dry run holds one w = 4 pair at tier A, then one each at
tier B for w = 4 and w = 1, then one forced expansion pair. In order:

| Step | Result |
|---|---|
| Sources | 25 match the frozen hashes |
| Tests | `trace_dispatch`, `trace_phases` and the instrument's tests pass |
| C6 | **same code: 104 `mibz2` functions, 32,298 instructions,** identical to the base |
| Corpora | 3 match their frozen hashes and decode to them at w = 1 and 4; every capture's output hash checked |
| Allocation | CPUs 0–3 (the VM has no SMT; the i9 gets CPUs 1–4, siblings 11–14 idle) |
| Tracepoints | 8 supported, 0 unavailable |
| Tier choice | tier A's single pair perturbed CPU-s/GB by +14%, so the run moved to tier B (the path exercised) |
| Expansion | forced, as a dry run; one pair |
| Summary, archive | written; `perf.data` deleted after scripting; 3.2 MB `tar.gz` |

**The per-run quality checks** (both w = 4 runs at tier B, and tier A's):
- every thread starts and ends, with 0 marks dropped;
- every TID was seen by the kernel;
- scheduler state known for 100% of queued-work nondecoding time;
- one sleep a run had no wake recorded, so it is counted as unresolved;
- alignment uncertainty 0.57–0.59 µs, against the 20 µs limit;
- coverage 94.3% and 94.5%, against 90%.

**The futex mapping** (tier A's run) shows each address where the protocol expects it:

| Phase | Waits | Addresses | Dominant share |
|---|---|---|---|
| receiver mutex | 100 | 1 | 100% |
| reader's condvar wait | 272 | 2 | 99.6%, the same address as the scanner's |
| scanner's condvar wait | 305 | 2 | 96.1%, the same address (the shared `cv`) |
| state lock | 6 | 1 | 100% |
| channel | 161 | 5 | per-thread parkers, as std's channel parks each thread on its own address |

**Not interpreted:**
- **The VM's perturbation** reads 0.85–1.18 between single pairs.
- **A 5-repetition split of the overhead** (medians, not frozen) gives:

  | Configuration | Wall | CPU |
  |---|---|---|
  | marks alone | −4.1% | −2.1% |
  | tier B | +0.6% | +2.5% |
  | tier A | +5.1% | +0.9% |

  Marks alone measuring faster than uninstrumented means the VM's noise is about ±4%, so
  the i9's pairs decide the tier and the gate.
- **The shares of this VM's runs** (receiver obstruction 1.2–2.5%, the class ranking) are
  a check that the arithmetic runs. They say nothing about the i9.

## Limitations the maintainer should know before authorising

1. **Perturbation may fail on the i9 at 3%.**
   - If tier B also moves throughput or CPU-s/GB by more than 3% over the first three
     w = 4 pairs, the registered rules say:
     - the run stops at three, with no expansion;
     - the decision is reported as provisional, naming perturbation.
   - Nothing lowers the bar.
   - The futex tracepoints are the costliest part on the VM.
2. **`runnable.mutex_wait` is not counted as receiver obstruction (D1).** This is the time
   a worker has been woken from the receiver mutex but isn't yet running.
   - By the registered classification, it is scheduling (D2).
   - It appears in `result.json`'s `classes_s`, beside the D1 share, but the ranked groups
     fold it into `runnable`.
   - On the VM it was the largest single class. Whether it should count toward MPMC
     promotion is a question for the maintainer, not one this freeze answers.
3. **The reader's path** is split into segments only when the reader is asleep in its
   wait at `Ready` and wakes after it.
   - For 21–26% of the blocks the reader takes next on the VM, the chain wake → run →
     relock → take is not ordered after `Ready`. That fits the reader having already been
     woken by another notification on the condition variable it shares with the scanner,
     but this freeze has not verified it.
   - These are counted as `unresolved` in the reader's table. They are reported, not
     decisional.
4. **The w = 1 reference's coverage is 77%** on the VM. With one worker, queued-work
   nondecoding time is small (31 ms), and much of it falls inside the scanner's send
   bracket.
   - It is a reference, not one of the gated w = 4 runs.
5. **The futex addresses** are identified by the phase they occur in, not by symbol.
   Std's channel parks per thread.
6. **CPU occupancy is seen only on the allocated CPUs** (perf `-C`). Our threads are pinned
   there.
7. **At trace level ≥ 1, the `RecvDone` mark inside the receiver guard lengthens the hold
   by one mark's time.** Level 0 and default builds are unchanged, by C6.
8. **This was validated on the VM only.** The i9's kernel, perf and PMU-independent
   tracepoints are probed at the start of the real run, and a missing essential one
   stops it.

## The command (after authorisation only)

On the i9, in the mibz2 checkout:

```
git fetch origin research/dispatch-causality && git switch research/dispatch-causality
just bench-dispatch i9-10900K
```

- **Privileges:** `sudo` once, prompted at the tracepoint step and kept alive for the
  run. Needed for perf's tracepoints and `-C`.
- **Data:** the walk trial's corpora in `target/walk-decoder/work/corpora`, hash-checked.
  `DISPATCH_CORPORA` points elsewhere.
- **The archive:** `target/dispatch-results/i9-10900K.tar.gz`. It holds:
  - the run log, the binaries' hashes, C6's report, the manifest, the allocation and a
    numbers-only environment;
  - `tracepoints.tsv`, `pairs.tsv` and `quality-A.json` / `quality.json`;
  - for each instrumented run, its marks with their sidecar and its `perf script` text
    (gzipped), and its analysis JSON;
  - `tier-A/`, if tier B was chosen;
  - `result.json`.
