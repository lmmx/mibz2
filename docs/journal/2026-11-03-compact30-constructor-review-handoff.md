# Compact30: the direct constructor, a handoff for independent code review (for approval)

**To:** the researcher. **Authorised by:** report 190, task 1
([2026-11-03-compact30-constructor-checkpoint.md](2026-11-03-compact30-constructor-checkpoint.md)).
**Status:** a draft for the guardian's approval. It asks for review only. No new
implementation, timing or benchmarking is authorised by it.

This asks you to audit the direct compact30 constructor's code. I wrote it, so this note
doesn't argue for it. It lists what to check, where the code is, what the existing tests do
and don't cover, and what I'd like back.

## 1. The audit surface

- **The code:** `examples/c30walk/ctor.rs`, 1,963 lines, at **`e003a32`** on
  `research/compact30-constructor`. It is unchanged since its implementation commit `446a3b2`
  (`git diff 446a3b2 e003a32 -- examples/` is empty).
- **The wiring:** `examples/c30walk.rs` gains 13 lines (`git diff ca7781e e003a32 --
  examples/c30walk.rs`): usage comments, `#[path = "c30walk/ctor.rs"] mod ctor;`, and a
  three-line dispatch of the `ctor-` modes in `main`. The walker, `Rec`, `RawG`, `C30::build`,
  `phi`, `mix` and the constants are the parent's, used unchanged.
- **The module's two halves:**
  - **The constructor:** lines 1–904, the part under review for memory safety and
    equivalence.
  - **The correctness harness:** lines 906–1963. Review it for blind spots, not for memory
    safety.
- **The journal** ([2026-11-03-compact30-constructor-prototype.md](2026-11-03-compact30-constructor-prototype.md))
  states the design, the results and an informal initialisation proof. Treat it as the claim
  to be checked against the code, not as evidence.
