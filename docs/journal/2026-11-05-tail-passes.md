# The room walk's tail in deferred-flag passes: registration and instruction count

Registered before any CI timing of this branch. The decision rule may be amended only in a
commit made before the first bench comment on this PR, and each amendment says what changed
and why.

## The change

This builds on the staged walk (#171, `2026-11-05-staged-walk.md`). Once the queue of segments
is empty, the lanes still walking finished one step at a time. Each step went through a `Vec`
of lanes, testing the entry's flag and the buffer's end: about 18 instructions a step, against
the busy phase's 7.

The tail now walks in passes, as the busy phase does:

- **A pass.** It runs over `room` offsets, the fewest bytes any live lane has spare. Each
  offset steps every live lane without testing its flag, then tests the OR of the entries
  once.
- **A start.** An offset whose entries hold a start is done again lane by lane, from the rows
  before it, which are kept in the other slot (as in #168). A lane that reads its start
  finishes there. The last live lane takes its place.
- **Order.** No segment is taken in the tail, so the order the lanes finish in changes no
  byte.
- **Speculative bytes and rows.** As in the busy phase of #168, each speculative byte goes at
  base + off, inside its lane's room. It is either written again by the replay, or left past
  the segment's end, which `set_len` never exposes. A speculative row goes only into the
  slot a replay discards. The prefetch uses `wrapping_add`.

The test `room_walk_deferred_handovers` now also requires a tail offset done again lane by
lane (`TAIL_REPLAYED`), at 32 lanes and at 5.

## Measured: whole-decoder instructions

Inclusive Ir of the driver's `measured` (one `decode_block` on a fresh `Scratch`), with the
driver of `2026-11-05-deferred-flag-walk/` (rustc 1.97.0, fat LTO, `codegen-units = 1`). The
input is `wikidata-independent.bz2`, sha256 `7f5c9a0044e5a305…`, plus CI's `runs` corpus (one
block, `ab gen` scale 4, sha256 `af3be4d130faf312…`). Profiles and outputs are in `cg/`:
`base-*` is master 6b337819 (#171), `branch-*` is this branch.

| block | master (#171) | this branch | change | output |
|---|--:|--:|--:|---|
| 0, 97 (short) | 6,981 / 7,425 | same | 0 | identical |
| 472 (short) | 8,199 | 8,192 | −7 | identical |
| 1 | 18,725,285 | 18,325,660 | −399,625 (−2.13%) | identical |
| 41 | 18,066,394 | 17,770,269 | −296,125 (−1.64%) | identical |
| 233 | 18,524,537 | 18,209,467 | −315,070 (−1.70%) | identical |
| 400 | 18,081,311 | 17,728,709 | −352,602 (−1.95%) | identical |
| `runs` 0 | 17,455,652 | 17,338,677 | −116,975 (−0.67%) | identical |

The outputs are also identical to master 4d4709de's, from before #171.

## The decision rule

- **Correctness (required):** CI green, Miri included; the same bytes on every corpus in
  every bench run; the room walk's tests pass.
- **Instructions (required):** at or below the base's on every full block. Met.
- **Benefit (required):** on `wikidata` at 1 or 4 threads, either the CPU change or the wall
  change excludes zero on the faster side in at least one run. The same figure's point
  estimate must not be slower in the other run.
- **No harm (required):** as in `2026-11-05-deferred-flag-walk.md`. A wall row flagged slower
  is settled by one more run on the same head, with at most three runs.
- **Power:** the tail is about 2% of a block's instructions, and it runs while fewer than 32
  lanes are live, so less of it overlaps. A gain of that size may be below the bench's noise.
  If no run shows a gain beyond noise, the change is reported as not shown and closed,
  whatever its instruction count.
