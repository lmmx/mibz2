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

## Correction 1: the admission (after #348's first bench)

**CI on #348 at e03dbb97 against #324** (the Coordinator's reading): `text` +4.0% at 1 thread
and +5.4% at 2, slower beyond noise; `records` −1.0% at 1 thread and −3.3% at 4; `random` and
`wikidata` level. A primary corpus slower fails PERFORMANCE-POLICY rule 2.

**The cause is mine: the sample of rows is not an estimate of the rows.** A few long runs hold
most of a block's rows, and 64 windows of 64 symbols mostly miss them.
- On `text`'s 25 blocks the true figure is 0.474–0.514 symbols a row and the sample gave
  0.247–0.562: 7 blocks were refused the records' path.
- On Wikidata's full blocks the sample was wrong more often still.

**The correction:** `2026-11-10-backward-walk/correction-1.diff.txt`, against e03dbb97, one
file. Admission is #324's bound test **and** a sampled share of run symbols of at least one
in 32.
- **A proportion samples well where a heavy-tailed sum does not.** The share of run symbols,
  whole block against sample: `text` 0.370–0.377 against 0.350–0.395; `random` 0.008 against
  0.005–0.010; `records` 0.003 against 0.002–0.005.
- **The cut has room on both sides:** `random` and `records` under 0.01; `text` 0.35 or more;
  232 of Wikidata's 233 full blocks 0.249 or more, and the other one 0.104.
- **It admits a subset of what #324 admits,** so #324's guarantee to its front (symbols at most
  half the bound) holds again, and the B5 question raised with the first patch falls away.

**Checked:** every test suite passes (exit 0); `fmt` and `clippy --all-targets` clean; bytes
equal to bzip2 on every local corpus. Whole-child instructions against master 27ed84df:

| input | #324's head | corrected patch |
|---|--:|--:|
| `text`, whole corpus | −10.62% | −10.58% |
| a `random` block | +0.00% | −1.00% |
| a `records` block | +0.00% | −1.07% |
| `random` cut to 200 kB | +10.29% | −0.89% |
| `random` cut to 440 kB | +11.37% | −1.04% |

`text` is back on #324's path: the 0.04% is the sample itself.

**What the patch is now worth, stated plainly:** no primary gain over #324. It removes #324's
short-block regression and takes about 1% of instructions off high-entropy blocks, which CI
saw as −1.0 to −3.3% on `records` and nothing on `random`.

## Outcome

**#348 is closed** (the Coordinator's LOG 6765942197 on #145), by the rule it fixed before the
second bench. With correction 1, `text` is level with #324 at 1, 2 and 4 threads and Wikidata
is level, so the regression was the admission's and is gone. But no primary corpus is faster
beyond noise, so PERFORMANCE-POLICY rule 1 fails. #324 ships alone.

- **Closed:** this construction as a performance change. Writing the column's table in row
  order saves about 1% of instructions on high-entropy blocks and no measurable time on CI.
- **Kept:** the admission rule (the bound, and a sampled share of run symbols of at least one
  in 32). It removes #324's +10–11% on short high-entropy blocks, admits a subset of what
  #324 admits, and adds no `unsafe`. It stands on its own without `build_back`.
- **The lesson is mine:** I shipped an estimator without measuring it against the truth on
  the primary corpus. Checking the admitted set against #324's, block by block, would have
  taken a minute and caught it before CI did.
