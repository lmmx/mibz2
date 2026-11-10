# 2026-11-10: `run16` in `FusedBuild::share`'s equal window, registered before the kernel

**Status: registered, not built, not counted.** This entry is committed and pushed before
any line of the kernel is written. Its bars do not move after a count. A count, never a
clock: nothing here is evidence of time.

## Hypothesis

From the Pathfinder's residual (`2026-11-10-team-residual.md`, #313 at `dae77cfe6`): inside
`FusedBuild::share` (`src/coop.rs`), a window of 16 equal bytes is written by 16 scalar
stores, 3.94 Ir/D on Wikidata (10.74 M windows, 81.9% of rows, about 77 Ir a window). One
worker writes the same window with `bwt::run16` (four SSE2 stores of bytes widened and
added to the links), about 33 Ir a window with its classifier. Replacing the scalar body by
`run16` removes about 44 Ir a window: 10.74 M × 44 / D ≈ **2.3 Ir/D**.

**The change, and only it:** the body of the equal-window branch in `share` becomes one call
to `bwt::run16(ll + s, i << 8, slots + s)` (made `pub(crate)`). The classifier (two `read_u64`
and the rotate), the guard `s + 16 > lim[c]`, the one-row step, the tail and the end check
stay as they are. No new invariant: the 16 entries are final, inside `[s, s + 16) ⊆` the
share's window of bucket c, already guarded.

## Base

Master `27ed84df4`, whose `src/` is `ae44497e9`'s (#330's count; no `src/` change since).
#334's staged walk is **not** on this base: the count is on the old walk. The predicted drop is
the build's alone, so it is the same ~2.3 Ir/D on either base.

## Prediction and falsifier (on this base)

| figure (Ir per decoded byte, D = 209,700,196) | old (#330, counted) | predicted new |
|---|--:|--:|
| `FusedBuild::share`, inclusive | 8.937 | **about 6.6** |
| team whole process | 37.910 | about 35.6 |

- **Falsifier:** `share`'s inclusive cost on the new arm **not below 6.9 Ir/D**. Then the
  port does not deliver what the residual priced.
- **On #334's base** the same drop would take its whole process 30.21 → about 27.9. Not
  counted here.

## Count method

- **Binary:** `cargo build --release --example ab` (fat LTO, `line-tables-only`), rustc 1.99.1,
  at this branch's head (new) and at master `27ed84df4` (old). Each binary's SHA-256 recorded.
- **Corpus:** all of `wikidata.bz2`, SHA-256 `82f65ace…c2c29915`, 431 blocks.
- **Arms**, each one whole decode under
  `valgrind --tool=callgrind --dump-instr=yes --separate-threads=yes`:
  1. old team: `ab child mibz2-pipe3-both 9` on the master binary (a recount of #330's
     team arm, the control);
  2. new team: the same on the branch binary;
  3. new one worker: `ab child mibz2 1` on the branch binary (G3: the plain path unchanged).
- **Figures:** `bench/team_gap_count.py report` (#330's script, unchanged) for the stage
  table and whole process, Ir/D. `share`'s inclusive Ir: its self Ir plus every call arc out
  of it, summed over threads, from the same profiles with the same parser.
- **Each arm counted once.** No re-run to pass a bar. A failed condition is reported as failed.

## Conditions (all must hold for the count to be read)

1. **Same bytes:** every arm writes 209,700,196 bytes with hash `a8830caee5c511a6` (#330's).
2. **Helpers execute `share`:** in both team arms, calls to `FusedBuild::share` total 466
   (233 cooperative blocks × 2 helpers), all on threads that run no MTF (helpers).
3. **The new window path is taken:** in the new team arm, the instructions inside `share`
   whose debug line is in `bwt.rs`'s `run16` execute at least 10,000,000 times each for its
   vector stores (expected: the equal-window count, about 10.74 M); in the old arm, none.
4. **The control reproduces #330:** the old arm's `share` inclusive within 0.05 Ir/D of 8.937
   and whole process within 0.2 Ir/D of 37.910 (the coordination spin is scheduling-dependent).
5. **The one-worker arm** of the new binary is within 0.01 Ir/D of #330's 25.792 (G3).

A deviation from this method (an inlining change, a function renamed by LTO) is disclosed as a
deviation, not fixed silently.

## Dispatch paths (G3, D083)

`FusedBuild::share` is shared by every team build. What changes and what does not:

| path | reached by | changes? | why | evidence covering it |
|---|---|---|---|---|
| pipelined team helpers (`invert_pipelined`, `FusedBuild::share`) | the 3 × 3 plan (10 physical cores, 20 MiB L3, ≥ 10 threads) | **yes** | the port | this count (arm 2), the unit tests |
| ordinary teams (`Cooperative::invert_checkpointed` → `coop::build_fused` → `share` on the pool) | ≥ 6 physical cores and threads, not the 3 × 3 topology | **yes**, the same function | the same port | the unit tests (`build_fused_is_build`, `miri_build_fused`, the new differential); not counted here |
| plain worker `bwt::build` / `build_one_pass` / `run16` | `Workers` plans, one thread, CI's A/B | **no** (`run16` only made `pub(crate)`) | already has it | arm 3; CI's A/B |
| team solo paths (`parallel::solo`, blocks below `MIN_COOP`, a failed pool) | any team plan | **no** | they already call `bwt::build` | read; arm 2 counts them inside the same decode |
| `build_two_pass` | counts not `ll`'s own | no | untouched | |

## Safety (B5)

The new `unsafe` reads `ll[s..s + 16]` and writes `slots[s..s + 16]` by pointer. Argument,
to be written in the code and the result entry: `s + 16 ≤ lim[c] ≤ n` (the guard, and `new`'s
checks that every checkpoint lies within the counts, which sum to n), so both ranges are
inside `ll` and inside the table's reserved capacity; `[s, s + 16)` is inside this share's
window of bucket c, which no other share's window overlaps. Tests before counts: a
differential against the current share's scalar window on crafted tables (all equal,
alternating, equal runs starting at every offset mod 16, block ends, short blocks, every
share count 1..=4) and real columns; Miri on a targeted small test; at least three mutants
of the window logic, each of which the tests must catch. The adversarial review (B5) is by
an agent other than the implementer, after this.

**Not claimed:** time, adoption, merge. CI's A/B never runs a team (#330 §7), so its neutral
reading on this PR is expected and says nothing about the gain.