- **The spec:** report 188 (Astra's design) in
  [2026-11-03-compact30-constructor-design.md](2026-11-03-compact30-constructor-design.md).
  The frozen representation and oracle are described in report 186,
  [2026-11-02-compact30-constructor-brief.md](2026-11-02-compact30-constructor-brief.md).

## 2. A map of the constructor (line numbers at `e003a32`)

| lines | item | role |
|---|---|---|
| 51 | `MAX_N` | 983,010: the largest n whose PARK label fits 20 bits |
| 55–70 | `PATTERNS`, `patterns()` | 30 records, `f[j] = j + 2·[p + j ≥ 30]`, header 0 |
| 72–74 | `forced` | the oracle's forced-RAW predicate, copied expression for expression from `C30::build` |
| 76–78 | `rd` | an 8-byte native-endian load, as production's `read_u64` |
| 292–296 | `Ctor` | reusable scratch: the RAW bitmap and the counters |
| 297–302 | `chk!` | `assert!` in `build::<true>`, `debug_assert!` in `build::<false>` |
| 304–318 | `W` | one build's state: `dst: *mut Rec`, `g_count`, `raw`, `bitmap`, cursors `x`/`xl`, source `y`/`p` |
| 321–324 | `W::is_raw` | bitmap test (bounds-checked index) |
| 326–345 | `W::header`, `set_header`, `fields` | raw-pointer accesses into the record arena |
| 347–358 | `W::advance` | source label/phase advance by k ≤ 30 rows |
| 360–375 | `W::create_raw` | push a zeroed `RawG`, write the whole record as RAW, set the bit |
| 377–414 | `W::promote` | ordinary group → RAW after j written fields |
| 416–477 | `W::fragment` | k rows at slots j..j+k−1 of one group |
| 479–499 | `W::complete` | `whole` complete groups from a pattern (natural configuration) |
| 501–529 | `W::complete_checked` | the same with a per-group RAW test (forced-RAW configurations) |
| 531–595 | `W::run` | one coalesced run: limit check, prefix, complete groups, suffix |
| 597–648 | `W::one` | the one-row path |
| 650–702 | `canonicalise` | RAW groups into increasing-group order in place |
| 704–826 | `Ctor::build` | validation, allocation, the call to `scatter`, finishing, `set_len` |
| 828–903 | `Ctor::scatter` | crossing groups, forced groups, the scan of L |

## 3. The focus areas

For each, the claim, where it lives, and what I'd check. These are prompts, not a complete
list.

### 3.1 The unsafe record arena and `set_len`

- **Where:** `Ctor::build` 719–760 (clear, capacity, `dst`), 780–803 (the finishing writes
  and the only `set_len`, line 802); every write through `dst` in `W` (326–345, 360–375,
  377–414, 416–477, 479–529, 597–648); `canonicalise` 660–702 (header rewrites).
- **The claim:** `rec` keeps length 0 until line 802. By then every one of its G + 1 records
  has all 30 fields and its header written in this build, and nothing was read before it was
  written. On any rejection the length stays 0. The journal's "Memory safety" section gives
  the argument:
  - a group whose bit is set was fully written when the bit was set (`create_raw`,
    `promote`);
  - a group whose bit is clear lies in one bucket by the counts; the final `x == xl` check
    (line 902) proves every slot below n was written once; the slot-0 write set the header;
    the last group's unused fields are filled at 784–790; PARK at 791–794.
- **Worth checking:**
  - that `x == xl` really implies every slot below n was written exactly once. That needs
    each cursor to advance by exactly one slot per row written, and the buckets to tile
    [0, n) in encoded space;
  - that no path writes a field of a group whose bit is set, other than the same 0xFFFF,
    including `complete` in the natural configuration (the claim: a complete group can't be
    crossing, forced or already promoted);
  - that every header read (`W::header`, lines 430, 441, 613, 620; `canonicalise` 668) is of
    a header written earlier in the same build, **for any counts, including counts that don't
    match L**. The claim rests on "non-crossing ⇒ the bucket starts at or before 30g ⇒ its
    cursor wrote slot 0 first";
  - that `promote` reads only fields 0..j−1 of g (395–397), all already written;
  - the raw-pointer provenance: `dst` is taken from `rec.as_mut_ptr()` at 762 and again at
    782, with `raw` and `bitmap` borrowed mutably alongside. Miri (Stacked Borrows) passed on
    the quick set, which doesn't cover every path (§4).

### 3.2 Release-mode bounds assumptions

- **Where `build::<false>` drops checks:** every `chk!` is a `debug_assert!` there:
  - `g < g_count` in `header`, `set_header`, `fields` (329, 336, 343);
  - `g < g_count && !is_raw(g)` in `create_raw` (363) and `promote` (385);
  - `1 ≤ j < 30` in `promote` (384);
  - `g < g_count && j < 30 && k ≥ 1 && j + k ≤ 30` in `fragment` (422);
  - `g + whole ≤ g_count` in `complete` and `complete_checked` (484, 504);
  - `h as u8 == c` in `fragment` and `one` (445, 624).

  The finishing writes (784–794) are unchecked in both builds.
- **What should guarantee each index:**
  - **record indices:** the run's `x_end ≤ xl[c]` (`run`, line 543) and the row's
    `x < xl[c]` (`one`, 603), checked before any write in both builds, with xl ≤ φ(n), so
    every touched group is at most G − 1;
  - **crossing groups:** B / 30 with B < n (`scatter` 837–852); forced groups: g < G;
  - **slots:** cursors skip labels 30 and 31 (`one`'s `+2·[x & 31 = 29]`; `run`'s `+2q`), so
    j < 30. In `run`, `j + k ≤ 30` follows from the prefix/suffix split;
  - **RAW indices** come from headers (`& M20`) and index `raw` through safe, bounds-checked
    indexing (394, 464, 635; `canonicalise` 668–700). An inconsistency would panic, not
    corrupt.
- **Worth checking:**
  - the arithmetic behind `x_end` and the claim that `x_end ≤ xl[c]` bounds every group the
    run touches;
  - the u32 subtraction `y - (h >> 8)` (446, 625). It assumes labels rise within a bucket. If
    that failed in release, it would wrap. In `fragment` the following `d + pat[k − 1]` could
    then wrap too and pass the 65,533 check, so a wrong offset would be written. That would
    be wrong output, not an out-of-bounds write;
  - the u16 sum `o + d` in `fragment` (457), bounded by the endpoint check;
  - that `rd`'s `ll[i..i + 8]` slices stay in bounds at the loop ends (`scatter` 864–900).

### 3.3 RAW promotion after partially written groups

- **Where:** `W::promote` 377–414; its callers in `fragment` (the endpoint check, 446–452)
  and `one` (625–627).
- **The claim:**
  - promotion happens before the overflowing fragment or row is written;
  - it reconstructs exactly the j earlier fields as `(base + offset) << 8 | F` into a freshly
    pushed zeroed `RawG`, then rewrites the record as RAW, sets the bit, and returns the
    provisional index;
  - the fragment is then written as RAW entries (464–467);
  - wide status can be decided early, because labels rise within the group, so the last
    label bounds every offset.
- **Worth checking:**
  - j ≥ 1 always at a promotion (a group's first row can't overflow);
  - the base and F byte taken from the old header;
  - that no ordinary field is left behind in a promoted record;
  - that a promoted group is never promoted again or written as ordinary;
  - that the endpoint check (offset of the last row > 65,533) matches the oracle's wide test
    (`psi[e−1] − psi[a] > MAX_OFF`) exactly, including at 65,533 and 65,534.
- **Testing limit:** promotion is covered only by the release runs (counted-build assertions,
  poisoned arenas, oracle comparison). **Miri never exercised it** (§4).

### 3.4 In-place RAW canonicalisation

- **Where:** `canonicalise` 650–702, called at 795.
- **The claim:**
  - the bitmap, scanned in increasing g, assigns ranks;
  - each group's provisional index (from its header) locates its `RawG`; word 31 of that
    `RawG` receives the rank; the header is rewritten to `RAW_BIT | rank`;
  - after asserting one RAW group per set bit (675), each cycle is rotated with one carry
    `RawG` via `mem::swap`;
  - every word 31 is set back to zero (697–700);
  - word 30 is never written.
- **Worth checking:**
  - the assertion `rank == raw.len()` (675) is the only check that the provisional indices
    are a permutation. It counts set bits; it doesn't prove that no two headers carry the
    same provisional index. The `debug_assert_eq!(raw[old].0[31], 0)` at 669 would catch a
    duplicate only in debug builds. If two groups shared an index, the cycle loop could
    misplace groups or fail to terminate. The claim is that each `create_raw`/`promote`
    pushes exactly one `RawG` and gives its index to exactly one newly set bit;
  - that the cycle loop terminates and places every group, and that a group already in place
    (tag = index) is skipped;
  - that word 31 starts at zero for every RAW group (only slots < 30 are ever written);
  - that no `RawG` entry other than word 31 changes during the permutation.
- **Tests:** `canon_tests` (1796–1907) runs `canonicalise` alone on identity, reverse,
  rotation, pair swaps, random and one-full-cycle permutations, up to 1,200 RAW groups.
  These ran under Miri (the quick sizes) and in release.

### 3.5 Reuse after changing block sizes

- **Where:**
  - `Ctor::build` 719–758: `rec.clear()`/`raw.clear()`; reallocation only when capacity is
    short, the old allocation dropped first (`*v = Vec::new()`, then `reserve_exact`);
  - the bitmap is `clear()` then `resize(words, 0)`, so it is zeroed every build;
  - the RAW reserve: the natural bound `min(G, σ⌊φ(n−1)/65,534⌋ + σ − 1)`, or G in forced
    configurations (736–742).
- **The claim:** nothing from a previous block is read, because nothing is read before this
  build writes it. Stale records past the new G + 1 stay beyond the length.
- **Worth checking:**
  - that a shrinking block (900,000 rows → 1 row) never reads a stale header or bit;
  - that a reserve exceeded (if the bound were wrong) only reallocates, through `push`, and
    can't invalidate `dst` (it can't: `dst` points into `rec`, not `raw`);
  - that a build which returns false leaves `rec` at length 0 and `raw` empty (774–777),
    whatever it wrote into spare capacity.
