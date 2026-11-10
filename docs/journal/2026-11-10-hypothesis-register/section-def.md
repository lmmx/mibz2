# Register, sections (d), (e), (f), redone as of 2026-11-10

Read only. Nothing was built or run. Every figure is copied from the file named beside it.
"(inferred)" marks my own inference or arithmetic.

**What the slice holds.** Sections (d), (e) and (f) of the 30 October register are P90 to P134,
P140 to P163 and P170 to P181: 81 IDs. The K-series (K1 to K63) is not section (e). It is the
register's section 4, the contradictions. It is covered here too, as 63 further rows at the end
of table (e), because the brief asks which K findings later runs contradict. K rows whose subject
belongs to sections (a) to (c) were not set against the later record in this slice and say so.

**Source abbreviations** (all under `docs/journal/` on this branch unless a ref is given):

| abbr. | file |
|---|---|
| REG | the 30 October register (`scratchpad/portfolio.md`); PM is its companion `portfolio-main.md` |
| WS | `2026-10-28-workstation-scaling.md` |
| EW | `2026-10-28-early-warning.md` |
| CD | `2026-10-29-cooperative-decoder.md` |
| BC | `2026-11-05-binding-census.md` |
| CP | `2026-11-05-corun-placement.md` |
| JH | `2026-11-05-job-handoff.md` |
| SC | `2026-11-05-split-condvars.md` |
| BP | `2026-11-05-bench-precision.md` |
| WIN | `2026-11-05-window.md` |
| TS | `2026-11-06-team-size.md` |
| 3T | `2026-11-06-three-team-policy.md` |
| GPP | `2026-11-07-general-plan-policy.md` |
| KM | `2026-11-07-knee-mechanism.md` |
| TPR | `2026-11-07-team-pipeline-result.md` |
| PLR | `2026-11-07-pipeline-levers-result.md` |
| PB, PBY, PL | `2026-11-08-plan-boundaries-result.md`, `-plan-bytes-result.md`, `-plan-levels-result.md`, read with `git show origin/research/general-plan-impl:docs/journal/…` |
| SCR | `2026-11-08-service-core-result.md`, read from `origin/research/service-core` |
| S23 | `2026-11-09-service-2x3-result.md` |
| SLA | `2026-11-09-service-lookahead-result.md` |
| SPIN | `2026-11-09-spin-audit.md` |
| RFLci, RFL1, RFL2 | `2026-11-09-run-fed-leader-result.md`, `-workstation-result.md`, `-second-attempt-result.md` |
| 2M | `2026-11-10-two-modes.md` |
| BW | `2026-11-10-bandwidth-titration.md` |
| SN | `2026-11-10-share-nothing.md` |
| PS | `2026-11-10-parallel-scan.md` |
| PSR | `2026-11-10-par-scan-result.md`, read from `origin/master` (not on this branch) |
| BDC | `2026-11-10-by-data-class.md` |
| CL | `CHANGELOG.md` |

**The policy as it stands on 2026-11-10** (GPP "Today's policy, exactly"; CL 0.2.6, 0.2.7):
below 6 physical cores, or topology unread, N plain workers; exactly 10 physical cores, a 20 MiB
L3 and 10 or more threads, three teams of three, pipelined with park and small; otherwise teams
of two, min(⌊c/2⌋, round(0.7 × L3 / 3.6 MB)). Every plan also runs one scanner thread and the
caller's reader.

### (d) Scheduling, concurrency and the knee

