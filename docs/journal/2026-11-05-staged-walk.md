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

## Result: merged as #171 (6b337819)

#168 was closed unmerged, so the rule was read against master with both changes together.
There were five CI runs (bench 3.2.0, 10 paired rounds each). The first three ran on head 0a32416
against 4d4709d; the last two ran on head 2301189 (master e7c30a8 merged in) against e7c30a8.

| run | runner | `wikidata` wall at 1 / 2 / 4 threads | CPU at 4 threads | amplification at 4 threads |
|---|---|---|---|---|
| 1 | EPYC 9V45 | −0.1% / −3.2% / **−14.4%** | **−17.6%** | **−17.5%** |
| 2 | EPYC 7763 | **−7.3%** / **−9.8%** / **−10.4%** | **−11.5%** | −4.5% |
| 3 | EPYC 7763 | **−12.6%** / **−9.8%** / **−13.0%** | **−15.2%** | −3.0% |
| 4 | Xeon 8573C | **−4.7%** / **−4.8%** / **−5.2%** | **−7.0%** | **−2.4%** |
| 5 | EPYC 9V45 | +0.4% / −1.1% / **−13.9%** | **−16.3%** | **−16.6%** |

Bold marks a 95% interval that excludes no change.

- **Benefit:** met in every run. No `wikidata` figure was slower.
- **No harm:** unresolved, as the rule requires it to be reported.
  - `runs` at 1 thread was flagged slower twice: run 2 (+1.7%, +0.0 to +3.4) and run 4 (+1.9%, +0.1 to +3.7).
  - Each time, the next run on the same head did not repeat the flag: run 3 −0.5%, run 5 +0.2%.
  - Instructions do not back a staging cost on `runs`. Its single block runs 0.8% fewer instructions (17,756,248 against 17,904,270).
  - Most of that decode is code this change leaves alone (CRC, an RLE1 memset, the build, MTF). That code moved in the binary: `crc::clmul_update` goes from 64-byte aligned to 16 mod 64. A code-layout check is under way.
- **Mechanism:** the gain is mostly lower contention between SMT siblings, not fewer instructions.
  - At 4 threads on Zen 5, amplification fell by about a sixth (the CPU spent on the same bytes, at 4 threads against 1).
  - At 1 thread on Zen 5, wall was unchanged.
  - On Zen 3 (7763) and Sapphire Rapids (8573C), the 1-thread gain was 5–13%.
- **Correctness:**
  - An adversarial review of the busy phase and of the tail passes (#173) found no defect.
  - It found coverage gaps at chunk edges for 32 lanes. `room_walk_staged_chunk_edges` now requires each of those states.
  - Four mutants planted at chunk edges each fail it within 0.25 s.
  - Without the test, the two at the queue's exhaustion fail only `room_walk_crafted_permutations`, a randomised test that takes 35–85 s.
