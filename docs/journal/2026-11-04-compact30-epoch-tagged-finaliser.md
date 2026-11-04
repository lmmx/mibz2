# Compact30: the epoch-tagged ordinary finaliser, one untimed prototype (constructor v3) — STOP on the compilation gate

**Verdict: STOP for this prototype** (a formal STOP under its compilation gate, report 238;
the gate is not relaxed).
- **Correctness passes.** Output is byte-exact everywhere, the walker output is unchanged, the
  markers are restored, the per-case counters match v2's, and every eligible finish takes the
  new leaf.
- **The compiled fast path misses two registered structural targets.**
  - **The body is 44 instructions** against at most 40, and **45 with its unchanged call**
    against at most 41: the audit's convention, as the guardian made it precise.
  - **No caller-side work is new:** the three call sites are v2's, instruction for
    instruction.
  - The whole boundary, with the call sites' existing argument setup, is 46–48.
  - Fast success makes **12 explicit reads** against at most 10.

  Its other targets hold: no snapshot chain, no span test, no register-save frame or
  spill, at most 6 writes (it makes 6), the per-row loops instruction-for-instruction
  unchanged, and the opening one-for-one.
- **One iteration on the Rust expression was made, as allowed, and it did not help:** 45
  instructions (§5.3). No further variant was tried. The scanner cleanup and the descriptor
  split weren't touched; they need the guardian's separate decision.
- **The checks pending at the first report are complete** (§9):
  - **Miri:** all 134 selected cases ran, every check exact, and no undefined behaviour was
    reported.
  - **The mutation check:** with v2's `== NO_RAW` dispatch put back, the quick suite panics
    on its first one-row block.
  - **The rebuild of the formatted source:** the four listings are identical, and every
    suite's output is byte-identical.

**The structural result, recorded as findings** (static, no runtime inferred):
- **The fast full finish fell from 81 instructions to 44** (body, including `ret`).
- **About 60.2 M fewer instructions** on the primary corpus net of every added cost, against
  the conditional 66.041 M (§6).
- **The per-row loops and the finish call sites are unchanged** against v2.
- **The cause of the 4-instruction excess is specific:** LLVM's lane-0 zero
  materialisation in chunk 0. u0 is loaded from field 0, and the header store at +60 can't
  alias the fields, so LLVM knows field 0 − u0 = 0 and builds that lane (§5.1).
- **What it rules in or out:** whether an explicit SSE2 expression would avoid this is
  untested. It is neither ruled out nor authorised. The choice between it and the scanner
  cleanup is the guardian's.

**Authority:** reports 224–237, at `ade14d5`
([the reassessment](2026-11-03-compact30-constructor-v2-reassessment.md)).
- Report 236's "What I would send Claude" is the binding scope.
- The design is report 227's §F, with report 235's dispatch correction.
- Two clarifications followed from the guardian:
  - verify walker output and marker restoration, as v2's `check_block` does;
  - check the eligible full-group finishes against v2's own fast finishes with 30 fields,
    not against v2's `fin_fast`.
- The audit's full report and reproducibility package arrived during this work
  ([2026-11-04-compact30-ultra-audit-evidence/](2026-11-04-compact30-ultra-audit-evidence/),
  `0b4cd00`). Its formula, its bound and its counterexample are cited in §6 from
  `report.txt`. Its scripts were not rerun here.

**Source and evidence.** The listings, suites and counters below came first from a build of
`ctor3.rs` before `rustfmt` re-wrapped a few `assert!` calls (formatting only). The pinned
source (`dbf5139`) is the formatted one. Its rebuild reproduces all of them exactly (§9.3).

**Untimed throughout.** No timing, benchmark, perf or cycle measurement, i9 use, PR, merge,
protocol change or production integration was done. Every figure is a static instruction
count or an untimed structural counter.

## 1. What was built

- **`examples/c30walk/ctor3.rs`** (new): `ctor2.rs` at `8beffe2`, copied, with the
  finaliser change only (§2), and v2's correctness harness adapted.
- **`examples/c30walk.rs`:** a `mod ctor3;` line and a `ctor3-` dispatch beside `ctor2-`,
  nothing else.
- **Unchanged** (each `git diff` empty):
  - `examples/c30walk/ctor2.rs` against `8beffe2`;
  - `ctor.rs`, the walker and `C30::build`;
  - `src/`, `cbuild.rs` and the `bench/` harness.
- **Not implemented:** the scanner and bounds-check cleanup, and the descriptor-array split.

**The change, in ctor2.rs's terms:**

