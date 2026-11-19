# Four forward arcs reconstructed in one allocation

The four-forward writer used **0.8324 [0.8071, 0.8585]** of the original edge emitter's CPU, and **0.9235 [0.8907, 0.9576]** of the preceding two-forward writer's CPU on the prespecified 14-block local panel. Three forwards measured 0.8770 [0.8451, 0.9100] over original and 0.9730 [0.9387, 1.0084] over two. All four same-code label intervals include parity. Each pair compares full emitter CPU over 40 position-balanced rounds, with exactly 4,480 measured invocations. Each invocation reconstructed the complete output and checked it outside the timed region. [Raw CPU observations](paid.csv) · [Complete intervals and strata](summary.json)

Each exact first-return arc writes forward into the original output allocation. Newly completed lengths release unused reservation bytes to the remaining arcs. Ordered in-place moves preserve completed and paused prefixes. Final fragment rotations order the marked cycle, including when some seeds belong to unrelated cycles. The sum of the true remaining positive weights bounds total remaining output space. At least one unfinished arc therefore fits each pass and k forward arcs finish in at most k passes. A four-row, length-113 example with outgoing word lengths 1, 33, 38 and 41 attains four passes. [Proof and plan](PLAN.md) · [Writer](writer.rs) · [Synthetic tests and tight witness](driver.rs)

| Preserved writer | Successor issue steps / retained edges | Relocated bytes during reservations | Final compaction | Bytes in final rotations | Total movement / n |
|---|---:|---:|---:|---:|---:|
| Two forwards | 0.7555 | 52,763,621 | 0 | 0 | 0.2516 |
| Three forwards | 0.6256 | 132,597,946 | 0 | 78,539,585 | 1.0069 |
| Four forwards | 0.5396 | 169,526,539 | 0 | 141,536,791 | 1.4834 |

The structures covered all 26,567,581 edge rows on all 233 Wikidata admissions, with 209,697,662 pre-RLE bytes. The byte totals for rotations count the full range participating in each in-place rotation. They do not predict cache transfers or physical store instructions. Ordinarily invisible byte movement is included in the native measurements. The three-lane worker used two passes on 147 blocks and three passes on 86. The four-lane worker used two passes on 82 blocks, three on 148 and four on three. Each actual output and all persistent output capacities matched the frozen single- and two-forward decoders. The independent Segmented decoder and the 322,553 synthetic outputs cover complete bytes, split cycles, long words, all marks in permutations through seven and the pass-count witness. [Per-block work and output capacities](all.csv) · [Real checks](gate.txt) · [Synthetic receipt](fixtures.txt)

The native algorithm at `8b744a7` recovers the known edge table from `badb2ae` and specializes for one, two, three or four active lanes. The incumbent two-forward kernel is reproduced side-by-side from its measured source. The new complete entries pair the four-forward emitter separately with the original edge constructor and the independent scalar-cap admission/control at `d0a4298`. Their correctness, resource consumption and full worker timing require measurements through the actual compressed input. [Complete-worker generator](build_worker.py)

From the repository root with Rust 1.99.1 and the frozen input, reproduce the native results as follows.

```sh
d=research/astra-e/two-seed/multi
scratch=$(mktemp -d)
python3 research/astra-e/edge-displacements/build.py "$scratch/support"
python3 "$d/build.py" "$scratch/test" "$scratch/support/libmibz2.rlib"
"$scratch/test/panel" synthetic
"$scratch/test/panel" gate corpora/wikidata.bz2 > "$scratch/all.csv"
cmp "$d/all.csv" "$scratch/all.csv"
taskset -c 3 nice -n 16 "$scratch/test/panel" price corpora/wikidata.bz2 > "$scratch/paid.csv"
python3 "$d/report.py" | cmp - "$d/summary.json"
```
