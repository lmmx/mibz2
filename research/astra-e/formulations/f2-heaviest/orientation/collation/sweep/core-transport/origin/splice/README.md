# First return after sparse physical cuts in an ordinary weighted token BWT

Given the original token object on the rank interval J and a sorted list of holes, the new constructor publishes the ordinary token BWT on H=J∖E using two incoming-run sweeps and sparse chain corrections. It preserves byte ranges for unchanged source buckets and creates one new contiguous compound word for each live source that enters a deleted chain. It transports a mark inside any constituent word, including a deleted one. A mark on a deleted-only cycle uses the old J object.

If q surviving sources enter holes, then **K_H ≤ K_J+2q** and **R_H ≤ R_J+3q**. Both constants are attained simultaneously for every q, even on one primitive cycle. The splicer never allocates a token-per-source or token-per-destination table. The unchanged second-stage consumer accepted every primary and extended real case. The five affected primary cases took 1.2230 [1.1392, 1.3129] times the thread CPU of consuming their already-built J object. [Construction and proof](PROOF.md) · [Sharp witnesses and refined run bounds](bounds/README.md) · [Source](splice.rs)

## Why both stable maps survive

Each incoming run translates consecutive occurrences in one source bucket into consecutive physical destinations. The first sweep locates the successor and the predecessor of every hole in that run. Starting from each live predecessor and following these sparse successors identifies all changed chains, their final physical landing slots and every intra-chain byte offset needed by a mark. Unvisited holes lie in deleted-only cycles. The second sweep drops old hole destinations, reassigns each surviving landing, and emits ordered aliases for stretches of unchanged sources. Stable occurrence order within every published bucket gives the exact induced successor.

Deleting destinations cannot increase run count. Along the old occurrence stream of ID a, its unchanged source stretches appear in increasing source order. At most one old run can split for each boundary between those stretches. Reinserting a changed singleton at its physical landing adds at most two runs. In particular an arbitrary number of deleted-only cycles does not force a new compound word. The sharp single-cycle family has incoming column bᵐaᵐ⁺¹, m=2q+1, successor i↦i+m mod (2m+1), and holes m+2j for 1≤j≤q. The changed retained sources are 2j and their surviving landing slots are 2j−1. These worst cases concern arbitrary hole choices. [Constructive proof and deleted-mark counterexamples](bounds/README.md)

For a flat standalone output with independently owned base, word-range and run arrays, publishing the arrays requires writing K_H+R_H records. The linear passes charge for that interface. The e=0 identity case borrows the entire input object. A consumer capable of interpreting sparse changes directly could have a different cost. This result does not require the first constructor to discover J or the holes for free. Their earlier construction, recognition, dictionary materialization and the original byte-level decoder remain outside this interface. [Source of the fixed hole sets](../holes.py)

## Exact checks

The Rust sparse implementation passed 27,014 independently induced finite instances, verifying all 85,273 live source successors/spellings and 431,542 marked offsets. The marked cases included 128,507 starts on deleted vertices that transported to live owners, and 87,264 marks requiring the complete old-J cycle fallback. The inputs include aliases that already share the same bytes, reversed numeric source ID order, no holes, all holes, multiple LF cycles and truncated repetition. An independent implementation checked 27,268 cases, 262,534 marked byte offsets, 1,296 equal-payload alias cases and the sharp families. [Sparse implementation checks](small.txt) · [Independent exhaustive receipts](bounds/)

The primary real panel used the ten original physical BWT tapes with the frequent h8 and quote h12 policies. The 616 KiB tape pack and regeneration program reconstruct the old J token objects and independently walk the new H section in the original byte row space. Every physical H successor and complete outgoing word, both marked offsets and both native byte outputs were verified. The protocol compares 17,999,830 original marked bytes over these twenty objects. [Reproducible source tapes and fixture format](real/README.md) · [Native gate](checked.csv)

| Policy | Holes | Changed live sources | Original J mass | H mass | Appended compound bytes |
|---|---:|---:|---:|---:|---:|
| Frequent h8 | 9 | 8 | 31,367 | 31,358 | 3,391 |
| Quote h12 | 23 | 15 | 299,698 | 299,675 | 1,262 |

