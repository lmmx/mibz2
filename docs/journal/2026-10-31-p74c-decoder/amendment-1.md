# P74-C full-decoder trial: amendment 1, harness v1.1 (a defect in `--check`; before any confirmatory timing)

**What failed (measured):**
- In the exploratory CI run 41196164480, jobs r1 and r2 stopped in `--check`, untimed and
  before any timing. The failure was at `examples/p74c_decoder.rs:701`: "w workers, the
  scanner, the feeder and main", left 6, right 5 (r1, w = 2) and left 5, right 4 (r2, w = 1).
- Both failures were on one-block adversarial files (after `edge-l9-s16383` in r1 and
  `hug-over-p31` in r2).
- r3 passed `--check` and the whole timed run.

**The cause (measured: the extra thread identified):**
- An instrumented scratch build of the frozen harness (not committed) printed every listed
  thread's tid, state and record on a mismatch. It ran 40 `--check` runs on 30 one-block
  files, at w = 1, 2 and 4.
- In both captured mismatches, the extra thread is **the previous decode's feeder**:
  - `t1a`, arm A, w = 2: `19813:R:p74c-feeder|0-3|…` is still running (state R). Its tid
    sits between main (19810) and the new decoder's workers (19814, 19815). The new
    scanner (19816) still carries the parent's name, as it hasn't started yet. The new
    feeder's own record is the separate `thread-self` entry, added after the snapshot.
  - `t6b`, arm B, w = 2: `6011::||||` is a task directory still listed whose `status` and
    `stat` were already gone: a task being released.
- **Why it can still be listed:**
  - The feeder runs inside `std::thread::scope`. `scope` returns when every scoped closure
    has finished, not when the OS threads have exited; a handle not joined explicitly is
    dropped and its thread detached.
  - So the feeder can still be exiting, or being released, when the next decode takes its
    snapshot.
  - The decoder's own threads are joined (`dec.join()`) and never appeared extra.
- **Not a scheduling discrepancy:**
  - every captured record was within its allowed CPUs;
  - r3's timed children all passed the allocation check (`all_threads_within_allocation`);
  - a timed child snapshots before it spawns its feeder, in a fresh process, so no previous
    feeder exists there.
- **Reproduction:** on the preparation VM, with the same manifest and 45 runs of `--check`
  per binary:
  - the frozen binary failed 3 times, each at line 701;
  - the amended binary failed 0 times.

**The change:**
- `--check` waits before each decode until `/proc/self/task` lists only main, bounded at
  5 s. If the 5 s pass, it panics, so a thread that really outlives its decoder still
  fails the check, now more strictly.
- **Nothing else changes:**
  - the timed children, which make one decode in a fresh process, run the same code as
    before;
  - the thread assertion (w + 3) is the same;
  - the protocol, corpora, schedule, gates and kernel are unchanged.
- The frozen source hashes that change are `examples/p74c_decoder.rs` and
  `bench/p74c_decoder_run.sh`, plus the new `text_source_commit` field. No corpus hash
  changes.
- The binary's SHA-256 changes with it. The structural audit runs on the exact timed binary
  as before.

**Status:**
- The exploratory figures from r3 (`../2026-10-31-p74c-decoder-ci-41196164480/`) used
  the frozen binary. They are not affected, since the check passed there.
- No confirmatory (i9, w = 4) timing has been made at either pin.

**A second defect, exposed by the first fix (measured):**
- The first v1.1 validation run (CI 41198123877) stopped on all three jobs before timing,
  on `AssertionError: text: compressed hash`.
- The `text` control is built from this crate's own top-level files in `src`, `tests` and
  `examples`, plus `README.md`. That is why the protocol says "at the pinned commit". So
  editing `examples/p74c_decoder.rs` changed a control corpus, and the frozen hash check
  stopped the run, as designed.
- **The correction keeps the corpus, not the hash check:**
  - `--prepare DIR TEXT_ROOT` builds `text` from a source tree. The algorithm is
    `common::corpora`'s, copied into the harness.
  - The runner extracts that tree with
    `git archive <text_source_commit> src tests examples README.md`.
  - `frozen.json` gains `text_source_commit` = `535296c` (the v1.0 pin).
  - **Checked on the preparation VM:** `--prepare` reproduces all four controls'
    frozen compressed SHA-256 exactly: random `10d444e6…`, records `ff8b5f6e…`,
    mixed-independent `11639f35…`, text `59236f2d…`.
- So the `text` corpus is byte-identical to v1.0's, and no corpus hash changes.

**Versions and records:**
- **Harness v1.0**, the original frozen harness, is pin `535296c` (head `4acf46f`). It
  stays in history unchanged, with the failed jobs' artifacts in
  `../2026-10-31-p74c-decoder-ci-41196164480/r1`, `r2` and `r3`.
- **Harness v1.1** is this amendment, pinned by `bench/p74c-decoder-pin`.
- CI run 41196164480 stays recorded as **2 of 3 jobs stopped in `--check`**. It isn't
  relabelled.
- **Validation:**
  - v1.1 is validated by a local dry run up to timing, then a new exploratory CI run of three jobs, with `--check` and timing
    at w = 1 and 2. It must complete all three before the i9 run.
  - The MTF kernel, the selector, the thresholds, the corpora and the primary corpus are
    unchanged.
