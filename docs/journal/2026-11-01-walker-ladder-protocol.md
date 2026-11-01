# The minimal real-T-vector walker diagnostic: preparation, counter audit and frozen protocol (for review)

- **Research only:** `examples/walker_ladder.rs` and `bench/walker_ladder.py`. The decoder is
  unchanged.
- **From:**
  - the researcher's last contribution (report 132);
  - bounded by the guardian (report 134);
  - authorized for VM preparation by report 135, within one engineer-day.
- **Its job:** to separate execution overhead from memory-request limits in the shipped
  walk. It is not a lane-count optimization project.
- **Branch:** `research/walker-ladder`, from master. **Records:** `2026-11-01-walker-ladder/` (the checks, the VM sweep, the recipe's VM dry run, the stand-in audit).

**Returned for review before any i9 capture** (report 135):

| item | where |
|---|---|
| source pin | the commit that adds this file (PR description) |
| correctness evidence | §3 |
| counter availability audit | §4: **run on the i9** (2026-11-01 23:21 UTC): all 22 events exist and count; every group of four multiplexed, so the groups are now of three |
| protocol | §5, **frozen** after the audit (event groups, preflight, recipe `bench-walker-ladder`); not authorized |
| stopping rules | §6, frozen |

## 1. The question

Is the shipped 32-lane walk limited by miss concurrency, by instruction throughput, or by
its bookkeeping (lane state, flags, stores, buffers)? And does the answer from one isolated
core hold when four cores compete?

**Why now:** FOR-14's stop rests on a calibration from stage A: about half of the
one-worker walk's time scales with its instructions (2026-11-01-for14-stage1.md §6). This
diagnostic measures that directly, with counters, on the real table.

**What it does not use:** the historical 38.55 and 4.49 cycles a byte. They come from
different contexts, and the researcher warned against dividing them. Every figure here is
measured fresh, one binary, one machine.

**The guardian's caveat (report 135):** the bare chase is an optimistic reference, not an
attainable decoder. It drops the output and termination work a walk must do. The quantity
of interest is where the gap opens as those costs return, not how fast the bare loop runs.

## 2. The ladder

**The workload:**
- One real T-vector: the first full block (900,000 rows) of the P87 corpus,
  `wikidata-20261026.bz2` (SHA-256 `cf13f439…`).
- Its last column comes from the library's scanner and MTF.
- Its T-vector is built twice, outside every timing:
  - by the library's `bwt::build`, for rung P;
  - by the example's plain copy of the same build, for the other rungs. Its entry is the
    library's, `link << 8 | byte`.
- The segment starts are the shipped walk's: production's 511 xorshift rows (copied) and
  the row the output begins in. That gives 511 segments on this block.
- The reference is the shipped walk's output: fnv `27d7635e0e1a9555` on this block.

**The rungs, each adding one cost to the one before** (`examples/walker_ladder.rs`):

| rung | what a lane does a step | adds |
|---|---|---|
| A | load T[row]; row = entry ≫ 8; sum the byte; count the segment down | — (the bare chase; termination by a known length) |
| B | A, and prefetch the next entry into L1 | the shipped prefetch, one lane turn ahead |
| C | B, but end the segment at a START-flagged row | the shipped termination test, instead of a counter |
| D | C, and store the byte into the output at the segment's known place | output stores, with nothing to grow or assemble |
| P | the shipped walk, `walk_segments_room_with_prefetch::<k, PF_L1>` | production's lanes, room passes, buffers, growth and assembly |
| S | the shipped walk's assembly alone (`SegScratch::assemble`) | — (to take from P) |

**Lanes:** 1, 8, 16 and 32 for every rung (S at 32 only). More lane counts are added only if
a decision in §6 depends on it.

**Scheduling:**
- Every rung's lanes take segments in order, from one queue.
- When the queue is empty, the lanes still walking finish theirs one turn each, as the
  shipped tail does.
- So A–D differ from each other only in the cost each rung adds.

## 3. Correctness (VM, done)

**Every walk is checked after its clock stops:**
- A–C: the rows stepped equal n, and the bytes' sum equals L's sum. Each row is visited
  exactly once.
- D and P: the output's hash equals the shipped walk's.
- S: the assembled output's hash.

A failed check aborts the run. The capture cannot report a figure from a wrong walk.

**`walker_ladder FILE check [BLOCK]`** runs every rung at every lane count, two walks each,
plus S. It passes on all of these (`check-release.log`, `check-debug.log`):

| input | block | rows | segments | builds |
|---|---|--:|--:|---|
| wikidata-20261026 | 1 (the protocol's) | 900,000 | 511 | release, debug |
| wikidata-20261026 | 3, 101 | 899,828 and 900,000 | 511 | release |
| wikidata-20261026 | 433 (a stream's tail) | 10 | 10 | release, debug |
| text-l1 (bzip2 -1) | 0 | 99,981 | 512 | release, debug |
| random, period-3 | 0 | 899,981 | 512 | release |

The debug build runs with debug assertions and overflow checks on.

**A VM sweep, indicative only** (`vm-sweep.txt`: 20 walks a config, one pinned vCPU of a
shared 4-vCPU VM with a 260 MB L3; no counters; not the i9's figures):

| ns a row | 1 lane | 8 | 16 | 32 |
|---|--:|--:|--:|--:|
| A (bare) | 20.0 | 4.50 | 2.36 | 2.26 |
| B (+ prefetch) | 17.6 | 3.76 | 2.23 | 1.47 |
| C (+ flag test) | 19.0 | 4.16 | 2.25 | 1.57 |
| D (+ stores) | 19.4 | 4.10 | 2.35 | 1.99 |
| P (shipped) | 21.6 | 4.29 | 2.37 | 2.05 |
| S (assembly) | | | | 0.056 |

**What the VM suggests:**
- At 32 lanes, prefetch is worth 35% (A→B).
- The flag test costs about 7% (B→C).
- The stores cost 27% (C→D).
- Production's bookkeeping beyond D costs 3% (D→P), of which assembly is about 3 points.

These are hypotheses for the i9 to test, not findings. The VM's cache keeps the whole
table resident, which the i9's 20 MB L3 under four cores may not.

## 4. The counter availability audit (run on the i9)

```
sudo -v && just walker-counter-audit 2>&1 | tee walker-audit-console.log
```

About a minute. Upload `walker-audit-*.json`, `walker-audit-*.md` and the console.

**What it does:**
1. Checks the corpus's hash.
2. Builds the ladder, and runs its correctness check (§3).
3. Records the environment, as numbers only: CPU family, model and stepping; logical
   CPUs, physical cores and threads a core; `perf_event_paranoid`; `nmi_watchdog`.
4. For each event below, records whether `perf list` has it and whether it counts (non-zero)
   on a short ladder run.
5. Runs each event group with cycles and instructions, and records whether every member
   counts and whether any is multiplexed.

**No figure from these runs is kept;** they exist only to give the events something to
count. With SMT on, the i9 has four programmable counters a logical CPU, so each group has
at most four events.

**The plumbing was exercised on the VM** with a stand-in `perf` that reports one event
missing, one unsupported and one group multiplexed. The audit reported all three correctly.
The real VM has no PMU, so it was a parsing check only.

**The result** (`2026-11-01-walker-ladder/i9-10900K/`: `walker-audit-20261101T232121Z.json`
and `.md`, and the console):
- **Environment:** CPU family 6, model 165, stepping 5; 20 logical CPUs on 10 physical
  cores, 2 threads a core; `perf_event_paranoid` 4; `nmi_watchdog` 1.
- **The ladder's correctness check passed on the i9:** every rung at every lane count, and
  the output fnv `27d7635e0e1a9555`, as on the VM.
- **Every one of the 22 events** (20 candidates, plus cycles and instructions) is in
  `perf list` and counts on a run.
- **Every group of four (with cycles and instructions) multiplexed.** The programmable
  events ran 82–86% of the time, cycles 65–66%.
- **The likely cause:** the NMI watchdog. On Intel it holds the fixed cycles counter, so
  perf's `cycles` takes one of the four programmable counters. Four events plus cycles is
  five for four.

**The fix, with no change to the machine's settings:**
- Groups of at most three events, so with cycles they fit four programmable counters.
- The alternative, disabling the watchdog for the capture (`kernel.nmi_watchdog=0`), is the
  maintainer's to choose. It is not needed.
- **A preflight** (below) checks the fix on the i9 itself, before the capture spends any
  time. It is exercised on the VM with the stand-in `perf`, both failing and passing.

**The candidate events as audited, in their original groups of four** (superseded by §5's
groups of three):

| group | events, each with cycles and instructions | for |
|---|---|---|
| exec | `uops_issued.any`, `uops_retired.retire_slots`, `br_misp_retired.all_branches`, `idq_uops_not_delivered.core` | issue-width share, front end, mispredicts |
| mlp | `l1d_pend_miss.pending`, `l1d_pend_miss.pending_cycles`, `l1d_pend_miss.fb_full`, `cycle_activity.stalls_l1d_miss` | demand MLP (Intel's: pending ÷ pending_cycles), fill-buffer exhaustion |
| stalls | `cycle_activity.stalls_l2_miss`, `cycle_activity.stalls_l3_miss`, `cycle_activity.stalls_total`, `resource_stalls.sb` | where loads stall; the store buffer |
| pf_tlb | `dtlb_load_misses.walk_active`, `sw_prefetch_access.t0`, `load_hit_pre.sw_pf`, `mem_load_retired.l1_miss` | page walks; whether the prefetches arrive before the demand load |
| loads | `mem_load_retired.fb_hit`, `mem_load_retired.l2_miss`, `mem_load_retired.l3_miss`, `resource_stalls.any` | where demand loads are served |

After the audit: no event was dropped, since all count. Every group was split to at most
three events. The `l1d_pend_miss` MLP pair stays in one group, with fb_full.

**What the audit cannot show:** demand MLP misses MLP that software prefetch has already
supplied. A low demand MLP with prefetch on does not mean low overlap (the researcher's
point). So the protocol reads `load_hit_pre.sw_pf` and `mem_load_retired.fb_hit` beside it,
and compares B with A.

## 5. The capture (frozen after the audit; not authorized)

```
sudo -v && just bench-walker-ladder 2>&1 | tee walker-ladder-console.log
```

**What the recipe does:**
1. Checks the corpus's hash.
2. Builds the ladder, and runs its correctness check.
3. Runs `bench/walker_ladder.py run FILE --rounds 3 --reps 100` with
   `WALKER_PERF="sudo perf"`.

Upload `walker-ladder-*.jsonl`, `walker-ladder-*-summary.json` and the console.

**The frozen event groups**, each with cycles and instructions:

| group | events | for |
|---|---|---|
| exec | `uops_issued.any`, `uops_retired.retire_slots`, `idq_uops_not_delivered.core` | issue-width share (D3), the front end |
| branch_sb | `br_misp_retired.all_branches`, `resource_stalls.sb`, `resource_stalls.any` | mispredicts; the store buffer (rung D's stores) |
| mlp | `l1d_pend_miss.pending`, `l1d_pend_miss.pending_cycles`, `l1d_pend_miss.fb_full` | demand MLP and fill-buffer exhaustion (D4) |
| stalls_l1 | `cycle_activity.stalls_l1d_miss`, `cycle_activity.stalls_total` | the share of stalls on an L1 miss |
| stalls_l2 | `cycle_activity.stalls_l2_miss`, `cycle_activity.stalls_l3_miss`, `dtlb_load_misses.walk_active` | deeper misses; page walks |
| pf | `sw_prefetch_access.t0`, `load_hit_pre.sw_pf`, `mem_load_retired.l1_miss` | whether the prefetches arrive before the demand loads |
| loads | `mem_load_retired.fb_hit`, `mem_load_retired.l2_miss`, `mem_load_retired.l3_miss` | where demand loads are served |

**The preflight:**
- Before any figure, each group runs once on a short walk.
- If any event is uncounted, or runs less than 99.9% of the time, the capture stops there.
  It makes no figures, and prints which event and group failed.
- So a grouping the i9 cannot count unmultiplexed costs seconds, not a void capture.

**One core:**
- Every config (A–D and P at 1, 8, 16 and 32 lanes, and S32) is run once per event group,
  pinned to one CPU of the first physical core.
- The order is shuffled by round, with a fixed seed. There are 3 rounds.
- **The walks:** 100 a run, 25 at one lane, where a walk is about ten times slower.
- **Counts a row:** each config also runs at 0 walks, the setup alone (decoding one block
  and building its tables). The counts are (at REPS − at 0) ÷ (REPS × rows).
- Times come from the ladder's own clock around the walks.

**Four cores:** A32, B32 and P32 as four copies at once, one per physical core. Each copy
builds its own T-vector, so there are four tables, as four workers keep. Groups exec and
mlp; 3 rounds.

**About 7 minutes in all** (seven groups). The output is `walker-ladder-<time>.jsonl` and a summary.
`--no-perf` runs the same plan timed only. That is the VM's dry run: it completed, and
with too few walks it was void by G1, as it should be.

## 6. Gates, decisions and stopping rules (frozen)

All figures are medians over the rounds.
- c(X, k) is cycles a row, rung X, k lanes.
- bare = the least of c(A, k) and c(B, k) over k: the best bare chase, prefetch or not.

**The gates:**

| gate | rule | if it fails |
|---|---|---|
| G0 | every walk checked; every run pinned; no counted group multiplexed | void |
| G1 | P32's ns a row, max ÷ min over its runs, at most 1.05 | void |

**The decisions:**

| | rule | decision |
|---|---|---|
| **D1** | c(P, 32) ≤ 1.15 × bare | the shipped walk's scheduling and bookkeeping are not the limit: **no walker scheduling rewrite** (the stopping rule of report 134) |
| **D2** | c(P, 32) ≥ 1.5 × bare | name the step, from B→C, C→D and D→P at 32 lanes, where most of the gap opens; any rewrite is of that step only, with its own budget, after review |
| between | 1.15 < ratio < 1.5 | reported; no rewrite on this evidence |
| **D3** | P32's uops issued a row ÷ 4 is at least 0.70 × c(P, 32) | execution-bound: any compact format must be nearly instruction-neutral (this would confirm FOR-14's premise) |
| **D4** | demand MLP of B rises less than 10% from 16 lanes to 32, while fb_full's share of cycles at least doubles | miss capacity is the limit: no more lanes |
| **D5** | four-core P32/B32 differs from one-core P32/B32 by more than 15 points | the isolated-core conclusions do not transfer: report it and stop |

**What none of them licenses:**
- no lane-count sweep beyond 1, 8, 16 and 32;
- no new walker variant;
- no format work.

The diagnostic ends with its summary. The next architecture is chosen from the measured
bottleneck (report 135), after review.

**What the decisions rest on:**
- The 1.15 and 1.5 bands are judgement, set before any measurement.
- D3's 4-wide issue is the core's documented width.
- D4 needs `l1d_pend_miss.fb_full`, which the audit confirmed counts.

**A prior from FOR-14's i9 confirmation** (2026-11-01-for14-stage1.md §7a, not a
prediction this protocol gates on):
- At one block, h's 22 extra instructions a step cost about 1.40 ns a row, about 7 cycles,
  over o's 1.275 ns.
- So the shipped walk had little idle issue capacity to absorb added work. That favours
  D3, or an instruction-window limit (the researcher's second row), over spare miss
  capacity.
- At four blocks, o's walk lost 56% a row and the half-sized table's only 8%. So D5's
  four-core check is where the memory side should show.

## 7. Cost so far

About two-thirds of the authorized engineer-day:
- the ladder, its checks, the driver, the audit, this draft;
- one VM sweep;
- the dry runs (no-perf, and a stand-in `perf` to exercise the parsing);
- the regrouping and preflight after the audit.

**i9 time used:** the one-minute counter audit. No capture.
