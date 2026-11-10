# 2026-11-10: The staged cooperative walk, counted (registration)

**Status: registered before any count of the staged walk.** This file is committed before
the first callgrind run of the staged arm, and its hypothesis, arms, inputs and filter are
frozen from that commit. Results are appended below the line at the end; nothing above it
is amended after the data. No clock is read anywhere in this work: every figure is
callgrind's Ir (**counted**), read from the source (**read**), or arithmetic on counts
(**model**).

## 1. Where this comes from

- **#330 (merged)** counted the team path at 37.91 Ir a decoded byte against one worker's
  25.79 (`2026-11-10-team-kernel-gap-count.md`). The walk is the largest gap: the team's
  walk stage is **17.71** Ir/D against the one worker's **10.19**, +7.52.
- **Read:** `coop::traverse` is `room_segments`' loop before #171's staging. It tests every
  lane's START flag at every step, and stores each byte through the lane's own pointer,
  straight into its segment.
- **The Optimiser's price** (#145 6758406452; report at `e43daa0f`,
  `state/records/reviews/inbox-6758227354/report.md`): a private, per-walker staged
  traverse is reachable inside a team without a team-wide barrier; 10–11 inclusive Ir a row
  is "a credible hypothesis, not a demonstrated price". This is its revised falsifier 2.

## 2. The construction (read, in `src/coop.rs`)

`traverse_staged` is `bwt::room_segments`' staged busy phase, its next segment claimed from
the team's one counter (`Counter`, the same Relaxed `fetch_add` as `traverse`):

- **Private state:** each walker (one call of `RoomWalk::share`) has its own 32 × 256 tile
  (8 KiB), rows and staged starts. Nothing staged is shared; there is no tile per team.
- **Ownership:** a claimed segment is walked to its end by the lane that claimed it. No
  member reads or takes another's unfinished segment.
- **The mid-pass take:** the marked lane's tile prefix `st[k]..j` is flushed, then its
  segment finished (length, successor); one claim; the new segment's first byte written
  directly (`take`); its staging starts at j + 1.
- **Queue exhaustion:** every other lane is flushed: an unmarked lane through j + 1 (it
  stepped at j), a marked lane not yet handled through j, a lane that took a segment at j
  through j + 1 (empty: st = j + 1). Rows are copied back to the lanes; the unstaged tail
  walks the rest. **The unstaged walk's rule ("the lanes after k have not stepped") is
  wrong here**, and is one of the negative controls (§4).
- **Before `share` returns,** every owned byte is in its buffer and every owned segment
  finished, so `Relay::walk_and_leave`'s AcqRel decrement (and the pool's run end, for
  plain teams) publishes it as before. No fence, flag or barrier is added. A late leader
  joins by the existing CAS and never reads another walker's tile.
- **Selection (research):** `coop_research::set_staged_traverse(true)`, read when a walk is
  set up. **Off by default:** the default decoder's teams run `traverse`, unchanged. The
  harness turns it on only with `AB_STAGED_TRAVERSE=1` (`examples/ab.rs`, `child`).

## 3. Every dispatch path (the Guardian's D083, G3)

The walk is duplicated: `bwt::room_segments` (one core) and `coop::traverse` /
`coop::traverse_staged` (teams).