- **Tests:**
  - the corpus and adversarial runs reuse one arena per worker thread across blocks of every
    size;
  - synthetic class 11 (1638–1662) runs a fixed sequence of sizes from 900,000 down to 0 and
    back, unpoisoned then poisoned, with the forced configuration changing the RAW reserve.

### 3.6 Exact oracle equivalence, and where the harness could be blind

- **What is compared** (`check_block` 1043–1133, `diff` 915–944, `classify` 946–976):
  - after each build (counted, then uncounted, into the same arena), both arrays against the
    oracle's: lengths, then every byte, via `bytes()` (910–913);
  - the groups' classes (crossing, wide, forced), derived from the candidate's arrays, against
    the oracle's `Build` counts;
  - then the walker on the candidate's arrays: counted and audited, against production's
    output and the oracle arrays' own counted walk (output and all 14 transition counts);
  - the arrays against the oracle's again after each walk.
- **Possible blind spots, to judge:**
  - **One poison value.** Spare capacity is filled with 0xA5A5 fields and 0xA5A5A5A5 headers
    (`poison` 978–988) before each build, except under Miri and in the unpoisoned half of
    class 11. A field the constructor failed to write would go unnoticed only if the oracle's
    value were exactly 0xA5A5 (42,405), a legal offset. In the measured rounds of the
    corpora the arena also holds the previous block's data rather than poison.
  - **`bytes()` assumes `Rec` and `RawG` have no padding.** The parent's compile-time asserts
    fix their sizes (64, 128), and their fields fill them.
  - **Shared definitions.** The candidate and the oracle share `phi`, `mix`, `ROWS`,
    `MAX_OFF`, `RAW_BIT`, `M20` and `RAWF` from the parent. An error in those would be
    common-mode and invisible here. They are the frozen definitions the traversal result
    used.
  - **Malformed counts.** Malformed cases check the return value and empty arrays.
    Out-of-bounds safety rests on the counted build's assertions firing before a write. The
    uncounted build is also run on the same malformed inputs, but in release an
    out-of-bounds write there would not necessarily be detected. Miri covered only the 40
    small quick malformed cases.
  - **Wider than the oracle.** The oracle needs n ≥ 1; n = 0 is checked against PARK alone.
    n > 983,010 and n ≥ 2²⁴ are checked only for rejection.
  - **Test-only paths.** Forced-RAW configurations exercise `complete_checked` and
    precedence; the natural decoder path never takes them.
  - **Thread scheduling.** The corpus and adversarial runs deal blocks to three threads
    dynamically, so which block follows which in an arena varies between runs.

