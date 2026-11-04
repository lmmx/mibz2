# Compact30 prototype 1: the scanner and tile-bounds cleanup (charter §6.1)

**Authority.** The standing research charter,
[2026-11-04-compact30-research-charter.md](2026-11-04-compact30-research-charter.md), revision 4,
ACTIVE at `d055c51`.
- **This prototype:** charter §6.1, row 1.
- **Its design:** the handoff,
  [2026-11-04-compact30-scanner-cleanup-handoff.md](2026-11-04-compact30-scanner-cleanup-handoff.md)
  at `25d7eb59`.
- **The coordinator's brief** narrows it further: all three attempts must be **safe Rust**,
  with no new `unsafe` block and no unchecked or raw-pointer access. The handoff's
  `read_unaligned` formulations are not authorised. If the safe forms miss the compile target,
  the miss is recorded under gate 2, the unsafe operations that would be needed are written
  out here, and that line stops as a §8.2 escalation.

**Branch:** `research/compact30-scanner`, from `d055c51`.

## 0. Pre-registration (committed before any implementation, Miri run or ledger figure)

### 0.1 The prototype

- **What it is:** a new module, `examples/c30walk/ctor2s.rs`. It is `ctor2.rs` at `8beffe2`,
  copied, with only the scanner's source-side reads and tile slicing re-expressed (handoff
  C1–C4).
- **Nothing else changes:** no finaliser change, no descriptor-layout change. `ctor2.rs`,
  `ctor3.rs`, `ctor.rs`, the walker, the oracle, `src/`, `cbuild.rs` and `bench/` stay
  byte-identical. `examples/c30walk.rs` gains a `mod` line and a `ctor2s-` dispatch.
- **Its baseline is B2,** frozen v2:
  - the listings archived at `cd14fa9`;
  - the frequencies from v2's capture counters, as recomputed in the handoff;
  - plus the counted instantiation's own counters for every changed path (§5.3b).

### 0.2 The compile target (gate 2, diagnostic): the handoff's per-path targets

| path | B2 | target |
|---|--:|--:|
| fresh pair, from the held test to the second load | 12 | **4** |
| lookahead, per matched tile | 16 | **11** |
| a nonuniform full tile's overhead outside its 16 row appends | 41 + 7X | **34 + 7X** |
| row loops (four scalar copies, both fragment fills, the emitter and its odd tail) | as listed | **identical** |

**Also targeted:**
- no `slice_index_fail` reachable from the full-tile or lookahead paths;
- no new spill, reload or base register in the scan loop;
- no unconditional jump in place of the phase test.

A miss is recorded as a miss, and gate 2 never disqualifies (charter §4).

### 0.3 The registered minimum (gate 3, the qualifier)

**60 M instructions net on B2,** at the lower end of the conservative §5 interval (charter
§5.4).

### 0.4 The Miri classes that must be reached (gate 1, charter §8.3)

1. **Tiny and empty:** n = 0, 1, 2, 15, 16, 17, 31, 32, 33, with σ = 1 and 2.
2. **Every tail length,** 1..=15, after at least one full tile, and with no full tile at all.
3. **Odd alignment:** the input passed as a subslice at byte offsets 1..=7 of its allocation,
   for a tiny block and for one with full, uniform and nonuniform tiles.
4. **The phase sweep:** a nonuniform full tile starting at every p = 0..=29. That gives
   n1 = 16 at p ≤ 14 and n1 = 1..=15 at p ≥ 15, so the second segment is both empty and
   nonempty.
5. **Held tiles:** a run followed by:
   - a different byte's uniform tile;
   - a nonuniform tile;
   - a tile matching the run's byte in its first qword only;
   - a tile matching it in its second qword only;
   - consecutive held tiles across alternating bytes.
6. **A run ending because no full tile remains** (Q > 0): at the block's end, and at an
   epoch's end.
7. **An epoch boundary:** a run straddling 61,440 (split), and a nonuniform tile at
   61,424..61,440. One block of 61,440 + 64 rows.
8. **Malformed counts:** a perturbed histogram, rejected with empty arrays, through both
   builds.

Every class must complete. A timeout is partial coverage: the cases are split and run to
completion.

### 0.5 New invariants and their mutation checks

- **The safety argument gains no new invariant.**
  - The re-expression uses only safe slice and array operations: `split_first_chunk`,
    fixed-size array halves and `split_at`.
  - Every source access stays bounds-checked by the language, unless the compiler proves it
    redundant.
  - The destination side, `one`, `run` and the writes, is v2's, unchanged, with v2's
    obligations.
- **Correctness relies on two facts,** each with a mutation check registered here:
  - **(M1) A held tile is the next tile.** After a run whose lookahead found a mismatching
    full tile, the scan resumes at that tile, using its already-read halves. Mutant: the
    held tile is skipped, so the scan resumes after it.
  - **(M2) The run's length is the rows its tiles span.** The rows passed to `run` equal
    16 × the run's tiles. Mutant: the length is short by one tile.

  Each mutant must fail the quick synthetic suite, with a mismatch or a counter difference.
  Not catching it is a §8.2 escalation.
- **The phase invariant 0 ≤ p < 30 is v2's** (handoff §2 C3), not new. Under the safe form it
  bounds no access: `split_at(n1)` stays checked, or is proven by the compiler from
  n1 ≤ 16.

### 0.6 The rejection conditions (handoff §5.1–§5.2)

**Gate 1 (correctness) fails if:**
- any `rec` or `raw` byte differs from the oracle's;
- any v2 counter differs field by field: scalar, fragment and bulk rows; runs and run rows;
  runs split at an epoch; complete groups; every finish class; normalisation; epochs and
  epoch searches; RAW work; every portion-shape cell;
- the walker's output or marker restoration differs;
- a malformed input publishes a nonempty array or isn't rejected;
- Miri reports undefined behaviour, or a registered class can't be completed;
- a registered mutant survives.

**Gate 2 records a miss if:**
- a credited check survives in an equivalent form;
- the fresh pair's second load isn't folded;
- an instruction is added per tile or per row;
- a spill or reload appears in the scan loop;
- a row loop's instructions change;
- the tail reads past `end`.

**Gate 3 fails if:**
- the §5 interval's lower end is below 60 M; or
- any relevant added cost stays unbounded.

The ledger recounts every changed function in full, takes Q at its least favourable end, and
reports loads, stores and stack operations alongside.
