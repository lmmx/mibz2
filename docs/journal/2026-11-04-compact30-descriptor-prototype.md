# Compact30 prototype 2: the hot descriptor pointer arrays (charter §6.1)

**Authority:** the standing research charter,
[2026-11-04-compact30-research-charter.md](2026-11-04-compact30-research-charter.md), ACTIVE;
§6.1, row 2.
- **The design:** report 227 §D.3, in
  [the reassessment](2026-11-03-compact30-constructor-v2-reassessment.md).
- **Its audit:** reports 228–235, with the evidence in
  [2026-11-04-compact30-ultra-audit-evidence/](2026-11-04-compact30-ultra-audit-evidence/), in
  particular `descriptor/findings.txt` in its zip.
- **The coordinator's brief:**
  - safe Rust only;
  - independent of prototype 1 (built on frozen v2, not on `ctor2s`);
  - prototype 1's lessons applied.

**Branch:** `research/compact30-descriptors`, from `origin/research/compact30-constructor` at
`9f1086f8`.

## 0. Pre-registration (committed before any implementation, Miri run or ledger figure)

### 0.1 The prototype

- **What it is:** a new module, `examples/c30walk/ctor2d.rs`. It is `ctor2.rs` at `8beffe2`,
  copied, with one change: the 40-byte per-character descriptor `Desc { write, end, w0, r0,
  lim, g, raw }` is split into:
  - two hot arrays, `write: [*mut u16; 256]` and `end: [*mut u16; 256]`, indexed by the byte;
  - a cold array of the other fields (`w0`, `r0`, `lim`, `g`, `raw`).
- **Where they live:** all three arrays live in the build state `W`, on the stack, as v2's
  descriptor array does. No allocation is added, and nothing proportional to rows or groups.
- **What it is meant to do:** let the scalar append address `write[c]` and `end[c]` with
  base + 8c (scale-8 addressing), removing v2's two stride-40 LEAs per row.
- **Nothing else changes:**
  - no scanner change, so `ctor2s` isn't used;
  - no finaliser change;
  - no change in what any path computes.
- **Frozen, each unchanged:** `ctor2.rs`, `ctor2s.rs`, `ctor3.rs`, `ctor.rs`, the walker, the
  oracle, `src/`, `cbuild.rs` and `bench/`. `examples/c30walk.rs` gains a `mod` line and a
  `ctor2d-` dispatch.
- **Its baseline is B2:**
  - the `cd14fa9` listings;
  - v2's capture counters;
  - and, as a cross-check, v2's `build::<false>` compiled in the same binary.

### 0.2 The compile target (gate 2, diagnostic)

**The row loops**, applying prototype 1's lesson:

| path | B2 | target |
|---|--:|--:|
| scalar append, common path, in **each of the four** copies (v2: `33a00`, `34350`, `343e0`, `344d0`) | 12 | **≤ 10** |
| explicit memory operations per scalar row | 4 reads, 2 writes | **no more**: the source byte, `write`, `end`, the pointer read-modify-write; the field store and the pointer read-modify-write |
| spills or reloads per scalar row | 0 | **0**: no reload of a base register inside a loop |
| fragment fill loops, the complete-record emitter and its odd tail | as listed | **no more instructions** |

**The non-row paths,** each recounted in full at its observed frequency:
- the refill caller detours and `refill` itself;
- `run`'s entry, fill, close and open;
- `finish`;
- the snapshot;
- descriptor initialisation;
- the final rank check and sweep.

**The aggregate target:** added non-row work ≤ 5,000,000 instructions (the report 227 cap).
The audit's concerns are each registered for explicit counting:
- the byte index preserved or reloaded around `refill`;
- the split reset stores at close: three scalar stores where v2 has one 16-byte and one 8-byte
  store;
- the snapshot's rank reads now coming from two arrays.

A miss is recorded as a miss; gate 2 never disqualifies.

### 0.3 The registered minimum (gate 3, the qualifier)

**40 M instructions net on B2,** at the lower end of the conservative interval.

**The ledger method:**
- **Primary:** the charter's §5 method, compiled path lengths × observed execution counts,
  from the counted build's counters. Every changed function is recounted, and unknown costs
  are listed as unresolved, never zero.
- **Cross-check:** callgrind's exact count of each `build::<false>` on the primary corpus. It
  is an untimed instruction simulation, not a perf counter or a timing. Both are reported, and
  any difference is explained.

### 0.4 The Miri classes that must be reached

1. **Tiny and empty:** n = 0, 1, 2, 15, 16, 17, 31, 32, 33, with σ = 1 and 2.
2. **Every refill slot:** portions ending at each slot 0..=29, with two and three characters
   interleaved, so descriptors open, fill and close in alternation.
3. **All 256 byte values present** in one block, so every hot and cold index 0..=255 is used,
   255 included.
4. **Crossing groups:** a bucket boundary at every offset for n = 61, so preset RAW portions
   are opened and finished.
5. **Runs:** a run's fill, close, complete and open, with a run ending inside a group and one
   spanning several.
6. **An epoch boundary:** one block of 61,440 + 64 rows, where the snapshot reads the split
   arrays.
7. **Pending portions** finished by the final sweep.
8. **Malformed counts:** both builds, empty arrays.
9. **Reuse:** a small sequence of sizes up and down on one constructor.

Every class must complete. A timeout is partial coverage: the cases are split and run to
completion.

### 0.5 Invariants and mutation checks

**The safety argument gains no new invariant.**
- v2's descriptor invariant, restated over split storage, is:

  > for every c, either `write[c] = end[c] = cold[c].w0 = null` (no open portion), or they
  > bound one open portion, with `cold[c].w0 ≤ write[c] ≤ end[c]` inside one group's fields.

  The same updates maintain it at the same sites: `open`, `close`, `one` and `fill`. The
  pointer writes and increments are v2's operations, at v2's sites, on the same pointers,
  now held in another array.
- **No new `unsafe` operation, pointer arithmetic or unchecked access is introduced.** The
  arrays are indexed by `c: u8` or `0..256`, checked by the language or proven by it.

**Mutation checks,** registered because the invariant's storage changes:
- **(M1) `end` inconsistent with its portion:** `open` sets `end[c]` one field short. It stays
  memory-safe, inside the group. Expected: an output or counter mismatch.
- **(M2) the snapshot's ranks read from the wrong array:** the rank is computed from `end[c]`
  instead of `write[c]`. It is integer only. Expected: an output or counter mismatch.

Each must fail the quick suite. Not catching one is a §8.2 escalation.

### 0.6 The rejection conditions

**Gate 1 fails if:**
- any `rec` or `raw` byte differs from the oracle's;
- any v2 counter differs field by field;
- the walker's output or marker restoration differs;
- malformed counts are published, or not rejected in both builds;
- Miri reports undefined behaviour, or a registered class can't be completed;
- a mutant survives.

**Gate 2 records a miss if:**
- any scalar copy is over 10 instructions;
- a row path gains a load, store or reload;
- a fragment or emitter loop grows;
- added non-row work exceeds 5 M.

**Gate 3 fails if:**
- the interval's lower end is below 40 M; or
- any relevant added cost stays unbounded.

**Overlap (§5.5):** the listing-diff intersection with prototype 1 (`7f74e9f`, PR #146) is
reported.
