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

### 0.7 Addendum (2026-11-04, after the reviewer's brief, PR #148 at `ff65301b`, §1; before Miri and before any gate-3 figure is used)

Information only, with no change of authority. The registered targets, minimum, Miri classes,
mutation checks and rejection conditions in §0.1–§0.6 are unchanged. Three points are made
explicit:

1. **The representation is a bijection.**
   - v2's `Desc` is stored as `wr[c]`, `en[c]` and `cold[c] = {w0, r0, lim, g, raw}`. Each
     field is held exactly once, so no pointer is duplicated as authoritative.
   - v2's `Desc` value is assembled from them only on reading (`W::desc`), never stored.
   - c < 256 comes from the byte input, or from `0..256` loops.
2. **The callee and allocator deltas are bounded without sampling.**
   - The cross-check is split into two parts:
     - (a) `build::<false>`'s own code plus its constructor callees (`refill`, `finish`, and
       v1's `canonicalise`, identical code). These are deterministic for a given input, and
       counted exactly.
     - (b) its library callees (allocation, `memset`, `memcpy`).
   - (b) is bounded by construction, under this argument:
     - both builders make the same library calls, with the same arguments, in the same
       order. The allocation and growth code (`reserve_exact`, `resize`, `reserve`, `push`)
       is v2's text, and is driven by the same block sizes and the same RAW pushes;
     - each builder is counted alone, in its own process, after the same decode of the same
       corpus, so the heap's history before each call is the same.
   - Then (b)'s instruction counts are equal by construction, and their delta is 0.
   - That equality is **checked**, call by call, as each builder's library-callee totals. If
     they aren't equal, (b) is listed as **unresolved**, and no two-sample interval is
     cited.
3. **The reviewer's adversaries,** and where each is covered (release suites; Miri as
   registered):

   | adversary | covered by |
   |---|---|
   | every character, 255 included | the `allbytes` class; σ = 256 random blocks in v2's synthetic suite |
   | zero buckets | every class with σ < 256 |
   | crossing groups | the `crossing` class; v2's bucket-boundary class |
   | preset records | crossing; forced at permille 50 and 1000 |
   | wide records | v2's span and far-wide classes |
   | partial records | the `pending` class; v2's pending class |
   | epochs | the `epoch` class; v2's epoch class, up to k = 15 |
   | maximum size | the 983,010-row blocks in v2's synthetic suite |
   | poisoned reuse | v2's repeated class, poisoned round, and the poisoned arenas of every release case |
   | malformed counts | v2's class (4,000 perturbations), and the `malformed` class |
   | an invariant-targeted mutation | M1 (`en` inconsistent with its portion) |

   Maximum size and wide records are covered in release builds only. Miri runs its nine
   registered classes.

## 1. Checkpoint (charter §9)

**Outcome: prototype 2 qualifies on B2,** with gate 2 missed in part.

| gate | role | outcome |
|---|---|---|
| 1, correctness and safety | qualifier | **PASS**: every suite exact, v2's counters equal field by field, malformed counts rejected with both arrays empty, both mutants caught, and all 9 registered Miri classes complete with no undefined behaviour (§3) |
| 2, machine code | diagnostic | **missed in part** (§4): every scalar copy is 10 instructions, with 4 reads and 2 writes per row and no spill or reload, and the fill and emitter loops don't grow; but the added non-row work is **+18.38 M net**, against the 5 M cap |
| 3, net work on B2 | qualifier | **PASS**: the conservative interval is **[54.55 M, 62.28 M] removed**, against the 40 M minimum, with the own-code figure at 58.21 M and no relevant cost left unbounded (§5) |

**The pin:** branch `research/compact30-descriptors`, from `9f1086f8`, at this entry's commit.
- **The prototype:** `examples/c30walk/ctor2d.rs` (new), plus the `mod` line and the
  `ctor2d-` dispatch in `examples/c30walk.rs` (7 added lines, nothing else).
- **Frozen files, each unchanged** (`git diff` empty):
  - `ctor2.rs` against `8beffe2`;
  - `ctor3.rs` against `dbf5139`;
  - `ctor.rs`, `cbuild.rs`, the walker and oracle in `c30walk.rs`, `src/` and `bench/`
    against `9f1086f8`.
  - `ctor2s.rs` isn't on this branch: the prototype is built on v2, not on prototype 1.
- **The counted binary:** `target/rust-1.97.0/release/examples/c30walk`, sha256
  `695ca5dd2ba9c2615c6d812694533d7520e88152fba56ba191f17c22f770d136`. It is rustc 1.97.0
  (2d8144b78, LLVM 22.1.6), `--locked --release`, with `CARGO_ENCODED_RUSTFLAGS` empty.
  Every listing in `listings/` and every profile in `callgrind/` is from this binary.

**Untimed.** Nothing was timed or benchmarked, and no perf counter was used. Callgrind is an
untimed instruction simulation, used as a **cross-check** beside the charter's §5 method
(§5.1).

## 2. The attempts (all safe Rust)

| attempt | the append's last two statements | scalar copy | kept? |
|---|---|--:|---|
| 1 | `let w = self.wr[ci]; w.write(u); self.wr[ci] = w.add(1);` | **11** in each copy: the loaded pointer is reused, so the increment is a register `add` and a store | no |
| 2 | `self.wr[ci].write(u); self.wr[ci] = self.wr[ci].add(1);` | **10** in each copy: the pointer is re-read after the store, and the increment is `addq $0x2` to memory, as in v2 | **yes** |

