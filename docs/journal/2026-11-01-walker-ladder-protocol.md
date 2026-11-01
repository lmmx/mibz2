# The minimal real-T-vector walker diagnostic: preparation and draft protocol (for review)

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
| counter availability audit | §4: a recipe for the i9, not yet run; the VM has no PMU |
| protocol | §5, a draft: its event groups are fixed only after the audit |
| stopping rules | §6 |

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

## 4. The counter availability audit (the i9; recipe ready, not run)

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

**The candidate events** (Skylake-derived names, for the i9's Comet Lake core):

| group | events, each with cycles and instructions | for |
|---|---|---|
| exec | `uops_issued.any`, `uops_retired.retire_slots`, `br_misp_retired.all_branches`, `idq_uops_not_delivered.core` | issue-width share, front end, mispredicts |
| mlp | `l1d_pend_miss.pending`, `l1d_pend_miss.pending_cycles`, `l1d_pend_miss.fb_full`, `cycle_activity.stalls_l1d_miss` | demand MLP (Intel's: pending ÷ pending_cycles), fill-buffer exhaustion |
| stalls | `cycle_activity.stalls_l2_miss`, `cycle_activity.stalls_l3_miss`, `cycle_activity.stalls_total`, `resource_stalls.sb` | where loads stall; the store buffer |
| pf_tlb | `dtlb_load_misses.walk_active`, `sw_prefetch_access.t0`, `load_hit_pre.sw_pf`, `mem_load_retired.l1_miss` | page walks; whether the prefetches arrive before the demand load |
| loads | `mem_load_retired.fb_hit`, `mem_load_retired.l2_miss`, `mem_load_retired.l3_miss`, `resource_stalls.any` | where demand loads are served |

After the audit, events that are missing or don't count are dropped, and groups that
multiplex are split. The groups are then frozen with the protocol.

**What the audit cannot show:** demand MLP misses MLP that software prefetch has already
supplied. A low demand MLP with prefetch on does not mean low overlap (the researcher's
point). So the protocol reads `load_hit_pre.sw_pf` and `mem_load_retired.fb_hit` beside it,
and compares B with A.

## 5. The capture (draft; frozen after the audit; not authorized)

```
python3 bench/walker_ladder.py run FILE --rounds 3 --reps 100     (with WALKER_PERF="sudo perf")
```

A recipe will be added when the protocol is frozen.

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

**About 5 minutes in all.** The output is `walker-ladder-<time>.jsonl` and a summary.
`--no-perf` runs the same plan timed only. That is the VM's dry run: it completed, and
with too few walks it was void by G1, as it should be.

## 6. Gates, decisions and stopping rules (draft)

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
- D4 needs the audit to confirm `l1d_pend_miss.fb_full`. Without it, D4 is read from MLP
  alone, and says so.

## 7. Cost so far

About half the authorized engineer-day:
- the ladder, its checks, the driver, the audit, this draft;
- one VM sweep;
- the dry runs (no-perf, and a stand-in `perf` to exercise the parsing).

No i9 time has been used.
