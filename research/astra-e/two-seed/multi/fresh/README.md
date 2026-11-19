# Constructed tables reduce the measured four-lane advantage

On the 14-block panel, four-forward/scalar emitter CPU was 0.7636 [0.7309, 0.7978] when both read separately retained tables and 0.9043 [0.8584, 0.9525] when each reconstructed its own table immediately before the identical emitter function. The four-by-fresh interaction was **1.1842 [1.1128, 1.2601]**. Using fresh tables reduced ordinary single-chain emitter time substantially, particularly on the more numerous ordinary admissions. This experiment identifies an actual dependence on construction order and retained table state. It does not measure which cache level supplied each load. [All 4,480 paired samples](paid.csv) · [All effects and same-code replicates](summary.json)

| Corpus subset | Retained-table emitter, four/scalar | Fresh emitter, four/scalar | Fresh construction + emitter, four/scalar |
|---|---:|---:|---:|
| All 14 | 0.7636 [0.7309, 0.7978] | 0.9043 [0.8584, 0.9525] | 0.9547 [0.9184, 0.9924] |
| Nine original ordinary admissions | 0.8468 [0.7963, 0.9005] | 1.0190 [0.9506, 1.0923] | 1.0333 [0.9862, 1.0827] |
| All five extra dense admissions | 0.7056 [0.6573, 0.7574] | 0.8223 [0.7592, 0.8907] | 0.8854 [0.8460, 0.9266] |

During the fresh path, the geometric mean 14-block emitter was 41.51 ms for scalar and 37.54 ms for four. The same functions on separately retained tables took 71.65 and 54.71 ms. The fresh constructor ratios were 1.0010 [0.9551, 1.0491] on all 14, 1.0442 [0.9901, 1.1013] on ordinary and 0.9509 [0.8936, 1.0119] on dense. The earlier panels represented all five extra dense admissions together with just nine of 228 ordinary admissions. Computing a true-corpus saving requires their actual cumulative share of complete decoder CPU. The broad panel stage gain cannot be applied uniformly to the remaining ordinary blocks.

Every fresh path runs the scalar-cap constructor from its original capped records on each call, uses its actual table and spelling arrays directly, and confirms every emitted byte with an independent full-table Segmented inverse. The retained path keeps one intact constructed kernel per block and reads the same emitter entry. Original and four-forward implementations both have identical-code labels. One logical Xeon CPU supplied thread time. Edge-table copy, entropy parsing, MTF/RLE2, final RLE1, stream scheduling and CRC costs are outside the measured stages. A subsequent comparison may use the original post-front density admission as an independent, already known selector. [Frozen protocol](PLAN.md) · [Code](main.rs)

```sh
d=research/astra-e/two-seed/multi/fresh
tmp=$(mktemp -d)
python3 research/astra-e/edge-displacements/build.py "$tmp/support"
python3 "$d/build.py" "$tmp/test" "$tmp/support/libmibz2.rlib"
taskset -c 0 nice -n 16 "$tmp/test/panel" corpora/wikidata.bz2 > "$tmp/raw.csv"
python3 "$d/report.py" | cmp - "$d/summary.json"
```
