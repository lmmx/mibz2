# Dispatch causality: the freeze, and the preparation VM's dry runs (untimed)

**Status:**
- Frozen with amendment 1 (validation only) and dry-run.
- The maintainer authorised the single real i9 capture once amendment 1's checks pass
  (report 112). They pass: see the last section.
- Nothing here is a finding.

## The freeze

| | |
|---|---|
| Protocol | `docs/journal/2026-11-01-dispatch-causality-protocol.md`, with `amendment-1.md` |
| Branch | `research/dispatch-causality` |
| Pin (`bench/dispatch-pin`) | `48f8cda32fe3580f87e49c6cd68242cf8a81c996` |
| Source base (C6) | `45b366d1455a487f1655def689ebd8c87a31f135` |
| Toolchain | rustc 1.97.0 (2d8144b78 2026-07-07), `--locked`, release, empty RUSTFLAGS |
| Frozen sources | 29 files, hashed in `frozen.json` (`src/*.rs`, the two examples, the dispatch and attribution scripts, the analysis's tests and fixture, the two trace tests, `.just/bench.just`, `Cargo.toml`, `Cargo.lock`) |
| Binaries (VM build) | `occupancy` `2d28fca5…`, `attribution_decode` `45463aa9…`: unchanged by amendment 1 |
| Corpora | the walk trial's frozen files, byte-identical; `wikidata-20261026` primary (`cf13f439…`, 207,000,624 decoded bytes), `records` and `random` for correctness |
| Schedule | 3 w = 4 pairs, 3 w = 1 pairs, expansion to 10 w = 4 pairs on the quality checks alone |
| Thresholds | materiality 5% of aggregate worker time, repeatability 80% of runs, coverage 90%, perturbation 3%, alignment 20 µs, scheduler state known 99%: all unchanged |
| Mark buffers | 32,768 marks a thread, preallocated |

**The runner refuses to start** in any of these cases:
- the tree is dirty;
- HEAD differs from the pin outside `bench/dispatch-pin` and this directory;
- a frozen source's hash differs;
- a corpus's hash differs;
- the label has been used before.

**On a real run, it stops before capturing** if:
- an essential tracepoint is missing;
- the kernel's scheduling stream is incomplete on the allocated CPUs (amendment 1's
  preflight probe).

## History

1. **`21cc812`: the implementation.** The first dry run found:
   - the analysis's run globs caught the marks' sidecars;
   - the runner read a crashed quality step as a failed check, failing open into tier B.

   Both were fixed in `0f7e3ac`.
2. **`382fde1`: dry runs force the expansion,** logged as forced. A real run never does.
3. **The researcher's review (reports 111–112)** found that the summary omitted the
   scheduler checks the expansion required. Checking whether trace loss could pass
   silently found three more gaps:
   - perf's loss records were invisible to the script;
   - events whose tid perf could not resolve were dropped;
   - the VM's kernel emits no switches out of idle on CPUs 1–3, which turned running time
     into "runnable".

   All are corrected in **amendment 1, `48f8cda`**, with 24 tests. The details are in
   `amendment-1.md`.

## The maintainer's decisions (report 112), recorded

- **(a)** The registered 5% receiver-obstruction rule is unchanged.
  - Runnable time in the receiver mutex's wait phase is reported apart, as
    `receiver_associated_exploratory`. It is never added to the obstruction.
- **(b)** There is no third capture tier. If tier B cannot meet 3%, the registered
  provisional outcome stands.
- **(c)** No condition-variable split is pre-registered or implemented. The reader path
  stays observational.

## The amended dry run (`preparation-vm`, untimed; `dry-run-preparation-vm/`)

On the 4-vCPU VM, as root, the steps in order:

