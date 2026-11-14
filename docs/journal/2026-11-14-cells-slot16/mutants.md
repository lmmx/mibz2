# Cells 16-byte slot (diet step 2): the mutation record

This is the B5 mutation evidence for the 16-byte slot (#145 LOG 6883380160, amended by
6883412763), on branch `perf/cells-slot16`. It covers the changed offset arithmetic and the
new code: the word's length kept at the word, the small words' headers, and the walk's row's
marked word. It was taken on a 4-core x86-64 VM with debug test builds, with `src/` as
committed in `perf/cells-slot16`'s first commit.

**Result:**
- Every mutant fails something deterministically (exit 101).
- 4 of the 10 fail their full suites outright: M3 (the lib suite; its integration suite
  timed out), M8, M9 and M10.
- The other 6 (M1, M2, M4, M5, M6, M7) time out under both full suites (exit 124 at 300 s).
  Those are recorded as timeout witnesses, not kills. Each then fails deterministically in
  focused tests run alone (§Focused).
- The baseline passes before the mutants and again after them, and the restored source is
  byte-identical.

## Method

Each mutant is one exact substitution in `src/cells.rs`, applied to the committed file and
reverted after. Every mutant runs both commands, each bounded:

    timeout 300 cargo test -q --lib cells::     # the 9 Cells unit tests (Miri witnesses included)
    timeout 300 cargo test -q --test cells      # tests/cells.rs: decoder parity, corruption, corpora

An exit of 0 means the mutant survived. An exit of 101 means a test failed or the binary died
(cargo reports a signal as 101 and names it). An exit of 124 means `timeout` fired; that is a
timeout witness.

## Results

| # | mutant | exact diff (`src/cells.rs`) | `--lib cells::` | `--test cells` |
|---|---|---|---|---|
| — | baseline | (none) | 0 (9 passed, the census test ignored) | 0 (5 passed) |
| M1 | a small word's length read at the word, not its header | `rd32(base, w - HDR)` → `rd32(base, w)` | 124 | 124 |
| M2 | a rule's length: its first factor's only | `rd32(base, w).wrapping_add(rd32(base, w + 8))` → `rd32(base, w)` | 124 | 124 |
| M3 | the rule test off by one | `if w >= ro {` → `if w > ro {` (in `word_len`) | **101** | 124 |
| M4 | no header written for a new small word | deletes `#[cfg(not(feature = "slot20"))] wr32(base, top, len as u32);` | 124 | 124 |
| M5 | a small word's offset at its header | `let word = if small { top + HDR } else { rp };` → `{ top }` | 124 | 124 |
| M6 | the walk's row's word unmarked | `copy_from_slice(&(1 \| MARK).to_ne_bytes())` → `copy_from_slice(&1u32.to_ne_bytes())` | 124 | 124 |
| M7 | `SLOT` 16 → 12 | `const SLOT: usize = 16;` → `12` | 124 | 124 |
| M8 | `OTHER_AT` 12 → 8 (the other order's slot over the word) | `const OTHER_AT: usize = 12;` → `8` | **101** | **101** |
| M9 | `WORD_AT` 8 → 4 (the word over the letter's length) | `const WORD_AT: usize = 8;` → `4` | **101** | **101** |
| M10 | `HDR` 4 → 0 (no room for the headers) | `const HDR: usize = 4;` → `0` | **101** | **101** |
| — | baseline restored | (none) | 0 (9 passed) | 0 (5 passed) |

First failures of the full suites:
- **M3:** the lib test binary did not exit successfully.
- **M8:**
  - `cells_exhaustive_arbitrary_columns` panicked at `src/cells.rs:1114` (its `try_invert(..).unwrap()`);
  - `cells_selective_exhaustive_state` at `src/cells.rs:422`;
  - `cells_long_writer_decoder_parity` at `tests/cells.rs:40`;
  - `cells_real_decoder_admission_reach` at `tests/common/cells.rs:76`.
- **M9:** the same, with the selective comparison at `src/cells_selective_test.rs:24`.
- **M10:** every Cells test panicked at `src/cells.rs:606` (the header's `copy_from_slice` into
  a zero-length slice).

## Focused: the mutants whose full suites timed out

For each such mutant, every Cells unit test was run alone under `timeout 30`:

    timeout 30 cargo test -q --lib cells::tests::<test> -- --exact

| mutant | deterministic failures (exit 101), first panic | timed out (124) | passed (0) |
|---|---|---|---|
| M1 | `miri_cells_slots_mark_and_links` (`src/cells.rs:1366`, the first word's length 1); `miri_cells_long_writer` (`src/cells.rs:1251`) | exact_fit, selective_boundaries, exhaustive_arbitrary, selective_exhaustive, fallback_and_bound | admission_and_invalid |
| M2 | `miri_cells_long_writer` (`:1251`); `miri_cells_selective_boundaries_reuse` (`src/cells_selective_test.rs`) | exact_fit, exhaustive_arbitrary, selective_exhaustive | slots_mark_and_links, admission_and_invalid, fallback_and_bound |
| M3 | `miri_cells_exact_fit_and_reuse` (`:1114`); `miri_cells_long_writer`; `miri_cells_selective_boundaries_reuse` (died); `cells_exhaustive_arbitrary_columns` (`:1114`); `cells_selective_exhaustive_state` (died) | — | slots_mark_and_links, admission_and_invalid, fallback_and_bound |
| M4 | `miri_cells_exact_fit_and_reuse` (`:1113`, its output check); `miri_cells_long_writer` (`:1249`); `cells_exhaustive_arbitrary_columns` (`:1113`) | selective_boundaries, selective_exhaustive | slots_mark_and_links, admission_and_invalid, fallback_and_bound |
| M5 | `miri_cells_exact_fit_and_reuse` (`:1114`); `miri_cells_long_writer` (`:1251`); `cells_exhaustive_arbitrary_columns` (`:1115`) | selective_boundaries, selective_exhaustive | slots_mark_and_links, admission_and_invalid, fallback_and_bound |
| M6 | `miri_cells_slots_mark_and_links` (`src/cells.rs:1377`, one mark an order); `miri_cells_long_writer` (`:1251`) | exact_fit, selective_boundaries, exhaustive_arbitrary, selective_exhaustive, fallback_and_bound | admission_and_invalid |
| M7 | `miri_cells_slots_mark_and_links` (`:1374`); `miri_cells_exact_fit_and_reuse`; `miri_cells_long_writer`; `miri_cells_selective_boundaries_reuse`; `cells_selective_exhaustive_state` (`:422`) | exhaustive_arbitrary, fallback_and_bound | admission_and_invalid |

So every mutant has at least one deterministic failure among the Cells witnesses that Miri
runs (`miri_cells_long_writer` fails on all seven). The marked word's two mutants (M6, and M1
on the header read) are caught by `miri_cells_slots_mark_and_links`, which checks that each
order holds exactly one marked letter, that its word is the walk's row's byte, and that every
first word is one byte long.
