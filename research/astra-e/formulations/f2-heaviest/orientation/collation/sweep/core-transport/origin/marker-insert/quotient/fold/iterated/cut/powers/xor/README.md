# Exact coloring without a successor graph

The marked weighted decoder can select exactly the original greedy cut in two scans of each destination bucket. For a color decision, one scalar pending vote already contains the edges to earlier destination buckets. Reading the current incoming fragments supplies every earlier-colored source, and rescanning sends signed votes to future sources. The singleton marked source is decided first. This removes both adjacency lists and leaves the old cut, all source partners and every grammar node unchanged. The working bound for H overlaid fragments and K types is O(H+K). Adjacent unselected destination classes in one outside source may additionally share one ordinal alias while their physical successors stay in order. [Exact coloring proof](STREAM.md) · [Gap proof](GAPS.md) · [Native source](stream.hpp)

A second graph-free route collects weights by XOR of source and destination type and chooses a parity cut. A bit-greedy linear histogram or exact Walsh transform certifies at least a quarter of the off-diagonal mass, with the heaviest nonzero difference class supplying a stronger matching bound. These cuts change the contraction history. Across ten original blocks the bit-greedy and exact histogram arms take 385 and 363 passes. The unrestricted graph-greedy coloring takes 251. [Proof and algorithm](PROOF.md) · [Heavy-matching certificate](HEAVY.md) · [Per-depth accounting](depths.csv)

The native differential audit checks every exact color, every published incoming run, ordinal dictionary, protected mark and grammar node against the graph-backed construction. It covers all 21,324 marks of all 3,279 ternary columns through length 7, all 251 original-block steps and three mandatory normalization families. It also checks marked output for the two Walsh selections and both gap variants against the dense stable permutation. Independent weighted and libbzip2 checks of bit-greedy, best Walsh and bit-greedy with gaps checked 54,942,371, 50,279,231 and 54,705,995 intermediate partners respectively on the ten actual blocks. [Native verifier](audit.cpp) · [Native receipt](audit.json) · [Bit-greedy receipt](four/greedy.jsonl) · [Walsh receipt](four/exact.jsonl) · [Alias receipt](five/gaps.jsonl)

Three separate matched single-block panels start from maximal runs and the true marked rank. Each row below compares arms from the same panel with Student intervals on paired thread-CPU log ratios. The graph-free colored stream retained the 26 passes of the original graph path and saved selection work. During the final panel, graph-backed compact selection took a median 14.11 ms, exact streaming selection 4.37 ms, and the compact publication itself 8.07 and 8.87 ms respectively. These phase medians summarize individual calls and need not sum to the paired total. [All single-block statistics](summary.json) · [Recompute](report.py)

| Panel | Numerator / denominator | Rounds | Ratio and 95% interval |
|---|---|---:|---:|
| Four-arm | Bit-greedy / graph | 24 | 0.676 [0.654, 0.698] |
| Four-arm | Exact Walsh / graph | 24 | 0.715 [0.672, 0.762] |
| Five-arm | Bit-greedy / graph with compact publication | 30 | 0.756 [0.703, 0.812] |
| Five-arm | Bit-greedy plus gaps / bit-greedy | 30 | 0.943 [0.822, 1.081] |
| Stream | Exact stream / graph with identical compact publication | 30 | 0.599 [0.575, 0.624] |
| Stream | Exact stream plus gaps / exact stream | 30 | 0.902 [0.846, 0.962] |
| Stream | Exact stream plus gaps / dense stable walk | 30 | 0.839 [0.768, 0.916] |

## Ten-block calibration from the actual capped records

All six arms start from the original capped Rust run stream. C++ pays maximal-run assembly, byte relabeling, all cut construction and weighted expansion. References use their frozen native run entry points. Across the ten blocks, 1,440 recorded calls matched 8,999,915 marked inverse bytes per six-arm sweep, with the earlier libbzip2 and CRC receipt covering 8,999,785 post-RLE1 bytes. The six arms share a single process for CPU pairing and retain their usual scratch policy. The absolute values below are medians of 24 equal-work ten-block totals. Process peaks come from three fresh invocations for each block. Each process decodes one independently recovered compressed block through the same frontend. [Fixed protocol](capped/PLAN.md) · [Raw CPU](capped/times.csv) · [Per-block rates and verified hashes](capped/receipt.json) · [Raw memory](capped/rss.csv)

| Complete local inverse | Median CPU over ten blocks (ms) | Median / maximum single-block VmHWM (KiB) |
|---|---:|---:|
| Exact stream | 248.75 | 5,822 / 10,464 |
| Exact stream plus gaps | 242.06 | 5,680 / 10,080 |
| Dense stable walk with paid run assembly | 226.93 | 7,950 / 8,916 |
| Frozen scalar-cap | 33.87 | 4,700 / 6,024 |
| LazyCells | 36.43 | 4,444 / 9,168 |
| OpenGated | 34.78 | 4,844 / 9,224 |

The gap variant takes 6.977 [6.504, 7.483] times scalar-cap, 6.552 [6.238, 6.882] times LazyCells, 6.571 [5.853, 7.377] times OpenGated and 1.042 [0.998, 1.088] times the simple dense inverse. Its separate single-block advantage over the simple inverse does not predict a production-kernel gain. The ratio to scalar exceeds 6.4 on every tested block. No calibrated all-worker or target benchmark is warranted for the complete grammar construction. A separate early-stop study tests how much initial contraction one could buy before handing the weighted residual to a different decoder. [Early-stop plan](early/PLAN.md)

## Reproduction

Generate the ten checked input.txt trees with the native commands in the parent powers README. The graph-free binary accepts check, check5 and checkstream. The panel modes panel, panel5 and panelstream take the frozen input.txt and a repetition count. Fresh single-arm invocations take graph, graphcompact, greedy, exact, gaps, stream, streamgaps or direct with one repetition. The Python selector runs accept greedy, exact or gaps, the ten original tape directory, compressed corpus and a fresh output directory. [Original-block procedure](../README.md) · [Selector driver](panel.py)

The actual-cap calibration used the frozen scalar worktree and scorer projection already identified in the route-pricing build manifest. Build the native original corpus extractor and controls as follows from the repository root, with the Rust 1.99.1 build environment available. Set `s` to an unused work directory. The lower-level source hashes, repository pin, tool and binary hashes are in the build receipt.

```bash
x=$(dirname "$(rg --files | rg '/cut/powers/xor/chooser.py$')")
s=$(mktemp -d)
bash research/astra-e/results/scalar-route-pricing/build.sh "$s/refs"
python3 "$x/capped/build.py" "$s/refs/scalar" "$s/refs/scalar-build/release" "$s/capped"
taskset -c 1 nice -n 5 "$s/capped/panel" corpora/wikidata.bz2 "$s/capped" 24
python3 "$x/capped/receipt.py" "$s/capped" corpora/wikidata.bz2 "$s/refs/scalar"
taskset -c 1 nice -n 10 python3 "$x/capped/rss.py" corpora/wikidata.bz2 "$s/refs/scalar-build/release" "$s/capped"
python3 "$x/report.py" | diff - "$x/summary.json"
```