| ID | idea, one line | the advantage it claims, as a quantity that could be false | status 2026-11-10 | closed by: construction / advantage / n.a., with the figure | whole-loop price | cheapest falsifier now | bearing on share-nothing | sources | premise the verdict rests on | premise still true? |
|---|---|---|---|---|---|---|---|---|---|---|
| P90 | Cooperative inversion: k cores share one block's table | The default at all CPUs is at least 1.15 times 4 plain workers on the i9, full decoder | shipped | n.a. Measured 1.257 (1.197–1.320), i9, unpinned, 20 rounds (CD). The reference was plain workers on 3.6 MB tables with one scanner | counted: the default ten-thread plan 38.0 instructions a decoded byte against one thread's 26.7; walk 17.8 against 10.1; build 9.2 against 6.2 (SN §1, callgrind, bias stated) | i9 recipe: plain workers with the finder on `lbzip2 -2` Wikidata at 2 to 10 (SN §4.2). It prices what would replace teams | removes cooperation: teams exist because four 3.6 MB tables fill the L3 | REG; CD; SN; PSR | "four of them fill the cache": a 3.6 MB table, a serial scanner, the best alternative being 4 plain workers | yes at level 9 (PSR: plain workers peak at 4, 1,145 MB/s, 631 at 10). no for small tables: level 1, ten plain workers with the finder 2,563 MB/s, CPU 0.66 → 0.84 s (PSR). Teams against plain workers with the finder at levels 1 to 6 was never run |
| P91 | Team shape: 2 against 3 cores a table | 4×2 is at least as fast as 3×3 (1.439 against 1.375 of p4, research `y` kernel, T-vector phase) | superseded | advantage: on the 0.2.5 build 3×3 at 10 threads is 1.089 (1.075–1.103) and 1.088 (1.076–1.101) times 4×2, CPU 0.88–0.89, RSS 0.81–0.82 (3T) | counted for 3×3 pipelined: 38.0 instructions a byte (SN §1). 4×2: unknown | read of an archived bundle: `plan-bytes/20261108T093442Z`. Plain 3×3 loses to 4×2 at level 6, 0.916 [0.890, 0.943], and level 7, 0.957 (PBY) | none: a choice among cooperative plans | REG; 3T; TS; GPP §1.1; PBY | "4x2 led 3x3 in PR 0", and the full-decoder tie 1.008 (0.994–1.023) on the 29 October build | no. moved: "The three-table advantage appeared between builds" (GPP §1.1). At four tables width buys 1.017 and 1.003 (TS) |
| P92 | Policy at 6 threads (3×2) | 3×2 at 6 threads is no slower than 4×2 at 8 or more (1,071 against 1,050 MB/s, "within noise") | open | n.a. Partly read: forced 3×2 against 4×2 at 8 threads is 0.989 [0.969, 1.009], UNRESOLVED, with CPU 1.097 → 0.850 s and RSS 82 → 67 MB; 2×4 pipelined is 1.079 [1.055, 1.102] (PB) | unknown | i9 recipe: a 6-thread cell, 3×2 against 2×3 pipelined against 6 plain workers. None is registered | none while tables are 3.6 MB; moot if small tables remove teams | REG; CD; PB; GPP §1.1 | one release-candidate run with load 4.3 at the start (CD) | moved: the 4×2 it was compared with is "dominated on this machine wherever a pipelined plan fits" (PB). 3×2 on the current build at 6 threads against any alternative: never run |
| P93 | The L3 budget is per L3 domain (multi-CCD AMD) | `Plan::for_threads` reads the L3 of the domain its threads run in | open | n.a. Code reading since: the exception reads "an L3 read from cpu0"; "`physical_cores` counts both sockets' cores; `l3_bytes` reads one socket's cache" (GPP). No multi-domain machine was measured | unknown | none on the hardware held: "The L3 can't be shrunk", no `cat_l3` (GPP §4). A second machine is needed | none: a small table removes the budget this tunes | REG; GPP; README "Unmeasured topologies" | "Untested; … not checked" | moved: now checked by reading. The mismatch exists on two-socket parts and is documented, not fixed |
| P94 | No two team threads on SMT siblings | Teams counted on physical cores beat teams on siblings | shipped | n.a. Ten teams of 2 on siblings 0.838 against the policy's 1.257 (CD). Packed against spread at 4×2: 1,055 against 1,369 MB/s, −23%; 4×3 −8%; 4×4 −1% (TS, CPU masks, 6 rounds) | unknown | i9 recipe: one thread a core by affinity against unpinned, in the timed harness (TS "What next" 2; KM H2, deferred "until per-thread affinity records exist") | removes cooperation only in part: plain workers still need one worker a core | REG; CD; TS; KM E6; 2M §5 | "siblings cost ~1.43× CPU a block" (P109, two plain workers) | yes in direction. But the rule is a count, not an affinity: "Unpinned, the scheduler may still place two of them on one core" (TS). 2M lists sibling packing (H2) as one candidate for the slow mode |
| P95 | Cooperative kernel at production's per-core speed | 4x1 ≥ 0.97 × p4 (T-vector phase, i9) | superseded | construction: gate not met, r4x1 0.923 × p4. The advantage, parity a core, was never reached and never disproved | counted: cooperative walk 16.9 and 17.1 instructions a byte on 29 October (REG, perf, i9); 17.8 on 10 November against the one-thread walk's 10.1; build 9.2 against 6.2 (SN §1) | counting probe: callgrind of `coop::traverse` against `bwt::room_segments` on one block at k = 1, on the VM | removes cooperation: this gap is the cost share-nothing avoids | REG P95, §6; SN §1; `src/coop.rs`, `src/bwt.rs` | "The remaining gap is all the build: 9.28 instructions a byte against production's 6.65" | no. moved: the build gap is unchanged (9.2 against 6.2) but the walk gap opened. Production's walk fell 16 → 9 a step with #157 (REG §6) and was staged by #171 (CL 0.2.2); "The cooperative and flagged walks: the same rolled loop. A port waits" (REG §6). `src/coop.rs` has no `TILE`; `src/bwt.rs` `room_segments` has (inferred from grep). See Mined 1 |
| P96 | Residency against overlap: why 3x3 wins | 4x2 wins by overlap; 3x3 by overlap and residency (hit share 0.967 against 0.945; reads in flight 41.9 against 28.1) | validated | n.a. T-vector phase, i9, old kernel, 588 runs. Full decoder since: per-table rate 558–583 MB/s at 2 tables, 461–466 at 3, about 320 at 4 (TS); plain 3×3 against 4×2 falls from 1.087 at level 9 to 0.916 at level 6, "PREMISE SUPPORTED" (PBY) | unknown | read of an archived bundle: `plan-bytes/20261108T093442Z` | shrinks the table: residency is what a small table buys without a team | REG; TS; KM E1–E3; PBY | raw, prefetch-inclusive hit shares, later "unreconciled" (P151) | moved: the kernel changed, and "there are no counter measurements of the cooperative decoder" (KM E2). The overlap half is weak on the full decoder: width beyond two buys 1.017 and 1.003 at four tables (TS) |
| P97 | Teams are instruction-bound | Fill-buffer-full 0.07–0.08 a cycle against p4's 0.24 | superseded | n.a. "neither label is established" on the shipped kernel (0.194) | counted: 38.0 instructions a byte for the default plan (SN §1) | i9 recipe: `just bench-bw-titrate`, frozen in `bench/bw-titrate.frozen` (BW §4). Reads BOUND, NOT BOUND or MIXED | removes cooperation: plain workers do "one thread's work exactly", 26.8 (SN §1) | REG; SN; BW; 2M | the old two-pass kernel's counters | no. What binds the default plan at ten is unresolved: bandwidth is a hypothesis (BW), and in the fast mode ten threads are "68% busy … (1.08 CPU-seconds in 0.159 s)" (SN §3) |
| P98 | Cross-core line handoffs cost | HITM is a material share of cycles in teams | rejected | advantage, for loads only: HITM 0.33 (4x2), 0.84 (3x3) a KB, "under 1%" of cycles, T-vector phase, old kernel. Store-side RFOs were never counted | unknown | i9 recipe needing counters: `perf c2c`. Blocked: `perf_event_paranoid` is 4 on the workstation (BW §1) | removes cooperation: no shared table, no handoffs | REG; BW §2 | "under 1%" at 4x2 and 3x3, pinned, 29 October | unknown for the pipelined 3×3. The simulated ledger puts 35.5% of cold written lines in `coop::FusedBuild::share` (BW §2), the store side the verdict left out; it counts last-level misses, not handoffs |
| P99 | Pool wake-up latency | Helpers that never park make a team faster (helper start 117–337 µs alone against 7–11 µs in the grid) | open | n.a. The walk-ready waits were tested: park against spin-and-yield 0.999 [0.981, 1.017] speed, CPU 1.055 [1.039, 1.071]; 4,096 → 2,000 spins NULL (PLR). The pool's own idle and join waits were not ("not the pool's", SPIN §4) | derived: at most about 3.3 M spins a decode at 3×3, about 0.10–0.13 CPU-s on the i9 by an assumed PAUSE cost (SPIN §3) | counting probe: `wait_audit` (feature `wait-stats`) on the i9, untimed, to count parks at `pool_helper_idle` and `pool_caller_join` | removes cooperation: no pool, no wake-ups | REG; PLR; SPIN | a single block alone, k = 2–4, 1.57–2.61 ns/B against 0.96 | moved: 94–100% of each site's spins belong to waits that parked anyway, on a 4-vCPU VM (SPIN §3). On the i9 it is not measured |
| P100 | L3-capacity cap rule: tables ≈ 0.7 × L3 | Adding a table past 14.4 MB of live T-vectors stops paying, at any block size | shipped | n.a. Still step 3 of `plan_with`. At ten cores it is overridden by the 3×3 exception | unknown | read of archived bundles: `plan-bytes` and `par-scan` | shrinks the table: SN sizes its target from this knee ("The knee sits at 14.4 MB live") | REG; WS §9; TS; GPP §1.2; PBY; PSR | "the -9 and -6 knees both sit at an aggregate 14.4 MB": plain workers, 28 October build, window 2 a thread | yes for plain workers at level 9 (PSR: peak at 4 workers). no for teams on the current build: "The policy's cap is one table too many on this machine now … the grid's best has 3 (10.8 MB, about 0.5 of the L3)" (TS). The recorded condition, a second machine, is not met |
| P101 | mibz2 worker cap below 6 cores; autodetect | A cap below the thread count is faster or cheaper on small machines (9V45: 2 spread threads 826 MB/s against 808 at 4 on siblings) | open | n.a. Autodetect was never built. Teams below 6 cores are closed: "0.71× the workers at 4 threads" (BC) | unknown | CI A/B: every bench run already has 2 and 4 threads; the question is which is the default | cuts CPU a byte, where threads exceed cores | REG; BC; WIN; CL 0.2.7 | 2 threads tie 4 on a 2-core runner, at 2.11× CPU a byte | moved: #180's window made 4 threads 8.1–10.3% faster in wall for 4.3–6.1% more CPU (WIN). 0.2.7 on a 9V45: 0.201 s at 2 threads, 0.182 s at 4 (CL) |
| P102 | Level-aware team cap formula | More teams of two at lower levels are faster: -3 1.092, -6 1.006 | rejected | advantage, at level 6 only and for teams of two on the October build: 1.006 (0.990–1.021), not met | unknown | read of archived bundles: `plan-levels/20261108T094232Z` and `plan-bytes` | none: a cooperative tuning | REG; GPP §1.6; PL; PBY | five teams of two on all ten cores tie four at level 6 | moved: the plans changed. At level 6, 3×3 pipelined is 1.163 [1.136, 1.191] times 4×2 (PBY); at levels 1 and 2, 4×2 is 1.30–1.32 times the 0.2.6 default (PL). The registration's arithmetic slip (K41) stands |
| P103 | Teams on every physical core at level ≤ 3 | -1 1.150 (1.138–1.162), -2 1.216 (1.193–1.241) against 4 teams | conditional | construction: #115 failed on a mixed-level file, 0.945 (0.930–0.959), and a -6 control. The research advantage stands and has grown into a shipped regression (P105) | unknown | i9 recipe: the registered fix, "A team whose block is below `MIN_COOP` lets its helpers take small blocks of their own" (PL). Not built | removes cooperation at low levels: "At levels 1 and 2 a team is one core" (GPP §1.6) | REG; GPP §1.6; PL; PSR; README | "The plan is chosen once, before the first block", and "the scanner binds first" at -1 to -3 (CD) | no, both moved. The recorded condition, "A design that re-plans, or spawns helpers only when they can run", is met in design: "The decision is made per block, so a mixed-level file needs no header read and no gate, unlike #115" (PL). The finder lifts the scanner cap: 1.615 at ten workers on level 1 (PSR) |
| P104 | One core a block at -2/-3, over the RSS budget | -3: 6 workers 1.148 at 0.79× CPU and 1.26× RSS; -2: 6 workers 1.355, 8 workers 1.362 | conditional | n.a. It waited on the maintainer. Adoption policy is now delegated to the team as a class (CLAUDE.md, 2026-11-09) | counted: plain workers do 26.8 instructions a byte at level 9 (SN §1). Level 1 to 3: unknown | i9 recipe: `par_scan … knee` on level-2 and level-3 input, find against scan | removes cooperation: this is plain workers on small tables | REG; SN; PSR | the 6-to-8-worker plateau, and an RSS cap of 1.15× | moved: the plateau was the scanner. Scan arm 1,628, 1,600, 1,598 MB/s at 6, 8, 10; find arm 1,842, 2,429, 2,563 (PSR). RSS under the finder design in the library is not measured: the harness holds input in memory |
| P105 | The default at 6 threads runs at 0.76–0.83× its 20-thread speed at levels 1–5 | The same loss appears wherever a plan runs three decoders on small blocks | validated | n.a. Measured: 0.2.6's default against 0.2.4's is 0.763 [0.752, 0.775] at level 1 and 0.776 [0.759, 0.794] at level 2, 10 threads; forced to 4×2 it recovers, K 1.320 and 1.297 (PL). At level 3 it is 1.047 [1.015, 1.081] faster | unknown | none needed for the finding. The fix needs an i9 recipe | removes cooperation: "the three teams leave six of the ten threads waiting" (README) | REG; GPP §1.6; PL; CL 0.2.6; README | `auto`@6 against `auto`@20 on the 29 October build | yes at levels 1 and 2. no at level 3. Levels 4 and 5: not re-measured. The fix is "being registered", not built |
| P106 | Phase-aware gating: at most K workers in `invert` | Gating the T-vector phase beats the plain worker cap (gate6 1.13 at 8 workers, 1.21 at 10 against ungated) | superseded | construction: a condvar gate over ten full workers; best gated 782 MB/s, 0.77× of 1,019. The advantage, fewer tables in inversion while serial stages run elsewhere, was later shown by another construction: svc2@10 against the default 1.045 [1.017, 1.073], CPU 12.7% less (S23) | unknown | read of an archived bundle: `service-2x3/20261109T010708Z` | removes cooperation in part: the service core takes serial work off the table's threads; plain workers on small tables need no gate | REG; WS §8; SCR; S23; SLA | "That points at total live footprint (the waiting workers' scratch, the wider window) rather than active walks alone (inference)" | moved: the inference was never tested, and the service core holds two tables with 64 MB peak RSS against the default's 74 (S23). See Mined 9 |
| P107 | The admission window starves workers | A wider window recovers idle worker time; bound 7–8% of worker time | shipped | n.a. The rejection was overturned. #180: window 2 → 4 blocks a thread; `wikidata` 4-thread wall −8.1%, −10.3%, −8.9% on three CI runs, CPU +4.3% to +6.1%, peak RSS +8 MB (WIN) | unknown | i9 recipe: 4×2 at window 8 against 16, and 3×3 at 6 against 12, through `parallel_planned(r, plan, slots, kernels)`. Never run at ten threads | shrinks other live memory, in reverse: each extra slot reserves 2.7 MB (WIN) | REG; WIN; BC item 8; TS; BW §5; SLA | "slot waits are 7–8% of worker time, an upper bound"; "The 8/16/32-slot counterfactual was never run" | no: the counterfactual ran and gained more than the bound. Now "Supply is exhausted at window 16" on CI (BC). At ten threads on the i9 the window's cache cost is "unpriced" (BW §5). See Mined 6 |
| P108 | Run-queue delay inflates block times | Run-queue wait is a material share of a block's time | rejected | advantage: about 6% of a block's time on CI, steal 0% | unknown | none needed | none | REG; SPIN §1; WIN | "The cost is on a CPU, not waiting for one" | yes on the i9: `runq_s` 0.15 ms fast, 0.40 ms slow, in runs of 140–370 ms (SPIN). On CI the reader is often not running: a ready block waits 1.98 ms against a 2.19 ms decode (WIN) |
| P109 | CI's four-thread ceiling is SMT | Spread over packed +40.5% (9V74), +39.6% (7763), +39.0% (i9) at 2 threads | validated | n.a. Refined by stage: four pinned copies inflate the walk ×2.12 (7763), ×2.22 (9V74), ×1.46 (8370C) (BC) | unknown | none needed | none | REG; BC; CP | two pinned plain workers | yes. "SMT buys only 1.13–1.24× from 2 to 4 threads" (BC) |
| P110 | The full-occupancy remainder (0.81) | CPU a block rises 14% with both cores busy, from something other than SMT | open | n.a. At 2 threads, one worker a core, whole-decoder amplification is 1.04–1.11 (BC) | unknown | CI A/B: the sibling-pair matrix with a cross-core control (BC; CP). Not in the record as run | none | REG; BC; CP | "CPU per block +14%" on a 7763 at 0.1.x | moved slightly: 1.04–1.11 on the 5 November decoder. The mechanism is "contested" (BC) |
| P111 | Shared L3 or bandwidth binds on CI's four copies | Each copy's walk doubles with four copies because of a shared memory limit | open | n.a. Weakened: "Kernels 3.3× apart in line rate all lose ×2.12–2.16 at four copies on the 7763" (BC), which a line-rate limit does not predict; the rival reading is lines in flight (walk MLP 9.3–9.8 alone, about 5.5 at four copies) | unknown | CI A/B: the sibling-pair matrix. "Core-side: the ALU-loop sibling costs as much as a walking one" (BC) | none | REG; BC; CP; BW §1 | "Four processes are not four threads"; no counters on CI | moved: the leading readings are core-side (BC; CP: Zen 3 and 4 partition the ROB and store queue). BW §1 says CI "has no bandwidth wall at 4 threads"; the record gives no measurement for that sentence |
| P112 | The many-core knee: L3 capacity, then queueing past the LLC | Past 4 plain workers on 3.6 MB tables the fall is stalls, not work: IPC 1.06 → 0.50, demand L3 hit share 0.76 → 0.38 | validated | n.a. i9 only, plain workers, 28 October. Reproduced in shape on 10 November without counters: peak at 4 workers, 1,145 MB/s, 631 at 10, CPU 0.58 → 3.69 s, in both arms (PSR) | counted: plain workers 26.8 instructions a byte at 10, as at 1 (SN §1). The knee adds stalls, not instructions | i9 recipe: `bench-bw-titrate` for the default plan; for plain workers none needed | shrinks the table: at 0.4 MB the CPU grows 0.66 → 0.84 s from 1 to 10 workers (PSR) | REG; WS §2, §7–§9; KM; BW; PSR; SN | counters read under `sudo perf` on 28 October | yes for plain workers at 3.6 MB. For the default plan the cause is not established: KM withdrew its memory-saturation claim; BW is a hypothesis; PSR "does not establish that memory is the cause". Counters are no longer available (P146) |
| P113 | Table bytes (block size) move the knee | The best worker count rises as blocks shrink: 4, 6, 6–8, then a plateau | validated | n.a. Strengthened. "consistent with the scanner bound … It isn't shown" (WS §9) is now shown: one scanner caps level 1 at about 1.6 GB/s and the finder gives 1.615 at ten workers (PSR) | derived: CPU-seconds a decode, 4 → 10 workers: ×3.50 at 3.6 MB, ×2.65 at 2.4, ×1.44 at 1.2, ×1.10 at 0.4 (WS §9) | i9 recipe: the same sweep on `lbzip2 -2` or `-3` input (SN §4.2) | shrinks the table: this is the evidence the objective rests on | REG; WS §9; SN §2, §3b; PSR; PBY | lbzip2-recompressed files; "Confounds: block count, tables, ratio and runs"; one process a run of 0.12–0.33 s | yes. Two cautions. The "about 1.6× at the knee, an upper bound" (WS §9) was scanner-capped. And at 1.2 MB ten workers lost to eight (P117; Mined 3) |
| P114 | One knee shared across decoders / rate per worker | Every decoder knees at the same thread count | rejected | advantage: lbzip2 keeps scaling to 8 and stays flat to 20 (WS §5) | unknown | none needed | none | REG; WS §5 | 6 rounds, i9, 28 October | yes. The rate-per-worker alternative is still open and untested |
| P115 | Past the knee, throughput ≈ 28 GB/s ÷ DRAM bytes a byte | T = W ÷ β predicts p10 572 (measured 567) and s10x1 747 (746) | conditional | n.a. An out-of-sample prediction now exists and misses: β = 13.1 from the simulated ledger gives 2.2 GB/s at W = 28.5 GB/s; "the i9 measures 1.50" (BW §2). The miss is in the direction of the model's stated bias | derived: 5.97 cold bytes written a decoded byte in the default plan, 12 with write-backs (BW, simulated) | i9 recipe: `bench-bw-titrate`. It is the recorded condition, "a probe of this workload's own mix" | shrinks other live memory if it reads BOUND: the ledger's candidates are cold writes, not table reads | REG; BW; PS as on master ("which left the memory cause unresolved") | "Correlation, not proof"; the 28–29 GB/s "is from the inversion-only harness of 29 October, not the decoder" (BW §1) | moved: untested on the decoder. At level 1 the decoder passes the modelled cap, 2.56 GB/s (PSR), so β is not a constant of the decoder |
| P116 | DDR4 four-activate window as a second knee | An output ceiling near 530 MB/s at 20 threads from tFAW | open | n.a. Modelled with single rank; the DIMMs are dual rank (K29) | unknown | i9 recipe: a random-lanes stressor or MLC `--loaded_latency`. Not registered | none: tables that stay in cache make few DRAM activates | REG; WS | single-rank arithmetic | no (K29). Nothing since bears on it |
| P117 | Request cost grows with cores at a high hit share | At -3, p10/p8 0.948 (0.937–0.959) with hit shares 0.940 against 0.929 | open | n.a. KNEE-ISO was never run. KM keeps it: "at level 3, 10 blocks keep a 0.940 hit share yet lose to 8 blocks, mainly in the build. A request or core-count limit is not ruled out" (E2) | unknown | i9 recipe: `par_scan FILE knee` on level-2 and level-3 input, find arm at 8 against 10 workers | shrinks the table, and bounds it: the 1.2 to 1.4 MB target is the size at which this was seen | REG; KM E2; WS §9; PSR; SN §3b | T-vector phase, pinned, no scanner, 29 October kernel | unknown. At 0.4 MB ten beat eight, 2,563 against 2,429 MB/s (PSR). At 1.2 MB the only figures are WS §9's 1,729 at 8 and 1,604 at 10, with a scanner. See Mined 3 |
| P118 | Write traffic past the knee | IMC write bytes an output byte rise 0.3 → 11.5–12.8 from 1 to 6–10 workers | open | n.a. Sharpened by a simulation: the default plan writes 19.6 M cold lines and reads 3.7 M; the T-vector build is 35.5% of cold writes, `memcpy` 20.3%, the walk's segment buffers 14.1%, `memset` 12.8%, RLE1's output 12.1% (BW §2) | derived: plain workers at 10, 3.60 cold bytes written a decoded byte, 16 MiB model (BW) | i9 recipe: `bench-bw-titrate`; then each candidate priced "in the simulator before any workstation run" (BW) | shrinks other live memory: at most 20% (no assembly copy), 13% (no last column), 9% (3-byte table), 6% (non-temporal output) of cold writes (BW §5) | REG; WS §7; BW | "hypothesis, testable" | unknown. The model "cannot rank plans whose difference is live tables" and its read columns "are lower bounds" (BW) |
| P119 | K blocks in lockstep per core | Interleaving K whole blocks hides latency: K = 2–8 gives 1.03–1.27× | rejected | construction: `bz2k walk` on the VM, before lanes; "K blocks' T-vectors are K × 3.6 MB" (`2026-10-27-compute-frontier.md`) | unknown | none needed while 32 lanes in one block supply the overlap (lanes16 0.86×, lanes8 0.71× at 10 workers, WS §6) | none | REG; compute-frontier; WS §6 | "K × 3.6 MB outgrows the cache as fast as K hides latency" | moved: a 0.4 MB table would not outgrow it, but lanes already do the hiding inside one block. A scanner version was counted out: "under 1.5× on the scanner, under 5% of a text decode" (BDC §4b) |
| P120 | AMAC / dynamic lane scheduling, direct table | Dynamic scheduling recovers walk imbalance: bounded at about 4% of a block's walk | rejected | advantage: bounded ≤ about 4%; "AMAC" is in the closed list at four copies on CI (BC) | unknown | none needed | none | REG; BC; PLR | "coordination is ~4% of a block's walk" | yes. The recorded conditions are not met: the pipelined leader's summed join delay is 14.3 ms over 233 team runs a decode at 3×3 (PLR), under 10% of a 1.39 ms block (inferred). A two-level format was built and failed on instructions, not on stalls (BDC §3) |
| P121 | AMAC-staged payloads for a two-tier format | Staging the second access of a two-tier table pays | conditional | n.a. A two-tier table now exists as built: design E, "98% one-access steps on Wikidata", 48 instructions a row against the room walk's 10.4 (BDC §3) | derived: any staging adds to a loop already 4.6 times the gate (SN §3a) | counting probe, only if a two-tier walk first comes within the gate | none | REG; BDC §3; SN §3a | "its stall profile being measured" | moved: the stall profile is still unmeasured (no counters), and the binding cost of the one two-tier table built is instructions |
| P122 | Batched segment dispatch | Fewer, larger segment hand-outs cut coordination (about 4% at 4x2 and 3x3) | conditional | n.a. Not run | unknown | counting probe: callgrind of `coop::traverse`, the segment take against the step, at k = 1, 2, 3 | removes cooperation | REG; SN §1; TS | "bounded by the measured 4%" | unknown. A team cuts a block into 512 × K segments (TS). SN §1 counts 7.7 more walk instructions a byte in the team; how much is dispatch is not split (Mined 1) |
| P123 | Work-optimal parallel inversion (Edwards and Vishkin) | Lower depth than a serial chain | rejected | n.a.: a judgement, class IV, never built: "depth is not the problem here" | unknown | none needed | none | REG | blocks are already decoded in parallel | yes |
| P124 | GPU rank-and-scatter inversion; scanned iMTF | A scatter-based inversion or a scanned MTF beats the chain | conditional | n.a. for the GPU scheme: no runs, about 14 B a row. The scanned iMTF's advantage is counted out: "Nothing is left for a chain to explain" (53 instructions and 0.67 mispredictions a move on text, BDC §4a) | derived: 14 B a row is 3.5 times the plain table | none needed | none: it grows the table | REG; BDC §4a; BW §2 | "pursued only if a cooperative arm's join … shows up as a cost" | partly met in a model: `memcpy` (assembly, the reader's copy, input chunks) is 20.3% of cold written lines (BW). The record's remedy for that is #183's ordered read, not this |
| P125 | Early warning: CPU amplification from 1 to 4 threads | A rise in 1 → 4 amplification flags sensitivity to a shared resource | shipped | n.a. Every CI bench reports it: 0.2.7, +1.0% at 2 threads and +2.1% at 4 (CL) | unknown | none needed | none; its i9 analogue is the exact CPU-seconds from 1 to 10 workers (PSR) | REG; EW; WIN; JH; BC; CL | on CI, amplification reads memory sensitivity | moved: on CI's 2 cores the 4-thread row is "CPU-capacity-bound" (BC) and amplification rises when workers are busier: +4.4% to +6.7% with #180 while wall fell (WIN); +3.5% (+0.6, +6.6) with the job hand-off (JH) |
| P126 | Contention fingerprint and its confound | A co-runner's slowdown fingerprints the decoder's memory demand | shipped | n.a. The confound is open | unknown | i9 recipe: the titration turns it round, a measured load beside the decoder: "at every level, 0 included, a hog costs one busy CPU and no cache" (BW §3) | none | REG; EW; BW | "More memory-level parallelism" and "less compute per byte" read alike | yes, still confounded. BW's hogs are the fixed-compute design the test asked for, applied to the aggressor. Not run on the i9 |
| P127 | Does a rise in CI's 2-spread amplification predict a lower knee on 10 cores? | Variants differing in footprint differ in few-core amplification | open | n.a. Answered on the i9 only: at -9 total CPU rises 1.36× from 2 to 4 workers, at -1 1.01× (WS §9); 0.636 → 0.853 s at level 9, 0.720 → 0.737 s at level 1 (PSR) | unknown | CI A/B: the level-1 and level-9 recompressions of one corpus at 1 and 2 spread threads | shrinks the table: a two-to-four-worker CPU reading prices a small table without a ten-worker run | REG; EW; WS §9; PSR | "two variants differing only in per-worker footprint" | the test exists on the i9 (the level ladder, with its confounds). On CI it was never run |
| P128 | Is lbzip2's 1 → 4 amplification a stable baseline? | It is a constant | open | n.a. "a control, not a constant" (1.11–1.42× by runner) | unknown | read of archived CI bench artefacts | none | REG; EW | as recorded | yes. Nothing since |
| P129 | Does the 2-core fingerprint order decoders as the knee does? | Same order | open | n.a. | unknown | CI A/B | none | REG; EW | none recorded | not in the record since |
| P130 | A twin that loops inside one process | A twin co-runner in one process is a cleaner load | conditional | n.a. | unknown | CI A/B | none | REG; EW; BP; 2M | "if the decoder gets much faster" | moved: v0.2.1 → master is −15.4% at 1 thread on a 7763 (BP). The twin's CPU screen was not re-read. The i9 record now separately wants decodes back to back in one process (2M §4) |
| P131 | Cachegrind / drcachesim LL-miss proxy per PR | A simulated last-level miss count warns of the knee | conditional | n.a. Built as a ledger: callgrind with a 256 KiB first level and a 20 MiB last level (BW §2) | derived: one simulation takes 2–6 minutes (BW) | counting probe: it is one | shrinks other live memory: it prices cold writes | REG; BW | "cachegrind serialises threads and ignores prefetch" | yes, as predicted: "concurrent random reads are under-counted"; "the write columns are the usable part" (BW) |
| P132 | USL α as a contention descriptor | A three-parameter fit describes the curve | open | n.a. | unknown | read of an archived bundle: TS's per-table rates at 2, 3, 4, 5, 6 and 8 teams | none | REG; EW; TS; GPP rules B, C | "at least six points" | six points now exist (TS). Nothing was fitted. GPP's rule C "would choose the wrong knee on the one machine measured" |
| P133 | Dirigent-style per-block progress against a solo profile | Per-block lag detects contention online | open | n.a. | unknown | read of archived occupancy traces (TS: blocks take 1.39, 1.38, 1.40 ms at k = 2, 3, 4) | none | REG; TS | inference, not pursued | not in the record since |
| P134 | Kernel+MTF 1,707 MB/s (pinned) against decoder 1,256 (unpinned): the gap | The gap has a nameable owner | superseded | n.a. Attributed in the main: "A team spends about 29–33% of each block outside the inversion call" (0.400–0.462 ms of 1.39 ms; KM E4), and acted on: the pipeline +4.2%, both levers +7.2% (TPR, PLR, CL 0.2.6), the service core beyond that (S23) | counted: scanner 3.5, MTF 4.4, everything else 3.1 of the default plan's 38.0 instructions a byte (SN §1) | none needed | removes cooperation: the leader's serial share is a cost of teams | REG; KM E4; TPR; PLR; TS | the research `y` kernel's 1,707 | moved: the placement share is a lead only, pinned spread 1,369 against unpinned 1,276 MB/s in a different binary (TS) |

