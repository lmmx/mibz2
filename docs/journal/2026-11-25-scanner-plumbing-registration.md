# bench-scanner-plumbing: registration

*25 November 2026. Written and frozen before any data, on `bench/scanner-plumbing` (from master
16aaee3b). The recipe is `just bench-scanner-plumbing` (`.just/bench.just`); the verdict is
`bench/scanner_plumbing.py`'s, tested by `bench/test_scanner_plumbing.py`. The recipe, the rule
and the verdict script are bench-open-relay8's (#707, `docs/journal/2026-11-21-open-relay8-registration.md`),
themselves bench-open-diet's (#693), bench-open-r4's (#680), bench-open-lanes's (#676) and
bench-open-vec16's (#669), with the same step, checks and median peak RSS report. What differs:
the head is a parameter, the counts, the judged count, a workers check, and the recipe fails soft
as bench-character-stage2's does.*

**The decision it settles:** whether a change that takes work off the decoder's serial scanner
thread merges. It is an ordinary merge under PERFORMANCE-POLICY.md. CI's bench reaches 4 threads,
where the workers bind and a scanner cut cannot show; this run measures where the scanner binds,
on the i9 (C5). CI's paired bench still judges every corpus and thread count it covers, as usual.
This run does not replace it.

## Which head a run measures

The recipe takes the head as positional arguments, `HEAD_REF` and `HEAD_SHA`, so that one frozen
recipe measures each of these without being edited:

- **(a) #846's symbol-buffer fix,** `perf/scanner-plumbing` (#846) once it is split to carry that
  fix alone, with no headroom. A block's `syms` buffer is kept at its high-water length, so the
  next block does not zero-fill it again (`src/block.rs`).
- **(b) the window fix,** `perf/scanner-window`, a headroom-free redesign of #846's first change.
  At a 4 MiB chunk boundary, `ChannelSource::view` (`src/parallel.rs`) no longer copies the whole
  new chunk onto the window.
- **a later combination** of the two, on a branch of its own.

The rules for the head and base:
- **`HEAD_REF`** is a branch on origin (default `perf/scanner-plumbing`).
- **`HEAD_SHA`** is a full SHA, or empty. Empty means the branch's tip when the run starts. For
  an evidence run, the queue entry pins it: `args=<branch>,<sha>`.
- **The base** is always the head's merge-base with `origin/master`, computed by the recipe. So
  (b) branched from a master that already has (a) measures (b) alone, and a combination branched
  from master measures both.
- **The record:** each run's bundle records what it measured, in `builds.txt` (`base`, `head`,
  `head_ref`, the SHA given if any, whether the head is on the branch, and master's tip), and in
  `head-commits.txt` and `src-head-against-base.diff`.
- **The Coordinator's LOG** for each run names the head it measures and which change that is.

**Each head is its own decision.** A run of another head is not a re-run. A verdict is never
re-run to change it. A VOID run may be repeated once, with the reason logged.

When this was written, `perf/scanner-plumbing` was still at #846's first head, 36084e6e. That head
carries both fixes and the 2 MiB headroom that failed CI's policy (onebyte +8% at 4 threads; peak
RSS +10–20%). The DRY below ran on that head, because it was the only one on origin. That checks
the wiring only. No evidence run is registered for 36084e6e.

## The contrast, checked by the recipe

