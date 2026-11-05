# Production walk evidence: master c4b69de3 → perf/walk-two-phase 75b13f00 (PR #157)

These files are the raw evidence for CHECKPOINT 6643481606 and PR #157, archived in answer to
D017.

## Setup
- **Input:** `wikidata-independent.bz2`, sha256 `7f5c9a0044e5a30589bf343b9b31e698849b2acc69461a55c97b550193128b45`. Blocks 0, 97 and 472 are tiny; 1,
  41, 233 and 400 are full. They cover both of the corpus's block classes.
- **Driver:** `driver/`. It is the same for both arms; only the `mibz2` path dependency
  differs: master's worktree at c4b69de3, or the branch at 75b13f00. Rust 1.97.0, fat LTO,
  `codegen-units = 1`, line tables.
- **Command:** `valgrind --tool=callgrind --dump-instr=yes --toggle-collect=c30cg::measured
  c30cg cg FILE BLOCK 1`. The profiles are `cg/{base,walk}-b<block>.out`, with the program
  stdout (length and CRC) and the console alongside each.

## Accounting, at `decode_block`'s own boundary
Every figure is inclusive Ir from `callgrind_annotate --inclusive=yes`. PROGRAM TOTALS are 16
higher in every arm: driver instructions outside `decode_block`, as D016 noted.

| block | arm | decode_block | build | walk: `room_segments` inclusive (setup, passes, handovers, tail) | rest |
|---|---|--:|--:|--:|--:|
| 1 | master | 25,751,796 | 5,429,062 | 15,426,892 | 4,895,842 |
| 1 | branch | 20,184,697 | 5,429,062 | 9,859,793 | 4,895,842 |
| 41 | master | 25,168,380 | 5,280,730 | 15,364,532 | 4,523,118 |
| 41 | branch | 19,480,512 | 5,280,730 | 9,676,664 | 4,523,118 |
| 233 | master | 25,588,404 | 5,215,492 | 15,429,922 | 4,942,990 |
| 233 | branch | 19,989,537 | 5,215,492 | 9,831,055 | 4,942,990 |
| 400 | master | 25,129,254 | 5,203,460 | 15,373,793 | 4,552,001 |
| 400 | branch | 19,490,536 | 5,203,460 | 9,735,075 | 4,552,001 |
| 0, 97, 472 | both | 7,557 / 7,926 / 8,764 | | 0 (plain walk) | |

- **Build and rest are identical in both arms.** The whole delta is the walk: −5.57 to −5.69 M
  per full block, −21.6% to −22.6% of `decode_block`, −36% to −37% of the walk. The walk
  figures are whole-walk inclusive, so they charge the handovers, the remainder loop, setup and
  the tail at their real frequencies.
- **Run-to-run variation** in the totals is under 50 instructions (allocator layout).
- **Output** (length and CRC) is identical in every block and arm.

## Listings
- `listings/{base,walk}-room_segments.asm`: the whole function.
- `listings/*-hot-b1.txt`: per-instruction counts on block 1, from `hotloop.py`.
  - **Master:** a 16-instruction step, executed 841,989 times.
  - **Branch:** 9 instructions per step, plus 4 per eight-lane turn.

"About 7" is the essential step from the source, not a measured or proven floor.

## Tests at 75b13f00
- `test-debug.txt`: `cargo test`.
- `test-release-trace.txt`: `cargo test --release --features trace --lib`.
- `miri.txt`: `cargo +nightly miri test --lib -- miri_`. 10 pass, including
  `bwt::tests::miri_room_walk_handoffs`, which asserts every earlier room-walk transition is
  reached.

The handover interleavings new in this change are covered by
`bwt::tests::room_walk_deferred_handovers` (commit 3fc3f8c6, test-only). It is 400 random
columns at 32 and 5 lanes, with up to n/2 segments, each output compared with `walk` and its
segments checked. It must reach all three classes:
- several lanes marked at one offset;
- the queue running out with a marked lane after it still waiting;
- the queue running out after an earlier marked lane at that offset took a segment.

Two mutants fail it: every waiting lane at `off + 1`, and `waiting = false`. The release
machine code of `room_segments` at 3fc3f8c6 is identical to 75b13f00's.

## The invariance argument, for the Researcher to break
The state a pass touches:
- `lanes[k]`: row, seg, p, end;
- `base[k]`;
- `bufs[seg]`;
- `next[seg]`;
- `queued`.

1. **A step of lane k** reads `T`, which the pass never writes. It writes only
   `lanes[k].row` and the byte at `base[k] + off`, which lies in `bufs[lanes[k].seg]`. That
   buffer is owned by lane k alone for as long as the lane holds that segment. So a step
   commutes with every operation on any other lane.
2. **A handover of lane k**:
   - writes `lanes[k].p`;
   - calls `finish`, which writes `bufs[lanes[k].seg]`'s length and `next[lanes[k].seg]`;
   - calls `take(queued)`, which writes `bufs[queued]` and `lanes[k]`;
   - writes `queued` and `base[k]`.

   The segment `queued` names has not yet been taken, so no other lane owns its buffer. The
   only state handovers share is `queued`.
3. **Order is preserved.** The old order at offset `off` interleaves steps and handovers in
   ascending k. The new order does all the steps, then the handovers in ascending k. By (1) the
   steps commute with the handovers, and the handovers keep their relative order. So
   `queued`, and with it the segment each lane takes, is assigned identically. Every byte
   lands at the same address, and `next[]` gets the same values.
4. **Exhaustion at lane k** (`queued == ns`). Every other lane's state must equal the state
   the old order left at that point.
   - An unmarked lane j < k had stepped in the old order too. The rule gives `off + 1`. ✓
   - An unmarked lane j > k had not stepped in the old order: `p = off`, row unchanged. The new
     order has stepped it: `p = off + 1`, row advanced. The tail would perform exactly that
     step first, writing the same byte at the same address. The resulting state is reachable,
     and the tail is order-independent across lanes by (1). ✓ (the same final buffers)
   - A marked lane j < k was handed over in both orders, so `base[j] + off + 1` is its new p.
     ✓
   - A marked lane j > k waits in both orders: `p = off`, row on its START entry. The tail
     finishes it first. ✓
5. **Buffer bounds.** Every pointer left at `off + 1` is at most `base + room`, which is at
   most `end`. The tail's pre-check grows a lane at `p == end` before writing, as before.

**Corrected (Researcher, 6643814937).** (4)'s second case is not state equality at the break:
the tail's pre-check does read `p` and `end`. The argument is a simulation relation over
commuting steps.
- The old order leaves lane j at `off` with its row unstepped. Its tail performs that ordinary
  step, then grows the buffer if it is full.
- The new order has already taken that step. The tail's pre-check grows the buffer if it is
  exactly full, leaving the written prefix intact.
- If j's next entry is its segment's end, both orders finish it at the same successor with the
  same bytes.
- `next_unset` reads `seg` and `next`, which j's step doesn't change.

The witness test for these cases is #159.
