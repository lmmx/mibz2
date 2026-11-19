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

## Inclusive complete-worker results

The official scalar-four score passed all 338 cases. Across nine complete paired repeat-two runs at each W, its local ratio to the release LazyCells worker was 0.990 [0.863, 1.135] at W1 and 0.865 [0.809, 0.926] at W4. Median process peak RSS ratios were 0.894 and 0.729. Against opend, the W1 and W4 ratios were 0.954 and 0.968. Every arm used scorer path `75297a1d57b2bdf262c3a19f3064848dedf2c493`, printed `v1 at 81585172f5fb9e7a59f330124779ba084fd6bf04`. [Official score, including both opend intervals](official/score/summary.txt)

In the separate balanced five-arm panel, four-forward's incremental whole-worker wall ratios crossed parity at both W1 and W4. There is no established incremental gain from this local comparison, despite the measured emitter gain. The prespecified scalar interaction log contrast also includes parity. Independent CI is evaluating four-forward on the i9-verified scalar baseline. [Decision rule](official/PLAN.md) · [200 direct official-worker observations](official/direct/panel.jsonl) · [All direct contrasts and clustered intervals](official/direct-summary.json) · [Independent CI run 44467969034](https://github.com/lmmx/mibz2/actions/runs/44467969034)

| Incremental complete comparison | W1 wall [95%] | W4 wall [95%] | W1/W4 peak RSS ratios |
|---|---:|---:|---:|
| Four-forward edge / edge | 1.0250 [0.9650, 1.0887] | 1.0548 [0.9950, 1.1182] | 1.0002 / 0.9935 |
| Four-forward scalar / scalar | 1.0462 [0.9812, 1.1155] | 1.0164 [0.9697, 1.0655] | 1.0074 / 1.0042 |
| Four-forward edge / two-forward edge | 1.0053 [0.9437, 1.0709] | 1.0076 [0.9402, 1.0797] | 1.0019 / 1.0064 |
| Scalar interaction | 1.0207 [0.9464, 1.1009] | 0.9636 [0.8966, 1.0357] | 1.0072 / 1.0108 |

The local official scorer and direct worker executables had identical scorer blob trees across their before-round master refreshes. The three new compiled executables each passed all 338 cases, including 209,700,196 Wikidata output bytes, periodic and multiple-cycle fixtures, and 600 corrupted input cases. The five-arm comparison used the exact two older executables from the original two-forward record. From the repository root run `python3 research/astra-e/two-seed/multi/official/verify.py` to verify all 254 timed runs, the gates and preserved compiler images. [Compiler and constructor hashes](official/manifest.json) · [Three complete worker images](official/workers.tar.xz) · [Edge and two-forward compiler images](../official/workers.tar.xz)