| where | v2 | v3 |
|---|---|---|
| `Desc::raw` | RAW index, or `NO_RAW` = `u32::MAX` | RAW index (high bit clear, < 2²⁰), or `epoch_tag(e) = 0x8000_0000 \| e << 24` for an ordinary candidate opened in epoch e |
| initial `raw` | `NO_RAW` | `UNOPENED` (`u32::MAX`), never read (§3) |
| build state | — | `active_tag: u32`, `UNOPENED` until the first snapshot |
| `snapshot()` | pushes `A_e` | pushes `A_e`, then `active_tag = epoch_tag(snaps.len() − 1)`: entering epoch e, before any of its rows |
| `open()` ordinary branch | `raw = NO_RAW` (immediate) | `raw = active_tag` (a load) |
| `finish` | one function, 81 instructions on fast success | **the leaf** (`#[inline(never)]`, takes `c: u8`): if `raw == active_tag` and `write − w0 == 60` bytes, the header is `(tag & 0x7fff_ffff) \| u0 << 8 \| c` at `w0 + 60` and v2's five-chunk normalisation runs; otherwise a tail jump to `finish_slow` |
| `finish_slow` | — | v2's `finish`, `#[cold]` `#[inline(never)]`, with `d.raw & ORD != 0` in place of `d.raw == NO_RAW`; counted-build assertions and counters added |

## 2. The `Desc.raw` audit (every reader and writer)

| site | access | v2 | v3 | why it is right |
|---|---|---|---|---|
| `build`: descriptor initialisation | write | `NO_RAW` | `UNOPENED` | never read: only a nonempty portion is finished, and a nonempty portion was opened (§3.1) |
| `build`: `W` initialisation | write `active_tag` | — | `UNOPENED` | replaced by epoch 0's tag at the first snapshot, which precedes every opening. For n = 0 there is no snapshot and no opening, and the final sweep closes only empty descriptors |
| `scatter`: crossing setup (`create_raw` per unaligned bucket boundary) | none | record header `RAW_BIT \| idx` | unchanged | `Desc::raw` isn't touched; the index lives in the record header |
| `scatter`: forced setup (`create_raw` where `forced(g)`) | none | as above | unchanged | as above |
| `snapshot` | write `active_tag` | — | `epoch_tag(len − 1)` | the epoch being entered (§3.2) |
| `open`, preset-RAW branch | write | `h & M20` | unchanged | < 2²⁰, so the high bit is clear |
| `open`, ordinary branch | write | `NO_RAW` | `active_tag` | the opening epoch is the first row's (§3.2) |
| `close` | none | doesn't touch `raw` | unchanged | an empty descriptor's stale `raw` is never read; the next `open` overwrites it |
| `refill` | via `close`, `open` | — | unchanged | |
| `run` | via `close`, `open`, `complete` | — | unchanged | |
| `complete`, forced configuration | none | reads the RAW index from the record header | unchanged | not `Desc::raw` |
| `finish` (leaf) | read | — | `raw == active_tag` | true only for an ordinary tag, since `active_tag` always has the high bit set: it is never a RAW index, and it never equals `UNOPENED` (`epoch_tag(e) ≤ 0x8F00_0000`) |
| `finish_slow` dispatch | read | `raw == NO_RAW` | `raw & ORD != 0` | **the audit's repair.** With v2's test, a tagged ordinary portion would take the RAW branch with index ≥ 2³¹ |
| `finish_slow`, RAW branch | read as index | `write_entries(…, d.raw, …)` | unchanged, reached only with the high bit clear | an index from `open`'s RAW branch. The counted build asserts `raw < raw.len()` |
| `finish_slow`, wide promotion | none | fresh `raw.len()` | unchanged | the descriptor isn't updated; `close` empties it next |
| `write_entries` | none | takes `idx` | unchanged | called with a RAW index (high bit clear) or a fresh index |
| final rank check, final sweep | none | — | unchanged | |
| reuse across builds | — | the descriptors are local to `build` | unchanged | `Ctor3` keeps only the bitmap, snapshots and counters; every build reinitialises all 256 descriptors and `active_tag` |
| failure (`refill` or `run` returns false) | — | `scatter` returns at once, `raw` cleared, arrays empty | unchanged | no finish runs after a failure |

**The counterexample is covered.** The class "the tag/RAW dispatch" and the Miri selection
build tagged ordinary portions that reach `finish_slow`:
- one-row blocks (n = 1–6);
- a partial group in epoch 0, with tag `0x8000_0000`;
- a partial group in epochs 1, 2, 5 and 15, with tags `0x8100_0000` … `0x8F00_0000`.

All of them are byte-exact, and the counted build asserts each one's tag epoch.

**On the unrepaired design these tests fail** (§4.4): with v2's `== NO_RAW` dispatch, the
very first one-row block panics, its tag read as a RAW index.

## 3. The proofs (release mode: what the uncounted build relies on)

### 3.1 Initialisation, bounds and pointer validity of the leaf

1. **The leaf is entered only for a nonempty portion of a descriptor opened since its last
   close.**
   - `finish` is called only from `close`, and only when `write ≠ w0`.
   - `w0`, `write` and `end` are set only by `open`, and nulled together by `close` and at
     initialisation.
   - An append happens only while `write ≠ end`:
     - `one` calls `refill`, which opens, when `write == end`;
     - `fill` is called with `m ≤ end − write`, and only when that is positive.
   - So `write ≠ w0` implies that an `open` happened since the last `close`, and that `open`
     wrote `raw`. `c < 256`, as a `u8` parameter, so `desc[c]` needs no bounds test.
2. **`raw == active_tag` implies an ordinary candidate.** `active_tag` is always
   `epoch_tag(e)` with bit 31 set, once the first snapshot is taken, and `open` is never
   reached before it. `open`'s RAW branch writes `h & M20 < 2²⁰`.
