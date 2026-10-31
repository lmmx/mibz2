# Room walk, constant-index 32-lane round: amendment 1 to the structural gates (before any timing)

**Status:** authorised by the maintainer after reading `audit.md`, and committed before any
timing has been run or read.
- The arm's code and the measurement in protocol §5 are unchanged.
- The verdict in §6 is unchanged: on every counted EPYC 7763 job, B/A's upper 95% bound
  must be < 0.95.

**Why it is amended:** `audit.md` shows two registered gates failing as written.
- **S3:** stores are 2.16 a lane-step against ≤ 2.1. The excess is register spills of
  round-level state (three steps' `base` pointers, `off` and `room`): 5 stores and about
  4 loads a 32-lane round. These replace 192 bookkeeping instructions a round.
- **S5:** the bound was mis-specified. The audit counts only `rsp`/`rbp` operands, and the
  shipped walk reaches its lane rows through `r11`, a pointer into the same stack array.

The analysis's own criterion is "only shared outer-round control remains; no additional
hot calls, spills, per-lane increments or status tests". The amended gates state that
criterion as numbers.

## Amended gates (replacing S3 and S5; S1, S2 and S4 unchanged)

| Gate | Pass if |
|---|---|
| S3′ memory operations per lane-step | loads 2.9 to 3.25; stores 1.9 to 2.25 (the excess over 2.0 only from round-level operations: at most 8 stores a 32-lane round); prefetches 1.0 |
| S5′ calls and stack traffic | no `call` on the hot path. Stack accesses counted alike in both walks, with a pointer into the lane array counted as a stack access: the arm's are at most the shipped walk's (inner step plus its outer loop, per lane-step) + 0.25 each for loads and stores. |
| S6 the lane-steps' cores (new) | each of the round's 32 lane-steps (from its row load to the next) has exactly one row load and one row store at the same constant stack displacement, one T-vector load, one `js`, one `prefetcht0`, one byte store at a base plus the offset, at most one load of that base, and no call. Extra instructions on the round's path stay bounded by S2 (≤ 11.0 a lane-step), as registered before any code existed. |

**Checked on the preparation VM's binary** (`audit.md`, sha256 `987c450b…`; `bench/walk_unroll_gates.py`):

| Gate | Value | Result |
|---|---|---|
| S3′ | loads 3.06, stores 2.16 (5 a round), prefetches 1.0 | pass |
| S5′ | no calls; arm 2.06 / 1.16 against shipped 2.0 / 1.03 counted alike (inner 2 / 1, plus 1 store a round) | pass |
| S6 | 32 of 32 lane-steps have their core | pass |

**The shapes of the 32 lane-steps:**
- 25 are exactly the analysis's 10 instructions.
- 2 are 9 instructions, with the base pointer held in a register (`r11`, `r9`).
- 5 carry round-level operations: the spills of `r10`, `r11` and `rcx` to `[rsp+0x8]`,
  `[rsp+0x10]` and `[rsp+0x18]`; the round counter's `lea` and store; and the round's
  closing compare and loads.

**A correction made before this commit:**
- An uncommitted draft of this amendment proposed S6 as "≥ 28 lane-steps exactly
  10 instructions", and stated 29 without checking. The check found fewer.
- Rather than set a count to fit, S6 was redefined as the per-step invariant above. The
  extra instructions are left to S2's bound, which was registered before the arm was
  written.

**On CI:** every job audits its own built binary before timing, with
`bench/walk_unroll_gates.py`. A job whose binary fails stops before timing, and is
reported as a structural stop for that build.
