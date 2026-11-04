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

## 1. Checkpoint (charter §9)

> **Erratum (§8, after review of PR #146):** the interval [56.18 M, 56.21 M] in §5.3 is
> withdrawn as a conservative interval. The claim is the own-body −56,195,817. The gate
> outcomes are unchanged.

**Outcome: prototype 1 does NOT qualify.**

| gate | role | outcome |
|---|---|---|
| 1, correctness and safety | qualifier | **PASS**: every suite exact, both mutants caught, the release-mode argument written, and all 8 registered Miri classes complete with no undefined behaviour |
| 2, machine code | diagnostic | **missed in part** (§4): the fresh-pair (4) and lookahead (11) targets met; the nonuniform-tile target, row-loop identity, spill-free scan and no slice checks on full tiles missed |
| 3, net work on B2 | qualifier | **FAIL: 56.20 M** removed against the 60 M minimum (§5) |

**Escalation.** The coordinator's brief (§0) asks for one if the safe forms miss the compile
target. They do: the nonuniform tile's checks remain. So this line STOPS as a §8.2 escalation,
with the unsafe operations that would be needed set out in §7. No unsafe variant was written.

**The pin:** branch `research/compact30-scanner`, from `d055c51`, at this entry's commit.
- **The prototype:** `examples/c30walk/ctor2s.rs` (new), plus the `mod` line and the
  `ctor2s-` dispatch in `examples/c30walk.rs`.
- **Frozen files, each unchanged** (`git diff` empty):
  - `ctor2.rs` against `8beffe2`;
  - `ctor3.rs` against `dbf5139`;
  - `ctor.rs`, the walker, the oracle, `src/`, `cbuild.rs` and `bench/`.

**Untimed.** Nothing was timed or benchmarked, and no perf counter was used. The exact counts
in §5 come from callgrind, which simulates instructions without timing them (§5.1). That is an
independent cross-check beside the charter's method, and is labelled as such.

## 2. The three attempts (all safe Rust; listings in `attempt-*/`)

**All three replace v2's `rd` reads in the same way.** Each epoch's unread rows are a slice,
`rest`. A full tile is `rest.split_first_chunk::<16>()`, so the language's own length test is
the loop guard. Its two qwords are read from the `[u8; 16]` at constant offsets (`halves`), and
a run's lookahead does the same on the remainder. A held tile stays at the front of `rest`.

| attempt | the nonuniform full tile | `build::<false>` | the row loops (v2: 12 each, four copies) | verdict |
|---|---|---|---|---|
| 1 | `tile16`: `split_at(n1)` on the 16-byte array, no slice of `ll` | 0x177d, 1,355 instructions | **13, 13, 12, 13**: the descriptor base is rematerialised in the loop (`lea 0xe8(%rsp)`), by register pressure | the row-loop regression costs ≈ 1 × the scalar rows on these paths |
| 2 | the same, with `rest = &rest[16..]` re-derived after the tile instead of keeping `after` live | code differs elsewhere; **row loops as attempt 1** | 13, 13, 12, 13 | no improvement |
| 3 (kept) | v2's own `tile(ll, i, 16)`, at i = `end − rest.len()` | **0x18a6, 1,414 instructions** | **12, 12, 12, 11** (the tail's second segment counts down to zero) | the variant checked below |

Attempt 3 gives up C3 and C4 by design, since v2's tile keeps its slice checks and its phase
test. Its row loops have v2's instruction counts, though not v2's registers: the descriptor
base is in `%rdx`, not `%r8`.

## 3. Gate 1: correctness and safety

**The suites,** each case as v2's `check_block` runs it:
1. the oracle;
2. the counted and uncounted builds, in poisoned reused arenas;
3. **v2's counted build on the same input, with every counter equal field by field,** all
   930 portion-shape cells included;
4. **the scan-path identities on every case:**
   - fresh + lookahead = ⌊n/16⌋;
   - nonuniform = ⌊n/16⌋ − run rows/16;
   - runs = held + no-full-tile;
   - fresh = nonuniform + no-full-tile;
   - tail rows = n mod 16;
   - scalar rows = 16 × nonuniform + tail rows;
5. the counted and audited walks against production and the oracle;
6. the arrays again after each walk.

