# Room walk, constant-index 32-lane round: pre-registered protocol

**Status:** registered before any arm code exists. Authorised by the maintainer as one bounded
research implementation. It follows the read-only analysis of
`docs/research/2026-10-31-astra-room-walk/` (commit `c4018b5`) and
`readiness.md` (commit `1121038`).

**Out of scope:**
- the P74-C trial;
- production dispatch and defaults;
- the T-vector format, the allocator and the scheduling;
- the i9;
- merges and releases.

## 1. Hypothesis

H: in `bwt::room_segments::<32, PF_L1>`'s full-lane pass, expanding each round's
`for k in 0..L` lane loop into constant-index steps removes the loop's per-step
bookkeeping while keeping the following unchanged:
- lane order within a round (0..L, every lane once before any lane again);
- the memory-backed lane state (`lanes`, `base`);
- the one-hop prefetch;
- the immediate `START` handling.

It lowers the complete walk's cost by at least 5%.

**What is already known (measured):**
- The shipped step is 16 instructions and 14 macro-ops: 3 loads, 2 stores, 1 prefetch and
  2 conditional branches per lane-step (`0x2a530–0x2a563`).
- Six of those instructions are lane-loop bookkeeping.
- The read-only analysis found no evidence that the 32 chains serialise.

**Prediction (model, from that analysis):** 4.49 → 4.0–4.3 c/B on the EPYC 7763 (run
40579732655's basis).

## 2. The arm

- **Where it lives:** `room_segments_unrolled<L, PF>`, a copy of `room_segments` whose
  only change is the full-lane pass's inner lane loop, expanded by a macro into
  constant-index steps `k = 0, 1, …` (each guarded by `k < L`, which folds at compile
  time).
- **Each step's body** is the shipped body: the same loads, stores, prefetch and `START`
  path, with `continue` becoming an exit from that step.
- **Unchanged:** the setup, the pass's room computation, the rebasing, the tail and the
  assembly.
- **Reachable only** through `bwt::walk_segments_room_unrolled_with_prefetch::<L, PF>`
  (doc-hidden), from the trial's harness, the tests and the sweep.
- `room_segments`, `Segmented` and every production path are untouched.

## 3. Correctness (before any timing; all must pass)

1. **The unit tests:** `room_walk_short_cycles`, `room_walk_crafted_permutations` and
   `segmented_walks_on_any_length`, extended to the arm. Each checks it against the plain
   walk and the safe segment oracle.
2. **New cases** for the arm's edges, each asserted against the plain walk, at L = 32 and
   L = 8:
   - a segment boundary at every lane index in a round;
   - the last segment ending mid-pass (`FULL_PHASE_ENDS_MID_PASS`);
   - a mid-pass take (`MID_PASS_TAKES`);
   - growth at a pass's start (`GROWS`) and in the tail;
   - ns < L (the full-lane pass never entered).

   The `transition!` counters assert that each was reached.
3. **Miri:** `miri_room_walk_handoffs` extended to the arm, with `cargo miri test --lib --
   miri_`.
4. **On real blocks:** the harness checks the arm's output against the plain walk on
   every Wikidata block before timing, and stops on any difference.

## 4. Structural audit (before reading any timing; on the exact timed binary)

**The method:** `bench/walk_asm_audit.py` (commit `1121038`), on the trial harness binary.
It reads the hot loop of `room_segments_unrolled::<32, 1>`.

**All of these must hold:**

| Gate | Expected | Pass if |
|---|---|---|
| S1 unrolled | 32 T-vector loads in the hot loop | `t_loads` = 32 |
| S2 instructions per lane-step | ≈ 10 | ≤ 11.0 |
| S3 memory operations per lane-step | 3 loads, 2 stores, 1 prefetch | loads 2.9–3.1, stores 1.9–2.1, prefetches 1.0 |
| S4 branches per lane-step | the `START` test, plus the round's back-edge amortised | conditional branches ≤ 1.1 |
| S5 no calls, no extra spills | no `call` in the hot loop; stack loads and stores per step ≤ the shipped walk's (1.0 and 0.0) | as stated |

- **If any gate fails, stop:** the arm is recorded as "LLVM did not produce the
  structure". Nothing is timed, and no alternative implementation is tried.
- **Report:** the shipped walk is audited in the same binary.

## 5. Measurement

**The harness:** `examples/walk_arm.rs FILE ROUNDS GHZ`, after `examples/sweep.rs`.
- It decodes the file block by block, in decode order (scanner, MTF, build, the walk
  timed, RLE1), as `decode` does on one thread.
- Each timed call is the complete walk: `walk_segments_room_with_prefetch` or the arm's
  equivalent, so setup, passes, tail and assembly are all included.
- **Three arms:** A, the shipped `room_segments::<32, PF_L1>`; B, the arm `::<32, PF_L1>`;
  and A′, the shipped walk again, as the A/A null. All have 512 segments.
- **20 rounds.** Round r runs the arms in permutation r mod 6 of (A, B, A′).
- A round's figure per arm is its walk time summed over the blocks.

**Figures per arm:**
- the median c/B at the reported clock, and ns/B;
- B/A and A′/A as geometric means of the per-round ratios, with two-sided 95%
  t-intervals on the logs (df 19).

**The host:**
- a 3-job matrix on GitHub `ubuntu-24.04`;
- one copy, `taskset` to CPU 1, so one physical core with its sibling idle;
- the binary built once a job with `--locked`, rustc recorded, audited (§4), then timed.
- **Secondary, reported only:** the same harness with a copy on every CPU.

## 6. Verdict (fixed now)

**The decision set** is every job whose CPU is an AMD EPYC 7763 (Zen 3).
- If a batch of 3 has none, run up to 2 more batches.
- If there is still none, the verdict is given on the jobs present and marked "not on
  Zen 3".
- **A job is void** if its A′/A interval excludes 1. Void jobs are reported, and not
  counted.

**Outcomes:**

| Outcome | Condition |
|---|---|
| **PASS** | correctness §3 and structure §4 pass, and on every counted Zen 3 job the B/A upper 95% bound is < 0.95 |
| **NEGATIVE** | any counted Zen 3 job has a B/A lower 95% bound ≥ 0.95 (a saving of 5% or more is excluded), or correctness fails |
| **STRUCTURAL STOP** | §4 fails |
| **INCONCLUSIVE** | otherwise |

The other CPU models are replication, reported but not gated.

**After any outcome:** no retuning and no second variant.
- **PASS** proposes a separate, pre-registered decoder-level trial.
- **NEGATIVE** and **STRUCTURAL STOP** are written up and archived.
