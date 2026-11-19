# Paid untyped-core transport on Rust Compact

At cap eight, discovery alone took 0.859 times the CPU time of Compact on the same named section. Constructing the atlas, projecting the original runs and publishing the recovered full words put the complete inverse at 1.992 times Compact's CPU time. In separate one-worker processes across 228 admissions, cap eight reached 13,000 KiB high water against 8,428 KiB for the named control. This result closes the explicit-atlas construction as a performance candidate and preserves its exact intermediate representation. [Paired stage observations](phase.csv) · [Memory receipts](rss.csv)

## Construction and source ownership

The forward pass maps each original source through at most h steps of Ψ. A step that would enter U ends the packet in place, preserving a nonempty core. The pass produces an ordered union V of physical terminals, affine original-source and prefix bands, and the set of hidden rows needed to project the original LF table. The packed eight-lane discovery reads a terminal bit and terminal V ordinal at arrival. All equal core bytes share one terminal type, regardless of source prefix. The bottom-up selector merge emits the incoming core column in original destination order. The mark either comes from an exact forward packet containing it, is already in U, or reaches a terminal by a positive projected LF walk. Its original source and offset then follow from the owner band and core-walk length. A cycle disjoint from U uses its original marked-cycle writer. [Atlas and independent checks](atlas-notes.md) · [Theory](../README.md)

Each core has one physical V source bucket. Sort the core minima, then intersect the K consecutive core buckets with B consecutive atlas bands in one sweep. This produces at most K+B−1 slices. Every slice carries the original source of its left endpoint, a core ID and a short prefix. Sorting those slices by original source puts all occurrences of each full word together. Within this fixed-cap construction the spelling determines the cut depth, so equality of full words agrees with equality of the core and the length-tagged prefix. The pass materializes the full dictionary once, without hashing its bytes. For each incoming run of core C, take the next ordinal span in C's bucket, using a monotone slice cursor per core. This directly supplies the original full-word IDs in destination order. The merge consumes exactly one intersection per maximal full-word run. The unchanged second induction accepts the recovered first dictionary, bases, runs and mark. [Publication code](compose.rs)

On 228 accepted Wikidata blocks, cap eight reduces discovery states from 28,053,831 to 21,967,795. It pays 2,201,933 forward run intersections, produces 515,724 original-source bands and constructs 988,258 dictionary slices. Every depth produced exactly the same 1,433,311 original output-run intersections. All 684 first objects matched the dense stable-rank oracle in original dictionary bytes, source bases, incoming maximal runs, mark and offset. Every case entered the existing second stage and reproduced the complete dense marked output. On arbitrary small columns 233,480 marked configurations passed, including 62,112 disjoint marked cycles. The independent Python constructor agrees on all 30 systematic geometries. [Real correctness receipts](admissions.csv) · [Tiny marked audit](small.txt) · [Cross-check](validate.py)

## Matched local stage

The four arms use the same original capped inputs, counts, U and mark. U is the source interval of the heaviest quote-return word, with stable tie breaking. Each receives U before timing. The controls and candidates therefore exclude the same selection cost. The optimized named Compact control is the one-bucket path using its trailing-marker word convention. The transport arms use source-oriented words and return the same original stream. Each of 32 paired repetitions sums ten systematic tapes. Within each block a rotating balanced schedule runs every arm, with warmed, separate retained storage. Timings use thread CPU time in a single pinned Linux thread. The intervals are 95% t intervals over the log ratios of these paired ten-block sums.

| Arm | Setup and mark, ms | Discovery, ms | Selector merge, ms | Compose, ms | Second and output, ms | Complete ratio to named Compact |
|---|---:|---:|---:|---:|---:|---:|
| Compact named | 3.387 | 8.582 | 2.279 | 0 | 4.215 | 1.000 |
| Core h0 | 4.107 | 9.148 | 2.216 | 2.408 | 4.184 | 1.195 [1.172, 1.219] |
| Core h4 | 10.867 | 8.178 | 2.164 | 3.500 | 4.060 | 1.556 [1.512, 1.601] |
| Core h8 | 18.999 | 7.396 | 2.178 | 3.878 | 4.389 | 1.992 [1.936, 2.049] |

Cap-eight discovery versus its source-oriented h0 control measures 0.809 [0.763, 0.859]. The complete cap-eight inverse versus that same control measures 1.666 [1.626, 1.707]. Giving h8 the measured Compact setup cost leaves a complete ratio of 1.149 [1.119, 1.180]. Giving h8 the entire setup and mark for free leaves 0.965 [0.936, 0.995]. The latter two are cost substitutions. With the other observed h8 phases held fixed, setup plus publication must fit about 4.5 ms per ten blocks to tie Compact. They used 22.9 ms. The next consequential change would have to eliminate most of those constructions. The narrower gain in already packed backward discovery does not justify a whole-worker scoring round on this implementation. [Phase calculations and all counterfactuals](summary.txt)

Separate persistent-process measures include the original-cap reader, decoder, atlas scratch, publication scratch, second stage and output. All 228 outputs per arm matched independent dense marked FNV hashes. Peak process measurements were 8,428 KiB for Compact, 9,380 KiB for h0, 12,324 KiB for h4 and 13,000 KiB for h8. Retained vector capacity with output peaked at 8,528,726 bytes for Compact and 9,791,070 bytes for h8. The candidate's first-object capacities and temporary atlas estimates are reported per block. These local single-worker results do not establish CI or target-machine throughput or RSS.

## Reproduction

Implementation commits are 012a238 and 0c63a5d. All local admission, phase and process memory receipts are at a611fb3. The generator checks the previously verified packed projected implementation, its source-hash chain to the original Compact, and its unchanged second stage. The actual Rust compiler was 1.99.1. Use the research worktree's release library and the canonical Wikidata corpus. Start from the original-cap exporter and dense reference command if the local retained inputs are unavailable. [Original admissions exporter and corpus instructions](../../../../seams/async/native/token-writer/RANK.md) · [Independent memory-hash generation](../../../clipped/native/memory.sh)

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/native
out=$(mktemp -d)
bash "$d/build.sh" "$out" "$PWD/target/release/deps"
python "$d/../../../clipped/native/tapes.py" research/astra-e/dense/target/wikidata "$out/ten.bin"
"$out/check"
"$out/check" check "$out/ten.bin" > "$out/ten.csv"
"$out/check" check /tmp/astra-token-all/admissions.bin > "$out/admissions.csv"
nice -n 8 taskset -c 3 "$out/price" price "$out/ten.bin" 32 > "$out/phase.csv"
bash "$d/memory.sh" "$out" /tmp/astra-token-all/admissions.bin /tmp/astra-clipped-final/expected.csv 2 > "$out/rss.csv"
python "$d/report.py" "$out"
python "$d/validate.py"
```

The ten-tape binary SHA-256 is 879fb79ed0e18980f2c74c8f7fd73df2789b6d976de03e87ddd0e4c136ec2cbb. The 228-admission original-cap stream SHA-256 is da0529bf4e73b906fd997c4be87238a892f1feaaa3f549a729c6dd7584cecf26. The independent marked hash CSV SHA-256 is 371c94a260c2f9a01cca5580cbee43ac687cc7b8d1a2b1a3d3e8b295c4131c10.
