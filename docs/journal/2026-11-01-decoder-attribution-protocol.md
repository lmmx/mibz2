# The shipped decoder's one-to-four-worker resource attribution on the i9: protocol (draft for approval)

**Status:**
- A draft for the maintainer's approval. Nothing is implemented, built for timing or
  timed.
- On approval, §§2–6 are registered as written. Only the instrumentation (§7) is then
  built.
- The freeze adds the hashes and a dry run's figures before any i9 run. After that, only
  a pre-timing amendment can change this protocol.

**What it is:**
- An **observational diagnostic** of the shipped decoder (master `45b366d`, `Segmented`
  under `Plan::Workers`).
- There are no arms to compare, nothing to accept or reject, and no production change.
- P74-C and the unrolled walk stay closed. Nothing here touches their branches or
  verdicts.

## 1. The two questions, and what they start from (measured, the walk trial's arm A)

From `docs/walk-unroll-decoder-records`, `2026-11-01-walk-unroll-decoder-i9-10900K/`:

| wikidata-20261026, CPUs 1–4 | w = 1 | w = 4 |
|---|--:|--:|
| Throughput (MB/s) | 369 | 808 |
| CPU-s/GB | 2.99 | 3.90 (+30%) |
| Cores busy (MB/s × CPU-s/GB) | 1.11 | 3.15 |

**Q1:** why does CPU-s/GB rise 30% from w = 1 to w = 4?

**Q2:** why are only 3.15 of the four allocated cores busy at w = 4?

**Prior evidence on this machine** (`2026-10-28-workstation-scaling.md`; older code; 2–10
workers, other allocations). It sets expectations; it doesn't substitute for this run.
- **The demand-load L3 hit share** falls from 0.96 at 2 workers to 0.76 at 4.
- **In a contention proxy** (independent `phases.rs` copies), build and walk carry 84–93%
  of the added time.
- **The clock** is 4.9 GHz all-core (cycles/ref-cycles 1.326) at 6–10 threads; it wasn't
  read at 1–4.

## 2. Hypotheses (registered)

| | Hypothesis | Registered test (§5) |
|---|---|---|
| H1 | **Inversion memory amplification:** build and walk carry most of the added CPU, with more cycles stalled on memory per byte at the same instructions | R1, R2, R3 |
| H2 | **A serial front end:** the single scanner can't serve 1.5× today's w = 4 throughput, or workers wait on it | R4 |
| H3 | **Ordered admission:** the scanner waits for a slot while an earlier block is still decoding and later ones are done, so workers idle | R5 |
| H4 | **Startup and drain:** most of the unused capacity falls before the workers fill, or after the scanner's last block | R6 |
| H5 | **Scheduling:** w + 3 threads on 4 CPUs, so runnable workers wait for a CPU | R7 |
| H0 | **Clock:** a lower all-core clock at w = 4 explains the CPU-s/GB rise without more cycles | R2 |

## 3. The configuration