| suite | cases | array comparisons | walk outputs | restorations | result |
|---|--:|--:|--:|--:|---|
| 11 corpora (sha256 = the census's) × 4 configurations | 44 | 12,104 | 18,156 | 12,104 | **exact** |
| the adversarial set | 200 | 1,056 | 1,584 | 1,056 | **exact** |
| full synthetic: v2's classes and the 8 registered scanner classes | 54,029 | 99,966 | 149,949 | 99,966 | **exact** |
| quick synthetic | 629 | 1,170 | 1,755 | 1,170 | **exact** |

**The rest of gate 1:**
- **Malformed counts:** v2's class gives 3,995 rejected and 7 accepted, exactly when the counts
  equal L's own, both builds agreeing. The scanner's own malformed class rejects in both builds,
  with both arrays empty.
- **Reuse:** v2's repeated-size class, growing and shrinking, n = 0 through both builds, exact.
- **The mutation checks** (§0.5): **both caught.**
  - **M1** (the held tile skipped): the quick suite panics at "small n 32 sigma 3: the counted
    build rejected valid counts". Exit 101.
  - **M2** (the run short by one tile): it panics at "small n 16 sigma 1: …". Exit 101.

  Both are in `mutation-M*.console`, with their diffs.

**The release-mode argument:**
- **No new `unsafe` operation.** The scanner's source accesses are:
  - `ll[i..end]` once per epoch;
  - `split_first_chunk::<16>()` on the remainder;
  - constant-offset `split_at(8)` and `try_into` on a `[u8; 16]`;
  - v2's `tile`, with its checked slicing.

  Every one is safe Rust, so an out-of-bounds index panics rather than reading. No pointer,
  `get_unchecked` or `read_unaligned` is introduced.
- **Bounds and overflow.**
  - `rest` is always a suffix of `ll[i₀..end]`, with end ≤ n = `ll.len()`.
  - The tile's index i = end − `rest.len()` can't underflow, since `rest.len()` ≤ end − i₀.
  - i + 16 ≤ end follows from `rest.len()` ≥ 16.
  - For the tail, i + `rest.len()` = end.
- **Initialisation:** `ll` is a `&[u8]`, so every byte read is initialised.
- **The destination side** (`one`, `refill`, `run`, `fill`, `complete`, `finish`) is v2's
  unchanged code. Its obligations are v2's, discharged by v2's audited argument.
- **Correctness facts** (not safety):
  - a held tile is the next tile;
  - a run's length is its tiles' rows;
  - the run lengths, splits and tile indices equal v2's.

  These are checked by the counter equality on every case and by mutants M1 and M2.

**Miri: all 8 registered classes complete, every check exact, no undefined behaviour
reported.**
- **The command:** `cargo +nightly miri run --example c30walk -- ctor2s-synthetic --miri
  CLASS`, one invocation per class, run one after another.
- **The toolchain:** nightly rustc 1.101.0 (`16d1ffac1`), Stacked Borrows, unpoisoned arenas.
- **Every case also ran v2's counted build** and the field-by-field counter comparison. The
  consoles are in `miri/`.

| class (§0.4) | cases | exit | fresh | lookahead | held | no full tile | nonuniform | second segment | tails |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| 1. tiny and empty (n = 0, 1, 2, 15, 16, 17, 31, 32, 33; σ 1, 2) | 18 | 0 | 12 | 2 | 0 | 5 | 7 | 2 | 12 |
| 2. every tail length 1..=15: alone, after a nonuniform tile, after a run | 45 | 0 | 30 | 15 | 0 | 15 | 15 | 0 | 45 |
| 3. odd alignment, offsets 1..=7, tiny and tiled | 14 | 0 | 14 | 35 | 14 | 0 | 14 | 0 | 14 |
| 4. the phase sweep, every reachable p (0, 2, …, 28) | 15 | 0 | 15 | 105 | 14 | 0 | 15 | 7 | 15 |
| 5. held tiles (another byte, nonuniform, first or second qword only, alternating runs) | 5 | 0 | 8 | 20 | 12 | 1 | 7 | 3 | 5 |
| 6. runs ending with no full tile left | 2 | 0 | 4 | 6 | 0 | 2 | 2 | 0 | 1 |
| 7. the epoch boundary: a run straddling 61,440, and a nonuniform tile ending at it (61,504 rows each) | 2 | 0 | 3,841 | 3,847 | 2 | 2 | 3,839 | 1,792 | 0 |
| 8. malformed counts, both builds, empty arrays | 4 | 0 | — | — | — | — | — | — | — |
| **total** | **105** | | | | | | | | |

The counter columns are the classes' own scan-path totals, as each console prints them. Its
column order is: fresh, lookahead, held, no-full-tile runs, nonuniform, second segments,
tails.

## 4. Gate 2: the compiled counts against target (diagnostic)

The binary is `e2e6286a…8049fac` (and, after a doc-comment-only edit, `b36885a7…1050ad53`, with identical listings at identical addresses), rustc 1.97.0, `--locked --release`, empty rustflags. The
static paths in the final listing (`listings/ctor2s-build-false.asm`, at 0x41130):

| path | B2 | target | prototype | |
|---|--:|--:|--:|---|
| fresh pair: held test, branch, two loads (`+0x928`–`+0x930`) | 12 | 4 | **4** | **met** |
| lookahead per matched tile: the loop `+0x970`–`+0x999`, its guard included | 16 | 11 | **11** | **met** |
| nonuniform full tile's overhead, outside its 16 row appends | 41 + 7X | 34 + 7X | 41 + 7X **+ 8** (exact, §5) | **missed** |
| row loops | 12 × 4 | identical | 12, 12, 12, 11, with the descriptor base in another register | **missed** (counts equal or fewer; not identical) |
| `slice_index_fail` unreachable from full tiles | — | yes | no: the nonuniform tile keeps v2's checks | **missed** |
| no new spill or reload in the scan loop | — | yes | no: +4 per first-segment refill, +3 per second-segment refill, +3 per run | **missed** |

`finish` and `refill` are identical to v2's (normalised), and their call sites keep their
form.

## 5. Gate 3: the net-work ledger on B2

### 5.1 The method

- **The charter's method** (§5.1) is Σ over compiled paths of their instructions × observed
  execution counts.
- **The counts:**
  - the counted build's new counters give the scan paths' frequencies exactly on the
    primary corpus (§5.2);
  - **callgrind** (valgrind 3.22, `--dump-instr=yes`, and `--cache-sim=yes` for reads and
    writes) gives each instruction's exact execution count in the uncounted build. It runs
    `ctor2s-count`, which builds each of the 434 blocks once with v2's `build::<false>` and
    once with the prototype's, and compares their outputs.
- **What callgrind is and isn't.** It is a deterministic instruction simulation: untimed, not
  a perf counter, and not a benchmark. It makes Σ instructions × executions exact for the
  whole function, which is what §5.3a asks for. The two sources agree: the instructions that
  execute A, J, S, R or H times execute exactly the counters' values.

### 5.2 The frequencies, from the counted build (primary corpus)

| A fresh | J lookahead | H held | Q no full tile left | S nonuniform | X second segment | R runs | tails |
|--:|--:|--:|--:|--:|--:|--:|--:|
| 2,396,084 | 10,541,706 | 607,750 | **2,771** | 2,393,313 | 1,116,556 | 610,521 | 230 |

Q is now known exactly, inside the handoff's bound [0, 3,450]. So the identities A = S + Q and
J = U − Q, with U = 10,544,477, hold exactly.

### 5.3 The result

> **Erratum (§8):** the callee difference below is not an allocator bound, the interval is
> withdrawn as conservative, and the inclusive read and write deltas are CONDITIONAL. See §8.

| | B2 (v2) | prototype | difference |
|---|--:|--:|--:|
| `build::<false>`'s own instructions (deterministic, both runs) | 1,026,601,670 | 970,405,853 | **−56,195,817** |
| including callees (refill, finish, canonicalise and the allocator), run 1 / run 2 | 1,294,414,196 / 1,294,422,962 | 1,238,210,529 / 1,238,217,311 | −56,203,667 / −56,205,651 |
| explicit reads, `Dr`, including callees (run 1) | 275,023,421 | 271,632,049 | −3,391,372 |
| explicit writes, `Dw`, including callees (run 1) | 182,174,544 | 184,169,209 | **+1,994,665** |

- `refill` and `finish` are identical in both: 201,234,507 and 177,278,484 instructions
  inclusive.
- **The callee difference** (−7,850 and −9,834 in the two runs) is the allocator's
  heap-state variation: the same calls, different internal paths. It is an unresolved term
  of at most about 10 k, listed under §5.3e.
- **The conservative interval** is **[56.18 M, 56.21 M]**: the deterministic −56,195,817
  allowing for that allocator term. Its lower end is **below the 60 M minimum. Gate 3
  fails.**

**Where the saving went:** the exact decomposition by execution-count class, B2 against the
prototype.

| event class (its count) | instructions on it, B2 → prototype | Δ instructions |
|---|---|--:|
| uniform tiles U (10,544,477): v2's lookahead guard and checks | 5 → 0 | −52,722,385 |
| lookahead pairs J (10,541,706) | 9 → 5 | −42,166,824 |
| matched lookaheads J − H (9,933,956): the new loop's advance and guard | 0 → 4 | +39,735,824 |
| fresh pairs A (2,396,084): C1 | 10 → 2 | −19,168,672 |
| outer iterations S + R (3,003,834) | 13 → 11 | −6,007,668 |
| nonuniform tiles S (2,393,313): v2's tile checks, now unmerged with `rd`'s, and recomputing i | 38 → 46 | **+19,146,504** |
| refills from a tile's first segment (996,634): spills and reloads | 10 → 14 | +3,986,536 |
| refills from a tile's second segment (302,049) | 9 → 12 | +906,147 |
| runs R (610,521) | 32 → 35 | +1,831,563 |
| one executed alignment `nop` (610,356) | 0 → 1 | +610,356 |
| other classes (fills, opens, completes, held exits, run returns) | | −2,347,198 |
| **total** | | **−56,195,817** |

**Against the handoff's design:**
- **The credited gross was 88,622,080 + 3Q,** which with the exact Q is 88,630,393.
- **The safe re-expression removes the C1 and C2 checks** that the design credits:
  8A + 5J = 71.9 M, achieved through the new loop shape.
- **It does not remove C3 or C4** (7S = 16.75 M of the credit).
- **It adds back:**
  - +8 per nonuniform tile. v2 merged the tile's i + 16 ≤ len check with `rd`'s; without
    `rd`, the tile runs it on its own.
  - spills and reloads around refills on tile rows;
  - 3 per run.

### 5.4 Overlap (charter §5.5)

- **This prototype's B2 listing diff** removes:
  - the fresh pair's checks, `33ae8`–`33b06`;
  - the lookahead's checks, `33b44`–`33b56`;
  - the lookahead loop's v2 guard form, `33b30`–`33b3e`, replaced.
- **It changes, without removing:**
  - registers and stack slots throughout the scan, `run`, `fill` and `complete`;
  - the refill detours.
- **It doesn't touch prototype 2's credited set,** the scalar loops' descriptor-address LEAs
  (`34355`/`34359`, `343e4`/`343e8`, `33a05`/`33a09`, `344d5`/`344d9`). Those LEAs are still
  present in all four row loops, in another register. **Instruction-level overlap with
  prototype 2's credit: none.**
- The full opcode-level diff is `listing-diff.txt`.

**The evidence in `2026-11-04-compact30-scanner-prototype/`:**
- `attempt-1/`, `attempt-2/` and `attempt-3/`: each attempt's `build::<false>` listing, and
  its source diff;
- `listings/`: the final `build::<false>`, `finish` and `refill`, and v2's `build::<false>`
  from the same binary;
- `listing-diff.txt`;
- the corpora, adversarial and synthetic consoles and records, and `corpora.sha256`;
- `mutation-M1` and `mutation-M2`: each mutant's console and diff;
- `miri/`: each class's console, and the toolchain;
- `callgrind/`: the per-function counts of both runs, the parser, and the per-instruction
  profile (gzipped).

## 6. Limitations and negative findings

- **The safe forms meet the read-side targets exactly, but a register-allocation
  side-effect on the tile path makes the net short.** The first two attempts put an
  instruction into three of the four row loops. The third avoided that by keeping v2's tile
  code, and with it v2's tile checks.
- **Reads fall (−3.39 M) and writes rise (+1.99 M),** from the new spills.
- **Callgrind's inclusive figure varies by about 10 k through the allocator.** The
  function's own count is exact and reproducible.
- **No runtime inference.** No cycles, micro-ops or timing are claimed.
- **Odd tile phases are unreachable.** The pre-registered phase class listed p = 0..=29, but
  tiles start at multiples of 16, so p = 16k mod 30 is even. The class covers every reachable
  phase (§0.4 is corrected here, before its Miri run, and the handoff's "n1 = 1..15" is in
  fact n1 = 2, 4, …, 14).
- **The registered epoch class runs as two blocks,** not one: a run straddling 61,440, and a
  nonuniform tile ending at it.

## 7. The escalation (§8.2): the unsafe operations that would be needed

**What the safe line can't do.** It doesn't reach the nonuniform tile's target, and it adds
spills. **No unsafe variant was implemented.** What follows is the exact formulation for the
user's decision.

**`read_unaligned` (the handoff's C1/C2 formulation) is NOT needed:** the safe
`split_first_chunk` form already meets both of those targets. The remaining gap is C3, C4 and
register allocation. It would need these operations instead, in the nonuniform full tile only:

| op | where | pointer / length | dominating invariant | overflow | validity and initialisation | expected saving |
|---|---|---|---|---|---|---|
| U1 | `tile`'s first segment, for a full tile | `ll.as_ptr().add(i)`, length n1 (`get_unchecked(i..i + n1)`) | `rest.len()` ≥ 16 at the split, so i + 16 ≤ end ≤ `ll.len()`; n1 = min(16, 30 − p) ≤ 16 for any p (wrapping included) | i + 16 ≤ n ≤ 983,010 < 2²⁰ | inside the live `&[u8]`, every byte initialised, immutable | with U2, the tile's 8 check instructions (jb; cmp, ja; add, cmp, ja; cmp, jb): up to 8 × S ≈ 19.1 M |
| U2 | `tile`'s second segment | `ll.as_ptr().add(i + n1)`, length 16 − n1 | as U1, n1 ≤ 16 | as U1 | as U1 | (with U1) |
| U3 | `tile`, the phase test (C4) | `core::hint::assert_unchecked(self.p < 30)` | the phase induction (handoff §2 C3): p only changes by `advance(k ≤ 30)` | — | undefined behaviour if false; the induction shows it never is | 2 × S ≈ 4.8 M, if the layout falls through |

**The Miri classes that would check them:**
- the registered phase sweep: every reachable p, both segment shapes;
- held tiles;
- odd alignment;
- the tail class (a tail must never take U1 or U2);
- the epoch class;
- a mutant with n1 forced to 17, which Miri must flag as out of bounds.

**What remains uncertain even then.** These remove checks, not register pressure. The first two
attempts removed C3 safely (`split_at` on the array) and lost more to spills in the row loops.
So whether U1–U3 clear 60 M net is unknown until compiled. On the arithmetic, 56.20 M + up to
23.9 M − whatever spills appear.

**This line stops here** pending the user's explicit approval (charter §12.3).

## 8. Erratum (2026-11-04, after the review of PR #146)

**Source:** the independent review of PR #146 (comment 6630265636), findings F1 and F2. Both
are accepted. **The gate outcomes don't change:** gate 1 PASS, gate 2 missed in part, gate 3
FAIL. The evidence for this section is in
[2026-11-04-compact30-scanner-prototype-erratum/](2026-11-04-compact30-scanner-prototype-erratum/).
`ctor2s.rs` and the evidence at `7f74e9f` are unchanged.

### 8.1 F1: the interval is withdrawn as conservative

- **What §5.3 claimed.** It cited **[56.18 M, 56.21 M]** as a conservative interval. It
  treated the two runs' callee differences (−7,850 and −9,834) as an allocator term of at
  most about 10 k.
- **Why that fails:**
  - two samples don't bound a term;
  - the callee difference isn't only the allocator: it includes **+1,840 `memcpy`
    instructions**, at the snapshot push (§8.3).
- **The interval is withdrawn as a conservative interval.**
- **The claim, restated:** `build::<false>`'s own instructions fall by **56,195,817**
  (1,026,601,670 → 970,405,853). This figure is exact, and it is reproduced in all four
  profiles: the two at `7f74e9f`, and the two regenerated here. It is achieved within that
  scope, the function's own body, and no wider.
- **Run 1's inclusive −56,203,667** is kept as an observation only. It is not a bound.
- **The inclusive read and write deltas are CONDITIONAL** on the unresolved library terms of
  §8.3: −3,391,372 reads and +1,994,665 writes. The own-body deltas are exact:
  - reads 205,899,121 → 202,509,646, **−3,389,475**;
  - writes 132,435,558 → 134,431,097, **+1,995,539**.
- **Gate 3 still fails.** The own-body 56.20 M is below the 60 M minimum. Because library
  terms remain unresolved (§8.3), no verified net saving is claimed beyond the own-body scope
  (charter §5.3e).

### 8.2 F2: the parser and the archive

- **The parser bug.** `callgrind/cgparse.py` (lines 22–25) assumes a single position column.
  The archived profile, `callgrind/per-instruction.callgrind.gz`, declares
  `positions: instr line`, with `*` compression, so the parser crashes on it. The crash is
  reproduced in `parser-test-archived-profile.txt`.
- The figures in `run1-functions.txt` and `run2-functions.txt` match the new parser exactly
  wherever both can be checked. That is consistent with their coming from `positions: line`
  profiles, which the old parser reads correctly. But those profiles weren't archived.
- **The archive's gap.** It held only Ir: the raw Dr/Dw profiles and the second run's profile
  were missing.
- **The fixed parser, `cgparse.py`, is header-aware.** It reads `positions:` and `events:`,
  and handles `*`, `+n`, `-n`, compressed names and `.gz` input. On the archived profile, it
  gives the same Ir as `run1-functions.txt`, self and inclusive, for every function listed.
  It also reads `positions: line` profiles (tested on prototype 2's).
- **The regenerated profiles.**
  - They come from a rebuild at `7f74e9f`: rustc 1.97.0, `--locked --release`, binary sha256
    `b36885a7c5d3e38c178886727ac14d6119a1835bba5e64704bc180861050ad53`. Its listings are
    identical to those archived, with 0 differences after normalisation.
  - Two runs were made of `valgrind --tool=callgrind --cache-sim=yes --dump-instr=yes …
    ctor2s-count corpora/wikidata-20261026.bz2`. They are archived raw (gzipped), with Ir,
    Dr, Dw and the cache events, per instruction. The `README.txt` holds the exact invocation.
  - **The two runs are identical** in every function, every call site and the process total
    (9,128,951,041 instructions). They are also identical, in every figure, to the archived
    run 1.
  - **The archived run 2 is not reproduced.** It gave v2 at 1,294,422,962 and ctor2s at
    1,238,217,311 inclusive. Its profile isn't archived, so the cause of the difference (such
    as a different process environment) is unresolved.

### 8.3 §5.3e: every callee and caller term

From `run1-functions.txt` and `run1-library-sites.txt` (run 2 identical). Each figure is
ctor2s minus v2, both in one process (`ctor2s-count`: v2 first, then ctor2s, per block).

| term | calls | v2 | ctor2s | Δ | status |
|---|--:|--:|--:|--:|---|
| `build::<false>`, own body | | 1,026,601,670 | 970,405,853 | **−56,195,817** | **exact**: the claim |
| `refill`, own body | 1,299,478 | 80,567,464 | 80,567,464 | 0 | **bounded by construction**: identical code, no library call; its count is a function of the block's data only |
| `finish`, own body (direct, and from `refill`) | 532,547 + 1,262,935 | 176,758,551 | 176,758,551 | 0 | **bounded by construction**, as `refill` |
| `canonicalise` | 434 | 3,903,384 | 3,903,384 | 0 | **bounded by construction**: no callee; data-dependent only, on the same inputs |
| `finish`'s `memset` (heap destination) | 41,120 | 519,933 | 519,933 | 0 observed | **unresolved**: depends on the heap layout |
| `memset`, heap destination (`+0x281` in v2) | 230 | 865,030 | 865,030 | 0 observed | **unresolved**: depends on the heap layout |
| `memcpy`, descriptor initialisation, stack to stack, 10,240 bytes | 434 | 4,433,744 | 4,433,744 | 0 observed | **unresolved**: the per-call cost depends on the frame's alignment, and ctor2s's frame offsets differ from v2's (`0xe8`/`0x2cf8` against `0xd8`/`0x2ce8`). The same copy costs 10,216 per call here and 10,248 in another process (prototype 2's evidence) |
| `memset`, once per epoch, stack destination | 3,654 | 285,012 | 285,012 | 0 observed | **unresolved**, as the previous row |
| `memcpy`, the snapshot push (heap destination) | 3,654 | 454,910 | 456,750 | **+1,840** | **unresolved**: same calls and sizes, a different destination address |
| `memset`, heap destination | 233 | 2,817 | 2,817 | 0 observed | **unresolved** |
| `grow_exact`, first site | 2 | 12,656 | 1,780 | −10,876 | **unresolved**: the allocator's state |
| `grow_exact`, second site | 8 | 6,150 | 7,197 | +1,047 | **unresolved** |
| `free` (2 sites) | 8 | 1,544 | 1,688 | +144 | **unresolved** |
| `reserve` (2 sites) | 3 | 1,331 | 1,326 | −5 | **unresolved** |
| library total, observed | | | | −7,850 | unresolved as a whole; equal to inclusive − own |
| **callers** of `build::<false>` | | | | n/a | **outside the ledger.** The only callers are this research binary's harnesses, with the same call for both builders. Nothing in the decoder calls either builder |
| **call sites inside** `build::<false>` (to `refill`, `finish` and the library) | | | | | in the own body, so counted in the −56,195,817 |

- **Bounded:** the own body (exact), and the code callees `refill`, `finish` and
  `canonicalise` (Δ 0, by construction).
- **Unresolved:** every library term. They are listed with their observed values, and no
  interval is cited for them.
- **No two-sample interval is cited anywhere in this erratum.**

## 9. The U1/U2 trial: pre-registration (committed before any implementation, Miri run or ledger figure)

**Authority:** the coordinator's authorisation, relaying the user's approval, of **one bounded
implementation trial of U1 and U2, exactly as written in §7.**
- **U3 is NOT authorised:** no `assert_unchecked`. C4 uses the handoff's safe do-while form.
  If that can't reach the intended machine code, it is reported, never replaced by U3.
- **No other unchecked access is authorised.**
- **The trial is one implementation plus routine repairs.**
- **Review input** is added in §9.5: the adversarial review of PR #150
  (`docs/journal/compact30-adversarial/scanner-146/report.md` at `48b39628`, A2, and its
  `safety/SAFETY.md`). It doesn't change the authorised scope.
- **Stop and escalate (§8.2)** if U1 or U2 turns out to need any safety obligation beyond the
  invariants documented in §7.

### 9.1 What is built

- **A new module, `examples/c30walk/ctor2su.rs`.** It is `ctor2s.rs` at `7f74e9f`, copied,
  with one change: a nonuniform full tile goes through a new `tile16(ll, i, t)`, where `t`
  is the 16-byte chunk that `split_first_chunk` split off `rest`.
  - **U1, the first segment:** `ll.get_unchecked(i..i + n1)`, with n1 = min(16, 30 − p).
  - **U2, the second segment:** `ll.get_unchecked(i + n1..i + 16)`.
  - **C4, safe:** the first segment becomes a do-while. Its first row's byte is the window's
    `t[0]`, a safe array access at a constant index. It then loops while k < n1, reading
    `s1[k]`, a checked index that the compiler is expected to prove from k < n1 =
    `s1.len()`. The handoff's safe form relies on p < 30 for correctness only: if p were 30,
    it would append one wrong row, and read in bounds.
- **The tail keeps v2's checked `tile`.** It never takes U1 or U2.
- **The contract checks,** asserted in the counted build and debug-asserted in the uncounted
  build, so active under Miri:
  - `ptr::eq(t.as_ptr(), ll.as_ptr().wrapping_add(i))`: the tile is the chunk split off
    `rest`, so i + 16 ≤ end ≤ `ll.len()`;
  - `self.p < ROWS`;
  - in `tile`, `len < 16`: the checked path is the tail's only.
- **A counted-build counter** `unchecked_tiles`, and a count mode `ctor2su-count --only v2|su`
  that runs one builder per process.
- **Frozen:** `ctor2s.rs`, `ctor2.rs`, `ctor3.rs`, `ctor.rs`, `cbuild.rs`, the walker, the
  oracle, `src/` and `bench/`. `examples/c30walk.rs` gains a `mod` line and a `ctor2su-`
  dispatch.

### 9.2 The safety argument (§7's invariants, and nothing more)

- **i + 16 ≤ end ≤ `ll.len()`.**
  - `rest` is `ll[i0..end]` shrunk from the front, and i = end − `rest.len()`.
  - The tile was split off `rest` by `split_first_chunk::<16>()`, so `rest.len()` ≥ 16.
- **n1 ≤ 16 for any p:** it is a `min` with 16.
  - If p were 30, n1 = 0 and U1 is the valid empty range [i, i).
  - If p were above 30, `30 − p` wraps in release, and n1 = 16.
  - So memory safety doesn't rest on the phase invariant.
- **Hence both ranges are well-ordered and inside `ll`:**
  - U1 = [i, i + n1) and U2 = [i + n1, i + 16), with i ≤ i + n1 ≤ i + 16 ≤ `ll.len()`;
  - no sum overflows, since i + 16 ≤ `ll.len()` ≤ `isize::MAX`;
  - every byte is initialised and immutable, inside the live `&[u8]`.

### 9.3 The compile target (gate 2, diagnostic)

| path | B2 | ctor2s (`7f74e9f`) | target |
|---|--:|--:|--:|
| a nonuniform full tile's overhead, outside its 16 row appends | 41 + 7X | 41 + 7X + 8 | **≤ 34 + 7X** (the handoff's) |
| the tile's 8 check instructions (§7) and C4's `cmp $0x1e` / `jne` | present | present | **absent** |
| fresh pair; lookahead per matched tile | 12; 16 | 4; 11 | **4; 11**, unchanged |
| row loops, the four scalar copies | 12 each | 12, 12, 12, 11 | **no more than ctor2s's**, per copy |