- Each attempt's listings and its source diff are in `attempt-1/` (against `ctor2.rs` at
  `8beffe2`) and `attempt-2/` (against attempt 1). The final
  source is attempt 2 plus two things that change no uncounted builder: three counters in
  the counted build only (`refills`, `refill_finishes`, `outer_iterations`, charter §5.3b),
  and the count harness's `--only` mode (§5.1). The uncounted
  `build::<false>`, `refill` and `finish` listings are identical to attempt 2's after address
  normalisation (0 differences).
- **The `addq` rule** (reviewer, §0.7): attempt 1 is the register form. Per row it costs one
  instruction more than attempt 2 (11 against 10) and reads one value less (3 reads, 2
  writes, against 4 and 2). It doesn't meet the ≤ 10 target, and it isn't the kept variant.
- **A third attempt was not made.** The remaining excess is outside the row loop (§4.2). Its
  largest single item that a source change could plausibly remove is the redundant `r0`
  store in `refill`'s open. Removing it means restructuring `open`, which isn't in the design
  (§D.3 is a layout change only), and the same restructuring would apply to v2. That would
  be a separate, pre-registered change, not an attempt at this one.

## 3. Gate 1: correctness and safety

### 3.1 The release suites

Each case runs the oracle, v2's counted build, and the prototype's counted and uncounted
builds. It compares each of the prototype's `rec` and `raw` with the oracle's, byte for
byte. It compares every `Stats2` counter with v2's, field by field (`same_as_v2`,
`cov_portion` included), and runs the walker on each output and checks its marker
restoration.

| suite | scope | result |
|---|---|---|
| corpora (`corpora.console`, `corpora.jsonl`) | the 11 corpora in `corpora.sha256`, each in 4 configurations: permille 0 with and without extra starts, and permille 50 and 1000 with them | 44 records, every check exact; the natural configuration alone is 1,513 blocks and 707,949,378 rows |
| adversarial (`adversarial.console`) | the adversarial inputs at bzip2 levels 1 and 9 | 132 blocks: every check exact |
| synthetic, full (`synthetic.console`) | v2's synthetic classes, plus the 9 descriptor classes of §0.4 | 54,145 cases: 100,194 array comparisons, 150,291 walk outputs, 100,194 restorations, every check exact |
| synthetic, quick (`synthetic-quick.console`) | the quick set, with the 9 descriptor classes | 745 cases, every check exact |
| malformed counts | v2's class, 4,002 cases, and the `malformed` class, 4 cases | 3,995 perturbations rejected by both builds, with `rec` and `raw` empty; the other 7 leave the counts equal to the input's own, and are built exactly; all 4 `malformed` cases rejected by both builds |

The poisoned arenas, the repeated class and its poisoned round are v2's, run unchanged.

### 3.2 The mutation checks (§0.5)

