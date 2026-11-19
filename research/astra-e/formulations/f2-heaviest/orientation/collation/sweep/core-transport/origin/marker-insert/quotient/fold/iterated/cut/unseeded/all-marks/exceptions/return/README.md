# Recovering modal exception clocks directly from physical LF

Inducing the physical LF first-return on the exceptional rows and the original primary recovered every marked event time and its physical exception identity for all 30 sampled block/context pairs. At depth 8 it took **84.19 ms** per ten-block lap, compared with **160.83 ms** for a dense stable LF clock that produced identical output. An earlier ten-block measurement put scalar-cap's complete inverse at **24.15 ms**. This constructor reduces the cost of acquiring a clock while exceeding the entire measured inverse budget. It also increases peak process memory. Neither constructor was promoted to an experimental worker. [Paired clock data](native/summary.json) · [Independent scalar/terminal panel](../price/README.md)

## What induction eliminates

Each maximal physical L run gives a stable LF translation. Pull the borders of the set of anchors back through those translations once. Contiguous anchors advance as a packet until their destinations disagree on run or return membership. Every physical row on a cycle touching the anchor set participates once. A fixed source border splits at most one packet over the entire construction, and a depth-first, lower-fragment-first schedule produces source-ordered translations without sorting. Forward-horizon output follows by phase shift even for periodic blocks and foreign LF cycles. An explicit two-run family shows the packet work can nevertheless be linear in block size when the final return map has just one entry. The constructor uses neither an ordered clock oracle nor a grammar for the bytes between exceptions. [Proof and counterexamples](PROOF.md) · [Structural preregistration](PLAN.md)

| Context depth | True events | Anchor intervals | Static source atoms | Packet translations | Translations per physical row | Singleton share | Actual splits | Merged return intervals |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 4 | 1,122,585 | 125,556 | 762,755 | 1,482,681 | 0.165 | 78.5% | 157,868 | 269,448 |
| 8 | 409,432 | 45,882 | 741,618 | 2,350,011 | 0.261 | 80.5% | 93,062 | 127,360 |
| 16 | 229,013 | 36,474 | 731,961 | 3,262,928 | 0.363 | 81.3% | 72,317 | 101,276 |

The same 8,999,915 rows were covered at each depth. Pulling back internal anchor borders accounted for 21,700, 18,183 and 14,605 actual splits. Adjacent static atoms admitted no further affine merge on these inputs. At depths 4, 8 and 16, the page-indexed maps still required 5.91, 7.75 and 9.08 million predecessor comparisons. A second predecessor search on the compact return intervals during the marked walk needed another 16.51, 5.59 and 3.05 million. The priced treatment therefore used one dense destination/distance pair per anchor after packet construction. Only the ten sampled indices 1, 41, 85, 134, 176, 225, 269, 313, 359 and 397 underlie these estimates. [Per-block geometry](blocks.csv) · [Aggregate counts](summary.csv) · [Input identities](inputs.json)

## Paid cost from matching inputs

Both arms receive byte-identical original 127-capped records, the true primary and sorted physical exception ranges. They recover (backward clock, sorted true-exception ordinal) in marked-cycle order and its original lap length. Direct LF packs the exception hint into its edge with a 256-row rank directory. The induced arm pays for re-coalescing, the two-pass interval overlay, packet queries, the dense return map, the marked walk and the output buffer. Both exclude the cost of recognizing physical exceptions and obtaining defaults, restart labels and final bytes. In the native preflight and replay, every output pair matched the independent dense LF and every event time and byte matched the historical oracle wire. The structural gate also tested every induced edge including its distance. Exhaustive binary physical columns, masks and marks through length six covered 61,896 layout/case combinations and all finite horizons from zero through twice the column length plus two. [Native gate](native/check.csv) · [Repeated gate](native/repeat-check.csv) · [Finite receipt](exhaustive.txt)

| Depth | Direct LF median ms | Induced median ms | Paired induced/direct, 95% interval | Direct max process KiB | Induced max process KiB | Direct / induced max vector bytes |
|---|---:|---:|---|---:|---:|---:|
| 4 | 169.09 | 85.70 | 0.507 [0.475, 0.542] | 6,988 | 9,108 | 4,722,048 / 6,947,336 |
| 8 | 160.83 | 84.19 | 0.535 [0.506, 0.566] | 6,108 | 7,320 | 4,027,232 / 5,320,436 |
| 16 | 161.85 | 92.05 | 0.576 [0.533, 0.623] | 5,928 | 6,936 | 3,867,656 / 4,957,156 |

Times are thread CPU on pin 2 of a shared local host, summed over ten blocks per arm/round. The study used 24 order-balanced paired laps for each depth, 1,440 raw calls, scratch reused across blocks, and a two-sided paired log-t interval using 23 degrees of freedom. Separate process high-water samples charged one input and its scratch per run. The vector maxima are capacities after that block's call in its fresh process. Resident process size includes stack and allocator effects. No independent CI or i9 comparison was made for either clock constructor. [Frozen cost protocol](native/PLAN.md) · [Source and executable identities](native/manifest.json) · [Raw times](native/time.csv) · [Fresh process sizes](native/rss.csv)

In the earlier prepared-model panel, scalar and the eight-byte terminal used median 24.15 and 14.87 ms at depth 8, and 25.05 and 17.21 ms at depth 16. Transferring those separate-panel medians as a planning assumption leaves only 9.28 and 7.84 ms for *all* missing acquisition work. This return constructor alone would have to become about 9.1 or 11.7 times faster at those depths. A combined same-process measurement would be needed to claim a total. The next useful distinction is whether a consumer can operate directly on a supplied physical return ring and its distance/payload without materializing chronological pairs. Measuring that conditional consumer against the already priced chronological terminal tests a genuine phase elimination. A material consumer gain would still leave acquisition of the ring and physical model to be paid. [Depth 8 absolute CPU](../price/k8-prepared/summary.json) · [Depth 16 absolute CPU](../price/k16-prepared/summary.json)

## Reproduce

From the repository root, rebuild the masks from the ten archived real tapes, reproduce the structural gate, and recover byte-identical original frontend records checked against the archived Rust donor's SHA-256. Use a fresh working directory for each replay.

```sh
d=$(dirname "$(rg --files | rg '/exceptions/return/PROOF.md$')")
w=$(mktemp -d)
mkdir -p "$w/positions"
taskset -c 4 nice -n 16 python3 -B "$d/panel.py" "$w/struct" "$w/models" \
    --tapes research/astra-e/dense/target/wikidata --positions "$w/positions"
cmp "$w/struct/blocks.csv" "$d/blocks.csv"
cmp "$w/struct/inputs.json" "$d/inputs.json"
cmp "$w/struct/exhaustive.txt" "$d/exhaustive.txt"
python3 -B "$d/report.py" "$w/struct/blocks.csv" | cmp - "$d/summary.csv"
python3 -B "$d/native/recover_caps.py" "$w/models" "$w/caps"
python3 -B "$d/native/run.py" "$w/paid" "$w/caps" "$w/models" build
taskset -c 2 nice -n 12 python3 -B "$d/native/run.py" "$w/paid" "$w/caps" "$w/models" price
python3 -B "$d/native/run.py" "$w/paid" "$w/caps" "$w/models" report
```

In the research workspace, reusing `/tmp/astra-clock-wires` as the second positional argument to `panel.py` skips wire generation. The original donor records reside at `/workspace/work/astra-powers-original/capped`. All ten recovered cap files were independently compared byte for byte with this donor.