**Also targeted:**
- no `slice_index_fail` reachable from `tile16`;
- no unconditional jump in place of C4;
- no new spill or reload per row;
- no more spills around refills or in `run` than ctor2s has;
- the tail path unchanged.

A miss is recorded, and gate 2 never disqualifies.

### 9.4 The registered minimum and the ledger (gate 3, the qualifier)

**The minimum is unchanged: 60 M instructions net on B2,** at the lower end of the
conservative interval.

**The ledger:**
- **Primary:** the §5 method. It is static path lengths × observed counts from the counted
  build's counters, with every changed caller, callee, spill and exceptional path counted.
- **Cross-check:** callgrind's exact counts, with each builder alone in its own process,
  twice.
  - The raw Ir, Dr and Dw profiles are archived from the start, with the toolchain, the
    invocation and the binary's sha256.
  - They are parsed with the header-aware parser of §8.2.
  - Library terms must be equal call by call (checked), or charged at their unfavourable
    end, or listed as unresolved.
  - No two-sample interval is cited.

### 9.5 The Miri classes and mutants (gate 1)

**The classes:** all eight of §0.4, as implemented: `tiny`, `tails`, `align`, `phase`,
`held`, `endrun`, `epoch` and `malformed`.
- These include §7's list: the phase sweep, held tiles, odd alignment, the tail class and the
  epoch class.
