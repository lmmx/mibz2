# 2026-11-07: A general rule for how the decoder spends its threads (design, not for merge until review)

**What this is.** The decoder's choice of how to spend the threads a caller gives it is, today,
one general rule plus one exception written for a single machine. This entry audits what is
actually measured about that choice, proposes three general rules that could replace the
exception, works out what each would do on machines nobody here has, and lays out how one of
them could reach production. It changes no code. Its one script,
[`2026-11-07-general-plan-policy/plan_tables.py`](2026-11-07-general-plan-policy/plan_tables.py),
is arithmetic only: it prints every plan table below from the rules as written here.

**The answer, in short.** Rule A below, the *two-budget rule*:

> Run as many teams as the L3 cache holds at about 6 MB of cache each, three threads wide and
> pipelined, when the cores allow that with one core of the budget to spare and it uses at least as many
> threads as the old plan would. Otherwise use the old plan: teams of two, as many as keep the
> tables near 0.7 of the L3.

On the i9-10900K it picks exactly what 0.2.6 picks at every thread count, and on CI's runners
it picks what CI runs today. It differs from today only on machines where nothing has been
measured, so the proposal is to ship the general code first with its reach limited to
measured topologies, and to widen that reach only on evidence, by a decision reserved to Louis.

## The terms

- **Block.** bzip2 compresses in independent blocks of up to 900,000 bytes (at level 9; level
  *n* allows *n* × 100,000). The decoder decodes several blocks at once.
- **Table** (the T-vector). Undoing the Burrows–Wheeler transform on a block needs a table of
  4 bytes a row: 3.6 MB for a full level-9 block. Decoding is mostly a chain of dependent reads
  through that table, so its speed depends on whether the table is in the CPU's cache.
- **L3.** The last and largest cache, shared by the cores: 20 MiB on the i9-10900K.
- **Physical core, SMT.** The i9 has 10 cores, each running two hardware threads (SMT, 20 CPUs
  in all). Two threads on one core share it. CI's runners show 4 CPUs on 2 cores.
- **Team, T × k.** A team is *k* threads that decode one block together, sharing one table. A
  plan of T × k runs T teams at once, so T tables are live. Today's teams have one *leader*,
  which does the block's serial stages (move-to-front before the inversion; reassembly, run-length
  decoding and the CRC check after it), and *k* − 1 *helpers*, which build the table and walk it
  with the leader.
- **Workers.** The plan without teams: N threads, each decoding its own block alone.
- **Pipelined team.** A team whose leader overlaps its serial stages for the next and previous
  blocks with its helpers' build and walk of the current one (#247). It runs with two settings
  on, *park* (waiting helpers sleep instead of spinning) and *small* (after a small block the
  leader keeps preparing the next one) (#249).
- **Budget, c.** The physical cores the decoder may use: the threads asked for, capped at the
  physical cores in the process's CPU affinity. Threads asked for beyond the physical cores are
  never used.
- **`MIN_COOP`.** Blocks below 262,144 rows (16 checkpoints of 16 Ki) are inverted on one core
  even inside a team: its helpers sit idle. Every block at levels 1 and 2 is below it.

## Today's policy, exactly (master at 4aa67e79, release 0.2.6)

`plan_with` in `src/parallel.rs` (lines 1184–1201):
1. topology unread (anything but Linux), or c < 6: N workers;
2. **exactly 10 physical cores, an L3 read as exactly 20 MiB, and at least 10 threads asked
   for: 3 × 3;**
3. otherwise teams of 2, T = min(⌊c / 2⌋, round(0.7 × L3 / 3.6 MB)), or 4 if the L3 is unread.

`StreamDecoder::defaulted` (lines 133–140) then runs the teams pipelined, with park and small,
**only when the plan is exactly 3 × 3**; every other plan runs plain.

So two decisions are tied to one machine's identity: how to allocate threads (workers or teams,
how many teams, how wide) and how to schedule them (plain or pipelined). A 12-core CPU with a
20 MiB L3, or a 10-core one with 19.25 MiB, gets the old plan, though nothing about the
mechanism behind 3 × 3 depends on those numbers being exact.

## 1. What is measured

Every figure here is from the i9-10900K, on the bench's Wikidata sample (11.85 MB of bzip2 at
level 9, 209.7 MB decoded), unless the row says otherwise. **M** = measured, **I** = inferred
from measurements, **U** = unknown. "Registered" means the comparison and its pass rule were
fixed before the run; "exploratory" means it was not.

### 1.1 Per-team rate against team count

