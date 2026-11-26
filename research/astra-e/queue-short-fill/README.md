# Short retained-row queue fills

The frozen edge constructor writes a consecutive interval of retained row numbers for each accepted piece. Its full-width input domain is 1–16, except that a primary row in an omitted long run becomes a singleton. The original appends each interval with an exact-size Rust range. The new singleton entry pushes one element for width one and uses the original range otherwise. A separate 1–4 entry gives width two, three and four fixed-size slices. Both variants retain checked vector initialization, standard growth and the original fallback. Each was built on the i9 edge source `badb2ae` and the combined scalar-cap/recovered-count source `50b8ecb`. All six isolated entries are pinned by `d38b8af5aac80579da3ede854c65c9469d18bd03`. [Generator](prepare.py) · [Experimental plan](PLAN.md)

The structural census covers all 14,386,594 retained-range writes on the 233 real admissions. Width one occurs on 75.43% of writes and widths 1–4 occur on 92.64%. Seven singleton calls preserve a primary split from an omitted run. Both variants match every value and queue allocation capacity under the tested Rust 1.99.1 growth rule. The original compiled ordinary singleton executes both the generic range checks and a counted store loop. The explicit push has a direct space check, one store and a length increment. The fixed-slice arm adds indirect dispatch and increases the constructor by 414–460 instruction bytes. [Census and capacity proof](census/README.md) · [Assembly and real-decoder disassembly](asm/README.md)

## Complete decoded blocks and scan attribution

Each observation called the real worker from Huffman symbols through final RLE1 output and block CRC, on one pinned native thread. The compressed stream scanner and reader assembly lay outside this clock. The 17-block panel comprised the first large qualifying block from each of twelve Wikidata streams and all five additional dense admissions. Thus dense admissions were intentionally oversampled. Eight labels rotated through all eight positions, with code-identical twins for each predecessor. The terminal measured interval was the sum of all chosen blocks per lap. Intervals below are two-sided paired 95% t intervals on complete-lap log ratios. [All 6,528 primary observations](paid.csv) · [All 3,264 separately metered observations](meter.csv) · [All contrasts including both twins and subsets](summary.json)

| Comparison | 48-lap complete CPU ratio | 24-lap constructor scan CPU ratio |
|---|---:|---:|
| edge twin / edge | 0.9731 [0.9392, 1.0082] | 1.0248 [0.9754, 1.0767] |
| edge1 / edge | 0.9794 [0.9450, 1.0151] | 0.9639 [0.8704, 1.0676] |
| edge4 / edge | 1.0039 [0.9507, 1.0602] | 0.9563 [0.8947, 1.0222] |
| both twin / both | 0.9920 [0.9596, 1.0256] | 0.9984 [0.9444, 1.0556] |
| both1 / both | 0.9680 [0.9313, 1.0061] | 0.9344 [0.8826, 0.9892] |
| both4 / both | 0.9873 [0.9462, 1.0301] | 0.9285 [0.8791, 0.9807] |

Among the twelve ordinary blocks, the combined singleton's complete CPU ratio was 0.9470 [0.9078, 0.9879] relative to its predecessor and 0.9430 [0.9136, 0.9732] relative to the predecessor's same-code twin. Its all-17-block comparison includes parity with both controls. The fixed 1–4 variant took 1.0443 [1.0092, 1.0807] times the singleton's complete CPU on ordinary blocks. The combined singleton was the sole survivor selected for independent builds and a larger compressed-stream test. The paid results exclude allocator changes and include the natural long-run and wide fallback. Each of the six entries also passed 17,064 arbitrary/periodic marked stable-LF checks and a direct byte-for-byte libbzip2 comparison across all 431 Wikidata blocks in 12 streams, including the 198 blocks that followed other routes. [Adversarial receipt](marked.txt) · [Full corpus receipt](checked.txt)

