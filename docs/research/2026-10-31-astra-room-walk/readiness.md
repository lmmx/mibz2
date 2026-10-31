# Readiness for a walker research arm (tooling check; nothing implemented or timed)

**Scope:** whether master's existing tooling can test one room-walk transformation as a
separate research arm, if the read-only analysis (`README.md` §6) returns one and it is
approved. This is a review only. No walker code has changed, and nothing has been timed.

## What exists (checked on master `45b366d`, whose `src/bwt.rs` is the same as at `ae942bd`)

| Need | Tool | Can it carry an arm? |
|---|---|---|
| Same-binary paired timing of the walk alone | `examples/sweep.rs`, through `.github/workflows/sweep.yml` (`workflow_dispatch`, `tool: sweep`) | **Yes.** It is a table of `Setting { walk, lanes, pf, segments, f }`, each paired round by round against `DEFAULT` (room, 32, `PF_L1`, 512). It gives the geometric mean of the paired ratios with a 95% t-interval, as one copy pinned to CPU 1 and then a copy on every CPU. An arm is one more `Setting` (a new `walk` label), generic over L, so it joins the lane sweep automatically. |
| Lane-count sweep | the same, with lanes 8 to 64 | **Yes,** if the arm is generic over `const L: usize`. |
| Stage table (the walk within a decode, the 4.49 c/B row) | `examples/phases.rs` (`tool: phases`); `bench/stages.py` for build against build | **Partly.** `phases` times the walk the decoder uses, plus fixed alternative rows. An arm needs one more row, "walk, room-X". `stages.py` compares builds, so it measures an arm only once that arm is the default, which is a later, separate step. |
| Output correctness on real blocks | `sweep.rs` checks each setting's output against the plain walk, block by block, before timing | **Yes.** |
| Unit and edge tests | `bwt::tests`: `room_walk_short_cycles`, `room_walk_crafted_permutations`, `segmented_walks_on_any_length`, `flagged_walk_equals_plain`, `segment_oracle` | **Yes,** extended to the arm. These are seeded and crafted generators; there is no `proptest` dev-dependency. |
| `unsafe` handoff paths under Miri | `bwt::tests::miri_room_walk_handoffs`, through `miri.yml` (runs on PRs touching `src/bwt.rs`, or on `workflow_dispatch`) | **Yes,** with new cases for the arm's own edges. |
| Hardware counters | the `sweep.rs` counter round (instructions, cycles, LLC misses, back-end stalls a byte) where a PMU is exposed; `pmu.yml` probes the runners | **Uncertain on CI.** Earlier runs exposed no PMU. The i9 has one. |
| Structural audit of the walk's machine code | none on master; the MTF audit (`bench/p74c_decoder_asm_audit.py`) doesn't apply | **Added:** `bench/walk_asm_audit.py` on this branch (below). |

## The added audit, `bench/walk_asm_audit.py`

- **The loop it finds:** in each instantiation of `bwt::room_segments`, the smallest
  backward-branch loop that holds a software prefetch and a `[base+idx*4]` T-vector
  load.
- **What it counts,** per T-vector load (so an unrolled step normalises to one lane-step):
  - instructions and macro-ops (with test/cmp + jcc fusion);
  - loads, stores and prefetches;
  - stack accesses;
  - conditional branches;
  - whether the loop body holds more than one T load.
- **Checked on master's `phases`** (rustc 1.97.0, fat LTO, one codegen unit): it
  reproduces the pack's hand census exactly.
  - `room_segments::<32, 1>`, loop `0x2a530–0x2a563`: 16 instructions, 14 macro-ops,
    3 loads, 2 stores and 1 prefetch per lane-step, with 2 conditional branches.
- **A limit:** `stack_loads` counts only `rsp`/`rbp`-based operands. The lane array's
  `[r11]` row load is a stack access through a pointer register, so it isn't counted
  as one (1 counted, 2 actual).
- **Status:** a heuristic signature reader, to compare an arm against an expected
  signature registered before timing. It is not a proof of dependence structure.

## Gaps an arm's trial would close (on its own research branch, not master)

1. **The timed binary is not audited or kept.** `sweep.yml` uploads only the text
   tables.
   - The trial's workflow copy should run `walk_asm_audit.py` on the exact
     `target/release/examples/sweep` it times, and upload the binary's SHA-256 and the
     audit JSON.
   - The P74-C trigger branches show the pattern: a push-triggered copy that checks out a
     pinned SHA.
2. **One runner per dispatch.** The CPU model is whatever the pool gives (7763, 9V74,
   9V45 and 8573C so far).
   - The targets in `README.md` §5 are on the 7763 basis. Use a 3-job matrix, and report
     by CPU model.
   - Register the decision on the 7763 rows, with the others as replication.
3. **A floating toolchain.** `dtolnay/rust-toolchain@stable` gives a different rustc
   from run to run.
   - Pin it for the trial, or record `rustc --version` beside each audit. The sweep's
     header line already prints it.
4. **Rounds:** `sweep.rs` caps ROUNDS at 2–31. 20 paired rounds fits.
5. **No justfile recipe for the i9** (the user's rule: recipes, not cargo commands). The
   trial adds one, for example `just walk-arm-sweep LABEL`, if an i9 confirmation is
   wanted.

## Review of `implementation-prompt.md` (amendments, if it is used)

- **Measurement:** name the tool exactly.
  - The arm is a new `Setting` in `examples/sweep.rs`, paired against `DEFAULT` in the
    same binary, through a 3-job copy of `sweep.yml` pinned to the arm's SHA.
  - `phases` gains one row for the arm, as a secondary.
  - `bench.yml` and `stages.py` aren't used: they compare builds, and the arm isn't the
    default.
- **The primary figure,** registered with the signature: one copy, L = 32, `PF_L1`, the
  arm's paired ratio against `DEFAULT` on the 7763 jobs. The criterion is stated against
  that ratio's 95% upper bound.
  - The all-CPUs table is a secondary, and must not regress beyond a registered bound,
    because the walk's SMT behaviour (`README.md` D4′) is part of the question.
- **Correctness:** replace "add a `proptest`" with seeded generators in the existing
  test style (no new dependency).
  - The arm joins `room_walk_short_cycles`, `room_walk_crafted_permutations`,
    `segmented_walks_on_any_length` and the Miri handoff test.
  - The prompt's own edge list stays: a boundary at every lane and tile position, the
    last segment ending mid-pass, `grow` mid-pass, ns < L.
- **Structural audit:** run `bench/walk_asm_audit.py` on the timed binary and compare it
  with the registered signature before reading any timing. A mismatch stops the trial, as
  the prompt already says.
- **Attribution and scope:** unchanged.
  - Louis Maddox as author and committer; no AI or tool metadata.
  - No production dispatch change, no master merge.
  - No changes to PR #117 or the P74-C branches.
