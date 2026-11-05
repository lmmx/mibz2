# The minimal real-T-vector walker diagnostic: protocol (revision 3, frozen), its i9 result, and its amended interpretation

- **Research only:** `examples/walker_ladder.rs`, `bench/walker_ladder.py` and
  `bench/test_walker_ladder.py`. The decoder is unchanged.
- **From:**
  - the researcher's last contribution (report 132), bounded by the guardian (report 134);
  - VM preparation authorized by report 135;
  - this revision, made after the i9's counter audit and the reviews in reports 136, 138
    and 139.
- **Its job:** to separate execution overhead from memory-request limits in the shipped
  walk. It is not a lane-count optimization project.
- **Branch:** `research/walker-ladder`. **Records:** `2026-11-01-walker-ladder/`.

**Returned for review before any capture** (report 139):

| item | where |
|---|---|
| the repaired measurement design | §3 |
| the validation results | §4 |
| the revised event plan, and the probe to confirm it on the i9 | §5 |
| the rungs' compiled contrasts | §2 |
| the gates and decisions, narrowed to what the counters support | §6 |

## 0a. What revision 2's probe got wrong (fixed in revision 3)

**The probe on the i9 (2026-11-02 00:06 UTC, at a245a45) measured nothing**
(`i9-10900K/walker-probe-20261102T000606Z-console.log`; the researcher's analysis of its
JSON, report 140).
- All 60 perf runs printed `Events disabled` and none `Events enabled`.
- All 210 event lines were `<not counted>` with zero run time.
- Every walker record said `controlled: false`.

**The causes, and the repairs** (the researcher, report 140; the guardian, report 141):

1. **`sudo` filtered the environment.**
   - perf was told its FIFOs on its command line; the walker was told them through
     `WALKER_CTL` and `WALKER_ACK`, which `sudo perf` stripped.
   - So the walker never sent `enable`, and `-D -1` kept the counters off.
   - **Repair:** the walker's FIFOs and barrier are now its own command-line arguments,
     after perf's `--` (`--ctl CTL --ack ACK`, `--barrier`). Arguments survive `sudo`
     unchanged.
   - A run given `--ctl` that cannot open its FIFOs, or given one path without the other,
     is a hard error. It is never a silent `controlled: false`.
2. **perf 6.8's acknowledgement is `ack\n\0`** (it writes `sizeof("ack\n")`).
   - A line reader passes the first ACK and then reads `\0ack\n` as the second, failing.
     That failure was latent: the run failed before it.
   - **Repair:** the walker reads the ACK FIFO byte by byte, skips leading NULs, and accepts
     exactly `ack`.
   - Rust tests: two consecutive ACKs in both framings, mixed framing, malformed ACKs, and
     no read past the frame.
3. **`100.00%` beside `<not counted>` was read as "not multiplexed".**
   - perf prints 100.00 for a counter that never ran.
   - **Repair:** an event's state is now "valid" only if it is counted and its counter's
     run time is non-zero; multiplexing is assessed only then. A zero count from a counter
     that ran is still a count.
   - The probe validates both halves of each pair (20 walks, and the 0-walk control)
     through `validity()`.
   - It **tests the control path first**, with cycles and instructions around 3 walks. If
     that fails it stops, records the raw output and the failures, and reports no group
     availability, rather than thirty misleading rows.
4. **The four-core result vector could grow inside the window.**
   - The deadline copies were launched with REPS 1, reserving two results.
   - **Repair:** REPS now caps the deadline loop (4096 by default, against about 150 walks
     in 300 ms), so the reserved vector never grows.
   - The walker asserts this after the window, the record carries it, and `validity()`
     rejects growth.

