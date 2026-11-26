# Does the retained-rank scatter need two passes?

The original edge constructor's final queue order is already close to an ideal streaming write at 8–32 KiB of modeled queue-cache capacity. Grouping by high nibble removes some return visits to the final row vectors, but adds one descriptor store and one descriptor load per piece. On every one of the 233 real edge admissions, the complete staged trace increases modeled line misses at all tested capacities from 2 through 32 KiB. At a forced 1 KiB limit the sign reverses on all 233. This closes the preregistered construction screen. Decoder timing and process RSS were outside its scope. [Plan at e72414c](PLAN.md) · [Raw Wikidata results](evidence/wikidata/summary.txt)

## What remains exact

The reference is the edge constructor at `badb2aec3624bd65606c429b2e6f79e9d9d0050f`. Its initial retained rank is zero. A maximal run of width at most 16 contributes its entire consecutive rank interval. A wider run containing the primary contributes its marked singleton. Other wide runs contribute no ranks. The extractor combines adjacent capped records of the same byte before applying these rules. It applies the candidate's existing density and width gates after the production run-front dispatch.

For each retained piece starting at rank m in byte c, write one 28-bit descriptor to stream floor(c/16). Encode m in bits 0–19, width minus one in bits 20–23 and the low nibble of c in bits 24–27. The bzip2 block limit of 900,000 rows makes the rank fit. Stable append keeps the relative order of any two pieces of the same byte. Reading one stream reconstructs c from its two nibbles. Appending each decoded rank interval to queue c therefore recovers the original rank vector, in order, for every c. The mark is still available at the original source pass. All downstream consumers see the same 256 vectors on arbitrary marked cycles. The executable materialized both sets of rank vectors independently and compared every entry. This checked 26,567,581 ranks on 209,697,662 real input rows.

The previous queue-spans experiment changed final queue storage and projection. Its raw spans and joined spans could be consumed directly. This experiment adds an intermediate partition while retaining the original flat projection. The earlier radix-partitioned walk grouped live suffix walks by target addresses during an entirely different stage. [Queue spans](../queue-spans/README.md) · [Partitioned walk](../../../docs/journal/2026-11-05-partitioned-walk/results.json)

## What the queue traces cost

Across 431 stream blocks, the run front received 383 and the measured edge gate admitted 233. Their 14,386,594 retained pieces included 10,851,747 singletons and 13,327,319 of width at most four. The busiest 16 bytes in each block accounted for 75.08% of all retained pieces, the busiest 32 for 93.40%. The median block activated 85 byte queues and seven high nibbles. The extra packed representation writes and rereads 57,546,376 logical payload bytes each way, with up to 717,616 live descriptor bytes per block before vector capacity slack. Existing row buffers still have their original lengths when projection begins. Peak process memory would also depend on capacities retained across blocks and the allocation schedule.

The table uses 64-byte aligned vector starts, all cold at each block boundary, with byte-stream base sets spread in byte order. Values count modeled misses summed across blocks. Fully associative queues use exact LRU stack distance. The conflict model uses 64 sets and LRU replacement within each set. Each final rank store touches every line its interval intersects. The complete staged trace first writes high-nibble descriptors, then reads each descriptor immediately before writing its final queue interval.

| Queue capacity/model | Original scatter | Grouped final writes alone | Complete staged scatter |
|---|---:|---:|---:|
| 1 KiB, fully associative | 4,575,892 | 1,911,338 | 3,985,253 |
| 2 KiB, fully associative | 2,364,899 | 1,734,309 | 3,581,091 |
| 4 KiB, fully associative | 1,926,099 | 1,697,009 | 3,519,505 |
| 8 KiB, fully associative | 1,789,794 | 1,681,132 | 3,492,347 |
| 16 KiB, fully associative | 1,730,828 | 1,675,618 | 3,479,939 |
| 32 KiB, fully associative | 1,701,768 | 1,671,647 | 3,474,651 |
| 64 sets × 8 ways, 32 KiB | 1,707,337 | 1,672,429 | 3,475,830 |

The compulsory line count is 1,670,121 for the original ranks and 2,570,053 including descriptors. Even a perfect final queue schedule that saved every excess 8 KiB miss could eliminate only 119,673 modeled misses, while the descriptors add 899,932 distinct lines. This comparison already grants every second-phase descriptor load a hit. It is conditional on disjoint stream addresses and this effective cache size. When all bases start in the same set, the 32 KiB eight-way comparison is 1,751,801 original to 3,482,961 staged. With mixed 16-byte-aligned offsets it is 1,712,588 to 3,483,504. The single frozen example, Wikidata ordinal 1, gives the same direction. [Block-by-block replay](evidence/wikidata/locality.csv) · [Frozen example](evidence/frozen/summary.txt) · [Byte frequencies](evidence/wikidata/alphabet.csv)

The pressure from strip and counter stores, real Vec growth, prefetching, store throughput, hardware replacement, and inter-worker LLC behavior are unmodeled. A measured effective budget persistently near 1 KiB would change the locality decision, although extra arithmetic and storage would still need a native comparison.

## Reproduce

Use Rust 1.99.1 and the scorer's verified Wikidata corpus (SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`). The frozen compressed block has SHA-256 `e3bf5372b15885da138764a25bb2b7cd784ee3cbee1e664937647921ae3122b0`. Rewrap it with the earlier frozen-block driver. The model harness uses the research-only run-record accessor and the unchanged edge admission predicate.

```sh
study=research/astra-e/queue-locality
bash "$study/run.sh" corpora/wikidata.bz2 /tmp/astra-scatter
python3 "$study/report.py" /tmp/astra-scatter
python3 docs/journal/2026-11-18-open-return/frozen.py corpora/wikidata.bz2 /tmp/astra-scatter/frozen.bz2
/tmp/astra-scatter/locality /tmp/astra-scatter/frozen.bz2 /tmp/astra-scatter/frozen
python3 "$study/report.py" /tmp/astra-scatter/frozen
```

The property checks compare independent ordinary LRU simulations against the stack-distance and set counters, and exercise marked singleton handling across capped records. Paths and checksums in the archived CSVs support reproduction without timing the scorer. [Replay harness](locality.rs) · [Block geometry](evidence/wikidata/blocks.csv)
