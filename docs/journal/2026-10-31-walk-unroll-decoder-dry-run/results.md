# The unrolled room walk's full-decoder trial: the dry run up to timing (preparation VM)

**The run (measured):**
- `WALK_STOP_BEFORE_TIMING=1 just bench-walk-decoder vm-prep`, at head `c647c8d`
  (pin `30018de`).
- rustc 1.97.0. The binary's SHA-256 is `251703c5…7b35b`.
- 4-core VM with no SMT: allocation CPUs 0–3, workers 1–4.
- **Nothing was timed:** the run stops before the trial. The self-test's figures are
  synthetic.

**Every check before timing passed:**
- The 20 harness sources match their frozen hashes.
- The 11 corpora match `frozen.json`, the 7 shared ones identical to the P74-C trial's.
  `bzip2 -t` and `bzip2 -dc` match the frozen output hashes.
- `--check`: both arms decode every corpus to the frozen SHA-256 at w = 1, 2, 3 and 4,
  and single-threaded with each kernel. The thread inventory is exactly w + 3 in all
  88 decodes (11 corpora × 2 arms × 4 w).
- **The arm's tests pass:** `--lib unrolled` (3 tests) and `--test segmented_unrolled`
  (3 tests).
- **The structural audit** (`asm-audit.json`): W1, W2, W3, W4 (a) and W4 (b) pass.
  **W4 (c) is review required**, with the same signature as `freeze.md` point 3:
  `decode_block::<Segmented>`, `Segmented::invert` and `SegScratch::assemble` are
  inlined differently in the two arms.
- **The harness self-test with synthetic times** ran the whole trial path: children,
  `wait4`, affinity and the summary.
  - Its verdict reads NEGATIVE, as synthetic times force it to. Every B/A time ratio is
    exactly 1.000, so the primary's upper bound is below 1.02.
  - Its RSS rows are real (`ru_maxrss` isn't synthetic) on small reused files.
  - None of it is a result.

**The timed trial is blocked, by design,** until two things happen:
1. the maintainer records the W4 (c) review in `frozen.json` (`w4_review`);
2. a separate authorisation is given (CI exploratory, then the i9).