The complete scorer builds used the refreshed scorer path `75297a1d57b2bdf262c3a19f3064848dedf2c493`, printed `v1 at 81585172f5fb9e7a59f330124779ba084fd6bf04`. Each passed all 338 official cases with real Wikidata, marked multiple-cycle outputs, RLE1, block/combined CRCs, truncation and corrupted inputs. The original kernel's regenerated source equals the frozen combined predecessor after normalizing three include paths. [Original 338-case gate](asm/both0-gate/README.md) · [Singleton 338-case gate](official/README.md)

To reproduce the isolated structural and stage work with Rust 1.99.1, fetch the frozen Wikidata corpus from the scorer recipe, then run the following from the research checkout. The historical uninstrumented support library is built from the frozen edge source. The metered executable changes only two timestamp sites around the initial constructor scan.

```sh
d=research/astra-e/queue-short-fill
tmp=$(mktemp -d)
python3 research/astra-e/edge-displacements/build.py "$tmp/support"
bzip2 -dc corpora/wikidata.bz2 > "$tmp/oracle"
python3 "$d/build.py" "$tmp/plain" "$tmp/support/libmibz2.rlib" original
python3 "$d/build.py" "$tmp/meter" "$tmp/support/libmibz2.rlib" meter
"$tmp/plain/panel" adversarial
"$tmp/plain/panel" gate corpora/wikidata.bz2 "$tmp/oracle"
taskset -c 3 nice -n 16 "$tmp/plain/panel" price corpora/wikidata.bz2 "$tmp/oracle" 48 > "$tmp/paid.csv"
taskset -c 3 nice -n 16 "$tmp/meter/panel" price corpora/wikidata.bz2 "$tmp/oracle" 24 > "$tmp/meter.csv"
python3 "$d/report.py" | cmp - "$d/summary.json"
```

## Entire compressed stream through the official worker

On one pinned CPU of the local Xeon host, the singleton's full-process wall ratio relative to unchanged combined was **0.9851 [0.9620, 1.0088]**. Grouping each complete four-round rotation gave [0.9584, 1.0125]. The empirical intervals span parity under both prespecified estimators, so this execution leaves the direction unresolved. The original and singleton same-binary label ratios were 0.9934 [0.9549, 1.0334] and 1.0081 [0.9669, 1.0510]. Whole-process CPU was 0.9869 [0.9647, 1.0097] and peak RSS was 1.0000 [0.9940, 1.0060]. The two algorithms' per-label median RSS ranged from 41,062 to 41,158 KiB. [Preregistered direct comparison](process/PLAN.md) · [Paired ratios](process/summary.json) · [128 raw official W1 runs](process/runs.jsonl)

Two independently built official binaries followed the same documented scorer refresh and Rust 1.99.1 toolchain. Each observation read all 209,700,196 output bytes with the scanner, decoder, ordered output, CRCs and 4W in flight inside the timed path. The input read and 100 ms settle precede that path. CPU1 affinity applies to the scorer, scanner, worker and reader together. All four labels occupied every position equally often and all within-round predecessor/successor pairs had equal counts. A raw execution archive preserves the exact binaries, including code layout. Independent W2/W4 CI against the same combined predecessor would discriminate a possible sub-five-percent effect. There is no such CI or i9 measurement for the singleton in this record. [Process inputs, toolchain control and order](process/manifest.json) · [Exact worker binaries](process/workers.tar.xz)

To verify the original evidence, set `d=research/astra-e/queue-short-fill` and run the commands below. The final archive check verifies each binary's SHA-256 against the original process record. A new live timing comparison first refreshes both scorer trees as specified in the two linked gate recipes and records the current scorer versions before running `process/run.py ORIGINAL SINGLETON CORPUS OUT` under the CPU affinity and niceness from the preregistration.

```sh
d=research/astra-e/queue-short-fill
python3 "$d/process/report.py" "$d/process" | cmp - "$d/process/summary.json"
python3 "$d/official/verify.py"
python3 "$d/process/check_archive.py"
```