- **New: `controls`.** One 32-byte input in an exact allocation (a boxed slice), with two
  nonuniform full tiles:
  - i = 0, with p = 0 and n1 = 16;
  - i = 16, with p = 16 and n1 = 14.

  The unmutated build must pass it.

**The mutants.** Each must be reported by Miri, and **its diagnostic must identify the
intended operation:** the source line of U1 or of U2, in the backtrace.

| mutant | change | what it tests | Miri must report it |
|---|---|---|---|
| MU1, the combined `n1 = 17` (§7) | n1 forced to 17 | **U2 and range order only.** At i = 0 in 32 bytes, U1's [0, 17) is still inside `ll` | at **U2** (the reversed range [17, 16)) |
| MU2, the U1 endpoint control | on a final full tile (i + 16 = `ll.len()`), U1's end becomes `ll.len()` + 1 | U1's bound | at **U1**: [16, 33) |
| MU3, the U2 endpoint control | at `ll.len()` = 32, i = 16, p = 16, n1 = 14: U1's [16, 30) kept, U2 becomes [30, 33) | U2's bound | at **U2** |

- **Each mutant runs twice on `controls`:**
  - under Miri's defaults, where the library's `get_unchecked` precondition check (UB
    checks) is on;
  - if the toolchain allows it, with `-Zub-checks=no`, so that Miri's own bounds checking
    reports it on the exact allocation.

  Both diagnostics are archived.
