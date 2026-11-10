# 2026-11-10: Share-nothing pricing: registration (diagnostic only)

**Status: registered as a draft, NOT FROZEN; revised after the Researcher's REVISE** (#145
6758300190; report `state/reports/share-nothing-pricing-6758228391` at `53625b5d`).
- **The recipe:** `just bench-share-nothing` (`.just/share-nothing.just`). It refuses every run
  but a DRY one until `bench/share-nothing.frozen` names a commit.
- **Who freezes it:** only the Coordinator freezes and queues it, after the Researcher's review.

**What it is:** falsifier 2 of `2026-11-10-share-nothing.md` §4 (the Pathfinder's proposal, #145
6757754125, on #313), built as a C5 workstation recipe for the i9-10900K. Plain independent
workers (`mibz2-workers`, today's decoder) on Wikidata recompressed with `lbzip2 -2` and `-9`, at
2 to 10 workers, read as CPU-seconds and wall.

**What it is for:** a bounded, descriptive workload-scaling diagnostic. It measures how the CPU of
a decode grows from 4 to 10 plain workers at two recompression levels, and their interaction.

**What it does not do:** it sets no kernel budget, and qualifies or closes no design. It cannot:
- falsify share-nothing, or a compact table, by an interval that spans 1;
- validate §2's historical ladder by being compatible with one of its points;
- identify a finder-fed kernel's contention, or table capacity in isolation (§4, §5).

**Diagnostic only** (RULES C1). No merge gate, acceptance rule or adoption decision reads its
outcome. Every record is kept: no run is excluded, replaced or retried because of its mode, its
wait or its value.

Everything below is fixed before any data: the corpora, the worker counts, the schedule and its
seed, the rounds, the analysis, the intervals, the mode split and the deadlines.

Labels: **read** (from the record), **historical model** (derived in the share-nothing entry, not
observed), **registered** (fixed here before data).

## 1. What is measured

**The estimand** (registered): with today's decoder and this harness (the sentinels included), on
the i9:
- **G₂:** the growth in CPU-seconds from **4 to 10** workers on the -2 corpus. It is 4-to-10
  inflation, not 1-to-10: there is no one-thread anchor, and none is claimed.
- **G₉:** the same growth on the -9 corpus, in the same rounds.
- **R = G₂ / G₉:** the **recompression-level × worker-count interaction**. Recompression changes
  the block size and with it the block count, the tables, the buffers and the run structure, and
  a decoder change need not affect the two corpora equally. R is not a measure of table capacity
  in isolation.

**The historical context it is read beside** (read, not re-established):
- §2 of the share-nothing entry read, from the workstation's 28 October run
  (`2026-10-28-workstation-scaling.md` §9) with the decoder of that date, a 4-to-10 CPU growth of
  ×1.10 at -1 (0.4 MB a block), ×1.44 at -3 (1.2 MB) and ×3.50 at -9 (3.6 MB);
- the band 1.10–1.44 is where the entry expected G₂ at -2 (0.8 MB a block).

**§3's model** (historical model, labelled so wherever it is shown): 3.76 GB/s / G₂. The numerator
is fixed: ten workers at one thread's 400 MB/s at an all-core clock of about 0.94. It transfers
two assumptions this run does not test:
- dividing a one-thread rate by a 4-to-10 factor takes the 1-to-4 factor as 1;
- it takes the Workers arm's growth as the growth of ten continuously supplied, finder-fed
  workers, which this arm does not reproduce (§5).

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
    matches the digests registered here for lbzip2 2.5 (another version is recorded, not refused),
    and each file's block headers counted by their 48-bit magic, for context (it counts the bench's
    corpus's 431 blocks exactly).
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
- all 20 CPUs are online and allowed;
- no CPU quota is **established**: the cgroup2 mount is found in mountinfo; every directory on the
  process's path exists; and at each level `cpu.max` reads `max`, or is absent where the parent's
  `cgroup.subtree_control` (read) does not delegate the cpu controller, or the level is the
  hierarchy's root. A missing mount, path, directory or file is **unknown** and refuses, as does a
  cpu controller on a cgroup v1 hierarchy. Levels above a cgroup namespace's root cannot be read
  by anyone; that is a limit (§5), not established;
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
  Each reading is a value, or null with its reason. Recorded per run, never an exclusion.
- **Times:** each run's spawn and exit (`t_spawn_unix_ns`, `t_exit_unix_ns`, ab's
  CLOCK_REALTIME in ns) and each slot's start and end (`time.time()`, the same clock, in s), and
  ab's busy-gate wait before each run (`waited_s`, whole seconds). The clock is the wall clock: a
  step backwards (an NTP correction) would put the record out of order, and the judge refuses an
  out-of-order record rather than repair it.
- **About 9 to 12 minutes** once built (about 0.7 s a slot on the dev VM), more with the busy
  gate's waits: a round's length varies with the waits and is recorded, not assumed.

### The bounds

Every step that can wait runs under `bench/bounded.py` (master's, merged with #317): a process group
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
- **Finalisation** (`bench/par_scan_finish.sh`, master's, from #317): the outcome is written, then the bundle
  runs under its deadline. If it fails or is late, the outcome is rewritten INVALID, the
  `.tar.gz` is renamed `.failed`, and nothing is left for `just submit` (exit 2). It cannot leave
  VALID after a bundling failure.

## 3. The analysis (`bench/share_nothing.py judge`, registered)

### Admission: fail closed on missing evidence

The attempt is INVALID, and nothing is reported, unless every item below is present and agrees.
Absent evidence never counts as evidence: a missing digest is not equal to another missing digest,
and an empty state is not an observation.
- **Identity:** a 40-hex head; outside DRY, a 40-hex frozen commit, `inputs_match_frozen`, and the
  registered schedule digest; the identity's schedule digest equal to the schedule's; the drive's
  exit 0; a clean checkout after.
- **Sources:** every registered source's SHA-256 at the head (64-hex), and outside DRY equal to
  its SHA-256 at the frozen commit (read by `git show`).
- **The executable:** `ab`'s SHA-256 present (64-hex) before and after, and equal.
- **Corpora:**
  - the record shows the registered source corpus and content (digest and bytes);
  - each corpus's digest is present on both sides (as built, and after the run) and equal;
  - each corpus is recorded decoding to the content;
  - the sentinel's corpus after the run is the registered corpus.
- **The preflight, replayed:** the archived files (`preflight/`) are judged again by the same code.
  The replayed verdict must agree with the recorded exit, and outside DRY it must PASS. A recorded
  exit 0 alone is not a pass.
- **Slots and runs:**
  - every scheduled slot ran, in order, with exit 0, and added exactly one run line, whose arm,
    thread count and corpus are the slot's;
  - every run decoded 209,700,196 bytes with ab's hash `a8830caee5c511a6` and no error, with a
    finite CPU, wall and `waited_s`.
- **Telemetry, with explicit missingness:**
  - each run's pre- and post-run state carries every reading (clock, governor, EPP, turbo,
    package temperature), each as a value or as null **with its reason**, and its own reading
    time;
  - each run's spawn ≤ exit, each spawn ≥ the previous run's exit, and each slot's start ≤ end in
    order;
  - an unread reading (null, with its reason) is admitted and counted as unread. A reading that is
    absent, or null without a reason, is INVALID.

A failure of the judge itself is INVALID with its reason, never a VALID.

### The readings

- **The unit is the round.** Within a round, a cell at 10 workers and the same corpus's cell at
  4 are paired: per-round ratio r = CPU₁₀ / CPU₄.
- **G₂ and G₉:** the geometric mean of the 60 per-round ratios on each corpus.
- **R:** the geometric mean of the per-round r₂ / r₉: the simultaneous within-round pairing,
  over all rounds.
- **The same in wall**, reported beside, never judged (the scanner binds the wall; §5).
- **The grid:** each cell's median CPU-seconds, wall, MB/s and pre-run clock (with its count of
  unread readings), and each worker count's CPU growth against 4, with intervals.
- **The pre-run state:** unread readings counted apart from read ones that differ from the
  declared settings. Counted, never an exclusion.
- **The busy gate's waits:** `waited_s` by cell and by position in the round (runs, runs that
  waited, total, maximum). Described, never an exclusion or an adjustment: a wait follows prior
  load, perhaps the previous treatment's.