**The tests that would have caught it** (`bench/test_walker_ladder.py`, 26; the walker's
own, 8):
- **The real command constructor, run through an environment-filtering wrapper** (`env -i`,
  as sudo's default policy) with a perf stand-in and an inert child. The FIFOs and the
  barrier reach the child as arguments, with no WALKER variable left.
- **The exact failed CSV shape**, which stays invalid and "unassessed", controlled or not.
- **A good 20-walk window paired with a failed control**, which is rejected.
- **A counter with zero run time**, which is invalid.
- **A zero count that ran**, which is valid.

**End to end on the VM, through the exact recipe** (`stand-in-probe-dry*.log`):
- The recipe ran with an environment-stripping `sudo` stand-in and a perf stand-in that
  emits perf 6.8's `ack\n\0` frames and counts only between `enable` and `disable`.
- The control path held, and the groups were probed. The stand-in's one planted
  unsupported event was reported invalid and unassessed.
- With the counters acknowledged but never enabled, the probe stopped after the
  control-path test, reported it, archived the run and exited 1.

**Archives** (report 142's convention, now the default for i9 diagnostics):
- Every file of a run goes under `target/<experiment>-<UTC time>/`, the console included.
- `bench/archive.py` adds `MANIFEST.json`: the commit, its branch, any tracked changes,
  the corpus's SHA-256, the binary's SHA-256, the command, its exit status, and every
  file's size and SHA-256.
- It then writes one `target/<experiment>-<UTC time>.tar.gz` holding that directory
  alone. The run's own files are not rewritten.
- The archive is made even when the run fails.

## 0. What revision 1 got wrong (fixed here)

**1. Its counters measured work its clock did not** (the researcher, report 138; confirmed
by the guardian, report 139).
- Revision 1 timed the walks inside the ladder but ran `perf stat` around the whole
  process.
- D, P and S hashed the whole output after every walk, outside the clock but inside the
  counters.
- Subtracting a zero-walk run does not remove that: counts(N) − counts(0) ≈ N × (walk +
  hash).
- So D, P and S would have carried validation work that A–C did not. That could have
  manufactured an apparent store or production-overhead cost.
- **Fixed (§3):** counters and clock now bracket the same walks, and validation is outside
  both.

**2. Its four-core counts escaped validation** (the guardian, report 136).
- The four-core path kept neither the counters' running percentages nor any check of them.
- The summary tested multiplexing only on one-core records.
- **Fixed (§4):** one validity function covers every record that enters the analysis, with
  tests that reject a multiplexed four-core capture.

**3. Its "groups" were comma-separated lists, not perf event groups.**
- Their members could be scheduled at different times.
- **Fixed:** each group is now one perf group, `{cycles,instructions,…}`.

**4. Its contrasts and decisions said more than they measured** (report 138):
- "the cost of stores" for a contrast that changes three things;
- D3's "execution-bound" read from issued µops;
- D4's fallback to MLP alone;
- D2 comparing a best-over-lanes reference with 32-lane steps;
- D5's prose and code disagreeing.

All are revised (§2, §6). None of this touches the FOR-14 confirmation, a separate
binary and harness.

## 1. The question

Is the shipped 32-lane walk limited by miss concurrency, by instruction throughput, or by
its bookkeeping? And does the answer from one isolated core hold when four cores compete?

- **What it does not use:** the historical 38.55 and 4.49 cycles a byte. Every figure is
  measured fresh, in one binary on one machine.
- **The guardian's caveat (report 135):** a reduced-semantics walk is an optimistic
  reference, not an attainable decoder. The quantity of interest is where the gap opens as
  the shipped walk's work returns.

## 2. The ladder, and what each contrast changes

**The workload:**
- the first full block (900,000 rows) of the P87 corpus, `wikidata-20261026.bz2`
  (SHA-256 `cf13f439…`);
- its last column from the library's scanner and MTF;
- its T-vector built by the library (for P) and by the example's copy of the same build
  (for the others);
- the shipped walk's segment starts: 511 segments;
- the reference output fnv `27d7635e0e1a9555`.

**The rungs, with their compiled ordinary steps at 32 lanes** (`rungs-disassembly.md`; data
loads and stores, the prefetch apart):

| rung | instructions | loads | stores | what changes from the rung before |
|---|--:|--:|--:|---|
| A | 19 | 3: count, row, entry | 2: count, row | — : counter termination, the byte summed into one register |
| B | 20 | 3 | 2 | + the shipped prefetch |
| C | 17 | 2: row, entry | 1: row | flag termination replaces the counter (its load, decrement and store) |
| D | 19 | 3: row, entry, output pointer | 3: row, byte, pointer | + the byte stored through a per-lane pointer; − the sum |
| **M** (new) | **13** | 2: row, entry | 1: row | C without the sum or the step count: **the shipped step without its output store** |
| P | 16 | 3: row, entry, room base | 2: row, byte | M + the output store at base + offset (the room pass); outside the step, buffers, growth and assembly |
| S | — | — | — | the shipped assembly alone (`SegScratch::assemble`) |

**Why M:**
- Revision 1's "bare" chase, A, is not minimal: its counter and summation make its step
  (19) longer than the shipped one (16).
- M is the shipped step with only the output store removed. So M → P is the cleanest
  contrast here: the output store, the room base and the room pass's bookkeeping.

**The contrasts are net, not isolated costs:**
- each changes the instructions shown, not one thing;
- they need not add up, because added work can overlap memory waits;
- **P − S is not exact traversal**: assembly after a walk and assembly alone run in
  different cache contexts.

A–C's sum is a one-register add, a 1-cycle recurrence. It is far shorter than the step,
but it is a dependency that P does not have. M has none.

**Lanes:** 1, 8, 16 and 32 for every rung (S at 32 only).

**Every walk is checked:**
- A–C: by rows stepped and the bytes' sum;
- M: by each segment arriving at its precomputed successor's start, checked only where
  segments end;
- D and P: by the output's hash against the shipped walk's.

## 3. The repaired measurement boundary

**One window holds the counters and the clock, and the walks only.**

**Before the window:**
- the setup (decode one block, build its tables);
- one warm-up walk, checked in full. So every window starts from a walked table's cache
  state, the same for every rung.

**The window:**
- The ladder writes `enable` to perf's control FIFO and waits for `ack`.
- It reads the clock, runs REPS walks, reads the clock again.
- It writes `disable` and waits for `ack`.
- perf runs with `--control fifo:CTL,ACK -D -1`, so nothing is counted before `enable`.
- Inside, A–C and M append each walk's two result words to a vector reserved beforehand.
  Nothing is hashed.

**After the window:**
- every A–C and M walk's result is checked;
- D's, P's and S's last output is hashed.
- Every walk of theirs writes the whole output afresh, and the warm-up was checked in full.

**The control overhead:**
- Each round, each group also runs a 0-walk window: an `enable` and a `disable` around
  nothing.
- Its counts must be under 1% of the same group's A32 window.
- This replaces revision 1's subtraction, which no longer exists.

**Four cores:**
- Four copies, one per physical core, each with its own table, set up and warm up.
- Each prints `ready` and waits. The driver releases all four with one `go DEADLINE`, and
  each walks until that common deadline (300 ms; at least one walk).
- So all four walk throughout each other's windows, and every copy's counts are its own
  bounded window.
- Each copy records its walks and its window's wall-clock bounds.
- A fixed number of walks is not used: copies running at different speeds would then end
  at different times, and the late ones would run uncontended. A VM test showed this:
  overlap 0.74–0.92 with fixed walks, 0.99 or more with a deadline.

## 4. Validation (G0), and its tests

**`validity()` checks every record that enters the analysis:** one-core, four-core and
control-overhead runs alike, the preflight's included. A record fails if:
- it is unchecked;
- it was counted but not bounded by the control FIFO;
- any event of its group is missing or not counted (a zero count is a count);
- any event ran less than 99.9% of its enabled time (multiplexed).

**Across records:**
- a four-core release without exactly four copies fails;
- one whose windows overlap less than 0.95 of any copy's window fails;
- a control overhead above 1% fails.

**`bench/test_walker_ladder.py` (17 tests, all passing)** covers each rule on synthetic
records:
- a multiplexed four-core capture is rejected;
- so is a single multiplexed four-core copy;
- so are multiplexed control-overhead and one-core runs;
- so are uncounted and missing events, unchecked runs, uncontrolled counts,
  non-overlapping windows, an incomplete four-core set, and excessive control overhead;
- a zero count passes;
- perf's CSV parsing is tested, including `:u` names and `<not counted>`;
- each group's size and the decision pairs' co-membership are tested.

**End to end on the VM, with a stand-in perf** (the VM has no PMU; `stand-in-*.log`):
- **A clean capture:** every group counted as a perf group, bounded by the FIFO, with the
  preflight passing and G0 holding. G1 failed on the shared VM's noise, as it should.
- **The same with only the four-core copies multiplexed:** rejected, VOID with 120 G0
  failures.

**The ladder's own checks** pass in release and debug (`check-release.log`,
`check-debug.log`):
- blocks of 900,000, 899,828, 99,981 and 10 rows;
- Wikidata, text, random and periodic data;
- every rung at every lane count.

## 5. The event plan, and the revised probe (the i9; recipe ready)

**The audit's finding (`i9-10900K/walker-audit-20261101T232121Z.*`):** all 22 events exist
and count. But every comma-list of four, with cycles and instructions, multiplexed, at
82–86% (cycles at 65%).
- **Likely cause:** the NMI watchdog (`nmi_watchdog` 1) holds the fixed cycles counter, so
  perf's cycles takes one of the four programmable counters (SMT on).
- **The machine's settings are not changed.** Disabling the watchdog remains the
  maintainer's option, not a requirement.

**The groups** (each one perf group with cycles and instructions; three events at most):

| group | events | read for |
|---|---|---|
| exec | `uops_issued.any`, `uops_retired.retire_slots`, `idq_uops_not_delivered.core` | issue and retirement pressure; undelivered front-end slots (D3) |
| branch_sb | `br_misp_retired.all_branches`, `resource_stalls.sb`, `resource_stalls.any` | mispredicts; store-buffer stalls |
| mlp | `l1d_pend_miss.pending`, `l1d_pend_miss.pending_cycles`, `l1d_pend_miss.fb_full` | demand MLP and fill-buffer pressure, together (D4) |
| stalls_l1 | `cycle_activity.stalls_l1d_miss`, `cycle_activity.stalls_total` | stall cycles with an L1 miss outstanding |
| stalls_l2 | `cycle_activity.stalls_l2_miss`, `cycle_activity.stalls_l3_miss`, `dtlb_load_misses.walk_active` | deeper misses; page walks |
| pf | `sw_prefetch_access.t0`, `load_hit_pre.sw_pf`, `mem_load_retired.l1_miss` | prefetches executed; demand loads meeting an in-flight software prefetch |
| loads | `mem_load_retired.fb_hit`, `mem_load_retired.l2_miss`, `mem_load_retired.l3_miss` | where demand loads are served |

**The revised probe** (`sudo -v && just walker-counter-probe`, about two minutes; revision
3's repairs in §0a):
- It checks the corpus's hash and runs the ladder's check.
- It runs each candidate group above as one perf group, bounded by the control FIFO around
  20 walks and once around none. The candidates include the fallbacks `exec_pair`,
  `mlp_pair` and `fb_full`, and every event alone.
- It keeps perf's **raw output and running percentages** for every run.
- If a group the decisions need, mlp or exec, cannot be scheduled whole, the protocol says
  so. That decision is then not read. Its ratios are never built across runs.
- The probe is exercised on the VM with the stand-in perf. Its real figures need the i9.

## 5a. The probe's result (the i9, 902a911, 2026-11-02 00:35 UTC): every group counts

**The archive:** `i9-10900K/walker-probe-20261102T003547Z.tar.gz`, SHA-256 `44f02155…`, as
uploaded.
- Its manifest: commit `902a911`, no tracked changes, exit status 0, corpus `cf13f439…`.
- All three files' checksums verify. perf is 6.8.12.

**What perf's raw output shows**, read directly, not only through the probe's summary:
- **The control path held.** All 60 runs printed `Events enabled`: each group's 20-walk
  window and its 0-walk control.
- **Every event counted.** All 210 event lines were counted (no `<not counted>`, no
  `<not supported>`), every one at 100.00% running with a non-zero run time.
- **Each group was scheduled together.** Within every run, all of its events report one
  identical run time: one perf group, counting as one.
- **The control overhead is small.** A 0-walk window's cycles are at most 0.058% of a
  20-walk window's (the gate is 1%). Such a window runs about 18 µs, the
  acknowledgement's round trip.
- **The NMI watchdog was on** (1). Three events with cycles and instructions fit beside
  it, as §5 expected, without changing the machine's settings.

**So the seven groups in §5 are frozen as they stand.** No fallback is needed:
- the mlp trio (`pending`, `pending_cycles`, `fb_full`) counts together, so D4 is
  readable;
- the exec trio counts together, so D3 is readable.

The probe's counts are kept as perf printed them. They are not analysed: it is an
availability probe, and its windows are not the capture's.

## 6. Gates and decisions (frozen)

**Notation:** c(X, k) is cycles a row, rung X, k lanes, the median over its runs. Each run's
cycles come from its own window.

**The gates:**

| gate | rule | if it fails |
|---|---|---|
| G0 | `validity()` returns no failures (§4) | void |
| G1 | P32's ns a row, max ÷ min over its runs, at most 1.05 | void |

**The decisions:**

| | rule | reading |
|---|---|---|
| **D1** | c(P, 32) ≤ 1.15 × bare32, where bare32 = min(c(A, 32), c(B, 32), c(M, 32)) | the shipped walk is within 15% of the best reduced walk at the same lane count: **no walker scheduling rewrite** (report 134's stopping rule) |
| **D2** | c(P, 32) ≥ 1.5 × bare32 | report the net contrasts at 32 lanes (A→B, B→C, C→D, M→P, D→P) and name the largest; any rewrite would be of that contrast's code only, with its own budget, after review |
| between | 1.15 < ratio < 1.5 | reported; no rewrite on this evidence |
| **D3** | from the exec group's simultaneous counts on P32 | **issue pressure**, the issued µops ÷ (4 × cycles), with the retired-slot and undelivered shares beside it. At 0.70 or more: added instructions are unlikely to be absorbed. That is a planning signal for FOR-14-like work, **not a bottleneck classification**; below 0.70, nothing is concluded |
| **D4** | from the mlp group's simultaneous counts on B16 and B32 | demand MLP (pending ÷ pending cycles) rising less than 10% from 16 lanes to 32, **and** fb_full events a cycle at least doubling **and** at least 0.01 at 32: **consistent with** a miss-capacity constraint, not an exclusive diagnosis. Not read if the mlp group was not counted whole |
| **D5** | r = (P32 ÷ B32, four cores) ÷ (P32 ÷ B32, one core) | \|r − 1\| > 0.15: the isolated-core conclusions do not transfer; report and stop |

**Event-specific control overhead** (the guardian, report 146):
- A small control overhead in cycles does not make every event's control count small.
- In the probe, `cycle_activity.stalls_l3_miss` counted 17,636 in a 0-walk control against
  28,608 in a 20-walk window: a ratio of 0.62. `dtlb_load_misses.walk_active` (0.70),
  `mem_load_retired.l3_miss` (0.67) and `idq_uops_not_delivered.core` (0.10) were also
  comparable.

**So the summary does three things:**
- It reports every event's control-only count beside its window count at each decision
  config (P32, B16, B32, A32, M32), as medians over the rounds, with their ratio.
- It marks an event **comparable** when its control count is at least 10% of the
  window's, or when a zero window has a non-zero control.
- Any decision whose events are comparable is **qualified**: reported, but not read as a
  classification.

**What it does not do:**
- It never subtracts the control. The two are separate windows, not an exact accounting of
  the same overhead.
- Both counts zero is a valid zero, not a comparison. Rung A, for example, issues no
  prefetches.

The capture's windows are 100 walks (25 at one lane), against the probe's 20, so its
ratios should be smaller. They are reported either way.

**How the counters are read:**
- **Demand MLP excludes overlap that software prefetch supplied earlier.** A buffer
  allocated by a prefetch enters `pending` only when a demand load meets it. So better
  prefetching can lower demand MLP while raising throughput.
- `load_hit_pre.sw_pf` and `mem_load_retired.fb_hit` are read beside it, and B is compared
  with A.
- `fb_full` is an event count. It is reported per row and per cycle, never as a share of
  cycles full.

**What none of them licenses:**
- no lane-count sweep beyond 1, 8, 16 and 32;
- no production walker change;
- no format work.

The diagnostic ends with its summary. The next architecture is chosen from the measured
bottleneck, after review.

## 6a. Deferred: an in-process counter backend (reports 144–145)

**The idea:** the walker could open, enable, disable and read its own counter group through
`perf_event_open` (for example the `perf-event2` crate). That would remove the control
FIFO, the acknowledgements and the CSV parsing that caused revision 2's failure.

**What it would not remove:**
- the access permissions;
- the counters' number and constraints;
- the duty to define each Intel event exactly. Raw encodings are CPU-specific.

**The guardian's ruling:** it is the likely direction if this harness becomes reusable
infrastructure. It is not an interruption of this diagnostic, and not a second system.

**A migration would replace only the counters' configuration, control and reading.** It
would have to show agreement on:
- event encodings, group membership and leader;
- thread and CPU scope, and the user/kernel policy;
- checks for unscheduled events;
- a measured control overhead.

Its results would be cross-checked against perf within measurement variability.

## 8. The result (measured, the i9, 29a0215, 2026-11-02 00:45 UTC)

**The run:** `just bench-walker-ladder`, from the frozen commit `29a0215`.
- **The archive:** `i9-10900K/walker-ladder-20261102T004501Z.tar.gz`, SHA-256 `bd1768a0…`,
  as uploaded.
- **Its manifest:** commit `29a0215`, no tracked changes, exit status 0, corpus
  `cf13f439…`. All three files' checksums verify. The summary is kept beside it.
- **Before the capture:** the Rust and Python tests, the ladder's correctness check, and
  the preflight (all seven groups counted, unmultiplexed, bounded by the FIFO) all passed.

### 8.1 The registered gates and decisions

| | measured | registered reading |
|---|---|---|
| G0 | no failures. That covers every record (checked, FIFO-bounded, counted, unmultiplexed, no result-vector growth), the four-core overlap and the control overhead | holds |
| G1 | P32's ns a row over its 21 runs: max ÷ min 1.026 | holds (≤ 1.05): **the run is valid** |
| **D1** | c(P, 32) = 6.90 cycles a row; bare32 = c(M, 32) = 6.29; ratio **1.096** | **D1: no walker scheduling rewrite** (report 134's stopping rule) |
| D3 | P32 issues 0.57 of 4 µops a cycle (retired slots 0.54, undelivered front end 0.01) | below 0.70: no classification follows |
| D4 | B's demand MLP 1.35 at 16 lanes, 1.61 at 32 (+19%); `fb_full` 0.12 → 0.16 events a cycle (×1.34) | no plateau with rising fill-buffer pressure, by the rule |
| D5 | (P32 ÷ B32, four cores) ÷ (the same, one core) = **1.055** | the P/B relation holds under four copies (\|r − 1\| ≤ 0.15). P/M is not covered: M was not run at four cores (§8.6) |

**No decision is qualified by report 146's safeguard.** Fourteen config-event pairs were
comparable, all in events no decision reads:
- `mem_load_retired.l3_miss`, about 120 counts a window against 64 in the control;
- `cycle_activity.stalls_l3_miss`, 0.46–0.63;
- `dtlb_load_misses.walk_active`, 0.28–0.47.

They are reported in the summary, not read.

### 8.2 Cycles a row (one core, medians)

| rung | 1 lane | 8 | 16 | 32 |
|---|--:|--:|--:|--:|
| A (bare, no prefetch) | 51.71 | 9.64 | 9.78 | 9.76 |
| B (+ prefetch) | 53.76 | 9.99 | 6.53 | 6.66 |
| C (flag end) | 54.41 | 9.73 | 6.49 | 6.36 |
| D (+ stores) | 54.36 | 10.07 | 6.65 | 6.84 |
| M (shipped step, no store) | 54.49 | 9.44 | 6.21 | 6.29 |
| P (shipped walk) | 54.79 | 10.26 | 6.76 | 6.90 |

**The net contrasts at 32 lanes, in cycles a row:**
- A→B: −3.10;
- B→C: −0.30;
- C→D: +0.48;
- **M→P: +0.60**, a net contrast between two kernels: P's output stores, its room-pass
  bookkeeping and assembly, and how that work interacts with the rest of the step. It is
  not an isolated cost of the output store (§8.6);
- D→P: +0.06.

### 8.3 What the counters describe (descriptive, not registered decisions)

**At 32 lanes, one core, per row:**

| | cycles | instructions | IPC | issued ÷ 4c | stall cycles | L1-miss stalls | demand L1 misses | demand MLP | `fb_full` a cycle |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| A | 9.78 | 19.4 | 1.99 | 0.45 | 0.26 | 0.25 | 0.978 | 5.06 | 0.000 |
| B | 6.70 | 20.4 | 3.05 | 0.69 | 0.15 | 0.05 | 0.004 | 1.60 | 0.165 |
| M | 6.29 | 13.3 | 2.12 | 0.45 | 0.34 | 0.08 | 0.005 | 1.85 | 0.603 |
| D | 6.85 | 19.6 | 2.86 | 0.64 | 0.17 | 0.05 | 0.005 | 1.60 | 0.254 |
| P | 6.90 | 17.0 | 2.46 | 0.57 | 0.23 | 0.07 | 0.018 | 2.45 | 0.377 |

**Read from that table:**
- **Without prefetch, every step misses.**
  - A misses L1 on 0.98 of rows and L2 on 0.91: the table comes from L3.
  - A demand miss is outstanding 99% of the time, with 5.06 outstanding on average. This is
    the demand MLP the historical 8.6× ratio could not measure.
- **With prefetch, the demand loads hit.** Demand L1 misses fall to 0.004–0.018 a row, and
  L1-miss stall cycles to 5–8%.
  - So the prefetched walks are not waiting on their demand loads.
  - Their issue shares (0.45–0.69) are below the registered 0.70 condition. That classifies
    nothing (§8.4, item 4).
- **Fill-buffer pressure rises as the walk gets faster.**
  - `fb_full`, requests blocked for want of a fill buffer, runs 0.17–0.60 events a cycle in
    the prefetched rungs. It is highest in M, the fastest.
  - The prefetched walks reach their speed by 16 lanes: 16 and 32 differ by 0.1–0.3 cycles
    a row. A's speed is reached by 8.
- **A Little's-law model** (a hypothesis with stated assumptions, not a measurement; §8.6):
  - one chain takes 52–55 cycles a row; 32 lanes take 6.3–6.9;
  - the quotient, about eight, is a **throughput ratio**. It becomes a count of lines in
    flight only if each row makes one line request, the request's residence time under 32
    lanes equals the whole one-lane step, and the one-lane step is all memory service. The
    one-lane step also holds the step's instructions, its address dependency and loop
    work, so none of those is shown;
  - "near the core's L1 fill-buffer count" is therefore a coincidence of the model's
    number, not evidence of a fill-buffer limit.
- **`fb_full` counts occurrences,** requests (load, store or software prefetch) that found
  no fill buffer free. It is not the fraction of time the buffers were full, nor their
  average occupancy.
- **This model is not D4.** D4's registered rule reads demand MLP. Demand MLP counts a
  prefetch-allocated buffer only once a demand load meets it. So D4's "no plateau" neither
  supports nor refutes a total-occupancy limit; testing one would take a registered
  measurement of the prefetch-held occupancy.

**Four cores** (A32, B32 and P32 as four copies, released together to a deadline):

| | ns a row, one core | four cores | ratio |
|---|--:|--:|--:|
| A | 1.856 | 2.744 | 1.479 |
| B | 1.265 | 1.856 | 1.467 |
| P | 1.306 | 2.021 | 1.548 |

- The windows overlapped (G0), with 110–205 walks a copy, inside the cap.
- In cycles, four-core P32 takes 9.89 a row against one core's 6.90: +43% for four
  independent tables (14.4 MB) in a 20 MB L3.

### 8.4 What it means (amended after reports 148–150; the first wording is in `92bfe45`)

**Measured, under the registered decisions:**
1. **No walker scheduling rewrite** (D1, report 134's stopping rule).
   - At one core and 32 lanes, the shipped walk costs 1.096 of the reduced walker M.
   - M omits required output work, so even that 0.60 cycles a row is not a saving
     available to better code. M is a reduced reference, not a hardware minimum: D1 does
     not show that a different algorithm or representation cannot do better.
2. **Prefetch helps this traversal substantially:** A→B, −31.8% cycles a row. This is an
   intervention contrast, not a counter reading.
3. **The prefetched walks change little between 16 and 32 lanes** (P: 6.76 → 6.90 cycles
   a row). This is descriptive. It authorizes no lane-count change and no new sweep.
4. **D3 classifies nothing.** P32 issues 0.57 of the four-per-cycle reference, below the
   registered 0.70 condition. The issued count is µops allocated to the scheduler, not
   the headroom of each port or dependency chain. So the capture shows neither that extra
   instructions are the walk's main limit nor that they cannot be one.
   - FOR-14's calibration, that about half the one-worker walk scales with instructions,
     is not confirmed. No replacement explanation is measured either: "added work delays
     each lane's next prefetch" is a hypothesis.
   - The FOR-14 stop stands on its own measurement (§7a of its record), not on this.
5. **D4's registered signature did not occur.** That fails to establish a miss-capacity
   diagnosis. It does not show memory-resource limits are absent.
6. **D5 holds for P against B only** (1.055). With no four-core M, the capture does not
   show the 9.6% P/M gap persists under four copies.
7. **A substantial concurrency penalty persists in reduced walkers without the production
   pipeline** (§8.6). This is the demonstrated architectural finding.

**Hypotheses, not measurements:**
- **A fill-buffer (L1 miss) occupancy limit at one core.** It rests on §8.3's
  assumption-dependent Little's-law model and on `fb_full` occurrences. Neither measures
  occupancy.
- **Slower L3 or memory service under four copies** as the cause of the four-copy
  penalty. Cache residency, request servicing, prefetch interactions and package
  operating conditions are not separated. The rare L3 events that might bear on this are
  too close to their controls to use (§8.5, item 2).

**What it does not establish:**
- **This is one block,** warmed and walked repeatedly, its table L3-resident at one core.
  The decoder's other phases are not in it.
- **The four-core runs are four copies of one block,** not the decoder at four workers. It
  gives no full-decoder speedup estimate.

### 8.5 The independent review (report 147), and what it narrows

**The researcher re-checked the archive from its raw records, not the summary:**
- 618 records: 525 one-core, 21 control-only and 72 four-core;
- 3,012 event readings, all matching their parsed values;
- every record enabled between disables, every event at 100% running with a positive run
  time, and one run time per group;
- all 18 four-core releases with four copies, the worst overlap 99.44%;
- 110–205 walks a copy;
- G1 at 1.0258.

**Its conclusions:**
- The repaired perf collection is trusted for this diagnostic's main comparison.
- D1 is accepted: P32 ÷ M32 = 1.096 in cycles, and 1.096 by the clock as well.

**Its qualifications, adopted here:**
1. **The counters' window slightly encloses the clock's.**
   - The walker starts its clock after perf acknowledges `enable`, and sends `disable`
     after stopping it.
   - The 0-walk controls measure that margin: at most 0.012% of an A32 window's cycles and
     0.017% of a P32 window's. P32's issued µops and retired slots are at 0.004% and
     0.003%.
   - That is negligible for cycles and the execution and MLP counters.
2. **The rare-event counters are not isolated well enough for fine claims.**
   - For P32, the controls are 62% of `cycle_activity.stalls_l3_miss` and 47% of
     `mem_load_retired.l3_miss`. Several page-walk counts are comparable too.
   - Nothing here rests on them, and they are not subtracted. If they ever became
     central, an in-process collector might narrow that margin. That benefit is possible,
     not shown.
3. **Issue share is not spare capacity.**
   - P32's 0.57 of 4 µops a cycle, 2.29 a cycle, does not mean 43% of execution capacity
     is free for added work.
   - The issued count measures µops sent to the scheduler, not each resource's headroom.
   - So §8.4's item 4 says only that the shipped walk does not meet the registered
     issue-pressure condition. Why FOR-14's added work cost what it did remains open.
4. **D5's scope is P against B.**
   - There are no four-core M runs. D5 shows that the P/B relation holds under four
     copies; it does not show that the 9.6% P/M gap does.
   - These are repeated walks of one warmed 900,000-row block. They are not the decoder.
5. **The low demand MLP of the prefetched rungs** (about 1.6 against A's 5.0) is not a
   count of total requests in flight. Prefetch-allocated buffers enter it only when a
   demand load meets them.

**On the collector (reports 144–145):**
- Keep the repaired perf backend for this diagnostic.
- An in-process Rust backend stays deferred. It becomes a candidate if the harness becomes
  reusable infrastructure, or if the rare events need cleaner isolation.
- This archive is the reference any replacement must reproduce.

### 8.6 The interpretation amendment (reports 148–151), and the matched four-copy comparison

**The guardian's instruction (report 150):** accept and close the capture under its
registered decisions; keep the repaired perf backend; tighten the descriptive conclusions;
keep fill-buffer limitation and slower memory service as hypotheses. No rerun, new
counters or collector migration. §8.1–§8.4 above are amended accordingly.

**What was corrected:**

| first wording (`92bfe45`) | corrected reading |
|---|---|
| "about eight lines are in flight on average" | the one-lane ÷ 32-lane ratio is a throughput ratio. As an occupancy it is an assumption-dependent model (§8.3), not a measurement |
| "the shipped walk issues at 57% of width, so it is not issue-bound" | D3 classifies nothing. The capture shows neither that extra instructions are the main limit nor that they cannot be (§8.4, item 4) |
| "the P/B relation transfers", read as the one-core conclusions transferring | D5 covers P/B only; M has no four-core runs, so P/M is untested at four cores |
| "the remaining 0.6 cycles a row is the output store and its room-pass bookkeeping" | a net contrast between kernels, not an isolated output-store cost |
| "the four-core penalty is in the memory system … the L3 and memory service" | a well-motivated hypothesis. The limiting resource is not identified |

**The four-copy comparison, reproduced from the raw records**
(`walker-ladder-20261102T004502Z.jsonl`). One core → four copies, 32 lanes:

| rung | cycles a row, all groups pooled | cycles a row, `exec` group only | instructions a row, `exec` group | issued µops a row | retired slots a row | cycles ÷ ns |
|---|---|---|---|---|---|---|
| A (no prefetch, no output) | 9.763 → 13.594 (+39.2%) | 9.779 → 13.526 (+38.3%) | 19.4381 → 19.4432 (+0.026%) | +0.26% | +0.065% | 5.27 → 4.95 |
| B (prefetch, no output) | 6.659 → 9.196 (+38.1%) | 6.696 → 9.146 (+36.6%) | 20.4347 → 20.4376 (+0.014%) | +0.15% | +0.045% | 5.29 → 4.96 |
| P (shipped walk) | 6.897 → 10.000 (+45.0%) | 6.900 → 9.891 (+43.3%) | 16.9915 → 16.9956 (+0.024%) | +0.51% | +0.095% | 5.29 → 4.95 |

These agree with reports 148 and 149.
- **The matched comparison is the `exec`-group columns.** There, cycles, instructions,
  issued µops and retired slots are counted together in the same runs.
- **It was computed three times** from the same archived records: by the researcher
  (report 149), by the guardian (report 151) and here. It is a stronger analysis of one
  capture, not an independent replication.
- **Pooled and `exec`-only cycles differ** because the pooled figure takes every group's
  cycle count. Elapsed time a row rises more, by about 47–55%.
These are per-copy costs. Four copies together still do more total work than one.

**The demonstrated finding, stated narrowly:**

> These simplified traversals show a substantial concurrency cost without materially more
> instruction work. The production queue, decoded-output work and software prefetch are
> not necessary for it to appear. This does not identify the physical bottleneck. It does
> not imply that cutting instructions or state traffic cannot help a different
> implementation.

- Four concurrent copies of A, B and P each take 37–45% more cycles a row. That is
  +38.3%, +36.6% and +43.3% in the matched `exec` group.
- They do essentially the same work a row:
  - instructions within 0.03%;
  - issued µops within 0.51%;
  - retired slots within 0.1%.
- The ratio of counted cycles to elapsed ns falls from about 5.29 to 4.95. That is
  consistent with a lower effective clock, and it is why elapsed time rises more than
  cycles. It is not a dedicated frequency measurement. A lower clock alone, with the same
  cycles needed a row, cannot produce the rise in cycles a row.
- **So the penalty does not require:**
  - the production job queue (these are independent copies);
  - decoded output or segment assembly (A and B have neither);
  - software prefetch (A has none).
- A and B still keep lane state: each lane's position and remaining work are updated
  every step. They are reduced walkers, not kernels without bookkeeping or stores.
- "Not necessary" is the whole claim. It does not say those mechanisms add nothing to
  P's or the decoder's cost. It does not show the full decoder's four-worker penalty has
  the same cause.
- **"Concurrency adds almost no instructions" is not "fewer instructions cannot help."**
  The two questions are separate:
  - Why does the current kernel slow under concurrency? This capture does not support an
    increase in instruction count as the answer.
  - Could a different implementation run faster? Possibly. Fewer loads, stores, address
    calculations or dependencies could still help. So closing the generic
    scheduling-rewrite question is compatible with continuing the bounded `k`/`j`
    assessment. The packed token removes explicit state traffic from the compact walk.
    It does not try to remove bookkeeping added between one core and four. Its payoff
    is unmeasured.

**For the ongoing Astra/researcher assessment of `k` and `j`:** this capture predicts
neither their success nor their failure. They were not measured.
- The question it leaves (report 151): can the traversal's representation or access
  pattern change so that concurrent execution gets cheaper, without adding more
  extraction and state-management cost than it saves?
- The frozen FOR-14 implementation answered no, for that implementation. `k` and `j` have
  not answered it.
- Two readings are unsupported:
  - "the smaller table must eventually win";
  - "the shipped loop is close to M, so representation research is exhausted".
  The ladder limits one class of improvement. It does not establish a hardware optimum.
- The frozen FOR-14 loss stands. The packed token's reduction in compiled work (§8a of
  the FOR-14 record) is separate evidence.
- Neither the 27-instruction listing nor the 0.57 issue share predicts their speed.
  - Removing a load/store pair could help without saturating issue width.
  - A shorter listing could still lose by lengthening the address dependency.

**Closed:**
- The diagnostic is closed under D1.
- The perf backend is kept. The in-process collector stays deferred (§6a).
- No further i9 run is requested. This is the diagnostic's endpoint: the next substantive
  decision belongs to the compact-candidate assessment (report 151).

## 7. Cost so far

About the authorized engineer-day: the ladder, M, the checks, the driver, the measurement
repair, the tests, the stand-in runs and this revision.

**i9 time used:**
- the one-minute audit;
- the failed probe (a245a45);
- the revised probe (902a911), well under the two minutes estimated.

No capture. **The capture, `sudo -v && just bench-walker-ladder`, is frozen at the commit
that adds report 146's safeguard, and is not authorized.** It would need its own
authorization.
- Since the probed `902a911`, only the analysis changed: the summary's control
  comparison, and its tests.
- The collection code, the launcher, the walker and the event groups are as probed.
