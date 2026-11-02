# The minimal real-T-vector walker diagnostic: protocol, revision 3 (for review; no capture authorized)

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

## 6. Gates and decisions (revised; frozen after the probe)

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

## 7. Cost so far

About the authorized engineer-day: the ladder, M, the checks, the driver, the measurement
repair, the tests, the stand-in runs and this revision.

**i9 time used:** the one-minute audit. No capture. Next on the i9, once reviewed, is the
two-minute probe, which makes no figure. The capture would follow only after the probe's
groups are frozen and the capture is authorized.
