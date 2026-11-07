# 2026-11-07: A general rule for how the decoder spends its threads (design, not for merge until review)

**Revised after review.** The first version (`2e1322f4`) was reviewed by a stand-in Researcher
and returned REVISE, with nine objections. Every one is answered here. Where this version
disagrees, it says why. The changes are listed at the end, in "What the review changed".

**What this is.** Today, the decoder's choice of how to spend the threads a caller gives it is
one general rule plus one exception written for a single machine. This entry does four things:
- audits what is actually measured about that choice;
- proposes three general rules that could replace the exception;
- works out what each would do on machines nobody here has;
- lays out how one of them could reach production.

It changes no decoder code. Its one script,
[`2026-11-07-general-plan-policy/plan_tables.py`](2026-11-07-general-plan-policy/plan_tables.py),
does arithmetic only: it prints every plan table below from the rules as written here.

**The answer, in short.** Rule A below, the *two-budget rule*:

> If the L3 cache holds fewer full tables at about 6 MB of cache each than the old plan would
> run, and the threads asked for can fill that many teams three wide, with one physical core
> left over, run exactly that many teams of three, pipelined. Otherwise use the old plan: teams
> of two, as many as keep the tables near 0.7 of the L3.

- **On the i9-10900K** it picks what 0.2.6 picks at every thread count but one. At 9 threads it
  picks three teams of three, pipelined, where 0.2.6 picks four teams of two. That configuration
  is the same as 0.2.6's own at 10 threads (§1.4), so the difference is the rule's one direct
  test on hardware we have.
- **On CI's runners** it picks what CI runs today.
- **Elsewhere,** every difference is on a machine where nothing has been measured.

The proposal is to build the rule as an opt-in first, to reach the default only through an
*evidence envelope* (a list of measured topologies and thread ranges), and to widen that list
only on evidence, by decisions reserved to Louis.

## The terms

- **Block.** bzip2 compresses in independent blocks of up to 900,000 bytes at level 9; level *n*
  allows *n* × 100,000. The decoder decodes several blocks at once.
- **Table** (the T-vector). Undoing the Burrows–Wheeler transform on a block needs a table of 4
  bytes a row: 3.6 MB for a full level-9 block. Decoding is mostly a chain of dependent reads
  through that table, so its speed depends on whether the table is in the CPU's cache.
- **L3.** The last and largest cache, shared by the cores: 20 MiB on the i9-10900K.
- **Physical core, SMT.** The i9 has 10 cores, each running two hardware threads (SMT), so 20 CPUs
  in all. Two threads on one core share it. CI's runners show 4 CPUs on 2 cores.
- **Team, T × k.** A team is *k* threads that decode one block together, sharing one table. A plan
  of T × k runs T teams at once, so T tables are live. A team has one *leader* and *k* − 1
  *helpers*. The leader does the block's serial stages: move-to-front before the inversion, and
  reassembly, run-length decoding and the CRC check after it. The helpers build the table and walk
  it with the leader.
- **Workers.** The plan without teams: N threads, each decoding its own block alone.
- **Pipelined team.** A team whose leader overlaps its serial stages for the next and previous
  blocks with its helpers' build and walk of the current one (#247). It runs with two settings on
  (#249): *park*, where waiting helpers sleep instead of spinning, and *small*, where the leader
  keeps preparing the next block after a small one.
- **N, P, c.** N is the threads asked for. P is the physical cores in the process's CPU affinity.
  c = min(N, P).
  - **A team plan** never uses more than c team threads.
  - **A workers plan** (c < 6, or topology unread) runs all N threads, even beyond the physical
    cores. CI at 4 threads runs 4 workers on 2 cores.
  - **Every plan** also runs a scanner thread and the caller's reader thread.
- **`MIN_COOP`.** Blocks below 262,144 rows (16 checkpoints of 16 Ki) are inverted on one core,
  even inside a team, and the helpers sit idle. Every block at levels 1 and 2 is below it.

## Today's policy, exactly (master at 4aa67e79, release 0.2.6)

`plan_with` in `src/parallel.rs` (lines 1184–1201):
1. Topology unread (anything but Linux), or c < 6: N workers.
2. **Exactly 10 physical cores, an L3 read from cpu0 as exactly 20 MiB, and at least 10 threads
   asked for: 3 × 3.**
3. Otherwise teams of 2: T = min(⌊c / 2⌋, round(0.7 × L3 / 3.6 MB)), with 4 in place of the second
   term when the L3 is unread.

`StreamDecoder::defaulted` (lines 133–140) then runs the teams pipelined, with park and small,
**only when the plan is exactly 3 × 3**. Every other plan runs plain.

So two decisions are tied to one machine's identity:
- **the allocation:** workers or teams, how many teams, how wide;
- **the scheduling:** plain or pipelined.

A 12-core CPU with a 20 MiB L3 gets the old plan, though nothing about the mechanism behind 3 × 3
depends on there being exactly 10 cores. Conversely, some machines that are not the i9 match the
exception today; see "A question for the live release" below.

## 1. What is measured

Every figure here is from the i9-10900K, on the bench's Wikidata sample (11.85 MB of bzip2 at
level 9, 209.7 MB decoded), unless the row says otherwise. The status column reads:
- **M**: measured;
- **I**: inferred from measurements;
- **U**: unknown.

"Registered" means the comparison and its pass rule were fixed before the run. "Exploratory"
means they were not.

### 1.1 Per-team rate against team count

From the team-size grid (`2026-11-06-team-size.md`, 20 rounds a cell, exploratory). The per-table
rate is total MB/s divided by planned teams. The tables' share of the L3 is T × 3.6 MB / 20.97 MB.

| teams T | tables / L3 | plans measured | total MB/s | per table MB/s | status |
|--:|--:|---|--:|--:|---|
| 2 | 0.34 | 2 × 3 @ 8, 2 × 4 @ 8, 2 × 4 @ 10 | 1,116–1,166 | 558–583 | M |
| 3 | 0.52 | 3 × 3 @ 10, 3 × 4 @ 12 | 1,382–1,399 | 461–466 | M |
| 4 | 0.69 | 4 × 2 @ 8, 4 × 3 @ 12, 4 × 4 @ 16 | 1,276–1,300 | 319–325 | M |
| 5 | 0.86 | 5 × 2 @ 10, 5 × 3 @ 16 | 1,147–1,152 | 229–230 | M |
| 6 | 1.03 | 6 × 2 @ 12 | 1,051 | 175 | M |
| 8 | 1.37 | 8 × 2 @ 16 | 882 | 110 | M |
| 3 | 0.52 | **3 × 2 at any budget, current build** | | | **U** |
| 3 | 0.52 | 3 × 2 @ 6, 29 October build, first run | 1,155–1,189 | 385–396 | M (older build) |
| 3 | 0.52 | 3 × 2 @ 6, 29 October release-candidate run | 1,071 | 357 | M (older build) |

- **Where throughput peaks (M):** total throughput peaks at three tables. The registered A/B then
  confirmed 3 × 3 at 10 threads against 4 × 2: 1.089 (1.075–1.103) and 1.088 (1.076–1.101), with
  11% less CPU and 19% less peak memory (`2026-11-06-three-team-policy.md`).