| path | where | walk | this change |
|---|---|---|---|
| plain worker | `Segmented` (`decode_block` → `walk_segments_room_parts`) | `room_segments`, already staged (#171) | **none, deliberately:** it is the staging's source, and the control's reference |
| team share, plain teams | `bwt::Cooperative<K>::invert_checkpointed` → `coop::walk_room` → `RoomWalk::share` on the pool | `traverse` | selectable: `traverse_staged` when the selector is on |
| team share, pipelined teams | `Relay::walk_and_leave` → `RoomWalk::share` (helpers, and the leader's join) | `traverse` | selectable, the same `share` |
| team solo, plain teams | `Cooperative::invert` below `MIN_COOP` | `room_segments` | none (one core) |
| team solo, pipelined leader | `parallel::solo` (blocks below `MIN_COOP`) | `room_segments`, or `walk` below `MIN_SEGMENTED` | none (one core) |

**Evidence scope:** the counts below cover the pipelined teams' share
(`mibz2-pipe3-both` at 9 threads: helpers and leader joins) and, through the control,
`RoomWalk::share` on one thread, which is the plain teams' share function. Plain teams
on a pool (`walk_room`) are covered by the correctness tests only. The plain worker and
both solo paths run unchanged code.

## 4. Correctness before counts (done, in this PR)

**Transition tests** (`src/coop.rs`, `staged_*`). A walker's claims are cut off after a
budget b (`RoomWalk::share_staged_budget`). It then finds the queue exhausted, exactly as
when another walker has taken the rest, and a second walker (staged, or `traverse`) walks
what is left. Every b from 0 to past the segment count is tried, so the exhaustion lands
at every claim the walk makes.

**Every case checks:** each segment's bytes and successor against the oracle
(`segment_oracle`), the flags taken off, and the output against `bwt::walk` and against the
unstaged team walk's.

**The inputs:** crafted tables (random, period 2, runs of 37, one byte value, real BWTs of
random and of text, and the existing `columns`), at 8 and 32 lanes and 3 to 400 segments.

**Every transition is asserted reached,** per thread (`staged_seen`), in 20,084 cases:

| transition | the Optimiser's list |
|---|---|
| `MARKED_SEVERAL` | multiple marker lanes in one offset |
| `ENDS_AFTER_A_TAKE` | another worker exhausting the queue between two local claims |
| `EXHAUSTED_AT_CHUNK_START`, `_AT_CHUNK_END`, `_AT_PASS_END` | exhaustion at a chunk's start and end and a pass's end |
| `LATER_STEPPED`, `LATER_STEPPED_ON_START`, `LATER_STEPPED_FULL` | an unmarked lane after the exhausting one, already stepped |
| `ENDS_WITH_LANES_WAITING` | marked lanes still pending at exhaustion |
| `DONE_IN_TAKING_CHUNK`, `TAKE_AT_CHUNK_END`, `TAKE_IN_LATER_CHUNK` | several takes in a lane and chunk; a take at j = w − 1 |
| `GROWS`, `TAIL_ENTERED_FULL`, `PASS_ENDS_FULL`, `ROOM_OVER_TILE` | grow after a flush; the tail entered full; passes over several chunks |
| `NEVER_FULL`, `NO_SEGMENT` | S < L; a walker (a late leader) finding no segment |

**Negative controls** (`staged_mutations_are_caught`). Each mutation is the same function
with one transition broken (`traverse_staged`'s `MUT`, 0 in the decoder). Each must fail
the same sweep:

| mutation | what it breaks | caught in |
|---|---|--:|
| `OLD_RULE` | lanes after the exhausting one flushed through j: "have not stepped" | 14,045 cases |
| `ST_AT_J` | the new segment's staged start at j: the tile's stale byte copied | 18,404 |
| `FINISH_BEFORE_FLUSH` | a length published before its last fragment | 14,882 |
| `WAITING_AS_STEPPED` | a pending marked lane flushed through j + 1 | 8,001 |
| `NO_ROWS_BACK` | rows not copied back before the tail | 14,980 |
| `NO_EXHAUST_FLUSH` | `share` returning with owned bytes still staged | 14,980 |
| `NO_TAIL_GROW` | a lane entering the tail full with its stale end | 150 (a panic before any write) |

**The other checks:**
- **The decoder's paths with real threads:** the existing walk tests run both walks:
  `walk_room_is_walk`, `split_build_and_walk_are_build_and_walk` (shares on threads, and the
  pipelined run at every wait) and `pipelined_leader_late_or_early` (the leader late and
  early).
- **The bench corpora** (`staged_on_corpora`, ignored, run with `MIBZ2_STAGED_CORPORA`):
  - the inputs: every block of at least `MIN_COOP` rows of `wikidata.bz2` (SHA-256
    `82f65ace…`), and of `text`, `random`, `records`, `runs`, `pattern` and `onebyte`;
  - the cases: 1,024 and 1,536 segments, exhaustion forced at 10 claims, and two and three
    staged walkers on threads;
  - the result: 247 blocks, 5,928 cases, every segment, successor and output exact.
- **End to end** (`tests/staged_traverse.rs`): with the selector on, the plain teams of 2,
  3 and 4 and the pipelined teams with park and small give libbzip2's bytes.
- **Miri:** `miri_staged_transitions` (CI's `miri_` filter), plus the staged arms of
  `miri_walk_room_handoffs` and `miri_split_build_and_walk` (threads, the race detector).

## 5. The hypothesis

**Model (the Optimiser's, not a count):** the team's walk stage falls from 17.71 toward
**about 10–11 inclusive Ir a decoded byte** at 9 threads. That is the one worker's 10.19
plus the team's segmentation: twice the segments (1,024 against 512), so twice the takes,
the sort (+0.06) and the end searches.

**The residual risks,** each countable here:
- **The tails:** 3.3% of `traverse`'s bytes were in its unstaged tail. The staged walk
  keeps the same tail, and its share may grow, since a walker's lanes drain on their own.
- **The flushes' distribution:** fragments per segment, and empty copies.
- **The late leaders:** a leader that joins with fewer than 32 segments left never fills
  its lanes, so gets no staging at all.

## 6. Arms, inputs and binary

**The binary:** this branch's head at the commit that adds this file, built with
`cargo build --release --example ab --example walk_count` (fat LTO,
`debug = "line-tables-only"`), rustc 1.99.1, valgrind 3.22.0, on the shared machine.
Its SHA-256 is recorded with the results. One binary serves both team arms, which differ
only in the environment variable.

**The inputs (frozen):**
- **I1–I3: three full Wikidata blocks.** Blocks 1, 41 and 233, counted from 0, of the
  held-out `wikidata-independent.bz2` (SHA-256
  `7f5c9a0044e5a30589bf343b9b31e698849b2acc69461a55c97b550193128b45`): the blocks the
  earlier journals use (`2026-11-05-tile-size.md`,
  `2026-11-09-build-window-runs-v2.md`). Each is extracted, its bits untouched, as a stream
  of its own (`walk_count extract`). The streams' SHA-256s are recorded with the results.
- **I4: the whole CI corpus,** `wikidata.bz2` (SHA-256 `82f65ace…c2c29915`, 431 blocks,
  233 of them cooperative), #330's input.

**The arms, each under**
`valgrind --tool=callgrind --dump-instr=yes --separate-threads=yes`:

| arm | command | what it is |
|---|---|---|
| **old** | `ab child mibz2-pipe3-both 9 FILE` | the team as #330 counted it |
| **staged** | `AB_STAGED_TRAVERSE=1 ab child mibz2-pipe3-both 9 FILE` | the same, its walk staged |
| **ctl-old** | `walk_count control FILE old 1024` | one caller, `RoomWalk` at 1,024 segments, unstaged |
| **ctl-staged** | `walk_count control FILE staged 1024` | the same, staged |
| **ctl-room1024** | `walk_count control FILE room 1024` | the one-core `room_segments` at the team's segment count |
| **ctl-room512** | `walk_count control FILE room 512` | the one worker's walk, #330's 10.19 |

**Why 1,024:** pipelined K = 3 requests 512 × (K − 1) segments. The control's
`RoomWalk::new` and `room_segments` draw the same pseudo-random starts for the same count,
so the four control arms walk identical segments.

**Counted, per arm** (`bench/team_gap_count.py staged`):
- **The stage table,** Ir/D: the walk stage is inclusive (`RoomWalk::new`, the start sort,
  the walker, its memcpy flushes, `RoomWalk::finish`). The **full inversion** is build +
  walk + assembly.
- **Per thread** (the team arms):
  - build shares, walker calls by walker, and claims (the `lock xadd`);
  - walked bytes, by store: take, full phase (busy) and tail;
  - fragments (the staged walker's memcpy calls, empty ones included) and their
    inclusive Ir;
  - the copied bytes are the full-phase bytes by construction: each staged byte is copied
    once, which the tests check.
- **Denominators:** D (decoded) and N (rows) per input, and M (walked bytes).

## 7. The filter (frozen)

**PASS requires all of these on I4 (the whole corpus):**

1. **Same bytes:** the old and staged arms' output hashes are equal, and equal to #330's
   (`a8830caee5c511a6`, 209,700,196 bytes). The four control arms' hashes are equal.
2. **Team engagement shown** in the staged arm:
   - every helper's `traverse_staged` calls equal its build shares (it walked in every
     block of its team);
   - no helper walked more than two thirds of its team's helper-walked bytes;
   - at least one leader called `traverse_staged`;
   - `traverse` itself has zero calls in the staged arm, and `traverse_staged` zero in
     the old arm.
3. **The walk:** the staged arm's inclusive walk stage is **≤ 12.0 Ir/D** at 9 threads.

**Otherwise:**
- **Condition 1 or 2 failing rejects the measurement** (FAIL-MEASUREMENT), not the
  mechanism.
- **Condition 3 failing is FAIL:** the hypothesis is falsified at this construction.

**Why 12.0:**
- **The model's budget:** from the one worker's 10.19, add
  - the doubled segmentation (ctl-room1024 − ctl-room512 measures it; expected
    +0.3 to +0.6);
  - the tail: 3.3% of bytes at about 7 Ir/B above the busy rate, about +0.25;
  - the leaders' unstaged joins: 2–4% of bytes.

  That gives about 10.7–11.3. 12.0 leaves 0.7–1.3 Ir/D for flush fragmentation and
  anything not modelled.
- **The bar's meaning:** above 12.0, staging recovered less than 76% of the 7.52 gap.

**Reported, not gating:**
- the three blocks I1–I3 (any block above 12.5 is named as a residual);
- the control's staged against old (expected: staged ≤ old − 4);
- the old arm against #330's 17.71 (a binary-identity check; a difference over 0.1 is
  explained or the measurement is rejected);
- the tail share, fragments a segment, and the leaders' share.

**What a pass earns:** a forced-team CI registration, **not a merge**. No forced-team CI
arm is adopted yet (the Guardian's D083). The screen design is written after this report.
Engagement on hardware is not shown by these counts: the pipelined default engages only
at exactly 10 physical cores and a 20 MiB L3, and the generic teams from 6.

**What a screen must assert executed for this port** (proposed for the screen's design):
- the team arm ran `mibz2::coop::traverse_staged` on at least two helper threads a team,
  and `coop::traverse` not at all;
- the base arm ran `coop::traverse` and not `traverse_staged`;
- both ran `FusedBuild::share`, `mtf_checkpoints_avx2` and `Relay::walk_and_leave`, so the
  team path, not a fallback, was measured;
- the plain worker's `room_segments` and the solo paths are unchanged, so the screen
  needs no arm for them.

---

## Results

(Appended after the counts. Nothing above this line changes.)

**Verdict: FAIL-MEASUREMENT under the frozen filter.**
- **Condition 3, the walk, is met:** 9.97 Ir/D against the 12.0 bar.
- **Condition 2(b), engagement, is not:** in one team of the staged arm, one helper walked
  69.5% of the two helpers' bytes, against a bar of two thirds.

By the rule, the measurement is rejected, not the mechanism. The data are recorded here
and not re-read; no rerun was made to get past the bar. Whether a re-registered count
follows is the coordinator's call.

### The run

- **Binary:** `examples/ab` and `examples/walk_count` at `c78c18d3d6178caa5abae03f07288974ac995a01`,
  rustc 1.99.1, valgrind 3.22.0. SHA-256: `ab` `b49a9146…b111c669`,
  `walk_count` `44e79421…cca6380`.
- **One change after the registration commit (`6c3bc692`), before any count:** with
  `share`'s choice of walker beside it, LTO had inlined `traverse` into `RoomWalk::share`.
  - The default walk's machine code had changed, and the counts would no longer separate
    it. `#[inline(never)]` on `traverse` restores it as a function of its own, as on
    master (`c78c18d3`).
  - No count had been taken: the first build was only inspected (`nm`).
  - The old arm's walk, 17.771, is within 0.06 of #330's 17.712, inside the registered 0.1.
- **The inputs, extracted at `c78c18d3`** (`walk_count extract`), each also decoded by
  libbzip2 (`bzip2 -dc`) to its length:

  | input | SHA-256 | bytes | block CRC |
  |---|---|--:|---|
  | I1, block 1 | `c9e36596fa95ef5cb1b865e52fd9027bd01d26994674bec96690b9ada23099d9` | 899,971 | `0f0a732d` |
  | I2, block 41 | `58c3e4f8fa3db16c4a89170c9f0c42834122282cf6ddedd4dd1fd19dde5cff9b` | 900,000 | `c91a77e1` |
  | I3, block 233 | `51cec03046c72e92dddc48e386ad3088b9ede2cb6848222543af619472adb954` | 899,978 | `25492446` |

- **The commands:** `run.sh`, run once, sequentially, on the shared machine.
- **The archive:**
  - `cg/` holds every profile, gzipped, one file a thread;
  - `out/` holds each arm's output: the harness's JSON, the control's line;
  - `report.txt` is the script's output from the archived profiles:

    ```
    python3 bench/team_gap_count.py staged --binary AB_OR_WALK_COUNT --rows N --decoded D \
        --arm NAME=docs/journal/2026-11-10-staged-traverse-count/cg/PREFIX ...
    ```

### Condition 1: the same bytes (met)

- **The team arms** on I4: `old` and `staged` both give 209,700,196 bytes, hash
  `a8830caee5c511a6`, which is #330's.
- **The control:** on its 233 cooperative blocks, all four arms give 209,691,304 bytes,
  hash `db1c87dcd46ca166`.
- **I1–I3:** each block's team arms agree, and so do its four control arms.

### Condition 3: the walk (met), and the stage table (counted)

**I4, the team arms at 9 threads,** in Ir per decoded byte (D = 209,700,196):

| stage | old | staged | staged − old |
|---|--:|--:|--:|
| scanner | 3.203 | 3.203 | 0.000 |
| MTF | 4.715 | 4.715 | 0.000 |
| build | 9.106 | 9.106 | 0.000 |
| **walk (inclusive)** | **17.771** | **9.966** | **−7.805** |
| assembly | 0.170 | 0.170 | 0.000 |
| RLE1 | 0.946 | 0.959 | +0.013 |
| CRC | 0.485 | 0.485 | 0.000 |
| coordination | 0.073 | 0.071 | −0.002 |
| harness | 1.533 | 1.497 | −0.036 |
| other | 0.064 | 0.036 | −0.028 |
| **full inversion (build + walk + assembly)** | **27.047** | **19.242** | **−7.805** |
| **whole process** | **38.064** | **30.207** | **−7.857** |

**The walk's parts** (Ir/D):

| part | old | staged |
|---|--:|--:|
| the walker, self | 17.584 | 9.631 |
| the flush (memcpy, from the walker) | none | 0.139 |
| `RoomWalk::new`, self | 0.041 | 0.041 |
| the starts' sort | 0.107 | 0.107 |
| reserve and memset | 0.039 | 0.048 |

**Per walked byte:** M = 209,697,662, so the staged walk is 9.967 Ir/M.

**The model's figures, for comparison:**
- **Against the one worker:** #330's 10.188. The staged team's walk is below it.
- **The hypothesis:** 10–11. The count is below its low end.
- **The registered bar:** 12.0.

### The control: one caller, the same segmentation (counted)

**I4's 233 cooperative blocks** (D = 209,691,304). The walk stage, inclusive, Ir/D:

| arm | walk | the walker's self Ir/D |
|---|--:|--:|
| ctl-old (`traverse`, 1,024 segments) | 17.691 | 17.512 |
| ctl-staged (`traverse_staged`, 1,024) | **9.885** | 9.560 |
| ctl-room1024 (`room_segments`, 1,024) | 10.096 | 9.821 |
| ctl-room512 (`room_segments`, 512: the one worker) | 10.220 | 10.026 |

**What the control shows:**
- **Staged against old:** −7.81 for one caller, the same as in the team (−7.81). The saving
  is the loop's construction, not anything about the team.
- **Staged against `room_segments` at the same segments:** 0.21 lower. Below the one-core
  kernel, not above it.
- **The doubled segmentation,** room1024 against room512, costs nothing measurable here:
  it is −0.12, against the +0.3 to +0.6 the model allowed.
- **The team against one caller:** the staged team's 9.966 against one caller's 9.885 (a
  slightly different D), +0.08. Coordination, late leaders and the tail together cost
  less than 0.1 Ir/D.

**I1–I3, the walk stage** (Ir/D, D the block's bytes):

| block | old (team) | staged (team) | ctl-old | ctl-staged | ctl-room1024 | ctl-room512 |
|---|--:|--:|--:|--:|--:|--:|
| 1 | 18.296 | 10.567 | 18.210 | 10.488 | 10.686 | 10.581 |
| 41 | 18.403 | 10.581 | 18.214 | 10.502 | 10.701 | 10.370 |
| 233 | 18.354 | 10.565 | 18.209 | 10.486 | 10.685 | 10.548 |

**About the single blocks:**
- **Each runs in a fresh process,** so its figures carry about 0.3 Ir/D of first-allocation
  `reserve`, which I4 amortises.
- **No block exceeds 12.5.**
- **One thread walked almost all of each block** in both team arms: valgrind ran one
  thread at a time. So the blocks show the per-byte cost, not engagement.

### Condition 2: engagement (not met)

**The staged arm, by thread:** walked bytes are take + full phase + tail, from the
executions of the walker's byte stores. Fragments are its memcpy calls.

| thread | role | build shares | `traverse_staged` calls | claims | walked bytes | tail | fragments |
|---|---|--:|--:|--:|--:|--:|--:|
| 3 | leader | 0 | 44 | 16,586 | 14,576,807 | 526,426 | 77,918 |
| 4 | leader | 0 | 46 | 17,134 | 15,052,199 | 612,031 | 81,088 |
| 5 | leader | 0 | 48 | 23,716 | 20,826,369 | 703,323 | 114,036 |
| 7 | helper, team of 78 | 78 | 78 | 30,636 | 26,897,682 | 976,342 | 145,150 |
| 9 | helper, team of 78 | 78 | 78 | 32,231 | 28,249,242 | 1,066,210 | 153,529 |
| 8 | helper, team of 79 | 79 | 79 | 44,742 | 39,282,386 | 1,306,146 | 214,999 |
| 11 | helper, team of 79 | 79 | 79 | 19,702 | 17,240,070 | 754,948 | 91,719 |
| 10 | helper, team of 76 | 76 | 76 | 28,213 | 24,732,665 | 898,138 | 135,337 |
| 12 | helper, team of 76 | 76 | 76 | 26,026 | 22,840,242 | 785,360 | 127,902 |

**Against each part of condition 2:**
- **(a) Met:** every helper's `traverse_staged` calls equal its build shares (78 + 79 +
  76 = 233, the cooperative blocks). Helpers are paired into teams by equal share counts,
  as in #330.
- **(b) Not met:** the largest helper shares of their team's helper-walked bytes are
  0.512 (team of 78), **0.695 (team of 79)** and 0.520 (team of 76). The bar is two thirds.
  For comparison, the old arm's are 0.513, 0.553 and 0.563.
- **(c) Met:** all three leaders called `traverse_staged`, 44 to 48 times, and walked
  24.1% of the bytes between them (the old arm: 24.7%).
- **(d) Met:** in the staged arm `traverse` has no calls, and in the old arm
  `traverse_staged` has none.

**The arithmetic checks:**
- **The bytes sum to M:** 238,382 + 201,830,356 + 7,628,924 = 209,697,662 (take, full
  phase, tail).
- **The claims:** 238,986 = 238,382 takes + 604 exhausting claims, one a call.

**Why (b) failed, read (not tested):**
- callgrind runs one thread at a time, so how the claims split between two helpers is
  valgrind's schedule;
- thread 11 also built 275 M Ir of its shares before walking;
- the old arm's split on the same machine was within the bar.

None of this changes the rule. The walk figure is not affected by the split: the control,
with one caller and no split at all, gives 9.885.

### The residual risks, counted

**The tail:**
- **In the staged team:** 3.64% of walked bytes, against 5.80% in the old team (this run)
  and 3.3% in #330's.
- **In the control:** 3.01% (staged) and 3.02% (old).
- **The tails did not grow;** the pricing's chief risk is not borne out.

**The flushes:** 1,141,678 fragments, 4.79 a segment, empty copies included. Their memcpy
costs 0.139 Ir/D, against the one worker's 0.123 at 512 segments.

**The late leaders:**
- they walked 24% of bytes with no loss visible in Ir;
- team, 9.966, against one caller, 9.885: +0.08 for everything a team adds, including
  claims;
- on real hardware, a leader joining late gets no staging; this count cannot price that.

### What the count does not show

- **Time.** Instructions are not time: the walk waits on memory on the i9. #171's
  contention evidence was taken on plain workers. The Optimiser's own note stands: missing
  10–11 Ir would not have disproved a contention gain, and meeting it does not prove a
  wall gain.
- **Balance on hardware.** Valgrind's schedule is not the i9's.
- **The plain teams on a pool** (`walk_room`): covered by the correctness tests, not
  counted.

### The model, updated (model, not observed)

**#330's model, with the measured walk:** closing the walk gap leaves the team at 30.39
Ir/B (#330 §6). Counted here, the staged team's whole process is 30.207, and its walk
9.966, below the one worker's walk.

**What is left of the team's excess over one worker:**
- **The excess:** 30.207 − #330's 25.792 = 4.42 Ir/B;
- **The rest of it** is mostly the build (+3.94 in #330) and MTF (+0.60).

---

## Addendum: the Guardian's D084 (appended; nothing above is changed)

D084 is #145 comment 6759487306; its record is `state/records/D084` at `f4012c4f`. It rules:
- **FAIL-MEASUREMENT stands.** The helper-only fractions reproduce as 0.512254, **0.694987**
  and 0.519890. 39,282,386 exceeds twice 17,240,070 by 4,802,246 bytes, and the leaders'
  bytes are not in that denominator.
- **What is held:** a replacement qualifying count, and the single-caller result standing
  in for the team gate. The pass-earned forced-team CI registration is unearned.
- **No line STOP.** The work continues with the D083 screen's design, independent B5
  review, and at most one separately registered diagnostic.

### A registered-binary deviation

**The pins:**
- §6 pins the binary to "this branch's head at the commit that adds this file":
  **`6c3bc692b26a1dd506d41f477ed0d5c34f68c918`**.
- Every count above was made with binaries built at
  **`c78c18d3d6178caa5abae03f07288974ac995a01`**.

**The difference** is the one source change between them (`git diff 6c3bc692 c78c18d3`:
`src/coop.rs`, 3 lines added): `#[inline(never)]` and a comment on the unstaged
`coop::traverse`.

**Why it was made, as reported:**
- At `6c3bc692`, LTO had inlined `traverse` into `RoomWalk::share`; `nm` showed no
  `coop::traverse` symbol.
- The binary was inspected and never profiled.
- The commit order is consistent with this account. Nothing else verifies that no count
  preceded the change.

**This is a deviation from the frozen pin, not an annotation.**
- The figures above hold for `c78c18d3`'s binaries only.
- The old arm's 17.771 against #330's 17.712 is a diagnostic check, not proof of
  machine-code identity with master or with `6c3bc692`.

**The full build identities** (both pins: rustc 1.99.1 `58cf8f9c3` 2026-10-13, cargo 1.99.1
`5f94df478`; release profile: fat LTO, one codegen unit, `debug = "line-tables-only"`;
`cargo build --release --example ab --example walk_count`, on the shared x86_64 machine):

| source | binary | SHA-256 | profiled |
|---|---|---|---|
| `6c3bc692` (the registration's pin) | `examples/ab` | `ba1d254da1056d16eb830c3e424ca1c6a0e9d0ff2a69d292bc823b4432adcb5e` | no |
| `6c3bc692` | `examples/walk_count` | `5328bd67e3842306d32a22fb674b02af5dfb62926cd8181d715c04d08a588fd2` | no |
| `c78c18d3` (counted) | `examples/ab` | `b49a914672963a904dbae3d03aefbc32d68d48a42426268915f61d1b0111c669` | every team arm |
| `c78c18d3` (counted) | `examples/walk_count` | `44e79421c6a9fbd12fbf657fc92fdabc2a17f8e35b856ad4c99d296bfcca6380` | every control arm, and the extraction |

**Any future measurement** is frozen only after the inlining and attribution arrangement is
fixed, and at the exact commit that will be built.

### Correction: a clock was read

The status line says "no clock is read anywhere in this work". **That is false of the
archive.**
- **What the archive holds:** `out/*-old.json` and `out/*-staged.json` are the harness's
  child records. They contain `wall`, `cpu`, `thread_cpu`, `runq_s` and `sampler_s`,
  recorded under valgrind.
- **The correction:** the inference here is from instruction counts only. Those timing
  fields are incidental instrumented output, kept unaltered and **unused for any
  performance inference**. They establish no native speedup.

### Corrections of scope (D084's reproduced figures)

**The one-caller saving** is (3,709,609,095 − 2,072,879,909) / 209,691,304 = **7.805422 Ir/D**.
- **It supports:** a construction-dependent instruction saving in this binary, without
  competing walkers.
- **It does not show** that the team figure is unaffected by scheduling.

**The team against the control** is **0.081459 Ir/M** on their common walked-byte
denominator (M = 209,697,662).
- **What it is:** a descriptive difference between two harnesses and paths.
- **What it is not:** a bound on every team overhead, on scheduling effects or on native
  time.
- **Withdrawn:** the results' line "coordination, late leaders and the tail together cost
  less than 0.1 Ir/D", and the report's "+0.08 for everything a team adds". Both are too
  strong.