- **Tails never take U1 or U2.** This is checked by:
  - `tile16`'s contract assertion and `tile`'s `len < 16`;
  - in every case, the counted build's `unchecked_tiles` equal to its nonuniform full tiles;
  - in the `tails` class, every case's tail going through `tile`.

  The tail class must still pass.
- **Prototype 1's release mutants M1 and M2** (§0.5) are rerun on `ctor2su`. Each must fail the
  quick suite.
- Every class must complete. A timeout is partial coverage: the cases are split and run to
  completion.

### 9.6 The rejection conditions

**Gate 1 fails if:**
- §0.6's conditions hold, applied to `ctor2su`;
- Miri reports undefined behaviour on the unmutated build;
- a registered class can't be completed;
- a contract assertion fires on any registered or suite input;
- a mutant survives, or is reported at an operation other than its intended one.

**Gate 2** records a miss for any §9.3 target not met.

**Gate 3 fails if:**
- the conservative interval's lower end is below 60 M; or
- a relevant added cost stays unbounded.

**Escalation (§8.2), stopping the trial:**
- if U1 or U2 needs any safety obligation beyond §9.2;
- if any other unchecked operation would be needed;
- if a mutant isn't reported.

## 10. The U1/U2 trial: checkpoint (charter §9)

**Outcome: the trial qualifies on B2,** with gate 2 missed in part.

| gate | role | outcome |
|---|---|---|
| 1, correctness and safety | qualifier | **PASS**: every suite exact; all 9 registered Miri classes complete with no undefined behaviour; MU1–MU3 each reported by Miri at the intended operation; M1 and M2 caught; no contract assertion fired |
| 2, machine code | diagnostic | **missed in part** (§10.3): the 8 check instructions and C4's test are gone, the row loops are ctor2s's, and the refill and run spills fall; but the nonuniform tile's overhead misses ≤ 34 + 7X, and the safe do-while's trip count costs 3 where C4 cost 2 |
| 3, net work on B2 | qualifier | **PASS**: **−84,060,421** own and inclusive; the conservative interval is **[84.04 M, 84.06 M] removed**, against the 60 M minimum (§10.4) |

**The pin:** branch `research/compact30-scanner`, at this entry's commit. The pre-registration
is §9, at `fa490c4`.
- **The trial:** `examples/c30walk/ctor2su.rs` (new), plus a `mod` line and the `ctor2su-`
  dispatch in `examples/c30walk.rs`.
- **Frozen, each unchanged:** `ctor2s.rs` and the evidence at `7f74e9f`, `ctor2.rs`, `ctor3.rs`,
  `ctor.rs`, `cbuild.rs`, the walker, the oracle, `src/` and `bench/`.
- **The counted binary:** rustc 1.97.0 (`2d8144b78`, LLVM 22.1.6), `--locked --release`, empty
  rustflags. Its sha256 is
  `3ce86859be086ee1c47bce4ab7426ceffad3e7ffe6755c3a162f32d4ceea050f`, and a clean rebuild
  reproduces it.
- **Evidence:**
  [2026-11-04-compact30-scanner-u1u2-trial/](2026-11-04-compact30-scanner-u1u2-trial/), with
  its `README.txt`.

**Untimed.** Nothing was timed or benchmarked, and no perf counter was used. Callgrind is an
untimed simulation, used as a cross-check.

### 10.1 The implementation and its one repair

