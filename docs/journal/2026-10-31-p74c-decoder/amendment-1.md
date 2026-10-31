# P74-C full-decoder trial: amendment 1 (harness defect in `--check`; before any confirmatory timing)

**What failed (measured):**
- In the exploratory CI run 41196164480, jobs r1 and r2 stopped in `--check`, untimed and
  before any timing. The failure was at `examples/p74c_decoder.rs:701`: "w workers, the
  scanner, the feeder and main", left 6, right 5 (r1, w = 2) and left 5, right 4 (r2, w = 1).
- Both failures were on one-block adversarial files (after `edge-l9-s16383` in r1 and
  `hug-over-p31` in r2).
- r3 passed `--check` and the whole timed run.

**The cause (deduced, then reproduced):**
- `--check` makes many decodes in one process. `pthread_join` returns once the kernel
  clears the exiting thread's tid. That happens before the kernel releases the task, so a
  joined thread can still be listed in `/proc/self/task` for a moment.
- The next decode's thread snapshot can then count it.
- **Reproduction:** on the preparation VM, with 30 one-block files, workers 1, 2 and 4, and
  45 runs of `--check` per binary:
  - the frozen binary failed 3 times, each time at line 701;
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
- The only frozen hash that changes is `examples/p74c_decoder.rs` in `frozen.json`,
  `5dc69a49…` → `d238c9e4…`.
- The binary's SHA-256 changes with it. The structural audit runs on the exact timed binary
  as before.

**Status:**
- The exploratory figures from r3 (`../2026-10-31-p74c-decoder-ci-41196164480/`) used
  the frozen binary. They are not affected, since the check passed there.
- No confirmatory (i9, w = 4) timing has been made at either pin.