3. **`write − w0 == 60` bytes implies 30 initialised fields from the record's field 0.**
   - `open` sets `end = w0 + (re − r)` with `re ≤ 30(g + 1)`, so `end − w0 ≤ 30 − j0`, where
     `j0 = r − 30g`.
   - Appends never pass `end`, so 30 fields implies `j0 = 0` and `write == end`.
   - Each append writes the field at `write` and then advances it, so `[w0, w0 + 30)` were all
     written.
   - `w0 = &rec[g].f[0]` with `g < g_count`: `field(r)` requires `r < lim ≤ n ≤ 30·g_count`.
4. **The header at `w0 + 60` bytes is `rec[g].h`,** inside record g (`Rec` is `repr(C)`, 64
   bytes, the header at 60).
   - Alignment: the arena's records are 64-aligned (`align(64)`), so `w0` is 64-aligned and
     `w0 + 60` 4-aligned for the `u32` store. The `[u16; 30]` read and write are at 2-aligned
     `w0`.
5. **Provenance.**
   - `w0` comes from `rec.as_mut_ptr()` (`W::dst`) by `add` and `addr_of_mut!`, raw
     throughout, with no reference created. The byte offset to the header stays in the same
     allocation.
   - `rec` is reserved before `scatter` and never reallocated during it; `raw`'s pushes are a
     separate allocation.
6. **The bytes it touches:** the header (one `u32` store at +60) and bytes 0–59 (one read
   and one write each). Nothing past the record.
7. **Publication is v2's:** temporary labels are never visible, because `set_len` happens
   only after the final sweep has finished every portion.
8. **The header value is v2's.**
   - u0 < 2¹⁶, so `u0 << 8 < 2²⁴`; `tag & 0x7fff_ffff = e << 24` with e ≤ 15; c < 2⁸. The
     three fields are disjoint.
   - So the header is `(65536e + u0) << 8 | c = y0 << 8 | c`, v2's header for h0 = e. y0 <
     2²⁰ for n ≤ 983,010, so nothing overflows.
   - The fields are `f_j − u0 mod 2¹⁶`, v2's.

### 3.2 The opening epoch is the epoch of the portion's first appended row

- **Epoch index = snapshot index.**
  - `scatter`'s outer loop pushes exactly one snapshot per source epoch, at its head, before
    any row i of that epoch (`end = min(i + 61,440, n)`).
  - Every inner path consumes rows only up to `end`: tiles, coalesced runs bounded by
    `end`, and the tail tile.
  - So during epoch e, `snaps.len() − 1 = e`, and `active_tag = epoch_tag(e)` from the moment
    epoch e is entered. It is never derived from `W.y`, which at an exact boundary already
    names the next epoch.
- **Only two routes open a descriptor:**
  - **refill → one:** `one` calls `refill` only when `write == end`; `refill` closes, opens
    and returns true; `one` then appends at once. No snapshot runs between them, since
    snapshots happen only at the outer loop's head.
  - **run → open → fill:** `run` opens at its loop's bottom only with m > 0 rows remaining:
    it returns when m reaches 0, after a fill or after `complete`. The next iteration has
    `room = re − r > 0`, so `k = min(room, m) ≥ 1`, and `fill` appends. A coalesced run lies
    within one epoch (runs are split at epoch boundaries).
  - **Nowhere else:** the final sweep and the rank check never open. Nothing opens eagerly.
- **At EOF** the sweep only closes, so the active epoch is the last one, and every portion
  keeps its opening tag.
- **Agreement with v2's h(r).** The first row (rank r0) is received in epoch e, after
  snapshot `A_e` and before `A_{e+1}`, so `A_e[c] ≤ r0 < A_{e'}[c]` for every e' > e. So e =
  max{e : A_e[c] ≤ r0} = h(r0), v2's epoch of r0.
- **Checked dynamically as well** (not as a substitute): the counted build asserts it on
  every slow ordinary finish, and on every leaf asserts that v2's snapshot test would also
  call the portion fast. That held across every suite (§4).

### 3.3 `tag == active_tag` on a finished portion implies one epoch, so the span is ≤ 65,533 and needs no test

- `active_tag` only advances. Equality at finish therefore means no epoch boundary passed
  between the opening and the finish.
- Every row of the portion was appended between those two moments, so all its rows were
  received in epoch e. Their labels are φ(i) = 65536e + φ(i mod 61,440), with u = φ(i mod
  61,440) in [0, 65,533], strictly increasing with i.
- The portion's ranks follow source order, so u0 ≤ … ≤ ul, and the span yl − y0 = ul − u0 ≤
  65,533. The group is ordinary.
- v2 reaches the same classification for the same portion: its fast test holds (§3.2) and
  its span test passes. So the leaf writes exactly v2's bytes.

## 4. Correctness evidence

The binary is built with rustc 1.97.0 (`2d8144b78`, LLVM 22.1.6), `--locked --release`,
`CARGO_ENCODED_RUSTFLAGS=` empty; sha256 `734915e3…5d622f663`.