| mutant | change (`mutation-M*.diff`) | caught by | result |
|---|---|---|---|
| M1: `en` inconsistent with its portion | `open` sets `en[c]` one field short | `finish`'s counted assertion `k >= 1 && d.write == d.end` (ctor2d.rs:453), at the first synthetic case | **caught** |
| M2: the snapshot's ranks read from the wrong array | the rank is computed from `en[c]` instead of `wr[c]` | `epoch k 1 n 61501 off -1 pm 0: Ctor2d's counted build differs: raw length 2 vs 3` | **caught** |

M1 is the invariant-targeted mutation the reviewer asked for. Each fails the quick suite, so
neither is a §8.2 escalation.

### 3.3 Miri

**All 9 registered classes complete, with no undefined behaviour.** The toolchain is Miri
0.1.0 on nightly rustc 1.101.0 (`16d1ffac1`, 2026-10-26), Stacked Borrows, no flags. Each
class ran on its own, one after another, as
`cargo +nightly miri run --example c30walk -- ctor2d-synthetic --miri CLASS`, with a 7,000 s
timeout and unpoisoned arenas. Every case runs the same checks as the release suites (§3.1):
the prototype's two builds against the oracle, its counters against v2's counted build, and
the walker.

| class (§0.4) | cases | array comparisons | what it reached | seconds |
|---|--:|--:|---|--:|
| 1. tiny and empty | 18 | 32 | n = 0, 1, 2, 15, 16, 17, 31, 32, 33 at σ = 1 and 2; 182 scalar rows, 52 fragment rows, 5 runs | 25 |
| 2. every refill slot | 60 | 120 | σ = 2 and 3, with shifts 0..=29; 6,270 scalar rows | 73 |
| 3. all 256 byte values | 1 | 2 | every index 0..=255, 255 included; 1,024 scalar rows, 272 crossing finishes | 6 |
| 4. crossing groups | 120 | 240 | a bucket boundary at every offset 1..=60 for n = 61, both orders; 154 runs, 232 crossing finishes | 124 |
| 5. runs | 6 | 12 | runs of 16 to 160 rows at 6 phases; 6 runs, 156 fragment rows | 8 |
| 6. an epoch boundary | 1 | 2 | 61,504 rows, 2 epochs, so the snapshot reads the split arrays | 580 |
| 7. pending portions | 4 | 8 | 1, 29, 30 and 31 pending portions at the end | 7 |
| 8. malformed counts | 4 | 0 | 4 perturbations, each rejected by both builds with both arrays empty | 1 |
| 9. reuse | 7 | 10 | n = 200, 1, 0, 95, 31, 0, 150 on one constructor, at permille 0, 1000 and 50; 2 forced finishes | 6 |

Every class's console ends `every check exact` with exit 0 (`miri/`). None timed out.

### 3.4 The safety argument

- **No new `unsafe` operation, pointer arithmetic or unchecked access.** The pointer writes
  and increments are v2's, at v2's sites (`open`, `close`, `one`, `fill`, `run`), on the same
  pointers, now held in `wr`, `en` and `cold`.
- **The representation is a bijection** (§0.7.1): each field of v2's `Desc` is stored exactly
  once; `W::desc(c)` assembles v2's value only on reading, and is never stored.
- **v2's descriptor invariant holds over the split storage** (§0.5): for every c, either
  `wr[c] = en[c] = cold[c].w0 = null`, or `cold[c].w0 ≤ wr[c] ≤ en[c]` bound one open
  portion inside one group's fields. `open` sets all three; `close` and `refill` null all
  three; `one`, `fill` and `run` advance `wr[c]` by at most `(en[c] − wr[c]) / 2`, checked
  as in v2.
- **Indices:** every index is `c: u8 as usize` or `0..256`, so each array access is in
  bounds by the language. The compiled row loops index with `(%rsp,%rbx,8)` after a
  `movzbl`, with no bounds check.

## 4. Gate 2: the compiled counts against target (diagnostic)

### 4.1 The row loops

From `listings/row-loops.txt` and `listings/v2-row-loops.txt` (same binary):

| scalar copy (v2 offset → prototype offset) | rows (callgrind) | B2 | prototype | target |
|---|--:|--:|--:|---|
| `+0x900` → `+0x9c0` | 1,769 | 12 | **10** | ≤ 10, met |
| `+0x1250` → `+0x1370` | 29,357,924 | 12 | **10** | met |
| `+0x12e0` → `+0x1410` | 8,935,084 | 12 | **10** | met |
| `+0x13d0` → `+0x14d0` | 106 | 12 | **10** | met |

- The four copies execute 38,294,883 rows, which is exactly the counter `rows_scalar`.
- **The two LEAs are gone** in each copy. v2 forms 40c with `lea (%rsi,%rsi,4)` and
  `lea (%r8,%rcx,8)`; the prototype addresses `wr[c]` as `0xd8(%rsp,%rbx,8)` and `en[c]` as
  `0x8d8(%rsp,%rbx,8)`.
- **Memory operations per row: 4 reads and 2 writes, as in v2** (target: no more): the
  source byte, `wr[c]`, `en[c]` in the compare, and the read of `addq`'s read-modify-write;
  the field store and `addq`'s write.
- **Spills or reloads per row: 0** (target 0). No base register is reloaded inside any of
  the four loops: the array base is `%rsp`.
- **The fragment fill loops, the complete-record emitter and its odd tail:** no class
  executing at their iteration counts changed (`callgrind/classes-run1.txt`), so their loop
  bodies are unchanged.

### 4.2 The non-row paths, each recounted at its observed frequency

Each row is a class of instructions with one exact execution count (§5.1), in
`build::<false>` or `refill`.

| path | frequency | B2 → prototype | Δ per event | Δ instructions |
|---|--:|---|--:|--:|
| outer scan iteration (S + R) | 3,003,834 (`outer_iterations`) | 13 → 17 | +4 | **+12,015,336** |
| `refill`, common path | 1,299,478 (`refills`) | 59 → 65 | +6 | **+7,796,868** |
| `refill`, the call to `finish` | 1,262,935 (`refill_finishes`) | 2 → 3 | +1 | +1,262,935 |
| `finish` (re-addressed, same path lengths) | 1,795,482 calls | 176,758,551 → 176,758,551 own | 0 | 0 |
| `run` | 610,521 (`runs`) | 32 → 35 | +3 | +1,831,563 |
| `fill`'s vector path, per call | 801,475 | 18 → 19 | +1 | +801,475 |
| `run`'s whole-group branch | 296,405 | 43 → 44 | +1 | +296,405 |
| the final rank check and sweep (2 characters per iteration) | 55,552 (128 × 434) | 13 → 14 | +1 | +55,552 |
| an `open` path in `build` | 18,684 | 36 → 37 | +1 | +18,684 |
| per block: entry, initialisation and exit | 434 (`blocks`) | 204 → 211 | +7 | +3,038 |
| the snapshot, per epoch | 3,654 (`epochs`) | 33 + 128 × 14 = 1,825 → 53 + 63 × 28 = 1,817 | −8 | −29,232 |
| nonuniform tile (S) | 2,393,313 (`outer_iterations` − `runs`) | 38 → 37 | −1 | −2,393,313 |
| `fill`, per call | 1,085,833 | 25 → 24 | −1 | −1,085,833 |
| `open`'s RAW-branch join | 597,255 | 2 → 0 | −2 | −1,194,510 |
| an `open` path in `build` | 496,004 | 34 → 33 | −1 | −496,004 |
| the refill caller detours in the scalar copies | 302,049; 13,266; 3,575; 786 | 9 → 8; 2 → 0; 5 → 3; 12 → 13 | | −334,945 |
| descriptor initialisation loop (64 per block) | 27,776 | 35 → 31 | −4 | −111,104 |
| 9 other classes (counts 1 to 55,986) | | | | −56,194 |
| descriptor initialisation's library calls | 434 | §5.2 | | not in this table: −4,068,750 by callgrind, +378,882 charged |
| **non-row total** | | | | **+18,380,721** |

The frequencies come from the counted build's counters wherever one exists (in brackets).
The others are exact execution counts from callgrind, with no counter of their own.

**The audit's registered concerns, counted explicitly:**
- **The byte index around `refill`.** `refill` now keeps `c` for `finish` and for the
  `wr`/`en` stores: `push %r12`, an alignment `push %rax` and `add $8,%rsp`, `pop %r12`, and
  `mov %rsi,%rbx`. With the redundant `r0` store below, that is the +6 per refill. The call
  to `finish` gains `mov %rbx,%rsi`, +1 per finish. **Together: +9,059,803.**
- **The split reset stores at close.** There are now three scalar `movq $0` stores where v2
  has `xorps` with one 16-byte and one 8-byte store. The instruction count is the same (3 and
  3), but there is **one write more per refill**. `open`'s `Cold { .. }` literal also stores
  `r0` a second time (`mov %r15d,0x8(%r12)`): **one instruction and one write more per
  refill.**
- **The snapshot's rank reads from two arrays.** LLVM vectorises the new rank loop (63
  iterations of 4 characters, 28 instructions each), where v2's runs 128 scalar iterations of
  2 (14 each). Net −8 per epoch, −29,232 in all.
- **Reloads at the outer loop's head.** Once per outer iteration, outside the row loops,
  the head now forms the cold base (`lea 0x10d8(%rsp),%r8`), makes two stack reloads where
  v2 makes one (`0x60(%rsp)` and `0x40(%rsp)`), and makes two register copies: +4 per
  iteration, one read more, **+12,015,336**, the largest item.

**Against the 5 M cap: missed.** The added non-row work is +18.38 M net (+24.08 M gross of
the positive rows), not ≤ 5 M. The four largest items (the outer iteration, `refill` with its
`finish` call, `run`, and `fill`'s vector path) are 23.71 M. Report 227 §D.3's allowance
doesn't survive compilation: the audit's question (reassessment, line 2512) is answered in the
negative.

## 5. Gate 3: the net-work ledger on B2

### 5.1 The method

- **Primary: the charter's §5 method.** It is Σ over compiled paths of (path length change)
  × (observed execution count), from the listings in `listings/` and the counted build's
  counters (§4.2). The rows of §4.2, with the row term, are that sum.
- **Cross-check: callgrind** (valgrind 3.22, `--cache-sim=yes --dump-instr=yes`). It runs
  `ctor2d-count --only W`. **Each builder runs alone in its own process**, after the same
  decode of the same corpus (§0.7.2), so each library call sees the same heap history.
  v2 and the prototype each ran **twice**. Each pair of runs is identical in every function,
  every call site and the process total (12,330,322,965 and 12,268,046,012 instructions).
  The invocation, the binary's sha256 and the raw profiles (Ir, Dr, Dw and the cache events,
  per instruction) are in `callgrind/`, with a header-aware parser (`cgparse.py`) that
  reads the `positions:` and `events:` headers.
- The per-class execution counts that identify the paths in §4.2 come from the same profiles
  (`callgrind/classes-run*.txt`, identical in both runs).

### 5.2 The library callees, call by call (§0.7.2)

From `callgrind/only-*-run*-library-sites.txt`: every library call made by
`build::<false>`, per call site, each builder alone.

| call site | v2 | prototype | Δ |
|---|---|---|--:|
| `memset` (230 calls) | 865,030 | 865,030 | 0 |
| `memset`, once per epoch (3,654 calls) | 285,012 | 285,012 | 0 |
| `memcpy`, the snapshot push, once per epoch (3,654 calls) | 454,910 | 454,910 | 0 |
| `memset` (233 calls) | 2,817 | 2,817 | 0 |
| `grow_exact`, `reserve`, `free` (all 7 sites, 21 calls) | 21,194 | 21,194 | 0 |
| **descriptor initialisation** (434 calls each) | `memcpy` of the 10,240-byte `desc` array: 4,447,632 (10,248 per call) | `memset` of `en`, 2,048 bytes: 58,156 (134); `memcpy` of the 6,144-byte `cold` array: 262,570 (605); `memset` of `wr`, 2,048 bytes: 58,156 (134) | −4,068,750 |

- **Every shared call site is equal,** call by call, in both runs: Δ = 0, checked.
- **The premise of §0.7.2(b) does not hold in full.** It said both builders make the same
  library calls. They don't at descriptor initialisation: the prototype initialises three
  arrays where v2 copies one (the calls at `+0x3ca`, `+0x3e5` and `+0x452` in
  `listings/ctor2d-build-false.asm`; v2's at `+0x402`). These three
  sites are part of a changed path, not allocator noise. Each has a compile-time constant
  size and stack-frame source and destination, and each costs the same at every one of its
  434 calls in both runs.
- **The 10,240-byte copy's 10,248 instructions per call** (with 10,222 reads and 10,218
  writes per call) are consistent with a byte-wise `rep movsb`, which callgrind counts once
  per byte moved; the 6,144-byte copy takes a vector path (605 per call). How a string
  instruction counts in the ledger's unit is a counting convention, not settled here. So the
  −4,447,632 **is not credited** at the conservative end. The added `memset`s and the 6,144-byte
  `memcpy` (+378,882) are charged in full.

### 5.3 The result

| term | Δ instructions | source |
|---|--:|---|
| scalar rows: −2 × 38,294,883 | **−76,589,766** | §4.1, `rows_scalar` |
| non-row paths, counter frequencies (outer, refill, finish call, run, snapshot, S, block) | +20,487,195 | §4.2 |
| non-row paths, callgrind-only frequencies: the positive rows | +1,173,383 | §4.2 |
| non-row paths, callgrind-only frequencies: the negative rows | −3,279,857 | §4.2 |
| **own code (`build::<false>` + `refill`)** | **−58,209,045** | equal to callgrind's exact own-instruction delta, −67,268,848 + 9,059,803 |
| `finish`, `canonicalise` (identical code and calls) | 0 | 177,278,484 and 3,903,384 in both |
| library, shared call sites | 0 | §5.2, checked |
| library, descriptor initialisation | +378,882 charged, −4,447,632 not credited | §5.2 |

**The conservative interval: [54.55 M, 62.28 M] removed.**
- **The lower end, 54,550,306:** the rows; every non-row term with a counter frequency; every
  positive callgrind-only term charged, with none of the negative ones credited; the library
  charges in full, with nothing credited.
- **The upper end, 62,277,795:** callgrind's exact inclusive difference, 1,294,427,597 →
  1,232,149,802, in both runs, with the `rep movsb` credit.
- **The own-code figure, 58,209,045,** is the primary §5 sum and callgrind's own-code delta.
  The two agree exactly.
- **The lower end exceeds the 40 M minimum by 14.55 M. Gate 3 passes.** No relevant added
  cost is unbounded: every changed path is counted, and every library term is either equal
  (checked) or charged in full.

**Against report 227's design:** a gross of 76.59 M was credited, less an allowance of up to
5 M, for ≥ 71.59 M. The achieved own-code figure is 58.21 M: the gross was met exactly, and
the overhead was +18.38 M.

**Reads and writes** (callgrind `Dr` and `Dw`, explicit data accesses, reported and not
gated):

| | B2 | prototype | Δ |
|---|--:|--:|--:|
| `build::<false>`'s own reads | 205,899,121 | 203,503,550 | −2,395,571 |
| `build::<false>`'s own writes | 132,435,558 | 132,241,802 | −193,756 |
| `refill`'s own reads | 16,929,671 | 18,229,149 | +1,299,478 (`pop %r12`) |
| `refill`'s own writes | 16,856,671 | 22,054,583 | **+5,197,912** (4 per refill: two pushes, the third reset store, the second `r0` store) |
| inclusive reads | 275,037,219 | 269,591,578 | −5,445,641 |
| inclusive writes | 182,188,379 | 182,900,709 | **+712,330** |

The inclusive figures include the descriptor initialisation's `rep movsb`, counted per byte
(−4.35 M reads and −4.29 M writes). Without it, own code reads −1.10 M less and writes
**+5.00 M** more. The prototype trades instructions and reads for writes in `refill`.

## 6. Overlap with prototype 1 (charter §5.5)

- **This prototype's credited set** is the two descriptor-address LEAs in each of the four
  scalar copies, at v2 line 542 (B2's `33a05`/`33a09`, `34355`/`34359`, `343e4`/`343e8`,
  `344d5`/`344d9`). **Prototype 1's final listing (`7f74e9f`) still has all four
  `movzbl` → `lea (%r,%r,4)` → `lea (%r,%r,8)` sequences.** So it removes none of them.
- **Prototype 1's credited set** is the fresh pair's and the lookahead's checks (classes A
  and J). **In this prototype, no instruction class executing A, J or U times changed**
  (`callgrind/classes-run1.txt`), so it removes none of them.
- **Instruction-level intersection of the credited sets: none.**
- **Where they interact:** both change register allocation and stack slots across the scan.
  The textual listing diffs (`listing-diff.txt` here, and prototype 1's) share 34 v2 source
  lines, with register renaming in the scan, the refill detours, `run`, `fill` and
  `complete`. Both charge the outer iteration (prototype 1: 13 → 11; this one: 13 → 17)
  and `run` (+3 each). So their savings aren't additive without a combined build: the
  combined outer-iteration and refill-detour costs are unknown until compiled together.

## 7. Deviations from the pre-registration

1. **§0.7.2(b)'s premise is partly false** (§5.2): the builders don't make the same library
   calls at descriptor initialisation. Following the registered rule, those calls are not
   treated as a zero delta. They are listed per call site, charged in full where they add
   work, and their saving is not credited at the conservative end. The shared call sites are
   equal, checked, as registered.
2. **The count harness gained `--only v2|d`** after the suites ran, to put each builder in its
   own process as §0.7.2 requires. It changes no builder (§2), and the suites' builders are
   identical code. The listings were re-archived from the counted binary, the quick suite was
   re-run on it (`synthetic-quick-counted-binary.console`: 745 cases, every check exact), and
   the Miri classes ran on this source. A clean rebuild reproduces the binary's sha256.
3. **Not every changed path has a counter** (charter §5.3b). The counters cover the rows, the
   outer iteration, `refill`, its `finish` call, `run`, the snapshot, the nonuniform tile and
   the block: −56,102,571 in all. The other changed paths (§4.2: `fill`, `open`, `run`'s
   whole-group branch, the final sweep, the refill detours, initialisation and 9 small
   classes) take their frequencies from callgrind's exact per-instruction counts of the
   uncounted build. No counter was added for them. The conservative end treats them
   least favourably instead: their +1,173,383 is charged and their −3,279,857 is not
   credited (§5.3).
4. **Earlier callgrind runs** (both builders in one process, binary
   `82a59cc3…`) are superseded and not used for any figure. They showed −62,271,757
   inclusive, with the shared `memcpy` sites differing by heap layout.

## 8. Limitations and negative findings

- **The 5 M allowance didn't survive compilation** (§4.2): +18.38 M net, chiefly the outer
  loop's head (+12.0 M) and `refill` (+9.1 M).
- **`refill` writes more:** +4 writes per refill, +5.2 M in all. This is untimed. Whether
  the stores cost anything on a machine is not measured here.
- **The register form of the increment** (attempt 1) saves a read per row at an instruction
  per row. It isn't kept (§2).
- **Only the primary corpus is ledgered.** The other corpora were run for correctness, not
  counted.
- **Maximum size and wide records** are covered in release builds only (§0.7.3). Miri runs
  the nine registered classes.
- **No combined build with prototype 1** was made (§6).

## 9. Next

- The coordinator decides whether to open the PR. The branch is pushed, and nothing is merged.
- Stacking with prototype 1 needs one combined, pre-registered build: the outer iteration and
  refill detours are where the two interact (§6).
- Recovering the overhead (the outer loop head's reloads, `refill`'s pushes and the redundant
  `r0` store) would be a separate change, pre-registered on its own.

**The evidence in `2026-11-04-compact30-descriptor-prototype/`:**
- `attempt-1/` and `attempt-2/`: each attempt's listings and source diff;
- `listings/`: the counted binary's `build::<false>`, `refill` and `finish`, v2's
  `build::<false>` and `refill` from the same binary, the row loops of both, and
  `ldiff.py`;
- `listing-diff.txt`: the opcode-level diff against v2, with the source line of every
  changed instruction;
- the corpora, adversarial and synthetic consoles and records, and `corpora.sha256`;
- `mutation-M1` and `mutation-M2`: each mutant's console and diff;
- `miri/`: each class's console, the summary and the toolchain;
- `callgrind/`: the README (toolchain, invocation, binary sha256), the raw profiles of all
  four runs (gzipped), the per-function and per-call-site tables, the class decompositions,
  and the parsers.

## 10. Addendum (2026-11-04): S62-F01's counters and the counted ledger

This addendum answers review finding S62-F01 and the Optimiser audit's F3. Charter §5.3(b)
requires an observed execution count for every changed path, from a counter added to the
counted instantiation only, with the uncounted listing shown unchanged. §7(3) records that
`fill`, `open`, the whole-group branch, the sweep, the refill detours, initialisation and
small classes took Callgrind-only frequencies. This section adds those counters and replays
§5.3 with counted frequencies. It records numbers only; it makes no qualification decision.
S62-F02 (the overlap) is not addressed here.

Evidence: [2026-11-04-compact30-descriptor-f01/](2026-11-04-compact30-descriptor-f01/), with
its `README.txt`. Untimed: no benchmark, no perf, and no timing run.

### 10.1 The counters (counted build only)

Every changed frequency class was read from the listings, with each instruction's own count
from the prototype's raw profile, and assigned to its source path or paths. There are 27 new
`Stats2d` fields. Each is incremented only inside `if COUNT { … }` (that is, only in
`build::<true>`), or in the harness's `same_as_v2`. `build::<false>`, `W<false>` and every
other line of the constructor are unchanged.

One new harness mode was added: `ctor2d-count --counted FILE`. It runs the counted build
alone, in the profiled `--only d` process's order and arenas: one `Ctor2d`, one pair of
arrays, and the 434 blocks in order.
- **Why it is needed.** Six paths (arena regrowth and the reserves) depend on arena history,
  which the three-thread check harness doesn't share.
- **How it was checked.** Its output digest, `4efb1c13ae58340f`, is the profile run's. Its
  record differs from the check harness's primary record in exactly those six fields, and in
  no other.

The ledger uses the sequential record. The table has one row per changed class; "(existing)"
marks a class whose counter already existed:

| function | path | counter | counted | profile | B2 → prototype | Δ |
|---|---|---|--:|--:|---|--:|
| `build` | full tiles' first-segment row loop | `rows_scalar - tile_rows_second - tail_rows` | 29,357,924 | 29,357,924 | 12 → 10 | −58,715,848 |
| `build` | full tiles' second-segment row loop | `tile_rows_second` | 8,935,084 | 8,935,084 | 12 → 10 | −17,870,168 |
| `build` | outer scan iterations (existing) | `outer_iterations` | 3,003,834 | 3,003,834 | 13 → 17 | +12,015,336 |
| `build` | nonuniform full tiles S (existing) | `outer_iterations - runs` | 2,393,313 | 2,393,313 | 38 → 37 | −2,393,313 |
| `build` | fill calls (k > 0) | `fills` | 1,085,833 | 1,085,833 | 25 → 24 | −1,085,833 |
| `build` | fill's vector loop entry (m ≥ 8) and its alias check | `fills_vector` | 801,475 | 801,475 | 18 → 19 | +801,475 |
| `build` | runs (existing) | `runs` | 610,521 | 610,521 | 32 → 35 | +1,831,563 |
| `build` | runs ending after a fill (B2 return path) | `runs_end_fill` | 597,255 | 597,255 | 2 → 0 | −1,194,510 |
| `build` | open from run | `run_opens` | 496,004 | 496,004 | 34 → 33 | −496,004 |
| `build` | B2's snapshot loop: 256 ranks, two per iteration (fixed trip) (existing) | `128 * epochs` | 467,712 | 467,712 | 14 → 0 | −6,547,968 |
| `build` | refill detour from a full tile's second segment | `tile_refills_second` | 302,049 | 302,049 | 9 → 8 | −302,049 |
| `build` | run's whole-group branch (complete) | `complete_calls` | 296,405 | 296,405 | 43 → 44 | +296,405 |
| `build` | the snapshot loop: ranks 4..256, four per iteration (fixed trip) (existing) | `63 * epochs` | 230,202 | 230,202 | 0 → 28 | +6,445,656 |
| `build` | final sweep loop head (two checks per iteration, plus the exit) | `sweep_checks // 2 + blocks` | 55,986 | 55,986 | 3 → 2 | −55,986 |
| `build` | final sweep loop body (two checks per iteration) | `sweep_checks // 2` | 55,552 | 55,552 | 13 → 14 | +55,552 |
| `build` | descriptor initialisation and the cumulative-rank loop (four per iteration) | `init_descs // 4` | 27,776 | 27,776 | 35 → 31 | −111,104 |
| `build` | crossing groups preset RAW (create_raw) | `preset_crossing` | 18,684 | 18,684 | 36 → 37 | +18,684 |
| `build` | runs ending after complete (B2 return path) | `runs_end_complete` | 13,266 | 13,266 | 2 → 0 | −26,532 |
| `build` | epochs (existing) | `epochs` | 3,654 | 3,654 | 33 → 53 | +73,080 |
| `build` | epochs with a full tile (scan loop entry) | `epochs_tiled` | 3,575 | 3,575 | 5 → 3 | −7,150 |
| `build` | the tail's first-segment row loop | `tail_rows - tail_rows_second` | 1,769 | 1,769 | 12 → 10 | −3,538 |
| `build` | refill detour from the tail's first segment | `tail_refills_first` | 786 | 786 | 12 → 13 | +786 |
| `build` | blocks (existing) | `blocks` | 434 | 434 | 204 → 211 | +3,038 |
| `build` | bitmap resize without a reserve | `bitmap_in_place` | 432 | 432 | 6 → 5 | −432 |
| `build` | the last group's padding (memset) | `pad_blocks` | 233 | 233 | 10 → 11 | +233 |
| `build` | tails & bitmap resize's fill (≥ 2 words) | `tails` & `bitmap_memsets` | 230 | 230 | 41 → 42 | +230 |
| `build` | the tail's second-segment row loop | `tail_rows_second` | 106 | 106 | 12 → 10 | −212 |
| `build` | epochs with no full tile (B2 path) | `epochs_short` | 79 | 79 | 3 → 0 | −237 |
| `build` | refill detour from the tail's second segment | `tail_refills_second` | 9 | 9 | 12 → 14 | +18 |
| `build` | raw arena regrowth | `raw_grows` | 8 | 8 | 15 → 13 | −16 |
| `build` | rec arena regrowth & bitmap reserve | `rec_grows` & `bitmap_reserves` | 2 | 2 | 28 → 27 | −2 |
| `build` | rec regrowth freeing the old allocation & snapshot reserve & bitmap reserve of one word | `rec_grow_frees` & `snaps_reserves` & `bitmap_reserves_single` | 1 | 1 | 16 → 14 | −2 |
| `refill` | refill calls (existing) | `refills` | 1,299,478 | 1,299,478 | 59 → 65 | +7,796,868 |
| `refill` | refills that finish a portion (existing) | `refill_finishes` | 1,262,935 | 1,262,935 | 2 → 3 | +1,262,935 |