From the team-size grid (`2026-11-06-team-size.md`, 20 rounds a cell, exploratory). Per-table
rate is total MB/s over planned teams; the tables' share of the L3 is T × 3.6 MB / 20.97 MB.

| teams T | tables / L3 | plans measured | total MB/s | per table MB/s | status |
|--:|--:|---|--:|--:|---|
| 2 | 0.34 | 2 × 3 @ 8, 2 × 4 @ 8, 2 × 4 @ 10 | 1,116–1,166 | 558–583 | M |
| 3 | 0.52 | 3 × 3 @ 10, 3 × 4 @ 12 | 1,382–1,399 | 461–466 | M |
| 4 | 0.69 | 4 × 2 @ 8, 4 × 3 @ 12, 4 × 4 @ 16 | 1,276–1,300 | 319–325 | M |
| 5 | 0.86 | 5 × 2 @ 10, 5 × 3 @ 16 | 1,147–1,152 | 229–230 | M |
| 6 | 1.03 | 6 × 2 @ 12 | 1,051 | 175 | M |
| 8 | 1.37 | 8 × 2 @ 16 | 882 | 110 | M |
| 3 | 0.52 | **3 × 2 at any budget, current build** | | | **U** |
| 3 | 0.52 | 3 × 2 @ 6 on the 29 October build | 1,155–1,189 | 385–396 | M (older build) |

- Total throughput peaks at three tables (M). The registered A/B then confirmed 3 × 3 at 10
  threads against 4 × 2: 1.089 (1.075–1.103) and 1.088 (1.076–1.101), with 11% less CPU and 19%
  less peak memory (`2026-11-06-three-team-policy.md`).
- Why it peaks there is **I**: the leading reading is that fewer tables compete for the L3, but
  synchronisation and core-sharing are not ruled out (`2026-11-07-knee-mechanism.md`).
- The width a team needs at three tables is **U** on the current build. The one 3 × 2 cell is
  from an older build, at a budget of 6 threads, and puts 3 × 2 at about 0.94–0.97 of
  4 × 2 (I, across builds).

### 1.2 L3 hit share against footprint

From the compact-Ψ grid's counters (one pinned core a table, the build and walk only, not the
whole decoder), as summarised in the knee entry:

| live table MB | 2.4 | 9.6 | 14.4 | 21.6 | 28.8 | 36.0 | status |
|---|--:|--:|--:|--:|--:|--:|---|
| L3 hit share of the walk's loads | 0.987 | 0.961 | 0.929 | 0.794 | 0.640 | 0.539 | M (inversion-only harness) |

- A smaller table (2.7 MB) moved that harness's knee from 4 tables to 6 (M, inversion only).
- **The whole decoder's hit share, at any plan: U.** perf could not count in the team-size run.
- **How much of the L3 the other per-team buffers take: U.** Their capacities are known (about 15
  MB of peak memory a team), their hot share is not.
- **In the workers decoder** (before teams), the best worker count moved inversely with block
  size from level 9 to level 6: 4 × 3.6 MB = 6 × 2.4 MB = 14.4 MB of tables, about 0.7 of the L3
  (`2026-10-28-workstation-scaling.md` §9, M). That is where today's 0.7 comes from.
- **In the team decoder it did not:** at level 6, five teams of two (12 MB of tables) tied four
  (1.006, 0.990–1.021) for 21% more CPU (`origin/research/level-policy`, M). So the team count is
  not set by table bytes alone. This is the strongest caution against extrapolating any L3 rule.

### 1.3 Team width at fixed team count

| teams | k = 2 | k = 3 | k = 4 | status |
|--:|---|---|---|---|
| 4 | 1.000 (ref) | 1.017 (1.008–1.026), CPU 1.26 | 1.003 (0.993–1.014), CPU 1.52 | M, registered (both failed the 1.03 bar) |
| 4, per table | 319 | 325 | 320 | M |
| 3, per table | U (current build) | 461 | 466 | M |
| 2, per table | U | 558 | 579–583 | M |
| 5, per table | 230 | 229 | | M |

Width beyond two buys almost nothing at four tables (M); at three tables k = 4 buys about 1% over
k = 3, at 12 threads on 10 cores, so with SMT sharing (M, exploratory). Pinned spread against
packed matters most at k = 2: −23% when 4 × 2's threads share cores (M, a CPU mask, not
per-thread placement).

### 1.4 Plain against pipelined

