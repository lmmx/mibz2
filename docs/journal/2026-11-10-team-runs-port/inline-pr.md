# For the maintenance PR: title and body

**Title:** perf(bwt): keep the one-core walk's joins inline, so decode_block's machine code is 0.2.8's

**Patch:** `team-runs-3-over-2.diff.txt` beside this file (SHA-256
`988bba1caa0aadf5bbcc3dffb89594b022cf54816fd6decf5dfa3ee60e24d37a`). It applies unchanged to
master `6782c0fbf`. `src/bwt.rs` alone, 12 lines, safe code.

**What it does.** #359 made the teams further callers of `SegScratch::assemble_reversed` and
`bwt::walk_back`. The compiler then stopped inlining them into `walk_back_room` and inlined
`walk_back_room` into `decode_block` instead, whose frame grew from 0x48 to 0x58 bytes (563
instructions to 574). The one-core path was not meant to change. Here the teams call an
out-of-line `assemble_reversed`, and `walk_back_room` keeps its join and its short-block
walk inline.

**Checked** (this container's toolchain, rustc 1.97; the bench harness's binary; tables in
`codegen/`):

| | `decode_block::<Segmented>` | `walk_back_room` |
|---|---|---|
| before the port (`910472e8`) | 563 instructions | 342 |
| master `6782c0fbf` | 574 | inlined away |
| master with this patch | 563, identical to the first row | 342, identical |

With the patch, the only functions off the team paths that still differ from `910472e8` are
`runs::fill` (the same instructions, a cold block moved), `fill_avx2` (two more at its
edges, outside its loop) and `ErrorKind`'s two formatters.

`cargo fmt --all --check`; `cargo clippy --all-targets --features trace,wait-stats,walk-stats
-- -D warnings`; `cargo test` and `cargo test --release`: clean and exit 0 on master
`6782c0fbf` with the patch.

**What decides it.** 0.2.9's release-scaling run on the i9 has `wikidata` at 1 thread 0.9%
slower than 0.2.8 (0.991, 0.983 to 0.998). CI's 1-thread bench on this PR decides whether
this is the cause: `wikidata` at 1 thread level with or faster than master, and nothing
slower beyond noise. If it is not, the remaining differences are where the code sits in the
binary and the two fill functions above.

**Not shown:** that the frame's growth caused the 0.9%. At one thread the decode runs on
the main thread, whose stack offset is drawn per launch, so a 16-byte shift should not move
a mean there; the inlining of `walk_back_room` into `decode_block` is the other half of the
change and is also undone.
