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
