# Compact30: constructor v2, the deferred-offset prototype (compile and correctness only)

The one bounded prototype that report 206 authorises for Astra's second design (report 205,
[2026-11-03-compact30-constructor-v2-design.md](2026-11-03-compact30-constructor-v2-design.md)),
with report 208's clarifications.
- **No timing of any kind,** no benchmark or perf/cycle measurement, and no i9 or
  workstation run. **Instruction counts below are static paths through the listing, not
  cycle or time measurements, and no speedup is claimed or forecast.**
- **Frozen and unchanged:** constructor v1, the walker, the oracle `C30::build`,
  production's `src/`, and the construction harness.

**Recommendation: PROCEED-TO-REVIEW** (§8).

## 1. The pin and the files

- **Implementation:** `8beffe2` on `research/compact30-constructor`.
- **New:** `examples/c30walk/ctor2.rs` (`Ctor2`, its harness and its synthetic cases).
- **Wiring:** `examples/c30walk.rs` gains 7 lines, all additions: `#[path] mod ctor2;` with
  its doc line, and a three-line `ctor2-` dispatch.
- **Unchanged, each diff empty:**
  - `git diff e003a32 8beffe2 -- examples/c30walk/ctor.rs`: v1, byte for byte;
  - the walker (`examples/c30walk.rs` gains only the lines above);
  - `src/`;
  - `examples/c30walk/cbuild.rs`, `bench/`, `.just/` and the `Justfile`.
- **Reused from v1, read-only:** `ctor::canonicalise` is called as it is, and v1's
  `Ctor::build::<true>` is called for the traffic comparison (§5). The patterns, the
  forced predicate and the run scan are copied into `ctor2.rs`.
- **Toolchain:** rustc 1.97.0 (`2d8144b78`, LLVM 22.1.6), verified with `rustc +1.97.0 -vV`;
  `CARGO_ENCODED_RUSTFLAGS=`, `--locked`, the release profile, baseline x86-64.

## 2. What was built

`Ctor2::build::<COUNT>(ll, counts, permille, &mut rec, &mut raw) -> bool` has the same
contract as v1's: the oracle's arrays exactly, or `false` with both arrays empty.

- **Kept from v1:**
  - the validation (counts summing to n, n < 2²⁴, n ≤ 983,010);
  - the RAW reserve bound and the release-before-grow allocation;
  - the crossing pre-marking, then the forced pre-marking;
  - production's 16-byte uniformity test with run coalescing;
  - the complete-record emitter: final records, never normalised afterwards;
  - the last group's padding, PARK, and `canonicalise`;
  - `set_len` only at the end.
- **Per-character write descriptors** (`Desc`, 40 bytes × 256). Each holds `write` (the
  next field), `end` (the portion's end), `w0` and `r0` (the portion's first field and its
  destination rank), `lim` (the bucket's end rank), `g` (the group), and `raw` (the
  provisional RAW index of a preset group, or `NO_RAW`).
  - A **portion** is a character's slots in one group: the whole group, or its share of a
    crossing group.
  - The character's next rank is `r0 + (write − w0)/2`. An empty descriptor has
    `w0 = write = end = null`.
- **The append** (`one`): `if write == end { refill }`, then `*write = u; write += 1`.
  `refill`, which is cold and out of line, finishes the full portion, then opens the next
  one or rejects an exhausted bucket.
- **Source labels per tile** (`tile`): a 16-row tile's labels are its base, `y as u16`, plus
  the row's offset, plus 2 after the tile's one possible label hole (`n1 = min(16, 30 − p)`).
  y and p advance once per tile.
- **Runs** (`run`): one limit check against the bucket, then a loop:
  - the open portion's remainder is filled from the pattern (`fill`);
  - the portion is closed;
  - whole groups at a group boundary go to the bulk emitter;
  - the suffix opens a new portion.