- **The chronological record:** every measured round in order, with:
  - its start (relative) and its actual duration;
  - its summed wait;
  - its sentinels' walls and class;
  - its per-round log ratios on each corpus, and log R;
  - its median pre-run clock.

### The intervals and the dependence treatment

- **The concern:** the i9's two modes persist from run to run, so neighbouring rounds are not
  independent, and a mode change inside a round can enter a pair.
- **Within a round:** the seeded order spreads each cell's position across the round, so a mode
  switch at a fixed point in the round does not fall on one cell systematically.
- **Between rounds:** a **circular moving-block bootstrap over the chronological rounds**, block 5
  rounds, 10,000 resamples, seed 20261110, 95% percentile interval. Every statistic of one length
  and block uses the same draws.
- **What the interval is:** conditional on this attempt and on the assumed dependence treatment.
  It is **not guaranteed 95% coverage** under the modes' unknown persistence:
  - dependence longer than the block is not bounded;
  - a shock common to the whole attempt can make every round alike and the interval narrow while
    another attempt's uncertainty is not.
- **Sensitivity** (reported, never chosen between after the data): blocks of 1 (independent
  rounds) and 10; the t interval on the log ratios; and each reading's lag-1 autocorrelation. The
  chronological record is the time-series reading. No interval is chosen for being narrowest, and
  there is no retry.
