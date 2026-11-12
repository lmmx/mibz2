# The 30-entry compact layout: a census on the corpus, with a reference round trip (untimed)

- **Authorised by:** external report 158, task 3. It covers census and reference validation
  only: no optimised traversal, no direct builder, no performance claims.
- **The layout:** external report 156's, unimplemented as a performance candidate.
- **Separate from:** the frozen FOR-14 experiment and its `k`/`j` arms, which stay untimed
  and unchanged.
- **Branch:** `research/compact30-census`, from master `13e9854`.
- **Records:** `2026-11-02-compact30-census/`.

## 1. What was run

**The program:** `cargo run --release --example census30 -- OUT.jsonl FILE.bz2 ...`
(`examples/census30.rs`). It runs single-threaded and is not timed.

**For each block, it:**
1. **Decodes the block to its last column** with the library's own scanner and MTF.
2. **Computes Ψ and F by its own counting scatter.** This is independent of `bwt::build`.
3. **Builds the proposed records** from Ψ, as an oracle builds them:
   - 64-byte records of 30 `u16` fields and a `u32` header;
   - labels φ(r) = 32⌊r/30⌋ + r mod 30;
   - the packed token q = φ | byte ≪ 20;
   - RAW groups of 128 bytes;
   - a parking record.
4. **Checks the specification against the data.** Any failure stops the run:
   - Ψ increases within every F bucket;
   - the largest live label and the parking label are below 2^20;
   - no stored successor lands on a header position (label mod 32 ≤ 29);
   - every ordinary offset is ≤ 65,533, so it is never confused with 0xFFFE or 0xFFFF;
   - for every ordinary row, header + offset = φ(Ψ(r)) | F[r] ≪ 20, with no carry;
   - the RAW-count bound R ≤ min(G, σ⌊M/65,534⌋ + σ − 1) holds for every block.
5. **Round-trips the block through an unoptimised scalar reference walk of the records
   alone.** One chain, seeded from L at the chain's start, must equal production's output
   (`bwt::Segmented`) byte for byte. It is walked twice:
   - as built;
   - with about 512 rows along the chain marked 0xFFFE. Each original field is saved and
     restored on the slow path, and START is tested before RAW.
6. **Records the block:**
   - rows and σ;
   - ordinary and RAW groups, with each RAW group's reason: the group crosses an F bucket,
     or is homogeneous with a span over 65,533;
   - the partial group's unused fields;
   - logical bytes 64(G + 1) + 128R;
   - the bound;
   - the largest ordinary span;
   - the starts marked, and how many fell on RAW rows.

**The corpora:** the established 11, as used by FOR-14's checks. They were hashed before the
run (`corpora.sha256`). The primary is `wikidata-20261026.bz2`, SHA-256 `cf13f439…`, the walk
trial's frozen file.

**The run:**
- every check passed;
- both round trips were exact on all 1,513 blocks: 707,949,378 rows (the first wording,
  "about 690 M", was an undercount, corrected after report 165);
- 39 s, one thread.

## 2. The result

| corpus | blocks (full / short) | rows | σ, full blocks | RAW groups: crossing / wide | RAW rows | bytes a row, all | full blocks, bytes a row: min / median / max | largest full block, bytes | RAW ÷ bound, worst full block | largest ordinary span |
|---|---|--:|---|---|--:|--:|---|--:|--:|--:|
| `wikidata-20261026` (primary) | 434 (230 / 204) | 207,006,515 | 80–98 | 18,684 / 30,949 | 0.718% | **2.1642** | 2.1603 / 2.1633 / 2.1707 | **1,953,600** | 0.198 | 65,529 |
| `wikidata-fitted` | 431 (233 / 198) | 209,706,557 | 79–102 | 18,853 / 30,755 | 0.708% | 2.1638 | 2.1580 / 2.1628 / 2.1698 | 1,952,704 | 0.195 | 65,531 |
| `wikidata-heldout` | 473 (245 / 228) | 235,773,776 | 81–98 | 21,022 / 34,703 | 0.708% | 2.1638 | 2.1570 / 2.1634 / 2.1685 | 1,951,424 | 0.187 | 65,533 |
| `text` | 13 (12 / 1) | 11,036,297 | 136 | 1,427 / 2,542 | 1.079% | 2.1795 | 2.1771 / 2.1790 / 2.1807 | 1,962,560 | 0.163 | 65,529 |
| `text-l5` | 23 (0 / 23) | 11,036,297 | – | 2,453 / 1,788 | 1.153% | 2.1828 | – | – | – | 65,533 |
| `text-l1` | 111 (0 / 111) | 11,036,297 | – | 10,405 / 104 | 2.848% | 2.2560 | – | – | – | 65,524 |
| `records` | 11 (10 / 1) | 9,600,002 | 256 | 2,702 / 0 | 0.844% | 2.1695 | 2.1679 / 2.1685 / 2.1689 | 1,951,936 | 0.065 | 11,369 |
| `random` | 9 (8 / 1) | 8,000,003 | 256 | 2,204 / 0 | 0.826% | 2.1687 | 2.1677 / 2.1683 / 2.1686 | 1,951,680 | 0.064 | 19,232 |
| `mixed-independent` | 3 (2 / 1) | 2,000,689 | 256 | 736 / 1,759 | 3.741% | 2.2931 | 2.2784 / 2.2794 / 2.2804 | 2,052,288 | 0.269 | 65,524 |
| `period-3` | 4 (3 / 1) | 2,700,000 | 3 | 8 / 0 | 0.009% | 2.1339 | 2.1337 (all) | 1,920,320 | 0.045 | 31 |
| `zeros` | 1 (0 / 1) | 52,945 | – | 1 / 0 | 0.057% | 2.1372 | – | – | – | 31 |

