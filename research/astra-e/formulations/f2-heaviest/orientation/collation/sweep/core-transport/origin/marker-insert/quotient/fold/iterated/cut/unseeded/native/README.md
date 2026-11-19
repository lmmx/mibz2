# An arbitrary-color native inverse with an ordinal mark

The two seeded controls both carry their mark through the same mark-independent coloring and cheaper-orientation rule. A singleton type and an ordinal agree on all 250 observed cuts. The unsplit column takes a slightly different sequence of cuts, with 1,149,624 total refined IDs versus 1,160,473 on the seeded histories. All ten unsplit final offsets, lap lengths, passes and recovered block outputs agree with the independent Python computation. Initial seeds use the donor's mandatory source split. All three arms use the same compact alias publisher, balanced coloring, power constructor and grammar expansion. The protected-first historical coloring is a different trajectory.

| Complete original inverse comparison | Local thread CPU ratio, 95% cluster interval | Local wall ratio, 95% cluster interval |
|---|---:|---:|
| Seeded ordinal / seeded singleton | 0.9947 [0.9693, 1.0208] | 0.9974 [0.9620, 1.0340] |
| Unseeded ordinal / seeded ordinal | 0.9881 [0.9521, 1.0254] | 0.9847 [0.9437, 1.0274] |
| Unseeded ordinal / seeded singleton | 0.9829 [0.9338, 1.0345] | 0.9821 [0.9292, 1.0380] |
| Unseeded ordinal / full dense stable inverse | 0.9404 [0.8867, 0.9973] | 0.9327 [0.8740, 0.9954] |

The first three intervals cross parity. Maximum post-exec RSS across the ten one-arm processes was 10,128 KiB for the unseeded route, 9,104 KiB for each seeded route and 8,020 KiB for dense. Raw receipts include 1,600 complete, byte-checked stage outputs and 40 fresh RSS processes on local Xeon Platinum 8573C, CPU 3, nice +12. Clocks start at the original run tape and end after materializing the full marked pre-RLE1 word. Clocks include local allocation and destruction of temporary grammar state, seeded construction or initial column copy, all checking of the stopping condition, and every normalization actually taken. Destruction of the returned dense byte vector, input parsing, Huffman/MTF, RLE1, CRC and validation are outside the clocks. The timing process keeps all ten original RLEs and dense expected outputs resident to validate every timed sample. Fresh RSS processes load one tape and execute one arm. Local native admissibility, full source CRC validation and target workstation throughput remain separate measurements. [Frozen comparison](PLAN.md) · [Raw times](pairs.csv) · [Per-tape RSS](rss.csv) · [Paired summary](summary.json)

## Review of the marked power bound

For constant outgoing type c, suppose x and LF(x) both have incoming label c. Among c-labeled rows, LF is strictly increasing. Hence x < LF(x) implies LF(x) < LF²(x), and the symmetric implication holds for decreasing steps. Equality is a fixed row and has already terminated. A bulk jump on a constant-displacement raw run leaves that run in the same direction, and its visited coordinate remains inside the source bucket for c. Starting at the raw run found by the ordinary frequency pass, a cursor moves monotonically and crosses at most R−1 old boundaries **in total**, including boundaries crossed without a marked visit. Thus after the mandatory normalizer’s raw index exists the extra mark search costs O(R) worst-case time and O(1) words. It needs no per-jump binary search. The retained ordinal can be captured in the existing mismatch-range enumerator under the same stable order. The native limit here is 900,000 rows; the independent unbounded integer test in the parent exercises more than four billion old rows. [Core transport](transport.hpp) · [Marked witnesses](tiny.json)

For an ordinary publication, the frequency pass captures the true incoming rank and run of x. The normal bucket-splitting loop immediately identifies its refined slice by the difference between the source class of x and the class at the start of the raw run. When the retained representative changes to LF(x), its bucket is on the source side and contains no deleted rows. Otherwise the deleted prefix inside its own bucket is available in the existing per-target deleted counter just before classifying that slice. Summing deleted counts before the representative's bucket is fused into the mandatory rem discount. No additional run or bucket pass is needed for that sum and no rank sample array is built. The native small oracle checks 66,712 marked cases separately in all three grammar arms. Among the unseeded cases there were 21,772 power passes, 16,618 shared fixed types, 55,362 ordinary moves and 33,996 nondividing horizons. All ten real RLE1 outputs independently match libbzip2 and its block CRC. [Per-block identities](real.json) · [Every pass](trace.csv)

## Reproduction

The native implementation and correctness gates are frozen at d33bcaffe0000292345cf84c823a2b523cdd7409. The compiler is g++ 14.2.0 and the measured executable SHA-256 is bed3f8bb59c4a1a6c64b03c7ec367830ca0c92672fa3f49fc2d2467722a1b377. Build reads donor code by its immutable commit. A full block scorer, multi-worker CI and i9 have not been run on this research-only stage.

```bash
d=$(dirname "$(rg --files | rg '/cut/unseeded/native/build.py$')")
tmp=$(mktemp -d)
python3 -B "$d/build.py" "$tmp/build"
taskset -c 3 nice -n 15 python3 -B "$d/check.py" "$tmp/build/run" research/astra-e/dense/target/wikidata corpora/wikidata.bz2 "$tmp/inputs" "$tmp/receipts"
diff "$tmp/receipts/tiny.json" "$d/tiny.json"
diff "$tmp/receipts/real.json" "$d/real.json"
diff "$tmp/receipts/trace.csv" "$d/trace.csv"
taskset -c 3 nice -n 12 python3 -B "$d/measure.py" "$tmp/build/run" "$tmp/inputs" "$tmp/receipts"
python3 -B "$d/report.py" "$tmp/receipts"
python3 -B "$d/report.py" "$d" | diff - "$d/summary.json"
```