- **A block as long as the rounds** only rotates them, so it gives no interval, and the judge
  says so (a DRY check's two rounds, never the registered run's 60).

### The model check (registered classes)

- **G₂ against the historical band 1.10–1.44**, by its interval: within the band; below it; above
  it; or overlapping its lower edge, its upper edge, or both (wider than the band). The point's
  place in the band is reported beside.
- **G₉ against the historical ×3.50:** "compatible" if its interval contains 3.50, else
  "excludes". Compatibility is not equivalence, and does not validate the historical ladder: no
  intermediate level is measured.
- **R**, by its interval:
  - below 1: a favourable level × worker interaction;
  - above 1: a contrary interaction;
  - including 1: **INCONCLUSIVE**.
  None of these isolates table capacity (§1).
- **§3, the historical model**, restated with G₂ and labelled so (§1).

The outcome line: `VALID: -2 CPU growth 4->10 ×G₂ [lo–hi], against the historical band 1.1-1.44:
<class>; -9 ×G₉ [lo–hi]; ratio of ratios ×R [lo–hi]: <a favourable interaction | a contrary
interaction | INCONCLUSIVE>`.

### The mode caveat, and the registered mode split (descriptive)

- **The i9 has two persistent modes at 10 threads** (about 0.15 s and about 0.30 s for the default
  plan, with per-thread CPU doubled in the slow mode). CPU-seconds are therefore contaminated
  too, and the mechanism is unresolved (the discriminator is #318).
- **The primary readings use every measured round.** No round or run is excluded by its value, its
  wait or its mode.
- **The split** (DESCRIPTIVE, registered): a round is **fast** if both its sentinels' own walls are
  below 0.21 s (`two_modes.py`'s separator), **slow** if both are at or above it, and **mixed**
  otherwise. For each class present: its count, G₂, G₉ and R as **point estimates only**, and its
  median pre-run clock.
- **Why no class interval:** selecting a class's rounds and then block-resampling the selection
  would compress the time axis (a block of five fast rounds can span many more chronological
  rounds), so its interval would not mean what the primary interval means.
- **What the split is:** an endpoint-bracketed indicator of another plan's state.
  - The sentinels are runs of another plan on another corpus. Matching sentinels, and their class,
    do not identify each intervening cell's mode.
  - The scheduled sentinel can itself affect the next runs' thermal and cache state, so the
    estimand includes this harness.
  - The pre-run clock is a covariate, reported with its missingness. It is not an exclusion, and a
    low pre-run clock is not a diagnosis of the mode during the decode.

## 4. What each outcome means

**No kernel budget comes from this run.**
- **The 1.44× "common gate" of the share-nothing entry is withdrawn here.** It compared
  concurrency, not kernels (REVIEW 6758204909).
- **No replacement gate is derived from G₂ alone, before or after the data.**
- **The design record stands as reviewed:**
  - C is withdrawn;
  - E is falsified as built;
  - D4-delta (about 1.089 MB, estimated) keeps its own registered filters unchanged (15 Ir a row,
    7.35 Ir a build row, 1.2 MB);
  - the entry's 1.44 MB D table and its proposed C step are historical.
  This run reopens none of them, by implication or otherwise.
- **The conditional surface** (REVIEW 6758204909, recorded before data):

  T_new(g) ≤ min(Q, A / ((1 − f + f·g)·h(g))),

  where:
  - A is the prospective small-table throughput at g = 1;
  - f is the share of matched one-thread CPU that is build and walk;
  - g is a candidate's one-thread build-and-walk CPU ratio;
  - h(g) is its added concurrency inflation;
  - Q is the scanner and sink supply ceiling.

  **This run observes none of these terms directly.**
  - G₂ is not h(g): it is the Workers arm's 4-to-10 growth on recompressed input, with no
    candidate table.
  - Q is not observed here: the Workers arm is scanner-fed.
  - A, f and g belong to a concrete candidate.

  A later qualification compares a concrete candidate with matched default throughput, and
  charges construction, traversal and the full live state.

**Each registered outcome:**
- **G₂ within or below the historical band:** the Workers arm's 4-to-10 growth on -2 input is
  compatible with what §2 read near that block size, under today's decoder. That supports, and
  does not establish, the favourable regime the share-nothing entry describes. No budget changes.
- **G₂ above the band:**
  - the observation is reported as it is: at -2 the Workers arm inflates more than §2's points
    suggested;
  - prospective experiments are revisited. The D4 filters do not change, and no new kernel budget
    is assigned after seeing it.
- **G₂ overlapping an edge, or wider than the band:** inconclusive at this resolution. Nothing
  changes, and the interval is reported as it is.
- **G₉:**
  - compatible with 3.50: compatibility only;
  - excluding it: the Workers arm's -9 scaling under today's decoder differs from §2's record,
    and §2's band is a weaker reference for G₂.
- **R:**
  - **below 1:** a favourable level × worker interaction, useful evidence of a favourable workload
    regime;
  - **including 1:** INCONCLUSIVE. Not "a smaller table buys nothing", and not share-nothing
    falsified;
  - **above 1:** a contrary interaction. It closes neither compact representation nor
    share-nothing.

## 5. Limits, which constrain every conclusion above

- **The scanner binds near 5×** on small blocks (`2026-11-10-parallel-scan.md`). One scanner thread
  caps the wall, so the wall at -2 cannot show what ten free workers would deliver; CPU-seconds are
  the reading, as in §2.
- **Scanner-limited workers are not finder-fed workers.** In the Workers arm, workers wait on one
  scanner:
  - idle workers can lower cache contention;
  - the scanner and its queue add CPU of their own.
  So G₂ does not identify a finder-fed kernel's inflation, in either direction.
- **4-to-10 only.** The grid (2, 4, 6, 8, 10, as §2's) answers the registered 4-to-10 question and
  describes the knee. It does not establish 1-to-10 inflation or calibrate the 400 MB/s numerator.
- **-2 is an optimistic footprint proxy, not a bound.** A -2 block has 200 k rows, so its table,
  output buffer, symbols and MTF state are all smaller, and there are about 4.5× as many blocks
  (headers, tables, scanner work). A compact table for a -9 block keeps 900 k rows and their
  buffers.
  - More blocks can raise setup cost while smaller buffers lower cache pressure, and neither sign
    dominates in general.
  - The record gives each file's compressed bytes and block count for context. No result here is
    presented as isolated cache evidence.
- **Recompression measures a level × worker interaction:** the boundaries, the Huffman tables and
  the run structure change with the level (§2's confound, unchanged).
- **The modes** (§3): the primary readings carry them; the split only describes them, without
  intervals.
- **The quota:** levels above a cgroup namespace's root cannot be read. The preflight establishes
  the visible path only.
- **One machine, one attempt, one corpus content.** No retry to pass.
- **A whole-process CPU total** (getrusage, exact) includes the scanner, the reader, the sink
  and the main thread (ab's sampler's own CPU is left out). The six more threads at 10 workers
  carry their own overhead, as §2 said of its 10% at -1.

## 6. The local checks (DRY, untimed, never evidence)

### After the revision

- **The Researcher's counterexamples** (#145 6758300190), each now a negative test and replayed
  against the judge:
  - both copies of a corpus digest deleted: was VALID, now INVALID ("a digest is missing");
  - every run's `machine_pre` empty and its spawn time null: was VALID, now INVALID (readings
    absent);
  - no head, frozen commit, source digests or archived preflight: was VALID, now INVALID (each
    named);
  - the quota on `0::/missing.scope` with no hierarchy: was `unlimited`, now `unknown`;
  - R at 0.8 in 30 rounds and 1.2 in 30 (×0.980, [0.879–1.084]): was read as "buys nothing", now
    INCONCLUSIVE.
- **The end-to-end DRY at the head intended for freezing** is recorded on #326, not here, so that
  recording it does not move the head it describes.

### Before the revision

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