- **On the 29 October build there was no such gap.** 3 × 3 at 10 threads gave 1,234 MB/s against
  1,223–1,252 for 4 × 2 (`2026-10-29-cooperative-decoder.md`, M). The three-table advantage
  appeared between builds, so it is a property of this build as much as of the machine.
- **Why it peaks there (I):** the leading reading is that fewer tables compete for the L3.
  Synchronisation and core-sharing are not ruled out (`2026-11-07-knee-mechanism.md`).
- **The width a team needs at three tables (U on the current build).** The older build's two
  3 × 2 cells, both at 6 threads, disagree with each other:
  - the first run put 3 × 2 at about 0.94–0.97 of 4 × 2 at 8;
  - the release-candidate run put 3 × 2 at 6 above 4 × 2 at 20: 1,071 against 1,050 MB/s,
    "within noise".

### 1.2 L3 hit share against footprint

From the compact-Ψ grid's counters, as summarised in the knee entry. That harness pins one core a
table and runs the build and walk only, not the whole decoder.

| live table MB | 2.4 | 9.6 | 14.4 | 21.6 | 28.8 | 36.0 | status |
|---|--:|--:|--:|--:|--:|--:|---|
| L3 hit share of the walk's loads | 0.987 | 0.961 | 0.929 | 0.794 | 0.640 | 0.539 | M (inversion-only harness) |

- **A smaller table moves the knee:** a 2.7 MB table moved that harness's knee from 4 tables to 6
  (M, inversion only).
- **The whole decoder's hit share, at any plan: U.** perf could not count in the team-size run.
- **How much of the L3 the other per-team buffers take: U.** Their capacities are known (about
  15 MB of peak memory a team); how much of them is hot at once is not.
- **In the workers decoder** (before teams), the best worker count moved inversely with block size
  from level 9 to level 6 (`2026-10-28-workstation-scaling.md` §9, M). Both knees sit at
  4 × 3.6 MB = 6 × 2.4 MB = 14.4 MB of tables, about 0.7 of the L3. That is where today's 0.7
  comes from.
- **In the team decoder it did not.** At level 6, five teams of two (12 MB of tables) tied four:
  1.006 (0.990–1.021), for 21% more CPU (`origin/research/level-policy`, M). So the team count is
  not set by table bytes alone. This is the strongest caution against extrapolating any L3 rule.
  It is also a result that block size can test again on the current build (§4).

### 1.3 Team width at fixed team count

| teams | k = 2 | k = 3 | k = 4 | status |
|--:|---|---|---|---|
| 4 | 1.000 (ref) | 1.017 (1.008–1.026), CPU 1.26 | 1.003 (0.993–1.014), CPU 1.52 | M, registered (both failed the 1.03 bar) |
| 4, per table | 319 | 325 | 320 | M |
| 3, per table | U (current build) | 461 | 466 | M |
| 2, per table | U | 558 | 579–583 | M |
| 5, per table | 230 | 229 | | M |

- **At four tables (M):** width beyond two buys almost nothing.
- **At three tables (M, exploratory):** k = 4 buys about 1% over k = 3. That cell ran at 12 threads
  on 10 cores, so with SMT sharing.
- **Placement (M):** pinned spread against packed matters most at k = 2. When 4 × 2's threads share
  cores, it runs 23% slower. This is from a CPU mask, not per-thread placement.

### 1.4 Plain against pipelined

| comparison | speed (95%) | CPU | status |
|---|---|---|---|
| 3 × 3 pipelined vs plain, 10 threads | 1.042 (1.016–1.068) | 9% more | M, registered |
| 3 × 4 pipelined vs plain, 12 threads | 1.032 (1.020–1.044) | 11% more | M, registered |
| 3 × 3 pipelined + park + small vs plain, 10 threads | 1.072 (1.053–1.091) | about 5% more | M, reported |
| 3 × 4 pipelined + park + small vs plain, 12 threads | 1.074 (1.058–1.090) | about 8% more | M, reported |
| **0.2.6 vs 0.2.5 (the confirmation), 10 / 11 / 12 / 16 / 20 threads** | 1.072 / 1.070 / 1.069 / 1.065 / 1.070, all lower bounds ≥ 1.045 | 1.04–1.05× | M, registered |
| 0.2.6 vs 0.2.5 at 8 and 9 threads (both 4 × 2 plain) | 1.002, 1.001 | 1.00 | M (same-plan controls) |
| **3 × 3 pipelined at 9 threads asked for, unmasked** | about 1.16 over 4 × 2 (1.088 × 1.072) | | **I, strong: the same configuration as the 10-thread default** |
| 3 × 3 pipelined under a 9-core mask (no physical core left over) | | | **U** |
| **2 × 4 pipelined (no levers) vs 2 × 4 plain, 10 threads asked for, 8 used** | medians 0.159 s vs 0.181 s (about 1.14) | U | **M, unregistered, no interval** |
| 2 × 2 pipelined vs plain | about 0.92 (pipelined slower) | more | indicative only: an agents' shared 4-core machine, not the i9 or CI |
| k = 2 pipelined on the i9, any T | | | **U** |
| 3 × 4 pipelined + levers vs 3 × 3 pipelined + levers | about 1.02 for about 19% more CPU | | I (from ratios in different runs) |

**Why 9 threads is not unknown.** At 9 threads asked for on the unmasked i9, forced 3 × 3 pipelined
is the 10-thread default's configuration:
- the same 9 team threads, scanner and reader;
- the same 10 physical cores;
- the same 12-block window: `ab` asks for 4 slots a thread, and ⌊36 × 3 / 9⌋ = ⌊40 × 3 / 10⌋ = 12.

Only the number the caller asked for differs. The first version marked this cell U and built a
reserved core into the rule against N. That was wrong: the old plan reserves no core (at 8 threads
it runs 4 × 2 plus the scanner and reader). Only the 9-core mask, with no physical core outside the
teams, is unknown.

**The pipeline's model** (`2026-11-07-knee-mechanism.md`, H1):

  speed = (S + P) / (max(S, αP) + H)

- **S** is the leader's serial stage.
- **P** is the parallel stage.
- **α** is how much the parallel stage slows when the leader leaves it.
- **H** is the handoff.

The measured 3 × 3 fits α ≈ 1.39–1.55, depending on the assumed H, so **α is not identified**
(`2026-11-07-team-pipeline-result.md`). With k = 2 the leader's leaving halves the parallel stage's
threads, so α is near 2. The model then predicts a loss whenever S < P (I; S ≈ 0.40 ms and
P ≈ 0.65 ms a block at 3 × 3).

### 1.5 CI's runners

- **Topology (M):** 4 CPUs on 2 physical cores, on EPYC 7763 and 9V74 runners
  (`2026-10-27-four-thread-ceiling.md`).
- **The bench never runs a team plan.** It runs 1, 2 and 4 threads, and with c ≤ 2 < 6 every plan
  there is workers. **CI's bench has never run a team plan or a pipelined team** (M, from the code
  and the workflow).