**The registered form (variant a) compiled badly.** Its first segment read the window's
`t[0]`, then `s1[k]` from U1. Those are two read bases, so LLVM built the first-segment row
loop with a separate pointer and three counters: **15 instructions per row,** against v2's 12
(`variants/variant-a-build-false.asm`, `+0xa00`–`+0xa2f`).

**Two routine variants of the same form changed nothing, or made it worse:**

| variant | first segment | first-segment row loop | C4 |
|---|---|--:|---|
| a, registered | do-while: `t[0]`, then `s1[k]` (U1) | 15 | gone |
| b | as a, with `s1.get(k)` | 15 (identical code) | gone |
| d | a plain `for` over U1 (no do-while) | 13 (the descriptor base rematerialised in the loop) | kept (`cmp $0x1e`; `je`) |
| l | the first row peeled from `t[0]`, then a `for` over U1 skipping 1 | 13, plus a peeled copy | an entry test |
| **c, final (repair 1)** | **do-while over the window, `t[k]`, k < n1 ≤ 16** | **12** | **gone; the trip count max(n1, 1) costs `cmp`, `mov`, `adc`** |

**Repair 1, the kept form, reads the first segment from the 16-byte window.** This is the
handoff's own safe C4 form (C4: "with checked indexing or the window").
- `t[k]` with k < n1 ≤ 16 needs no check.
- **U1 is then not used.** The final source has only U2, `ll.get_unchecked(i + n1..i + 16)`.
- It uses strictly less unchecked access than was authorised.
- No other unchecked operation was added. U3 isn't used.

**What C4's safe form can't reach.** The do-while has to run at least once, so without
knowing n1 ≥ 1 (that is, p < 30, which only U3 would give the compiler) LLVM computes the trip
count max(n1, 1). That costs 3 instructions where C4's test cost 2. As the brief required, this
is reported, and U3 was not substituted.

### 10.2 Gate 1

**The release suites,** on the final binary. All were exact, exit 0:
- **synthetic, quick:** 630 cases;
- **synthetic, full:** 54,030 cases (99,968 array comparisons, 149,952 walk outputs);
- **adversarial:** 200 records;
- **corpora:** the 11 corpora × 4 configurations, 44 records. The natural configuration is
  1,513 blocks and 707,949,378 rows.

The counted build asserts the contract checks:
- `tile16`'s chunk identity and p < 30;
- `tile`'s `len < 16`;
- the harness's `unchecked_tiles` = nonuniform full tiles, and one tail exactly when
  n mod 16 > 0.

No assertion fired. On the primary corpus, `unchecked_tiles` = `tiles_nonuniform` =
2,393,313, and the 230 tails (1,875 rows) all went through the checked `tile`.

**Miri** (nightly 1.101.0, `16d1ffac1`, Stacked Borrows, unpoisoned arenas, one class at a
time): **all 9 classes complete, with no undefined behaviour.**

| class | cases | seconds | reached |
|---|--:|--:|---|
| tiny | 18 | 25 | n = 0..33 edge sizes, 7 nonuniform full tiles, 12 tails |
| tails | 45 | 38 | every tail length 1..=15, alone, after a nonuniform tile, after a run; each tail through `tile`, each full tile through `tile16` |
| align | 14 | 15 | byte offsets 1..=7 |
| phase | 15 | 18 | every reachable p (even, 0..=28), 7 with a second segment |
| held | 5 | 6 | 12 held reuses |
| endrun | 2 | 3 | runs to the block's end |
| epoch | 2 | 1,073 | the run straddling 61,440, and a nonuniform tile ending at it; 3,839 nonuniform tiles |
| malformed | 4 | 1 | rejected by both builds, both arrays empty |
| controls | 1 | 2 | the exact-allocation 32-byte input: tiles at i = 0 (n1 = 16) and i = 16 (p = 16, n1 = 14) |

**The mutants** (`mutations/`). Each was run on `controls`, under Miri's defaults (the
library's UB checks on) and with `-Zub-checks=no` (Miri's own checks):

| mutant | applied to | Miri's defaults | `-Zub-checks=no` | at the intended operation? |
|---|---|---|---|---|
| MU1, n1 forced to 17 (U2 and range order only) | final | `unsafe precondition(s) violated: slice::get_unchecked requires that the range is within the slice`, at `ctor2su.rs:667:30`, U2's line | `Undefined Behavior: arithmetic overflow in unchecked_sub` at `667:27`, U2 | **yes, U2** |
| MU2, U1's end = `ll.len()` + 1 on a final full tile | the registered form (variant a), where U1 exists | the precondition, at `666:30`, U1's line | `constructing invalid value of type &[u8]: encountered a dangling reference (going beyond the bounds of its allocation)`, at `666:27`, U1 | **yes, U1** |
| MU3, U2 = [30, 33) at i = 16, p = 16, n1 = 14 | final | the precondition, at `666:30`, U2's line | dangling reference, at `666:27`, U2 | **yes, U2** |

The final source has no U1, so MU2 checks the U1 bound in the form that has one.

**Prototype 1's release mutants**, ported:
- M1 (the held tile skipped) is caught at "small n 32 sigma 3".
- M2 (the run short by one tile) is caught at "small n 16 sigma 1".

**The safety obligations are §9.2's, and nothing more.** U2's range is
[i + n1, i + 16), with i + 16 ≤ `ll.len()` from the chunk split and n1 ≤ 16 from the `min`.
No escalation was triggered.

### 10.3 Gate 2 (diagnostic)

Each path is counted by its exact execution-count class (`callgrind/classes-run1.txt`):

