# RLE1 over the walk's segments, the CRC folded in: registration

Registered before any CI timing of this branch. The decision rule may be amended only in a
commit made before the first bench comment on its PR, and each amendment says what changed
and why.

## The change

At master (0769b52f), after the room walk a block's bytes take three passes:

1. `SegScratch::assemble` follows the walk's segments in order and copies them into `ll`
   (read n, write n).
2. `rle::unrle` reads `ll` and writes the output (read n, write m).
3. `crc::crc32` reads the output back (read m).

That is 3n + 2m logical bytes, for n rows and m output bytes. This branch makes it n + m:

- **`rle::unrle_chain`** undoes RLE1 straight from the segments, in order, where they lie.
  A run that crosses a segment boundary goes through a 72-byte stitch. The kernel is one
  `Step` trait with an AVX2 step and a scalar twin. The contiguous `unrle_avx2` now runs on
  the same step.
- **`crc::Running`** folds the CRC in as the output grows, once 32 KiB is pending, while those
  bytes are still in cache. `crc::update_with` continues a CRC from any state.
- **`InverseBwt::invert_split`** returns `Inverted::InPlace` (the kernel's output is over
  `ll`, as before) or `Inverted::Chain` (the segments, for `unrle_chain`).
  `SegScratch::chain_pieces_at_most` bounds the pieces; a walk with more pieces is assembled
  as before. `decode_block` routes through it; the cooperative kernel's path is unchanged.

Six files change, +988 −43, including 8 new `unsafe` lines in `rle.rs`. The walk, staging,
prefetch policy, representation and window (#180's 4 × threads) are as on master.

## Why

Astra's REVIEW 6651150258 (#145; report at 6b720b24, `reviews/inbox-6651023083/report.md`)
ranks this fusion first. It is priced on the memory traffic shared by four workers on two
physical cores, not on instructions, which have mispredicted the walk's time (#181).

## Measured: instructions and simulated traffic

Inclusive counts for the c30cg driver's `measured` (one `decode_block` on a fresh `Scratch`;
rustc 1.97.0, fat LTO, `codegen-units = 1`), against master d1fd4405 (0769b52f has the
same `src/`).

- **Arms:**
  - *base* is master.
  - *refactor* is the `Step` refactor of the contiguous kernel alone.
  - *head* is this branch.
- **Inputs:** `wikidata-independent.bz2` plus CI's `runs` corpus (sha256
  `af3be4d130faf312…`).
- **Output:** identical on every block.

**Instructions:**

| block | base | head | change | of which refactor | of which fusion |
|---|--:|--:|--:|--:|--:|
| wd 1 | 19,273,427 | 18,987,897 | −285,530 (−1.48%) | −168,884 | −116,646 |
| wd 41 | 18,647,555 | 18,374,741 | −272,814 (−1.46%) | | |
| wd 233 | 19,071,830 | 18,815,536 | −256,294 (−1.34%) | | |
| wd 400 | 18,651,061 | 18,405,032 | −246,029 (−1.32%) | | |
| `runs` 0 | 17,394,476 | 16,928,356 | −466,120 (−2.68%) | −507,439 | **+41,319** |

**Lines missing a simulated L2:** Callgrind's cache simulator, with D1 32 KiB 8-way and LL
set to one core's L2, 1 MiB 16-way. Counts are misses (reads + writes).

| block | phase | base | head | change |
|---|---|--:|--:|--:|
| wd 1 | post-walk | 67,396 (4.31 MB) | 28,727 (1.84 MB) | −57% |
| wd 1 | whole `decode_block` | 839,609 | 800,940 | −4.6% |
| wd 233 | post-walk | 67,630 (4.33 MB) | 28,747 (1.84 MB) | −57% |
| `runs` 0 | post-walk | 324,118 (20.7 MB) | 161,714 (10.3 MB) | −50% |
| `runs` 0 | whole `decode_block` | 372,197 | 209,834 | −44% |

The refactor alone leaves the simulated traffic unchanged; the fusion is all of the fall.
On `runs` the fusion adds instructions and halves the traffic: that is the trade it makes.

**Tests:**

- Every lib and integration test passes, `--features trace` included.
- Clippy and fmt are clean.
- Miri runs the `miri_chain_*` tests on both the scalar target and the AVX2/PCLMULQDQ target.

**Astra's controls** each have a test in `tests/in_place.rs` or the lib tests:

- runs and counts split at every boundary position;
- empty and tiny blocks;
- repeated cycles on one `Scratch`;
- high expansion;
- reuse after an error. On crafted corrupt blocks that take the chain (`RunUnfinished`, a
  wrong stored CRC), the error and the truncated output match those of the `Plain`
  kernel's separate-buffer path, on one thread and through the 4-worker reader.

