# Paid native test of grouped forward tail resolution

The experiment materializes the actual first-level return information from capped original L runs. It compares the existing eight-lane dynamic constructor, the earlier certified whole-run successor and the compact/preloaded endpoint constructor with a threshold-pruned copy of compact/preload. Every arm pays its original LF preparation, mark transfer, first discovery and root inverse merge. The cold arms additionally publish the missing source slots and destination anchors, build the reversed Ψ index and physically resolve and store every missing tail. This price includes all index construction and staged cold bytes. Second induction and the final output loop are outside the first-stage timer.

Width-one and width-eight cuts use the same compact eight-lane discovery as their comparator. A pruned group retains its already known suffix; root projection supplies each cutoff row's destination rank. Sorting hot buckets and subtracting them from the section gives the disjoint source gaps. A scan of original runs with the cutoff boundaries counts records per byte; a second scan scatters eight-byte (exclusive end, hit bit, byte, displacement) descriptors into F order. The original page bitmap and base arrays are reused for directional lookup after hot discovery. The eight walkers each own a reusable spelling path and a stack of suspended affine pieces; one is refilled from the next source gap when it completes. Terminal spans append their actual raw bytes to one arena and publish one offset/length/cutoff reference per source row. A one-lane width-eight control measures the physical effect of overlapping those independent walkers. Original run records, the directional table, endpoint arrays, staged references, payload arena and lane stacks all remain charged.

Across 228 admissions, every first-source arc from each cold arm was checked against a separate dense stable byte Ψ, including its target rank and every byte of its assembled spelling. The verifier then reconstructed the entire marked output and compared it with Segmented (205,197,794 admitted bytes per arm). The physical table size, emitted terminal spans and staged bytes also matched the previous source-interval census on all nine systematic admissions. The small suite covered 61,096 admissible arbitrary marked-column/arm cases and 24,200 disjoint refusals. Verification uses an untimed dense reference representation; this is a bounded first-stage experiment. [Geometry](geometry.csv) · [Small checks](small.txt) · [Corpus receipt](geometry.log)

Two pinned six-arm panels used 36 balanced rotations per block on nine systematic admissions with Linux thread CPU. The controls used their ordinary front-end interfaces: compact/preload used the byte histogram already available from Runs, while the older sources prepared their original index from capped records. The first panel ran on CPU 4 and the repeat on CPU 3, at nice 8. The repeat driver also supports isolated memory measurement. Every compared implementation and timed path uses the same algorithm in both panels.

| Paid first stage vs compact/preload | First panel | Repeat |
|---|---:|---:|
| Width 1, eight walkers | 1.896 [1.827, 1.968] | 1.929 [1.870, 1.990] |
| Width 8, eight walkers | 2.131 [2.021, 2.247] | 2.107 [2.049, 2.166] |
| Width 1 giving the entire Ψ index for free | 1.291 [1.235, 1.350] | 1.285 [1.245, 1.326] |

On the repeat, the nine-block width-one mean was 18.324 ms: 2.123 ms original preparation/mark, 2.759 ms hot discovery, 1.548 ms root merge, 1.252 ms publication, 6.112 ms Ψ index and 4.532 ms tail resolution/staging. Compact/preload reached the comparable complete first-level information in 9.489 ms. Width eight did beat the superseded known-successor source (.934 [.901,.969]) and eight walkers reduced the width-eight traversal phase to .796 [.760,.834] of one walker. Neither result offsets the price against compact/preload. Reusing the directional index alone cannot close the observed gap. [Raw first panel](phase.csv) · [Independent repeat](repeat.csv) · [Paired summaries](summary.txt)

Steady reusable scratch capacities on frozen were 1,596,200 bytes for compact/preload, 1,979,296 for width-one/eight-lane and 1,740,488 for width-eight/eight-lane. Their maxima across the full ordered corpus were 3,459,664, 5,311,760 and 5,172,432 bytes respectively. Per-block isolated-process RSS sampling used a fresh process per arm and the same scanner/corpus, running only the priced stage. Its high-water readings were 17,808, 18,996 and 19,360 KiB respectively. The vector ledger includes all heap buffers touched by the first stage, while the process readings include the common corpus, scanner and executable. A full decoder and concurrent worker RSS would require a separate benchmark. [Per-block process RSS](rss.csv) · [Per-block capacities](geometry.csv)

The native result is a negative for the paid forward staging schedule. Preserve the [partial second-induction and direct forward-streaming theorem](../widths/) independently of this result. A later candidate would need a materially different way to resolve or use cold tails; further tuning of the lookup count alone is unsupported by these measurements.

From the frozen research worktree with corpus and release dependencies present:

```sh
d=research/astra-e/formulations/f2-heaviest/cold-links/native
bash "$d/run.sh" /tmp/f2-cold-native "$PWD/bench/moonshot/target/release/deps" 4 3
```

The generator verifies the hashes of the existing constructor sources before compiling the experiment. The manifest identifies the release, corpus, current scorer-path revision and compiler. [Source hashes](sources.txt) · [Manifest](manifest.txt)
