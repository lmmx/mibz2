# P74-C decoder trial, harness v1.1 first attempt: CI run 41198123877 (stopped before timing)

**The run (measured):**
- Head `c4f5fa7`, pin `6d22929`.
- All three jobs stopped in the corpus manifest, before `--check` and before any timing,
  on `AssertionError: text: compressed hash`.

**Why:**
- The `text` control is built from the crate's own `src`, `tests` and `examples`.
- The first v1.1 commit edited `examples/p74c_decoder.rs`, so the corpus changed, and the
  frozen hash check stopped the run, as it should.
- The correction builds `text` from the v1.0 pin's tree (`text_source_commit` = `535296c`).
  It is described in `../2026-10-31-p74c-decoder/amendment-1.md`.
- No figures were produced. The jobs' `run.log` and `files.txt` are kept here.