Between the base and the head:
- **May differ:** `src/`, `tests/` and `examples/` (#846 touches all three).
- **Must be identical:** `examples/ab.rs` (the harness), `Cargo.toml`, `Cargo.lock`, `build.rs`
  and `rust-toolchain.toml`. If any of them differs, the run stops.
- **Must differ at all:** the head must not already be on master. An empty contrast stops the run.

Both builds use rustc 1.99.1. The head's harness drives the run, and the base runs as
`mibz2@build-base`.

## The hypothesis and the mechanism

The decoder is two stations (journal `2026-11-24-algorithm-level.md` §57): one serial scanner
thread, which finds blocks and Huffman-decodes them, feeding k workers. On this corpus and on the
i9, the scan profile (§57) and stage 1 (§§66, 71) put the knee at k\* ≈ 7.2–7.7 workers. Below
it, the workers bind and the wall goes as a/k. Above it, the scanner binds and the wall is about
its own demand, 0.18–0.19 s on this file.

Neon's mining (§62), checked in master's code, found two costs on the scanner's thread that are
not decode work:
1. **The window copy:** `ChannelSource::view` copies each new 4 MiB chunk onto a window that
   still holds bytes. About 10 ms at 8 workers, 5.4% of the scanner's demand.
2. **The re-zeroing:** pooled symbol buffers are zero-filled again on every take. About 5 ms,
   2.7–3.0%.

Together that is about 15 ms (8%). A cut in the scanner's demand moves the wall only where the
scanner binds, and only until the workers bind again (bottleneck migration, §§61, 62, 69).

**The cells, and production's workers at each.** Production's decoder on the i9, unpinned, from
master's `src/parallel.rs` at 16aaee3b:

| threads | workers | the code |
|---|---|---|
| 6 | 6, lazy | `UNPINNED_AT_SIX`, line 3333; `unpinned_cell`, line 3369 |
| 8 | 8, lazy | `plan_with`, line 3493, gives `Workers(8)`; no unpinned trim at 8; `LAZY_FROM_THREADS`, line 2763 |
| 9 | 9, lazy | `plan_with` gives `Workers(9)`; no trim |
| 10 | 7, lazy | trimmed to `TRIMMED_WORKERS`: `unpinned_trims`, line 3464, holds from 10 threads; `trimmed`, line 3383 |
| 12 | 7, lazy | as at 10 |

The block-reader-on-wormhole bundle's records confirm this at 8 and 10: `observed_threads` shows 8
and 7 `mibz2-N` workers. The verdict reports each cell's median worker count, base and head. A
difference between them is a VOID: the head would have changed the plan, which this run does not
measure.

**So, on the i9, production runs more workers than the knee at 9 threads alone.** At 10 and 12
threads it runs 7 workers, below the knee, where a cut in the scanner's demand is predicted to be
masked. At 8 the workers are within 0.3 ms of the scanner (§69).

The `mibz2-workers-lazy` diagnostic runs exactly 8, 9 and 10 lazy workers. Its 10 is the
decoder's plan on a 10-core machine without the i9's trim. That cell is where §69's "about 7.5%
at 10" was priced (10 workers, not production's 10 threads on the i9).

## The prediction (stated before the data; it is not the rule)

These are the bottleneck-migration predictions, from §§62 and 69 on stage 1's numbers. They are
conditional: the instrumented stage 1 is not this uninstrumented run, and the lower bound is 4–5%
loose at 8–11 workers (§71).

- **Production, 6 threads (6 workers):** about 1.00. The workers bind.
- **Production, 8 threads (8 workers):** little gain, about 1.00, a few tenths of a percent at
  most. The workers are within 0.3 ms of the scanner.
- **Production, 9 threads (9 workers):** the scanner binds. The workers' demand there is about
  8/9 of their demand at 8, about 0.173 s, below a scanner cut by 15 ms (about 0.179 s). So
  nearly the whole cut shows:
  - about 0.974 for (a), the 5 ms symbol-buffer fix;
  - about 0.95 for (b), the 10 ms window fix;
  - about 0.92–0.93 for the two together.
- **Production, 10 and 12 threads (7 workers):** about 1.00. The workers bind.
- **Diagnostic, `mibz2-workers-lazy`:** about 1.00 at 8 workers. At 9 and 10 workers, as
  production's 9 above, with §69's "about 7.5% at 10" for both fixes together.

A run that contradicts this pattern does not change the verdict. It is reported as evidence
against the two-station reading. For example, a gain at 10 or 12 threads, or none at 9 and 10
workers.

## The rule

bench-open-relay8's rule (R4's), unchanged in form. Only the counts change, and the judged count
moves to where the change's mechanism acts (PERFORMANCE-POLICY §4a, item 2).