### (e) Measurement findings

The first 24 rows are the register's section (e), P140 to P163. The K rows after them are the
register's section 4. In K rows "status" reads: validated, the recorded resolution stands;
superseded, a later record replaces both claims; open, unresolved.

| ID | idea, one line | the advantage it claims, as a quantity that could be false | status 2026-11-10 | closed by: construction / advantage / n.a., with the figure | whole-loop price | cheapest falsifier now | bearing on share-nothing | sources | premise the verdict rests on | premise still true? |
|---|---|---|---|---|---|---|---|---|---|---|
| P140 | Warm-repeat bias | A repeated stage call is faster than the first: walk 4.02 → 4.97 c/B in decode order on a 9V74 | shipped | n.a. Fixed in bench 2.0.0 | derived: none, an instrument | none needed | none | REG; BDC §1; CL 0.2.7 | "a stage table's shares are a runner's, not the decoder's" | yes, and extended: they are a data class's too. Walk 55% on Wikidata, 38% on text, 13% on records (BDC §1) |
| P141 | A/B order bias (head always first) | With the order alternated an A/A flags nothing: "0 of 42" | shipped | n.a. Fixed in bench 3.0.0 | derived: none, an instrument | read of archived CI runs: the three true A/A dispatches (BP) | none | REG; BP | "After the fix: 0 of 42" | no, not in general. Nine harness-only links: τ between runs 0.69% at 1 thread and 1.04% at 2, above one run's SE; "2 of 9 nulls look like changes". True A/A: 4 of 63 intervals exclude zero (BP). See Mined 11 |
| P142 | Per-thread CPU accounting bug | The sampler lost up to 50 ms a worker; exact totals 4 → 10 workers ×3.50, ×2.65, ×1.44, ×1.10 | shipped | n.a. | derived: none, an instrument | none needed | shrinks the table: SN §2 rests on these exact totals | REG; WS §9; SN §2; SPIN §1 | process totals from `getrusage` are exact | yes. Per-thread CPU is still coarse: "per-thread CPU comes in 10 ms ticks" (WS §9); SPIN reads the feeder "at the 0.01 s resolution". The even 2.2–2.4× growth across threads in the slow mode rests on thread totals as small as 0.020 s |
| P143 | Sampler cost | The sampler costs 1-thread runs +1.0% (+0.4, +1.7) | shipped | n.a. Fixed | derived: none, an instrument | CI A/B: sampler on against off at 4 threads | none | REG; BC; 3T | the fix removed the cost | moved: at 4 threads on CI the sampler "is pinned on the decoder's CPUs … where it costs +1.2% to +2.4% of wall with ±4% intervals" (BC). The i9 census thread's cost "is not measured separately" (3T) |
| P144 | Footprint and queue ledger (#70) | "The footprint lever is the T-vector per block × blocks in its phase, and nothing else" | shipped | n.a. The instrument shipped. Its conclusion is contradicted for teams at ten threads | derived: none, an instrument | i9 recipe: `bench-bw-titrate`; then the simulator a candidate | shrinks other live memory: a team's workspace is a 3.6 MB table beside 6.1 MB of segment buffers, 1.35 MB `rle` and 0.9 MB `ll` in capacity (KM E5) | REG; KM E5; TS; BW | VM, 4 threads, 0.1.6: the hot set is about 13% of reserved | no at ten threads: "The traffic is the decoder's sequential buffers, not the T-vector's random reads" (BW, simulated); other per-block buffers of 14–16 MB "are a plausible reason the knee has moved" (TS). Neither is a measurement of hot bytes: E5 is "capacities and high-water marks" |
| P145 | Occupancy trace under team plans | Complete traces under a team plan | shipped | n.a. #116, released in 0.2.2 (CL). Used by TS, TPR, PLR, S23, SLA | derived: none, an instrument | none needed | none | REG; CL 0.2.2; TS; SLA | the trace's labels name the wait's cause | no for two labels: "these labels don't identify the bottleneck" (`TeamIdleStart` records whether a prepared block exists once; SLA correction). "Busy … is not CPU utilisation" (SLA). Helpers "record no marks" (TS) |
| P146 | PMU availability (CI none; VM none; i9 yes) | The i9 can count hardware events | validated | n.a. for CI. For the i9 the later record disagrees | derived: none | read of an archived bundle: `telemetry-check/20261109T125938Z` on `evidence`. Its result entry is not in the journal | none directly; it decides which falsifiers exist | REG; TS; KM E2; BW §1 | counters were read under `sudo perf` on 28–29 October (WS §4) | no for unprivileged runs: "perf could not count on the machine (`perf_event_paranoid`)" (TS, 6 November); "`perf_event_paranoid` is 4 on the workstation" (BW). No counter of the cooperative decoder exists. See Mined 10 |
| P147 | Busy-desktop plateau at about 470 MB/s | Another program's DRAM traffic set the plateau: 884–907 MB/s quiet against 471 | validated | n.a. | derived: none | none needed | none | REG; WS §1; RFL2; SPIN | paired comparisons survive a shared load | yes. A later busy host is recorded: `load1` 3.1 against 0.41 for the bundle before it (RFL2); "The load of about 3 was the harness's own" (SPIN) |
| P148 | memcpy counter calibration | IMC counters read 1.56–1.58× reads and 1.11× writes of a copy | validated | n.a. | derived: none | none needed | none | REG; WS §3; BW | as recorded | yes. BW's W of 28.5 GB/s uses these counters' October figures |
| P149 | Slow runs after an idle gap | A 0.15 s gap before a run makes some runs slow: 5 of 12 with a gap, 0 of 12 without | open | n.a. "Recorded, not pursued" | derived: none | i9 recipe that exists: `just bench-regimes` (`bench/regimes.py`), gap against no gap, at 10 threads on the default plan; or one long decode (BDC §6) | none directly; it decides whether burst figures transfer | REG; WS §4; 2M; RFL1; RFL2; PSR | "a release decodes without pauses, so the gap-free figures are the relevant ones" | no as a reason to set it aside: the registered i9 harness runs with a 0.260 s gap before every run and shows two modes, 0.159 s and 0.306 s (2M). The recorded condition, "unless it shows up in gap-free runs", is not met. See Mined 4 |
| P150 | Native block table size: by count against by time | The corpus's working table is 3.600 MB time-weighted, not 1.946 MB by count | rejected | advantage of the by-count figure: a measurement error; #110 reverted | derived: none | none needed | shrinks the table: 3.6 MB is the size to beat | REG; TPR; SPIN | "54% of 431 blocks are full" | yes: "198 of the 431 are below `MIN_COOP`" (TPR); 233 team runs a decode (SPIN) |
| P151 | Raw offcore excess = L2 streamer prefetches | Bit 7 explains the excess | rejected | advantage: refuted twice (1.219–1.365 without bit 7; bit 7 alone 0.004–0.036 a byte) | derived: none | i9 recipe needing counters; blocked (P146) | none | REG; KM E2; GPP §1.2 | raw hit shares are "unreconciled" | yes, still unreconciled. KM E2 and GPP §1.2 list hit shares of 0.987 to 0.539 against live table MB as measured, without that label; whether they are the same raw event was not checked here |
| P152 | Six pilot-harness negatives | Each was a harness fault: a thread a block 1.05×; unpinned spinning pools 0.65–1.15× → 1.67× pinned | rejected | construction: each fixed in the pilot harness, VM | unknown | none needed | removes cooperation: "a thread a block" lost on a VM with no cache pressure; it is now the objective for small tables | REG; TS; 2M §5 | the VM, four cores, no capacity pressure | moved: production ships unpinned pools that spin 2,000 times and park. Pinned spread read 1,369 MB/s against unpinned 1,276, a lead only (TS) |
| P153 | Code-layout control for about 2% kernel A/Bs | Same-source arms differ by layout: `mtf_cp` 6.3% against 3.9% | open | n.a. Not built | derived: none | CI A/B: the same source in several builds with its code shifted | none | REG; 3T; CL 0.2.7, 0.2.4 | as recorded | yes, with more cases. A same-plan i9 control read 0.986 (0.977–0.995), "a build or layout effect, is untested" (3T). The build stage read −5.2% (−6.5, −3.8) with unchanged code (CL 0.2.7) and +6.7% (CL 0.2.4) |
| P154 | The scatter compiles two ways | Two closures about 1 instruction a byte apart | open | n.a. | derived: about 1 of the build's 6.2 instructions a byte (REG; SN §1) | counting probe: callgrind of the build in both binaries | none | REG | "a question for later" | not in the record since, in this slice |
| P155 | The given-counts research build's own overhead | q4x1 build 54.6 ms against p4 37.2 | open | n.a. "Recorded, not started" | unknown | none needed: the one-pass fused build shipped (0.2.1) and the `q` arm is not used since (inferred) | none | REG | a research arm | moot (inferred) |
| P156 | l3x3 is 7–8% faster than its twin x3x3 | l3x3/g3x3 1.082 (1.072–1.092): code, not placement | open | n.a. Parked by the maintainer | unknown | counting probe: instructions a row of the `l` and `x` walks | removes cooperation only if it transfers | REG | "pursued only if it points at an optimisation that transfers" | unchanged. It shows kernels of one design differing by 8% in the team harness, the scale of Mined 1's question |
| P157 | The -6 control slowdown under #115's build | Pooled 0.984 (0.969–0.999) against A/A 1.006 | open | n.a. Unexplained; #115 is closed | unknown | none needed | none | REG; 3T; BP | a slowdown of 1.6% is a real effect of the build | moved: a same-plan control on the i9 read 0.986 (0.977–0.995) and did not recur (3T). 0.984 is the size of drift the i9 shows between two builds of one plan |
| P158 | Controlled version ladder (#52) | A ladder of releases on one runner gives the true history | superseded | n.a. Never run as designed. In practice replaced by a direct paired run of tag against tag each release, and `bench-release-scaling` on the i9 (`docs/RELEASING.md`; BP) (inferred) | derived: none | read of archived records: `bench/release-scaling/` | none | REG; BP; GPP §1.7 | chained per-PR links add up | no: "The chain misses a third to a half of the 1-thread gain" (−4.2% chained against −8.3% direct; BP) |
| P159 | Correction of libsais's decode path | libsais takes the symbol-rows path for a 900,000-byte block | validated | n.a. Source reading | unknown | none needed | none | REG | as recorded | not re-read in this slice |
| P160 | bz2ngrs's gap is Wikidata's inverse BWT | bz2ngrs 1.45× (1.45–1.46) slower on Wikidata; peak RSS 0.45× mibz2 at 1 thread (12 against 26 MB) | validated | n.a. CI 7763, 0.1.x | unknown | read of an archived run; "The 26 MB is not broken down" still holds | shrinks other live memory: a decoder at 12 MB a thread exists | REG; CL 0.2.7 | 0.1.x kernels | moved in size only: the walk is 24.9% faster since (CL 0.2.2). mibz2 at one thread is still 26 MB (CL 0.2.7) |
| P161 | Fair multicore comparison with bz2ngrs; `bench-compete` | A fair multicore comparison | open | n.a. `just bench-compete` exists (`.just/bench.just`, `bench/compete.sh`) and is one physical core only; no workstation result is in the journal | unknown | i9 recipe: run it | none | REG; `.just/bench.just` | "It is not built" | moved: the one-core recipe is built. The multicore comparison is not |
| P162 | External report 3's ranking | n.a.: a ranking | superseded | n.a. | unknown | none needed | none | REG | as recorded | yes |
| P163 | Literature tasks and citation checks | n.a. | open | n.a. | unknown | none needed | none | REG | as recorded | not re-read in this slice. Later entries exist: `2026-11-07-left-field-archive-check.md`, `2026-11-10-borrowed-tricks.md` |
| K1 | libsais can or cannot decode bzip2 blocks | n.a. | validated | n.a. | unknown | none needed | none | REG | resolved on 30 October | not re-read in this slice (section (a) to (c) subject) |
| K2 | "Reportable Shortcuts" not located | n.a. | open | n.a. | unknown | none needed | none | REG | as recorded | not re-read in this slice |
| K3 | Huge pages: no change, open, or 36.1 → 28.2 c/B | Page walks are 1–2.5% of cycles | validated | n.a. Closed as a speed lever by #40 | unknown | i9 recipe: 2M's readings (`thp_fault_alloc`, minor faults) | none | REG; 2M; BC | page walks, i9, full decoder | yes as a lever. Reopened as a diagnostic: huge pages are mechanism H4 for the slow mode, argued against "as the whole story" (2M). The runners' THP setting "is unrecorded" (BC) |
| K4 | H3 predicted "low-medium" gains; measured 81.9% of rows via run stores | n.a. | validated | n.a. Prediction wrong | unknown | none needed | none | REG | per-run mean against row-weighted share | not re-read in this slice |
| K5 | Peak RSS at 4 threads: 71, 98/99, 113, 74 MB | n.a. | validated | n.a. Boundary | unknown | none needed | shrinks other live memory | REG; WIN; CL 0.2.7 | state version and boundary | yes. Later figures: 66 → 74 and 69 → 77 MB with #180 (WIN); 78 MB at 4 threads, 26 MB at 1 for 0.2.7 on a 9V45 (CL) |
| K6 | Report MB against CHANGELOG MB (ratio 1.048) | n.a. | open | n.a. Label | unknown | none needed | none | REG | as recorded | not re-read in this slice |
| K7 | Scanner share 5.4%, about 7%, about 8%; binds at 9–11, 13 or 11 workers | One scanner feeds about 12 workers | superseded | n.a. Measured on one instrument and paired on the i9, as the row asked | counted: the scanner is 3.5 of 26.7 instructions a byte (SN §1) | none needed | removes cooperation: the finder is the other half of plain workers | REG; PS; PSR; BDC §1 | "one scanner feeds about 12 workers" at 24–30 c/B | no: the share is 9–11% on Wikidata at level 9, 20.6% at level 1, 23.2–24.8% on incompressible data (PS). Measured: at level 1 the scan arm stops at about 1.6 GB/s and the finder gives 1.615 at ten workers; at level 9, 0.981, the scanner does not bind first (PSR) |
| K8 | "No shared resource … binds at 4 cores" (VM) against CI's doubling walk | n.a. | validated | n.a. Boundary | unknown | none needed | shrinks the table | REG; PSR; BC | machines differ | yes as a boundary point. On the i9 a shared resource costs at four workers at 3.6 MB: CPU 0.575 → 0.853 s from 1 to 4 workers at level 9, 0.705 → 0.737 s at level 1 (PSR). On CI the 4-thread row is SMT co-run (BC) |
| K9 | Evenly spaced starts stall, yet pbz2 uses them | n.a. | open | n.a. | unknown | CI A/B (P30's sweep) | none | REG | as recorded | not re-read in this slice |
| K10 | The "local" VM's spec differs between documents | n.a. | validated | n.a. | unknown | none needed | none | REG; SC; BDC | treat VM figures as different hardware | yes, and it moved again: "Xeon @ 2.80 GHz, 1 thread a core" on 5 November (SC); "an EPYC with a 2.1 GHz TSC and an unknown core clock" on 10 November (BDC) |
| K11 | #37's gain differs by runner and head | n.a. | validated | n.a. Boundary | unknown | none needed | none | REG; BP | a same-head A/B on both vendors would settle it | generalised: "Replicate across CPU models, not rounds, at 1 and 2 threads" (BP) |
| K12 | Decode-order walk penalty +23.9%, +3.6%, +40–50% | n.a. | validated | n.a. Boundary | unknown | none needed | none | REG | "A stage table's shares are a runner's" | yes (see P140) |
| K13 | Super-alphabet and Precopy "not yet tried" against tried | n.a. | validated | n.a. Timeline | unknown | none needed | none | REG | as recorded | not re-read in this slice |
| K14 | Prefetch saving 34%, 26–28%, 13%, 12% | n.a. | validated | n.a. Boundary | unknown | none needed | none | REG | as recorded | not re-read in this slice |
| K15 | Local split CPU −30 to −37% against CI's −20.5% | n.a. | open | n.a. | unknown | none needed | none | REG | accept CI as the authority | not in the record since |
| K16 | libsais 12.10 against 10.86 c/B run to run | n.a. | open | n.a. | unknown | none needed | none | REG | as recorded | not re-read in this slice |
| K17 | Triage checkboxes unticked | n.a. | open | n.a. | unknown | none needed | none | REG | as recorded | not re-read in this slice |
| K18 | libsais "1.20 × our 3x3" against 3×3 1.114× libsais | n.a. | validated | n.a. Stale | unknown | none needed | none | REG | the kernel changed | not re-read in this slice |
| K19 | libsais H1's remark against E2 | n.a. | validated | n.a. | unknown | none needed | none | REG | as recorded | not re-read in this slice |
| K20 | E11's baseline (q3x3 against PR #73's) | n.a. | open | n.a. Label | unknown | none needed | none | REG | as recorded | not re-read in this slice |
| K21 | One-pass build on CI: #93 against #114 | n.a. | validated | n.a. Resolved in PM §2.1 | unknown | none needed | none | REG; PM | as recorded | not re-read in this slice |
| K22 | Compact pilot `c` build 85 or 155 MB/s | n.a. | open | n.a. | unknown | read of #91's results file | none | REG | as recorded | not re-read in this slice |
| K23 | Marker audit against the feasibility review | n.a. | open | n.a. | unknown | none needed | none | REG | as recorded | not re-read in this slice |
| K24 | Census labels: groups against rows | n.a. | validated | n.a. Label | unknown | none needed | none | REG | as recorded | not re-read in this slice |
| K25 | "#8's fusion" label | n.a. | validated | n.a. | unknown | none needed | none | REG | as recorded | yes |
| K26 | "#81 improved … by 31.3%" without its baseline | n.a. | open | n.a. Label | unknown | none needed | none | REG | as recorded | not re-read in this slice |
| K27 | E10 proposes lanes8/16 at 4–10 workers; WS §6 had measured them | n.a. | validated | n.a. Stale | unknown | none needed | none | REG; WS §6 | lanes16 0.86×, lanes8 0.71× at 10 workers | yes for plain workers on 3.6 MB tables. Never read on small tables or in teams |
| K28 | E4 / #77 status: superseded, validated or open | n.a. | open | n.a. No closing verdict is in the record read | counted: see P95 | counting probe (P95) | removes cooperation | REG; SN §1 | the 0.97 × p4 rule was never waived | still unclosed. The gap it gated is now counted: 38.0 against 26.7 instructions a byte (SN §1) |
| K29 | tFAW ceiling assumes single rank; the DIMMs are dual rank | n.a. | open | n.a. | unknown | i9 recipe (MLC) | none | REG; WS | as recorded | yes, still not redone |
| K30 | Journals cited on master that exist only on branches | n.a. | open | n.a. It has grown | unknown | none needed | none | REG; 3T; S23; README | merge or link | no, worse: 3T cites `2026-11-06-three-tables.md`, S23 cites `2026-11-08-service-2x3-registration.md` and `2026-11-08-service-core-result.md`, README cites `2026-11-08-plan-levels-result.md`; none is in master's `docs/journal/` (checked with `git ls-tree origin/master`). The README names its branch; the others do not |
| K31 | "The faster reference is libsais": i9 only | n.a. | validated | n.a. Boundary | unknown | none needed | none | REG | as recorded | not re-read in this slice |
| K32 | libsais's one-core lead 1.13×, 0.989, 1.075 | n.a. | validated | n.a. Boundary | unknown | none needed | none | REG | as recorded | not re-read in this slice |
| K33 | Straggler: imbalance 4% against busiest 1.60× | n.a. | validated | n.a. Boundary | unknown | none needed | none | REG | as recorded | not re-read in this slice |
| K34 | Team shape: 4x2 ahead on the `y` kernel, 3x3 on the `l` kernel | n.a. | superseded | advantage: 3×3 at ten threads 1.089 and 1.088 against 4×2, registered and confirmed (3T) | counted: see P91 | none needed | none | REG; 3T; GPP §1.1 | "3 × 3 tied it at the full decoder (1.008)" | no: the tie was the 29 October build's. "The shipped policy uses teams of 2" is stale since 0.2.5 |
| K35 | Native table size 18 MB, about 2 MB, 1.95 MB, 3.600 MB | n.a. | validated | n.a. | unknown | none needed | shrinks the table | REG | time-weighted 3.6 MB | yes (P150) |
| K36 | Teams instruction-bound, or neither label | n.a. | open | n.a. | counted: 38.0 instructions a byte (SN §1) | i9 recipe: `bench-bw-titrate` | removes cooperation | REG; SN; BW | old kernel | still unresolved for the default plan (P97) |
| K37 | Elasticity of throughput to walk instructions: predicted 0.49, measured 0.32 and 0.16 | n.a. | validated | n.a. Prediction wrong | unknown | none needed | cuts CPU a byte, at a discount | REG; REG §6 | a forecast | yes, and lower still at four workers: the 1 November walk-unroll trial took the walk from 16.4 to 10.44 instructions a step and the i9 decoder read 1.017 (1.006–1.028) at 1 worker and 1.000 (0.987–1.012) at 4; #157, "a closely related" change, showed no gain at 4 threads on CI, +0.4% (−2.8, +3.8) and +0.2% (−2.1, +2.7) (REG §6). It discounts every instruction-count bound in this file |
| K38 | `mtf_cp` 6.3% against 3.9%, same source | n.a. | open | n.a. | unknown | CI A/B (P153) | none | REG | code layout | yes (P153) |
| K39 | A register-held MTF list queued as new although #25 was rejected | A list head in registers is faster | superseded | construction: #25 (two AVX2 registers) lost on CI. The advantage holds in another construction: #207, "its first 32 bytes in two registers", shipped in 0.2.4: `wikidata` 1 thread −4.8% (−8.8, −0.7), `records` −23.5%, MTF + RLE2 stage −17.1% (CL 0.2.4) | unknown | none needed | cuts CPU a byte | REG; CL 0.2.4; RFLci | "#25 rejected on CI" | no: a later construction shipped. A further register-list checkpoint kernel read 0.9745 [0.9675, 0.9816] at 9 threads on CI (RFLci), "a separate lead" |
| K40 | The d ≤ r − 1 bound "loose" against exact | n.a. | validated | n.a. | unknown | none needed | none | REG | as recorded | not re-read in this slice |
| K41 | The level-policy registration's arithmetic at level 8 | n.a. | validated | n.a. Label | unknown | none needed | none | REG | "Re-register if the formula is revived" | yes; the formula was not revived (P102) |
| K42 | Two-tier sizing: illustrative against exact | n.a. | validated | n.a. | unknown | none needed | none | REG | use census v3 | not re-read in this slice |
| K43 | #115's -6 slowdown against A/A | n.a. | open | n.a. | unknown | none needed | none | REG; 3T | a real, unexplained effect | weakened (P157) |
| K44 | Fusion predictions missed high; p4 drifted between runs (1,206 against 1,282 MB/s) | n.a. | validated | n.a. | unknown | none needed | none | REG; CD | "Record p4's drift beside the ratios" | yes, and the drift has a name now: the same arm read 863 against 1,007 MB/s between two runs (CD), and 0.159 s against 0.306 s within one bundle (2M) |
| K45 | Pair walk sign by machine and pressure | n.a. | validated | n.a. Boundary | unknown | none needed | none | REG | as recorded | not re-read in this slice |
| K46 | Output in L's buffer: branch text against master | n.a. | validated | n.a. Stale | unknown | none needed | none | REG | as recorded | not re-read in this slice |
| K47 | #42 described as a Ψ² negative | n.a. | validated | n.a. Label | unknown | none needed | none | REG | as recorded | yes |
| K48 | "#10 … stay closed" although #10 was deferred | n.a. | superseded | n.a. | unknown | none needed | removes cooperation | REG; PS; PSR | "#10 stays closed" (WS) | no: "#10 reopened" (PS title); "reopened by the registration, and now measured" (PSR §6) |
| K49 | #42's criterion drift | n.a. | validated | n.a. Label | unknown | none needed | none | REG | as recorded | not re-read in this slice |
| K50 | #78's registered pass replaced | n.a. | open | n.a. | unknown | none needed | none | REG | as recorded | not re-read in this slice |
| K51 | L3 hit share at 4 workers: 0.76 against 0.945 | n.a. | open | n.a. Boundary plus an unreconciled event | unknown | blocked: needs counters (P146) | shrinks the table | REG; KM E2 | different boundaries and events | yes. A third figure is in use: 0.929 at 14.4 MB of live tables (KM E2; GPP §1.2), from the compact-Ψ grid |
| K52 | #65's cap predates the team plan; the interaction "is not recorded" | n.a. | validated | n.a. Now written down | unknown | none needed | none | REG; GPP "The terms"; README | as recorded | resolved: c = min(N, P); below 6, N workers (GPP; README "How the threads are spent"). Whether #65 itself was updated is not in the record |
| K53 | E1's status label | n.a. | validated | n.a. | unknown | none needed | none | REG | as recorded | not re-read in this slice |
| K54 | Precopy: rejected against a later condition | n.a. | open | n.a. | unknown | i9 recipe | none | REG | as recorded | not re-read in this slice |
| K55 | First-column packing's status label | n.a. | validated | n.a. | unknown | none needed | none | REG | as recorded | not re-read in this slice |
| K56 | "0.2.x has never run in the release" | n.a. | validated | n.a. Stale | unknown | none needed | none | REG | wikidata-pq pins 0.2.1 | as of 30 October. Whether wikidata-pq pins a later release is not in this repository's record (Mined 8) |
| K57 | The occupancy trace "collects no complete round under a team plan" | n.a. | validated | n.a. Fixed by #116 | unknown | none needed | none | REG; CL 0.2.2 | as recorded | yes; released in 0.2.2 |
| K58 | "lbzip2 stays the default" | n.a. | validated | n.a. Stale | unknown | none needed | none | REG | as recorded | not checked since 30 October |
| K59 | Raw write-pointer lane state: no result, or shipped in #12 | n.a. | open | n.a. | unknown | read of #12's diff | none | REG | as recorded | not re-read in this slice |
| K60 | Weißenberger and Schmidt: rejected against conditional | n.a. | validated | n.a. | unknown | none needed | none | REG | conditional (P124) | yes |
| K61 | E9 (P87): open against conditional | n.a. | superseded | n.a. | unknown | none needed | shrinks other live memory | REG; `2026-11-06-team-neutral.md` | "Open, not run" | no: `2026-11-06-team-neutral.md` lists "the research portfolio register (P87 still "open")" as historical framing; `2026-11-02-p87-production-integration.md` and `2026-11-05-p87-production-validation.md` exist. Not read in this slice |
| K62 | `v`'s link-only build 1.55× against 1.28–1.49× | n.a. | validated | n.a. Boundary | unknown | none needed | none | REG | as recorded | not re-read in this slice |
| K63 | Checkpoint memory units | n.a. | validated | n.a. Label | unknown | none needed | none | REG | use KiB | yes |

### (f) The application

**The application's needs, as the record states them.**
- **What it is.** wikidata-pq's split decodes the Wikidata dump, bzip2 at level 9 ("Wikidata's
  dumps are level 9", PL; "the 103 GB dump", PS §2).
- **Decoder choice.** mibz2 is its default: `DEFAULT_BZ2_DECODER = "native"` on wikidata-pq
  master dd9e1031, checked for the register on 30 October (REG P170). When that changed, and
  which of the seven conditions were met, is not in the record.
- **Thread cap.** `BZ2_THREADS_CAP = 4`, the rule `min(physical cores, 4)`, "still a hypothesis
  from two machine classes" (REG P172). 8 threads were tried and refused: "cap stays 4" (REG
  P173).
- **What the cap selects in mibz2.** Below 6 physical cores serving the threads the plan is
  N plain workers (GPP), so at 4 threads wikidata-pq runs four plain workers, each with its own
  3.6 MB table, one scanner and the reader. None of the ten-core plans of 0.2.5 to 0.2.7 is
  reached (inferred from `plan_with` as GPP states it). "The 4-core release target plans
  `Plan::Workers`" (PM §4).
- **One thread a physical core.** "wikidata-pq decodes with one thread a physical core" (CP);
  "one thread a physical core, wikidata-pq's configuration" (JH).
- **CPU is a cost there, not only wall.** The decoder shares the cores with the split's own
  threads: "18 parse workers, 18 writer threads and 9 decoder threads on 20 logical CPUs" in
  the 8-thread arm (PM §3.4). "On one where the decoder shares the cores with other work
  (wikidata-pq's pipeline), the extra CPU is a cost. It should be measured there before a
  release" (WIN).
- **Its merge rule.** "a change merges only when an end-to-end figure improves beyond noise"
  (CL 0.1.5, Downstream). A bump follows each mibz2 release, "under that repository's bench
  rules" (`docs/RELEASING.md` §6).
- **How much of it is the decoder.** "decoding is about a third of the split's CPU" (CL 0.1.5,
  an interpretation). In the i9's release-shaped run the split took 25 s of 103 s of wall, and
  the decoder is modelled at about 11 s, "under 11% of that run's wall" (PM §3.3). The modelled
  release ETA was 4h34m37s ± 13m13s, the split's share 1h33m22s ± 3m19s (CL 0.1.5).
- **Its ceiling is unknown.** The decoder-free arm N0 "measures this bound directly" and was
  not built on 30 October (PM §3.3). Nothing since records it.
- **Pin.** 0.2.1 at d09edd91 on 30 October (REG P174). Whether it has moved to 0.2.2 or later
  is not in this repository's record.

So the application needs CPU a decoded byte at four plain workers on level-9 input, beside its
own parse workers. It does not need ten-thread throughput as configured.

| ID | idea, one line | the advantage it claims, as a quantity that could be false | status 2026-11-10 | closed by: construction / advantage / n.a., with the figure | whole-loop price | cheapest falsifier now | bearing on share-nothing | sources | premise the verdict rests on | premise still true? |
|---|---|---|---|---|---|---|---|---|---|---|
| P170 | mibz2 as wikidata-pq's bz2 decoder | Split CPU −20.6% ±4.7%, wall −16.6% ±1.6% against lbzip2 (application, CI 7763, 3 paired rounds) | shipped | n.a. | counted: 26.7 instructions a decoded byte on one thread (SN §1) | none needed for the choice | cuts CPU a byte: the application pays for CPU | REG; CL; README | the 0.1.x decoder against lbzip2 | yes. The decoder is faster since: v0.2.1 → master −15.4% wall at 1 thread, −25.8% at 4, on a 7763 (BP). Whether the application runs that is not in the record |
| P171 | Making native the default: seven conditions | n.a. | superseded | n.a. Native became the default; which conditions were met is not recorded | unknown | none needed | none | REG | as recorded | unchanged. One condition, "the scanner at 16 cores", now has an i9 answer: one scanner binds ten workers at level 1 and on records, not at level 9 (PSR §6) |
| P172 | wikidata-pq's thread cap of 4 | 4 threads against auto (20) on the i9: split wall −20.3% ±0.9%, CPU −60.1%, 72/72 parity | shipped | n.a. The comparison was against 20 plain workers, before teams | derived: four plain workers at level 9 spend 0.853 CPU-s a decode against one worker's 0.575, i9 (PSR) | i9 recipe, application: release-shaped paired rounds, cap 4 against a small-table decoder when one exists; today none needed | shrinks the table: four 3.6 MB tables are 14.4 MB, the knee. A small table's prize here is CPU, not threads | REG; PSR; WS §9 | plain workers knee at 4 on the i9; CI's 4 threads are SMT | yes for plain workers: they still peak at 4 workers, 1,145 MB/s (PSR). The rule `min(physical cores, 4)` is still two machine classes. See Mined 5 |
| P173 | 8 threads (4 teams of 2) in the release | Whole-run wall −2.0% ±0.6% for CPU +6.7% ±0.4%; split wall −4.2%, split CPU +16.9% | rejected | construction: 4×2 teams of 0.2.1 at 8 threads, beside the split's own threads. The conditional advantage, the wall gain at less CPU, was not tested | unknown at the application. Decoder alone, i9: 3×2 against 4×2 at 8 threads spends 1.097 → 0.850 CPU-s at 0.989 [0.969, 1.009] speed (PB) | i9 recipe, application: D1's per-role CPU attribution (PM §3.4), never run; then one release-shaped pair on a current plan | removes cooperation: the extra CPU of teams is what was refused | REG; PM §3.4; PB; PLR; S23; 3T | "contention suspected, not demonstrated"; teams means 4×2 | no: the plans moved. Park cuts CPU 1.055 at equal wall (PLR); 3×3 is 0.88–0.89 of 4×2's CPU and 1.089 faster (3T); svc2@10 is 12.7% under the default's CPU and 1.045 faster (S23). See Mined 8 |
| P174 | Pin mibz2 0.2.x in wikidata-pq | n.a. | shipped | n.a. 0.2.1, wpq#213 | unknown | none needed | none | REG; CL; `docs/RELEASING.md` §6 | pinned 0.2.1 | not in the record since. Six releases followed, 0.2.2 to 0.2.7 (CL). At cap 4 a bump delivers the plain-worker gains only (inferred) |
| P175 | No fresh pages per chunk | 804,748 minor faults at about 8.5 µs; sys 15% of split CPU (VM, one run) | open | n.a. "to be benched on CI" | unknown | CI A/B in wikidata-pq | shrinks other live memory in spirit: buffers "kept across decodes in one process fault once" (2M §5) | REG; 2M; BC | a single VM run | not in the record since. The decoder's own first-touch faults are "about 0.5–2% of 4-thread CPU", bench-only (BC) |
| P176 | No thread oversubscription in the split | `RAYON_NUM_THREADS=1`: split CPU −13%; `=2`: wall −9% | open | n.a. "CI must measure" | unknown | CI A/B in wikidata-pq; D1 on the i9 | cuts CPU a byte at the application | REG; PM §3.4; WIN | VM, single runs | not in the record since. The i9 arm had 18 + 18 + 9 threads on 20 logical CPUs (PM §3.4) |
| P177 | Finalise's Parquet decode built for AVX2 | decode −10% (1.47 → 1.29 CPU-s) | open | n.a. | unknown | CI A/B in wikidata-pq | none | REG | as recorded | not in the record since |
| P178 | Parquet encode with levels from the decoder | bounded ≤ 1.3×, not measured | open | n.a. | unknown | none cheap: upstream work | none | REG | as recorded | not in the record since |
| P179 | Schema-compiled claims JSON → Arrow | bounded ≤ 2.5–4× on the parse; callgrind 32.8 instructions a byte | open | n.a. The SIMD scan alone was rejected: −5.6% instructions, 0% CPU | counted: 32.8 instructions a byte for the present parse (REG), more than the whole decode's 26.7 (SN §1) | counting probe in wikidata-pq | none; SN §5's "Peeks" would give "record boundaries for a parallel consumer" without an inversion, "not built" | REG; SN §5 | as recorded | not in the record since |
| P180 | wpq#194 dead ends: asm/AVX-512 walk, zstd, bandwidth, line split | n.a. | rejected | advantage, each bounded on the VM: zstd 8% of encode; ≤ 0.2 GB/s a core; 0.9 c/B | unknown | none needed for zstd and line splitting | none | REG; BW; PSR | "No shared resource … binds at 4 cores" (VM, K8) | no for the bandwidth member on the i9: four plain workers already cost 0.853 CPU-s against 0.575 for one (PSR), and bandwidth at ten threads is an open, registered question (BW). The others stand |
| P181 | gz input instead of bz2 | The bz2 split, about 2h34m modelled, beats gz at Wikimedia's rate | conditional | n.a. The maintainer's decision | unknown | none needed | none | REG | modelled | not in the record since. A faster decoder moves it further toward bz2 (inferred) |

## Mined

Twelve findings, ranked by what they could be worth. Each names a sentence in the record and a
later sentence that contradicts, weakens or satisfies it.

1. **P95 and P90, the cooperative walk's price.** The verdict: "Cooperation costs 42% more work
   for the same bytes, 76% more in the walk" (SN lead; walk 17.8 against 10.1 instructions a
   byte, §1). Its premise is that the difference is cooperation. The earlier record says
   otherwise. On 29 October the cooperative room walk ran 16.9 and 17.1 instructions a byte (REG
   P95) when production's room walk was about 16 a step. Production then fell "16 → 9
   instructions per step" with #157 (REG §6) and was staged by #171 (CL 0.2.2), and the register
   lists as an open item "The cooperative and flagged walks: the same rolled loop. A port waits"
   (REG §6). `src/coop.rs` `traverse` has no `TILE`; `src/bwt.rs` `room_segments` has (my grep,
   inferred). So most of the walk's 7.7 extra instructions are a kernel that was never ported,
   not sharing a table. **What that makes possible:** a cheaper cooperative default with no new
   table, and a corrected price for cooperation, which lowers the share-nothing prize.
   **Cheapest falsifier:** a counting probe, callgrind of `coop::traverse` against
   `room_segments` on one block at k = 1. **Worth:** derived: (17.8 − 10.1) ÷ 38.0 = 20% of the
   default plan's instructions at most. Discount it by the measured elasticity: the 1 November walk-unroll
   trial, "a closely related precedent" to #157 (16.4 → 10.44 instructions a step), read 1.000
   (0.987–1.012) at four plain workers on the i9 (REG §6), and elasticities of 0.32 and 0.16 were
   measured on the cooperative kernel (K37). So 0 to about 6% of throughput (inferred: 0.32 ×
   20%). If
   the walk premium is about 1 instruction, as on 29 October, cooperation's own premium is about
   4.6 of the 11.3 instructions, 17% over one thread and not 42% (inferred).

2. **P90 and SN §3b, the break-even across two harnesses.** The verdict: "Break-even with the
   cooperative default (about 1.5 GB/s) is near g = 2.5" (SN §3b). The 3.0 GB/s numerator is
   from "three long-lived processes" (SN §3b). The 1.5 is `bench/release-scaling`, 0.2.6, 10
   threads (BW §2), from the `ab` harness, whose runs are each a fresh process ("Every bench run
   is a fresh process", BC; "one process a run", 2M) of about 0.14 s (0.142 and 0.144 s medians,
   PLR, S23). The record says a short fresh
   process pays costs a long one does not: "a 0.15 s decode on ten cores pays ten ramps and ten
   stacks of first faults" (2M §5); start-up and tail are "a bench-only cost that is about 0 at
   release scale" (BC). SN itself says its harness "differs in both" from the one that showed
   two modes. The same plan reads higher in the long-lived harness: four plain workers give
   1,145 MB/s there (PSR) against 1,016–1,021 in release scaling (GPP §1.7), on different dates
   and builds. **What that makes possible:** a same-harness break-even. **Cheapest falsifier:**
   an i9 recipe: the `lib` arm, which `par_scan` already has (PS §3), added at 10 threads to the
   knee sweep. **Worth:** derived: if the default gains the 1.127 that four plain workers show
   (1,145 ÷ 1,016, inferred), break-even moves from g ≈ 2.5 to about 2.1 (3.0 ÷ (1.5 × 1.127) =
   1.775 = 0.3 + 0.7 g). Every plan verdict in table (d) taken on the i9 (CD, WS §9, TS, 3T,
   TPR, PLR, PB, PBY, PL, SCR, S23, SLA) is a burst verdict of this kind: one process a run of
   0.12 to 0.33 s. Their paired intervals are tight, so they stand as verdicts on bursts. None
   has been read on a long decode.

3. **P117 and P113, ten workers at 1.2 MB.** The objective: "a table under about 1.2 MB a
   block … lets ten workers each run the one-thread decoder and share nothing" (SN lead), with
   the gate assuming "the table is small enough to behave as level 1's does" (SN §3b). The
   record has a counterexample at exactly that size: at -3 (1.2 MB) eight workers gave 1,729
   MB/s and ten 1,604 (WS §9), and without any scanner "p10/p8 0.948 (0.937–0.959)" at hit
   shares 0.940 against 0.929 (REG P117); "at level 3, 10 blocks keep a 0.940 hit share yet lose
   to 8 blocks, mainly in the build. A request or core-count limit is not ruled out" (KM E2).
   Level 1 (0.4 MB) does scale from 8 to 10: 2,429 → 2,563 MB/s (PSR). **What that makes
   possible:** the target size is set by a measurement, not by the 14.4 MB knee alone.
   **Cheapest falsifier:** the i9 recipe SN §4.2 already names, `par_scan FILE knee` on
   `lbzip2 -2` and `-3` input, reading the find arm at 8 against 10. **Worth:** derived: if ten
   workers at 1.2 MB give 0.948 of eight where level 1 gives 1.055 (2,563 ÷ 2,429, inferred),
   the modelled ten-worker figures fall to 0.90 of SN's table (0.948 ÷ 1.055): 2.6 → about 2.3
   GB/s at g = 1.2, and break-even moves from g ≈ 2.5 to about 2.1.

4. **P149 and the two modes.** The verdict on 28 October: slow runs came with "plain, 0.15 s gap
   before the run", 5 of 12, against 0 of 12 with no gap, each slow run "costing about 1.5× the
   CPU a block with the same occupancy and no run-queue wait"; with `sudo perf` and a gap the
   clock ratio read 1.255–1.317 against 1.326 (WS §4). It was set aside: "a release decodes
   without pauses"; "Recorded, not pursued". The later entry reads the same signature (CPU and
   wall growing together, no run-queue wait) at ten threads, 0.159 s against 0.306 s, and says
   "Not the gap: 0.256 to 0.264 s before every run" (2M §1). A gap that never varies cannot be
   ruled out; WS §4 is the only run in the record that varied it, and 2M does not cite it. A
   long-lived process shows no second mode: "81, 83, 81, 82, 83 ms" (SN §3b). **What that makes
   possible:** naming the mode with a recipe that exists. **Cheapest falsifier:** an i9 recipe:
   `just bench-regimes` (gap against no gap) at 10 threads on the default plan, before 2M's
   probe; or the long decode (BDC §6). **Worth:** cannot be bounded from the record as
   throughput. It decides whether two closed registrations (RFL1, RFL2, both "control fidelity
   FAILED") failed for a harness reason, and whether burst figures transfer.

5. **P172, the application's own break-even.** The premise in circulation: an inversion up to
   about 2.5 times production's still ties the default (SN §3b). That is at ten threads against
   teams. The application runs four plain workers (table (f)). There a level-9 decode costs
   0.853 CPU-s against one worker's 0.575, and a level-1 decode 0.737 against 0.705 (PSR).
   **What that makes possible:** a CPU saving at the application's own configuration, which the
   ten-thread framing hides, and a tighter gate for it. **Cheapest falsifier:** read of the
   archived `par-scan` bundle for the four-worker cells, then the `lbzip2 -2` run. **Worth:**
   derived, with SN's 70% inversion share: a small table at four workers costs about 0.575 ×
   (0.3 + 0.7 g) × 1.045 CPU-s, which equals 0.853 at g ≈ 1.6 (inferred). So at the cap of four
   on the i9 the table pays in CPU only below about 1.6 times production's inversion, and at
   g = 1 it saves about 30% of the decoder's CPU (1 − 0.601 ÷ 0.853, inferred). On the 4-core
   release target, whose L3 the record does not give, it cannot be bounded.

6. **P107 and P144, the window at ten threads.** The verdict: "The footprint lever is the
   T-vector per block × blocks in its phase, and nothing else" (REG P144), and the window was
   widened on CI's 4-thread evidence: 2 → 4 blocks a thread, −8.1% to −10.3% wall (WIN). Later:
   "The decoder's other per-block buffers, 14–16 MB in the occupancy ledger at the default
   window, also contend for the cache, and are a plausible reason the knee has moved since 29
   October. That is not shown" (TS); the three-table advantage "appeared between builds" (GPP
   §1.1); "Fewer output slots in flight (the window) … unpriced" (BW §5); "#180 widened it for
   CI". **What that makes possible:** a fourth table back, or a smaller live set for ten plain
   workers, by a window chosen for the i9. **Cheapest falsifier:** an i9 recipe with explicit
   `slots`: 4×2 at window 8 against 16, 3×3 at 6 against 12. One reading exists and is null for
   another plan: svc2 at window 12 against 8, 1.000 [0.972, 1.029] (SLA). **Worth:** cannot be
   bounded from the record. For scale: each slot reserves 2.7 MB, and ten threads reserve 139.5
   → 193.5 MB at 2 → 4 slots a thread (WIN).

