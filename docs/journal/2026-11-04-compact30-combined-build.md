# Compact30: the combined build, assembly 1 (charter §6.3), with v3's finaliser under §7

**Authority:** the standing research charter,
[2026-11-04-compact30-research-charter.md](2026-11-04-compact30-research-charter.md), ACTIVE:
§3 (the combined build is delegated), §5 (the ledger), §6.3 (its gates), §7 (v3's
conditions), §8 (repairs and escalation). The U2 unchecked access is carried unchanged under
§6.1 and report 264. The floor follows Guardian decisions D010 (the demonstrated-overlap floor)
and D011 (the counted-path evidence, and the "normalised closure unchanged" wording).

**This entry records numbers and gate results. It makes no qualification or composition
decision.** That belongs to the coordinator and, where the charter says so, the guardian.

**Branch:** `research/compact30-combined`, from `research/compact30-constructor` at `1bbe34fd`.
**Untimed throughout:** no timing, benchmark, perf counter, i9 or workstation use. Callgrind is
an untimed instruction simulation.

## 1. Summary

| gate | role | result |
|---|---|---|
| 1, correctness and safety | qualifier | **PASS** (§3): every suite exact; Miri over the union of the 22 registered classes (§3.3); every component's mutant caught at its intended test or operation; no new safety obligation |
| 2, machine code | diagnostic | **findings** (§4): the scanner's first-segment do-while compiles **fully unrolled** in the combined build, so `build::<false>` grows from 1,431 to 1,784 instructions; each component's other paths are reproduced or listed with their deviation |
| 3, net work on B2 | qualifier | **[258,494,335, 262,948,911] instructions removed** (§5); every changed class counter-supported |

**The floor and the pass conditions** (§6), with v3 included:
- O ≥ 2,393,313 (S62-F02, the registered known minimum). This assembly demonstrates a further
  3,654 between the scanner and the descriptor arrays, and none involving v3.
- F = 60 M + 40 M + 50 M − 2,393,313 = **147,606,687** (147,603,033 with the further 3,654).
- **(i)** 258,494,335 ≥ 147,606,687: **yes**, by 110,887,648.
- **(ii)** 258,494,335 > 84,060,421: **yes**, by 174,433,914.

**v3's own conditions** (§7):
- **§7(b):** 59,688,510 at its lower end, against 50 M: **met**.
- **§7(c):** the combined leaf is **44 instructions including `ret`**, against at most 44: **met**,
  equal.
- If v3 is carried: "included under §7; compiler-gate STOP unchanged". Its recorded STOP and
  every component's recorded gate outcomes are unchanged by this entry.

**Without v3:** not assembled. Its F is 97,606,687. An unmeasured estimate is in §7.3, and it is
not a ledger.

## 2. What was built

**`examples/c30walk/ctor2c.rs`** (new). `examples/c30walk.rs` gains a `mod` line and a
`ctor2c-` dispatch. The constructor is v2 (`ctor2.rs` at `8beffe2`) with the three components,
each carried from its pinned module:

| component | pin | carried |
|---|---|---|
| scanner U2 | `ctor2su.rs` @ `ba261ef` | the `rest` slice scan (`split_first_chunk::<16>`, `halves`); `tile16` with the window do-while and **U2, `ll.get_unchecked(i + n1..i + 16)`, unchanged**; `tile` for the tail only |
| descriptor arrays | `ctor2d.rs` @ `7620c0c` | `wr[256]`, `en[256]` and `cold[256] = {w0, r0, lim, g, raw}` in `W`; `desc(c)` and `rank(c)` assembled on reading; attempt 2's append |
| v3's finaliser | `ctor3.rs` @ `dbf5139` | `ORD`, `UNOPENED`, `epoch_tag`, `active_tag` set with each snapshot; `open` stores `active_tag`; the leaf `finish(c: u8)`; `finish_slow` with the high-bit test; `count_leaf` |

**The one change of expression is v3's leaf on the split storage.** It reads `cold[c].raw`,
`cold[c].w0` and `wr[c]` where v3 read `desc[c].raw`, `desc[c].w0` and `desc[c].write`. By the
descriptor split's bijection these are the same values, each stored once. Every other line of
each component is its pinned text, apart from naming (`Stats2c`, `Ctor2c`).

**The frozen files are unchanged:** `ctor2.rs` (blob `168c2bac`), `ctor3.rs` (blob `01f91419`),
`ctor.rs`, `cbuild.rs`, the walker and oracle in `c30walk.rs`, `src/` and `bench/`. `ctor2su.rs`
and `ctor2d.rs` aren't on this branch: the combined module is a new file, as the charter
requires.

**The harness** is the scanner trial's, with the descriptor's and v3's added (the generator is
`tools/mkharness.py`):
- `same_as_v2` compares every v2 field with v2's own counted build, field by field and with the
  portion shapes, plus the union of the components' identities:
  - the scanner's: tiles, fresh/lookahead, S, R, Q, tails and `unchecked_tiles = tiles_nonuniform`;
  - the descriptor's partitions;
  - v3's: leaf = v2-eligible; v2-fast = leaf + same-epoch partial; every finish is leaf or slow;
    the slow classes; preset RAW;
  - this build's own (§5.2).
- The synthetic suite holds v2's classes, the scanner's 9, the descriptor's 9, and v3's
  tag-dispatch and later-epoch classes.