| path | B2 | ctor2s | trial | target | |
|---|--:|--:|--:|--:|---|
| a nonuniform full tile, instructions outside its row loops (S class + X class) | 38 + 8X | 46 + 8X | **37 + 4X** | 31 + 8X (≤ 34 + 7X in the handoff's accounting, i.e. B2 − 7) | **missed**: by 6 per tile without a second segment, and 2 with |
| the 8 check instructions (§7) and C4's `cmp $0x1e` / `jne` | present | present | **absent** | absent | **met**, but replaced by the trip count (3) |
| fresh pair (A class); lookahead per pair (J class) | 10; 9 | 2; 5 | **2; 5** | unchanged from ctor2s | **met** |
| row loops, the four scalar copies | 12, 12, 12, 12 | 12, 12, 12, 11 | **12, 12, 12, 11** | no more than ctor2s's | **met** |
| refill detour from a tile's first segment; from its second; `run` | 10; 9; 32 | 14; 12; 35 | **12; 11; 31** | no more than ctor2s | **met** |
| `slice_index_fail` reachable from `tile16` | | | **none** (the three left are the tail's two and `run`'s pattern index) | none | **met** |
| a new jump in place of C4 | | | **none** (v2's S path has the same one `jmp`) | none | **met** |

**Where the tile's remaining overhead is** (`listings/ctor2su-build-false.asm`,
`+0x9c0`–`+0xa09` and `+0x8f0`–`+0x937`). It is:
- recomputing i (`end − rest.len()`);
- stack stores and reloads around it (`rest`'s pointer, the tile base, `ll`'s pointer, and p and y
  from the scan state);
- the trip count (3).

The row loops were the hazard in prototype 1's first attempts. Here they are unchanged.

### 10.4 Gate 3: the ledger on B2

**The frequencies** come from the counted build (`corpora.jsonl`, primary corpus):

| A | J | H | Q | S | X | R | epochs | tails |
|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| 2,396,084 | 10,541,706 | 607,750 | 2,771 | 2,393,313 | 1,116,556 | 610,521 | 3,654 | 230 |

**The primary §5 ledger** is the path change × frequency, every changed class counted:

| class | frequency | B2 → trial | Δ |
|---|--:|---|--:|
| uniform tiles U = J + Q | 10,544,477 | 5 → 0 | −52,722,385 |
| lookahead pairs J | 10,541,706 | 9 → 5 | −42,166,824 |
| matched lookaheads J − H | 9,933,956 | 0 → 4 | +39,735,824 |
| fresh pairs A | 2,396,084 | 10 → 2 | −19,168,672 |
| outer iterations S + R | 3,003,834 | 13 → 11 | −6,007,668 |
| second segments X | 1,116,556 | 8 → 4 | −4,466,224 |
| nonuniform tiles S | 2,393,313 | 38 → 37 | −2,393,313 |
| runs R | 610,521 | 32 → 31 | −610,521 |
| held reuses H | 607,750 | 4 → 3 | −607,750 |
| epochs | 3,654 | 33 → 35 | +7,308 |
| blocks | 434 | 204 → 200 | −1,736 |
| tails | 230 | 41 → 39 | −460 |
| **counter-matched subtotal** | | | **−88,402,421** |
| refill from a first segment (no counter) | 996,634 | 10 → 12 | +1,993,268 |
| `fill`'s vector path (no counter) | 801,475 | 18 → 19 | +801,475 |
| an alignment `nop` in `run` (no counter) | 610,356 | 0 → 1 | +610,356 |
| refill from a second segment (no counter) | 302,049 | 9 → 11 | +604,098 |
| `run`'s whole-group branch (no counter) | 296,405 | 43 → 44 | +296,405 |
| 8 smaller positive classes (no counter) | | | +53,601 |
| 6 smaller negative classes (no counter) | | | −17,203 |
| **own code, `build::<false>`** | | | **−84,060,421** |
| `refill`, `finish`, `canonicalise` | | identical code and counts (80,567,464; 176,758,551; 3,903,384) | 0 |
| library calls, all 11 sites | | **equal call by call,** each builder alone, both runs | 0 |

- **Callgrind agrees exactly.**
  - `build::<false>`'s own count is 1,026,601,670 → 942,541,249, and its inclusive count is
    1,294,420,653 → 1,210,360,232. Both deltas are −84,060,421.
  - The two runs are identical in every function, every call site and the process total
    (12,330,316,333 for v2, 12,246,256,780 for the trial).
  - The library sites are equal between the two builders, so their delta is 0, checked, by
    construction (§9.4).
- **The conservative interval is [84,043,218, 84,060,421] removed.** The lower end charges
  every counter-less positive class (+4,359,203) and credits none of the counter-less
  negatives (−17,203).
- **The lower end exceeds the 60 M minimum by 24.04 M. Gate 3 passes.**
- **No relevant cost is unbounded.**
- **Against prototype 1:** the own body falls by a further 27,864,604 (−56,195,817 →
  −84,060,421). On the nonuniform tile alone that is 46 → 37 per S, and 8 → 4 per X.

**Reads and writes** (own and inclusive alike, since the callees are equal; reported, not
gated):
- reads 205,899,121 → 205,393,236, **−505,885**;
- writes 132,435,558 → 135,826,990, **+3,391,432**.

The extra writes are spills on the tile and refill paths. Against ctor2s, reads are +2.88 M
and writes +1.40 M.

### 10.5 Limitations and negative findings

- **U1 isn't used in the final form** (§10.1). The trial tests U2 and the window do-while.
  U1's bound is checked by MU2, in the registered form only.
- **The safe C4 can't drop the test outright.** Its trip count costs one instruction more than
  C4's test, as §10.1 explains. Only U3, which is not authorised, would let the compiler drop it.
- **The tile overhead target is missed** by 2–6 per tile, through spills and recomputing i.
- **Writes rise** +3.39 M against B2, untimed.
- **Only the primary corpus is ledgered.** The other corpora were run for correctness.
- **No combined build with prototype 2** was made. Prototype 2's descriptor LEAs are still
  present in all four row copies here: each `movzbl` → `lea (%r,%r,4)` → `lea (%r,%r,8)`
  remains. So the credited sets don't intersect at instruction level. Both charge the outer
  iteration and the refill detours, though, so the gains aren't additive without a combined,
  pre-registered build.

## 11. Addendum (2026-11-04): S62-F01's counters and the counted ledger

This addendum answers review finding S62-F01 and the Optimiser audit's F3. Charter §5.3(b)
requires an observed execution count for every changed path, from a counter added to the
counted instantiation only, with the uncounted listing shown unchanged. §10.4 used
Callgrind-only frequencies for several changed paths. This section adds those counters and
replays §10.4's ledger with counted frequencies. It records numbers only; it makes no
qualification decision. S62-F02 (the overlap) is not addressed here.

Evidence: [2026-11-04-compact30-scanner-f01/](2026-11-04-compact30-scanner-f01/), with its
`README.txt`. Untimed: no benchmark, no perf, and no timing run.

### 11.1 The counters (counted build only)

Every changed frequency class in §10.4 was read from the listing, with each instruction's own
count from the trial's raw profile. Each was then assigned to its source path. There are 19
new `Stats2su` fields. Each is incremented only inside `if COUNT { … }` (that is, only in
`build::<true>`), or in the harness's `same_as_v2`. `build::<false>`, `W<false>` and every
other line of the constructor are unchanged.

| path (listing, `ctor2su-build-false.asm`) | counter | counted | profile | B2 → trial | Δ |
|---|---|--:|--:|---|--:|
| refill detour from `tile16`'s first segment (`+0xa3d`–`+0xa76`) | `tile_refills_first` | 996,634 | 996,634 | 10 → 12 | +1,993,268 |
| `fill`'s vector-loop entry, m ≥ 8, and its alias check (`+0xc18`–`+0xc25`, `+0x1078`–`+0x109b`, `+0x10c7`–`+0x10d2`) | `fills_vector` | 801,475 | 801,475 | 18 → 19 | +801,475 |
| runs entering the lookahead loop: the executed alignment `nop` (`+0x973`) | `runs_lookahead` | 610,356 | 610,356 | 0 → 1 | +610,356 |
| refill detour from `tile16`'s second segment (`+0xad1`–`+0xb01`) | `tile_refills_second` | 302,049 | 302,049 | 9 → 11 | +604,098 |
| `run`'s whole-group branch, `complete` (`+0xd04`–`+0xd2a`, `+0x1030`–`+0x106f`, `+0x119e`–`+0x11ee`) | `complete_calls` | 296,405 | 296,405 | 43 → 44 | +296,405 |
| `finish` from the final close loop (`+0x1577`–`+0x1594`) | `final_finishes` | 23,559 | 23,559 | 8 → 9 | +23,559 |
| crossing groups preset RAW, `create_raw` (`+0x4b0`–`+0x543`, `+0x5a3`–`+0x5b2`) | `preset_crossing` | 18,684 | 18,684 | 36 → 37 | +18,684 |
| epochs with a full tile: scan-loop entry (`+0x8db`–`+0x8ea`) | `epochs_tiled` | 3,575 | 3,575 | 5 → 4 | −3,575 |
| epochs with no tail: B2's epoch-loop back edge (B2 `+0x7d0`–`+0x7d6`) | `epochs_untailed` | 3,424 | 3,424 | 3 → 0 | −10,272 |
| runs whose lookahead exhausts the epoch (`+0x9aa`–`+0x9b7`) | `runs_exhausted` | 2,606 | 2,606 | 0 → 4 | +10,424 |
| refill detour from the tail's first segment (`+0x13d1`–`+0x13f7`) | `tail_refills_first` | 786 | 786 | 12 → 8 | −3,144 |
| tail advance loop (unrolled by two), entry for len ≥ 2 (`+0x1333`–`+0x133e`, `+0x137b`–`+0x1384`) | `tails_len_ge2` | 218 | 218 | 6 → 7 | +218 |
| runs with no lookahead tile (`+0xb10`–`+0xb18`) | `runs_no_lookahead` | 165 | 165 | 0 → 3 | +495 |
| tail advance loop exit, even len (`+0x138a`) | `tails_even` | 121 | 121 | 0 → 1 | +121 |
| tail second-segment row loop (`+0x1420`–`+0x1444`) | `tail_rows_second` | 106 | 106 | 12 → 11 | −106 |
| epochs with no full tile (`+0x12a0`–`+0x12a3`) | `epochs_short` | 79 | 79 | 3 → 2 | −79 |
| tail second-segment entry and exit (`+0x13f9`–`+0x1411`, `+0x1446`) | `tails_second_segment` | 22 | 22 | 4 → 8 | +88 |
| a tail of length 1 (`+0x7c7`) | `tails_len1` | 12 | 12 | 0 → 1 | +12 |
| refill detour from the tail's second segment (`+0x144b`–`+0x1471`) | `tail_refills_second` | 9 | 9 | 12 → 9 | −27 |

**Every counted value equals the profile's frequency exactly.** The new counters are the 19
rows above. The other 12 changed classes (A, J, J + Q, J − H, S, X, S + R, R, H, epochs,
blocks, tails) were already counted (§10.4).

Notes on the mapping:
- **One class, one path.** Each class's instructions were read in the listing and lie on
  one path.
- **The alias check.** `fill`'s vector entry tests m ≥ 8 (`lea -0x2(,%rsi,2)`, `cmp $0xe`).
  Its alias check (`+0xc21`, `jae`) compares the field array with the static `PATTERNS`,
  which are distinct allocations, so the check always passes. The counter counts m ≥ 8.
- **B2's own classes** (J + Q and the epoch back edge, which have no trial instruction) are
  the same logical events, counted by the trial's counters. B2 is frozen, and its counted
  build agrees with the trial's on every shared field (`same_as_v2`).
- **The harness checks partitions.** `same_as_v2` now asserts:
  - `runs_lookahead + runs_no_lookahead = runs`;
  - `runs_exhausted + runs_no_lookahead = runs_no_full_tile`;
  - `epochs_tiled + epochs_short = epochs`;
  - `epochs_untailed + tails = epochs`;
  - `tails_len1 + tails_len_ge2 = tails`;
  - bounds on the rest.

  No assertion fired on any suite.

### 11.2 The uncounted listing is unchanged

- **The pinned binary reproduces.** A clean build of `49cb46b6` with the trial's recipe has
  sha256 `3ce86859…a050f`, the trial's archived binary. The counted-counter binary is
  `6b1f1d95…784d44`.
- **The tool.** `asmid.py` follows `build::<false>`'s transitive callee closure in both
  binaries. Each function is compared over its symbol extent after
  `bench/c30_build_listing.py`'s normalisation:
  - addresses dropped;
  - internal branches as offsets;
  - external targets as symbols;
  - RIP displacements masked;
  - comment symbols kept.

  The sizes must be equal.
- **The result is identical.** All 52 functions (4,057 instructions) match:
  - `build::<false>`, 1,382 instructions;
  - `refill`, 67;
  - `finish`, 379;
  - `canonicalise`, 173;
  - the allocator and panic paths.

  The new binary's `build::<false>`, `refill` and `finish` also equal the trial's archived
  listings.
- **The control shows the tool detects a change.** The same tool on `build::<true>` reports
  DIFFERENT.

### 11.3 The suites

The counted-counter binary passes every suite, each exit 0, with every check exact:
- corpora: 11 × 4, 44 records;
- adversarial: 200 records;
- synthetic: 54,030 cases;
- synthetic, quick: 630 cases.

Every pre-existing field of every jsonl record equals the trial's archive. So does every
console, once the 19 new fields are removed.

### 11.4 The ledger, replayed with counted frequencies

`f01_replay.py` reads the trial's raw profiles (the S62 review's parser, copied unchanged)
and the counted record. It checks each changed class's counter against the profile's
frequency. It charges any class without an agreeing counter at its least favourable end.

