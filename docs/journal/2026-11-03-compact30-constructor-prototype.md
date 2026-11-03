# Compact30: the direct-constructor prototype (compile and correctness only)

The bounded prototype that report 189 authorised: Astra's direct constructor (report 188,
[2026-11-03-compact30-constructor-design.md](2026-11-03-compact30-constructor-design.md)),
checked byte for byte against the frozen oracle `C30::build`, with untimed structural
counters, a memory-safety argument and the generated code. **There is no timing of any
kind in this work, and no performance claim.** Nothing here touches the walker, production
(`src/`), the closed VALID ADVANCE traversal result or its archive.

## The source pin

- **Base:** `ca7781e` (`research/compact30-walker`, reports 188–189).
- **Implementation:** `446a3b2` on `research/compact30-constructor`. Every figure, listing
  and console below is from that commit's tree.
- **The change is additive.** `examples/c30walk/ctor.rs` is new. `examples/c30walk.rs` gains
  13 lines and loses none (`git diff ca7781e -- examples/c30walk.rs` shows only `+` lines):
  six lines in its header comment (the `ctor-` modes' usage), the module declaration
  (`#[path = "c30walk/ctor.rs"] mod ctor;` with its doc line and a blank line), and a
  three-line dispatch of the `ctor-` modes in `main`. The walker, `Rec`, `RawG`,
  `C30::build`, `mix` and every other item are unchanged. No visibility change was needed:
  a child module reads its parent's private items.
- **Toolchain:** rustc 1.97.0 (`2d8144b78`, LLVM 22.1.6), `CARGO_ENCODED_RUSTFLAGS=` (empty),
  `--locked`, the release profile (codegen-units = 1, fat LTO, line tables), baseline x86-64:
  the traversal protocol's recipe. The binary's sha256 is `2bbcd632…` (not an identity:
  its line tables hold build paths). Miri: nightly rustc 1.101.0 (`16d1ffac1`).
- **Not run:** clippy (not installed for 1.97.0 here). The module is rustfmt-clean.

```
cargo +1.97.0 build --release --locked --example c30walk     # CARGO_ENCODED_RUSTFLAGS=
c30walk ctor-check OUT.jsonl CORPUS.bz2 ...                  # the 11 corpora
c30walk ctor-adversarial OUT.jsonl                           # the adversarial set
c30walk ctor-synthetic [--quick]                             # the direct L/counts cases
cargo +nightly miri run --example c30walk -- ctor-synthetic --quick
```

## What was built

One constructor, `Ctor::build`, from L and its counts to `rec` and `raw`, following report
188 section by section:

| report 188 | the prototype |
|---|---|
| validate the block size and counts | counts must sum to n, n < 2²⁴ (production's one-pass condition), and n ≤ 983,010 (the oracle's PARK label fits 20 bits; the oracle asserts this). Otherwise `false` at once, both arrays empty |
| encoded bucket cursors and limits | `x[c] = φ(bucket start)`, `xl[c] = φ(bucket end)`; group `x >> 5`, slot `x & 31` |
| crossing groups up front | every prefix sum 0 < B < n with B mod 30 ≠ 0 makes group B / 30 RAW, deduplicated, in increasing g |
| production's 16-byte test, coalescing, the failing chunk's words kept | `a == b && a == a.rotate_left(8)`; following chunks with both words equal to `a` extend the run; the first failing chunk's two words are held and reused by the next iteration |
| one limit check and one division per coalesced run | `q = (j + m) / 30`, `x_end = x + m + 2q`, `x_end ≤ xl[c]`, checked before any write of the run |
| prefix / complete groups / suffix | prefix `(j, 30 − j)` when the run leaves its first group, `(j, m)` when it doesn't; `q − [j > 0]` complete groups; suffix `(0, (j + m) mod 30)` |
| thirty complete-record patterns, header += 32 << 8 | `static PATTERNS: [Rec; 30]`, `f[j] = j + 2·[p + j ≥ 30]`, header 0 (1,920 bytes); `complete` writes `PATTERNS[p]` with header `(y << 8) | c`, adding `32 << 8` per group, `y += 32·whole`, p unchanged. No header read, no RAW test, no span check |
| bounded fragment writer | `fragment(c, g, j, k)` writes exactly k fields (or k RAW entries) at slots j..j+k−1: at j = 0 the bitmap decides RAW, otherwise the header; a fresh group's header is set by its slot-0 row |
| early wide promotion before the overflowing fragment | the fragment's last label against `base + 65,533`; on overflow, a zeroed `RawG` is appended, the j ≤ 29 written fields are reconstructed into it as `(base + offset) << 8 | F`, the record becomes RAW (all fields 0xFFFF, provisional header, bitmap bit), then the fragment is written as RAW entries |
| source label and phase without per-row division | `y = φ(i)`, `p = i mod 30`; `advance(k)` for k ≤ 30 is one comparison and one wrap adjustment |
| the one-row fallback | `one(c)`: limit check, `x + 1 + 2·[x & 31 = 29]`, then the slot's header/RAW/span logic for one row |
| end: every cursor at its limit, the last group's unused fields, PARK, canonical RAW order, publish | as listed, then `rec.set_len(G + 1)` |
| canonical RAW ordering in place | ranks in increasing g from the bitmap; each rank stored in its `RawG`'s word 31 and the header patched; cycle rotation with one carry `RawG` (`mem::swap` of the carry with its destination); every word 31 zeroed |

**Forced RAW (test configurations only):** after the crossing groups, every group whose bit
is clear and for which the oracle's exact predicate `permille > 0 && mix(g ^ n << 32) % 1000
< permille` holds is made RAW (a second pass, so their provisional indices interleave with
the crossing groups'). Complete groups then go through `complete_checked`, which tests the
bitmap per group and routes RAW groups to the RAW writer. The RAW reserve is G groups. The
RAW set is crossing ∪ wide ∪ predicate, so the arrays don't depend on precedence; the
harness classifies each RAW group from the arrays in the oracle's order (crossing, then
wide by its span, then forced) and compares the classes with the oracle's `Build` counts.

**`build::<true>` and `build::<false>`** are the same constructor. The first adds the
counters and coverage tallies and turns every index condition into an `assert!`; the second
has neither (its index conditions are `debug_assert!`) and is the one whose code is
inspected below. Both are compared with the oracle on every case.

**Deviations from report 188, and choices it left open:**
- **A nonuniform chunk's bytes are read again individually** (`ll[i]`), as production's
  one-row step reads them. Only the uniformity test's two words are reused.
- **Coalescing stops at whole 16-byte chunks**, as specified: the partial chunks at either
  end of a longer equal run take the one-row path.
- **Forced groups are pre-marked in a separate pass after the crossing groups.** Report 188
  says only "pre-mark selected groups". This gives the canonical ordering nontrivial
  permutations to undo in the forced configurations.
- **The bitmap is tested with a bounds-checked index**, and the RAW arena is written through
  bounds-checked `Vec` indexing. Neither is on the complete-record path.
- **Capacity growth releases first:** an insufficient `rec` or `raw` allocation is dropped
  (`*v = Vec::new()`) before `reserve_exact`, so old and new arenas never coexist. The bitmap
  is a reused `Vec` (`clear` + `resize`), which may grow in place.
- **No other strategy, variant or format change.** The walker is untouched.

## Correctness: every class passed

Each case runs the oracle's build, then `build::<true>` and then `build::<false>` into the
**same reused arena**, each compared with the oracle's arrays in length and every byte
(headers and indices, partial-record fields, RAW entries and padding, PARK), and the classes
compared with the oracle's. Then, with production's output for the block, the **unchanged
walker** runs on the constructor's arrays, counted (`walk::<true>`) and audited
(`walk::<false>`): each output against production's, the counted output and all fourteen
transition counts against the oracle arrays' own counted walk, and the arrays against the
oracle's again after each walk (marker restoration; `walk::<true>` also asserts it
internally). In the corpus and adversarial runs, and the full synthetic run, the record
arena's spare capacity is **poisoned** (0xA5A5 fields, 0xA5A5A5A5 headers) before each build,
so a field left unwritten shows as a difference.

| class | cases | array comparisons | walk outputs | restorations | result |
|---|--:|--:|--:|--:|---|
| 11 hash-checked corpora (`corpora.sha256`, 11 of 11 OK) × the 4 configurations (permille 0 / 0 + adversarial starts / 50 / 1000) | 44 (6,052 blocks) | 12,104 | 18,156 | 12,104 | **exact** |
| the adversarial set: 25 inputs × levels 1 and 9 × 4 configurations | 200 (528 blocks) | 1,056 | 1,584 | 1,056 | **exact** |
| synthetic, full (`ctor-synthetic`) | 174,591 | 341,014 | 511,521 | 341,014 | **exact** |
| synthetic, quick, release | 942 | 1,748 | 2,622 | 1,748 | **exact** |
| synthetic, quick, **under Miri** | 942 | 1,748 | 2,622 | 1,748 | **exact, no undefined behaviour reported** |

**The synthetic classes** (`ctor-synthetic`, every one exact; the walk compares with
production's own inversion of the same L and counts):

| class | cases | what it covers |
|---|--:|---|
| small blocks | 4,192 | every n from 0 to 130 (every partial-record length, n mod 30 = 0..29), σ ∈ {1, 2, 3, 5}, mixed run lengths; n = 0 checked as PARK alone (the oracle needs n ≥ 1) |
| a bucket boundary at every offset | 1,620 | two bytes, boundary B = 1..n−1 for n ∈ {30, 59, 60, 61, 90, 97, 150}, three arrangements each: every internal boundary offset within a group |
| source phase × destination slot × run length | 137,370 | a run of m ∈ {1, 15, 16, 17, 29–33, 47, 48, 59–61, 64, 90, 120, 150, 256} after P = 0..240 filler rows (every source offset mod 16 and every even coalesced-run phase), with the bucket at every slot 0..29 |
| equal-run lengths 1..=200 | 25,600 | two runs per block at 32 source offsets and 4 destination offsets |
| spans 65,530..=65,538 through one-row writes | 189 | the group's first and last labels exactly that far apart, at 7 source offsets and 3 destination offsets; **65,533 ordinary in 21 cases, 65,534 wide in 21 cases** (asserted) |
| promotion through a run's prefix fragment | 1,392 | t = 1..29 earlier rows, then a coalesced run whose fragment endpoint is swept across the limit; the promotion count is asserted equal to `[endpoint offset > 65,533]`; an endpoint of exactly 65,533 (ordinary) in 3 cases; **79 promotions where the fragment's first offset fits and its last doesn't** |
| promotion through the one-row path | 58 | t = 1..29 earlier rows, then a far row; one promotion and t reconstructed fields asserted |
| RAW discovery order unlike group order | 18 | rare bytes spread over 300,000–899,999-row blocks (wide groups discovered late) among run-heavy filler (crossings first), at permille 0, 50 and 1000: 87,769 groups moved in 60 cycles, the longest 19,880 |
| one-symbol blocks | 48 | n ∈ {1, 2, 15–17, 29–32, 47, 48, 60, 61, 65,536, 899,999, 900,000}, bytes 0, 97, 255 |
| 900,000-row and 983,010-row blocks | 30 | random (σ 256 and 4) and runs (σ 20, 98, 2), at permille 0, 50 and 1000; 983,010 is the largest n the oracle accepts |
| repeated construction with changing sizes | 24 | 900,000 → 1 → 899,990 → 31 → 65,537 → 30 → 900,000 → 59 → 500,000 → 0 → 65,536 → 899,999, unpoisoned then poisoned, permille cycling 0 / 1000 / 50 (so the RAW requirement grows and shrinks) |
| malformed counts and sizes | 4,002 | 4,000 perturbed counts (one more, one fewer, k moved between bytes, counts swapped between bytes) at n from 1 to 70,000: **3,995 rejected, 7 accepted, accepted exactly when the perturbed counts equal L's own** (asserted); both builds agree; rejected builds leave both arrays empty. Plus n = 983,011 and n = 2²⁴: rejected |
| RAW canonical ordering, independently | 48 | `canonicalise` alone on prepared records: 8 sizes (up to 30,000 groups, 1,200 RAW) × identity, reverse, rotation, adjacent pairs, random, and one full cycle (Sattolo); afterwards the headers hold ranks in group order, each `RawG` holds its own group's entries, **every word 30 and 31 is zero**, the ordinary records are untouched, and the moves equal the non-fixed points: 8,057 groups moved in 1,645 cycles |

**Coverage, tallied by `build::<true>` over the synthetic run:**
- complete groups at **all 30 source phases**;
- run fragments: **all 71 reachable (start slot, length) shapes, ordinary and RAW**. A
  coalesced run is whole 16-row chunks, so its fragment is a prefix `(j, 30 − j)` (29 shapes),
  a suffix `(0, r)` (29), or the whole run inside one group, `(j, 16)` with j ≤ 13 (13);
  no other shape can occur;
- one-row writes at **all 30 slots, ordinary and RAW**;
- promotions after **every prefix length 1..29, through a fragment and through one row**.

**Malformed counts fail before any out-of-bounds write.** Each write's index is bounded by
the check that precedes it (the run's `x_end ≤ xl[c]`, the row's `x < xl[c]`), and
`build::<true>` asserts every record index below G before the write; none of the 4,002
malformed cases tripped an assertion, and under Miri the 40 quick malformed cases ran with
every access checked. Rejection matches production's `build_one_pass`, which returns false
for the same counts; the prototype also rejects n > 983,010, which the oracle can't build.

**Miri** (`cargo +nightly miri run --example c30walk -- ctor-synthetic --quick`, Stacked
Borrows, no flags): the quick set, 942 cases, with fresh unpoisoned arenas so that any read
of an unwritten byte is reported, through the oracle, both builds, the canonicalisation tests
and the plain walk (every quick case is under 65,536 rows, so the room walk and its prefetch
don't run). The corpora, the adversarial set and the large synthetic cases can't run under
Miri in reasonable time; they ran in release with the counted build's assertions instead.
**Miri's result:** every quick class passed (942 cases, exit status 0) and Miri reported no
undefined behaviour: no read of uninitialised memory, no out-of-bounds or misaligned
access, no aliasing violation, no leak. **Its coverage is partial:** the quick set reaches
7 of the 30 complete-group phases, 55 of the 71 ordinary and 62 of the 71 RAW fragment shapes,
all 30 one-row slots (ordinary and RAW), forced RAW and the canonical ordering, but **no wide
promotion** (that needs a span over 65,533, so blocks over 61,000 rows, too large for Miri
here). `promote` is therefore covered only by the release runs, with the counted build's
assertions and the poisoned arenas.

## The structural counters (untimed)

`build::<true>`, the natural configuration (permille 0), each corpus's blocks summed. A row
is counted on exactly one path: **complete** (a complete-record pattern), **fragment** (a
run's prefix or suffix), or **one-row** (a nonuniform chunk, or the last n mod 16 rows).
**Run rows** are the rows in coalesced runs, exactly the rows production's 16-row path
takes.

| input | blocks | rows | complete | fragment | one-row | runs | run rows | wide promotions | reconstructed fields | RAW groups | moved | cycles |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| mixed-independent | 3 | 2,000,689 | 0.051% | 0.015% | 99.934% | 14 | 0.066% | 1,759 | 31,349 | 2,495 | 2,492 | 15 |
| period-3 | 4 | 2,700,000 | 99.983% | 0.010% | 0.007% | 10 | 99.993% | 0 | 0 | 8 | 0 | 0 |
| random | 9 | 8,000,003 | 0.000% | 0.000% | 100.000% | 0 | 0.000% | 0 | 0 | 2,204 | 0 | 0 |
| records | 11 | 9,600,002 | 0.000% | 0.000% | 100.000% | 0 | 0.000% | 0 | 0 | 2,702 | 0 | 0 |
| text-l1 | 111 | 11,036,297 | 15.309% | 7.881% | 76.810% | 40,971 | 23.190% | 104 | 2,466 | 10,509 | 3,974 | 90 |
| text-l5 | 23 | 11,036,297 | 20.153% | 9.431% | 70.416% | 47,684 | 29.584% | 1,788 | 29,092 | 4,241 | 4,146 | 123 |
| text | 13 | 11,036,297 | 24.803% | 11.641% | 63.556% | 57,579 | 36.444% | 2,542 | 39,324 | 3,969 | 3,959 | 57 |
| **wikidata-20261026** (primary) | 434 | 207,006,515 | **74.340%** | 7.160% | 18.499% | 607,999 | 81.501% | 30,949 | 476,120 | 49,633 | 48,833 | 1,086 |
| wikidata-fitted | 431 | 209,706,557 | 74.992% | 6.952% | 18.056% | 597,553 | 81.944% | 30,755 | 473,666 | 49,608 | 48,813 | 1,170 |
| wikidata-heldout | 473 | 235,773,776 | 76.056% | 7.326% | 16.617% | 707,859 | 83.383% | 34,703 | 532,340 | 55,725 | 54,931 | 1,270 |
| zeros | 1 | 52,945 | 99.839% | 0.128% | 0.032% | 2 | 99.968% | 0 | 0 | 1 | 0 | 0 |
| **all 11** | 1,513 | 707,949,378 | **70.609%** | 7.044% | 22.346% | 2,059,671 | 77.654% | 102,600 | 1,584,357 | 181,095 | 167,148 | 3,811 |

**The key quantity:** on the primary corpus, **74.34% of rows reach the complete-record
path**, 7.16% go through run fragments and 18.50% through the one-row path. Of the 81.50% of
rows that production's 16-row path takes, 91.2% reach a complete record here; the rest are
the runs' prefix and suffix fragments. The other two Wikidata corpora agree (74.99% and
76.06%). The three text corpora reach 15–25%, and random, records and mixed-independent
essentially none.

**More counters** (the same configuration):

| input | complete groups | batches (runs with ≥ 1) | prefix fragments | suffix fragments | fragment rows into RAW | one-row rows into RAW | partway promotions | crossing groups | longest cycle |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| wikidata-20261026 | 5,129,645 | 294,250 | 587,567 | 493,488 | 182,914 | 826,774 | 0 | 18,684 | 230 |
| wikidata-fitted | 5,242,119 | 288,492 | 577,797 | 485,775 | 184,459 | 827,156 | 1 | 18,853 | 234 |
| wikidata-heldout | 5,977,368 | 335,261 | 684,249 | 575,709 | 231,422 | 904,964 | 1 | 21,022 | 237 |
| text | 91,245 | 15,878 | 55,649 | 42,742 | 3,379 | 76,367 | 0 | 1,427 | 312 |
| all 11 | 16,662,615 | 954,470 | 1,990,975 | 1,661,185 | 614,471 | 3,223,875 | 2 | 78,495 | 989 |

- **Wide promotions equal the oracle's wide groups** on every corpus (30,949 on the primary,
  the census's figure), with 15.4 fields reconstructed per promotion on the primary
  (476,120 / 30,949), against the bound of 29.
- **RAW groups moved by the canonical ordering:** 48,833 of 49,633 on the primary (98.4%):
  the crossing groups are created first and the wide groups are appended in discovery order,
  so almost every group is displaced. 1,086 cycles over 434 blocks, the longest 230.
- **The coalesced runs, all 11 corpora** (bins by upper bound, rows):

| run length | ≤ 16 | 17–32 | 33–48 | 49–64 | 65–128 | 129–256 | 257–512 | 513–1,024 | 1,025–4,096 | 4,097–65,536 | > 65,536 |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| runs | 760,136 | 318,854 | 157,685 | 107,022 | 203,808 | 152,188 | 116,603 | 132,400 | 96,075 | 14,879 | 21 |
| rows | 12,162,176 | 10,203,328 | 7,568,880 | 6,849,408 | 20,328,768 | 28,951,232 | 43,316,528 | 95,155,280 | 196,814,768 | 124,889,328 | 3,508,128 |
| complete-record rows | 0 | 954,060 | 2,990,160 | 3,740,700 | 14,417,850 | 24,542,910 | 39,935,760 | 91,309,110 | 194,023,050 | 124,457,370 | 3,507,480 |

  A 16-row run never fills a group; 37% of runs are 16 rows long but they hold 2.2% of the run
  rows. Runs over 512 rows hold 76.5% of the run rows and 82.7% of the complete-record rows.

**The adversarial set** (natural, 132 blocks, 15.65 M rows): complete 70.12%, fragment
0.94%, one-row 28.94%; 16 promotions; 9,096 RAW groups, 164 moved. **The forced
configuration** (permille 1000) routes every complete group to the RAW writer (on the
primary, 153,889,350 rows), and the canonical ordering then moves 6,801,899 of 6,900,346
groups on the primary; these are test configurations, not the natural construction.

All counters, per corpus and configuration, are in `corpora.jsonl` and `adversarial.jsonl`.

## Actual capacities

`Vec::capacity` after each build (the constructor reserves exactly), and the allocator's
usable bytes behind it (glibc `malloc_usable_size`, read by the harness), the maxima over
each corpus's blocks:

| array | requested at 900,000 rows | allocated (`capacity`) | usable bytes (allocator) |
|---|--:|--:|--:|
| `rec` (G + 1 records, PARK included) | 30,001 records = 1,920,064 B | 30,001 | 1,920,072 |
| `raw`, natural bound, σ = 256 (random, records, mixed-independent) | 3,839 groups = 491,392 B | 3,839 | 491,400 |
| `raw`, natural bound, primary corpus (σ ≤ 98) | 1,469 groups = 188,032 B | 1,469 | 188,040 |
| `raw`, forced-RAW tests (G groups) | 30,000 groups = 3,840,000 B | 30,000 | 3,840,008 |
| RAW bitmap | 469 words = 3,752 B | 469 | — |
| cursors and limits (stack) | 2,048 B | — | — |
| patterns (static, shared) | 1,920 B | — | — |

- The RAW groups actually used are far below the reserve: at most 262 in a primary-corpus
  block (33,536 bytes of the 188,032 reserved), and 1,033 on mixed-independent.
- **No RAW push ever exceeded its reserve** (`raw_overflows` 0 everywhere): the natural
  bound held on every block.
- **Reuse:** each of the three worker threads keeps one arena across its blocks. A worker
  reallocates `rec` only when a block needs more records than it holds (38 allocations
  over 1,513 blocks and 3 workers), and `raw` when the bound rises (80); smaller blocks
  reuse the larger capacity.
- No successor table, F array or class array is allocated.

## Memory safety

**The approach:** only the record arena is used uninitialised, where the proof below is
direct. The RAW arena and the bitmap are initialised by safe code, as the design has them
(each created `RawG` pushed zeroed, 128 bytes; the bitmap zero-filled, 3,752 bytes at
900,000 rows). There is no zero-fill of the record arena, and no extra initialisation pass
beyond the design's.

**The record arena (`rec: Vec<Rec>`).**
- **Its length stays 0** until the end. Writes go through `dst = rec.as_mut_ptr()` into the
  reserved capacity (G + 1 records, 64-aligned), with `ptr::write` and `addr_of_mut!`. No
  reference to an unwritten record is made; the only reads are headers and fields this
  build has written (below). On failure the length stays 0, so nothing is exposed.
- **Every index is in bounds.** A crossing group is B / 30 with B < n; a forced group is
  below G. A row's slot satisfies x < xl[c] ≤ φ(n), and a run's x_end ≤ xl[c] ≤ φ(n); since
  slots never take labels 30 or 31, every group touched is at most (φ(n) − 1) >> 5 = G − 1.
  PARK is at G. These are asserted in `build::<true>`.
- **Every record is complete when the length is set.** At the end each group g < G is in
  one of two states:
  - **Its bit is set.** It was made RAW by `create_raw` or `promote`, each of which writes
    all 64 bytes (30 × 0xFFFF and the header) when it sets the bit. After that no field of
    g is written: every writer tests RAW status first (the bitmap at slot 0, the header at
    j > 0), and the complete-record emitter touches only groups that can't be RAW (in the
    natural configuration a complete group lies inside one bucket, so it isn't crossing,
    and none of its rows was written before, so it hasn't been promoted; in a forced
    configuration `complete_checked` tests the bit). Only the header changes afterwards, in
    the canonical ordering.
  - **Its bit is clear.** Then g is not crossing, so all its rows lie in one bucket c by
    the counts. The build succeeds only if every cursor ends at its limit; each cursor rises
    by exactly the rows written through it, so every slot below n was written exactly once.
    Each of g's slots was written on the ordinary path, since a RAW path would have set the
    bit. The write at slot 0 set the header; every slot's write set its field. The last
    group's fields at slots ≥ n − 30(G − 1) are set to 0xFFFF explicitly. So all 30 fields
    and the header are written.
  - **PARK** is written explicitly.
- **Every read is of written memory.** A header is read at slot j > 0 of g. If g is RAW,
  the header was written when it became RAW. Otherwise g isn't crossing, so bucket c starts
  at or before 30g, and its cursor wrote slot 0 (setting the header) before slot j. At slot
  0 the header is read only when the bit is set. `promote` reads fields 0..j−1 of an
  ordinary group, which the same cursor wrote before slot j. This holds whatever the counts:
  it uses only the counts' bucket ranges and the cursors' order, not their agreement with L.
  So malformed counts can't cause a read of unwritten memory.
- **Stale data from a reused arena is never read,** because nothing is read before it is
  written in the same build. The poisoned-arena runs (every corpus, adversarial and full
  synthetic build) would show a missed field as a difference; Miri, on fresh allocations,
  would report a read of an unwritten byte as undefined behaviour.

**The RAW arena (`raw: Vec<RawG>`).** Safe operations only: `reserve_exact` after release,
one `push(RawG([0; 32]))` per RAW group (so every entry starts at zero), entries written by
bounds-checked indexing at slots below 30, and the canonical ordering by bounds-checked
indexing and `mem::swap`. **Word 30** is never written. **Word 31** holds the destination
tag only inside `canonicalise`, and is zeroed for every group before it returns. Entries for
rows past n stay zero. If a push exceeded the reserve, `Vec` would reallocate safely; it
never did.

**The rest:** the bitmap is a zero-filled `Vec<u64>`, indexed with bounds checks; the
cursors are stack arrays, fully initialised; the patterns are an immutable static.

## The generated code

`Ctor::build::<false>`, from the build above (0x1b5e bytes; `canonicalise` and `promote`
are out of line; `complete`, `fragment`, `one` and `run` are inlined). The full listing and
the excerpts are in `2026-11-03-compact30-constructor-prototype/`. There is no `div`
instruction in the function: each division by 30 is a multiply by its reciprocal.

**The complete-record loop does exist as a tight bulk path.** It is unrolled by two:
`%rdi` points at `PATTERNS[p]`, `%eax` holds the header, `%r10` the destination.

```
246f0:  movdqu 0x2c(%rdi),%xmm0        ; pattern bytes 44..59 (fields 22..29)
246f5:  movdqu %xmm0,-0x50(%r10)       ; record A, bytes 44..59
246fb:  movdqa (%rdi),%xmm1            ; pattern bytes 0..15
246ff:  movdqa 0x10(%rdi),%xmm2        ; pattern bytes 16..31
24704:  movdqa 0x20(%rdi),%xmm3        ; pattern bytes 32..47
24709:  movdqa %xmm3,-0x5c(%r10)       ; record A, bytes 32..47
2470f:  movdqa %xmm2,-0x6c(%r10)       ; record A, bytes 16..31
24715:  movdqa %xmm1,-0x7c(%r10)       ; record A, bytes 0..15
2471b:  mov    %eax,-0x40(%r10)        ; record A, header
2471f:  add    $0x2,%r8
24723:  movdqa %xmm1,-0x3c(%r10)       ; record B, bytes 0..15
24729:  movdqa %xmm2,-0x2c(%r10)       ; record B, bytes 16..31
2472f:  movdqa %xmm3,-0x1c(%r10)       ; record B, bytes 32..47
24735:  movdqu %xmm0,-0x10(%r10)       ; record B, bytes 44..59
2473b:  lea    0x2000(%rax),%r11d      ; B's header = A's + (32 << 8)
24742:  mov    %r11d,(%r10)            ; record B, header
24745:  add    $0x4000,%eax            ; the next pair's header
2474a:  sub    $0xffffffffffffff80,%r10  ; destination += 128
2474e:  cmp    %r8,%r9
24751:  jne    246f0
```

- **Per record:** 64 bytes stored as three aligned 16-byte stores, one unaligned 16-byte
  store at byte 44 (fields 22–29, overlapping fields 22–23 of the third store and ending at
  byte 59, so it never reaches the header), and one 4-byte header store. The header advances
  by `0x2000` per record with a `lea`/`add`, in a general register.
- **No per-group header read, RAW test, span check, bitmap access or bound check** inside
  the loop; its only branch is the back-edge. An odd last record takes the same five stores
  once after the loop (`24773`–`2479f`).
- **It differs from Astra's four-XMM hypothesis in two ways.** The pattern is **reloaded
  from the static table every iteration** (four 16-byte loads per two records) instead of
  staying in registers, and the header is a **separate scalar store** rather than a lane of
  the fourth vector updated by `paddd`: five stores per record rather than four. Both are
  observations, not changes; the brief rules out tuning the code generation.

**The fragment writer stores only its own fields.** The ordinary fragment at j > 0 (the
prefix) reads the header (`mov 0x3c(%rcx,%rax,1),%eax`), tests RAW
(`test $0x10000000,%eax`), bounds-checks `pat[k − 1]` (`cmp $0x1d`, a branch to a panic never
taken), and compares the endpoint offset with 65,533 (`cmp $0xfffd,%edi; ja` to the
out-of-line `promote`). It then stores the offsets `d + pattern` with a loop of two 16-byte
`paddw`/`movdqu` stores (16 fields) at `24200`, a 4-field `movq` loop at `242f0`–`24307`, and a
2-byte scalar tail (`mov %ax,(%rcx)` at `2434b`):

```
24200:  movdqa -0x10(%r10,%rdi,2),%xmm1
24207:  paddw  %xmm0,%xmm1             ; offsets + d (d broadcast in xmm0)
2420b:  movdqa (%r10,%rdi,2),%xmm2
24211:  paddw  %xmm0,%xmm2
24215:  movdqu %xmm1,-0x10(%rbp,%rdi,2)
2421b:  movdqu %xmm2,0x0(%rbp,%rdi,2)
```

  Each loop's trip count is the fragment's own length rounded down to its width, so no
  store passes slot j + k − 1. The suffix fragment (j = 0) stores the header, then copies the
  pattern's first k fields: byte-pair scalar stores below 7 fields, otherwise the same
  16-field and 4-field vector loops (`24901`, plain `movdqa`/`movdqu` copies), after a
  run-time check that the destination doesn't overlap `PATTERNS` (LLVM can't prove it).
  RAW fragments write entries 8 at a time (`punpcklwd`, `paddd`, `pslld $8`, `por`, two
  `movdqu`, at `24120`) with a scalar tail.
- **The fragment and one-row paths are branchy and spill to the stack.** Each fragment
  computes its loop bounds (including a multiply by the reciprocal of 30 in the suffix's
  vector setup, `movabs $0x8888888888888889`), chooses among scalar and vector loops by
  length, and reloads several values from the stack. The one-row path (`23cea`–`23d12`) is a
  header load, a RAW test, a subtraction, `cmp $0xfffe` and one 2-byte store, plus its
  cursor update. These are the parts report 188 named as the risk; their cost isn't
  measured here.

## What failed, and what remains open

- **Nothing failed** in any correctness class. One harness slip was corrected before the
  final runs: the uncounted build first moved its 16 KB counter struct in and out
  (`mem::take`) on every build, visible in the listing as two `memcpy` and a `memset`; the
  counters are now borrowed in place, and `build::<false>` touches none of them.
- **The first Miri attempt** used a quick set too large to finish (its run-length class had
  640 cases with walks) and was stopped before reaching its end, after three classes had
  passed. The quick run-length class was reduced to 44 cases. A second Miri run was stopped
  part-way when the module was reformatted (rustfmt only), so that the reported run, the
  release runs and the listing all come from the committed source. The final Miri run is the
  one reported above.
- **Not tested:** the room walk under Miri (no quick case reaches 65,536 rows); a block
  with n ≥ 2²⁴ beyond the rejection check; big-endian targets (`from_ne_bytes` and the
  uniformity test, as production's, assume nothing about endianness, but nothing here ran
  on one); any target other than baseline x86-64.
- **Open, as report 188 says:** whether the 74% complete-record share pays for the 7%
  fragment rows and the 18.5% one-row rows within the 1.06714 construction budget. This
  prototype measures no cost. The listing shows the bulk path survives compilation; it also
  shows the pattern reloads, the separate header store, and fragment and one-row paths with
  more bookkeeping than their few stores. Any construction-cost comparison needs its own
  separately approved protocol.
- **Not answered here:** the cache state a direct constructor leaves for the walk (the
  traversal capture copied a prebuilt image), which report 188 leaves to integration.

## Files

In `2026-11-03-compact30-constructor-prototype/`:
- `corpora.console`, `corpora.jsonl`, `corpora.sha256.check`: the 11 corpora;
- `adversarial.console`, `adversarial.jsonl`: the adversarial set;
- `synthetic.console`, `synthetic-quick.console`, `miri-quick.console`: the synthetic suite,
  in release and under Miri;
- `build-false.asm`: `Ctor::build::<false>`, `objdump -d -C --no-show-raw-insn -l`, paths
  shortened; `complete-loop.asm` and `fragment.asm`: the excerpts quoted above.