| Step | Result |
|---|---|
| Sources | 29 match the frozen hashes |
| Tests | `trace_dispatch`, `trace_phases`, the instrument's, and **the analysis's 24** pass |
| C6 | **same code: 104 `mibz2` functions, 32,298 instructions,** identical to the base |
| Corpora | 3 match their frozen hashes and decode to them at w = 1 and 4; every capture's output hash checked |
| Allocation | CPUs 0–3 (the VM has no SMT; the i9 gets CPUs 1–4, siblings 11–14 idle) |
| **Stream probe** | **incomplete:** switches into idle on CPUs 1, 2, 3 = 33, 18, 15, and out = 0, 0, 0; CPU 0 paired (3, 3). A real run stops here; the dry run continues |
| Tracepoints | 8 supported, 0 unavailable |
| Tier choice | tier A's single pair over 3%, so tier B (the path exercised) |
| Expansion | forced, as a dry run |
| Summary | **`outcome: invalid evidence`**. Failing checks: `trace_complete_all`, `sched_coverage_all`, `coverage_all` and `perturbation`. Each run's checks and trace problems are listed |
| Archive | `perf.data` deleted after scripting; each run keeps its `perf.stats.txt` and `perf.err` |

**Per run** (the w = 4 runs at tier B):
- 0 loss records, 0 lost events, 0 unreadable lines;
- 6 unresolved-tid `sched_switch` events, which are kept: our threads' exit switches;
- 306 and 290 missed switch-ins, now unknown and not runnable;
- scheduler coverage 0.929 and 0.909, below 0.99;
- trace unsound for idle pairing on CPUs 1–3.

The figures are not interpreted. Receiver obstruction reads 1.6–1.9% and the exploratory
associated share 3.5–4.7%, but on this kernel neither means anything.

## Limitations the maintainer should know

1. **The i9's kernel is the first thing tested.** If its scheduling stream fails the probe
   (switches into and out of idle unpaired, or anything lost), the run stops before any
   capture. It records why in `stream-probe.json`.
   - The VM's guest kernel fails it. A standard distribution kernel is expected to pass,
     but it hasn't been verified on the i9.
2. **Perturbation may fail on the i9 at 3%.** By decision (b), the outcome is then
   provisional, the run stops at three, and nothing is added or lowered.
3. **The reader's path** is split into segments only when the reader is asleep in its
   wait at `Ready` and wakes after it.
   - On the VM, 21–26% of the reader's next blocks did not fit that order. This fits the
     shared condition variable having already woken the reader, but it hasn't been
     verified.
   - The reader path is observational, by decision (c).
4. **The w = 1 reference's coverage** is low on the VM, at 0.78. It has little queued-work
   time, much of it inside the send bracket. It is a reference, not gated.
5. **The futex addresses** are identified by phase, not symbol. Std's channel parks per
   thread.
6. **CPU occupancy is seen only on the allocated CPUs** (perf `-C`). Our threads are pinned
   there.
7. **At trace level ≥ 1, the `RecvDone` mark inside the receiver guard lengthens the hold
   by one mark's time.** Level 0 and default builds are unchanged, by C6.

## The command (authorised: report 112)

On the i9, in the mibz2 checkout, with the desktop quiet:

```
git fetch origin research/dispatch-causality && git switch research/dispatch-causality && git pull --ff-only
cat bench/dispatch-pin    # 48f8cda32fe3580f87e49c6cd68242cf8a81c996
just bench-dispatch i9-10900K
```

- **Privileges:** `sudo` once, prompted at the tracepoint step and kept alive for the run.
- **Data:** the walk trial's corpora in `target/walk-decoder/work/corpora`, hash-checked.
- **The archive:** `target/dispatch-results/i9-10900K.tar.gz`. It holds:
  - the run log, the binaries' hashes, C6's report, the analysis's tests, the manifest,
    the allocation and a numbers-only environment;
  - `tracepoints.tsv`, `stream-probe.json`, `pairs.tsv`, and `quality-A.json` /
    `quality.json`;
  - for each instrumented run:
    - its marks with their sidecar;
    - its `perf script` text (gzipped), with `perf.stats.txt` and `perf.err`;
    - its analysis JSON;
  - `tier-A/`, if tier B was chosen;
  - `result.json`.