**Every counted value equals the profile's frequency exactly.** That includes each path of
the three classes that hold more than one path (230, 2 and 1).

Notes on the mapping:
- **Three classes hold more than one path.**
  - 230: the tails, and the bitmap resize's fill.
  - 2: `rec`'s regrowth, and the bitmap's reserve.
  - 1: a regrowth that frees, the snapshot reserve, and a one-word bitmap reserve.

  A class counts as counter-supported only when every one of its paths' counters equals
  its frequency.
- **Fixed-trip loops.** The snapshot loops, the sweep and initialisation are loops whose trip
  count is fixed: 256 characters, at the compiled unroll (`128 * epochs`, `63 * epochs`,
  `sweep_checks // 2`, `init_descs // 4`). The sweep and the cumulative-rank loop have their
  own counters. The `cold` array literal's initialisation loop has the same 64 trips per
  block.
- **The alias check.** As in prototype 1, `fill`'s vector entry tests m ≥ 8. Its alias check
  compares the field array with the static `PATTERNS`, which are distinct allocations, so the
  check always passes.
- **Labels corrected (frequencies unchanged).** §4.2 gave some classes labels that the
  counters correct:
  - 597,255 is `run`'s return after a fill that ends the run (B2's return path), not "`open`'s
    RAW-branch join";
  - 13,266 is the return after `complete`;
  - 3,575 is the scan-loop entry of an epoch with a full tile;
  - the first 18,684 "`open` path" is `create_raw` in the crossing pre-marking loop.
