# 2026-11-10: Share-nothing pricing: registration (diagnostic only)

**Status: registered as a draft, NOT FROZEN.**
- **The recipe:** `just bench-share-nothing` (`.just/share-nothing.just`). It refuses every run
  but a DRY one until `bench/share-nothing.frozen` names a commit.
- **Who freezes it:** only the Coordinator freezes and queues it, after the Researcher's review.

**What it is:** falsifier 2 of `2026-11-10-share-nothing.md` §4 (the Pathfinder's proposal, #145
6757754125, on #313), built as a C5 workstation recipe for the i9-10900K. Plain independent
workers (`mibz2-workers`, today's decoder) on Wikidata recompressed with `lbzip2 -2`, at 2 to 10
workers, read as CPU-seconds and wall. It prices §3's model before any small table is built.

**Diagnostic only** (RULES C1). No merge gate, acceptance rule or adoption decision reads its
outcome. Every record is kept: no run is excluded, replaced or retried because of its mode or its
value.

Everything below is fixed before any data: the corpora, the worker counts, the schedule and its
seed, the rounds, the analysis, the intervals, the mode split and the deadlines.

Labels: **read** (from the record), **model** (derived, not observed), **registered** (fixed here
before data).

## 1. The hypothesis

**The claim under test** (the share-nothing entry, §2 and §3):
- the CPU a decode costs with plain workers grows with the table a block, and not with
  cooperation's absence;
- so at about 0.8 MB of table a block (`lbzip2 -2`, 200 k rows × 4 B: the footprint of design C,
  0.75 MB), ten independent workers pay **1.10 to 1.44×** the CPU of four. That band is §2's
  -1 (0.4 MB) and -3 (1.2 MB) points, read from the workstation's 28 October run
  (`2026-10-28-workstation-scaling.md` §9), with the decoder of that date.

**§3's model** (model): ten workers at one thread's 400 MB/s at the all-core clock (about 0.94),
divided by that growth: 3.76 GB/s / G, so **2.7 to 3.4 GB/s** for G in 1.10 to 1.44, against about
1.5 for today's cooperative default on level-9 input.

**What this run measures** (registered):
- **The primary reading:** G₂, the growth in CPU-seconds from 4 to 10 workers on the -2 corpus,
  with today's decoder.
- **The in-run comparator:** G₉, the same growth on the -9 corpus. §2's ×3.50 at -9 came from an
  earlier decoder; this run measures it again beside G₂, in the same rounds.
- **The ratio of ratios:** R = G₂ / G₉. What a smaller table buys in CPU growth under today's
  decoder, with whatever the decoder changed since 28 October common to both.

## 2. The run

### The build and the corpora

- **The decoder:** `examples/ab.rs` built at HEAD (`cargo build --release --example ab`), its arm
  `mibz2-workers`: THREADS workers of a block each (`Plan::Workers`), each running the one-thread
  decoder. Nothing of the decoder changes.
- **The bench's corpus** (`corpora/wikidata.bz2`): built from its windows as every recipe builds
  it, checked against `bench/wikidata.json`'s SHA-256 (`82f65ace…`).
- **The content:** the corpus decoded by **bzip2** (libbzip2, not lbzip2 and not mibz2), checked
  against the registered SHA-256 of the decoded bytes, `3f075e30…cdaf` (209,700,196 bytes).
- **The two corpora:** the content recompressed by `lbzip2 -2 -n 4 -c` and `lbzip2 -9 -n 4 -c`.
  - **Pinned:** lbzip2's thread count is pinned to 4 (`-n 4`).
  - **Checked:** each level is compressed again at `-n 1`; whether the output differs is recorded
    (`thread_invariant`), never refused, since the pin fixes it.
  - **Decoded back:** each file is decoded by bzip2 and its SHA-256 must equal the content's, or
    the attempt is refused.
  - **Recorded:** lbzip2's version line, the flags, each file's size and SHA-256 and whether it
    matches the digests registered here for lbzip2 2.5 (another version is recorded, not refused).
- **Registered digests** (lbzip2 2.5, Ubuntu 2.5-2.3, on the dev VM, where `-n 1` and `-n 4` gave
  byte-identical files):

  | corpus | flags | bytes | SHA-256 |
  |---|---|--:|---|
  | wikidata-l2 | `-2 -n 4` | 15,837,840 | `d3bcb14179480e0d811fa2dfa73c20560808252a9199a2679ff14c84a9323045` |
  | wikidata-l9 | `-9 -n 4` | 11,855,421 | `3241376ca757f2fe786fcd07cf28012a6364eb523c821c10a9dc405fec120150` |

- **Why -9 recompressed and not the bench's corpus:** §2's -9 point was the same content
  recompressed by lbzip2, so both corpora here come from one compressor and differ in the block
  size alone (and what follows from it: block count, tables, run structure).

### The preflight

Read-only and passive, after the build and before anything is timed, each collector under
`timeout --kill-after=5 30`: `lscpu`, `ab sample --count 3 --interval 1` (three sysfs readings) and
`share_nothing.py static` (procfs, sysfs and cgroupfs only). It passes only if:
- every collector's exit status is recorded once in `preflight/exits.txt` and is 0;
- lscpu's and /proc/cpuinfo's model is the **i9-10900K**;
- all 20 CPUs are online and allowed, and the cgroup v2 path has no CPU quota at any level;
- every cpufreq policy reads `intel_pstate`, `powersave` and `balance_performance` and together
  they cover the 20 CPUs; `intel_pstate/no_turbo` reads 0;
- each of the three samples reads that governor and EPP, and turbo `true`.

Otherwise the attempt is **REFUSED**, archived, exit 4. A DRY check records the refusal and goes on.

### The schedule (registered)

- **Cells:** `mibz2-workers` × {wikidata-l2, wikidata-l9} × {2, 4, 6, 8, 10} workers: ten cells.
- **The sentinel:** `mibz2` (today's default plan) at 10 threads on the bench's corpus: the
  condition in which the two modes were recorded. It runs first and last in every round.
- **A round:** sentinel, the ten cells, sentinel: 12 runs.
- **The order:** the ten cells' order is drawn for each round from `random.Random(202611102)`
  (`SCHEDULE_SEED`), once, before data. The schedule is a function of the registered constants
  alone, and its SHA-256 is registered: `1fb24aee9592e70f764ee7dc739f2372c18796ef43a30f415eb534a2dc419ad2`
  (`SCHEDULE_SHA256`, checked by the tests and by the judge).
- **Rounds:** 1 warm-up round (run and recorded, never analysed) and **60 measured rounds**:
  732 runs.
- **One `ab run` a slot:** `ab run runs.jsonl --arms ARM --threads T --rounds 1 --settle 0.25
  --max-busy 1.5 --sysfs-state FILE`. The settle and the busy gate are the run-fed leader's
  harness's (`--settle 0.25`, ab's default gate of 1.5 cores). Unpinned, as §2 ran.
- **Pre-treatment state:** `--sysfs-state` (bench 3.5.0, ported unchanged from the run-fed
  leader's branch, eddfcca0): the clock of each CPU, the governor, EPP, turbo and package
  temperature, read in the parent before each run is spawned and after it exits, with timestamps.
  Recorded per run, never an exclusion.
- **About 12 minutes** once built (about 1 s a slot).

### The bounds

Every step that can wait runs under `bench/bounded.py` (as on #317 at efc1bd88): a process group
of its own, TERM at the deadline, KILL after the grace, then checked empty.

| step | deadline, grace (s) |
|---|--:|
| the build | 3600, 30 |
| the bench-data fetch | 300, 30 |
| the corpus's windows and build | 1800, 30 each |
| the corpora | 1800, 30 |
| the drive (all slots) | 5400, 30 |
| one slot (`ab run`, its gate's waits included) | 120, 10 |
| the judge | 600, 30 |
| the bundle | 600, 30 |

- **A slot that fails or passes its deadline** stops the drive there. It is recorded, never
  retried, and the attempt is INVALID.
- **A group that cannot be emptied** (exit 125) makes the attempt INVALID.
- **Finalisation** (`bench/par_scan_finish.sh`, #317's): the outcome is written, then the bundle
  runs under its deadline. If it fails or is late, the outcome is rewritten INVALID, the
  `.tar.gz` is renamed `.failed`, and nothing is left for `just submit` (exit 2). It cannot leave
  VALID after a bundling failure.

## 3. The analysis (`bench/share_nothing.py judge`, registered)

### Admission

The attempt is INVALID, and nothing is reported, unless:
- the identity records the frozen inputs (`inputs_match_frozen`), the preflight's pass, the
  drive's exit 0, the schedule's registered digest, the same `ab` digest before and after, and a
  clean checkout after;
- the corpora's record is ok, and each corpus's digest after the run is the one built (the
  sentinel's, the registered corpus's);
- every scheduled slot ran in order with exit 0 and added exactly one run line, whose arm, thread
  count and corpus are the slot's;
- every run decoded 209,700,196 bytes with ab's hash `a8830caee5c511a6` and no error, with a
  finite CPU and wall, and a pre-run sysfs state.

A failure of the judge itself is INVALID with its reason, never a VALID.

### The readings

- **The unit is the round.** Within a round, a cell at 10 workers and the same corpus's cell at
  4 are paired: per-round ratio r = CPU₁₀ / CPU₄.
- **G₂ and G₉:** the geometric mean of the 60 per-round ratios on each corpus.
- **R:** the geometric mean of the per-round r₂ / r₉.
- **The same in wall**, reported beside, never judged (the scanner binds the wall; §4).
- **The grid:** each cell's median CPU-seconds, wall, MB/s and pre-run clock; each worker count's
  CPU growth against 4, with intervals.

### The intervals and the dependence treatment

- **The concern:** the i9's two modes persist from run to run, so neighbouring rounds are not
  independent, and a mode change inside a round can enter a pair.
- **Within a round:** the seeded order spreads each cell's position across the round, so a mode
  switch at a fixed point in the round does not fall on one cell systematically.
- **Between rounds:** a **circular moving-block bootstrap over rounds**, block 5 rounds (about a
  minute), 10,000 resamples, seed 20261110, 95% percentile interval. Every statistic of one
  length and block uses the same draws.
- **Sensitivity** (reported, never chosen between after the data): blocks of 1 (independent
  rounds) and 10; the t interval on the log ratios beside each; and each reading's lag-1
  autocorrelation over rounds.

### The model check (registered classes)

- **G₂ against §2's band 1.10–1.44**, by its interval: within the band; below it; above it; or
  overlapping its lower edge, its upper edge, or both (wider than the band). The point's place in
  the band is reported beside.
- **G₉ against §2's ×3.50:** whether its interval contains 3.50.
- **R:** whether its interval lies below 1.
- **§3 restated** (model, labelled so): 3.76 GB/s / G₂ with G₂'s interval. Not observed.

The outcome line: `VALID: -2 CPU growth 4->10 ×G₂ [lo–hi], against the band 1.1-1.44: <class>;
-9 ×G₉ [lo–hi]; ratio of ratios ×R [lo–hi]`. A block as long as the rounds only rotates them, so
it gives no bootstrap interval, and the judge says so (a DRY check's two rounds, never the
registered run's 60).

### The mode caveat, and the registered mode split (descriptive)

- **The i9 has two persistent modes at 10 threads** (about 0.15 s and about 0.30 s for the default
  plan, with per-thread CPU doubled in the slow mode). CPU-seconds are therefore contaminated
  too, and the mechanism is unresolved (the discriminator is #318).
- **The primary readings use every measured round.** No round or run is excluded by its value or
  its mode.
- **The split** (DESCRIPTIVE, registered): a round is **fast** if both its sentinels' own walls are
  below 0.21 s (`two_modes.py`'s separator), **slow** if both are at or above it, and **mixed**
  otherwise. G₂, G₉ and R are given for the fast and the slow rounds where a class has at least
  5 rounds; the counts of each class are given always.
- **What the split is not:** the sentinels are runs of another plan on another corpus, so the
  class is an indicator of the round's machine state, not an identified mode of the cells, and a
  mode can change between a sentinel and a cell. It is reported beside the primary readings and
  never replaces them.
- **The pre-run sysfs state:** the runs whose governor, EPP or turbo read otherwise than declared
  are counted, never excluded.

## 4. What each outcome means for designs C, D and E

**The common gate** (the share-nothing entry, §4.1): a design's one-thread walk must stay within
**1.44×** the plain walk a byte, or the cooperative plan on the plain table still wins at ten.
That gate assumed §2's factor at a small table.

| table a block | design |
|---|---|
| 0.75 MB | C, runs and a directory (Wikidata) |
| 0.8 MB | this run's -2 |
| 1.2 MB | §2's -3 |
| 1.44 MB | D, k-step cells at k = 4 (Wikidata) |
| 1.8 MB | E, the line table |
| 3.6 MB | -9, today's T-vector |

- **G₂ within or below the band:** §3's factor holds for a 0.8 MB table with today's decoder.
  - C sits in the priced regime: its gate stands as written, and its next step is its own cheapest
    falsifier (the step's instructions, then a one-thread inversion).
  - D and E have larger tables than -2. This run does not price them. Their premise still rests on
    §2's -3 and -6 points, from the earlier decoder; G₉ and R say whether §2's ladder still
    describes today's decoder.
- **G₂ above the band:** at 0.8 MB the workers inflate more than §2 implied.
  - The model's range falls to 3.76 / G₂ GB/s, and the gate as written is no longer sufficient:
    the margin it leaves is consumed by the factor itself.
  - C, D and E each need their walk cost priced against the measured G₂ before a build. D and E
    have larger tables, and in §2 the growth rose with the table at every point, so they are not
    expected to fare better.
  - The share-nothing direction moves down the queue until a design shows a walk cheap enough.
- **G₂ overlapping an edge, or wider than the band:** inconclusive at this resolution. Nothing
  changes, and the interval is reported as it is.
- **G₉:**
  - if its interval contains 3.50, §2's ladder still describes today's decoder;
  - if it lies well below, today's decoder has changed the plain workers' scaling, and R, not §2's
    band, is the figure to carry.
- **R's interval including 1:** a smaller table buys nothing in CPU growth under today's
  decoder, and the share-nothing premise fails whatever the band says.

## 5. Limits

- **The scanner binds near 5×** on small blocks (`2026-11-10-parallel-scan.md`): one scanner
  thread caps the wall, so the wall at -2 cannot show what ten free workers would deliver. That
  is why CPU-seconds are the reading, as in §2. The wall is reported and never judged.
- **CPU growth from 4 to 10 workers is not a design's walk cost.** It prices the factor §3's
  model applies; it says nothing about a smaller table's instructions a byte.
- **A -2 block is not a -9 block in a small table.** It has 200 k rows, so its output buffer,
  its symbols and its MTF state are also smaller, and there are about 4.5× as many blocks
  (headers, tables, scanner work). A design C block keeps 900 k rows and their buffers. -2 is
  therefore an optimistic proxy for C's footprint.
- **Recompression changes more than the block size:** the boundaries, the Huffman tables and
  the run structure (§2's confound, unchanged).
- **The modes** (§3): the primary readings carry them; the split only describes them.
- **One machine, one attempt, one corpus content.** No retry to pass.
- **A whole-process CPU total** (getrusage, exact) includes the scanner, the reader, the sink
  and the main thread (ab's sampler's own CPU is left out). The six more threads at 10 workers
  carry their own overhead, as §2 said of its 10% at -1.

## 6. The local checks (DRY, untimed, never evidence)

On the shared dev VM (4 vCPUs of a Xeon, no cpufreq, other agents building), untimed. Its
figures are not evidence: 10 workers on 4 vCPUs cannot grow, and nothing here is a reading.

- **DRY 1, head `bee5bb8c`** (`just bench-share-nothing 1`), exit 0:
  - the build, the bench-data fetch, the windows' check (431 blocks, every CRC identical in
    libbzip2, lbzip2 and mibz2) and the corpus's digest;
  - the corpora: the content decoded by bzip2 to `3f075e30…`; -2 and -9 at `-n 4` gave the
    registered digests, `-n 1` gave the same bytes (`thread_invariant` true at both levels), and
    each decoded back by bzip2 to the content's digest;
  - the preflight ran its three collectors (each exit 0) and was REFUSED on this host, as it must
    be (model, 4 CPUs, no cpufreq policies, no turbo file, the samples' governor, EPP and turbo
    null), recorded and passed over because DRY;
  - the drive ran all 36 slots (3 rounds × 12), each adding one run line with ab's hash
    `a8830caee5c511a6` and a `machine_pre` (here every field null with its reason: a VM);
  - the judge admitted it and wrote its table; the bundle (2.5 MB: the attempt's files and its
    `ab`, the corpora left out) was made.
- **The registered run refused, head `765cdf3b`** (`just bench-share-nothing`), exit 1:
  `REFUSED: bench/share-nothing.frozen names no frozen commit yet; nothing to run`, bundled.
- **The never-reused directory:** a second attempt started in the same second refused to reuse
  the first's directory (exit 1) and made no bundle.
- **The idle gate, bounded:**
  - the gate cannot be made to stall on demand here (the machine was quiet enough to pass it);
    forced to stall (`--max-busy -1`), ab waited and `bench/bounded.py 30 10` stopped it at 30 s,
    exit 124, its group empty, no run line written;
  - the drive's own handling of a stalled slot (exit 124 recorded, nothing after it run) is a
    test (`test_drive_bounds_a_stalled_slot`).
- **DRY 2, head `765cdf3b`** (`just bench-share-nothing 1 4`, the gate relaxed to 4 cores), exit
  0: all 36 slots, about 0.7 s a slot. The first slot still waited 41 s: ab's gate also requires
  that no one CPU be more than half busy, and `--max-busy` does not relax that condition. The
  registered run on the workstation keeps the gate as the run-fed harness ran it; each slot's
  120 s deadline bounds any wait.

## Reproduce

```text
just bench-share-nothing 1           # the DRY check: one warm-up and two measured rounds
just bench-share-nothing 1 4         # the same with ab's idle gate relaxed (DRY only)
just submit bench-share-nothing      # the registered run, once frozen and queued
python3 -m pytest bench/test_share_nothing.py bench/test_bounded.py bench/test_par_scan_finish.py
```
