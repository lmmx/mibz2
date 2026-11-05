# The build's runs of 16 in SSE2: registration and instruction count

Registered before any timing of this branch. The decision rule may be amended only in a commit
made before the first bench comment on this PR, and each amendment says what changed and why.

## The change

`build_one_pass` gives 16 equal last-column bytes their 16 consecutive slots with one cursor
step (libsais's RUN_LANES). Their 16 entries, `ll[s + j] | (base + (j << 8))`, were written as
16 scalar 4-byte stores. `t` is a raw pointer that might alias `ll`, so LLVM didn't vectorize
it: about 31 instructions a run on Wikidata's block 1, with 37.5k runs. A version that first
copied into local arrays was no better (+0.61M Ir: still not vectorized at the baseline target).

`run16` now builds the 16 entries in SSE2, which is x86_64's baseline, so there's no runtime
dispatch:
- one 16-byte load;
- the bytes widened to 32 bits by unpacking with zero, twice;
- each quarter added to `base + (j << 8)`, which is an OR, since a byte and a link share no bit;
- four 16-byte stores.

Other targets keep the scalar loop. The bounds check, the cursor proof and the scalar one-row
path are unchanged.

## Measured: whole-decoder instructions

Inclusive Ir of the driver's `measured` (one `decode_block` on a fresh `Scratch`), with both
arms built from the driver of `2026-11-05-deferred-flag-walk/` (rustc 1.97.0, fat LTO,
`codegen-units = 1`). The input is `wikidata-independent.bz2`, sha256 `7f5c9a0044e5a305…`.
Profiles and outputs are in `cg/` (`base-*` is master 4d4709de, `branch-*` this branch); the
listings are `build-master.asm` and `build-branch.asm`.

| block | master | this branch | change | build, master → branch | output |
|---|--:|--:|--:|--:|---|
| 0 (54 B) | 7,289 | 7,290 | +1 | 3,150 → 3,151 | identical |
| 1 | 20,184,311 | 19,716,020 | −468,291 (−2.32%) | 5,429,062 → 4,960,733 | identical |
| 41 | 19,480,138 | 19,003,853 | −476,285 (−2.44%) | → 4,804,511 | identical |
| 97 (56 B) | 7,665 | 7,666 | +1 | | identical |
| 233 | 19,989,050 | 19,509,457 | −479,593 (−2.40%) | → 4,735,803 | identical |
| 400 | 19,490,067 | 19,009,706 | −480,361 (−2.46%) | → 4,723,131 | identical |
| 472 (70 B) | 8,503 | 8,504 | +1 | | identical |

The short blocks never reach `run16`, so their +1 is code layout. This is independent of the
walk changes in #168 and counts no saving of theirs.

## The decision rule

- **Correctness (required):** CI green, Miri included; the same bytes on every corpus in every
  bench run; the build's tests pass (one-pass and two-pass agreement, and the wrong-count
  fallback).
- **Instructions (required):** `decode_block` at or below master on every full block. Met.
- **Benefit (required):** on `wikidata` at 1 or 4 threads, the CPU change or the wall change
  excludes zero on the faster side in at least one run, and the same figure's point estimate
  is not slower in the other.
- **No harm (required):** as in `2026-11-05-deferred-flag-walk.md`. A wall row flagged slower
  is settled by one more run on the same head, with at most three runs.
- **Power:** a gain of about 2% of decode instructions, in the build stage, may be below
  the bench's noise at 4 threads. If no run shows a gain beyond noise, the change is reported
  as not shown and closed, whatever its instruction count.

## Result: closed as not shown (#169)

Two runs, bench 3.2.0, head f0a768a against 4d4709d:

| run | runner | `wikidata` wall, 1 / 2 / 4 threads | CPU at 4 threads | build stage (1 thread) |
|---|---|---|---|---|
| 1 | Xeon 8573C | +0.1% / −2.4% / −0.5% (−4.2, +3.4) | −0.3% | **−3.1%** (−3.5, −2.7) |
| 2 | EPYC 9V45 | **+0.2%** (+0.0, +0.3) slower / +1.5% / −3.4% (−8.2, +1.6) | −4.0% (−8.3, +0.4) | |

- **Benefit:** not shown. The build stage itself was 3.1% faster beyond noise in run 1, but no
  `wikidata` figure was.
- **No harm:** `wikidata` wall at 1 thread was flagged slower in run 2, by +0.2%. It wasn't
  settled, because the power clause had already closed the PR.
- **What it says:** the build's 2.4% fewer instructions were real at the stage and invisible
  end to end. An inference, not measured: the build's scattered writes bound it, and the
  scalar stores this removed overlapped them.