- **The harness checks partitions.** `same_as_v2` now asserts:
  - `runs_end_fill + runs_end_complete = runs`;
  - `epochs_tiled + epochs_short = epochs`;
  - `sweep_checks = init_descs = 256 × blocks`;
  - bounds on the rest.

  No assertion fired on any suite.

### 10.2 The uncounted listing is unchanged

- **The pinned binary reproduces.** A clean build of `29dd071f` with the prototype's recipe
  has sha256 `695ca5dd…770d136`, the archived binary. The counted-counter binary is
  `5b1fed92…7e4bcb`.
- **The tool.** `asmid.py` follows `build::<false>`'s transitive callee closure in both
  binaries. Each function is compared over its symbol extent after
  `bench/c30_build_listing.py`'s normalisation, with comment symbols kept. The sizes must be
  equal.
- **The result is identical.** All 52 functions (4,132 instructions) match:
  - `build::<false>`, 1,450 instructions;
  - `refill`, 74;
  - `finish`, 379;
  - `canonicalise`, 173;
  - `RawVec<RawG>::grow_one`;
  - the allocator and panic paths.

  The new binary's `build::<false>`, `refill` and `finish` also equal the archived listings.
- **The control shows the tool detects a change.** The same tool on `build::<true>` reports
  DIFFERENT.

