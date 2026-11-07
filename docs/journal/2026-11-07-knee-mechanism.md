# 2026-11-07: Why the decoder stops scaling at 8–10 threads, and why three teams help

**Revised after review.** The first version (`853dd1af`) was reviewed adversarially by the
Researcher: REVIEW 6699591835 on #145, with the calculations at
`team-state/researcher:state/reports/knee-mechanism-6699480548/` (`570a89ee`). All seven
objections are accepted, and this version takes them in. What changed is listed at the end.

**The question.** The three-team policy (#234, 0.2.5) is confirmed on the i9-10900K. 3 × 3 at
10 threads is 1.088–1.089× faster than 4 × 2, with 11% less CPU and 19% less peak memory. The
release-scaling run also shows 0.2.4 8–12% slower from 10 threads up. None of that says why it
works. This entry goes back through the i9 measurements already collected. It separates what
they establish from what is inferred, and turns the inference into the next experiments.

No new timing was run for it. Sources:
- **The team-size run:** `2026-11-06-team-size.md` and its directory: the timed grid, and the
  occupancy traces at 4 × 2, 4 × 3 and 4 × 4, spread and packed.
- **The compact-Ψ grid with counters:** `origin/research/compact-psi:docs/journal/2026-10-29-compact-psi.md`
  §5, `coop-counters-20261029T215323Z.md` and `2026-10-29-libsais-knee.md`.
- **The cooperative decoder's gates:** `2026-10-29-cooperative-decoder.md`.
- **The two three-team runs and the release-scaling record:** `2026-11-06-three-team-policy.md`,
  `bench/release-scaling/`.
- **The CI stage harness:** `examples/phases.rs`, bench run 42310037840.

## Established (measured)

**E1. Throughput follows the planned team count, not the thread count.** In the team-size grid
(exploratory cells, 20 rounds each), per-table MB/s by planned teams. That is the maximum number
of concurrent tables, not a measured count of tables live at once.

| planned teams (maximum tables) | 2 | 3 | 4 | 5 | 6 | 8 |
|---|---|---|---|---|---|---|
| per-table MB/s | 558–583 | 461–466 | about 320 | 230 | 175 | 110 |

- At a fixed team count, the team size barely matters. 4 teams give 319, 325 and 320 MB/s at
  k = 2, 3 and 4.
- Total throughput peaks at 3 teams (1,382–1,399 MB/s), not at 4.

**E2. In an inversion-only harness, the tables compete for the L3 by their bytes.** These are
the i9's counters from the compact-Ψ grid: one pinned core per table, build and walk only, not
the cooperative whole decoder.
- **The L3 hit share against live table MB** (production's 4-byte table): 0.987 at 2.4 MB,
  0.961 at 9.6, 0.929 at 14.4, 0.794 at 21.6, 0.640 at 28.8 and 0.539 at 36.0.
- **Read waits past L2:** 67 cycles at 4 tables, 137 at 6 and 361 at 8.
- **DRAM:** 28–29 GB/s past the knee.
- **The control:** the `v6` arm holds production's bytes and gets production's hit share
  (0.814). That pins the effect on table bytes.
- **A smaller table moves that harness's knee:** the 24-bit `u` table, at 2.7 MB a block, moved
  it from 4 tables to 6 (u6/p4 1.100, 1.093–1.108).
- **What table bytes alone don't explain:** at level 3, 10 blocks keep a 0.940 hit share yet lose
  to 8 blocks, mainly in the build. A request or core-count limit is not ruled out.
- **What it doesn't cover:** there are no counter measurements of the cooperative decoder
  (perf was blocked in the team-size run).

**E3. Additional helpers buy little whole-decoder speed at a fixed team count. These
measurements do not separate memory limits from serial, synchronisation or core-sharing
costs.**
- **The timed comparisons:** 4 × 3 against 4 × 2 is 1.017 (1.008–1.026) with 26% more CPU;
  4 × 4 is 1.003 (0.993–1.014) with 52% more.
- **The traced blocks:** blocks take 1.394, 1.380 and 1.403 ms at k = 2, 3 and 4 (4 teams,
  pinned spread).
- **Explanations still open:**
  - pool wake and completion skew;
  - serial build preparation and assembly;
  - the build's write-window checks;
  - `FLAGGED_SEGMENTS × K` segmentation;
  - SMT pressure;
  - memory.

**E4. A team spends about 29–33% of each block outside the inversion call.** These are the
Researcher's recomputation, per run: the medians of six spread runs.
- **The method:** inversion duration = `q.tvec_mean` × span / blocks. The outside share =
  1 − `q.tvec_mean` / `w.mean_decoding`.
- **What the inversion call includes:** `Cooperative::invert_checkpointed` covers pool setup and
  dispatch, the build, the walk, completion waits and the leader-only segment assembly.
- **What is outside it:** MTF with its checkpoints, RLE1 and the CRC, plus scheduling and
  instrumentation.
- **So the total serial work can't be recovered from these summaries.**

| plan (4 teams, spread, full trace) | decode | inside the inversion call | outside it | outside share |
|---|--:|--:|--:|--:|
| 4 × 2 | 1.394 ms | 0.993 ms | 0.400 ms | 28.75% |
| 4 × 3 | 1.380 ms | 0.935 ms | 0.441 ms | 32.14% |
| 4 × 4 | 1.404 ms | 0.946 ms | 0.462 ms | 32.64% |

- **Agreement:** the timeline-mode traces agree, at 28.8%, 32.2% and 32.9%.
- **Block sizes:** the 431-block corpus includes small blocks, so these are per-block averages,
  not full-block service times.
- **Helper waits cost some CPU:** helpers spin up to 2,000 times before parking, so their waits
  are not free.
- **The CI stage harness's split** at 1 thread: MTF 0.41, assembly 0.04, RLE1 0.05 and CRC 0.07 ns
  a byte serial; build 0.48 and walk 1.09 ns a byte shared within a team.

**E5. What the traces' buffer ledger records.** These are capacities and high-water marks, not
bytes being accessed at the same time.

Per team workspace:
- **the T-vector:** 3.6 MB used, 5.4 MB of capacity;
- **the walk's segment buffers:** 6.1 MB of capacity;
- **`rle`:** 1.35 MB of capacity for the inverse BWT's output, which RLE1 reads;
- **`ll`:** 0.9 MB.

Shared:
- **the scanner's compressed-input window:** 6.27 MB at its maximum;
- **queued symbols:** about 1.6 MB.

Peak RSS rises about 15 MB per team (52, 67, 82, 96, 110 and 138 MB at 2–8 teams). How much of
this is hot in the L3 at once is not measured.

**E6. CPU-mask placement changes the result most at k = 2.** At 4 teams, packed against spread:
- 4 × 2: 1,060 against 1,374 MB/s, −23%;
- 4 × 3: −7%;
- 4 × 4: −2%.

The masks change how many physical cores are available (4 against 8 at the eight-thread mask).
They are not a per-thread affinity experiment.

**E7. Three teams win, as registered and confirmed.**
- **The two runs** at 10 threads: speed 1.089 (1.075–1.103) and 1.088 (1.076–1.101), CPU 0.88–0.89, RSS 0.81–0.82.
- **No harm** at 11, 12, 16 and 20 threads.
- **The release-scaling run:** 0.2.4 takes 11.9% longer than 0.2.5 at 10 threads, and 8.1–8.7% longer at 12–20.
- **The plan:** at every budget of 10 or more on this topology, the policy picks 3 × 3. A request
  for 12–20 threads still decodes on 9 threads.

## Inferred, not established: why three teams win

The leading hypothesis is a footprint effect.
- **The claim:** with 3 teams, fewer tables compete for the L3, so each block's build and walk
  is faster by more than the lost team costs.
- **For it:** E1's shape and E2's harness make it the strongest candidate.
- **Not ruled out:**
  - **Synchronisation:** E3's flat block times at a fixed team count may come from pool waits.
  - **Core-sharing and dispatch:** three teams of three leave a free core at 10 threads, which
    changes scheduling.
- **Not yet measured:** no counter or phase measurement of the cooperative decoder separates
  these.

## Hypotheses for the next experiments, ranked

**H1. Pipeline each team** (being prototyped on `research/team-pipeline`).
- **The change:** the leader runs block i+1's MTF, and block i−1's RLE1 and CRC, while the
  other k − 1 threads build and walk block i. One table per team, as now. One more `ll`
  buffer, about 1 MB.
- **The model** (the Researcher's): C0 = S0 + P0, C1 = max(S1, α · P0) + H, speed = C0 / C1.
  - S is the leader's stage.
  - α is the parallel stage's slowdown, from fewer workers and more contention.
  - H is the handoff cost that can't overlap.

  From the grid's 3 × 3 cycle proxy (1.056 ms per team per block) and S = 0.40 ms:

  | α | H = 0 | H = 0.05 ms |
  |--:|--:|--:|
  | 1.0 | 1.61× | 1.50× |
  | 1.2 | 1.34× | 1.26× |
  | 1.4 | 1.15× | 1.09× |
  | 1.6 | 1.01× | 0.96× |

  - If the parallel stage slows toward 4 × 2's inversion (0.94–0.99 ms), the gain is 1.07–1.12×
    with no extra overhead, and 1.02–1.07× with 0.05 ms of it.
  - **The archive doesn't bound α.** Null and regression are credible outcomes and will be
    registered as such.
- **The comparisons to register:**
  - **Causal:** pipelined against non-pipelined, at the same plan, budget and placement:
    3 × 4 against 3 × 4, and 3 × 3 against 3 × 3.
  - **Product, kept separate:** pipelined 3 × 4 against today's default (3 × 3) at 10 threads.
  - **Reported with wall time:** CPU, and peak RSS.
- **What counts, the reviewer's outcome map:**

  | observation | interpretation allowed |
  |---|---|
  | no cross-block overlap shown in the trace | the implementation or the instrument is unresolved; no mechanism verdict |
  | overlap shown, wall better than the registered threshold | the pipeline helps on this machine and workload; the mechanism still needs stage evidence |
  | overlap shown, no gain, build and walk service longer | added service cost cancels the overlap; locate it with phase timing and counters |
  | overlap shown, service unchanged, synchronisation, leader or dispatch cost up | pipeline overhead or another stage limits it |
  | trace invalid, or the interval spans both gain and harm | inconclusive, not a falsification |

- **What the prototype needs:** per-block phase marks on every thread that does the work:
  - MTF;
  - build setup and the build's work and waits;
  - the walk's work and waits;
  - assembly;
  - RLE1 and CRC;
  - publish.

  Cross-block overlap must be shown from the trace, not inferred. Requested of the prototype.

**H2. Placement as a matched control.**
- **The arms:** pinned against unpinned, recording the actual mapping of threads to CPUs. Run it
  for today's plan and for the forced 3 × 4 plans.
- **What isn't claimed:** no expected size, and no preferred pairing (same-team siblings against
  different teams').
- **Why it runs with H1:** it is needed to read H1's result.

**H3. A smaller table under teams.**
- **For it:** E2's footprint evidence is real.
- **Against it:**
  - u6/p4's 1.100 is an inversion-only gain, not a gain against cooperative production. The same
    archive gives u6/l4×2 0.687 (0.680–0.695).
  - Staged compact30 lost 4.8% and 7.7% of 4-thread wall on CI.
  - The feasibility review's premise ("four teams already keep tables in cache") is contested by
    E1.
- **Status:** never measured under cooperative teams. Ranked behind H1 and H2.

**H4. A diet for the non-table buffers.**
- **The evidence:** none for how much of the L3 these buffers take (E5 is capacities).
- **A caution:** non-temporal stores are not automatically better for data that is soon read
  back.
- **Status:** last.

## What needs the i9, and what doesn't

**To be done here,** without new workstation timing (in progress at the time of writing):
- H1's prototype, with phase marks, correctness tests and an indicative local run;
- H2's pinning arm;
- the registration above, frozen with a recipe;
- CI's paired bench, to check that the default path is unchanged.

**Needs the i9:** the frozen recipe, run with `just submit` (RULES C5):
- **arms:** 3 × 3 and 3 × 4, each pipelined and not, pinned and not;
- **thread counts:** 10 and 12;
- **with:** occupancy traces carrying the new phase marks.

Nothing in this entry changes the decoder.

## What the review changed

- **E1:** the axis is planned teams, not live tables.
- **E2:** marked as an inversion-only harness, with the level-3 counterexample added.
- **E3:** the memory-saturation claim is withdrawn and replaced by the reviewer's wording, with
  the timed 4 × 3 and 4 × 4 ratios and their CPU.
- **E4:** the denominator is corrected. The inversion call includes assembly, setup and pool
  waits, and helper waits are not CPU-free.
- **E5:** the labels are corrected. The 6.27 MB is the compressed-input window, and `rle` is the
  inverse BWT's output. The 20 MiB hot-set claim is withdrawn.
- **E7:** the plan at budgets of 10 or more is noted.
- **The inference section:** now only a leading hypothesis.
- **H1:** the 1.10–1.30× floor is replaced by the sensitivity model. Matched causal comparisons
  are kept separate from the product comparison, the outcome map is added, and phase marks are
  required.
- **H2:** the 0–3% estimate and the pairing claim are removed; H2 is now a matched control.
- **H3:** u6/l4×2 and the compact30 CI losses are added.
- **H4:** the hot-set claim it rested on is removed.