7. **P103, P104 and P105, low-level files.** The verdicts: #115 "rejected (shipping)" because
   "The plan is chosen once, before the first block" (GPP §1.6); one core a block at -2/-3
   "pending the user"; and at -1 to -3 "the scanner binds first" (CD). Later: 0.2.6's default is
   0.763 [0.752, 0.775] of 0.2.4 at level 1 (PL); the fix "is made per block, so a mixed-level
   file needs no header read and no gate, unlike #115" (PL); the finder gives 2,563 MB/s at ten
   workers where the scan arm stops at 1,598 (PSR); adoption policy is delegated (CLAUDE.md).
   The recorded reopening condition, "A design that re-plans, or spawns helpers only when they
   can run", is met in design. **What that makes possible:** share-nothing shipped first where
   it needs no new table: blocks below `MIN_COOP` on plain workers, with the finder.
   **Cheapest falsifier:** an i9 recipe: the registered scheduling fix against forced 4×2 at
   levels 1 to 3, then with the finder. **Worth:** derived: at level 1, 2,563 ÷ 1,524 = 1.68
   over four plain workers with a scanner (PSR, one harness); over 0.2.6's default about 2.2
   (1.68 ÷ 0.763, inferred across harnesses).

8. **P173, the release's 8-thread verdict.** The verdict: "cap stays 4"; the extra CPU is
   "contention suspected, not demonstrated"; the open condition: "if the extra CPU is avoidable
   overhead rather than inherent latency-hiding cost, the wall gain might be had at less CPU"
   (REG P173). What was tested was 4×2 of 0.2.1. Later, at the decoder boundary on the i9:
   parking "cuts CPU by about 5% while the wall holds" (PLR); 3×2 against 4×2 spends 19–25%
   less CPU "at an unresolved speed" (PB); svc2@10 spends 12.7% less CPU than the default and
   is 1.045 faster (S23). So avoidable CPU in team plans is shown; the application run is not.
   D1, the per-role attribution "attribute ≥ 80% of the +40.6 CPU-s to roles" (PM §6), is not in
   the record as run. **Cheapest falsifier:** an i9 recipe at the application: D1 first, then
   one release-shaped pair. **Worth:** cannot be bounded from the record: the split's wall gain
   was −4.2% (−1.04 s of 25 s), and the decoder is under 11% of the run's wall (PM §3.3).