| comparison | speed (95%) | CPU | status |
|---|---|---|---|
| 3 × 3 pipelined vs plain, 10 threads | 1.042 (1.016–1.068) | 9% more | M, registered |
| 3 × 4 pipelined vs plain, 12 threads | 1.032 (1.020–1.044) | 11% more | M, registered |
| 3 × 3 pipelined + park + small vs plain, 10 threads | 1.072 (1.053–1.091) | about 5% more | M, reported |
| 3 × 4 pipelined + park + small vs plain, 12 threads | 1.074 (1.058–1.090) | about 8% more | M, reported |
| **0.2.6 vs 0.2.5 (the confirmation), 10 / 11 / 12 / 16 / 20 threads** | 1.072 / 1.070 / 1.069 / 1.065 / 1.070, all lower bounds ≥ 1.045 | 1.04–1.05× | M, registered |
| 0.2.6 vs 0.2.5 at 8 and 9 threads (both 4 × 2 plain) | 1.002, 1.001 | 1.00 | M (controls) |
| **2 × 4 pipelined (no levers) vs 2 × 4 plain, 10 threads asked for, 8 used** | medians 0.159 s vs 0.181 s (about 1.14) | U | **M, unregistered, no interval** |
| 2 × 2 pipelined vs plain | about 0.92 (pipelined slower) | more | indicative only: an agents' shared 4-core machine, not the i9 or CI |
| k = 2 pipelined on the i9, any T | | | **U** |
| 3 × 3 pipelined with a budget of exactly 9 (no core of the budget to spare) | | | **U** |
| 3 × 4 pipelined + levers vs 3 × 3 pipelined + levers | about 1.02 for about 19% more CPU | | I (from ratios in different runs) |

The pipeline's model (`2026-11-07-knee-mechanism.md`, H1) is speed = (S + P) / (max(S, αP) + H),
S the leader's serial stage, P the parallel stage, α how much the parallel stage slows when the
leader leaves it, H the handoff. Measured 3 × 3 fits α ≈ 1.39–1.55 depending on the assumed H;
**α is not identified** (`2026-11-07-team-pipeline-result.md`). With k = 2 the leader's leaving
halves the parallel stage's threads, so α near 2, and the model then predicts a loss whenever
S < P (I; S ≈ 0.40 ms, P ≈ 0.65 ms a block at 3 × 3).

### 1.5 CI's runners

- **Topology:** 4 CPUs on 2 physical cores (EPYC 7763 and 9V74; `2026-10-27-four-thread-ceiling.md`, M).
  The bench runs 1, 2 and 4 threads. With c ≤ 2 < 6, every plan there is workers: **CI's bench
  has never run a team plan or a pipelined team** (M, from the code and the workflow).
- **Spreading two threads over two cores, rather than packing them on one,** is worth +40% on
  both EPYC models (M).
- The bench runs on `ubuntu-latest`; no workflow uses GitHub's larger runners (`.github/workflows/*.yml`: `ubuntu-latest`, `ubuntu-22.04`, `ubuntu-24.04`,
  `windows-latest`, `macos-15`). macOS and Windows read no topology, so they always plan workers.
- **The 4-core pass on the i9** (CPUs 0–3, one of each of 4 cores): the policy is workers, 0.997
  (0.980–1.014) against plain workers (M).

### 1.6 Level (block size)

All of these are from the 29–30 October builds, **before teams of three and before the
pipeline** (M on those builds; U on the current one).

| level | rows a block | best workers (workers decoder) | 5 × 2 vs 4 × 2 (teams decoder, 10 vs 20 threads) |
|--:|--:|---|---|
| 1 | 100 K | 6–10, tied | 1.150 (1.138–1.162) |
| 2 | 200 K | | 1.216 (1.193–1.241) |
| 3 | 300 K | 6–8 | 1.075–1.092 |
| 4 | 400 K | | 1.085 (1.064–1.105) |
| 5 | 500 K | | 1.049 (1.034–1.064) |
| 6 | 600 K | 6 | 1.006 (0.990–1.021) |
| 9 | 900 K | 4 | 0.917 (0.907–0.927) |

- At levels 1 and 2 every block is below `MIN_COOP`, so **a team is one core**: 4 × 2 was 4
  one-core decoders, and the gain at 5 × 2 was a fifth decoder, not cooperation (M, by the code
  and by `auto`@4 against `auto`@20, 1.005–1.007).
