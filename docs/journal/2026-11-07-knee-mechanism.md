# 2026-11-07: Why the decoder stops scaling at 8–10 threads, and why three teams help

**The question.** The three-team policy (#234, 0.2.5) is confirmed on the i9-10900K. 3 × 3 at
10 threads is 1.088–1.089× faster than 4 × 2, with 11% less CPU and 19% less peak memory. The
release-scaling run also shows 0.2.4 8–12% slower from 10 threads up. None of that says why it
works. This entry goes back through the i9 measurements already collected. It separates what
they establish from what is inferred, and turns the inference into the next experiments.

No new timing was run for it. Sources:
- **The team-size run:** `2026-11-06-team-size.md` and its directory: the timed grid, and the
  occupancy traces at 4 × 2, 4 × 3 and 4 × 4, spread and packed.
- **The compact-Ψ grid with counters:** `origin/research/compact-psi:docs/journal/2026-10-29-compact-psi.md`
  §5, its `coop-counters-20261029T215323Z.md` and the footprint run in `2026-10-29-libsais-knee.md`.
- **The cooperative decoder's gates:** `2026-10-29-cooperative-decoder.md`.
- **The two three-team runs and the release-scaling record:** `2026-11-06-three-team-policy.md`,
  `bench/release-scaling/`.
- **The CI stage harness:** `examples/phases.rs`, bench run 42310037840.

## Established (measured)

**E1. Throughput follows the number of live tables, not the thread count.** In the team-size
grid (exploratory cells, 20 rounds each), per-table MB/s by live tables:

| live tables | 2 | 3 | 4 | 5 | 6 | 8 |
|---|---|---|---|---|---|---|
| per-table MB/s | 558–583 | 461–466 | about 320 | 230 | 175 | 110 |

- At a fixed table count, the team size barely matters. 4 tables give 319, 325 and 320 MB/s at
  k = 2, 3 and 4.
- Total throughput peaks at 3 tables (1,382–1,399 MB/s), not at 4.

**E2. The tables compete for the L3, by their bytes.** These are the i9's counters from the
compact-Ψ grid: one pinned core per table, build and walk only.
- **The L3 hit share against live table MB** (production's 4-byte table): 0.987 at 2.4 MB,
  0.961 at 9.6, 0.929 at 14.4, 0.794 at 21.6, 0.640 at 28.8 and 0.539 at 36.0.
- **Read waits past L2:** 67 cycles at 4 tables, 137 at 6 and 361 at 8.
- **The DRAM plateau:** 28–29 GB/s past the knee, for every arm. A probe measured 26 GB/s for
  random reads.
- **The control:** the `v6` arm holds production's bytes in a different layout and gets
  production's hit share (0.814). That pins the effect on table bytes, not on the layout's
  instructions.
- **A smaller table moves the knee:** the 24-bit `u` table (2.7 MB a block) moved it from 4
  tables to 6 (u6/p4 1.100, 1.093–1.108).

**E3. Within a team, extra helpers don't shorten a block at a fixed table count.** At 4
tables, pinned spread, blocks take 1.394 ms at k = 2, 1.380 at k = 3 and 1.403 at k = 4. At 3
tables the timed rates are 461 against 466 MB/s at k = 3 and 4. The parallel phase is
saturated, and the saturation comes from the memory system, since more threads add nothing.

**E4. The serial stages are about a third of each team's time on the i9.** This is from the
occupancy traces. A table counts as live from the cooperative build to the end of the walk.
At 4 teams the mean live count is 2.75, 2.61 and 2.59 at k = 2, 3 and 4. So each team spends
65–69% of its time in build and walk. The other 31–35% goes to the leader's serial stages:
- **MTF**, which writes the checkpoints;
- **RLE1** and the **CRC**.

The helpers wait through them, and the waiting costs no CPU.

| plan (4 teams, pinned spread) | block | in build and walk | outside, serial on the leader |
|---|--:|--:|--:|
| 4 × 2 | 1.394 ms | 0.96 ms | 0.43 ms |
| 4 × 3 | 1.380 ms | 0.90 ms | 0.48 ms |
| 4 × 4 | 1.403 ms | 0.91 ms | 0.49 ms |

The CI stage harness agrees on the split. At 1 thread it measures:
- **serial:** MTF 0.41, assembly 0.04, RLE1 0.05 and CRC 0.07 ns per byte;
- **shared within a team:** build 0.48 and walk 1.09 ns per byte.

**E5. What a team keeps in memory.** From the traces' buffer ledger, per team workspace:
- the T-vector: 3.6 MB live (5.4 MB of capacity);
- the walk's segment buffers: 6.1 MB of capacity;
- RLE1's output: 1.35 MB;
- `ll`: 0.9 MB.

Peak RSS rises about 15 MB per team (52, 67, 82, 96, 110 and 138 MB at 2–8 tables). The
window adds about 6.3 MB of live output buffers and 1.6 MB of queued symbols. Live tables
average 9.3–10.2 MB at 4 teams. So the hot set reaches the L3's 20 MiB with the tables alone
at about 0.5 of it.

**E6. Placement matters most at k = 2.** At 4 tables, packed (SMT siblings first) against spread:
- **4 × 2:** 1,060 against 1,374 MB/s, −23%, with blocks of 1.753 against 1.394 ms;
- **4 × 3:** −7%;
- **4 × 4:** −2%.

Pinned spread 4 × 2 (1,374) also beat the timed unpinned 4 × 2 (1,276). That compares
different binaries (the trace build) and pinning by CPU mask, so it is a lead, not a
measurement.

**E7. Three teams win, as registered and confirmed.**
- **The two runs** at 10 threads: speed 1.089 (1.075–1.103) and 1.088 (1.076–1.101), CPU 0.88–0.89, RSS 0.81–0.82.
- **No harm** at 11, 12, 16 and 20 threads.
- **The release-scaling run:** 0.2.4 takes 11.9% longer than 0.2.5 at 10 threads, and 8.1–8.7% longer at 12–20.

## Inferred: why three teams win

This follows from E1–E5, but no counter run has shown it directly for the decoder in teams.
- **The mechanism.** With 4 teams the live tables, on average 2.6–2.75 of them, and their
  streams overrun the L3. Every walk step's read then costs more, and the build-and-walk phase
  takes about 0.9–0.96 ms a block whatever the team size.
- **Why 3 teams are faster.** With 3, fewer tables are live. Each block's build and walk
  speeds up enough that 3 teams of 3 deliver more than 4 teams of 2, from 9 cores rather
  than 8.
- **Why CPU and memory fall too.** CPU falls because fewer reads wait on DRAM. Memory falls by
  one team's workspace, about 15 MB.
- **The serial third.** It is now a large share of each team's time. It also relieves the L3,
  since while a team is in it, its table isn't live (E4). This matters for any change that
  makes teams busier.

## Hypotheses for the next experiments, ranked

**H1. Pipeline each team** (in progress: `research/team-pipeline`).
- **The change.** The leader runs block i+1's MTF, and block i−1's RLE1 and CRC, while the
  other k − 1 threads build and walk block i. That is one table per team, as now, and one
  more `ll` buffer, about 1 MB.
- **The prediction.**
  - A team's block time falls from serial plus parallel to the larger of the two.
  - At 3 teams, serial (about 0.35–0.45 ms) overlaps a parallel phase that k − 1 helpers
    can still saturate (E3).
  - Each team's table is then live nearly all the time: about 3 tables, about 10.8 MB, against
    today's estimated 2. That is about as much as 4 × 2 holds today.
  - So the walk slows toward 4 × 2's per-block phase.
- **Net:** 1.10–1.30× for pipelined 3 × 4 at 12 threads against 3 × 3 at 10. Pipelined
  3 × 3 at 10 should beat 3 × 3 by less, since only 2 threads walk.
- **The falsifiers.**
  - **No gain, and the trace shows mean live tables near 3 with the build-and-walk time
    risen to cancel it:** then the serial third's L3 relief was carrying the result, and the
    knee is pure table capacity. That points at H3, not at more overlap.
  - **No gain, with live tables unchanged:** the pipeline isn't overlapping, which is a bug.

**H2. Pin teams to distinct physical cores.**
- **The evidence:** E6, the −23% packed cost at k = 2.
- **The prediction.** It is small at 10 threads, where 9 threads on 10 cores leave the
  scheduler room to spread: 0–3%. At 12–20 threads, siblings can't be avoided. Pinning then
  decides which threads share a core: a team's own threads (one table, one walk) rather than
  two teams' (two tables). That is the better pairing by E2 and E6.
- **Cost:** cheap, Linux `sched_setaffinity` per team thread.

**H3. A smaller table under teams.**
- **The evidence:** E2. The `u` table moved the knee by two tables. Compact30 removed most of the
  1 → 4 contention penalty (+23% against +68%) for +15% per step at 1 core.
- **Never measured under cooperative teams** on the i9, at any table count.
- **It composes with H1.** H1 makes tables live more of the time, which makes table bytes the
  binding term. A smaller table then admits a fourth live team, or the third's tables fit
  with room.
- **The feasibility review's bound** (`2026-10-30-compact-psi-feasibility.md` §5): about
  1.1–1.15× for the decoder before per-step costs, with a fifth team. Its cost is a new
  cooperative kernel.

**H4. A diet for the non-table hot set.**
- **The target:** the walk's segment buffers (6.1 MB of capacity a team) and the window's output
  buffers (about 6.3 MB live) stream through the L3 beside the tables.
- **The levers:** shorter-lived buffers, or non-temporal stores for the bytes the reader will
  copy out anyway. Either would leave more of the L3 to the tables.
- **The evidence:** none on the i9. It is ranked last until a trace or counter run shows what
  share of the misses these buffers cause.

## What needs the i9, and what doesn't

**Done here, without new workstation timing:**
- H1's prototype, its correctness tests, and an indicative local run (on 4 cores and a large
  L3, where tables don't contend, isolating the overlap);
- H2's implementation as an arm;
- the registration of both, with the predictions above, and a recipe frozen for the next run;
- CI's paired bench, to check that the default path is unchanged.

**Needs the i9:** the measurement that decides H1 and H2. A frozen recipe, `just submit` (RULES
C5), will run them:
- the arms 3 × 3 (today's), 3 × 4, pipelined 3 × 3, pipelined 3 × 4, and both pinned;
- at 10, 12, 16 and 20 threads, with occupancy traces for the live-table count.

Nothing in this entry changes the decoder.