**A count's step** is the 16 rounds' paired wall ratios, head over base: their geometric mean and
its 95% interval (Student's t on the logs, 15 degrees of freedom). A step is FASTER if the
interval lies below 1, SLOWER if it lies above 1, and INCONCLUSIVE otherwise.

- **ADOPTS:** all three hold:
  - the step at **9 threads** (judged) is FASTER;
  - none of **6, 8, 10 and 12 threads** (supporting) is SLOWER;
  - at each of 6, 8, 9, 10 and 12 threads, the median peak RSS over the rounds, head over base,
    is at most 1.10.
- **HELD ON MEMORY:** as ADOPTS, but with peak RSS above 1.10 at some count. The Coordinator
  weighs the growth under PERFORMANCE-POLICY rule 4.
  - **Rule 4, accepted by rule,** applies when all of these hold:
    - a workstation run on the target machine (C5, paired) shows a primary wall-time gain of at
      least 5%, beyond noise, on the corpus and at the thread counts where memory grows;
    - the growth, in percent, is at most twice that gain at each of those thread counts;
    - the absolute growth stays at most 64 MB of peak RSS per process;
    - no other corpus grows by more than 10%.
  - **Otherwise:** growth beyond 10% needs the PR's own justification for the target machine, and
    the Coordinator weighs it.
  - **What the verdict prints** at each count over the gate: whether the first three conditions
    hold there, with the gain, the growth in percent and in MB. It decides nothing by itself. The
    fourth condition, other corpora, is CI's bench's to show.
- **NOT ADOPTED:** any other outcome: 9 threads not FASTER, or 6, 8, 10 or 12 SLOWER.
- **Supporting counts:** 6, 8, 10 and 12 threads are reported with their intervals. Only a loss
  there counts against adoption.
- **VOID**, with no verdict, if any of these holds:
  - the machine is not the i9's profile (an i9-10900K with 10 cores, 20 CPUs and no CPU quota);
  - any run is unsuccessful, or has a non-finite or non-positive wall, CPU or peak RSS;
  - any run is pinned or beside a spinner;
  - any record is duplicated, unexpected or missing;
  - the corpus is not `wikidata-x4`;
  - any output is not 838,800,784 bytes, or the outputs' hashes are not all one;
  - a record has no `observed_threads`, or a count's median workers differ between base and head.
- **Reported beside each step, deciding nothing:**
  - the workers;
  - CPU time;
  - the median peak RSS at the base and the head, in MiB, with their difference.
- **The diagnostic, `mibz2-workers-lazy`:** at 8, 9 and 10 workers, with the same checks and
  steps. It decides nothing.

**Why 9 and not 10 is judged.** The earlier open-\* registrations judged 10 threads, production's
default on the i9, because their changes were in the workers' kernel, which runs at every count.
This change is on the scanner. Production's 10-thread cell runs 7 workers, below the knee, where
the mechanism predicts nothing. Judging there would test a cell the change cannot move. 9 threads
is production's own cell with the most workers above the knee, and production runs it as asked.
This choice is made before the data and from the code, not from a result.

**What each outcome does:**
- **ADOPTS:** the measured head merges once its CI is green and CI's paired bench passes
  PERFORMANCE-POLICY §2 on its own corpora and counts.
- **HELD ON MEMORY:** the Coordinator weighs it under rule 4, with the trade the verdict reports.
- **NOT ADOPTED:**
  - this run gives that head no gain;
  - it merges only if CI's bench shows a primary gain on its own;
  - the result is written up as a negative result (journal, PR comment).
  - (a) alone is the weakest case: a predicted 2.6% against intervals of about ±1.2–1.3% at 8
    and 10 threads in the last comparable run (block-reader-on-wormhole,
    `run-20261122T093349Z`). A NOT ADOPTED for (a) alone does not rule out (a) inside a
    combination. A combined head is its own run.
- **VOID:** repeated once, with the reason logged.

## The run

- **The machine:** the maintainer's i9-10900K (10 cores, 20 CPUs, no CPU quota), under C5. Any
  other machine voids the run.
- **The corpus:** `corpora/wikidata.bz2`, the whole bench corpus (SHA-256 `82f65ace…`, 431 blocks,
  209.7 MB decoded), checked by its digest. Four copies end to end make the timed file
  (`wikidata-x4`), as the i9 recipes do, so that each run is long against process start.
- **The cells:** `ab run --arms mibz2@build-base,mibz2`, production's decoder at each build,
  unpinned on every CPU (no `--spread`, no spinner), at 6, 8, 9, 10 and 12 threads.
- **Rounds:** 16. Each round runs both arms at each count, interleaved, with the arms' order
  rotated, and a 2 s settle gate before each run.
- **The diagnostic:** `mibz2-workers-lazy` at each build, at 8, 9 and 10 explicit workers,
  16 rounds, the same settle.
- **Fail-soft:**
  - each phase's status goes to `phases.tsv`: resolve, the two builds, inputs, machine, the
    timed run, the verdict, the diagnostic and its report;
  - a phase whose prerequisite failed is SKIPPED, and the diagnostic still runs if the verdict
    is VOID;
  - the EXIT trap always bundles, and the recipe exits 1 if any phase failed;
  - the run leaves exactly one `target/*/*.tar.gz`, its bundle, so `just submit` finds one.
- **Cost:** 160 timed decodes and 96 diagnostic ones, at about 2.5–3 s each with the settle (the
  block-reader-on-wormhole run took 6.4 minutes for 128 decodes with its builds). That is about
  12–14 minutes timed, about 15–20 minutes with the two builds and the corpus. It is below
  PERFORMANCE-POLICY §4a's 30-minute line.

## Limits

- **The corpus:** Wikidata alone, the decoder's primary workload, where the knee was measured.
  `text` is not in this run, and CI's bench covers it at 1–4 threads. The knee depends on the
  corpus (§57).
- **The i9's default is barely touched:** production on the i9 runs 7 workers from 10 threads
  up. So even an ADOPTS here predicts little at the Python binding's default (20 threads) on this
  machine. The gain is at 9 threads, and on machines whose plan runs more workers than the knee
  (the diagnostic's 10). A cheaper scanner may argue for re-trimming to 8 or 9 workers (§62).
  That is a separate recalibration (RELEASING §5c), not decided here.
- **What 12 threads covers:** at 12, production runs 7 workers. This run does not test the
  scanner's SMT slowdown at 16 or more workers (§57).
- **A whole-decoder result, not its cause:** a gain at 9 shows the decoder got faster, not which
  cost it removed. The pattern across counts is consistent with the two-station reading or not;
  it does not prove it.
- **The parameterised head:** the recipe checks the contrast's files and the workers, but not
  what a head's diff does inside `src/`. The Coordinator reads `src-head-against-base.diff` in
  the bundle against the PR before accepting a verdict for it.
- **The head's branch:** with an empty `HEAD_SHA` the run measures whatever the branch's tip is
  when it starts. Evidence runs pin the SHA in the queue entry. A pinned SHA that is no longer
  on its branch (`head_in_ref no`) is fetched by its SHA and still measured, and the bundle says
  so.

## The DRY

The recipe was run untimed in the agents' container: `just bench-scanner-plumbing
perf/scanner-plumbing "" 1`. That is two rounds, no settle and the machine not checked. It is
never evidence. The result is recorded below, and its bundle is archived beside this file.