- **A rule using the level was built and failed its shipping gates** (#115, closed unmerged): it
  met its gates at levels 1 to 3, but its first run fell short on a level-6 control, and its
  revision ran a level-3-then-level-9 concatenation at 0.945 (0.930–0.959). The plan is chosen once, before the first block, and a file's
  later streams can have bigger blocks.
- **Inferred, not measured: 0.2.5 and 0.2.6 decode level-1 and level-2 files with three
  one-core decoders at 10 or more threads, where 0.2.4 used four.** By the 29 October runs that
  costs on the order of 15–25% on such files (the default at 6 threads, 3 teams, ran at
  0.76–0.83 of itself at 20 threads, 4 teams, at every level). bzip2 and the dumps this project
  serves use level 9, so the reach is small, but it is a regression no run has looked for.

### 1.7 Release scaling (the records in `bench/release-scaling/`)

Medians of 10 paired rounds, MB/s (the site's generated table is built from these records):

| threads | 1 | 2 | 4 | 8 | 10 | 12 | 16 | 20 |
|---|--:|--:|--:|--:|--:|--:|--:|--:|
| 0.2.4 (4 × 2 from 8 up) | 397 | 701 | 1,021 | 1,293 | 1,272 | 1,269 | 1,267 | 1,261 |
| 0.2.5 (3 × 3 from 10 up) | 403 | 700 | 1,016 | 1,302 | 1,415 | 1,368 | 1,371 | 1,362 |

No run covers 5, 6, 7 or 9 threads, and the 0.2.6 record is still in the workstation queue (U).

### 1.8 The constraints a general rule must meet

From the above, the measured facts any rule must reproduce on the i9 (or justify departing from):
1. c < 6: workers (the 4-core pass, and CI).
2. c = 6–9: teams of two; at 8 and 9, four of them (registered controls, unchanged since 0.2.4).
3. c = 10: three teams of three, pipelined with park and small (two registered A/Bs and the 0.2.6
   confirmation).
4. Never k = 2 pipelined (no evidence for it; the model predicts a loss).
5. Threads beyond the physical cores are not used.

And what nobody knows: whether 3 × 2 at 8, 3 × 3 at 9 or 2 × 4 pipelined at 8 would beat 4 × 2;
whether anything here holds on another CPU; anything about levels on the current build.

## 2. Three candidate rules

Each rule is a pure function of its inputs, so its answer can be printed for any machine. All
three keep step 1 of today's policy (unread topology or c < 6: N workers) because nothing
measures teams below six cores.

### Rule A: the two-budget rule

**Inputs:** N threads asked for; P physical cores in the affinity (or unknown); L, the L3 bytes
(or unknown); whether that L3 is shared by every usable core (new; unknown counts as no); block
size from the first stream header, **reported but not used** (§1.6: the team decoder's best
team count did not follow table bytes, and the one level-aware rule that shipped failed).

**Constants:** F = 3.6 MB (a full table); 0.7 (today's); **6 MB of L3 per fast team**, the one new
constant; width 3 for fast teams.

**The rule:**
1. P unknown, or c = min(N, P) < 6: N workers.
2. The old plan: T_b = min(⌊c / 2⌋, round(0.7 L / F)) teams of 2 (4 for round(…) when L is
   unknown), plain.
3. If L is known and shared by every usable core: T_cap = ⌊L / 6 MB⌋, T_f = min(T_cap, ⌊(c − 1) / 3⌋).
   **Use T_f × 3, pipelined with park and small, if T_b > T_cap** (the old plan puts more tables
   in the L3 than the fast budget allows) **and 3 T_f ≥ 2 T_b** (the fast plan runs at least as
   many decoding threads).
4. Otherwise the old plan.

**The spare core.** ⌊(c − 1) / 3⌋ keeps one core of the budget free of teams, for the scanner
and the reader. Every measured 3 × 3 ran with a budget of 10 or more, so 9 team threads and one
spare; whether 3 × 3 also pays on a budget of exactly 9 is unknown (§1.4), and the rule doesn't
assume it.

**Scheduling:** pipelined if and only if k ≥ 3. There is no separate switch: the width decides.

**The constant.** The i9 pins T_cap = 3, which holds for any figure in (5.24, 6.99] MB of L3 per
fast team (20.97 MB / 4 to 20.97 MB / 3). 6 MB is a round number inside that interval, not a fit.
Where in the interval it sits changes the answer on other caches:

| L3 per fast team | 10 cores, 20 MiB | 12 c, 20 MiB | 8 c, 16 MiB | 16 c, 32 MiB | 32 c, 64 MiB |
|---|---|---|---|---|---|
| 5.30 MB | 3 × 3 pipelined | 3 × 3 pipelined | 3 × 2 | 5 × 3 pipelined | 10 × 3 pipelined |
| 6.00 MB | 3 × 3 pipelined | 3 × 3 pipelined | 2 × 3 pipelined | 5 × 3 pipelined | 10 × 3 pipelined |
| 6.99 MB | 3 × 3 pipelined | 3 × 3 pipelined | 2 × 3 pipelined | 7 × 2 | 9 × 3 pipelined |

**At the edges:**
- *Unknown topology* (macOS, Windows, a container hiding sysfs): workers, as today.
- *Requests above the physical cores:* capped at P, as today.
- *L3 not shared by all usable cores* (multi-die AMD parts, two sockets) or unknown sharing: the
  old plan, which is today's.
- *Hybrid CPUs* (performance and efficiency cores): the physical-core count mixes core types.
  Proposed: treat as not shared (old plan) until measured.
- *Tiny streams* (fewer blocks than teams): the plan only spawns threads; a team's helpers start
  at its first block above `MIN_COOP`. Unchanged from today.
- *Small blocks and levels 1–9:* no change from today at any level. At levels 1–2 that keeps the
  inferred regression of §1.6 (three one-core decoders on the i9); fixing it is a scheduling
  change, not a plan change (§5).

**Where it differs from today:** nowhere on the i9 (every thread count, every CPU mask) and
nowhere on CI. It differs on any other topology where the old plan puts more tables in the L3
than 6 MB each allows and the cores can fill wider teams (table in §3).

### Rule B: a throughput model fitted to the per-team rates

**Inputs:** as rule A.

**The rule:** choose the T × k (k ∈ {2, 3, 4}, T × k ≤ c) that maximises

  Q = T × r(T F / L) × p(k),

where r(x) is the per-table rate of §1.1 as a function of the tables' share of the L3,
interpolated linearly through the six measured points and held flat below the first (no
extrapolated gain), and p(k) is the pipeline's gain: 1 at k = 2 (no evidence), 1.072 at k ≥ 3.
Width otherwise counts for nothing (§1.3). Plans within 2% of the best are tied, and the tie
goes to the fewest threads. Pipelined if and only if k ≥ 3. Unknown L3: rule A's answer.

**At the edges:** as rule A for unknown topology, requests above the cores, sharing and level.
It extrapolates r(x) to any cache by the share x alone, so it assumes another CPU's per-table
rate falls with the L3 share exactly as the i9's does, at the i9's memory latency and clock.

**Where it differs from today:** on the i9 at 8 threads (3 × 2) and 9 threads (3 × 3 pipelined, with no core of the budget
to spare); and on most hypothetical parts, including one-team plans on 8 MiB caches (1 × 3
pipelined, which nothing supports). Its 8-thread prediction, about 1.08 for 3 × 2 over 4 × 2, is
contradicted by the older build's 3 × 2 cell at 6 threads (§1.1; I, across builds and budgets).

### Rule C: calibrate at startup

**Inputs:** as rule A, plus a probe: the first decoder in a process builds T synthetic full
tables for T = 2, 3, … and walks them concurrently for a few milliseconds each, and takes the T
where the aggregate walk rate stops rising as T_cap. Width and scheduling then follow rule A's
steps 2–4. Fallback: rule A when the probe can't run (c < 6, unread topology) or when two probes
disagree.

**At the edges:** a probe costs on the order of 10–30 ms against a 150 ms decode of the bench
sample, more than the whole decode of a small file; it adds noise from whatever else the machine
runs, and two runs of the same program could choose different plans.

**What it predicts:** nothing on paper. On the i9, the inversion-only harness of §1.2 put the
knee at four tables, not three, so a walk-only probe would likely choose 4 × 2 at 10 threads and
undo the measured 3 × 3 gain (I). To pick three it would have to be calibrated against the whole
decoder, which needs the very measurements rule A already encodes.

## 3. What each rule predicts

From `plan_tables.py`. "Pipelined" always means with park and small.

**The i9-10900K, every budget (N threads asked for; 10 cores, 20 MiB L3):**

| threads | today (0.2.6) | rule A | rule B |
|--:|---|---|---|
| 4, 5 | N workers | N workers | N workers |
| 6, 7 | 3 × 2 | 3 × 2 | 3 × 2 |
| 8 | 4 × 2 | 4 × 2 | **3 × 2** |
| 9 | 4 × 2 | 4 × 2 | **3 × 3 pipelined** |
| 10–20 | 3 × 3 pipelined | 3 × 3 pipelined | 3 × 3 pipelined |

**The i9 under CPU masks** (one CPU of each of c cores; the L3 stays 20 MiB): 4 cores → workers
under every rule; 6 cores → 3 × 2 under every rule; 8 cores → 4 × 2 (today, A) or 3 × 2 (B); 9
cores → 4 × 2 (today, A) or 3 × 3 pipelined (B), at any N ≥ c.

**CI's runners** (4 CPUs, 2 cores): 1, 2 and 4 workers at 1, 2 and 4 threads under every rule.
macOS and Windows: workers.

**Hypothetical parts, all CPUs asked for** (← marks a change from today):

| cores | L3 | today | rule A | rule B |
|--:|---|---|---|---|
| 6 | 8 / 12 MiB | 2 × 2 | 2 × 2 | 1 × 3 p ← / 2 × 3 p ← |
| 6 | 16 MiB | 3 × 2 | 3 × 2 | 2 × 3 p ← |
| 6 | 20–64 MiB | 3 × 2 | 3 × 2 | 3 × 2 |
| 8 | 8 / 12 MiB | 2 × 2 | 2 × 2 | 1 × 3 p ← / 2 × 3 p ← |
| 8 | 16 MiB | 3 × 2 | 2 × 3 p ← | 2 × 3 p ← |
| 8 | 20 MiB | 4 × 2 | 4 × 2 | 3 × 2 ← |
| 8 | 32 / 64 MiB | 4 × 2 | 4 × 2 | 4 × 2 |
| 10 | 16 MiB | 3 × 2 | 2 × 3 p ← | 3 × 3 p ← |
| 10 | 20 MiB | 3 × 3 p | 3 × 3 p | 3 × 3 p |
| 10 | 32 / 64 MiB | 5 × 2 | 5 × 2 | 5 × 2 |
| 12 | 20 MiB | 4 × 2 | **3 × 3 p ←** | 3 × 3 p ← |
| 12 | 32 MiB | 6 × 2 | 6 × 2 | 5 × 2 ← |
| 16 | 20 MiB | 4 × 2 | 3 × 3 p ← | 3 × 3 p ← |
| 16 | 32 MiB | 7 × 2 | 5 × 3 p ← | 5 × 3 p ← |
| 16 | 64 MiB | 8 × 2 | 8 × 2 | 8 × 2 |
| 32 | 32 MiB | 7 × 2 | 5 × 3 p ← | 5 × 3 p ← |
| 32 | 64 MiB | 13 × 2 | 10 × 3 p ← | 10 × 3 p ← |

(Rows for 8 and 12 MiB caches at 10–32 cores are the 6- and 8-core rows again: 2 × 2 today and
under rule A, 1 × 3 p and 2 × 3 p under rule B. The script prints all 36.)

Rule A's changes are all of one kind: on a part where the old plan holds tables beyond the fast
budget and the cores can fill wider teams, it trades about a quarter of the tables for width and
pipelining, as 3 × 3 did on the i9. Every one of those rows is unmeasured.

## 4. Falsifiable predictions, and the hardware that can test them

### What the i9 can test

Rule A changes nothing on the i9, so the i9 can't show it is *better* there; it can show whether
its boundaries are wrong. Those boundaries are exactly the cells where rule B, and the one
unregistered cell, say something else:

| budget | rule A's (today's) plan | alternatives | A predicts | B predicts |
|--:|---|---|---|---|
| 8 | 4 × 2 | 3 × 2; 2 × 4 pipelined | neither beats 4 × 2 by 3% | 3 × 2 about 1.08 faster |
| 9 | 4 × 2 | 3 × 3 pipelined, the whole budget | does not beat 4 × 2 by 3% | about 1.16 faster |
| 10 | 3 × 3 pipelined | 4 × 2 (old) | 3 × 3 p faster (already measured: about 1.16 compounded) | same |
| 6 | 3 × 2 | 2 × 3 pipelined | does not beat 3 × 2 | does not beat 3 × 2 |

The 2 × 4 pipelined arm comes from the unregistered 0.159 s against 0.181 s cell (§1.4): if it
beats 4 × 2 at 8, rule A's "never fewer than the old plan's threads" clause is wrong at small
budgets, and a third budget (teams of four when T_cap ≤ 2) would be indicated.

### What CPU masks test, and what they don't

`taskset` with one CPU of each of c cores makes the decoder see c physical cores
(`physical_cores` reads the affinity). It emulates a c-core part **with the i9's 20 MiB L3, its
memory system and its clock**:
- **It tests** the core-count branches: 6-, 8- and 9-core masks reproduce budgets 6, 8 and 9 with
  no spare cores outside the mask for the scanner and the OS, which is what a real 8- or 9-core
  part has.
- **It does not test** anything about cache size: T_cap and the 0.7 cap stay at the i9's values,
  so no L3-scaling prediction of rule A or B is reached. The i9 cannot shrink its L3: its CPU flags
  list no cache allocation support (`lscpu.txt` of the team-size run has no `cat_l3`). Running a
  cache-thrashing neighbour to occupy part of the L3 is a crude stand-in, not an emulation, and is
  not proposed as evidence.
- A mask of both CPUs of 2 cores reproduces CI's topology (c = 2: workers), which every rule
  already agrees on.

### What CI can test

Every rule plans workers there, so CI tests that the default path is unchanged (its paired bench)
and runs the plan-table and correctness tests. It cannot measure any team plan.

### What stays unvalidated

- **Every topology but the i9's:** different core counts with a different L3, multi-die and
  multi-socket parts, hybrid parts, AMD at all (beyond CI's 2-core runners, which plan workers).
- **Every level but 9 on the current build**, and in particular the inferred level-1/2
  regression.
- **Widths above 3 on physical cores** (the only k = 4 data at three tables ran on SMT siblings).
- **Rule A's constant** inside (5.24, 6.99] MB.

**How rule A behaves there by default: as today.** The proposal (§5) gates the fast branch on an
*evidence envelope*: a list of topologies whose plan a registered run has measured, today only
(10 physical cores, 20 MiB). Outside it, the default decoder takes the old plan, which is 0.2.6's
behaviour exactly, and the general rule's answer is computed and reported but not used unless the
caller opts in. The envelope is data, one line per registration, not a code path.

## 5. Phases toward production

### Phase 2: the opt-in implementation

- `Topology { phys, l3, l3_shared_by_all, hybrid }` with `Topology::detect()` (sysfs:
  `cache/index3/shared_cpu_list` against the affinity; the `cpu_core`/`cpu_atom` PMU directories
  for hybrid parts).
- `Plan::general(threads, &Topology, block_rows: Option<usize>) -> Decision`, pure, where
  `Decision { plan: Plan, mode: Mode, branch: Branch }`, `Mode` is `Plain` or `Pipelined(Levers)`,
  and `Branch` names the step taken (unread topology, below six cores, old plan, fast plan,
  outside the envelope with the plan it would have taken). This is the inspectable part.
- `StreamDecoder::defaulted` takes the decision's mode instead of comparing the plan with
  `PIPELINED_PLAN`.
- The selector: `Plan::for_threads` keeps today's `plan_with` (legacy) by default; a
  `Policy::General` on the constructors, an `ab` arm (`mibz2-general`), and `examples/plan.rs`,
  which prints this machine's decision for 1 to 2 × CPUs threads, are the opt-ins.
- `with_plan` gains a `with_decision` twin so tests and arms can force a mode as well as a plan.

### Tests (the plan cannot change output or errors)

- **Equivalence:** the general rule with today's envelope equals `plan_with` plus `defaulted`'s
  mode on a grid: P ∈ {unknown, 1–64}, L ∈ {unknown, 2–128 MiB including 20 MiB ± 1 byte and
  19.25 MiB}, N ∈ {1–128, `usize::MAX`}, shared and not, hybrid and not.
- **The general rule's table**, pinned as a snapshot: the i9 rows and the hypothetical grid above.
- **Invariants:** T × k ≤ c; pipelined if and only if k ≥ 3; never k = 2 pipelined; the threads
  used never fall as N rises; a team count of at least 1; unread inputs never panic.
- **Output and errors unchanged by the plan:** every distinct plan the rule can produce on the
  grid (2 × 2 to 13 × 2 plain, 2 × 3 to 10 × 3 pipelined) decodes, byte for byte against
  libbzip2, every level 1–9, concatenations of mixed levels, and corrupted, truncated and empty
  inputs failing with the same errors, through `with_decision`.

### The preregistered i9 experiment (for the workstation queue, PROTOCOL §9)

A recipe `bench-plan-boundaries`, frozen, reviewed and dry-run before it is queued:
- **Arms:** the default (0.2.6's policy, which is rule A on the i9); forced plans 3 × 2, 2 × 4
  pipelined, 3 × 3 pipelined, 2 × 3 pipelined; and `mibz2-general` (the opt-in rule) as an A/A on
  the default's plan.
- **Cells:** unmasked at N = 6, 8, 9, 10 and 20; masks of one CPU each of 6, 8 and 9 cores at
  N = c and N = 20. Each forced plan only where it fits the budget.
- **20 rounds, paired by round, unpinned inside the mask,** on the bench's Wikidata sample, with
  every run's plan read from its threads as in the three-team registration.
- **Primary questions:** at budgets 8 and 9 (unmasked and masked), does any alternative beat the
  default by at least 1.03 with its lower bound above 1, Bonferroni over the alternatives? Rule A
  survives if none does; rule B's 8- and 9-thread predictions are falsified if its plan's upper
  bound is below 1.03.
- **Controls:** `mibz2-general` against the default at N = 10 and 20: same plan, flagged outside
  ±2%; N = 6 and the 6-core mask: every rule's plan is 3 × 2, reported.
- **Reported, not judged:** CPU and peak RSS for every cell.
- **A second, separate entry** (its own registration, after this one is read): 0.2.6 against
  0.2.4 on the level-1, -2 and -3 recompressions at 10 and 20 threads, to measure the inferred
  regression of §1.6. If it is real, the fix to register is a scheduling one: a team whose block is
  below `MIN_COOP` lets its helpers take small blocks of their own, so a team is k decoders on
  small blocks and one on large ones. That is decided per block, so a mixed-level file needs no
  header read and no gate, which is what sank #115.

### Admission terms (proposed; RULES C6 reserves the acceptance rule to Louis)

The precedent is the 2026-11-07 decision in `decisions.md`: a plan change CI can't reach is
admitted on a frozen i9 A/B and its confirmation, at least 1.03 at the changed budget with its
lower bound above 1, and no more than 2% slower at the others. That decision applies to its
change only. Proposed for this one, in two steps:

1. **The general code, default plans unchanged.** The rule with today's envelope becomes the
   default code path. Since the equivalence test proves every plan and mode identical to 0.2.6 on
   every topology, the terms proposed are those for a measurement-neutral change: CI green, CI's
   paired bench showing nothing changed, and the `mibz2-general` A/A control of the i9
   experiment within ±2% at 10 and 20 threads. No release note claims a speed change.
2. **Widening the envelope.** A topology joins the envelope only by a registered run on it, on the
   precedent's terms (the general rule's plan against today's, run and confirmation). The i9 can
   add only masked shapes, where rule A agrees with today. **Turning the fast branch on for
   unmeasured topologies** (removing the envelope) would be a decision to ship an extrapolation;
   it is Louis's alone, and the most this entry can offer for it is the boundary experiment
   passing and the plan tables above.

## 6. Recommendation: rule A

**Why:**
- It reproduces every measured i9 choice, plain and pipelined, at every budget and mask, and CI's,
  with one new constant bounded by the i9 to an interval.
- It is the smallest rule that states the mechanism the 3 × 3 exception assumes: fewer tables,
  wider and pipelined teams, when the cache is the limit and the cores allow it.
- Its scheduling decision has a stated reason (k ≥ 3, from the pipeline model with α near 2 at
  k = 2) instead of a plan equality.
- It can ship as general code with no behaviour change, and its reach then grows only by evidence.

**Its weak points:**
- **Its L3 scaling is an extrapolation from one machine.** The level-6 result (five teams tied
  four with tables to spare) says the team count is not set by table bytes alone, and the whole
  decoder's cache behaviour is unmeasured. On a 16-core, 32 MiB part it would run 5 × 3 where
  today runs 7 × 2, and nobody knows which is faster.
- **The constant is only bounded,** and its choice inside (5.24, 6.99] MB flips 8-core 16 MiB and
  16-core 32 MiB parts between plans.
- **Width is fixed at 3** for fast teams; wider teams on spare physical cores are untested.
- **It ignores block size,** so the inferred level-1/2 regression stays until the scheduling fix.
- **Its boundaries at 8 and 9 threads are untested alternatives,** which the proposed run checks.
- **Multi-die, multi-socket and hybrid parts get the old plan,** which is itself unmeasured there.

Rule B stays as the experiment's competing hypothesis: it is the one that makes the i9 say
something new. Rule C is not recommended: it predicts nothing on paper, and on the evidence it
would choose the wrong knee on the one machine measured.

## What this entry doesn't do

No decoder change, no timing, no new measurement. Every figure is from the journals and records
cited. The inferred level regression and the rule B predictions are hypotheses until the runs of
§5 measure them.