- **Placement (M):** spreading two threads over two cores, rather than packing them on one, is
  worth +40% on both EPYC models.
- **Runners:** the bench runs on `ubuntu-latest`, and no workflow uses GitHub's larger runners. The
  others are `ubuntu-22.04`, `ubuntu-24.04`, `windows-latest` and `macos-15`. macOS and Windows read
  no topology, so they always plan workers.
- **The 4-core pass on the i9** (CPUs 0–3, one of each of 4 cores): the policy is workers, 0.997
  (0.980–1.014) against plain workers (M).

### 1.6 Level (block size)

Everything in this section is from the 29–30 October builds, **before teams of three and before
the pipeline**. It is M on those builds and U on the current one.

| level | rows a block | best workers (workers decoder) | 5 × 2 vs 4 × 2 (teams decoder, 10 vs 20 threads) |
|--:|--:|---|---|
| 1 | 100 K | 6–10, tied | 1.150 (1.138–1.162) |
| 2 | 200 K | | 1.216 (1.193–1.241) |
| 3 | 300 K | 6–8 | 1.075–1.092 |
| 4 | 400 K | | 1.085 (1.064–1.105) |
| 5 | 500 K | | 1.049 (1.034–1.064) |
| 6 | 600 K | 6 | 1.006 (0.990–1.021) |
| 9 | 900 K | 4 | 0.917 (0.907–0.927) |

- **At levels 1 and 2 a team is one core.** Every block there is below `MIN_COOP`, so 4 × 2 ran as
  4 one-core decoders, and 5 × 2's gain came from a fifth decoder, not from cooperation. M, from the
  code and from `auto`@4 against `auto`@20 (1.005–1.007).
