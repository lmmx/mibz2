# The room walk with deferred flag checks: registration and instruction count

Registered before any timing of this branch. The decision rule below may be amended only in a
commit made before the first bench comment on this PR, and each amendment says what changed and
why.

## The change

The room walk's busy phase (every lane walking, the queue not yet empty) steps all 32 lanes at
each offset without testing their flags. It ORs the entries it read and tests the OR once. If
no lane read a segment's start, the result is exactly the scalar offset's. If one did, the
offset is done again by the scalar offset, unchanged from master, from the rows as they were
before it. That path hands segments over in lane order, so the hand-over order is master's.

- **Rows in two slots.** An offset reads one slot and writes the other, so the rows from before
  it survive it with no copy. A replay steps the rows in place, and the next offset reads that
  same slot.
- **Speculative bytes.** Each goes at base + off, inside its own lane's room, so it's in
  bounds. Each is then either written again by the replay, or left past its segment's length,
  which `set_len` never exposes.
- **Speculative rows.** A start's entry gives a row with its flag in bit 23 (bit 31 shifted). It
  goes only into the written slot, which a replay discards, so it is never read. A
  speculative prefetch uses `wrapping_add` and doesn't fault.
- **Machine code.** `spec_offset` is out of line: inlined, the table pointer was reloaded from
  the stack twice per step (+0.92M Ir on a full block). The 32 lanes are unrolled, with no loop
  control. A step is 8 instructions (row load, entry load, OR, base load, byte store, shift,
  row store, prefetch), against master's 9 plus a 4-instruction turn per 8 lanes
  (`spec_offset-P0.asm`, `room_segments-master.asm`).

**Why one offset, not eight.** The Optimiser's B speculates 8 rounds per tile (review
`d143d8fe`, §3). Past a start, the speculative row has bit 23 set and can be beyond the table,
so every speculative round after the first would need a mask before its load. That is one
instruction a step, the instruction this saves, plus a replay probability of about
1 − e^(−256/1758) ≈ 13.6% a tile instead of about 1.8% an offset. T = 1 needs neither the mask
nor a snapshot.

Production is otherwise unchanged: the build, segmentation, the prefetch, the tail, assembly and
P87's in-place output.

## Measured: whole-decoder instructions

Inclusive Ir of the driver's `measured` (one `decode_block` on a fresh `Scratch`), from
Callgrind. Both arms are built from `driver/` with only the `mibz2` path differing: rustc 1.97.0,
fat LTO, `codegen-units = 1`. The input is `wikidata-independent.bz2`, sha256
`7f5c9a0044e5a305…`. The profiles, stdout (length and CRC) and consoles are in `cg/`.

| block | master 4d4709de | this branch | change | walk, master → branch | output |
|---|--:|--:|--:|--:|---|
| 0 (54 B) | 7,289 | 7,289 | 0 | short path | identical |
| 1 (899,971 B) | 20,184,311 | 19,673,153 | −511,158 (−2.53%) | 9,859,793 → 9,348,635 | identical |
| 41 (900,000 B) | 19,480,138 | 18,971,641 | −508,497 (−2.61%) | 9,676,664 → 9,168,167 | identical |
| 97 (56 B) | 7,665 | 7,665 | 0 | short path | identical |
| 233 (899,978 B) | 19,989,050 | 19,477,424 | −511,626 (−2.56%) | 9,831,055 → 9,319,429 | identical |
| 400 (899,993 B) | 19,490,067 | 18,981,100 | −508,967 (−2.61%) | 9,735,075 → 9,226,108 | identical |
| 472 (70 B) | 8,503 | 8,503 | 0 | short path | identical |

**Block 1's accounting:**
- 26,313 busy-phase offsets: 25,835 kept and 478 replayed (1.82%).
- `spec_offset` is 6.95M Ir over the 26,313 calls, 264 per offset (8.25 per step).
- The replays add about 478 × 32 scalar steps.
- The tail (58k steps at about 18 Ir each) and the build are unchanged.

A block's saving is in the walk alone and recurs every full block: this is not a first-growth
effect.

## The decision rule

- **Correctness (required):**
  - CI green, Miri included;
  - the same bytes on every corpus in every bench run;
  - the room walk's tests pass, including `room_walk_deferred_handovers`, which now requires an
    offset kept and an offset replayed, at 32 lanes and at 5.
- **Instructions (required):** `decode_block` at or below master's on every full block. Met
  above.
- **Benefit (required):** on `wikidata` at 1 or 4 threads, the CPU change or the wall change
  excludes zero on the faster side in at least one run, and the same figure's point estimate is
  not slower in the other run.
- **No harm (required), each run read on its own:**
  - a wall row (any corpus × thread count) flagged **slower** is a harm signal;
  - one more run on the same head settles it: flagged again, it is a harm, and the rule fails;
  - not flagged again, the row is reported as unresolved, with all three runs;
  - at most three runs in all.
- **Power:** control rows' 95% intervals in recent runs are ±1–5%. A harm smaller than a row's
  half-width can't be excluded by these runs, and this PR doesn't claim to exclude it.
- **Falsified, and closed with a write-up, if** the benefit condition fails or a harm is
  confirmed.

A pass establishes an end-to-end gain from fewer walk instructions. It doesn't establish that
the gain comes from instruction count rather than from the branch-free offset's effect on
overlap. Both change together here.