- **Epochs:** the scan runs epoch by epoch over 61,440 source rows. Before each epoch every
  character's next rank is saved (`snapshot`, `Vec<[u32; 256]>`, at most 16, reserved
  once). Coalescing stops at the epoch's end, so runs are split there.
- **Finishing** (`finish`, out of line), once per portion. It is called from `refill`, from
  a run's close, or from the end of the scan:
  - **Fast case:** the latest snapshot ≤ the first rank, so h = the current epoch for every
    row.
  - Otherwise `epoch_of` scans the snapshots down for the greatest e with A_e[c] ≤ r, for
    the first and last ranks.
  - **Ordinary candidate:** y₀ = 65536·h₀ + u₀ and y_last = 65536·h_last + u_last, **full
    labels, before any subtraction**.
    - If y_last − y₀ ≤ 65,533: the header is (y₀ << 8) | c. A full group's 60 field bytes are
      read whole into a register copy, `wrapping_sub(u₀)`, then written back. A partial
      last group reads and writes only its own k fields.
    - Otherwise wide: a zeroed `RawG` is pushed, its entries written as
      ((65536·h(r) + u) << 8) | c, epoch interval by epoch interval (`write_entries`), and
      the record replaced (all fields 0xFFFF, the RAW header), with the bit set.
  - **Preset RAW portion** (crossing or forced): the entries go into its preset `RawG`, then
    only that portion's k fields are restored to 0xFFFF.
- **Forced configurations:** a complete group that is preset RAW gets its entries
  (y + pattern) directly, with no portion.

**Choices left open by the design:**
- **A portion is finished at its character's next arrival,** or at the end. A run that
  continues past a full portion closes it at once, since the run's next row is that
  arrival.
- **The temporary labels** are `φ(i) as u16`, in 0..=65533 (φ(61439) mod 65536 = 65533).
  They never equal 0xFFFE or 0xFFFF, and nothing relies on that, since no temporary label
  is ever published.
- **No variant was made,** and no tuning was attempted.

## 3. Correctness: every check passed

Each case runs the following, with both arenas reused across blocks and sizes:
- the oracle;
- `Ctor2::build::<true>` into an arena poisoned with one of two alternating patterns
  (0xA5A5 or 0x5A5A), compared with the oracle's `rec` and `raw` in length and every byte;
- `build::<false>`, re-poisoned with the other pattern, compared again;
- the unchanged walker on Ctor2's arrays, counted and audited, against production's output,
  with the counted walk's output and all transition counts against the oracle arrays' own;
- the arrays against the oracle's after each walk.

| class | cases | array comparisons | walk outputs | restorations | result |
|---|--:|--:|--:|--:|---|
| 11 hash-checked corpora (11 of 11 OK) × 4 configurations | 44 (6,052 blocks) | 12,104 | 18,156 | 12,104 | **exact** |
| the adversarial set, 25 inputs × levels 1, 9 × 4 configurations | 200 | 1,056 | 1,584 | 1,056 | **exact** |
| synthetic, full | 53,924 | 99,768 | 149,652 | 99,768 | **exact** |
| synthetic, quick (release) | 524 | 972 | 1,458 | 972 | **exact** |
| synthetic, quick, **under Miri** (first five classes; stopped in the sixth, see below) | 473 | | | | **exact, no UB reported** |

**The synthetic classes** (`ctor2-synthetic`), every one exact:

