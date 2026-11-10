# The universal backward walk: the column's blocks walk backward too

10 November 2026. The Pathfinder (Architect). A patch for #324's successor, handed to the
Coordinator as a file because this session pushes only to #313's branch.

- **The patch:** `2026-11-10-backward-walk/backward-walk.diff.txt`, a unified diff against
  #324's head `e88c063155c3e734e748e99ccda025393302720f`. Two files, +159 −6. It applies with
  `git apply` from the repository root.
- **What it does:**
  - `bwt::build_back`: the LF table of a block's last column, written a row at a time in row
    order, with one cursor a byte and no scatter. `Segmented` then walks every block backward
    (`walk_back_room`), the column's as well as the records'. The forward build is kept for
    counts that are not the column's own, which the decoder never passes.
  - `runs::takes_runs`: admission to the records' path by a sample of the block's rows (64
    windows of 64 symbols), not by its bound. A short high-entropy block now keeps the column.
- **No new `unsafe`.** Both functions are safe Rust. `takes_runs` no longer guarantees "symbols
  at most half the bound"; `runs::front` and `front_avx2` size their buffers from the symbol
  count and the bound and refuse oversize inputs themselves, so nothing in them leans on the
  old test. That reading deserves the B5 reviewer's own check.
- **Checked here:** `cargo test --release` passes in every suite (exit 0); `cargo fmt` and
  `cargo clippy --all-targets --release` are clean; bytes equal to bzip2 on `random`,
  `records`, `text`, `pattern`, `runs`, `onebyte`, Wikidata at levels 1 and 2, and cuts of
  `random` at 200, 440 and 460 kB.
- **Counted** (callgrind, `ab child mibz2 1`, this container's toolchain), whole-child
  instructions against master 27ed84df:

  | input | #324's head | this patch |
  |---|--:|--:|
  | a `random` block (890 kB) | +0.0% | −1.0% |
  | a `records` block (890 kB) | +0.0% | −1.1% |
  | `random` cut to 200 kB | +10.3% | −0.8% |
  | `random` cut to 440 kB | +11.4% | −1.0% |
  | `random` cut to 460 kB | +0.0% | −1.0% |

- **Not timed** beyond a noisy local run. CI's bench under PERFORMANCE-POLICY decides.