## 4. The honest limits of what was run

- **Miri ran only the quick synthetic set** (942 cases: n ≤ 130, two-byte boundary sweeps,
  small run sweeps, the size sequence 200 → 0 → 61, 40 malformed cases, 18 canonicalisation
  cases). It reported no undefined behaviour. Its coverage, from the counted build's tallies:
  - 7 of 30 complete-group phases;
  - 55 of 71 ordinary and 62 of 71 RAW fragment shapes;
  - all 30 one-row slots, ordinary and RAW;
  - **no wide promotion.** That needs a span over 65,533 labels, so blocks over about
    61,000 rows, which the quick set doesn't reach;
  - the room walk didn't run under Miri (every quick case is under 65,536 rows).
- **Clippy was not run** (not installed for 1.97.0 here). The module is rustfmt-clean.
- **Nothing ran on a non-baseline target:** other CPUs' feature levels, other
  architectures, big-endian targets, or a non-glibc allocator. The harness's
  `malloc_usable_size` is glibc-only and reports 0 elsewhere.
- **No claim of exhaustive memory-safety verification is made.** The evidence is:
  - the written argument (the journal);
  - the counted build's assertions;
  - poisoned arenas;
  - byte-exact comparison on 6,052 corpus blocks, 528 adversarial blocks and 174,591
    synthetic cases;
  - Miri on the quick set only.

## 5. Reproduction

From a checkout of `e003a32` (`git worktree add <dir> e003a32` in a clone of lmmx/mibz2):

