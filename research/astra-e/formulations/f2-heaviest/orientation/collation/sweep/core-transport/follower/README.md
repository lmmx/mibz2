# A physical token BWT that can feed second induction directly

The moved cut section V supports a complete ordinary token BWT. Its outgoing spelling carries the current positive core C and the **next** original source's prefix P. Its incoming symbols and source bases can be constructed from the untyped old-destination core RLE and the prefix atlas alone. The existing Rust second induction consumed this object successfully on all eighty tested Wikidata section/depth pairs and returned 71,999,320 correct original marked bytes in total. [Exact constructor and independent partner checks](physical.py) · [Native ABI receipt](native-gate.csv)

## What the alphabet must distinguish

Write g(s) for the physical ordinal selected along the old arc from s∈U, and D(v) for the old destination reached from v∈V by its positive core C. Both g and D are bijections. The physical return sends v to g(D(v)) and spells C(v) followed by P(D(v)). The old core theorem puts all equal C into one contiguous V source bucket and orders their D(v) increasingly. It does not guarantee that g(D(v)) increases across the entire bucket. Among consecutive entries of this bucket sharing the same follower prefix P, advancing by its fixed length is strictly order-preserving under Ψ along identical bytes. Hence their physical destinations do increase. Splitting the bucket whenever P changes gives one exact, stably matched ordinal per resulting interval. Each carries an ordinary positive spelling. A zero-length prefix causes no special output case because its preceding core is positive. [Original core theorem](../) · [Independent first-hit interval proof](../audit/)

Three precise minima measure this cost. With core identity fixed and arbitrary external payload, each descent of g(D(v)) forces a split and splitting at all descents suffices. With one fixed spelling per symbol and core identities retained, every follower prefix change forces a split and splitting there suffices. With only a fixed spelling required, group consecutive equal physical words in V source order across core boundaries as well. Stability follows because Ψ over a single fixed word preserves source order. No two adjacent different spellings can use one fixed-spelling symbol. Identical spellings occurring in separated groups may still require distinct ordinals. All three results concern an ordinary token alphabet with contiguous buckets in the prescribed physical rank order. An alternating transport through two ranked sections can use a different representation.

## Constructing the column and transferring the mark

Each old incoming core run dispenses one consecutive slice from its core's source bucket. Sweep that run list against the prefix atlas in original U source order. On an intersection allocate a new ordinal when the previous follower prefix of that core changes, storing its start slot as the source base. Append that ordinal and width to the receiving atlas band's buffer, whose final position in physical V order is known. Concatenating the buffers in that order and coalescing adjacent equal ordinals gives the physical incoming column. There are at most R_core+B−1 intersections of the two U partitions, where B is the number of prefix bands. Discovery and atlas construction are upstream costs. Given both orders the sweep costs O(K_core+B+R_core) plus the resulting representation and spelling storage. The reference comparison-sorts the old atlas order and core bases. It prices at least one partial-run destination per intersection during scatter. The verifier alone enumerates every physical occurrence. [Implementation](physical.py)

To enter from an arbitrary original primary, follow original LF until a predecessor lies in V, accumulating the number of bytes skipped after that physical source. Use its rank in sorted V and that byte skip in the ordinary second writer. A cycle that closes without reaching V uses the original marked-cycle fallback. Exhaustive testing also reconstructed from the first forward old-source visit with an explicit prefix prelude. Both conventions retain the n-byte length and handle a final partial word or partial lap.

## Evidence and paid decision

All 22,520 small interval/depth objects passed, with 63,496 individually checked physical partners and 145,448 marked reconstructions. There were 2,778 objects requiring a core stability split, 3,302 where a fixed payload required more splits than stability alone, 648 where the same physical spelling recurred in separate minimal buckets, and 30 admitting a cross-core merge. The small receipt preserves the first witnesses. On every real row the byte spelling, physical successor, ordinary LF, and marked stream were compared with independent direct navigation. The independent native adapter recovered every real marked stream with second induction selected on all eighty. It required at most 83 backward original hops for the mark. [Small witnesses](small.json) · [Independent native mark and checked writer](native_gate.py)

| Section and depth | Original types | Physical types | Original runs | Physical runs | Largest ordinal mass summed over blocks | Both size gates |
|---|---:|---:|---:|---:|---:|---:|
| Frequent 0 | 38,648 | 38,648 | 66,373 | 66,373 | 31,358 | 10/10 |
| Frequent 4 | 38,648 | 40,250 | 66,373 | 68,529 | 31,357 | 10/10 |
| Frequent 8 | 38,648 | 42,903 | 66,373 | 70,696 | 13,341 | 10/10 |
| Frequent 12 | 38,648 | 40,598 | 66,373 | 73,804 | 12,867 | 6/10 |
| Quote 4 | 35,117 | 44,227 | 84,317 | 100,858 | 63,788 | 0/10 |
| Quote 8 | 35,117 | 45,527 | 84,317 | 105,060 | 180,744 | 0/10 |
| Quote 12 | 35,117 | 46,375 | 84,317 | 110,256 | 193,643 | 0/10 |

For frequent h8 the separate core plus prefix dictionaries totaled 1,639,676 bytes. Materializing one full spelling per ordinal used 2,028,895 bytes versus 1,926,457 for the old words. Its smaller largest bucket can reduce second-level residual links but can also increase the lengths of second words. The preregistered h8 frequent size gate passed on all ten tapes and triggered a native paid comparison, including the cost of both dictionaries, the scatter, second induction, output and peak storage. [Pre-data plan](PLAN.md) · [Frequent geometry](frequent.csv) · [Quote geometry](quote.csv) · [Native writer interface and fallback](../native/ORDINAL-INTERFACE.md)

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/follower
python3 "$d/physical.py" | diff - "$d/small.json"
python3 "$d/physical.py" research/astra-e/dense/target/wikidata frequent | diff - "$d/frequent.csv"
python3 "$d/physical.py" research/astra-e/dense/target/wikidata quote | diff - "$d/quote.csv"
bash research/astra-e/formulations/f2-heaviest/orientation/seams/async/native/token-writer/build.sh /tmp/astra-token-all
python3 "$d/native_gate.py" /tmp/astra-token-all/libtoken.so research/astra-e/dense/target/wikidata | diff - "$d/native-gate.csv"
```