**Every case runs the following,** in v2's `check_block`, adapted:
1. the oracle `C30::build`;
2. Ctor3's counted build into a poisoned, reused arena, compared byte for byte (`rec` and
   `raw`);
3. **v2's counted build on the same input** (`ctor2.rs`, read-only), with every v2 counter
   compared field by field:
   - the scalar, fragment and bulk rows, runs, runs split at epochs and complete groups;
   - the finish classes and normalisations;
   - the RAW fields and entries, fields restored and epoch searches;
   - epochs, the RAW groups' moves and cycles, the spans, multi-epoch portions, and all 930
     portion-shape cells.

   Plus four identities:
   - leaf finishes = v2-eligible finishes (v2 fast, 30 fields);
   - v2's fast = leaf + same-epoch partial;
   - every finish is leaf or slow;
   - the slow classes sum to the slow entries;
4. the uncounted build, re-poisoned with the other pattern, compared again;
5. the unchanged walker on its arrays, counted and audited: its output against production's
   and the oracle's own counted walk, with all transition counts;
6. **the arrays compared again after each walk** (marker restoration).

| suite | cases | array comparisons | walk outputs | restorations | result |
|---|--:|--:|--:|--:|---|
| 11 corpora (sha256 all equal to `2026-11-02-compact30-census/corpora.sha256`) × 4 configurations (permille 0, 0 with extra starts, 50, 1000) | 44 (6,052 blocks) | 12,104 | 18,156 | 12,104 | **exact** |
| the adversarial set, 25 inputs × levels 1, 9 × 4 configurations | 200 | 1,056 | 1,584 | 1,056 | **exact** |
| synthetic, full (v2's suite and two new classes) | 53,960 | 99,840 | 149,760 | 99,840 | **exact** |
| synthetic, quick | 542 | 1,008 | 1,512 | 1,008 | **exact** |
| synthetic, targeted Miri selection | 134 | 262 | 393 | 262 | **exact, no UB reported** (§4.2) |

**The targeted cases** are all in the full synthetic run, all exact:
- **The tag/RAW dispatch counterexample:**
  - one-row and tiny blocks, n = 1–6, σ 1 and 2;
  - partial ordinary groups of 1, 7 and 29 rows in epochs 1, 2, 5 and 15, each asserted to
    reach `finish_slow` as same-epoch partial.
- **Epoch boundaries:** v2's class, k ∈ {1, 2, 3, 5, 14, 15}, natural and forced.
- **Portions finished in a later epoch than they opened** (new): full groups wholly before a
  boundary, finished at the next arrival after it, or at a later epoch's end; and 29 + 1
  across a boundary. Each is asserted to take the old-epoch slow path.
- **Absent bytes across epochs:** v2's repeated-snapshot class, gaps 1–13.
- **The span cases:** 3 → 61,440 (span 65,533, ordinary) and 2 → 61,440 (65,534, RAW),
  asserted; spans 65,530..=65,538 at nine first positions; and far-wide groups.
- **Forced RAW** at permille 50 and 1000 (corpora, adversarial, synthetic classes 4 and 8, and
  the 900,000- and 983,010-row blocks).
- **Crossing groups:** a bucket boundary at every offset.
- **Partial last groups:** the pending and tag classes.
- **Malformed counts:** 3,995 rejected, 7 accepted exactly when the counts equal L's own;
  rejected builds leave empty arrays, and n = 983,011 and 2²⁴ are rejected.
- **Reuse,** growing and shrinking: 900,000 → 1 → 899,990 → 0 → … → 899,999, unpoisoned then
  poisoned. **n = 0 goes through both builds** in the small and repeated classes.

### 4.1 The leaf's coverage, against v2

**On the primary corpus** (`wikidata-20261026.bz2`, permille 0, 434 blocks):

| counter | v3 | v2 (its own counted build) |
|---|--:|--:|
| leaf finishes (tag = active, 30 fields) | **1,647,375** | — |
| v2 fast finishes with 30 fields (the eligible cases), recorded independently | **1,647,375** | |
| v2's `fin_fast` (equal field by field) | 1,647,416 | 1,647,416 |
| the difference: v2-fast partial groups, now in `finish_slow` | 41 | |
| slow entries: old-epoch ordinary (non-fast ordinary 75,997 + wide 30,949) | 106,946 | |
| slow entries: same-epoch partial | 41 | |
| slow entries: preset RAW (crossing 41,120; forced 0) | 41,120 | |
| ordinary openings (each one tag load) | **1,754,362** | |
| preset-RAW openings | 41,120 | |
| epochs (tag set per epoch) | 3,654 | 3,654 |

The leaf count, 1,647,375, lies inside the audit's bound of 1,647,374–1,647,407. The
ordinary openings equal the audit's **1,754,362 extra reads exactly**.

**On every corpus, configuration and synthetic case,** the per-case identities in §4 held,
including leaf = v2-eligible. The suite totals (natural configuration):

| | leaf | v2-eligible | `fin_fast` | same-epoch partial | old-epoch | preset RAW |
|---|--:|--:|--:|--:|--:|--:|
| 11 corpora | 6,427,040 | 6,427,040 | 6,427,260 | 220 | 437,899 | 171,508 |
| full synthetic | 2,655,214 | 2,655,214 | 2,701,123 | 45,909 | 22,015 | 477,853 |

### 4.2 Miri

**Complete:** every selected case ran, with no timeout, and nothing was cut off. Every check
was exact, and **no undefined behaviour was reported**. The run exited 0 (`miri.console`).
- **The command:** `cargo +nightly miri run --example c30walk -- ctor3-synthetic --miri`, on
  nightly rustc 1.101.0 (`16d1ffac1`) with Stacked Borrows and unpoisoned arenas, after the
  first report was sent.
- **The 134 cases** (`synthetic_miri`), each running Ctor3's counted and uncounted builds,
  v2's counted build with the counter comparison, the three walks and the restoration
  checks:

  | class | cases | completed |
  |---|--:|---|
  | small blocks, n 0..=40, σ ∈ {1, 2, 3} (n = 0 through both builds, so with no array comparison), plus four forced one-symbol blocks at permille 1000 (n = 1, 2, 31, 61) | 127 | all, exact |
  | the tag/RAW dispatch, tiny (n = 1–4) | 4 | all, exact |
  | the exact span cases, 3 → 61,440 (65,533, ordinary) and 2 → 61,440 (65,534, RAW, `fin_wide` = 1 asserted); 61,640 rows each | 2 | both, exact |
  | the tag/RAW dispatch in epoch 1 (a one-row partial group, tag `0x8100_0000`; 61,741 rows) | 1 | exact |

  (The selection was described as 137 cases while running; that was a miscount. It is 127 +
  4 + 2 + 1 = 134.)
- **Coverage of v3's own paths under Miri,** from the run's counters:
  - **the leaf:** 6,162 finishes;
  - **the slow finish:** 189 entries. Of these, 105 were same-epoch partial (the tag
    dispatch, epochs 0 and 1, including the one-row blocks), 8 old-epoch ordinary (the span
    cases' groups opened in epoch 0 and finished in epoch 1, one of them wide), and 76 preset
    RAW (crossing and forced).

**Against v2's focused Miri coverage** (473 cases in five classes: small blocks n 0..=64,
bucket boundaries at n = 61, bulk-to-partial transitions, epoch boundaries and absent
characters; it stopped in the span class):

| v3-specific path | reached under Miri |
|---|---|
| the tag dispatch (a tagged ordinary portion in `finish_slow`) | yes: 105 entries, tags `0x8000_0000` and `0x8100_0000` |
| tagged one-row blocks | yes: n = 1 in the small and tiny classes |
| a portion finished in a later epoch than it opened | yes: 8 entries, in the span cases |
| the fast-path boundary (exactly 30 fields: leaf; 29 or fewer: slow) | yes: 6,162 leaf finishes and 105 partial |
| the exact 65,533/65,534 span cases | yes, both (v2's Miri didn't reach them) |

**What this run didn't cover,** all of it v2 code that v3 doesn't change, covered by v2's Miri
run and by v3's release suites:
- bucket boundaries at every offset;
- bulk-to-partial transitions;
- multi-epoch blocks beyond epoch 1;
- forced RAW at permille 50;
- malformed counts;
- large reuse;
- poisoned arenas.

No gap in v3's own paths remains, so no further targeted run was made.

### 4.3 What the counted build asserts on top (each held everywhere)

- **At each leaf finish:**
  - v2's snapshot test would call the portion fast;
  - the tag's epoch is the last snapshot's;
  - `write == end`, and `r0` is group-aligned;
  - the span is ≤ 65,533.
- **At each slow ordinary finish:** the tag's epoch equals h(r0), and a same-epoch portion is
  partial.
- **At each slow RAW finish:** `raw < raw.len()`.

### 4.4 The test fails on the unrepaired design

**Done** (`mutation.diff`, `mutation.console`):
1. The slow finish's two `d.raw & ORD != 0` tests (the counted-build classification and the
   dispatch) were replaced with v2's `d.raw == u32::MAX`, keeping the tags.
