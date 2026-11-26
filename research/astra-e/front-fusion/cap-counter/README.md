# Queue-counter recovery with and without scalar first-cap on the edge worker

Both changes together took **0.9663 [0.9388, 0.9946]** of the original edge worker's post-Huffman CPU time on the 14-block sample. Adding queue-counter recovery to the scalar worker took **0.9763 [0.9514, 1.0018]**. The four-arm interaction interval includes 1. The native scan clock showed a reduction on both parent workers. The incremental complete-worker effect warrants an independent comparison against scalar. No target i9 or peak-RSS result for the combined entry is part of this experiment. [Preregistered plan](PLAN.md) · [Complete contrasts](results/contrasts.csv)

## Frozen implementations

Source commit `50b8ecb8946a498601e3fd894ed84a2687df6cc8` contains all four entries and their required sibling modules. Each uses the edge gate of `badb2ae` and the same original queue, immediate-word encoder, endpoint-width guard, lookahead, stock fallback, RLE1 and CRC paths. A compile-time constant selects the scalar first-cap test from `d0a4298`. A second constant selects the queue-counter transformation from `af0664a`. The earlier counter experiment used the original `80d8409` admission policy, with its own separate controls. [Generator](generate.py) · [Earlier counter receipts](../../narrow-short/cost/README.md)

| Arm index | Scorer entry | Scalar first-cap | Recover counters from queues |
|--:|---|---|---|
| 0 | [edge.rs](edge.rs) | Off | Off |
| 1 | [counter_edge.rs](counter_edge.rs) | Off | On |
| 2 | [scalar_edge.rs](scalar_edge.rs) | On | Off |
| 3 | [both.rs](both.rs) | On | On |

When the first record's width is below 127, the capped-record producer has already finished its maximal same-byte run. A full first record still takes the original coalescing loop. For counter recovery, the queue length of byte a is its retained occurrence count at every scan prefix. The temporary total counts omitted occurrences alone. Adding the queue length at a thick strip gives its original destination occurrence rank. Adding each queue length at the end restores all 256 original totals and thin counts. If the primary lies in a thick run, its two possible omitted parts retain their classification and the primary singleton is enqueued once. The nonqueue specialization continues counting every occurrence directly. Neither switch changes accepted source formats or creates a new retained array.

## Paired native measurements

The two binaries linked the same refreshed gate's Rust 1.99.1 release library. The unmetered binary timed four actual `decode_block` paths from symbol tape to final bytes and block CRC. The separately metered binary put two thread-CPU reads around geometric scan and queue fill, including the final counter-recovery pass. Scratch clearing precedes that inner clock. Each binary checked all 431 Wikidata blocks against the same library's LazyCells worker, totaling 209,700,196 bytes. The prespecified timing sample contained nine ordinary large blocks and five extra edge admissions. Each complete round decoded the same 14 blocks once per arm, balancing all four order positions within every block. Thread CPU was measured at nice +12 on CPU3 of the shared Xeon. The estimate is the geometric mean of complete-round ratios, with a two-sided paired log t interval. [Unmetered observations](results/original.csv) · [Metered observations](results/meter.csv) · [Compiler, binary and library identities](results/environment.txt)

| Contrast | Complete post-Huffman worker, 40 rounds | Scan and queue fill, 32 rounds |
|---|---:|---:|
| Counter on original edge | 1.0030 [0.9705, 1.0365] | 0.9071 [0.8741, 0.9414] |
| Counter on scalar | 0.9763 [0.9514, 1.0018] | 0.9240 [0.8875, 0.9620] |
| Scalar on original edge | 0.9898 [0.9536, 1.0274] | 0.9593 [0.9285, 0.9912] |
| Scalar on counter/edge | 0.9635 [0.9306, 0.9975] | 0.9772 [0.9547, 1.0002] |
| Both on original edge | 0.9663 [0.9388, 0.9946] | 0.8864 [0.8513, 0.9229] |
| Interaction, counter-on-scalar / counter-on-edge | 0.9734 [0.9275, 1.0215] | 1.0186 [0.9805, 1.0582] |

The two clocks measure different work, with separate binaries and repetition counts. On the unmetered worker, the counter-on-scalar ratios were 0.9911 [0.9560, 1.0276] over the ordinary inputs and 0.9661 [0.9240, 1.0101] over the five extra admissions. The 14-block panel deliberately includes all five extras. Its ratios have no full-corpus weighting. The four controls share one executable, which can differ in layout from separately linked historical or CI executables. Each selected input received verified output on every measured invocation. [All strata and statistical extraction](report.py) · [Timing harness](panel.rs)

## Correctness and scorer provenance

The two new entries independently passed all **338/338** scorer cases, including libbzip2 bytes and CRC checks, empty streams, periodic/multiple-cycle inputs, the actual corpus and mutation regression checks. The scorer was fetched again for each entry in the detached designated checkout at `50b8ecb`. Both fetched master `3df195b10e9fb6f86f0c06ad9fad1feb992406d5`, scorer path `75297a1d57b2bdf262c3a19f3064848dedf2c493`, printed `scorer v1 at 81585172f5fb9e7a59f330124779ba084fd6bf04`. The official runs used CPU4, nice +15, one build job and correctness-only options. [Counter/edge receipt](results/counter_edge/summary.txt) · [Both receipt](results/both/summary.txt)

Independent marked tests also compared each of the four scans with a byte-column reference: all 256 total/thin counts, every thick strip's original start, occurrence rank, width and byte, the retained mark, every output byte and the complete published constructor counters. They exercised 10,823 marked constructions, including the 126/127/128 cap seam, periodic and arbitrary LF cycles, three wide-endpoint cases and an eight-byte packed-edge case. In 3,651 of those constructions, all four nonqueue specializations also matched. Six dense, independently generated rejected cases covered the short-size and large-over-budget stock paths at three primary positions apiece. All four entries gave the original LazyCells return status and nonempty caller-buffer result. [Targeted checker](adverse.rs) · [Targeted receipt](results/adverse.txt)

## Reproduction

From the repository root with Rust 1.99.1 and canonical corpus SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`, the replay below builds the gates in a new designated checkout and executes the recorded native design. A future scorer revision requires comparisons on that same refreshed revision. For historical identification the exact fetched revisions and both gate argument records are retained in the results. The scorer entries themselves need no generator preflight when taken from the frozen commit.

```bash
d=research/astra-e/front-fusion/cap-counter
source /workspace/work/env.sh
bash "$d/replay.sh" /workspace/work/astra-cap-counter-replay "$PWD/corpora/wikidata.bz2"
python3 -B "$d/report.py" "$d/results" | diff - "$d/results/contrasts.csv"
```
