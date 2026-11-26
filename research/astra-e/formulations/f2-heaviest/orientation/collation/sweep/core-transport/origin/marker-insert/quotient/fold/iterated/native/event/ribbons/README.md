# Reconstructing the terminal grammar from affine ribbons

Both implementations obtained the same terminal grammar outcomes on all 228 full Wikidata admissions as the dense event reference, without constructing a successor or membership array per token. The ordered tree reduced fresh-process high-water memory from 9,484 to 8,136 KiB. Its full inverse took 4.3040 [4.1080, 4.5093] times the dense event CPU time. The flat implementation took 2.5790 [2.3677, 2.8092] times the dense time and used 18,916 KiB. These containers did not meet the throughput criterion. [Measured sources and gates at a5c1c8e](manifest.json) · [All paired samples](phase.csv) · [Recomputed summary](summary.txt)

**Broken affine adjacencies never heal.** A surviving source class's map is noncontracting in original coordinates. Composing two such maps cannot close a physical image gap. Products of maximal surviving ribbon intersections are therefore already maximal. There were zero coalescences among 12,624,466 generated pieces, and only 310,004 new internal breaks beyond the 12,314,462 selected source bands. The independent all-schedules checker compared 77,400 intersections in 75,858 possible events on 60,507 reachable column states and observed the same invariant. Its 10,785 input cases cover binary columns to length 10, ternary to 7 and quaternary to 6, including coincident labels. The frozen paid sources still contain the coalescence guard. [Proof and resource bounds](REPRESENTATION.md) · [Exhaustive result](strict-gaps.json)

The original named constructor supplies ordered source buckets, spelling leaves and incoming runs. Sources are identified by occurrence count and the initial 1,786,472 incoming runs yield 1,786,472 initial ribbons across the corpus. At every accepted class fold the writer splices out disjoint target intervals, publishes composed translations, updates the mark within its one affected interval and creates one immutable concatenation. Source and target storage and all intermediate work are charged. The sparse methods performed 12,624,466 intersections for 30,868,167 dense row edits, a reduction of 59.1%. They also performed 28,463,466 eligibility tests and 55,470,360 static-owner predecessor queries. Flat-vector cut splices displaced 597,641,635 trailing records by logical array position changes. The tree removed these shifts and paid tree navigation and node allocation. The largest published map had 19,680 ribbons, or 20,082 live records including publication scratch. Flat vectors retained high-water capacity separately for each discovery-order class across the sequence, reaching 12,014,400 bytes of table capacity despite the small live count. Logical payload estimates for tree nodes omit their unexposed capacity. The dedicated RSS processes include that cost. [Per-block flat](rss-4.csv) · [Per-block tree](rss-5.csv)

| Complete native stage | μs/block | Initial named stage | Sparse/dense index | Contraction | Output | Fresh-process VmHWM KiB |
|---|---:|---:|---:|---:|---:|---:|
| Named | 1,899 | 1,144 | 0 | 0 | 754 | 8,280 |
| Repeated grammar | 6,681 | 1,241 | 141 | 3,736 | 1,562 | 8,108 |
| Direct indexed walk | 2,602 | 1,179 | 217 | 0 | 1,204 | 7,380 |
| Dense events | 4,932 | 1,157 | 305 | 1,709 | 1,759 | 9,484 |
| Flat ribbons | 12,737 | 1,170 | 213 | 9,645 | 1,707 | 18,916 |
| Tree ribbons | 21,183 | 1,199 | 663 | 17,581 | 1,738 | 8,136 |

The complete flat/named and tree/named ratios were 6.7306 [6.3540, 7.1296] and 11.2324 [10.8206, 11.6597]. Across the 36 rounds and ten fixed inputs, all 2,160 timed calls produced exact output. Each arm occupied each position per block in six disjoint chronological clusters. Timing used thread CPU on CPU0 at nice +16. On the full production sequence both sparse backends checked 205,197,794 bytes against independent dense LF in both measurement and audit specializations. The dedicated HWM process used previously verified hashes without allocating an oracle. All 13,567 tiny marked cases passed. It also checked that every edge on every small event actually followed the dense two-edge bypass. These are native inverse results and do not establish full-worker throughput or i9 peak memory. [Dense check](dense.txt) · [Small results](small.csv)

## Reproduction

Use Rust 1.99.1, the capped input archives and the release library of the frozen dense predecessor. The builder verifies the inherited native control and restores the two frozen grammar sources directly from git. [Parent instructions](../README.md)

```sh
source /workspace/work/env.sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/origin/marker-insert/quotient/fold/iterated/native/event/ribbons
out=/tmp/astra-terminal-ribbons
dst=/tmp/astra-ribbon-results
taskset -c 0 nice -n16 bash "$d/gates.sh" "$out" "$PWD/target/release/deps" "$dst"
taskset -c 0 nice -n16 "$out/ribbons" price /tmp/astra-clipped/ten.bin > "$dst/phase.csv"
python3 "$d/report.py" "$dst"
python3 "$d/strict_gaps.py"
```