- `--miri CLASS` runs one class of the union, as `COMPONENT:CLASS` (§3.3).
- `ctor2c-count` has `--only v2|c` (one builder per process, the profile's mode) and `--counted`
  (the counted build alone, in the `--only c` process's order and arenas).

**Commits:**
- `c4e45d30`: the profiled source (every figure's binary, §5.1).
- `589d810e` and `591c84f7`: counted-only counters (§5.2). `591c84f7` moves one counter after
  U2 (§8).

## 3. Gate 1: correctness and safety

### 3.1 The release suites

On the final counted-counter binary (`591c84f7`, sha256 `bc5fdae6…fcfb9808`), every check exact,
each exit 0 (`counted/`). The same suites on the profiled binary (`c4e45d30`, `c21ec741…e8230dd`)
are in `suites/`, with every pre-existing field of every record equal.

| suite | result |
|---|---|
| corpora: the 11 corpora of `2026-11-02-compact30-census/corpora.sha256` (all hashes checked) × 4 configurations | 44 records: 12,104 array checks, 18,156 walk checks, 12,104 restorations. The natural configuration is 1,513 blocks and 707,949,378 rows |
| adversarial | 200 records: 1,056 / 1,584 / 1,056 |
| synthetic, full | **54,287 cases**: 100,466 array comparisons, 150,699 walk outputs, 100,466 restorations |
| synthetic, quick | 869 cases: 1,634 / 2,451 / 1,634 |

**What each case runs:**
- The oracle `C30::build`.
- The combined counted build into a poisoned, reused arena, byte for byte against the oracle's
  `rec` and `raw` (the reference builder).
- v2's counted build on the same input, with every counter compared and the union of identities
  above.
- The uncounted build, re-poisoned, compared again.
- The unchanged walker on each output against production's and the oracle's, then the arrays
  again (marker restoration).

The malformed-count classes reject in both builds with both arrays empty, and reuse runs across
sizes. No assertion fired, including the U2 contract (`ptr::eq` of the chunk and `ll + i`,
p < 30) and `tile`'s `len < 16`.

**Against the components' archives:**
- Every one of the 44 corpus records equals the scanner's and v3's archived records on every
  shared field (2,821 and 1,897 fields).
- Against the descriptor's, they differ only in its arena-history fields (`rec_grows`,
  `raw_grows`, the bitmap reserves). Those depend on the three-thread harness's scheduling, as
  that prototype's §10.1 records. The sequential counted record (§5.2) has the profile's
  output digest, `4efb1c13ae58340f`.

### 3.2 The safety argument

**No new safety obligation.** The `unsafe` items of `ctor2c.rs` are exactly the union of the
components':
- v2's raw-pointer arena writes (`field`, `create_raw`, `open`, `one`, `fill`, `complete`,
  `write_entries`, the padding and PARK);
- the scanner's `tile16` with U2, the module's **only** unchecked access. The source has no U1,
  no `assert_unchecked` (U3) and no `read_unaligned`;
- v3's `finish`, `finish_slow` and `count_leaf`.

No pointer arithmetic, unchecked access or uninitialised read is added or widened.

- **U2, unchanged** (the scanner journal's §9.2):
  - `tile16` is called only from the `split_first_chunk::<16>()` arm of the scan, with `t` the
    chunk split off `rest`. `rest` is a suffix of `ll[i0..end]`, and i = end − `rest.len()`, so
    i + 16 ≤ end ≤ `ll.len()`.
  - n1 = min(16, 30 − p) ≤ 16 for any p.
  - So [i + n1, i + 16) is well-ordered, inside the live `&[u8]`, initialised and immutable.
  - The contract is asserted in the counted build and debug-asserted under Miri.
  - Neither the descriptor split nor v3 touches the scan, `rest`, `tile16` or p.
- **The descriptor split** (its §3.4, §0.7.1):
  - `wr[c]`, `en[c]` and `cold[c]` hold v2's `Desc` fields, each exactly once.
  - `open` sets all three for one group; `close` computes the rank and then nulls them.
  - The indices are `c: u8` or `0..256`.
  - v3's changes to `open` (the tag store) and `close` (`finish(c as u8)`, c < 256 checked in
    the counted build) are value changes inside that same structure.
- **v3's leaf** (its §3.1–§3.3):
  - It is entered only for a nonempty portion, opened since its last close: `close` tests
    `wr[c] != cold[c].w0`.
  - `raw == active_tag` (high bit set) means an ordinary portion opened in the current epoch.
    The opening is followed at once by its first append: refill → `one`, or run → `open` →
    `fill`. In the combined build, `tile16`'s and `tile`'s rows reach `refill` through `one`,
    exactly as v2's and v3's do, and the snapshot still precedes every epoch's rows.
  - 30 fields means `w0` is field 0 of its record, so the header store at `w0 + 60` and the
    60-byte read and write lie in the reserved record.
  - The leaf now takes `w0` and `write` from `cold[c].w0` and `wr[c]`: the same pointers, by
    the bijection.
  - `finish_slow` tests the high bit, so a tagged portion is never read as a RAW index.

No step of the three arguments depends on another component's code beyond what it already
assumed of v2's.

### 3.3 Miri: the union of the registered classes

Miri 0.1.0 on nightly rustc 1.101.0 (`16d1ffac1`, 2026-10-26), Stacked Borrows, no flags. Each
class ran alone, one after another, as
`cargo +nightly miri run --example c30walk -- ctor2c-synthetic --miri CLASS`, with unpoisoned
arenas and a 7,000 s timeout. **A timeout would be partial coverage, never a PASS.**

The classes are the scanner trial's 9 (`scanner:*`, 106 cases registered), the descriptor's 9
(`descriptor:*`, 221 cases), and v3's targeted selection of 134 cases. v3's selection is split
into its four marked groups (`v3:small` 127, `v3:tag-tiny` 4, `v3:span` 2, `v3:tag-epoch1` 1), as
§8.3 allows. Every case runs the full §3.1 checks.

| class | registered cases | ran | exit | seconds | reached (this run's counters) |
|---|--:|--:|--:|--:|---|
| `scanner:tiny` | 18 | 18 | 0 | 15 | nonuniform 7, second segments 2, tails 12; held 0 |
| `scanner:tails` | 45 | 45 | 0 | 39 | nonuniform 15, second segments 0, tails 45; held 0 |
| `scanner:align` | 14 | 14 | 0 | 14 | nonuniform 14, second segments 0, tails 14; held 14 |
| `scanner:phase` | 15 | 15 | 0 | 18 | nonuniform 15, second segments 7, tails 15; held 14 |
| `scanner:held` | 5 | 5 | 0 | 6 | nonuniform 7, second segments 3, tails 5; held 12 |
| `scanner:endrun` | 2 | 2 | 0 | 2 | nonuniform 2, second segments 0, tails 1; held 0 |
| `scanner:epoch` | 2 | 2 | 0 | 1,296 | nonuniform 3839, second segments 1792, tails 0; held 2 |
| `scanner:malformed` | 4 | 4 | 0 | 1 | nonuniform 0, second segments 0, tails 0; held 0 |
| `scanner:controls` | 1 | 1 | 0 | 2 | nonuniform 2, second segments 1, tails 0; held 0 |
| `descriptor:tiny` | 18 | 18 | 0 | 13 | scalar rows 182, refills 21, crossing 12, epochs 16 |
| `descriptor:slots` | 60 | 60 | 0 | 66 | scalar rows 6270, refills 325, crossing 174, epochs 60 |
| `descriptor:allbytes` | 1 | 1 | 0 | 5 | scalar rows 1024, refills 273, crossing 272, epochs 1 |
| `descriptor:crossing` | 120 | 120 | 0 | 118 | scalar rows 3000, refills 288, crossing 232, epochs 120 |
| `descriptor:runs` | 6 | 6 | 0 | 8 | scalar rows 335, refills 27, crossing 22, epochs 6 |
| `descriptor:epoch` | 1 | 1 | 0 | 706 | scalar rows 61408, refills 2048, crossing 4, epochs 2 |
| `descriptor:pending` | 4 | 4 | 0 | 7 | scalar rows 891, refills 39, crossing 16, epochs 4 |
| `descriptor:malformed` | 4 | 4 | 0 | 2 | scalar rows 0, refills 0, crossing 0, epochs 0 |
| `descriptor:reuse` | 7 | 7 | 0 | 7 | scalar rows 477, refills 30, crossing 20, epochs 5 |
| `v3:small` | 127 | 127 | 0 | 108 | leaf 6, slow 167 (old epoch 0, partial 101, preset RAW 66), wide 0 |
| `v3:tag-tiny` | 4 | 4 | 0 | 4 | leaf 0, slow 7 (old epoch 0, partial 1, preset RAW 6), wide 0 |
| `v3:span` | 2 | 2 | 0 | 1,571 | leaf 4100, slow 12 (old epoch 6, partial 2, preset RAW 4), wide 1 |
| `v3:tag-epoch1` | 1 | 1 | 0 | 693 | leaf 2056, slow 3 (old epoch 2, partial 1, preset RAW 0), wide 0 |
| **all 22** | **461** | **461** | 0 | 4,701 | |

**All 22 classes completed, each with exit 0, every check exact, and no undefined behaviour
reported.** None timed out. The case counts equal the components' registrations: scanner 106,
descriptor 221, v3 134.

v3's own paths are reached exactly as in its closeout: 6,162 leaf finishes and 189 slow
entries (8 old-epoch ordinary, 105 same-epoch partial, 76 preset RAW), summed over its four
groups. The scanner's 3,839 epoch-class nonuniform tiles each went through U2, and
`unchecked_tiles` equalled the nonuniform tiles in every case, asserted.

### 3.4 The mutation checks, run again on the combined code

Each component's registered mutants were carried to the same operation in `ctor2c.rs`
(`tools/mutate.py`; each diff and console is in `mutations/`).

| mutant | component | how it was run | caught by | intended? |
|---|---|---|---|---|
| MU1, n1 forced to 17 | scanner | Miri on `scanner:controls`, default | `unsafe precondition(s) violated: slice::get_unchecked requires that the range is within the slice`, at `ctor2c.rs:840:30`, U2's line | **yes, U2** |
| MU1 | scanner | Miri, `-Zub-checks=no` | `Undefined Behavior: arithmetic overflow in unchecked_sub`, at `840:27`, U2 | **yes, U2** |
| MU2, U1's end = `ll.len()` + 1 on a final full tile | scanner | Miri, on the U1 form (variant a, carried as the trial did, since the final form has no U1), default | the precondition, at `840:30`, U1's line | **yes, U1** |
| MU2 | scanner | Miri, `-Zub-checks=no` | `constructing invalid value of type &[u8]: encountered a dangling reference (going beyond the bounds of its allocation)`, at `840:27`, U1 | **yes, U1** |
| MU3, U2 one past the tile | scanner | Miri, default | the precondition, at `839:30`, U2's line | **yes, U2** |
| MU3 | scanner | Miri, `-Zub-checks=no` | dangling reference, at `839:27`, U2 | **yes, U2** |
| M1, the held tile skipped | scanner | quick suite, release | `small n 32 sigma 3: the counted build rejected valid counts`, exit 101 | **yes** (the trial's case) |
| M2, the run short by one tile | scanner | quick suite | `small n 16 sigma 1: the counted build rejected valid counts`, exit 101 | **yes** (the trial's case) |
| DM1, `en` one field short at open | descriptor | quick suite | the finish invariant `k >= 1 && d.write == d.end` (`ctor2c.rs:601`, `finish_slow`), exit 101 | **yes** (the descriptor's assertion) |
| DM2, the snapshot's rank from `en` | descriptor | quick suite | `epoch k 1 n 61501 off -1 pm 0: Ctor2c's counted build differs: raw length 2 vs 3`, exit 101 | **yes** (the descriptor's case and message) |
| V3M, `finish_slow`'s two high-bit tests back to `== u32::MAX` | v3 | quick suite | `assertion failed: (d.raw as usize) < self.raw.len()` at `ctor2c.rs:633`, exit 101: a tag read as a RAW index | **yes** (v3's assertion) |

**Controls.**
- The unmutated `scanner:controls` passes, and so does the unmutated U1 form (variant a).
- Variant a was used only to host MU2. It is not the built form.

## 4. Gate 2: the compiled code (diagnostic)

All listings are from the profiled binary (`listings/`). B2's same-binary listings equal
`cd14fa9`'s after normalisation: 1,431, 67 and 379 instructions, all equal
(`gate2/build-vs-b2.diff` and the refill and finish diffs, by `tools/ndiff.py`).

| function | B2 | scanner | descriptor | v3 | combined |
|---|--:|--:|--:|--:|--:|
| `build::<false>` | 1,431 (0x18e5 bytes) | 1,382 | 1,450 | 1,432 | **1,784** (0x20bf) |
| `refill` | 67 | 67 | 74 | 67 | **74** |
| `finish` (v3: the leaf) | 379 | 379 | 379 | 44 | **44** |
| `finish_slow` | — | — | — | 378 | **378** |

Normalised diffs of each function against B2 and against each component's isolated listing are
in `gate2/*.diff`.

`gate2/paths.txt` puts every execution-count class side by side, B2 → scanner → descriptor →
combined. Each component's count comes from its own archived same-binary profiles. B2's three
same-binary profiles are identical, which is asserted.

### 4.1 The finding: the window do-while is unrolled

In the scanner trial alone, `tile16`'s first segment is a 12-instruction loop over the 16-byte
window. In the combined build, **LLVM unrolls it fully by row position** (`movzbl 0x0(%r15)` …
`movzbl 0xf(%r15)`):
- Each row is 9 instructions:
  - the source byte;
  - the `wr[c]` load and the `en[c]` compare;
  - the refill branch;
  - the label `lea`, the store and the `addq` to `wr[c]`;
  - a per-position exit test on n1 or p (`cmp $0x2,%rbx; jb` …, `cmp $0x1c,%r9; je` …).
- That compares with 10 in the descriptor's loop and 12 in B2's and the scanner's.
- Row 0 also recomputes i = end − `rest.len()`: 3 instructions with 2 stack reloads, once per
  tile, as the scanner trial has.
- **Each position has its own refill detour,** 16 in all, where B2 had one:
  - 13 instructions at positions 0 and 15, and 17 at positions 1–14;
  - against 10 in B2, 12 in the scanner and 10 in the descriptor.
- The 17-instruction detours **spill and reload three registers around the `refill` call**:
  three stores and three reloads per detour. Those are new spills and reloads on the refill
  path (996,634 detours), not on the common row path.

**Net, on the primary corpus,** B2's first-segment loop (12 × 29,357,924 = 352.3 M) and its
refill detour (10 × 996,634) give way to:
- the unrolled positions: 222.4 M in their own classes, plus rows 0 and 1 inside the S class;
- the 16 detours: 16.4 M.

Rows and first-segment detours together come to about 286.7 M: the unrolled positions, 20
instructions per tile for rows 0 and 1, and the detours. The descriptor's 10-instruction loop
and its detour would be 303.5 M on the same counts, so the unrolled form is about 16.8 M fewer
instructions. That is an integration effect: no component's design asks for it, it depends on
this compiler's unrolling decision, and it is counted where it lands (§5).

### 4.2 Each component's compiled paths: reproduced or deviated

| path (class) | B2 | component's isolated | combined | |
|---|--:|--:|--:|---|
| **scanner:** fresh pair (A) | 10 | 2 | 2 | reproduced |
| lookahead pair (J); matched lookahead (J − H) | 9; 0 | 5; 4 | 5; 4 | reproduced |
| uniform tile test (U = J + Q) | 5 | 0 | 0 | reproduced |
| held reuse (H) | 4 | 3 | 3 | reproduced |
| outer scan iteration (S + R) | 13 | 11 | 11 | reproduced; the descriptor's 17 (+4 head reloads) is **not** |
| nonuniform full tile (S) | 38 | 37 | 44 | **deviation:** the class now holds unrolled rows 0 and 1 (20 instructions, with the i recompute); its non-row part is 24 against the scanner's 37 |
| second segment (X) | 8 | 4 | 6 | **deviation +2** against the scanner |
| first-segment refill detour | 10 | 12 | 13 or 17 per position | **deviation** (§4.1) |
| second-segment refill detour | 9 | 11 | 11 | reproduced (the descriptor's is 8) |
| run (R) | 32 | 31 | 34 | **deviation +3** against the scanner (the descriptor's is 35) |
| `complete` | 43 | 44 | 45 | **deviation +1** against both (44, 44) |
| epochs | 33 | 35 | 60 | B2 + 2 (scanner) + 20 (descriptor) + 4 (v3's tag) = 59: **+1** |
| blocks | 204 | 200 | 205 | B2 − 4 (scanner) + 7 (descriptor) = 207: **−2** |
| tails; tail refill (first) | 41; 12 | 39; 8 | 39; 9 | tails reproduced; **+1** on the tail's refill |
| **descriptor:** second-segment row loop | 12 | 10 | 10 | reproduced: 4 reads and 2 writes, no reload |
| tail row loops | 12, 12 | 10, 10 | 10, 9 | reproduced; the tail's second segment is 9 (the scanner's counter form, merged) |
| first-segment rows | 12 | 10 (a loop) | 9 per position (unrolled) | **deviation** (§4.1) |
| `refill`; its `finish` call | 59; 2 | 65; 3 | 65; 3 | reproduced (the 6-per-refill pushes, the third reset store and the second `r0` store are still there) |
| snapshot loops | 14 × 128 | 0; 28 × 63 | 0; 28 × 63 | reproduced |
| final sweep (head; body); initialisation loop | 3; 13; 35 | 2; 14; 31 | 2; 14; 31 | reproduced |
| `fill` per call | 25 | 24 | 25 | **not reproduced:** B2's 25 |
| runs ending after a fill or after `complete`; open from run | 2; 2; 34 | 0; 0; 33 | 0; 0; 33 | reproduced |
| descriptor initialisation library calls | one 10,240-byte `memcpy` | two `memset`s and a 6,144-byte `memcpy` | the same three | reproduced (§5.4) |
| **v3:** the leaf | 81 (fast full, report 227) | 44 | 44 | reproduced in count. 5 instructions re-addressed: `lea (%rsi,%rsi,2)` and `add $0x1040` for `&cold[c]`, `mov 0x40(%rdi,%rsi,8)` for `wr[c]`. 12 explicit reads and 6 writes, as v3 |
| `finish_slow` | 379 (B2 `finish`) | 378 | 378 | reproduced: 7 instructions re-addressed (`gate2/finish-slow-vs-v3.diff`) |
| `active_tag` per epoch | — | 4 | 4 | reproduced (`shl $0x18`, `add $0x7f000000`, `or $0x80000000`, the store) |
| `open`'s tag load (refill; run) | immediate | one for one | one for one | reproduced (`refill` 74 = the descriptor's 74; open-from-run 33 = the descriptor's 33) |
| **new:** `finish` call from `run`'s close (508,988) | 5 | 5 (each) | 6 | **+1:** a reload of the cold-row pointer (`mov 0x8(%rsp),%r10`) after the call |
| **new:** final close loop finish call (23,559) | 8 | 9; 8 | 7 | −1 |
| **new:** crossing scan and final close loop (111,104) | 37 | 37 | 36 | −1 |

### 4.3 Spills, reloads and memory dependences (`gate2/stack.txt`)

**Dynamic stack traffic** of `build::<false>` on the primary corpus, from the plain
`disp(%rsp)` operands:

| build | slot loads | slot stores |
|---|---|---|
| combined | 36,437,451 → 31,723,527 (−4,713,924) | 15,524,890 → 14,886,006 (−638,884) |
| scanner alone | −505,885 | +3,391,432 |
| descriptor alone | −1,892,187 | +1,328 |

The indexed `(%rsp,%reg,8)` loads, +122.9 M, are the `wr`/`en` row accesses. In B2 the same
accesses go through a base register, so this is a reclassification and not new traffic.

In `refill`, the descriptor's pushes and pops are reproduced: +3,898,434. In `finish`,
push/pop falls by 19,768,500 and slot stores by 3,294,750 (v3's leaf has no frame).

**Per-row paths:**
- No new memory dependence and no reload inside any row.
- The unrolled rows make the descriptor loop's 4 reads and 2 writes per row.
- The per-tile i recompute's 2 stack reloads are the scanner trial's.

**New spills and reloads:**
- The three-register spill and reload around each of the 14 middle-position refill detours
  (§4.1).
- The cold-row pointer reload after `run`'s finish call (+1 × 508,988).

**Register allocation:** the scan state, the cold-array base (`0x10e8(%rsp)` into `%r8`) and the
descriptor arrays' stack offsets (`0xe8(%rsp,…)`, `0x8e8(%rsp,…)`) are allocated differently
from each component. The static frame of `build::<false>` holds 270 stack loads and 108 stack
stores, against B2's 127 and 62, mostly in the unrolled positions' detours.

## 5. Gate 3: the combined whole-constructor ledger on B2

### 5.1 The method (§5, counted once and from scratch)

**The binary:** `c4e45d30`, rustc 1.97.0 (`2d8144b78`, LLVM 22.1.6), `--locked --release`,
`CARGO_ENCODED_RUSTFLAGS=` empty, `CARGO_TARGET_DIR=target/rust-1.97.0`, sha256
`c21ec7416937a6494a355c8683e0efd88141c977f96f37dc70aad25ede8230dd`. B2's `build::<false>`,
`refill` and `finish` are compiled into the same binary and equal `cd14fa9` (§4).

**The profiles:**
- Valgrind 3.22.0 Callgrind, `--cache-sim=yes --dump-instr=yes`, each builder alone in its own
  process: `ctor2c-count --only v2|c corpora/wikidata-20261026.bz2`, the primary corpus,
  434 blocks.
- Order: v2 run 1, combined run 1, v2 run 2, combined run 2. All four raw profiles are in
  `callgrind/`.
- Both builders' outputs have FNV-1a `4efb1c13ae58340f`, the components' digest.
- **The two runs are identical** in every function, call site and class (asserted).

**The ledger** (`tools/comb_replay.py`, the components' `f01_replay.py` generalised; the S62
parser is copied unchanged):
1. The closure is every `c30walk` function reachable from `build::<false>` through the
   profile's call edges.
2. B2's `finish` is paired with the combined leaf + `finish_slow`, as one group.
3. Every other closure function (`canonicalise`) must be equal in Ir, Dr and Dw, which is
   asserted.
4. Within each changed group (`build`, `refill`, `finish`), every own instruction is grouped by
   its exact execution count, and a class whose instruction count changes is a changed path.
   That covers every changed caller, callee, spill and library call (§5.3(a)).
5. Each class needs a counted-build expression equal to its frequency (§5.3(b)).
6. Library call sites are compared site by site. Added sites are charged at the lower end;
   removed sites are credited at the upper end only.
7. The identity inclusive Δ = Σ own Δ + added library − removed library is asserted.

### 5.2 Counters on every changed path (§5.3(b))

**The union of the components' counters covers every class but these.** The new counters are
incremented only under `if COUNT` (in `build::<true>`):

| new counter | path | why it is new |
|---|---|---|
| `tile_n1[n1]` (n1 = 2, 4, …, 16) | the unrolled first segment: rows 2j − 2 and 2j − 1 run on the tiles with n1 ≥ 2j | the unrolling (§4.1) gives each position pair its own frequency |
| `tile_refills_first_at[k]`, k = 0..15 | each position's refill detour | as above |
| `run_finishes` | the `finish` call from `run`'s close (508,988) | the call site changed (+1) in the combined build only |
| `crossing_scan`, `final_closes` | the crossing pre-marking scan of the 256 counts, and the final close loop: two paths in the class at 111,104 | it changed (37 → 36) in the combined build only |
| `norm_partial_scalar` | `finish_slow`'s partial-normalisation scalar remainder: k when k < 4, else k mod 4, from the compiled loop (16-wide while k ≥ 16, then 4-wide) | the class at 61 changed (3 → 2) |

The harness asserts their partitions: Σ `tile_n1` = S, n1 even in 2..=16,
Σ `tile_refills_first_at` = `tile_refills_first`, and `crossing_scan` = `final_closes` =
256 × blocks.

**The uncounted build's normalised closure is unchanged.** `tools/asmid_c.py` compares the
profiled binary with the final counted-counter binary (`591c84f7`, `bc5fdae6…`):
- **The tool:** the components' `asmid.py`, with one change. objdump names a RIP-relative
  constant by its nearest preceding symbol, and the counters renumber the `GCC_except_tableN`
  labels. So that name is masked, and every RIP-relative operand is compared instead by the 16
  bytes it addresses, read from each binary.
- **The result:** 53 functions and 4,509 instructions, from `build::<false>` through its
  transitive callee closure, are **identical**: `build::<false>` 1,784, `refill` 74, `finish`
  44, `finish_slow` 378, `canonicalise` 173, and the allocator and panic paths. The four
  archived listings are identical too.
- **Without the change:** the unmodified `asmid.py` reports 23 differing lines in 10 functions
  (`identity/kept-comments/`). Every one is a `GCC_except_table992`/`993` comment and nothing
  else.
- **The control:** on `build::<true>` the tool reports DIFFERENT.

**The counted record:** `counted/sequential.jsonl`, the counted build alone in the profiled
process's order (`--counted`), with output digest `4efb1c13ae58340f`. **Every changed class's
expression equals its profile frequency exactly**, including each path of the multi-path
classes:
- 230: the tails, and the bitmap fill;
- 111,104: the crossing scan, and the final close loop;
- 1: the rec regrowth free, the snapshot reserve, and the one-word bitmap reserve.

The suites on the final binary are exact (§3.1).

### 5.3 The result

`ledger.txt` and `ledger.json` list all 83 changed classes. Grouped:

| term | Δ instructions |
|---|--:|
| `build::<false>` own (70 classes, all counter-supported) | **−207,720,906** (1,026,601,670 → 818,880,764) |
| `refill` own (2 classes) | **+9,059,803** (80,567,464 → 89,627,267) |
| `finish` group own: B2 `finish` → leaf + `finish_slow` (11 classes) | **−60,212,114** (176,758,551 → 116,546,437) |
| **own code, counter-supported** | **−258,873,217** |
| uncounted classes | 0 added, 0 removed |
| `canonicalise` | 0 (3,903,384 both; equal Ir, Dr, Dw) |
| library, 11 shared sites | 0 (equal site by site, in both runs) |
| library, added: two 2,048-byte `memset`s and the 6,144-byte `memcpy` (434 each) | +378,882, charged at the lower end |
| library, removed: the 10,240-byte `memcpy` (434) | −4,454,576, credited at the upper end only |
| **inclusive `build::<false>`** | 1,294,434,062 → 1,031,485,151, **−262,948,911** |

**The conservative interval: [258,494,335, 262,948,911] removed.**
- **The lower end** is the counter-supported own delta, with the added library calls charged and
  the removed `memcpy` not credited.
- **The upper end** is Callgrind's exact inclusive difference.
- **The width,** 4,454,576, is exactly the removed `rep movsb` copy's per-byte count convention.
  The descriptor's archive has 4,447,632 for the same copy in its own binary.
- **Frequencies:** every one is a counted value, so there is no bounded frequency to take at an
  end.

**The largest terms,** from `ledger.txt`:

| term | Δ |
|---|--:|
| B2's first-segment loop | −352.3 M |
| the unrolled positions | +222.4 M |
| the S class | +14.4 M |
| the 16 detours | +16.4 M |
| B2's single first-segment detour | −10.0 M |
| the scanner's U, J and A paths | −52.7 M, −42.2 M (with +39.7 M for J − H), −19.2 M |
| the second-segment rows | −17.9 M |
| the leaf's | −52.1 M, −33.3 M, −32.7 M and −15.5 M removed, against +52.7 M and +11.5 M added, and +9.2 M for `finish_slow` |
| `refill` | +9.06 M |

**The caller boundary (S62-F04):**
- `c30walk::main`'s own count rises by 868 = 2 × 434 (`harness-callsite.asm`).
- The cause is the `--only` dispatch. The combined arm makes `cmp $0x1,%rdx; jne;
  cmpb $0x63; jne` where v2's makes `cmpw $0x3276; jne`.
- The 10-instruction call setup is identical in both arms, so this is not constructor work.
- **If it were charged anyway,** the interval would be [258,493,467, 262,948,043].

### 5.4 Writes and reads (§5.3(g), reported, not gated)

| own code | Δ reads | Δ writes |
|---|--:|--:|
| `build::<false>` | −5,217,308 | −833,968 |
| `refill` | +2,562,499 | **+5,197,912** |
| `finish` group | −17,824,829 | −13,179,000 |
| **total** | **−20,479,638** | **−8,815,056** |

**`refill`'s +5,197,912 writes** are the descriptor prototype's 4 per refill: two pushes, the
third reset store and the second `r0` store. Its +2,562,499 reads are the descriptor's `pop`
(+1,299,478) plus v3's `active_tag` load at each of `refill`'s openings.

**Across the constructor, writes fall,** because v3's leaf drops the 12 push/pop of v2's
finish. Writes still rise on `refill`'s path, and that is a finding for the review.

**Inclusive** (with the library sites): reads −24,836,130 and writes −13,113,826. The removed
`memcpy` alone is 4,443,292 reads and 4,441,556 writes, counted per byte; the added sites are
+86,800 reads and +142,786 writes.

## 6. The floor F and the overlap O (D010)

### 6.1 O from the known minimum

**S62-F02's shared B2 instruction:** `mov 0x8(%rsp),%r13` at build +0x12b5 (the `ll.len()`
reload in the nonuniform tile's second-slice setup), executed S = 2,393,313 times. Both isolated
ledgers credit its removal. This is **the registered known minimum, and O's lower bound for the
conservative floor.**

### 6.2 Further demonstrated overlap (`tools/overlap.py`, `overlap-classes.json`)

`overlap.py` puts the scanner's and the descriptor's archived class tables side by side, each
against its own same-binary B2 profile. Four B2 classes are ones that both isolated ledgers
credit with a removal. Their total bound is 2,397,073:

| B2 class | B2 → scanner → descriptor | the B2 instructions | demonstrated shared removal |
|---|---|---|--:|
| 2,393,313 (S) | 38 → 37 → 37 | S62-F02's +0x12b5 reload | **2,393,313** |
| 3,575 (epochs with a full tile) | 5 → 4 → 3 | B2 `xor; lea; mov %r12,%r9; mov 0x38(%rsp),%rcx; jmp`. The scanner keeps `xor, lea, jmp` and adds one instruction. The descriptor keeps `xor, lea, jmp` | **3,575**: both drop `mov %r12,%r9` and the `0x38(%rsp)` reload. The scanner's one credited removal is one of those two, and the descriptor credits both |
| 79 (epochs with no full tile) | 3 → 2 → 0 | B2 `lea; mov %r12,%r9; mov 0x38(%rsp),%rcx`. The scanner drops the reload only; the descriptor drops all three | **79** (the reload) |
| 106 (the tail's second-segment rows) | 12 → 11 → 10 | the scanner removes the loop's `cmp` and label `lea` and adds an `inc`. The descriptor removes the two descriptor LEAs | **0**: the removed sets are disjoint |

So **O ≥ 2,393,313 + 3,654 = 2,396,967** demonstrated, between the scanner and the descriptor
arrays.

**Overlap involving v3: none demonstrable, so 0 is subtracted.**
- v3's isolated ledger credits only B2's `finish`: the fast full path, 81 → 44. Its `build` and
  `refill` entries are costs only (+4 per epoch, and the opening one for one).
- Neither isolated profile changes any B2 `finish` class: `overlap.py` prints empty changed sets
  for both, and both components' `f01_replay.py` assert it.
- So no B2 instruction is credited both by v3 and by another component.

### 6.3 F and the pass conditions

| | with v3 | without v3 |
|---|--:|--:|
| Σ translated minimums | 150,000,000 | 100,000,000 |
| **F, with O = 2,393,313** (the registered known minimum) | **147,606,687** | 97,606,687 |
| F, with O = 2,396,967 (including §6.2's further 3,654) | 147,603,033 | 97,603,033 |
| combined lower end | **258,494,335** | not assembled |
| **(i)** lower end ≥ F | **yes**, margin 110,887,648 | — |
| **(ii)** lower end > 84,060,421 (the scanner U2's isolated lower bound, the best single component's) | **yes**, margin 174,433,914 | — |

**The further 3,654 doesn't change either outcome.** Whether it enters O is the coordinator's
call; the conservative floor above doesn't use it.

## 7. v3 under §7

**(a) Safety:** accepted in the §6.2 review. It is re-checked here on the combined code: Miri's
`v3:*` classes (§3.3), V3M (§3.4), and the argument in §3.2.

### 7.1 §7(b): v3's contribution, re-derived from the combined listing

`tools/floor.py` gives these terms:

| term | instructions |
|---|--:|
| the `finish` group's class delta: B2's `finish` against the combined leaf + `finish_slow`, every class × its counted frequency (all 11 counter-supported). It includes B2's fast full finish against the 44-instruction leaf on 1,647,375 leaf finishes, the leaf's 5-instruction tag-exit prefix on all 148,107 slow entries, the 7-instruction length test on 41 same-epoch partials, and `finish_slow` against B2's slow paths | **−60,212,114** |
| `active_tag`: 4 instructions per epoch × 3,654 (the `shl`/`add`/`or` and the store, in the combined epoch class) | +14,616 |
| `open`'s tag load (refill and run) | 0: one for one, since `refill` (74) and open-from-run (33) equal the descriptor's without v3 |
| the added reload at `run`'s finish call site, +1 × 508,988. v3's isolated call sites equal B2's, so this is an integration effect; it is **charged to v3** here at its least favourable end | +508,988 |
| **v3's contribution, lower end** | **59,688,510** |
| §7(b)'s minimum | 50,000,000 |

**§7(b) is met, by 9,688,510.**

**The cross-check:** (81 − 44) × 1,647,375 = 60,952,875 gross. v3's isolated net was
60,197,437 ± 148,107. The class-exact finish delta here, 60,212,114, lies inside that band.
Nothing here credits the final close loop's −1 (23,559) to v3.

### 7.2 §7(c): the leaf

`listings/ctor2c-finish-false.asm`: **44 instructions from entry to `ret`**, the same as v3's at
`dbf5139`. **§7(c) is met, with no margin:** the bound is ≤ 44, and the leaf is equal to it.
- Five instructions differ from v3's (`gate2/finish-vs-v3-leaf.diff`). They are the address
  computation for the split storage.
- The chunk-0 materialisation that cost v3 its compiler gate is unchanged.

### 7.3 Without v3 (not assembled)

v3 meets (b) and (c), so no component was dropped and no second assembly was made.
**Without v3, F = 97,606,687.**

An unmeasured estimate puts the without-v3 lower end at about 258.49 M − 59.69 M ≈ 198.8 M.
That is this ledger less v3's §7.1 contribution. It assumes the scanner's and descriptor's code
would compile as it does here without v3, which is untested given §4.1's unrolling. **It is a
model, not a ledger, and it decides nothing.**

## 8. What didn't go to plan

1. **The first profiling launch failed before running.** The script called `/usr/bin/time`,
   which isn't installed. It was relaunched without it; no profile was lost or mixed.
2. **The unrolling (§4.1) created 23 classes no component counter covered.** The counters of
   §5.2 were added after profiling, and the uncounted closure was shown unchanged against the
   profiled binary.
3. **The first `norm_partial_scalar` formula was wrong.** It assumed a 4-wide loop entered at
   k ≥ 8, and counted 81 against the profile's 61. The compiled loop compares 2k bytes (k ≥ 4
   enters the vector path, 2k ≥ 32 the 16-wide one). The corrected formula counts 61, and the
   identity check was rerun.
4. **MU1 was first caught by the wrong check.** The new counter `tile_n1[n1]` stood before U2,
   so the counted build panicked on its checked index (len 17, index 17) before reaching U2:
   caught, but not at the intended operation. The counter was moved after U2 (`591c84f7`,
   counted-only; the identity check, the suites and the counted record were rerun). MU1 is now
   reported at U2. The first, mis-located MU1 consoles were superseded and aren't archived.
5. **One mutation-tool slip.** While the mutation patterns were checked, a `git checkout`
   reverted the uncommitted counter move. It was reapplied and committed before the archived
   mutant runs.
6. **The descriptor's arena-history fields in the threaded corpus records** differ from its
   archive (§3.1), as its own §10.1 explains. The ledger uses the sequential record.

## 9. Limitations

- **Only the primary corpus is ledgered.** The other corpora are run for correctness.
- **Coverage rests on the listing mapping.** Frequency classes show net changes per frequency,
  so equal-frequency additions and removals within one class can cancel before enumeration
  (D011's caution).
  - The mapping in `tools/comb_classes.py` was read from the listings with per-instruction
    counts (`--listings`, which prints each class's instructions).
  - The multi-path classes are split.
  - The counter equalities alone don't prove coverage.
- **The unrolling is this compiler's decision** (rustc 1.97.0, LLVM 22.1.6, x86-64). The saving
  it brings over the components isn't a property of any registered design.
- **Miri covers the registered classes,** not the release suites. Large blocks, wide records and
  poisoned arenas are release-only, as in each component.
- **The profiled binary's clean rebuild reproduces its code, not its hash.** `c4e45d30` was
  rebuilt clean in a second worktree with the same recipe (`rebuild/`):
  - the `.text`, `.rodata`, `.data.rel.ro` and `.eh_frame` sections are byte-identical;
  - every symbol's address and size is identical;
  - the normalised closure of both `build::<false>` (combined and B2) is identical.

  The whole-binary sha256 differs (`37ab15f2…` against `c21ec741…`). The build directory's path
  is embedded in the debug information, and it occurs in the rebuilt binary's strings.
- **Static and simulated counts only.** Nothing here says how fast anything runs.

## 10. Files and commands

Evidence:
[2026-11-04-compact30-combined-build/](2026-11-04-compact30-combined-build/), with its
`README.txt`, which gives every replay command.