```
# the reviewed toolchain, baseline x86-64, no rustflags (the traversal protocol's recipe)
rustup toolchain install 1.97.0           # must report 2d8144b78, LLVM 22.1.6
CARGO_ENCODED_RUSTFLAGS= cargo +1.97.0 build --release --locked --example c30walk
B=target/release/examples/c30walk

# the 11 corpora, hash-checked first
sha256sum -c corpora.sha256               # 11 of 11 OK
$B ctor-check corpora.jsonl  mixed-independent.bz2 period-3.bz2 random.bz2 records.bz2 \
    text-l1.bz2 text-l5.bz2 text.bz2 wikidata-20261026.bz2 wikidata-fitted.bz2 \
    wikidata-heldout.bz2 zeros.bz2        # 44 configurations, each line ends "exact"

$B ctor-adversarial adversarial.jsonl     # 25 inputs × levels 1, 9 × 4 configs: 200 lines
$B ctor-synthetic                         # 174,591 cases, "every check exact (full)"
$B ctor-synthetic --quick                 # 942 cases

# Miri (a nightly with the miri component; the run used rustc 1.101.0-nightly 16d1ffac1)
cargo +nightly miri run --example c30walk -- ctor-synthetic --quick
```

**The harness's behaviour:**
- Any mismatch panics with the block, the configuration and the first differing record or
  group, and the process exits non-zero.
- `ctor-check` and `ctor-adversarial` use three worker threads.
- The release suites hold each corpus's blocks decoded (last column and production's output),
  several hundred MB for a Wikidata corpus; I didn't measure the peak.

**The corpora** are the established 11 (the walker journal's §8). Their SHA-256s are those
`corpora.sha256` lists (recorded in
`2026-11-03-compact30-constructor-prototype/corpora.sha256.check`):

| file | SHA-256 |
|---|---|
| mixed-independent.bz2 | `11639f3599a85391b5ac97447bd06a26481806cfb8e79e909c1ad5a25d0778db` |
| period-3.bz2 | `69893044ae8bff40453cf535878880aa97044e91f488a7f73a8a58df3716c4fe` |
| random.bz2 | `10d444e6204289af739688e54a348940d7915957172ea65035b08b72dfdc3f18` |
| records.bz2 | `ff8b5f6ed3dd44459908a01a588f6979b2544c6a444c9520741a2aa96c391f8c` |
| text-l1.bz2 | `d4778f8d965bb4b105b2b63f8899a0f95159687c260b438e13ff1a584b60be94` |
| text-l5.bz2 | `61498135ae2a358542022a77b1e7feedbaac40d8706bbb7c8baa5bd1bf04e660` |
| text.bz2 | `59236f2dff3afbd477d8cf4030055c301a9911e2ce26eb3bdbe26c39a26c695a` |
| wikidata-20261026.bz2 | `cf13f4394e4ea0a38a0e9e3bc5271b2dce717c8f98fe3a0b8a5186c84216b8fe` |
| wikidata-fitted.bz2 | `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915` |
| wikidata-heldout.bz2 | `7f5c9a0044e5a30589bf343b9b31e698849b2acc69461a55c97b550193128b45` |
| zeros.bz2 | `07dca4a334909c5f5bb3c88a380d760f40d7ec4481e6f5aff632a625e14add7b` |

**The recorded outputs to compare against** are in
`2026-11-03-compact30-constructor-prototype/`:
- `corpora.console`/`.jsonl`, `adversarial.console`/`.jsonl`, `synthetic.console`,
  `synthetic-quick.console`, `miri-quick.console`;
- the listing `build-false.asm`, which `objdump -d -C --no-show-raw-insn -l` of
  `<c30walk::ctor::Ctor>::build::<false>` should reproduce (symbol size 0x1b5e), apart from
  paths.

The counters are deterministic for a given input. The capacity and allocation counts depend
on which thread took which block.

## 6. What I'd like back

For each focus area in §3 (3.1–3.6), either:
- **a sign-off:** "reviewed, no issue found", with anything not reviewed stated; or
- **issues**, each with:
  - the location (file and line at `e003a32`);
  - the input or sequence that would trigger it, if known;
  - the failure it could cause: undefined behaviour, an out-of-bounds write, a read of
    uninitialised memory, wrong but safe output, a panic, a hang, or a harness blind spot that
    could hide one of these;
  - your severity: invalidates the correctness checkpoint, needs a fix before any cost
    measurement, or minor.

Also useful, if you find them:
- any case the suites should have run and didn't;
- any statement in the journal the code doesn't support.

A fix, if one is needed, would be a separate change for the guardian to authorise. This
handoff doesn't authorise any.