2. The source was rebuilt with the same toolchain and settings.
3. `ctor3-synthetic --quick` was run.
4. The change was reverted with `git checkout`.

The mutant **panicked on the first portion it finished** with "assertion failed: `(d.raw as
usize) < self.raw.len()`" at `ctor3.rs:579`, in `finish_slow<true>`, under `check_block`. It
exited 101.
- The run's first cases are n = 0, which finishes nothing, and then n = 1. So the failure is
  the one-row block's single portion: tag `0x8000_0000`, read as a RAW index.
- Without that counted-build assertion, `write_entries` would index `raw[0x8000_0000]` and
  panic on the bounds check.
- The repaired source passes the same suite (`synthetic-quick.console`).

## 5. The compiled code (static observations only)

Listings are in `listings/`:
- `ctor3-build-false.asm`: `<c30walk::ctor3::Ctor3>::build::<false>`, 0x1904 bytes, 1,432
  instructions;
- `ctor3-finish-false.asm`: the leaf `W<false>::finish`, 0xbc, 44;
- `ctor3-finish-slow-false.asm`: `W<false>::finish_slow`, 0x5f8, 378;
- `ctor3-refill-false.asm`: `W<false>::refill`, 0xed, 67.

v2's functions are in the same binary for comparison.

### 5.1 The leaf, annotated (attempt 1, the pinned source)

```
81090 movzbl %sil,%esi              ; c (u8 argument)                         1
81094 lea    (%rsi,%rsi,4),%rax     ; 5c                                      2
81098 mov    0x64(%rdi,%rax,8),%ecx ; R  desc[c].raw (tag)                    3
8109c cmp    0x284c(%rdi),%ecx      ; R  == W.active_tag ?                    4
810a2 jne    finish_slow            ; tail jump: old epoch, preset RAW        5
810a8 lea    (%rdi,%rax,8),%rax     ;                                         6
810ac add    $0x40,%rax             ; &desc[c]                                7
810b0 mov    (%rax),%rdx            ; R  write                                8
810b3 mov    0x10(%rax),%rax        ; R  w0                                   9
810b7 sub    %rax,%rdx              ;                                        10
810ba cmp    $0x3c,%rdx             ; 60 bytes = 30 fields ?                 11
810be jne    finish_slow            ; tail jump: partial                     12
810c4 and    $0x7fffffff,%ecx       ; e << 24                                13
810ca or     %esi,%ecx              ; | c                                    14
810cc movq   (%rax),%xmm0           ; R  bytes 0–7                           15
810d0 movzwl (%rax),%edx            ; R  u0                                  16
810d3 movd   %edx,%xmm1             ;                                        17
810d7 shl    $0x8,%edx              ;                                        18
810da or     %ecx,%edx              ; header                                 19
810dc mov    %edx,0x3c(%rax)        ; W  header at +60                       20
810df movq   0x8(%rax),%xmm2        ; R  bytes 8–15                          21
810e4 xor    %ecx,%ecx              ;  \                                     22
810e6 pshuflw $0x0,%xmm0,%xmm3      ;   |                                    23
810eb pinsrw $0x0,%ecx,%xmm0        ;   | chunk 0: LLVM knows f0 − u0 = 0    24
810f0 punpcklqdq %xmm2,%xmm0        ;   | and builds that lane: two loads,   25
810f4 pshufd $0x44,%xmm3,%xmm2      ;   | a zero insert, a masked broadcast  26
810f9 pand   -0x732b1(%rip),%xmm2   ; R | (a constant load)                  27
81101 psubw  %xmm2,%xmm0            ;   |                                    28
81105 movdqu %xmm0,(%rax)           ; W /  bytes 0–15                        29
81109 pshuflw $0x0,%xmm1,%xmm0      ; a second broadcast of u0               30
8110e pshufd $0x44,%xmm0,%xmm1      ;                                        31
81113 movdqu 0x10(%rax),%xmm2       ; R  bytes 16–31                         32
81118 movdqu 0x20(%rax),%xmm3       ; R  bytes 32–47                         33
8111d psubw  %xmm1,%xmm2            ;                                        34
81121 movdqu %xmm2,0x10(%rax)       ; W                                      35
81126 psubw  %xmm1,%xmm3            ;                                        36
8112a movdqu %xmm3,0x20(%rax)       ; W                                      37
8112f movq   0x30(%rax),%xmm1       ; R  bytes 48–55                         38
81134 psubw  %xmm0,%xmm1            ;                                        39
81138 movq   %xmm1,0x30(%rax)       ; W                                      40
8113d movd   0x38(%rax),%xmm1       ; R  bytes 56–59                         41
81142 psubw  %xmm0,%xmm1            ;                                        42
81146 movd   %xmm1,0x38(%rax)       ; W                                      43
8114b ret                           ;                                        44
```

**Counts:**
- **44 instructions** from entry to `ret`.
- **12 explicit reads** (lines 3, 4, 8, 9, 15, 16, 21, 27, 32, 33, 38 and 41):
  - the tag, `active_tag`, `write` and `w0`;
  - u0;
  - six field loads: two 8-byte loads for chunk 0, then 16, 16, 8 and 4 bytes;
  - one `.rodata` mask constant.
- **6 explicit writes:** the header, then 16, 16, 16, 8 and 4 bytes.
- **No push, pop, spill or stack access.** Both exits are direct tail jumps (`jne
  finish_slow`), so no frame is needed on either route.
- **No snapshot access and no span test.** Bytes 0–59 are each written once and the header
  once; nothing is read or written past byte 63 of the record. The constant load is from
  `.rodata`.

**The whole fast-success boundary, by caller.** The three call sites are identical, instruction
for instruction, to v2's:

| route | caller-side work before the call | call | leaf incl. ret | boundary |
|---|---|--:|--:|--:|
| scalar (`refill` → `close`) | `mov %rbx,%rdi` | 1 | 44 | **46** |
| run (`close` in `run`) | `lea 0x98(%rsp),%rdi`, `mov 0x90(%rsp),%rsi` | 1 | 44 | **47** |
| final sweep | `mov %r15,%rdi`, `mov %r14d,%esi`, `mov %r8,%rbx` | 1 | 44 | **48** |

`close`'s own nonempty test, and its reloads and rank update after the call, are v2's,
unchanged, and outside the finish in both versions.

**Against the registered convention** (the audit's `report.txt` line 281, as the guardian
made it precise):

| figure | target | compiled |
|---|--:|--:|
| the fast function's body | ≤ 40 | **44, missed** |
| the body with its unchanged call | ≤ 41 | **45, missed** |
| new caller-side work, charged on top | — | 0 (the call sites equal v2's) |
| the whole boundary with the existing argument setup | — | 46–48 |

The five-chunk normaliser is kept: five stores of 16, 16, 16, 8 and 4 bytes, none
overlapping. The four-chunk overlapping variant is out of scope.

**Where the excess comes from:**
- **Chunk 0** (lines 15, 21–29): u0 is loaded from field 0, and the header store at +60
  provably can't alias the fields. So LLVM knows lane 0's result is 0 and materialises it
  with two 8-byte loads, a zero insert, a masked second broadcast and a constant load. That
  is 9 instructions and 4 reads where v2's chunk was 3 instructions and 1 read, plus a second
  broadcast (lines 30–31).
- **The guards:** they take 12 instructions where report 227's illustration had 9. That is
  the `movzbl` of the `u8` argument, and `lea` + `add` to form `&desc[c]`.

### 5.2 The rest of the timed path against v2 (same binary)

- **The per-row loops are instruction-for-instruction unchanged** (normalised comparison of
  `build::<false>`):

  | range (v2 prototype addresses) | instructions | equal | v3 offset |
  |---|--:|---|---|
  | scalar append, part 1 (`34350`–`3437a`) | 12 | yes | +0x1270–0x129a |
  | scalar append, part 2 (`343e0`–`34406`) | 12 | yes | +0x1300–0x1326 |
  | fragment fill, scalar (`33d00`–`33d17`) | 7 | yes | +0xc20 |
  | fragment fill, vector (`34180`–`341a5`) | 9 | yes | +0x10a0 |
  | complete-record emitter (`341d0`–`34233`) | 20 | yes | +0x10f0 |
  | its odd tail (`3423a`–`34269`) | 11 | yes | +0x115a |
  | uniformity scan (`33ae3`–`33b27`) | 19 | yes | +0xa03 |
  | run scan (`33b30`–`33b6d`) | 16 | yes | +0xa50 |
  | tile advance (`34470`–`344bc`, once a tile) | 17 | same instructions, one stack slot renumbered (`0x8(%rsp)` → `0x10(%rsp)`) | +0x1390 |

  No per-row work was added: there is no tag maintenance on the scalar or fragment paths.
- **What `build::<false>` adds (1,432 instructions against v2's 1,431):**
  - at the snapshot, `shl $0x18`, `add $0x7f000000`, `or $0x80000000` and a store of
    `active_tag`: **4 instructions once per epoch**;
  - in `run`'s `open`, `mov $0xffffffff,%edi` becomes `mov 0x28e4(%rsp),%edi`: **one for
    one, plus a read**;
  - otherwise register and stack-slot renaming (equal counts, apart from two moves
    traded between sites).
- **`refill`:** 67 instructions in both. Its `open` turns the immediate into
  `mov 0x284c(%rbx),%edi`, one for one, plus a read.
- **`finish_slow`** is v2's `finish`, 378 instructions against 379:
  - `cmp $0xffffffff; je` became `test; js` (the high-bit test);
  - the tail block was reordered (one `jmp` fewer).

  Every slow class runs v2's path, after the leaf's guard prefix: 5 instructions to the tag
  exit (old epoch, preset RAW), 12 to the length exit (same-epoch partial).

### 5.3 The one iteration (recorded; not kept)

- **Attempt 2** read u0 with `read_volatile`, to hide its equality with field 0
  (`attempt-2/`). LLVM still assumed it for lane 0 and gave **45 instructions**: an unaligned
  4-byte load at +2, `psllq`, `pinsrw` and `punpcklqdq` for chunk 0, and two broadcasts.
- The pinned source is attempt 1's (`attempt-1/` holds its leaf listing).
- **No third variant was tried.** An explicit SSE2-intrinsic expression, or deriving u0 from
  the loaded first chunk, are the obvious candidates. Whether either compiles to ≤ 40 is
  unknown and untested.

## 6. The revised ledger (compiled paths × observed primary-corpus frequencies)

Instructions are architectural instructions on the paths named; nothing is converted to time.

| item | per event | events | instructions |
|---|--:|--:|--:|
| v2's fast full finish (report 227's 81, excluding the call) | 81 | 1,647,375 | 133,437,375 |
| v3's leaf (including `ret`; call sites unchanged) | 44 | 1,647,375 | 72,484,500 |
| **gross saving** | **37** | 1,647,375 | **60,952,875** |
| slow entries now pass the leaf's tag exit first | +5 | 148,066 | −740,330 |
| same-epoch partials pass both guards first | +12 | 41 | −492 |
| `finish_slow`'s own path against v2's `finish` | 0 (±1, the reordered tail) | 148,107 | 0 (±148,107) |
| `active_tag` set per epoch | +4 | 3,654 | −14,616 |
| tag loads at ordinary openings | 0 (one for one) | 1,754,362 | 0 |
| **net saving** | | | **≈ 60.197 M (60,197,437 ± 148,107)** |

**In the audit's own terms** (`report.txt`, lines 290–295): epoch saving = (81 − b)F −
d(P − F) − aO − B.
- F = 1,647,375 eligible finishes, observed, inside its bound of 1,647,374–1,647,407.
- P − F = 148,107 other portions; P = 1,795,482 is the audit's figure, and this run's slow
  entries are exactly 148,107.
- b = 44, the body convention, the entry call excluded as in its 81.
- d = 5 per tag exit and 12 per length exit, mean 5.0006.
- a = 0, O = 1,754,362.
- B = 14,616.

That gives the 60,197,437 above. The audit's target assumed b ≤ 40 (41 saved per event),
d ≤ 10 and B ≤ 20,000. Here d and B are within their caps, and b is 4 over.

**Against report 227's conditional 66.041 M:** the compiled leaf saves 37 a finish, not 41,
which is about **5.8 M less**. The boundary target of ≤ 40 isn't met on either convention: the
body is 44, the whole boundary 46–48. The 66.041 M is not credited.

**Memory operations on fast success,** v2 against v3:

| | v2 (report 227) | v3 |
|---|--:|--:|
| explicit reads | 17 | 12 |
| explicit writes | 8 | 6 |
| push / pop | 6 / 6 | 0 / 0 |

Corpus totals (primary):
- **reads:** −5 × 1,647,375 + 1,754,362 (opening tag loads) + 2 × 148,107 (the slow
  entries' extra tag and `active_tag` reads) + 2 × 41 ≈ **−6.19 M**;
- **writes:** −2 × 1,647,375 ≈ **−3.29 M**;
- **stack operations:** −12 × 1,647,375 ≈ **−19.77 M**. These are push and pop
  instructions, already inside the instruction saving, not a further credit.

**The structural targets, as registered (report 227 §F, report 236):**

| target | result |
|---|---|
| fast body ≤ 40 (≤ 41 with its call), new caller work charged on top | **missed: body 44, with call 45**; no new caller work; whole boundary 46–48 |
| ≤ 10 explicit reads | **missed: 12** |
| ≤ 6 explicit writes | met: 6 |
| no snapshot chain, no span test | met |
| no register-save frame or spills | met |
| no added per-row work (scalar, fragment) | met: loops unchanged |
| opening one for one | met: instruction-neutral, +1 read per ordinary opening (1,754,362) |
| slow dispatch and setup within allowance (1.481 M + 0.020 M) | met: 0.741 M + 0.015 M |
| byte-exact output, coverage unchanged | met |
| no new table-sized workspace | met: one `u32` in `W` |

## 7. Limitations

- **Static counts only.** Nothing here says how fast either finish runs. Report 235's solved
  requirement (about 8.1 ns of effective saving per eligible finish) is untouched by this
  work.
- **The compilation result is for this exact expression and toolchain.** A differently
  written normaliser might meet the target; that is a further variant, and not authorised by
  the stop rule.
- **Miri's coverage is a selection** (§4.2), not the full suites. Release tests are not a
  sanitiser.
- **x86-64 and rustc 1.97.0 only.**
- **The counted build's assertions** check the proofs on the inputs run; they are not the
  proofs (§3 is).
- **Clippy wasn't run** (not installed for 1.97.0). `ctor3.rs` is rustfmt-clean.

## 8. Files

- `examples/c30walk/ctor3.rs`; `examples/c30walk.rs` (the `mod` line and dispatch).
- This folder:
  - `listings/`;
  - `attempt-1/` and `attempt-2/` (the leaf listings and the iteration's source diff);
  - `corpora.console` and `.jsonl`, `adversarial.console` and `.jsonl`, `synthetic.console`,
    `synthetic-quick.console`, `miri.console`, `mutation.diff`, `mutation.console`;
  - `rebuild/`: every suite's console from the rebuilt pinned source (§9.3);
  - `corpora.sha256`.

## 9. The checks completed after the first report

### 9.1 Miri

Complete, exact, and no undefined behaviour reported: 134 cases (§4.2).

### 9.2 The mutation check

v2's `== NO_RAW` dispatch fails on the first one-row block (§4.4).

### 9.3 The rebuild of the formatted source

The pinned source at `dbf5139` was rebuilt with rustc 1.97.0, `--locked --release` and empty
rustflags. The binary's sha256 is `4deb4234…3e1cc7e`; it differs from the first build's by
line tables only.
- **Listings:** `Ctor3::build::<false>`, the leaf `finish`, `finish_slow` and `refill` are
  **identical instruction for instruction, at the same addresses** (0x3aed0, 0x81090,
  0x80a90, 0x81150), as the archived listings.
- **Suites:** the corpora, adversarial, full synthetic and quick synthetic runs, rerun on it,
  give **byte-identical** records (`corpora.jsonl`, `adversarial.jsonl`) and consoles. The
  only difference is the runner's appended `exit 0` line in the archived copies
  (`rebuild/`).

So every figure in §4–§6 holds for the pinned source.