| class | cases | what it covers |
|---|--:|---|
| small blocks | 4,192 | every n 0..=130, σ ∈ {1, 2, 3, 5}, mixed runs. n = 0 goes through **both** builds (the audit's gap): PARK alone, `raw` empty |
| a bucket boundary at every offset | 1,620 | two bytes, B = 1..n−1 for seven n, three arrangements: crossing groups shared by two characters (ownership) |
| bulk-to-partial transitions | 43,740 | a run of m (18 lengths) after P = 0..240 filler rows, with the bucket at every slot 0..29, then the same byte again in a scalar row |
| epoch boundaries | 60 | n around 61,440·k (k ∈ {1, 2, 3, 5, 14, 15}; n = b−1, b, b+1, b+30, b+61), a rare byte's rows at offsets −2..30 around the boundary, a run straddling it, natural and forced |
| absent characters across repeated snapshots | 60 | byte 1 in epoch 0, then absent for 1, 2, 3, 6 or 13 epochs (equal snapshots), then back inside the same group (wide), alone in a later epoch (ordinary, non-fast), or in a second group |
| ordinary/wide span boundaries from full labels | 97 | **source 3 → 61,440: span 65,533, ordinary** and **2 → 61,440: span 65,534, RAW** (asserted). Spans 65,530..=65,538 at nine first positions up to 30,001; **far-wider groups** at last = first + 61,440·k (+0, 1 or 31), whose truncated u16 difference is small: all RAW (asserted) |
| pending portions finished at the end of input | 18 | a byte whose last group never sees a later occurrence (1 to 61 rows, partial and full last groups) in 61,500- to 300,017-row blocks |
| RAW discovery order and forced RAW | 18 | rare bytes spread over 300,000–899,999-row blocks (wide groups found late) at permille 0, 50, 1000 |
| one-symbol blocks | 57 | n ∈ {1, …, 61, 61,439, 61,440, 61,441, 65,536, 899,999, 900,000} × three bytes |
| 900,000-row and 983,010-row blocks | 30 | random (σ 256 and 4) and runs (σ 20, 98, 2) at permille 0, 50 and 1000; 983,010 rows gives 16 epochs, the workspace's maximum |
| repeated construction with changing sizes | 30 | 900,000 → 1 → 899,990 → **0** → 31 → 65,537 → 30 → 900,000 → **0** → 59 → 500,000 → 61,441 → **0** → 65,536 → 899,999, up and down, unpoisoned then poisoned, permille cycling 0 / 1000 / 50 |
| malformed counts and sizes | 4,002 | 4,000 perturbed counts, **3,995 rejected and 7 accepted, exactly when the counts equal L's own**; both builds agree; rejected builds expose empty arrays. Plus n = 983,011 and n = 2²⁴ rejected |

**The synthetic run's own tallies** (`build::<true>`):

| tally | count |
|---|--:|
| fast finishes | 2,436,958 |
| non-fast ordinary finishes | 19,497 |
| wide finishes | 2,248 |
| crossing portions | 262,835 |
| forced portions | 214,976 |
| portions whose rows span two or more epochs | 31,299 |
| epoch searches | 33,209 |
| runs split at an epoch boundary | 646 |
| distinct (first slot, length) portion shapes | 367 |
| the largest ordinary span seen | **65,533** |
| the smallest wide span seen | **65,534** |

**Miri** (`cargo +nightly miri run --example c30walk -- ctor2-synthetic --quick`; nightly
rustc 1.101.0, `16d1ffac1`; Stacked Borrows, no flags; unpoisoned arenas, so any read of an
unwritten byte would be reported): the run was stopped from outside after 1 h 35 min, while it was inside the sixth class. The five classes before that completed, every case exact and with no undefined behaviour reported: **small blocks** (260 cases, n 0..=64, σ ∈ {1, 2, 3, 5}, including n = 0 through both builds), **a bucket boundary at every offset** (180 cases, n = 61: crossing groups shared by two characters, so **crossing RAW was reached**), **bulk-to-partial transitions** (24 cases), **epoch boundaries** (3 cases, n = 61,501, offsets −1, 0, +1 around row 61,440: snapshots, the epoch search and a run split at the boundary, **above 61,440 rows**), and **absent characters across repeated snapshots** (6 cases, n = 61,845, gap 1, first ∈ {0, 29}: the "wide" case finishes a group whose rows span epoch 0 and row 61,445, so **wide RAW from full labels was reached above 61,440 rows**, as were an ordinary non-fast group in epoch 1 and two groups across the gap). **Miri did not reach** the exact span cases 3 → 61,440 and 2 → 61,440 (the sixth class, cut off), the pending portions at n = 61,500, repeated construction, malformed counts, or any forced RAW (none of the five completed classes uses a permille above 0). Those are covered only by the release runs above. A 61k-row case costs about 9 minutes under Miri. The console is `miri-quick.console` in the evidence folder.

## 4. The initialisation and bounds argument

**Notation:** G = ⌈n/30⌉. The record arena `rec` has capacity ≥ G + 1, 64-aligned, length 0
until the single `set_len(G + 1)`. `dst` is its base.

**1. Malformed input is rejected before any out-of-range write.**
- The size checks and Σcounts = n come first (with a 64-bit `usize`, n ≤ 983,010 makes
  every rank and pointer offset below fit; the arithmetic is u32 or usize with no overflow).
- Each descriptor starts at `r0 = C[c]`, `lim = C[c] + counts[c]`, with lim ≤ n.
- **A field is written only through `d.write`.** `open(c, r)` is called only with r < lim,
  which `refill`, `run` and the end check enforce. It sets `end = w0 + (re − r)` with
  re = min(30·(r/30) + 30, lim) ≤ n, so `[w0, end)` lies in record r/30 < G, fields
  r mod 30 .. re − 30·(r/30) − 1 ≤ 29.
- `one` writes only after `write != end` (or a successful `refill`). `fill` writes
  m ≤ (end − write)/2 fields, and `run` takes m from the room.
- `run` first checks rank + m ≤ lim, before any write, so every bulk group
  (r + 30·whole ≤ lim ≤ n) is below G.
- **These conditions are ordinary release-mode branches,** not `debug_assert!`. A bucket
  that overflows, or an absent byte's bucket (lim = C[c]), fails `r < lim` before any
  write, and the build returns false.
- The other indices:
  - `desc` is indexed by a byte (256 entries);
  - `snaps` is a `Vec`, bounds-checked, at most ⌈n/61440⌉ ≤ 16 pushes, reserved once;
  - `raw` is written through bounds-checked indexing;
  - the bitmap is a bounds-checked `Vec`.
- **The `chk!` conditions** (g < G, r < lim, r mod 30 = 0 for an ordinary candidate, the
  room) are assertions in the counted build and debug assertions in the uncounted one. They
  restate conditions that the branches above already establish.

**2. Every slot below n is written exactly once before publication.**
- A slot's first write is the one append, bulk or forced-bulk write it receives.
- Each character's rank rises by exactly one per row placed, through append, fill or bulk,
  from C[c].
- The build publishes only after every rank equals its lim (checked after the scan). With
  Σcounts = n and the buckets tiling [0, n), every slot received exactly one row.

**3. Every temporary label is overwritten before publication.**
- Temporary labels are written only into a descriptor's open portion.
- A portion is closed, and finished if it has rows, when its character's next arrival finds
  it full (`refill`), when a continuing run closes it, or at the end, where `close` runs for
  all 256 characters after the rank check. Each descriptor holds at most one open portion,
  and every earlier one was finished at its close. So every portion with rows is finished
  exactly once.
- `finish` requires k = (write − w0)/2 ≥ 1 and write = end, so the portion is full.
- It overwrites all k fields:
  - with their offsets (ordinary);
  - with the whole record (wide: all 30 fields 0xFFFF and the RAW header);
  - with 0xFFFF (a preset portion's own k fields).
- The bulk emitter writes final records directly and is never normalised. A forced complete
  group's fields were 0xFFFF from `create_raw` and are not touched.
- **No cross-character overwrite:** a portion's fields are its character's own slots in its
  group (`[r, re)` within one bucket), and buckets are disjoint, so two characters' portions
  in one crossing group cover disjoint fields. Restoring 0xFFFF touches only `[w0, w0 + k)`.

**4. Every header and padding word is written before publication.**
- **Preset RAW groups** (crossing, forced): the whole record (fields 0xFFFF, header
  RAW_BIT | idx) is written by `create_raw` before the scan. Only the header changes
  afterwards, in `canonicalise`.
- **A wide group:** the whole record is written at its finish.
- **An ordinary group:** its header is written at its finish (bulk: by the emitter). Its
  slot-0 row is in its one portion, which covers the whole group: an ordinary candidate
  isn't crossing, so its group's slots all lie in one bucket, and the portion is opened at
  slot 0 (the `chk!` restates this).
- The last group's slots ≥ n − 30(G − 1) get 0xFFFF; PARK is written explicitly.
- **`RawG` words:** zeroed at the push; entries written at slots < 30 only; word 31 used and
  zeroed by `canonicalise` (unchanged from v1); word 30 never written.

**5. No uninitialised read.**
- `finish` reads only the portion's fields `[w0, w0 + k)`, which its appends wrote.
- The full-group normalisation reads the 60 bytes of a full group, all appended; a partial
  last group reads only its k fields.
- The only header reads are of preset RAW groups in `open` and `complete`, written by
  `create_raw`, and in `canonicalise`, of RAW groups.

**6. The normaliser stays inside bytes 0–59** (report 208; listing §6):
- three 16-byte loads at +0x00, +0x10 and +0x20, all before the three stores at the same
  offsets;
- then an 8-byte load and store at +0x30 (bytes 48–55);
- then a 4-byte load and store at +0x38 (bytes 56–59).

These regions don't overlap, so no field is subtracted twice. Nothing at or past byte 60 is
loaded or stored. The header store at +0x3c comes before them, separately.

**7. Classification uses full labels:** the span is y_last − y₀ with
y = 65536·h(r) + u, computed before the modular subtraction. The tests above cover 3 / 2 →
61,440 and far-wider groups.

**8. Snapshots:**
- A_e[c] is the next rank at epoch e's start, pending rows included (`rank()` counts the
  open portion's written fields).
- Ranks received in epoch e lie in [A_e[c], A_{e+1}[c]), so h(r) = max{e : A_e[c] ≤ r}.
- `epoch_of` scans down from the latest snapshot and stops at the first (greatest) e with
  A_e[c] ≤ r. Equal snapshots from absent epochs resolve to the greatest.
- `write_entries` advances e upward while r ≥ A_{e+1}[c], so ties again resolve to the
  greatest.
- A_0[c] = C[c] ≤ r always holds, so the scan terminates.

**Not proven here:** no formal proof; Miri's coverage is in §3. The argument assumes a
64-bit `usize` and baseline x86-64, as v1's audit did.

## 5. The structural counters (untimed), against v1

**Primary corpus (`wikidata-20261026`, natural configuration):**

| | v1 | v2 |
|---|--:|--:|
| rows by path: bulk / fragment / scalar | 74.340% / 7.160% / 18.499% | 74.306% / 7.194% / 18.499% |
| complete groups emitted | 5,129,645 | 5,127,300 |
| wide groups | 30,949 promotions | 30,949 wide finishes |
| portions finished | — | fast 1,647,416; non-fast ordinary 75,997; wide 30,949; crossing 41,120 |
| normalisations | — | 1,723,371 full groups (one vector pass each), 42 partial |
| epochs | — | 3,654 (434 blocks) |
| portions spanning epochs | — | 117,029 (127,249 epoch searches) |
| runs split at an epoch | — | 2,638 |

Splitting runs at epoch boundaries costs 2,345 complete groups on the primary corpus
(70,350 rows, 0.034% of rows), which go to fragments instead.

**Logical field traffic per input row,** counted from the counters, not measured traffic:

| per input row | v1 | v2 |
|---|--:|--:|
| u16 field stores | 0.2517 (one per ordinary non-bulk row) | 0.5094 (0.2569 temporary + 0.2498 normalised + 0.0027 restored) |
| u16 field loads | 0.0023 (promotion's reconstruction) | 0.2736 (0.2498 normalisation + 0.0167 first/last labels + 0.0072 RAW finishing) |
| u32 header loads | 0.1817 (one per one-row write at slot > 0 and per prefix fragment) | ≈ 0.0002 (one per preset portion opened) |
| u32 header stores, ordinary | (in the per-group start) | 0.0083 |
| u32 RAW entry stores | 0.0072 | 0.0072 |
| complete-record stores (64 B) | 0.0248 | 0.0248 |

- **v2 does about twice v1's explicit field stores, and many more field loads** (the
  re-read and rewrite Astra predicted, about 1.0 B a row of field stores and 0.55 B a row of
  field loads). It removes **the per-row header load** (0.18 a row in v1) and its dependency
  chain.
- These are static logical counts. The net cost is not established here.

**All 11 corpora (natural):** bulk 70.578%, fragment 7.076%, scalar 22.346% (v1: 70.609%,
7.044%, 22.346%). There were 6,427,260 fast, 335,299 non-fast ordinary, 102,600 wide and
171,508 crossing finishes, and 12,467 epochs. Per corpus:

| input | rows | bulk | fragment | scalar | fast | ordinary | wide | crossing | epochs |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| mixed-independent | 2,000,689 | 0.051% | 0.015% | 99.934% | 59,419 | 4,743 | 1,759 | 1,472 | 34 |
| period-3 | 2,700,000 | 99.937% | 0.057% | 0.007% | 6 | 45 | 0 | 16 | 46 |
| random | 8,000,003 | 0.000% | 0.000% | 100.000% | 232,673 | 31,795 | 0 | 4,408 | 134 |
| records | 9,600,002 | 0.000% | 0.000% | 100.000% | 279,161 | 38,144 | 0 | 5,404 | 160 |
| text-l1 | 11,036,297 | 15.306% | 7.884% | 76.810% | 293,837 | 7,255 | 104 | 22,650 | 221 |
| text-l5 | 11,036,297 | 20.146% | 9.438% | 70.416% | 280,392 | 9,152 | 1,788 | 5,407 | 199 |
| text | 11,036,297 | 24.795% | 11.649% | 63.556% | 264,727 | 7,973 | 2,542 | 3,116 | 184 |
| wikidata-20261026 | 207,006,515 | 74.306% | 7.194% | 18.499% | 1,647,416 | 75,997 | 30,949 | 41,120 | 3,654 |
| wikidata-fitted | 209,706,557 | 74.958% | 6.985% | 18.056% | 1,625,027 | 75,952 | 30,755 | 41,407 | 3,693 |
| wikidata-heldout | 235,773,776 | 76.022% | 7.360% | 16.617% | 1,744,600 | 84,243 | 34,703 | 46,505 | 4,141 |
| zeros | 52,945 | 99.839% | 0.128% | 0.032% | 2 | 0 | 0 | 3 | 1 |

Every corpus's wide count equals v1's promotions (the census's wide groups).

**The workspace:**
- snapshots: at most 16 × 256 × 4 = 16 KiB (`Vec<[u32; 256]>`, reserved once);
- descriptors: 256 × 40 = 10 KiB (on the stack, then moved into the build state);
- the bitmap: 3,752 B at 900,000 rows;
- `canonicalise`'s one 128-byte carry.

That is about 30 KiB, slightly over the design's 28 KiB because each descriptor is 40 bytes,
not 32. There is no successor table, F array or second scan of L. `rec` and `raw` are
reserved as in v1.

## 6. The machine code (rustc 1.97.0, `2d8144b78`, release)

Listings are in `2026-11-03-compact30-constructor-v2-prototype/`:
- `ctor2-build-false.asm`: `<c30walk::ctor2::Ctor2>::build::<false>`, 0x18e5 bytes, 1,431
  instructions;
- `ctor2-finish-false.asm`: `W<false>::finish`, 0x5fa bytes, 379 instructions;
- `ctor2-refill-false.asm`: `W<false>::refill`, 0xec bytes, 67 instructions;
- `excerpts.asm`.

**The ordinary scalar append loses the header-read, RAW-test and span-test chain.** A tile's
two part-loops (before and after its label hole) are each 12 instructions per row:

```
343e0:  movzbl (%r15),%esi          ; L[i]
343e4:  lea    (%rsi,%rsi,4),%rcx   ; c·5 (descriptors are 40 bytes)
343e8:  lea    (%r8,%rcx,8),%r14    ; &desc[c]
343ec:  mov    (%r8,%rcx,8),%rax    ; desc[c].write
343f0:  cmp    0x8(%r8,%rcx,8),%rax ; == desc[c].end ?
343f5:  je     3440d                ; -> refill (cold)
343f7:  mov    %bx,(%rax)           ; the temporary label
343fa:  addq   $0x2,(%r14)          ; write += 1
343fe:  inc    %r15
34401:  inc    %ebx                 ; the next label
34403:  cmp    %r12,%r15
34406:  jne    343e0
```

The first part (`34350`–`3437a`) is the same with `lea (%r14,%r13,1),%rcx` forming the label.
- **No record-header load, RAW test, offset subtraction, span comparison or encoded-cursor
  arithmetic** is on the row path.
- **Against v1's 49-instruction scalar iteration** (`23c99`–`23d1d` then `23c60`–`23c93`),
  v2's is **12**. That is a static path count only.
- **What remains per row:**
  - the descriptor's `write` load and store (a memory read-modify-write through `addq`,
    since the descriptor lives in memory, indexed by the byte);
  - the `end` load and compare;
  - the label store.

**Source coordinates are amortised per tile.**
- Within a tile the label is an incremented register (`inc %ebx`).
- After a tile, y and p advance once (`34470`–`344bc`: one comparison with 29, one wrap
  adjustment, two stack stores).
- The `n1 = min(16, 30 − p)` split is computed once per tile.

**Normalisation compiles to bounded vector work.** The full-group path in `finish`:

```
61f74:  movdqu (%rdi),%xmm2        ; bytes 0..15
61f78:  movdqu 0x10(%rdi),%xmm3    ; bytes 16..31
61f7d:  movdqu 0x20(%rdi),%xmm4    ; bytes 32..47
61f82:  psubw  %xmm1,%xmm2
61f86:  movdqu %xmm2,(%rdi)
61f8a:  psubw  %xmm1,%xmm3
61f8e:  movdqu %xmm3,0x10(%rdi)
61f93:  psubw  %xmm1,%xmm4
61f97:  movdqu %xmm4,0x20(%rdi)
61f9c:  movq   0x30(%rdi),%xmm1    ; bytes 48..55
61fa1:  psubw  %xmm0,%xmm1
61fa5:  movq   %xmm1,0x30(%rdi)
61faa:  movd   0x38(%rdi),%xmm1    ; bytes 56..59
61faf:  psubw  %xmm0,%xmm1
61fb3:  movd   %xmm1,0x38(%rdi)
```

- **The loads and stores** are at +0x00, +0x10, +0x20 (16 B), +0x30 (8 B) and +0x38 (4 B):
  every byte 0–59 exactly once, none at or past 60. The header store is
  `mov %esi,0x3c(%r8,%rcx,1)` at `61f4e`, before it.
- **The fast ordinary finish** of a full group is 81 instructions from entry to return:
  - 29 to the fast-case branch;
  - 5 of class dispatch;
  - 39 from the label reads through the full-label span test (`cmp $0xfffe`) and the header
    store to the normalisation;
  - 8 of epilogue.

  That is once per 30 rows.
- **The partial-group path** (`62156` onward) uses 16-field and 4-field `psubw` loops with a
  scalar tail, bounded by k.
- **Not a scalar walk:** the wide and preset RAW paths (`write_entries`) are a per-entry
  scalar loop with an epoch-interval compare, but they run only on wide and crossing
  portions.

**The slow edge** (`refill`, out of line, 67 instructions in all) runs once per portion:
- the `finish` call if there are rows;
- the descriptor's reset;
- the bucket check (`cmp %eax,%r14d; jae` → reject);
- the group by multiply-reciprocal (`imul $0x88888889`), with no `div`;
- the portion end;
- the bitmap test (`bt`), plus the header read only for a preset RAW group.

**The bulk emitter is intact.**
- `341d0`–`34233` is v1's complete-record loop instruction for instruction in form: two
  records an iteration, four pattern loads, eight 16-byte stores, two header stores,
  `lea 0x2000` and `add $0x4000`.
- Its coverage is 74.306% of the primary corpus's rows (v1: 74.340%).

**The workspace is bounded:**
- `snapshot` pushes one `[u32; 256]` per epoch into a `Vec` reserved for 16;
- the descriptors are a fixed `[Desc; 256]`, moved into the build state with one `memcpy`
  per build;
- there is no allocation proportional to n other than `rec` and `raw` (and v1's
  `rec`/`raw` reserve policy).
- There is no `div` instruction in `build::<false>`.

## 7. Uncertainties and negative findings

- **More explicit field traffic than v1** (§5): twice the u16 field stores and about 0.27
  field loads a row, in exchange for removing about 0.18 u32 header loads a row and the
  per-row classification chain. Whether that nets out is unmeasured.
- **The descriptor is updated in memory per row** (`addq $0x2,(%r14)`). That is a
  store-to-load dependency whenever the same byte repeats in consecutive rows. Its cost is
  unknown and not measured.
- **Epoch splitting** moves 0.034% of the primary corpus's rows from bulk to fragments.
- **The workspace** is about 30 KiB with 40-byte descriptors, not the design's 28 KiB.
- **Finishing reads fields that may have left L1.** A portion is finished at its next
  arrival or at the end, possibly long after its rows were written. This is the risk Astra
  and the researcher name; nothing here measures it.
- **The wide and crossing finishing** is a scalar per-entry loop (rare paths).
- **Miri's coverage is partial:** five of the quick classes (473 cases, including crossing RAW, an epoch boundary and wide RAW above 61,440 rows). The exact 65,533/65,534 span cases, pending portions, repeated construction, malformed counts and forced RAW ran only in release builds.
- **Clippy wasn't run** (not installed for 1.97.0). `ctor2.rs` is rustfmt-clean.
- **Not tested:** other targets or a non-64-bit `usize`; the harness's poisoned arena is
  replaced when a block outgrows it (as in v1).

## 8. Recommendation: PROCEED-TO-REVIEW

The prototype meets report 206's two gates as far as this untimed work can show:
- **Correctness:** byte-exact `rec` and `raw` against the oracle, and unchanged walker
  output, on every suite. That includes epoch boundaries, absent characters, pending
  portions, crossing ownership, the 65,533/65,534 boundaries from full labels (including 3
  and 2 → 61,440), far-wider groups, bulk-to-partial transitions, forced RAW, malformed
  counts, and reuse up and down with n = 0 through the uncounted build.
- **Compilation:**
  - the scalar append is 12 instructions with no header/RAW/span chain;
  - source coordinates advance per tile;
  - normalisation is three 16-byte, one 8-byte and one 4-byte `psubw` operation per full
    group, confined to bytes 0–59;
  - the bulk emitter is unchanged;
  - the workspace is bounded.

Whether this is enough to close report 205's economic gap is not addressed. That would need
a separately approved measurement.

## 9. Files

`2026-11-03-compact30-constructor-v2-prototype/`:
- `corpora.console`, `corpora.jsonl` (with v1's traffic counts for the primary corpus),
  `corpora.sha256.check`;
- `adversarial.console`, `adversarial.jsonl`;
- `synthetic.console`, `miri-quick.console`;
- `ctor2-build-false.asm`, `ctor2-finish-false.asm`, `ctor2-refill-false.asm`,
  `excerpts.asm`.
