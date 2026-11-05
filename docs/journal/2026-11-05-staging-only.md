# The staged walk without its deferred flags: registration

Registered before any CI timing of this branch. The decision rule may be amended only in a
commit made before the first bench comment on this PR, and each amendment says what changed
and why.

## The change

#171 merged two changes to the room walk's busy phase together:

- **The deferred flags (#168).** All 32 lanes are stepped without testing their entries'
  start flags. The flags are ORed and tested once an offset. An offset that read a start is
  done again lane by lane from the rows before it, which are kept in a second slot.
- **Staging.** The bytes go to one tile instead of 32 buffers, and are copied out when a chunk
  or a segment ends.

This branch keeps the staging and removes the deferred flags. Each offset tests each lane's
entry as master did before #168. A lane that reads a start is marked and handed over in lane
order. There is one slot of rows, no replay and no `spec_offset`. `src/bwt.rs` loses 133 lines
and gains 51. The change is research/staging-only (7c6bfd30), ported onto master: #174's
chunk-edge counters are kept, and the replay counters, which no longer exist, are dropped
from the tests.

## Why

From `2026-11-05-walk-round.md`:

- **On CI, #168 alone** (the deferred flags without staging) bought no time. Its 1-thread wall
  was flagged slower once, +1.7% (+1.1, +2.3) on an EPYC 9V74. Its walk stage was +4.0% on an
  EPYC 7763.
- **Locally, staging alone** carries the whole of #171's gain. Against #171, at 32 paired
  rounds, #171 over staging alone is:
  - 1.0075 (0.973, 1.023) at 1 thread;
  - 1.0026 (0.962, 1.050) at 4 threads;
  - 1.0295 (0.981, 1.093) in a pinned 1-thread supplement of 12 rounds.

  So staging alone was faster in every point estimate, though no interval excluded zero.
- **The walk is bound by misses in flight, not instructions**, so 4% more instructions in its
  loop may cost nothing. The replay's second slot of rows and its out-of-line `spec_offset`
  call may even cost something.

## Measured: whole-decoder instructions

Inclusive Ir of the c30cg driver's `measured` (one `decode_block` on a fresh `Scratch`;
rustc 1.97.0, fat LTO, `codegen-units = 1`), against master c9433318:

| block | master | this branch | change | output |
|---|--:|--:|--:|---|
| 1 | 18,539,867 | 19,273,484 | +733,617 (+3.96%) | identical |
| 41 | 17,896,118 | 18,647,570 | +751,452 (+4.20%) | identical |
| 233 | 18,332,436 | 19,071,853 | +739,417 (+4.03%) | identical |
| 400 | 17,907,569 | 18,651,084 | +743,515 (+4.15%) | identical |
| `runs` 0 | 17,325,992 | 17,394,484 | +68,492 (+0.40%) | identical |
| 0, 97, 472 (short) | 6,888 / 7,328 / 8,076 | same | 0 | identical |

The input is `wikidata-independent.bz2` plus CI's `runs` corpus (sha256 `af3be4d130faf312…`).
Every lib and integration test passes, and clippy and fmt are clean.

## The decision rule

The incumbent stays unless this branch wins. A neutral result keeps the deferred flags.

- **Correctness (required):** CI green, Miri included; the same bytes on every corpus in
  every bench run.
- **Instructions:** not a criterion. The branch runs about 4% more by construction, and the
  question is whether they cost time.
- **Benefit (required):** on `wikidata` at 1 or 4 threads, the CPU change or the wall change
  excludes zero on the faster side in at least one run, and the same figure's point
  estimate is faster in the other run.
- **No harm (required):** a wall row flagged slower in a run is a harm signal. One more run on
  the same head settles it: flagged again, it is a harm and the rule fails; not flagged
  again, it is reported as unresolved. At most three runs.
- **Power:** the 1-thread rows' 95% half-widths are about ±0.5–1%, so the local 1-thread
  estimate (0.7–3%) is within what CI can resolve. The 4-thread rows (±2–5%) are not. If no
  run shows a gain beyond noise, the result is reported as not shown, the PR closes, and the
  deferred flags stay.
- **What a pass establishes:** that the deferred flags cost time on CI's runners, net of the
  instructions they save. It does not establish the mechanism.