Five primary cases had nonempty E. On those cases original source IDs fell from 20,002 to 19,988 and incoming runs from 29,872 to 29,858. On the other fifteen cases the borrowed J object already was H. A 60-object extension at h4, h8 and h12 checked 964,569 retained partners. Thirty cases had holes, totaling 28,625 holes and 28,367 changed live sources. On those cases labels increased from 106,717 to 116,924 and runs from 175,858 to 180,255. The existing native second induction accepted every J and every H. The test oracle independently uses dense arrays to verify every physical row. These arrays are excluded from splicing and from the paid comparisons. [Extended geometry and ABI results](extended.csv) · [Per-policy totals](summary.json)

The direct native producer uses the trailing-marker spelling convention, which rotates every cylinder return word left by one byte. Its mark incorporates that byte shift, including moving to the previous word on a zero natural skip. The splicer accepts the supplied coherent convention. On all twenty primary native-convention objects it recovered the independently rotated H successors, words and marked bytes. Finer producer identity checks compare the actual source bases, dictionary slices, runs, mark and output exported by the original direct-native kernel with the rotated fixtures. [Rotated native gate](rotated.csv) · [Producer reconciliation](real/native-match.csv)

## Charged native costs

A 24-round, four-arm thread CPU comparison used the preconstructed J objects and fixed hole lists. Each arm occupied every position six times per case. The measured interval includes sparse topology, compound copying, the already-existing consumer, materialized marked output and release of splice metadata. The controls consume J under a twin label and also splice without using H. The five affected sparse patches together took a median 763 µs per lap before the second-stage call. Across all twenty, splice plus H consumption divided by J consumption was 1.0790 [1.0139, 1.1483]. Against a control paying the same splice before consuming J, the ratio was 1.0218 [0.9700, 1.0764]. Native second-stage compatibility and controlled growth are established; end-to-end decoder throughput remains unmeasured for this splice. [All 1,920 observations](paid.csv) · [Paired results and controls](summary.json) · [Frozen plan](plan.md)

The original append-as-you-trace implementation, frozen in `01c9dcc`, accepted all cases. In the cold correctness replay every affected primary case moved its original dictionary backing. Across those five objects the exact-fit arena capacities rose from 2,709,756 to 5,419,512 bytes even though only 4,653 bytes were required. The alternative in `6e56121` creates a temporary list of at most e+q original spelling ranges while following the same chains once, reserves the exact final extension, then copies the ranges. It produces byte-identical metadata and requires 2,714,409 bytes across those same arenas. On all thirty affected extended cases the reserved capacity is 10,950,018 bytes versus 19,505,014 bytes originally, with 1,197,511 appended bytes. These are summed dictionary capacities, excluding the splice metadata and native consumer's workspace. [Exact reservation and marked parity](reserved-primary.csv) · [Expanded comparison](reserved-extended.csv)

A separate 24-round comparison over just the five affected primary objects paid the complete patch from both freshly prepared exact-fit and reusable arenas. Exact reservation divided by incremental growth was 0.8461 [0.6008, 1.1916] cold and 0.8984 [0.5863, 1.3767] warm. Both policies moved the pointer in 72/120 cold calls and zero warm calls. These intervals leave their CPU ordering unresolved. The exact-capacity improvement is preserved independently of the local CPU result. [All 480 allocation observations](allocation.csv) · [Allocation replay](allocation.json)

## Reproduction

Use the token-writer shared library from the existing ABI recipe, with its Rust compiler and dependency record. The archived run used Rust 1.99.1. The first compiler command below generates a short independent verifier and the native ABI executable. The inputs contain the original marks and the patch program derives its own H mark. For the frozen earlier paid source instead check out `01c9dcc`. The extended and rotated fixture generators are described in the real-data directory. [Shared library build](../../../../../seams/async/native/token-writer/build.sh) · [Binary and source hashes](manifest.txt) · [Fixture regeneration](real/README.md)

```bash
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/origin/splice
out=/tmp/astra-splice-build
bash "$d/build.sh" "$out" /path/to/libtoken-directory
"$out/small"
mkdir -p /tmp/astra-splice-primary
python3 "$d/real/generate.py" "$d/real/sources.xz" /tmp/astra-splice-primary > /tmp/astra-splice-primary/cases.csv
"$out/panel" check /tmp/astra-splice-primary
"$out/panel" reserved /tmp/astra-splice-primary
nice -n 16 taskset -c 3 "$out/panel" price /tmp/astra-splice-primary > "$out/price.csv"
nice -n 16 taskset -c 3 "$out/panel" alloc /tmp/astra-splice-primary > "$out/allocation.csv"
python3 "$d/report.py"
python3 "$d/allocation.py"
```
