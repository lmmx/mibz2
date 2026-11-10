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
