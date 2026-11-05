# The room walk's bytes staged in one tile: registration and instruction count

Registered before any CI timing of this branch. The decision rule may be amended only in a
commit made before the first bench comment on this PR, and each amendment says what changed
and why.

## The change

This builds on the deferred-flag walk (#168, `2026-11-05-deferred-flag-walk.md`). The busy
phase now stages each byte at `tile[k][j]`, where j is the offset within a chunk of at most
`TILE` = 256 offsets. Lane k's buffer is no longer written at base + off. The tile is 32 × 256
bytes, 8 KiB in L1, and every lane addresses it from one base. That removes a pointer load
from every step, so a step is 7 instructions (`spec_offset-P0.asm`).

- **Flush.** Lane k's current segment has its bytes from `st[k]` in its tile row. They are
  copied to its buffer at its `p` at the chunk's end, and at the segment's end before
  `finish`. Every byte lands where the unstaged walk wrote it.
- **Bounds.** The pass's room bounds the copies as it bounded the writes. A lane taken
  mid-chunk writes its first byte in its new buffer (`take`) and stages from j + 1.
- **The queue running out mid-offset.** Each lane flushes up to j, or up to j + 1 if it
  stepped at j, which matches the unstaged walk's p = base + off (+1).
- **Speculative bytes** go to the tile. They are either rewritten by the replay or left past
  `st..end`, so they are never copied.

The rest of the walk is unchanged: the build, segmentation, the prefetch, the deferred-flag
replay, the tail and assembly.

## Measured: whole-decoder instructions

Inclusive Ir of the driver's `measured`, with the same driver, toolchain and input as
`2026-11-05-deferred-flag-walk.md`. Profiles are in `cg/`: `base-*` is master 4d4709de,
`branch-*` this branch.

| block | master | deferred flags (#168) | this branch | change against master | output |
|---|--:|--:|--:|--:|---|
| 0, 97, 472 (short) | 7,289 / 7,665 / 8,503 | same | same | 0 | identical |
| 1 | 20,184,311 | 19,673,153 | 19,051,749 | −1,132,562 (−5.61%) | identical |
| 41 | 19,480,138 | 18,971,641 | 18,338,734 | −1,141,404 (−5.86%) | identical |
| 233 | 19,989,050 | 19,477,424 | 18,854,633 | −1,134,417 (−5.68%) | identical |
| 400 | 19,490,067 | 18,981,100 | 18,350,906 | −1,139,161 (−5.84%) | identical |

On block 1, `spec_offset` is 6.10M Ir (7.25 a step), and the flush's `memcpy` is 0.35M.

**Indicative local timing.** This is on the shared VM (Xeon @ 2.80GHz), so it is not
evidence under the charter; it is reported because it motivated sending this to CI ahead of
the alternatives. The input is the 12 MB Wikidata file, 5 interleaved rounds, medians:

| arm | 1 thread | 4 threads |
|---|--:|--:|
| master | 1.288 s | 0.474 s |
| deferred flags | 1.257 s | — |
| this branch | 1.120 s | 0.416 s |

## A negative result: gathering the 32 entries

A third arm (`research/gather-walk`) gathers each offset's 32 entries in four AVX2
`vpgatherdd`. Rows are held in four vectors, the flags are tested with one `vmovmskps`, and
the bytes are stored from the gathered dwords with `vpextrb`. Instructions: block 1 at
13,859,535 (−31.3% against master), output identical on every block. Time, locally: 1.204 s
at 1 thread and 0.469 s at 4, slower than this branch at both. The gathers retire far fewer
instructions, but they cost more than the scalar loads they replace, and they leave no room
for the per-lane prefetch. It is not sent to CI. The branch is kept as a reference.

## The decision rule

- **Correctness (required):** CI green, Miri included; the same bytes on every corpus in
  every bench run; the room walk's tests pass.
- **Instructions (required):** at or below the base's on every full block. Met.
- **Benefit (required):** on `wikidata` at 1 or 4 threads, the CPU change or the wall change
  excludes zero on the faster side in at least one run, and the same figure's point estimate
  is not slower in the other. Until #168 merges, this PR's diff and bench include #168's
  change, and the result is read as the two together. Once it merges, this PR is re-benched
  against the new master, and the rule is applied to that run.
- **No harm (required):** as in `2026-11-05-deferred-flag-walk.md`. A wall row flagged slower
  is settled by one more run on the same head, with at most three runs on the final head.