- **A rule using the level was built and failed its shipping gates** (#115, closed unmerged). It met
  its gates at levels 1 to 3, but its first run fell short on a level-6 control. Its revision ran a
  level-3-then-level-9 concatenation at 0.945 (0.930–0.959). The plan is chosen once, before the
  first block, and a file's later streams can have bigger blocks.
- **Inferred, levels 1 and 2 only: 0.2.5 and 0.2.6 decode such files with three one-core decoders at
  10 or more threads, where 0.2.4 used four.**
  - **The inference:** on the 29 October build, `auto`@6 (3 teams, so 3 one-core decoders at these
    levels) ran at 0.76–0.83 of `auto`@20 (4 teams, 4 decoders).
  - **The reading:** 0.2.5 and later may be about 17–24% slower than 0.2.4 on level-1 and level-2
    files at 10 or more threads.
  - **Confidence: moderate to high.** The decoder count is certain from the code, and the scale is
    from one older build, not measured on the current one.
  - **Levels 3–5 are not covered.** Their blocks reach `MIN_COOP`, so the helpers do work there,
    and the 6-thread figures there compare different team plans, not decoder counts.
  - **The reach is small.** bzip2 and the dumps this project serves use level 9, but the slowdown
    is one no run has looked for.

### 1.7 Release scaling (the records in `bench/release-scaling/`)

Medians of 10 paired rounds, in MB/s. The site's generated table is built from these records.

| threads | 1 | 2 | 4 | 8 | 10 | 12 | 16 | 20 |
|---|--:|--:|--:|--:|--:|--:|--:|--:|
| 0.2.4 (4 × 2 from 8 up) | 397 | 701 | 1,021 | 1,293 | 1,272 | 1,269 | 1,267 | 1,261 |
| 0.2.5 (3 × 3 from 10 up) | 403 | 700 | 1,016 | 1,302 | 1,415 | 1,368 | 1,371 | 1,362 |

No run covers 5, 6, 7 or 9 threads, and the 0.2.6 record is still in the workstation queue (U).

### 1.8 What a general rule must reproduce, and what it may change

**Measured choices** (a rule that departs from one must justify it):
1. c < 6: workers (the 4-core pass, and CI).
2. c = 10 or more on the i9: three teams of three, pipelined with park and small. This rests on two
   registered A/Bs and the 0.2.6 confirmation.
3. Never k = 2 pipelined: there is no evidence for it, and the model predicts a loss.

**Measured only as the same plan against itself:**
- 4 × 2 at 8 and 9 threads, and 3 × 2 at 6 and 7. The 8- and 9-thread rows of the registrations
  (1.002, 1.001) are same-plan equivalence controls, not evidence that 4 × 2 is the best plan there.
- A rule may change these on evidence. 3 × 3 at 9 threads is already supported by the identity in
  §1.4.

**Unknown:**
- whether 3 × 2 at 8, or 2 × 4 pipelined at 8, would beat 4 × 2;
- whether anything here holds on another CPU;
- anything about levels on the current build.

## 2. Three candidate rules

Each rule is a pure function of its inputs, so its answer can be printed for any machine. All three
keep step 1 of today's policy (unread topology or c < 6: N workers), because nothing measures teams
below six cores. All three use the same budget conventions, so the experiment can tell them apart:
- **Width-2 plans** fit in c = min(N, P) team threads, as today, with no core left over.
- **Plans of width 3 or more** fit in N team threads and in P − 1 physical cores, one core being
  left for the scanner and the reader.

The second convention is counted against P, not N (review point 1). Every measured 3 × 3 had at
least one physical core outside its teams. Whether that core is needed is open (the 9-core mask,
§4); the rules assume it rather than test it.

### Rule A: the two-budget rule

**Inputs:**
- N, the threads asked for;
- P, the physical cores in the affinity (or unknown);
- L, the L3 bytes (or unknown);
- whether that L3 is shared by every usable core, and whether the cores are of one kind (new
  inputs; see the edge cases);
- block size from the first stream header: **reported but not used.** §1.6: the team decoder's
  best team count did not follow table bytes, and the one level-aware rule that was built failed
  its shipping gates.

**Constants:**
- inherited: 6 cores (the threshold for teams), 0.7, and F = 3.6 MB (a full table, at every level);
- new: **6 MB of L3 per fast team**, and width 3.

**The rule:**
1. P unknown, or c < 6: N workers.
2. **The old plan:** T_b = max(1, min(⌊c / 2⌋, round(0.7 L / F))) teams of 2, plain. The second term
   is 4 when L is unknown.
3. **The fast plan:** if L is known, shared by every usable core, and the cores are of one kind, let
   T_cap = ⌊L / 6 MB⌋. **Use T_cap × 3, pipelined with park and small, if all of these hold:**
   - T_cap ≥ 2;
   - T_b > T_cap: the old plan puts more tables in the L3 than the fast budget allows;
   - 3 T_cap ≤ N and 3 T_cap ≤ P − 1: the budget fills every fast team, with a core left over.
4. Otherwise the old plan.

**Scheduling:** pipelined if and only if k ≥ 3. There is no separate switch: the width decides.

**Why the fast plan must fill its whole budget** (review point 3). The first version allowed fewer
fast teams than T_cap, provided 3 T_f ≥ 2 T_b. Whether that clause held depended on c mod 6, so
plans flipped as threads rose. For example, at 16 cores and 32 MiB:

| threads | first version | modelled MB/s |
|--:|---|--:|
| 12 | 6 × 2 | |
| 13 | 4 × 3 pipelined | |
| 14 | 7 × 2 | about 10% lower than at 13 |
| 16 | 5 × 3 pipelined | |

Throughput here is modelled with rule B's per-table curve. Requiring the full T_cap removes every
such flip.
- **The scan.** `plan_tables.py` scans 8,128 topologies: 1–64 cores, L3 2–128 MiB in 1 MiB steps,
  threads 1 to 2 × cores.
  - **Threads used:** in the team region, the revised rule never uses fewer threads as N rises.
  - **Modelled throughput falls of more than 2% that involve a width-3 plan:** 1,519 in the first
    version, 0 now.
- **The modelled falls that remain** (6,953 in the scan) are all the old plan's own: it adds a
  table of width 2 that the curve scores lower. Today's policy has 7,057 of them. They depend on the
  per-table curve, which rule B assumes and the i9 experiment tests at 8 threads. They are inherited
  from today, not introduced.
- **The step from workers to teams** (c = 5 to c = 6) can also use fewer threads, as today: 649
  steps in the scan, such as 6-core parts with 8–12 MiB of L3 going from 5 workers to 2 × 2. The
  review's own scan counted 531 of 7,375. This step is exempt from the invariant, because the
  equivalence with today requires it.
- **What remains open: parts that can't fill the fast budget.** Their cores cannot fill T_cap teams
  of three, so rule A keeps the old plan, even where that plan holds tables at about 0.7 of the L3,
  the share at which the i9's 4 × 2 lost to 3 × 3. For example, 16 cores with 36 MiB: T_cap = 6
  needs 19 cores, so it runs 7 × 2, with tables at 0.67 of the L3. With 37 MiB it runs 8 × 2.
  - **Rule A makes no claim there;** it keeps today's plan.
  - **A partial fast plan** (fewer teams of three than T_cap) is what caused the flips, so it is
    left out until something measures one.

**The constant, and what the i9 can and can't fix** (review point 4).
- **The interval.** The i9 pins T_cap = 3, which holds for any figure in (5.24, 6.99] MB of L3 per
  fast team: 20.97 MB / 4 to 20.97 MB / 3. 6 MB is a round number inside that interval, not a fit.
- **Conditional on the rule's form:** a different rule would read the same measurement differently.
- **Conditional on this build:** on the 29 October build, 3 × 3 tied 4 × 2 (§1.1).
- **Cache and cores can't be separated on the i9.** At c = 10, three teams of three is also the most
  a 10-core budget allows with a core left over (⌊9 / 3⌋ = 3). So the i9 cannot tell a cache limit
  from a core or synchronisation limit; both give T = 3 there.

Where the constant sits inside its interval changes the answer on other caches:

| L3 per fast team | 10 cores, 20 MiB | 12 c, 20 MiB | 8 c, 16 MiB | 16 c, 32 MiB | 32 c, 64 MiB |
|---|---|---|---|---|---|
| 5.30 MB | 3 × 3 pipelined | 3 × 3 pipelined | 3 × 2 | 7 × 2 | 13 × 2 |
| 6.00 MB | 3 × 3 pipelined | 3 × 3 pipelined | 2 × 3 pipelined | 5 × 3 pipelined | 13 × 2 |
| 6.99 MB | 3 × 3 pipelined | 3 × 3 pipelined | 2 × 3 pipelined | 4 × 3 pipelined | 9 × 3 pipelined |

**The free choices, counted.** The i9 constrains these only jointly, through a few plan cells.

| choice | new or inherited | how it is set |
|---|---|---|
| teams only from 6 cores | inherited | the 4-core pass |
| 0.7 of the L3 for the old plan | inherited | the workers decoder's knee |
| F = 3.6 MB at every level | a choice | the level-6 tie |
| 6 MB per fast team | new | bounded to an interval |
| width 3 | new | the one measured fast width |
| one core left over, counted against P | new | assumed, untested |
| fill the whole fast budget, with T_cap ≥ 2 | new | monotonicity, not evidence |
| pipelined iff k ≥ 3 | new | the model and the k ≥ 3 data |
| the sharing and core-kind guards | new | caution, not evidence |

That makes 9 choices, of which 6 are new and 4 of those are not set by any measurement. The rule
is simple to state, but it is not a fit.

**At the edges:**
- **Unknown topology** (macOS, Windows, a container hiding sysfs): workers, as today.
- **Requests above the physical cores:** team plans are capped at c. Workers plans use all N
  threads, as today.
- **L3 sharing** (review point 9).
  - **How it is read:** from a CPU in the affinity, not cpu0. The rule requires the whole affinity
    to sit inside that cache's `shared_cpu_list`.
  - **When the check fails:** multi-die AMD parts, two sockets, or an affinity that spans caches
    all get the old plan.
  - **VMs:** a hypervisor may misreport `shared_cpu_list`, showing one flat L3 or one per vCPU.
    Cloud guests are therefore unvalidated in either direction.
- **Cores of more than one kind** (review points 2 and 9): the old plan.
  - **Detection on Intel:** Intel hybrid parts expose `/sys/devices/cpu_core` and
    `/sys/devices/cpu_atom`, but that is Intel only.
  - **Detection elsewhere:**
    - on ARM, the per-CPU `cpu_capacity` differs between core kinds;
    - in general, `cpufreq/cpuinfo_max_freq` differs across the affinity. That needs a wide
      threshold, about 20%, because Turbo Boost Max can give the i9's favoured cores a higher
      maximum (5.3 against about 5.1 GHz) without them being a different kind. Whether it does on
      this i9's sysfs is unverified, and should be recorded in the next run's `env.json`.
  - **No positive proof required:** with no such signal, the cores count as one kind.
- **Tiny streams** (fewer blocks than teams): the plan only spawns threads, and a team's helpers
  start at its first block above `MIN_COOP`. Unchanged from today.
- **Small blocks and levels 1–9:** no change from today at any level. At levels 1–2 that keeps the
  inferred slowdown of §1.6, three one-core decoders on the i9. Fixing it is a scheduling change,
  not a plan change (§5).

**Where it differs from today:**
- **On the i9:** only at N = 9, unmasked: 3 × 3 pipelined against 4 × 2.
- **On CI:** nowhere.
- **Elsewhere:** on any topology where the old plan holds more tables than the 6 MB budget allows
  and the cores can fill the fast teams with one left over (the table in §3).
- **On hybrid and multi-socket parts that match today's exception:** the guards give the old plan
  where 0.2.6 gives 3 × 3. That differs from 0.2.6 deliberately, and only once the default uses the
  rule (§5, step 1 keeps 0.2.6's choice there).

### Rule B: a throughput model fitted to the per-team rates

**Inputs:** as rule A, with the same budget conventions (above).

**The rule:** choose the T × k (k ∈ {2, 3, 4}) that maximises

  Q = T × r(T F / L) × p(k)

- **r(x)** is the per-table rate of §1.1, as a function of the tables' share of the L3. It is
  interpolated linearly through the six measured points and held flat below the first, so it
  claims no extrapolated gain.
- **p(k)** is the pipeline's gain: 1 at k = 2 (no evidence), 1.072 at k ≥ 3.
- **Width** otherwise counts for nothing (§1.3).
- **Ties:** plans within 2% of the best tie, and the tie goes to the fewest threads.
- **Scheduling:** pipelined if and only if k ≥ 3.
- **Unknown L3:** rule A's answer.

**At the edges:** as rule A for unknown topology, requests above the cores, sharing, core kinds and
level. It extrapolates r(x) to any cache by the share x alone. So it assumes another CPU's per-table
rate falls with the L3 share exactly as the i9's does, at the i9's memory latency and clock.

**Where it differs from today:**
- **On the i9:** at 8 threads (3 × 2 for 4 × 2), and at 9 threads (3 × 3 pipelined, as rule A).
  Under 8- and 9-core masks, at any thread count, it picks 3 × 2.
- **Elsewhere:** on most hypothetical parts, including one-team plans on 8 MiB caches (1 × 3
  pipelined, which nothing supports).
- **Its 8-thread prediction** is about 1.08 for 3 × 2 over 4 × 2. The evidence on it is **mixed, not
  contradictory** (§1.1): one older-build run put 3 × 2 at 6 below 4 × 2 at 8, and another put it
  level with 4 × 2 at 20.

### Rule C: calibrate at startup, or once offline

**The startup form:**
- **The probe:** the first decoder in a process builds T synthetic full tables for T = 2, 3, … and
  walks them concurrently for a few milliseconds each. It takes the T where the aggregate walk rate
  stops rising as T_cap.
- **The rest:** width and scheduling then follow rule A's steps 2–4.
- **Fallback:** rule A, when the probe can't run (c < 6, topology unread) or when two probes
  disagree.

**What it costs:**
- on the order of 10–30 ms, against a 150 ms decode of the bench sample, and more than the whole
  decode of a small file;
- noise from whatever else the machine runs;
- two runs of the same program could choose different plans.

**An offline form** (review point 5) avoids those costs. A one-off command (`examples/plan.rs
--calibrate`, say) times the whole decoder at a few plans on the machine, and writes a small
per-machine file that later decoders read. This is the "measure your own machine" route, and the
natural way for a user's machine to join rule A's evidence envelope. It needs a file location and a
staleness rule, and it trusts a measurement that no registration covers.

**What it predicts:** nothing on paper. On the i9, the inversion-only harness of §1.2 put the knee
at four tables, not three. So a walk-only probe would likely choose 4 × 2 at 10 threads, undoing
the measured 3 × 3 gain (I). The offline form, timing the whole decoder, would not have that
problem, but it is a calibration procedure, not a rule.

## 3. What each rule predicts

From `plan_tables.py`. "Pipelined" (p) always means with park and small.

**The i9-10900K, every budget** (N threads asked for; 10 cores, 20 MiB L3):

| threads | today (0.2.6) | rule A | rule B |
|--:|---|---|---|
| 4, 5 | N workers | N workers | N workers |
| 6, 7 | 3 × 2 | 3 × 2 | 3 × 2 |
| 8 | 4 × 2 | 4 × 2 | **3 × 2** |
| 9 | 4 × 2 | **3 × 3 pipelined** | **3 × 3 pipelined** |
| 10–20 | 3 × 3 pipelined | 3 × 3 pipelined | 3 × 3 pipelined |

**The i9 under CPU masks.** The mask has c cores, the L3 stays 20 MiB, and the threads asked for are
the CPUs in the mask.

| cores | mask | threads | today | rule A | rule B |
|--:|---|--:|---|---|---|
| 4 | one CPU a core / both | 4 / 8 | workers | workers | workers |
| 6 | one CPU a core / both | 6 / 12 | 3 × 2 | 3 × 2 | 3 × 2 |
| 8 | one CPU a core / both | 8 / 16 | 4 × 2 | 4 × 2 | 3 × 2 |
| 9 | one CPU a core / both | 9 / 18 | 4 × 2 | 4 × 2 | 3 × 2 |

**CI's runners** (4 CPUs, 2 cores): 1, 2 and 4 workers at 1, 2 and 4 threads, under every rule.
macOS and Windows: workers.

**Hypothetical parts, all CPUs asked for.** ← marks a change from today.

| cores | L3 | today | rule A | rule B |
|--:|---|---|---|---|
| 6 | 8 MiB | 2 × 2 | 2 × 2 | 1 × 3 p ← |
| 6 | 12–64 MiB | 2 × 2 or 3 × 2 | as today | as today |
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
| 32 | 64 MiB | 13 × 2 | 13 × 2 | 10 × 3 p ← |

The rows for 8 and 12 MiB caches at 10–32 cores repeat the 8-core rows: 2 × 2 today and under rule
A, 1 × 3 p and 2 × 3 p under rule B. The script prints all 36, and the monotonicity scan.

All of rule A's changes are of one kind. On a part where the old plan holds tables beyond the fast
budget and the cores can fill the fast teams, it trades about a quarter of the tables for width and
pipelining, as 3 × 3 did on the i9. Every one of those rows, apart from the i9's, is unmeasured.

## 4. Falsifiable predictions, and the hardware that can test them

### What the i9 can test: the boundaries

| cell | rule A | rule B | alternative | A predicts | B predicts |
|---|---|---|---|---|---|
| 9 threads, unmasked | 3 × 3 p | 3 × 3 p | 4 × 2 (today) | 3 × 3 p faster, about 1.16 | same |
| 8 threads, unmasked | 4 × 2 | 3 × 2 | | 3 × 2 not faster by 3% | 3 × 2 about 1.08 faster |
| 8 threads, unmasked | 4 × 2 | 3 × 2 | 2 × 4 p | not faster than 4 × 2 by 3% | slower than 3 × 2 |
| 8-core mask, 8 threads | 4 × 2 | 3 × 2 | | as at 8 unmasked | as at 8 unmasked |
| 8-core sibling mask, 16 threads | 4 × 2 | 3 × 2 | | as at 8 unmasked | as at 8 unmasked |
| 9-core mask, 9 threads | 4 × 2 | 3 × 2 | 3 × 3 p (no core left over) | no claim: the assumption under test | no claim |

- **The 2 × 4 pipelined arm** comes from the unregistered cell of §1.4 (0.159 s against 0.181 s).
  If it beats 4 × 2 at 8, the rule is missing a budget for teams of four at small budgets.
- **The 9-core mask cell** tests the core-left-over assumption: if 3 × 3 p beats 4 × 2 there, the
  assumption is unnecessary.

### What CPU masks test, and what they don't

`taskset` makes the decoder see only the cores in its mask, because `physical_cores` reads the
affinity.
- **One CPU of each of c cores** emulates a c-core part without SMT.
- **Both CPUs of c cores** emulates a c-core part with SMT: twice the CPUs, the same physical
  count, the scanner and reader on siblings.

Both keep **the i9's 20 MiB L3, its memory system and its clock**.

- **What masks test:** the core-count branches, with no physical core outside the mask for the
  scanner, the reader and the OS, as a real 8- or 9-core part would have.
- **What they don't test:** anything about cache size.
  - **No L3 scaling is reached.** T_cap and the 0.7 cap stay at the i9's values, so no L3-scaling
    prediction of rule A or B can be tested.
  - **The L3 can't be shrunk.** The i9's CPU flags list no cache allocation support (the team-size
    run's `lscpu.txt` has no `cat_l3`).
  - **A cache-thrashing neighbour** occupying part of the L3 is a crude stand-in, not an emulation,
    and is not proposed as evidence.
- **Turbo.** The i9's clock depends on how many cores are busy: about 4.9 GHz all-core, up to 5.3 GHz
  on its favoured cores with few busy. A masked run can therefore clock higher than a real c-core
  part under the same load.
  - **Fair within a cell:** both arms of a comparison share the mask, so the paired ratio is fair.
  - **Not comparable across masks:** absolute rates across masks and against unmasked cells are not.
  - **What to record:** the recipe should sample each run's CPU frequency (harness mechanics, which
    RULES C6 delegates).
  - **What not to change:** turbo settings are the workstation's, and no one but Louis changes them.

### What block size can test: the one i9 test of the bytes-per-L3 premise

Rule A's extrapolation to other caches rests on one premise: what limits the team count is table
bytes against the L3. The i9's L3 can't change, but the table size can, by level. A level-6 block
has a 2.4 MB table, so four of them hold 9.6 MB, less than the 10.8 MB of three full tables.
- **If the premise holds,** plain 3 × 3 against 4 × 2 at 10 threads should fall from about 1.08 at
  level 9 to about 1.00 or below at level 6. Level 7, at 11.2 MB for four tables, should sit near
  the boundary.
- **If cores or synchronisation are the limit,** the ratio should stay near level 9's.
- **Confounds:** smaller blocks also mean more blocks, more per-block overhead and a different share
  below `MIN_COOP` (none at levels 6 and 7). The October level-6 tie (five teams against four) hints
  against the premise already, though that run had five teams on all ten cores.

This is a separate diagnostic registration (below), and the closest the i9 can come to the premise.
**Nothing the i9 can run is evidence for the L3 extrapolation itself:** no cell changes the L3.

### What CI can test

Every rule plans workers on CI. So CI tests that the default path is unchanged (its paired bench),
and runs the plan-table and correctness tests. It cannot measure any team plan.

### What stays unvalidated

- **Every topology but the i9's:**
  - different core counts with a different L3;
  - multi-die and multi-socket parts;
  - hybrid parts;
  - virtual machines;
  - AMD at all, beyond CI's 2-core runners, which plan workers.
- **Every level but 9 on the current build,** and in particular the inferred level-1/2 slowdown.
- **Widths above 3 on physical cores:** the only k = 4 data at three tables ran on SMT siblings.
- **Rule A's constant,** inside (5.24, 6.99] MB.
- **The core-left-over assumption,** until the 9-core mask cell runs.

**How rule A behaves there by default: as today.** The proposal (§5) gates the default on an
*evidence envelope*:
- **Entries:** a list of (topology, thread range) entries whose plan a registered run has measured.
  Today that is only 10 physical cores with 20 MiB, at 10 threads or more.
- **Outside the envelope:** the default decoder takes 0.2.6's plan, exactly. The general rule's
  answer is computed and reported, but used only if the caller opts in.
- **The envelope is data,** one line per registration, not a code path.

## 5. Phases toward production

### Phase 2: the opt-in implementation

- **`Topology`:** `Topology { phys, l3, l3_shared_by_all, one_kind }`, with `Topology::detect()`.
  It reads sysfs: the L3 of a CPU in the affinity, its `shared_cpu_list` against the affinity, and
  the core-kind signals of §2.
- **`Plan::general`:** `Plan::general(threads, &Topology, block_rows: Option<usize>) -> Decision`,
  a pure function.
  - `Decision { plan: Plan, mode: Mode, branch: Branch }`.
  - `Mode` is `Plain` or `Pipelined(Levers)`.
  - `Branch` names the step taken: unread topology, below six cores, old plan, fast plan, or outside
    the envelope (with the plan the rule would have taken). This is the inspectable part.
- **`StreamDecoder::defaulted`** takes the decision's mode instead of comparing the plan with
  `PIPELINED_PLAN`.
- **The selector:** `Plan::for_threads` keeps today's `plan_with`, unchanged, by default. The
  opt-ins are:
  - a `Policy::General` on the constructors;
  - an `ab` arm, `mibz2-general`;
  - `examples/plan.rs`, which prints this machine's decision for 1 to 2 × CPUs threads.
- **`with_plan`** gains a `with_decision` twin, so tests and arms can force a mode as well as a
  plan.

**A cheaper first step.** The Decision type, the detection, the envelope and the tests are a few
hundred lines. They buy inspectability now, and default changes later. Until the i9 boundary run
says the rule predicts something worth shipping (N = 9 being the first candidate), the general
branch would stay off by default (RULES C3).
- **The cheaper alternative:** implement rule A only as a pure, doc-hidden function, with
  `examples/plan.rs` and the `ab` arm, and leave the default path's code untouched.
- **Recommended order:** the cheaper alternative first; the full selector only if the boundary run
  supports a default change.

### Tests (the plan cannot change output or errors)

- **Equivalence.** The default with today's envelope equals `plan_with` plus `defaulted`'s mode on
  the tested grid:
  - P ∈ {unknown, 1–64};
  - L ∈ {unknown, 2–128 MiB}, including 20 MiB ± 1 byte and 19.25 MiB;
  - N ∈ {1–128, `usize::MAX`};
  - shared and not, one kind and not.

  For the envelope check, the default reads the L3 as 0.2.6 does, from cpu0, before any guard, so
  it keeps 0.2.6's choice on the hybrid and multi-socket parts of the note below. The claim is
  "identical on the tested grid", not on every machine.
- **The general rule's table,** pinned as a snapshot: the i9 rows and the hypothetical grid above.
- **Invariants:**
  - T × k ≤ c;
  - pipelined if and only if k ≥ 3, so never k = 2 pipelined;
  - at least one team;
  - unread inputs never panic;
  - the threads used never fall as N rises, within the team region (c ≥ 6 at both N and N + 1).
    The step from workers to teams is exempt, as today's policy has it.
- **Output and errors unchanged by the plan.** Every distinct plan the rule can produce on the grid
  (2 × 2 to 13 × 2 plain, 2 × 3 to 10 × 3 pipelined) is run through `with_decision`:
  - every level 1–9 decodes byte for byte against libbzip2, as do concatenations of mixed levels;
  - corrupted, truncated and empty inputs fail with the same errors.

### The preregistered i9 experiments (for the workstation queue, PROTOCOL §9)

RULES C1 allows a diagnostic question registered with an information objective, but a diagnostic
can never qualify a production change. So the qualifying runs and the diagnostic runs are separate
registrations.

**Registration 1, diagnostic: `bench-plan-boundaries`** (information objective: which rule's
boundaries hold).
- **Cells and arms:** the boundary table of §4, each plan forced through `with_decision`.
  - unmasked: N = 8 and N = 9;
  - the 8-core mask, one CPU a core, N = 8;
  - the 8-core mask, both CPUs a core, N = 16;
  - the 9-core mask, one CPU a core, N = 9.
- **Dropped:** the first version's masked N = 20 cells. They give the same plan, window and threads
  as N = c, so they would only repeat a cell.
- **Controls:** N = 6 and the 6-core mask, where every rule picks 3 × 2. These are reported, not
  judged.
- **The method:** 20 rounds, paired by round, unpinned inside the mask, on the bench's Wikidata
  sample, with every run's plan read from its threads and each run's CPU frequency sampled.
- **One multiplicity family for both rules:** the 7 comparisons against 4 × 2.
  - unmasked 8: 3 × 2, and 2 × 4 p;
  - unmasked 9: 3 × 3 p;
  - the masked 8s: 3 × 2 at each;
  - the 9-core mask: 3 × 2, and 3 × 3 p.
- **The family's rule:**
  - **Bonferroni:** two-sided 95% overall, so each interval at 1 − 0.05/14 a tail, t₁₉ ≈ 3.1.
  - **"Faster":** a point estimate of at least 1.03 and a lower bound above 1.
  - **"Slower":** the upper bound below 1.
  - **"Unresolved":** anything else.
- **The minimum detectable effect:**
  - earlier runs' per-round log-ratio standard deviations were about 0.027–0.043 (from their
    intervals);
  - at 80% power, a true difference of about (3.1 + 0.86) × sd / √20 = 2.4–3.8% is detectable;
  - together with the 1.03 point bar, effects below about 3–4.5% will usually read "unresolved".
- **How each rule is read:**
  - **Rule A** survives if no alternative is "faster" at its 4 × 2 cells, and 3 × 3 p is "faster"
    at unmasked 9.
  - **Rule B's 8-thread claim is falsified** if 3 × 2 is "slower" or its point estimate is below
    1.03 at all three 8-thread cells.
- **What it does not show:** anything about the L3 extrapolation.

**Registration 2, diagnostic: `bench-plan-bytes`** (information objective: does the table's size
move the three-team advantage?).
- **Cells:** N = 10 on the level-6, -7 and -9 recompressions of the bench content.
- **Arms:** plain 3 × 3, 3 × 3 pipelined and 4 × 2.
- **The method:** 20 rounds, paired, with the same family treatment over its 6 comparisons.
- **Predictions:** as in §4.

**Registration 3, qualifying: the N = 9 entry.** This runs only if registration 1 shows 3 × 3 p
"faster" at unmasked 9.
- **The comparison:** the general rule's default against 0.2.6 at 8, 9, 10, 12 and 20 threads,
  forced to nothing.
- **The precedent's terms:** at least 1.03 at 9 with its lower bound above 1, and no more than 2%
  slower elsewhere, run plus confirmation.
- **If it passes,** the envelope entry widens to 9 threads or more.

**Registration 4, qualifying: the step-1 A/A** (below). It is a separate registration, because
step 1 changes the default path's code.

**Registration 5, diagnostic: levels 1–3** (§1.6).
- **Arms:** 0.2.6 against 0.2.4, plus 0.2.6 forced to 4 × 2.
- **Corpora:** the level-1, -2 and -3 recompressions.
- **Threads:** 10 and 20.
- **What the forced arm separates:** the decoder count from everything else that changed between
  the releases.
- **If the slowdown is real, the fix to register is a scheduling one.** A team whose block is below
  `MIN_COOP` would let its helpers take small blocks of their own, so it decodes as k decoders on
  small blocks and as one on large ones. Because that is decided per block, a mixed-level file
  needs no header read and no gate, which is what sank #115. It needs:
  - **its own buffers for each helper,** a `Front` (the move-to-front stage's buffers) and a `Solo`
    (the one-core kernel's buffers): about 15 MB more peak memory at 3 × 3;
  - **a window that scales with decoders, not teams,** or the extra decoders starve;
  - **a head-of-line control:** a level-3-then-9 concatenation, the case #115 failed on.

  It does not help levels 3–5, whose blocks already cooperate.

### Admission terms (proposed; RULES C6 reserves the acceptance rule to Louis)

**The precedent** is the 2026-11-07 decision in `decisions.md`. A plan change CI can't reach is
admitted on a frozen i9 A/B and its confirmation:
- at least 1.03 at the changed budget, with its lower bound above 1;
- no more than 2% slower at the others.

That decision applies to its change only. Proposed for this one, in two steps:

1. **The general code, default plans unchanged.**
   - **What changes:** the rule with today's envelope becomes the default code path, with plans and
     modes identical to 0.2.6 on the tested grid.
   - **It needs Louis's explicit decision.** It improves no end-to-end figure, so it cannot meet the
     usual merge rule, and the merge rule is his under C6.
   - **The proposed evidence:**
     - CI green, and CI's paired bench showing nothing changed;
     - the equivalence test;
     - a workstation A/A: `mibz2-general` against 0.2.6 at 10 and 20 threads, on the same plan. Louis
       runs it (C5), as registration 4.
   - **The A/A's criterion,** proposed: flagged if the 95% interval lies wholly outside ±2%, as the
     precedent's controls are.
     - **False alarms from noise:** negligible. With the earlier controls' half-widths of about
       0.02, a true ratio of 1 flags only with an estimate beyond about 1.04, roughly 4 standard
       errors.
     - **A build or layout offset** of about 1.5% has been seen (0.986 at 8 threads, three-team
       first run) without flagging.
     - **It is a screen, not an equivalence test:** it misses regressions smaller than about 4%.
       Earlier same-plan controls of 1.008 [0.992, 1.026] and 1.007 [0.987, 1.027] would both pass
       it. They would fail a two-one-sided equivalence test at ±2% with 20 rounds, whose 90% bounds
       reach about 1.024.
     - **A stronger claim** needs ±3% or 40 rounds.
     - **Which, if either, is Louis's choice:** the statistical method is reserved (C6).
   - **The weighing (C3):** the complexity bought against a branch that stays off by default. The
     cheaper first step of phase 2 avoids most of it.
2. **Widening the envelope.**
   - **By registered run:** a topology or thread range joins the envelope only by a registered run
     on it, on the precedent's terms. Registration 3 is the first.
   - **The i9's reach:** it can add only its own thread ranges and masked shapes.
   - **Turning the fast branch on for unmeasured topologies,** that is removing the envelope, would
     be a decision to ship an extrapolation. That decision is Louis's alone. The most this entry can
     offer for it is the boundary and bytes runs and the plan tables above.

## A question for the live release (for the Coordinator; not fixed here)

0.2.6's exception checks only two things: 10 physical cores, and an L3 read from cpu0 as exactly
20 MiB. Reasoning from the code and from published specifications, without access to those
machines' sysfs, these may match it and so run 3 × 3 pipelined, unmeasured:
- **Hybrid 10-core parts with a 20 MB L3,** such as the i5-12600K, i5-13400 and i5-14400 (6
  performance and 4 efficiency cores, 16 CPUs).
  - **Why they would match:** Linux gives each efficiency core its own `core_id`, as far as is
    known, so `physical_cores` would count 10. Their L3 is reported as `20480K`, which `l3_bytes`
    reads as 20 MiB.
  - **The risk:** a team's helpers could land on efficiency cores, and a team waits for its slowest
    thread.
- **Two-socket machines with 5 cores and 20 MiB of L3 a socket,** or VMs presenting such a
  topology. `physical_cores` counts both sockets' cores; `l3_bytes` reads one socket's cache.

Whether 0.2.6 needs a guard for these is a live-release question. It is flagged here for the
Coordinator, not decided. Rule A's step 1 keeps 0.2.6's choice there, so that any change is made
deliberately, in its own PR.

## 6. Recommendation: rule A

**Why:**
- **It reproduces every measured choice:** every i9 choice that is a measured choice, plain and
  pipelined, and CI's. It changes only a cell the evidence already supports, 3 × 3 at 9 threads,
  which makes that cell a direct test.
- **It states the mechanism the 3 × 3 exception assumes,** as simply as possible: fewer tables,
  wider and pipelined teams, when the cache is the limit and the cores allow it.
- **Its scheduling decision has a stated reason:** k ≥ 3, from the pipeline model with α near 2 at
  k = 2, instead of a plan equality.
- **It is monotone in the threads used,** and adds no modelled throughput fall of its own.
- **It reaches the default only by evidence,** one envelope entry at a time.

**What it does not have** (review point 5): evidence that 4 × 2 is the best plan at 8 threads. The
8- and 9-thread rows are same-plan controls. Rule B's alternative there is mixed in the older
evidence and is exactly what registration 1 tests. Rule A is preferred because:
- it keeps today's plan where nothing has been measured against it;
- it predicts nothing on 8 MiB parts that nothing supports (rule B's 1 × 3 pipelined).

Rule A is not preferred because the evidence favours its 8-thread choice.

**Its weak points:**
- **Its L3 scaling is an extrapolation from one machine.** The level-6 result says the team count
  is not set by table bytes alone. At c = 10 the i9 can't separate a cache limit from a core limit.
  On a 16-core, 32 MiB part it would run 5 × 3 where today runs 7 × 2, and nobody knows which is
  faster.
- **Four of its six new choices are set by no measurement.** The constant is only bounded, and its
  value flips some parts between plans.
- **Parts that can't fill the fast budget keep the old plan,** even at about 0.7 of the L3.
- **Width is fixed at 3** for fast teams; wider teams on spare physical cores are untested.
- **It ignores block size,** so the inferred level-1/2 slowdown stays until the scheduling fix.
- **Multi-die, multi-socket, hybrid and VM topologies** get the old plan under the general rule,
  which is itself unmeasured there.

**The other two rules:**
- **Rule B** stays as the experiments' competing hypothesis. It makes a different i9 prediction, at
  8 threads.
- **Rule C's startup form is not recommended:** it predicts nothing on paper, and on the evidence it
  would choose the wrong knee on the one machine measured. Its offline form is a plausible way for a
  user's machine to join the envelope later.

## What this entry doesn't do

- No decoder change, no timing, no new measurement. Every figure is from the journals and records
  cited.
- The inferred level-1/2 slowdown, the 9-thread identity and rule B's predictions are hypotheses
  until the registrations of §5 measure them.

## What the review changed

The stand-in Researcher review at `2e1322f4` returned REVISE.

| # | objection | disposition | where |
|--:|---|---|---|
| 1 | The spare core is counted against N, not P; unmasked 9 threads is the 10-thread default's configuration | **Accepted.** The core left over is now counted against P (3 T_cap ≤ P − 1), and teams against N. Rule A now picks 3 × 3 p at 9 threads, a direct i9 test. §1.4 marks the unmasked cell I (strong); U only under the 9-core mask | §1.4, §2 conventions, §3, §4, registration 3 |
| 2 | Hybrid and two-socket parts; "identical on every topology" | **Accepted.** Step 1's envelope check reads the L3 as 0.2.6 does and comes before the guards, so it keeps 0.2.6's choice. The guards apply to the general rule only. The claim is now "on the tested grid". The live-release question is a separate note | §2 edges, §5 tests, "A question for the live release" |
| 3 | Non-monotone plans from the 2/3 clause | **Accepted.** The fast plan must fill its whole budget, with T_cap ≥ 2. The scan shows no fall in threads used in the team region, and no modelled throughput fall involving a width-3 plan (1,519 before). The old plan's own modelled falls, the 36 MiB case and the workers-to-teams step are stated. That step is exempt from the invariant | §2 "Why the fast plan must fill", §5 invariants, `plan_tables.py` |
| 4 | The constant is conditional; the i9 can't separate cache from cores | **Accepted.** The October tie is cited. At c = 10, T = 3 is fixed by the cores too. The free choices are tabled: 9, of which 6 are new and 4 are set by no measurement | §1.1, §2 "The constant" |
| 5 | Fairness to B and C | **Accepted.** The 8- and 9-thread rows are same-plan controls. B's 8-thread evidence is mixed (1,071 against 1,050). B uses rule A's budget conventions. C has an offline variant | §1.1, §1.8, §2 B and C, §6 |
| 6 | The experiment | **Accepted.** One family of 7 comparisons for both rules, with the minimum detectable effect. Redundant masked N = 20 cells are dropped and sibling masks added. A plain statement that nothing tests the L3 extrapolation. A block-size registration as the one i9 test of the premise. The A/A's criterion and false-alarm rate. Turbo. Qualifying and diagnostic registrations split (C1) | §4, §5 registrations 1–4, admission step 1 |
| 7 | Admission | **Accepted.** Step 1 has no gain, so it needs Louis's explicit decision (C6). The A/A is a workstation run (C5). The complexity is weighed against a branch that stays off by default (C3), and a cheaper first step proposed | §5 phase 2, admission |
| 8 | The level-1/2 finding | **Accepted.** The claim is limited to levels 1–2: 0.76–0.83, from `auto`@6 against `auto`@20 on the October build, confidence moderate to high. The fix's buffers (about +15 MB at 3 × 3), window and head-of-line control are listed, and it doesn't help levels 3–5. A forced-4 × 2 arm is added | §1.6, registration 5 |
| 9 | Minor points | **Accepted:** workers plans do use threads beyond the physical cores; max(1, ·) on T_b; the L3 is read from a CPU in the affinity, and the affinity must sit inside one cache's `shared_cpu_list`; VMs may misreport; `cpu_core` and `cpu_atom` are Intel only, so other signals are given | the terms, §2 rule and edges |

**One point taken further than asked.** Under the new conventions, rule B also picks 3 × 3 p at 9
threads. So the two rules now differ on the i9 only at 8 threads and under the masks, which is where
registration 1 concentrates.
