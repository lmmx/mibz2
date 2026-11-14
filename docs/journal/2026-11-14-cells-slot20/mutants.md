# Cells 20-byte slot (#391): the mutation record of its offset arithmetic

This is the B5 mutation evidence for #391, Guardian D106's closure item. It was taken on
branch `perf/cells-slot20` at `ee84010861700647aa3bac62f131dd524e4fcbd0` (`src/` identical
to `c915a95e`), on a 4-core x86-64 VM, with debug test builds. Results:

- 5 of the 6 mutants are killed by a test failure (exit 101), and the SIGSEGV mutant
  reproduces.
- M5 (`SLOT` 20 → 16) times out under both full-suite commands (exit 124): that is recorded
  as a timeout witness, not a kill. Two focused tests then fail on it deterministically (§M5).
- The baseline passes before and after the mutants, and the restored source is
  byte-identical to the committed one.

## Method

Each mutant is one textual substitution in `src/cells.rs`, applied to the committed file,
then reverted. Every mutant runs both commands, each bounded by `timeout 300`:

    timeout 300 cargo test -q --lib cells::     # the 9 Cells unit tests, Miri witnesses included
    timeout 300 cargo test -q --test cells      # tests/cells.rs: decoder parity, corruption, corpora

An exit of 0 means the mutant survived. An exit of 101 means a test failed or the test binary
died (a signal is reported by cargo as 101 with the signal named). An exit of 124 means the
timeout fired.

## Results

| # | mutant (diff in `src/cells.rs`) | `--lib cells::` | `--test cells` | outcome |
|---|---|---|---|---|
| — | baseline (no change) | 0 (9 passed) | 0 (5 passed) | passes |
| M1 | `wr_other`: `wr32(base, slot + 16, other as u32)` → `wr64(base, slot + 16, other as u64)` | 101 | 101 (0 of 5 passed) | killed (assertions) |
| M2 | `rd_other`: `rd32(base, slot + 16) as usize` → `rd64(base, slot + 16) as usize` | 101 (SIGSEGV) | 101 (SIGSEGV) | killed (signal 11) |
| M3 | both: `slot + 16` → `slot + 12` (read and write) | 101 | 101 (0 of 5 passed) | killed (assertions) |
| M4 | both: `slot + 16` → `slot + 17` (read and write) | 101 (SIGSEGV) | 101 (SIGSEGV) | killed (signal 11) |
| M5 | `const SLOT: usize = 20;` → `const SLOT: usize = 16;` | **124 (timeout, 300 s)** | **124 (timeout, 300 s)** | timeout witness; deterministic failures below |
| M6 | write only: `wr32(base, slot + 16, …)` → `wr32(base, slot + 12, …)` | 101 (SIGSEGV) | 101 (1 of 5 passed) | killed |
| M2′ | M2 again, to reproduce the SIGSEGV | 101 (SIGSEGV) | 101 (SIGSEGV) | reproduces |
| — | baseline restored | 0 (9 passed) | 0 (5 passed) | passes; the source is byte-identical |

### First failures (as the runs printed them)

- **M1:**
  - `cells_selective_exhaustive_state` panicked at `src/cells_original_test.rs:51`.
  - `cells_exhaustive_arbitrary_columns` panicked at `src/cells.rs:645` (`ahead!`).
  - `miri_cells_admission_and_invalid_records` panicked at `src/cells.rs:738`.
  - All 5 of `tests/cells.rs` failed.
- **M2, M2′, M4:** the process died: `signal: 11, SIGSEGV: invalid memory reference`, in both
  the lib and the integration test binaries.
- **M3:**
  - `cells_selective_exhaustive_state` panicked at `src/cells_selective_test.rs:39`.
  - `miri_cells_exact_fit_and_reuse` failed its output check (`src/cells.rs:986`).
  - All 5 of `tests/cells.rs` failed at `tests/common/cells.rs:51`.
- **M6:**
  - The lib binary died with SIGSEGV.
  - `cells_long_writer_decoder_parity` failed at `tests/cells.rs:40`, and
    `cells_real_decoder_admission_reach` at `tests/common/cells.rs:76`.

### M5: the timeout, then deterministic failures

Under both full-suite commands, M5 ran until `timeout 300` stopped it (exit 124). An
earlier, unbounded run of the same mutant was stopped by hand after several minutes. The
overlapping slots corrupt the induction's links, and some tests loop. So, with the same
mutant, the focused tests were run one at a time, each under `timeout 120`:

    timeout 120 cargo test -q --lib cells::tests::miri_cells_slots_mark_and_links -- --exact
        exit 101: panicked at src/cells.rs:1236, the partner-slot bound
        (`partner >= other && partner < other + kernel.letters * SLOT`)
    timeout 120 cargo test -q --lib cells::tests::miri_cells_exact_fit_and_reuse -- --exact
        exit 101: panicked at src/cells.rs:985 (`try_invert(..).unwrap()`: the forced
        inversion refused)
    timeout 120 cargo test -q --lib cells::tests::miri_cells_admission_and_invalid_records -- --exact
        exit 0 (this one does not see it)

So M5 is killed deterministically by `miri_cells_slots_mark_and_links` and
`miri_cells_exact_fit_and_reuse`, both of which are in the Miri job's required list. Its
full-suite outcome is a timeout witness.