| | §10.4 / S62 replay | counted (this section) |
|---|--:|--:|
| counter-supported Δ | −88,402,421 | **−84,060,421** |
| profile-only added | +4,359,203 | **0** |
| profile-only removed (withheld) | −17,203 | **0** |
| library sites | 11 equal | 11 equal |
| **interval removed** | [84,043,218, 84,060,421] | **[84,060,421, 84,060,421]** |
| minimum | 60,000,000 | 60,000,000 |

- **The interval collapses to a point.** Every changed class is now counted, and the
  library sites are equal, so there is no width left.
- **The memory deltas are unchanged** (own code, all in `build::<false>`; `refill`, `finish`
  and `canonicalise` are equal). They are reported, not gated:
  - reads: **−505,885**;
  - writes: **+3,391,432**.

### 11.5 The caller boundary (S62-F04)

- **The harness's own count rises.** `c30walk::main`'s own count goes from 4,499,105,219
  (v2 alone) to 4,499,106,087 (trial alone): +868 = 2 × 434 blocks.
- **The two call sites match.** `harness-callsite.asm` lists both call sites with each
  instruction's count in both profiles. Each arm sets up and makes the call in 10
  instructions: 7 for arguments, then the `call`, `test` and branch. They are identical but
  for the arena's stack offset.
- **The difference is the dispatch.** The `--only` match tests `"v2"` first (`cmpw $0x3276`,
  `je`). The trial's arm then runs `cmpw $0x7573` and `jne`: two instructions per block that
  v2's arm skips.
- **So it is not constructor work.** The +868 is the harness's string dispatch, in code
  outside `build::<false>`'s closure. The call's setup costs the same in both arms.
- **If it were charged anyway,** the interval would be [84,059,553, 84,059,553].

### 11.6 Commands

These run from the worktree root. `README.txt` gives them in full.

```sh
CARGO_ENCODED_RUSTFLAGS= CARGO_TARGET_DIR=target/rust-1.97.0 cargo +1.97.0 build --locked --release --example c30walk
F=docs/journal/2026-11-04-compact30-scanner-f01; T=docs/journal/2026-11-04-compact30-scanner-u1u2-trial
python3 $F/asmid.py PINNED_BINARY target/rust-1.97.0/release/examples/c30walk OUT '<c30walk::ctor2su::Ctor2su>::build::<false>' \
  --archived "<c30walk::ctor2su::Ctor2su>::build::<false>=$T/listings/ctor2su-build-false.asm"
target/rust-1.97.0/release/examples/c30walk ctor2su-check $F/counted/corpora.jsonl corpora/*.bz2
python3 $F/f01_replay.py scanner $T $F/counted/corpora.jsonl ledger.json
python3 $F/callsite.py PINNED_BINARY $T/callgrind/only-v2-run1.callgrind.gz $T/callgrind/only-su-run1.callgrind.gz \
  '<c30walk::ctor2su::Ctor2su>::build::<false>'
```