"Full" means at least 899,000 rows. The other blocks are short: a stream's tail, or the
smaller blocks of levels 1 and 5.

**On the primary corpus:**
- **Logical bytes a row:** 2.164. For comparison:
  - the shipped T-vector's 4.0, so 0.541×;
  - FOR-14/raw's 2.076 on the same corpus (its census, `2026-11-01-for14-stage1/`). The
    wider fields cost 4.2% more bytes than FOR-14.
- **The largest full block:** 1,953,600 logical bytes. That is 0.543× of the shipped
  3.6 MB, and under the σ ≤ 98 bound of 2,108,096.
- **RAW groups in a full block:** 189–262, median 210. Each block's bound is 1,199–1,469, so
  the worst block uses 19.8% of its bound. Of the RAW groups:
  - 62% are too wide: a homogeneous group whose 30 successors span more than 65,533 labels;
  - 38% cross an F bucket.
- **RAW rows:** 0.714% of full blocks' rows. This is a storage frequency, counted while
  encoding.
  - In this census's single-chain reference walk, each row is visited once, so there it is
    also the share of steps that took the RAW path.
  - It is not a measured slow-path frequency for an optimised multi-lane segmented walker.
    Parking, waits and re-checks, lane turns and starts can make that walker's dynamic
    rate differ (report 159).
- **Short blocks:** 204 blocks of 1–307 rows. Together they hold 86,400 bytes of 448 MB.
- **The partial-group overhead:** 3,865 unused fields over all 434 blocks.

**The specification held everywhere:**
- largest live label 959,997 and parking label 960,000, both below 2^20;
- no successor on a header position;
- every ordinary offset ≤ 65,533; the largest seen was 65,533;
- the token identity exact on every ordinary row;
- 0xFFFE and 0xFFFF never produced by an ordinary row.

**The marked starts:** 8,992 of the primary's 126,084 marked starts fell on RAW rows. Starts
on both ordinary and RAW rows round-tripped.

## 3. What this establishes, and what it doesn't

**It establishes:**
- **What the grouping does on the corpus.** The proposed records come to about 2.16 logical
  bytes a row on Wikidata. RAW use is well inside the worst-case bound.
- **The specification holds on the data:** the coordinate limits, the sentinel separation,
  and the token arithmetic.
- **The encoding round-trips exactly** through a reference walk, including starts marked
  over ordinary and RAW rows.

**It does not establish:**
- **The footprint or the cache.** These are logical bytes. They are not allocated capacity,
  resident memory or RSS, and the census says nothing about any cache-hit rate.
- **Speed.** The reference walk is scalar and untimed, and nothing here measures speed or
  predicts it.
- **The exceptional path's cost.** The census counts how often the RAW and start paths
  occur, not what they cost.
- **Construction.** The records are built from a full Ψ as an oracle. No direct builder
  was written.
- **Alignment.** The census's records are a `Vec<u16>`, which is not guaranteed 64-byte
  aligned. This doesn't affect the counts or the round trips. But a walker that relies on
  each record occupying one cache line needs an explicitly aligned 64-byte record type
  (report 165).
- **The segmented walk in its production shape:** lanes, parking and segment ends. The
  START sentinel's restoration was exercised on one chain only.

## 4. For the Astra review: the corrected wording (report 158)

The paragraph proposed earlier transferred the old compact-Ψ `u` result's cache behaviour
and percentage costs to this layout. It is replaced by:

> The historical four-block compact-Ψ result demonstrates that improved memory behaviour
> can coexist with slower traversal when access overhead is too high. The proposed 30-entry
> layout removes the old F-directory lookup and packed-bit extraction sequences, but
> retains other costs that must be evaluated.
>
> Its storage bound supports a materially smaller logical representation; it does not
> establish its cache-hit rate, resident footprint or performance. Please compare the
> actual access primitives and dependencies, rather than transferring the old format's
> measured gains or penalties directly.

**Two corrections are kept explicitly:**
- Four tables' logical size does not establish their cache-hit range.
- Removing the old lookup and packing sequences does not recover their historical
  percentage costs automatically.

**This census adds measured corpus counts:**
- 2.164 logical bytes a row;
- RAW on 0.72% of rows, at most 19.8% of the bound in any full block;
- 62% of RAW groups too wide and 38% crossing a bucket.

These are counts, not a forecast.