9. **P106, phase-aware gating.** The verdict: "Phase-aware scheduling is dropped as a remedy
   beyond the worker cap", on a construction whose best was 782 MB/s, with the reason an
   inference: "total live footprint (the waiting workers' scratch, the wider window) rather
   than active walks alone" (WS §8). Later: a plan that keeps two tables in inversion and moves
   the serial work to two service threads beats the default, 1.045 [1.017, 1.073] with 12.7%
   less CPU and 64 MB against 74 MB (S23), replicated in direction at 1.073 [1.041, 1.106]
   (SLA). The construction was rejected; the advantage was later shown. **Worth:** measured,
   not a bound: about 4.5 to 7.3% on the i9 at ten threads. The cause is unseparated ("C1 is a
   bundle", S23).

10. **P146, counters on the i9.** The register: "the i9 has a PMU". Later: "perf could not
    count on the machine (`perf_event_paranoid`), so there is no counter pass" (TS);
    "`perf_event_paranoid` is 4 on the workstation" (BW §1). Every mechanism stated after 29
    October (why three teams win, what the pipeline recovers, what binds at ten) is from timing
    and traces. KM withdrew its own claims for that reason. Hit shares of 0.987 to 0.539 are
    still listed as measured (KM E2; GPP §1.2) although the raw offcore shares of that period
    were labelled "unreconciled" (REG P151, K51). **Cheapest falsifier:** read of the archived
    `telemetry-check/20261109T125938Z` bundle. **Worth:** cannot be bounded. It is a fact only
    Louis can change (the workstation's state).

11. **P141, the A/A after the order fix.** The verdict: "After the fix: 0 of 42." Later: "At 1
    and 2 threads, τ is larger than one run's own SE … 2 of 9 nulls look like changes"; true
    A/A, 4 of 63 intervals exclude zero; and "The chain misses a third to a half of the
    1-thread gain" (BP). **What that makes possible:** nothing new to build; it marks which old
    verdicts are thin: any single-run CI verdict at 1 or 2 threads within about 0.7% and 1.0%
    of zero. **Cheapest falsifier:** a second CI run on another CPU model. **Worth:** cannot be
    bounded.

12. **P127, the early warning that already works on the i9.** The lead: "Does a rise in CI's
    2-spread amplification predict a lower knee on 10 cores?", open, with the test "two variants
    differing only in per-worker footprint". The level ladder is that pair: from 2 to 4 workers
    CPU rises 1.36× at -9 and 1.01× at -1 (WS §9); 0.636 → 0.853 s at level 9 and 0.720 →
    0.737 s at level 1 (PSR). **What that makes possible:** pricing a small table from a two- and
    four-worker reading of exact CPU-seconds, which is cheap and also the application's regime.
    **Cheapest falsifier:** a CI A/B of the two recompressions at 1 and 2 spread threads.
    **Worth:** cannot be bounded; it shortens every later test.

### What the record says binds at ten threads, with its evidence

- **Plain workers, 3.6 MB tables (28 October):** "the knee is memory contention in the T-vector
  phase, in two stages: L3 capacity … queueing past the LLC" (WS §8). Evidence: counters under
  `sudo perf` (demand L3 hit share 0.76 → 0.38, occupancy a missing read 396 → 858 cycles), IPC
  1.06 → 0.50, DRAM reads flat at 18.6–19.2 GiB/s from 8 threads, and the block-size ladder.
- **Teams of two (29 October):** "The L3 cap is what holds it" (CD). Evidence: timing only,
  five teams 1.162 against four teams 1.274.
- **Teams, current build (6 November):** "The 8-thread ceiling is the four live tables … It is
  not shown by counters: perf did not run" (TS). Evidence: per-table rates and traces; leaders
  decode 96% of the run.
- **Why three teams win (7 November):** "Inferred, not established" (KM). Footprint leads;
  synchronisation and core-sharing are "not ruled out". 29–33% of each block is outside the
  inversion call.
- **The pipelined default (7 to 9 November):** α "is not identified" (TPR). The service core's
  trace labels "don't identify the bottleneck" (SLA).
- **The default plan (10 November):** memory bandwidth is "a hypothesis with a ledger"; the
  ledger's β of 13.1 predicts 2.2 GB/s against 1.50 measured; the titration is registered and
  has no bundle on `evidence` (BW; `git ls-tree origin/evidence`). In the fast mode ten threads
  are "68% busy" (SN §3).
- **Plain workers, 10 November:** at level 9 "something else bounds ten workers, in both arms …
  It does not establish that memory is the cause" (PSR §4). At level 1 with the finder "the
  cores are full and nothing else binds" (SN §3b: wall is CPU ÷ 10, 0.84 s and 0.082 s). With
  one scanner, the scanner binds: about 1.6 GB/s (PSR).
- **CI at 4 threads on 2 cores:** "CPU-capacity-bound"; the walk's SMT co-run excess leads (BC).

## Changed since 30 October

- **P90:** counted price of cooperation and the level-1 prize (SN; PSR).
- **P91:** teams of two replaced at ten cores by 3×3 (3T; CL 0.2.5). **K34** with it.
- **P92:** 3×2 against 4×2 at 8 threads read, unresolved in wall, cheaper in CPU (PB).
- **P93:** the L3 read is now known by code reading (GPP; README).
- **P94:** packed against spread measured, −23% at 4×2 (TS).
- **P95:** production's walk moved under it (#157, #171); the gap counted (SN §1). **K28** unclosed.
- **P96:** residency supported on the full decoder by level (PBY); width buys little (TS).
- **P97, K36:** the question is now what binds the default plan (BW; SN).
- **P99:** the walk-ready waits tested (PLR); spins bounded (SPIN).
- **P100:** one table too many for teams (TS; 3T); premise supported for plain width 3 (PBY).
- **P101:** #180 changed the 2-against-4-thread balance on CI (WIN).
- **P102, P103, P104, P105:** the level 1–2 slowdown measured and a per-block fix registered
  (PL; CL 0.2.6); the scanner plateau removed by the finder (PSR).
- **P106:** superseded by the service core (SCR; S23; SLA).
- **P107:** rejected → shipped, #180 (WIN).
- **P108:** confirmed on the i9 (SPIN).
- **P109, P110, P111:** per-stage co-run factors; mechanism contested (BC; CP).
- **P112:** reproduced in shape without counters (PSR); for teams, unestablished (KM; BW).
- **P113:** the level-1 plateau shown to be the scanner (PSR); re-read as a price list (SN §2).
- **P115:** an out-of-sample prediction made and missed; its probe registered (BW).
- **P117:** kept as an open exception (KM E2); now on the critical path (SN §3b).
- **P118:** a simulated ledger of cold writes (BW).
- **P120, P121:** a two-tier table built and failed on instructions (BDC §3; SN §3a).
- **P124:** the scanned iMTF counted out (BDC §4a).
- **P125, P126, P127, P130, P131, P132:** WIN, JH and BC for amplification; BW for the hog and
  the simulated ledger; PSR for the ladder; TS for six points.
- **P134:** attributed and acted on (KM E4; TPR; PLR).
- **P140:** shares differ by data class (BDC §1).
- **P141, K11:** between-run τ and the chain against a direct run (BP).
- **P143:** sampler cost at 4 threads (BC).
- **P144:** contradicted at ten threads (TS; KM E5; BW).
- **P145, K57:** released in 0.2.2; two labels withdrawn (SLA).
- **P146:** unprivileged counters unavailable on the i9 (TS; BW).
- **P147, K44:** a busier host and two modes recorded (RFL1; RFL2; 2M).
- **P149:** the same signature at ten threads (2M; SPIN).
- **P152:** "a thread a block" is now the objective (SN).
- **P153, P157, K43:** same-plan and same-code offsets of 1.4% to 6.7% (3T; CL 0.2.4, 0.2.7).
- **P158:** replaced in practice (BP; `bench/release-scaling/`).
- **P161:** the one-core recipe exists (`.just/bench.just`).
- **K3:** huge pages back as a mode candidate (2M). **K5:** new RSS figures (WIN; CL 0.2.7).
- **K7, K48:** the scanner measured on the i9; #10 reopened (PS; PSR).
- **K8:** a shared resource costs at four workers on the i9 (PSR).
- **K10:** the local machine changed again (SC; BDC).
- **K30:** more dangling citations (3T; S23; README).
- **K37:** a further elasticity reading near zero (REG §6).
- **K39:** a register-held MTF list shipped as #207 (CL 0.2.4).
- **K52:** the thread-count rule written down (GPP; README).
- **K61:** P87 has production entries (`2026-11-06-team-neutral.md`).
- **P171:** one condition answered on the i9 (PSR).
- **P172, P173, P174:** the plans the cap and the 8-thread verdict assumed have moved (3T; PB;
  PLR; S23; CL 0.2.2 to 0.2.7).
- **P180:** its bandwidth member's premise moved on the i9 (PSR; BW).

## Reopen candidates

Rejected or superseded IDs whose rejection was of a construction only.

- **P95, cooperative kernel at production's per-core speed.** Rejected construction: the 29
  October kernel, 0.923 × p4. A new one is the port of #157 and #171 to `coop::traverse`. It
  would have to bring the team's walk from 17.8 toward the one-thread 10.1 instructions a byte
  (SN §1) and show a wall gain at 3×3 on the i9, where a closely related cut (the 1 November
  walk-unroll trial) read 1.000 (0.987–1.012) at four plain workers (REG §6).
- **P106, phase-aware gating.** Rejected construction: a condvar gate over ten full workers.
  The service core is the new construction and already reads 0.895 CPU-s against the default's
  1.026 at 0.137 s against 0.144 s (S23). What it still owes: a registration that qualifies
  under the target-gated route (`2026-11-09-target-gated-route-decision.md` on master), B5 and
  D042's fuzz evidence (S23).
- **P103 (and P102), teams or workers by level.** Rejected construction: a plan chosen once
  from the first header, 0.945 on a mixed-level file. A per-block rule would have to cost no
  more than "about 15 MB more peak memory at 3 × 3" (PL) and recover the 1.30–1.32 that forced
  4×2 shows at levels 1 and 2.
- **P173, more than 4 threads in the release.** Rejected construction: 4×2 of 0.2.1, split CPU
  +16.9%. A new arm would have to hold the split's CPU at or below auto (4)'s while keeping a
  wall gain; the candidates are 3×2 at 6 (0.850 against 1.097 CPU-s for 4×2, decoder alone, PB)
  and svc2 at 10 (S23). Wikidata-pq's own rule decides.
- **P91, team shape.** Already reopened and replaced; listed because "teams of 2" was a verdict
  on the `y` kernel and the 29 October build only.
- **P107, the window.** Already reopened and shipped on CI. Its ten-thread side is the open
  construction: a window chosen for the i9 would have to show a gain at 4×2 or 3×3 against the
  default's 16 and 12 blocks.
- **P152, "a thread a block" (pilot negative, 1.05× on the VM).** It lost where nothing pressed
  on the cache. With a table of about 1.2 to 1.4 MB and the finder it is the objective; the
  inversion would have to cost at most about 2.5 times production's at ten threads (SN §3b), or
  about 2.1 if Mined 2 or 3 holds, and about 1.6 at the application's four workers (Mined 5,
  inferred).
- **P119, K blocks in lockstep.** Rejected at K × 3.6 MB on the VM, before lanes. A new
  construction would have to beat 32 lanes inside one block, which the record gives no reason
  to expect; listed for completeness.
- **K39, register-held MTF.** #25's construction was rejected; #207's shipped. The remaining
  lead is the register-list checkpoint kernel, 0.9745 at 9 threads on CI (RFLci).

## Not resolved in this slice

- **K rows on section (a) to (c) subjects** were carried from the register without a later
  reading: K1, K2, K4, K6, K9, K13, K14, K16, K17, K18, K19, K20, K21, K22, K23, K24, K26, K31, K32, K33, K40, K42, K45, K46, K49, K50, K53, K54, K55, K59, K62. Their rows say "not re-read in this
  slice".
- **P159, P163:** no later reading made here.
- **P170, P174, K56, K58:** wikidata-pq's present default, cap and pin are as checked on 30
  October. This repository's record does not say whether they have moved.
- **P146:** the `telemetry-check/20261109T125938Z` bundle is on `evidence`; its result entry is
  not in the journal on this branch or on master. BW's one sentence is the only reading.
- **P115, P97, P118:** `bench-bw-titrate` is registered and frozen; no bundle for it is on
  `evidence`. PSR and master's PS say it "left the memory cause unresolved".
- **P106:** the `service-guarded/20261109T195345Z` run is known here only from master's commit
  log ("VALID, P1 met") and the route decision ("NOT SHOWN at 12 and 16"); its result entry was
  not found on master.
- **P151, K51:** whether KM E2's hit shares are the "unreconciled" raw event was not checked.