Local timing on the shared machine was uninformative.

## The decision rule

The incumbent stays unless this branch wins. A neutral result keeps master's three passes.

- **Correctness (required):** CI green, Miri included; the same bytes on every corpus in
  every bench run.
- **Instructions and simulated traffic:** not criteria. They say where the change acts, not
  what it saves in time.
- **Benefit (required; Astra's matched four-thread condition):** on `wikidata` at 4 threads,
  the wall change excludes zero on the faster side in at least one run, and its point
  estimate is faster in the other run. A 1-thread or CPU gain alone does not pass: the
  hypothesis is about traffic shared by four workers on two cores.
- **No harm (required):** a wall row flagged slower in a run, on any corpus and at any thread
  count, is a harm signal. One more run on the same head settles it: flagged again, it is a
  harm and the rule fails; not flagged again, it is reported as unresolved. At most three
  runs. The named failure mode is a loss of vector RLE1 or bulk-CRC efficiency; `runs`,
  where the fusion adds 41k instructions a block, is where it would show.
- **Attribution (required for the whole branch to merge).** Two runs from the Actions tab,
  with the same harness and 10 rounds:
  - *fusion alone*: this head against the refactor arm, a commit that differs only in
    `src/lib.rs` (`decode_block` as on master);
  - *refactor alone*: the refactor arm against master.

  The whole branch merges only if the fusion-alone run's `wikidata` 4-thread wall point
  estimate is faster and it flags no wall row slower. Otherwise the fusion is reported as
  not shown, and the refactor alone may become its own candidate, under its own
  registration and fresh runs.
- **Not built:** Astra's CRC-on-output-alone arm and the assembly-to-RLE1-alone arm. The
  CRC fold sits inside the chain loop, so neither separates without new code. The
  fusion-alone run measures the two together.
- **Power:**
  - The 4-thread rows' 95% half-widths have been about ±2–5%. #181's 4-thread gain of
    −3.2% (−5.6, −0.7) was resolved; a gain much under 2% will probably not be.
  - If no run shows a 4-thread wall gain beyond noise, the result is reported as not
    shown and the PR closes.
- **What a pass establishes:** that removing these passes saves 4-thread wall time on CI's
  runners. That the saving comes from the shared traffic is an inference: no hardware
  counters are read.

## Result: not shown at 4 threads; #183 closed

Two PR runs on head beef4e8f against master 0769b52f, and two dispatched attribution runs,
all bench 3.2.1 at 10 rounds. The summaries are in `2026-11-05-walk-round/bench-runs/`.

| run | runner | `wikidata` wall at 1 / 2 / 4 threads | wall rows flagged slower |
|---|---|---|---|
| PR 1 | EPYC 7763 | **−1.1%** (−1.6, −0.7) / −2.4% / −4.2% (−10.0, +1.8) | `runs` 4t, +1.3% (+0.2, +2.4) |
| PR 2 | EPYC 9V45 | **−2.5%** (−4.1, −0.9) / **−2.3%** / +1.5% (−3.9, +7.2) | none |
| fusion alone (head against the refactor arm) | Xeon 8370C | **−2.3%** / **−2.7%** / **−1.5%** (−2.4, −0.6) | `text` 2t, +3.4% (+0.0, +6.8) |
| refactor alone (the arm against master) | EPYC 7763 | −3.5% / −1.3% / +1.8% (all within noise) | none |

Every arm gave the same bytes on every corpus. CI was green, Miri included, on Linux,
aarch64, macOS and Windows.

- **Benefit:** failed. The 4-thread wall was not faster beyond noise in either PR run, and its
  point estimate was slower in run 2.
- **No harm:** unresolved. `runs` at 4 threads was flagged in run 1 and not in run 2.
- **Attribution:** failed as registered. The fusion-alone run flagged `text` at 2 threads.
  The clause gave no further run to settle a flag, a gap in how it was written. Even had it
  passed, the benefit criterion had already failed.
- **What it says:**
  - By Astra's own refutation clause ("if line traffic falls but wall does not, the removed
    traffic was not the binding cost at that point"), the post-walk traffic is not shown to
    bind the 4-thread wall on two SMT cores. The fusion halves the simulated L2 misses of
    the post-walk phase, and the 4-thread wall does not follow.
  - Measured but not registered: the whole branch was faster beyond noise at 1 thread in
    both PR runs, and the fusion alone at 1, 2 and 4 threads in its one run.
  - The registration required the 4-thread wall, so these gains decide nothing here. Any
    claim on them needs a registration of its own, judged only on fresh runs.
- **The stage table** (`examples/phases.rs`) times segment assembly, RLE1 and the CRC as
  separate library calls. It cannot see the fusion, so its rows say nothing about this
  change.