### 10.3 The suites

The counted-counter binary passes every suite, each exit 0, with every check exact:
- corpora: 11 × 4, 44 records;
- adversarial: 200 records;
- synthetic: full and quick.

Every pre-existing field of every jsonl record equals the archive. So does every console,
once the 27 new fields are removed.

### 10.4 The ledger, replayed with counted frequencies

`f01_replay.py` reads the prototype's raw profiles (the S62 review's parser, copied
unchanged) and the sequential counted record. It checks each changed class's counters
against the profile's frequency.

| | §5.3 / S62 replay | counted (this section) |
|---|--:|--:|
| counter-supported Δ (`build::<false>` + `refill` own) | −56,102,571 | **−58,209,045** |
| profile-only added | +1,173,383 | **0** |
| profile-only removed (withheld) | −3,279,857 | **0** |
| library: 10 shared sites | equal | equal |
| library: added initialisation calls (434 each) | +378,882 charged | +378,882 charged |
| library: removed 10,240-byte `memcpy` (434 calls) | −4,447,632, upper end only | −4,447,632, upper end only |
| **interval removed** | [54,550,306, 62,277,795] | **[57,830,163, 62,277,795]** |
| minimum | 40,000,000 | 40,000,000 |

- **What the width is.** The remaining width, 4,447,632, is exactly the removed `memcpy`. Its
  frequency is counted (434 = `blocks`), but its per-call cost (10,248 instructions per
  10,240 bytes, consistent with `rep movsb` counted per byte) is a counting convention. It
  stays uncredited at the lower end (§5.2).