- **The binary under study:** master `45b366d`'s decoder, via `BzReader::parallel_with(File,
  w, 2w, Kernels::detect())`, the path `occupancy` and `bzcat` both use.
  - `Plan::for_threads(w)` must equal `Workers(w)`; this is asserted.
- **Two builds of the same source, one on each side of the tracing split:**
  - **instrumented:** `examples/occupancy` with feature `trace` (§7), for the timeline;
  - **uninstrumented:** `examples/bzcat` with default features, for counters and samples.
  - Its decoder functions are checked identical to master's by disassembly (§7 C6).
- **Placement:** the whole process (workers, scanner, feeder and reader) is pinned to CPUs
  1–4, four physical cores, at both w. Their siblings, 11–14, must be idle. This is the
  walk trial's allocation.
- **w ∈ {1, 4} only.** No sweep.
- **Corpora:** the walk trial's frozen files, re-hashed.
  - primary: `wikidata-20261026` (`cf13f439…`);
  - controls: `records` (`ff8b5f6e…`) and `random` (`10d444e6…`).
  - The decision rules read the primary. The controls are reported beside it.
- **The toolchain:** rustc 1.97.0, `--locked`, release profile, empty `RUSTFLAGS`.

## 4. Measurements: what each is, and what it can't say

**M1. The timeline** (`occupancy`, src/trace.rs):
- **Schedule:**
  - 10 rounds;
  - three modes in a rotated order: `off`, `timeline` (level 1) and `full` (level 2);
  - each run a fresh process;
  - back to back (no `--gap`; `2026-10-28-workstation-scaling.md` §4).
- **Per run:**
  - the share of the span each role spends in each state:
    - workers: decoding, idle by what the scanner is doing meanwhile, idle with a job
      queued (the handoff);
    - the scanner: input wait, slot wait (split by whether the earliest untaken block is
      still decoding or is ready), reading a block;
    - the reader: waiting, or consuming;
  - head-of-line holds: blocks ready but not taken while an earlier one decodes.
- **In `full`, per block:** elapsed, CPU time, run-queue wait and timeslices over
  `decode_block`, split by §7 C1 into MTF, build, walk and RLE1+CRC. Plus each thread's
  total CPU, every role's included (C2).
- **Can't say:**
  - which hardware resource a stalled thread waits on. CPU time counts memory stalls as
    running.
  - The states are wall-clock. One idle worker's time isn't one idle core, because other
    threads can fill it.

**M2. Counters** (`perf stat` on `bzcat`, process-scoped):
- **One group per pass**, so nothing is multiplexed. Every pass is checked for 100%
  coverage. 5 decodes per (corpus, w, group), back to back.

| Group | Events | Gives |
|---|---|---|
| G0 | cycles, ref-cycles, instructions; task-clock | cycles and instructions per output byte; the clock (cycles / ref-cycles × the TSC rate) |
| G1 | `cycle_activity.stalls_l3_miss`, `cycle_activity.stalls_mem_any`, `cycle_activity.stalls_total`, `mem_load_retired.l3_miss` | memory-stall cycles per byte; L3 misses per KB |
| G2 | `l1d_pend_miss.pending`, `l1d_pend_miss.pending_cycles`, `l1d_pend_miss.fb_full`, `mem_load_retired.l3_hit` | misses in flight per core (pending / pending_cycles); fill-buffer-full cycles; L3 hit share |
| G3 | `offcore_requests.all_data_rd`, `offcore_requests_outstanding.all_data_rd`, `idq_uops_not_delivered.core`, `int_misc.recovery_cycles` | queue occupancy per read past L2; front-end and recovery slots |
| G4 | `uops_issued.any`, `uops_retired.retire_slots` (with G0 and G3) | top-down level 1 (front end, bad speculation, retiring, back end) by the 4-slot formula |
| IMC | `uncore_imc/data_reads/`, `uncore_imc/data_writes/`, system-wide | DRAM bytes per output byte, each reading less a paired idle reading (`bench/imc.py`'s method) |

- **Each event is confirmed in `perf list` first.** A missing one is reported, never
  substituted.
- **Can't say:** which stage or thread incurs a count. That needs M3.

**M3. Samples** (`perf record` on `bzcat`): the sampled events are `cycles` and
`cycle_activity.stalls_l3_miss`, in separate passes, 3 decodes each.
- **They give** each thread role (by thread name: `mibz2-<i>`, `mibz2-scan`, the feeder,
  main) and each stage symbol's share.
- **The stage symbols in this binary:**
  - `bwt::build` (build);
  - `room_segments::<32, 1>` and `SegScratch::assemble` (walk);
  - `mtf::mtf_avx2` (MTF);
  - `rle::unrle_avx2` and `crc::*` (RLE1+CRC);
  - the scanner's Huffman functions.
- **Can't say:**
  - glue inlined into `decode_block`, which is reported as "other";
  - sample skid of a few instructions, which can't move cycles between out-of-line
    functions.
- **Overhead check:** the default sampling rate. Wall time is compared with G0 passes; a
  difference above 3% is reported.

**Stated limits:**
- These are observations of one machine.
- A per-thread figure from M3 multiplied by an M2 total assumes the clock is the same
  across threads at a given w. G0 reads the clock for the process only.

## 5. Decision rules (registered; on the primary, means over runs)

Each rule's conclusion is the decision it supports. Figures are per output byte, where
C_w is the decoder's CPU-s/GB at w workers.

- **R1 (H1, attribution).**
  - **Defined:** S_bw is the build+walk share of cycles (M3).
    ΔC_bw = S_bw,4 · C_4 − S_bw,1 · C_1, and ΔC = C_4 − C_1.
  - **If ΔC_bw / ΔC < 0.5:** H1 is **falsified**. The role or stage with the largest
    share of ΔC is reported as the owner to investigate next.
  - **Cross-check:** the timeline's per-phase CPU (M1 `full`) must give the same side
    of 0.5. If it doesn't, R1 is reported as unresolved.
- **R2 (H0 against H1, the clock).** With K_w as process cycles per byte (G0):
  - **K_4 / K_1 − 1 ≤ ⅓ · (C_4 / C_1 − 1):** the clock explains at least two-thirds of
    the rise. Memory amplification is **rejected** as the main cause of Q1.
  - **Otherwise:** the rise is in cycles, and R3 decides.
- **R3 (H1's signature).** H1 is **supported** only if all three hold:
  - R1 ≥ 0.5;
  - `stalls_l3_miss` per byte in build+walk (M3 × G1) rises by more than half the added
    build+walk cycles;
  - instructions per byte change by at most 5% (G0).
  - **Which inversion mechanism comes next depends on the signature:**
    - in-flight misses per core (G2 `pending` / `pending_cycles`) near Skylake's 10
      fill buffers, with `fb_full` rising: the misses in flight bind. That points to work
      reduction (N1).
    - in-flight misses well below that, with the L3 hit share (G2) falling: capacity
      binds. That points to compact or resident representations.
    - These branches are read descriptively. Their figures are reported, not
      thresholded.
- **R4 (H2, the scanner's ceiling).**
  - **Defined:** S_cap = output bytes / the scanner's time spent reading blocks (M1, at
    w = 4). It is an optimistic single-scanner ceiling.
  - **S_cap < 1.5 × X_4** (1.5 × today's w = 4 throughput): one scanner can't serve the
    target. **Front-end ownership** (#10, #8) becomes a prerequisite of any 1.5× plan.
  - **S_cap ≥ 2 × X_4, and workers idle while the scanner reads for under 10% of worker
    time:** the scanner is demoted.
  - **Otherwise:** it is reported as bounded but not binding.
- **R5 (H3, ordered admission).**
  - **Defined:** I_HOL is the worker idle time during which the scanner waits for a slot
    while the earliest untaken block is still decoding (M1).
  - **If I_HOL ≥ 25% of the workers' idle time at w = 4:** admission and ordering are
    promoted for investigation; the scanner's parallelism doesn't address them.
  - **Beside it:** the head-of-line hold time of ready blocks.
- **R6 (H4, startup and drain).**
  - **Defined:** startup runs from the span's start to the w-th block's `DecodeStart`;
    drain runs from the scanner's last `Issued` to the span's end.
  - **If the workers' idle time in those windows is ≥ 50% of their idle time:** sustained
    saturation is rejected as the explanation of Q2.
  - It also marks the corpus's length as a distortion: a release-length stream amortises
    both windows. That is a finding about the bench, not a target to engineer.
- **R7 (H5, scheduling).** Scheduling contention is promoted if, at w = 4, the
  workers' run-queue wait inside `decode_block` (M1 `full`) is either:
  - at least 10% of their decoding time; or
  - at least 25% of the unused capacity (below).
  - **Note:** CPU-s/GB excludes run-queue time, so R7 bears on Q2, not Q1.

**Q2's account:**
- The unused capacity, 4 × span − Σ thread CPU, is reported split into:
  - the startup and drain windows;
  - steady-state worker idle time by cause (scanner reading, slot wait with head of line,
    slot wait with the reader binding, input wait, handoff);
  - run-queue wait.
- **The largest owner at ≥ 25% is named.** If none reaches 25%, the result is reported
  as diffuse.

## 6. Validity checks (any failure: the affected figures are not interpreted)

- **Correctness:** every run's output hash equals the frozen decoded hash, and the plan
  is `Workers(w)`.
- **Placement:** every thread's CPU list lies inside 1–4. The siblings are idle (< 2%
  busy over the run).
- **Tracing perturbation:** the instrument's own rule is used. A traced mode whose paired
  ratio to `off` excludes 1.00 is "perturbing", and its figures are not interpreted.
  - **If `full` perturbs:** R1's cross-check and R7 are unresolved. R1 stands on M3.
  - **If `timeline` perturbs:** R4–R6 are unresolved.
  - The 95% interval of each traced mode against `off` is reported.
- **The run-queue counter** passes the instrument's oversubscription check.
- **Counters:** every group shows 100% coverage. A pass whose wall time differs from G0's
  by more than 3% is reported.
- **Consistency:** `off`-mode throughput and G0's CPU-s/GB are reported against the walk
  trial's arm A (§1). The driver differs (the `occupancy` and `bzcat` feeders and
  consumers), so this is reported, not gated.

## 7. The smallest instrumentation (built only after approval; `research/decoder-attribution`)

| | Change | Where | Production effect |
|---|---|---|---|
| C1 | A `WalkStart` mark between `build` and the walk in `Segmented::invert`; at level 2, CPU time and schedstat read at `InvertStart`, `WalkStart` and `InvertEnd` as they already are at `DecodeStart` and `DecodeEnd` | `src/bwt.rs`, `src/lib.rs`, `src/trace.rs`, all `#[cfg(feature = "trace")]` | none: compiled out of default builds |
| C2 | `ThreadStart` / `ThreadEnd` marks in `BzReader::parallel_with`'s feeder, so its CPU is in the trace | `src/read.rs`, `cfg(feature = "trace")` | none |
| C3 | In `occupancy`'s report: the startup and drain windows; the scanner's ceiling S_cap; per-role CPU and cores busy; per-phase CPU (C1); I_HOL; the unused-capacity account | `examples/occupancy.rs` | example only |
| C4 | `bench/imc.py`: `--exe` (the uninstrumented `bzcat`), `--cpus 1-4` (taskset); groups G1–G4 one per pass; a `record` step (M3) reporting by thread name and symbol | `bench/imc.py` | bench only |
| C5 | `bench/attribution_run.sh` and the recipe `just bench-attribution LABEL`. It does the following, then archives: the pinned checkout; rebuilding and hashing the corpora (the walk trial's specs, copied); the allocation and idle-sibling checks; a numbers-only environment snapshot; both builds; C6; M1, M2 and M3 | `bench/`, `.just/bench.just` | bench only |
| C6 | **Production unchanged, shown:** the default-feature decoder functions (`decode_block::<Segmented>`, `worker::<Segmented>`, `scanner`, the stage functions) disassemble identically at master and at this branch; the runner refuses otherwise | the runner | the check itself |

- **Tests:** the trace's existing tests, extended to C1 and C2. The occupancy instrument's
  completeness rule then covers the new marks: every `WalkStart` lies inside its
  `InvertStart`/`InvertEnd`.
- **A dry run** on the preparation VM, up to timing.

## 8. The proposed i9 command (after approval, implementation, freeze and the dry run)

```
just bench-attribution i9-10900K
```

- **Run by the maintainer** with the desktop quiet.
- **The counter passes need `sudo perf`,** as `just bench-imc` did. The recipe asks once.
- **Expected duration (estimate):** about 5 minutes of timeline and 10–15 minutes of
  counters and samples.
- **Output:** one archive.
- **Nothing else is run:** no worker sweep, variants or other corpora.
