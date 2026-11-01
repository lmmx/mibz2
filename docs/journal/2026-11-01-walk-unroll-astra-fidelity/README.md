# The unrolled room walk against Astra's assembly-level prescription

**What this is:**
- A disassembly comparison. Nothing is timed. Production code, the frozen trial branches
  and the registered verdicts are unchanged.
- **The binaries:**
  - **The timed arm:** the stage trial's exact timed `walk_arm` (CI run 41222735026,
    SHA-256 `48ddfd16…`, kept on `research/walk-unroll`).
  - **A local rebuild of it** (rustc 1.97.0, `--locked --release`, no `RUSTFLAGS`). Its
    `room_segments_unrolled::<32, 1>` matches the timed binary's instruction for
    instruction (5,885 instructions, with no difference once addresses are stripped).
  - **A scratch variant** with Astra's shared fallback (`shared-fallback.patch`), built
    the same way, to read its code. It is not timed and not pushed as a branch.
- **Provenance:** `research/walk-unroll` was implemented from Astra's analysis (pack
  `c4018b5`). Its protocol, §2, chose "each step the shipped body (`continue` leaves its
  step)", not Astra's `break 'ordinary k` into one shared fallback. This note checks what
  that choice cost.

## Signature by signature

| Astra's prescription | Timed arm | Shared-fallback variant |
|---|---|---|
| One fully unrolled round, 32 lanes at constant offsets | **achieved**: 32 T loads; every row and base at a constant `[rsp+disp]` | achieved |
| No per-lane index arithmetic, comparison or backedge | **achieved**: the ordinary step is 10 instructions, 9 macro-ops, with no lane counter, pointer or bound | achieved |
| A shared scalar fallback from the first boundary lane | **missed**: 32 inline copies of the boundary handler (finish, take, the mid-pass end), each re-entering the round at lane k + 1 through a tail-duplicated copy of that lane's head | **achieved**: 32 cold stubs, `mov r10d,k; jmp fallback`, into one loop that is the shipped 16-instruction step, verbatim |
| Traversal and prefetch semantics kept | achieved | achieved |
| Ordinary step ≈ 14.4 → 9 macro-ops | **achieved**: 9 a step; 9.53 a lane-step over the whole round | 9 a step; 9.34 a lane-step |
| No significant extra spills | **small miss**: +4 spill stores and 1 reload a round (three steps' bases kept in registers, spilled for the cold blocks); the round counter and `room` go through memory | spills gone; but the T-vector base is **reloaded from the stack** in the last five steps (+6 loads a round) |
| No hot-path fallback overhead | achieved (one `mov r10,r9` a round) | achieved |
| No loop rerolling | achieved | achieved |
| Code size | 5,885 instructions (29.7 KB); the round is in 3 pieces joined by 3 `jmp`s | 1,904 instructions (9.8 KB); the round is contiguous, with no `jmp` |

## The hot round, measured from the disassembly (`trace_round.py`)

| Per lane-step | Shipped (`room_segments`) | Timed arm | Shared fallback |
|---|--:|--:|--:|
| Instructions | ≈ 16.4 | 10.44 | 10.38 |
| Macro-ops (cmp/test + jcc fused) | ≈ 14.4 | 9.53 | 9.34 |
| Loads | 3.0 | 3.06 | 3.19 |
| Stores | ≈ 2.03 | 2.16 | 2.00 |
| Prefetches | 1 | 1 | 1 |
| Branches | ≈ 2.03 | 1.12 (3 of them `jmp`) | 1.03 |

## Reading (deduced, with the static floors labelled as a model)

**The hot path:**
- The timed arm already is Astra's instruction sequence. The ordinary step is the
  10-instruction, 9-macro-op body the pack predicted.
- The measured stage gain (one copy, EPYC 7763, B/A 0.877 [0.863, 0.891]) is about
  0.55 c/B on the 4.49 c/B basis. That is inside Astra's 0.3–0.6 c/B estimate.

**What the shared fallback changes in the round:**
- −0.19 macro-ops, −0.16 stores and +0.13 loads a lane-step; the memory operations are
  unchanged in total (6.22 against 6.19).
- **Static floors (model):**
  - **Zen 3:** memory operations, at 3 a cycle, bind both forms, at 2.07 against
    2.06 c/step.
  - **Comet Lake:** ports 2 and 3 carry the loads, the prefetch and the indexed byte
    store's address, so they bind. That floor is 2.53 against 2.60 c/step: slightly
    worse with the shared fallback, because of the T-base reloads.
- **Neither is a material hot-path improvement.**

**What the shared fallback costs at boundaries:**
- After a boundary at lane k, the rest of that round, lanes k to 31, runs at the shipped
  14 macro-ops a step instead of 9.
- That is about 80–90 macro-ops for each boundary event.
- **Per round, at 512 segments a block (model):**
  - a 900,000-row block has about 0.018 events a round, which is +0.5%;
  - a 100,000-row block (bzip2 -1) has about 0.16, which is +5%;
  - a block at `MIN_SEGMENTED` has about 0.25, which is +7%.
- The timed arm's tail-duplicated re-entry avoids this cost. That is why its cold
  structure, though larger, suits short blocks better.

**`records`:**
- Its blocks are full-size, so boundaries are rare there, and the cold blocks are touched
  a few hundred times a block.
- So neither the fallback structure nor the code-size difference plausibly accounts for
  the `records` regressions of 5–8% on three CI hosts. That question lies outside the
  walk's hot path.

**Conclusion:**
- Astra's prescription is what the arm implements. The one structural element the arm
  departed from, the shared fallback, doesn't improve the hot path when built: LLVM
  trades the spills for T-base reloads. It also adds per-boundary cost on short blocks.
- No source change is recommended, and nothing is proposed for timing.

## Correctness of the scratch variant (measured)

- `cargo +1.97.0 test --release --lib -- unrolled room_walk segmented`: 6 of 6 pass.
  This includes `unrolled_room_walk_equals_plain`, with a boundary and a take at every
  lane index.
- Miri was not run.