- **The memory deltas are unchanged.** They are reported, not gated (own code):
  - reads: −1,096,093 (`build::<false>` −2,395,571; `refill` +1,299,478);
  - writes: **+5,004,156** (`build::<false>` −193,756; `refill` **+5,197,912**, four per
    refill).

### 10.5 The caller boundary (S62-F04)

- **The harness's own count rises.** `c30walk::main`'s own count goes from 4,499,110,446
  (v2 alone) to 4,499,111,314 (prototype alone): +868 = 2 × 434 blocks.
- **The call sites match.** `harness-callsite.asm` lists both call sites with each
  instruction's count in both profiles. Each arm sets up and makes the call in 10
  instructions: 7 for arguments, then the `call`, `test` and branch. They are identical but
  for the arena's stack offset.
- **The difference is the dispatch.** Both arms share the length test `cmp $0x2`. v2's arm
  then tests `"v2"` (`cmpw`, `jne`). The prototype's arm re-tests the length (`cmp $0x1`,
  `jne`) and the byte (`cmpb $0x64`, `jne`): two more instructions per block.
- **So it is not constructor work.** The +868 is the harness's string dispatch, outside
  `build::<false>`'s closure.
- **If it were charged anyway,** the interval would be [57,829,295, 62,276,927].

### 10.6 Commands

These run from the worktree root. `README.txt` gives them in full.

```sh
CARGO_ENCODED_RUSTFLAGS= CARGO_TARGET_DIR=target/rust-1.97.0 cargo +1.97.0 build --locked --release --example c30walk
F=docs/journal/2026-11-04-compact30-descriptor-f01; P=docs/journal/2026-11-04-compact30-descriptor-prototype
python3 $F/asmid.py PINNED_BINARY target/rust-1.97.0/release/examples/c30walk OUT '<c30walk::ctor2d::Ctor2d>::build::<false>' \
  --archived "<c30walk::ctor2d::Ctor2d>::build::<false>=$P/listings/ctor2d-build-false.asm"
target/rust-1.97.0/release/examples/c30walk ctor2d-count --counted corpora/wikidata-20261026.bz2 > $F/counted/sequential.jsonl
python3 $F/f01_replay.py descriptor $P $F/counted/sequential.jsonl ledger.json
python3 $F/callsite.py PINNED_BINARY $P/callgrind/only-v2-run1.callgrind.gz $P/callgrind/only-d-run1.callgrind.gz \
  '<c30walk::ctor2d::Ctor2d>::build::<false>'
```
