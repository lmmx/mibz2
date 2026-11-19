# Two forward walks in the existing output allocation

The native emitter on the measured i9 edge implementation used **7.7% less CPU** with two in-place forward walks on the 14-block Xeon panel, ratio 0.9228 [0.8879, 0.9591]. The new operation splits the marked cycle before knowing the second stream's final output offset. The constructor already supplies enough information to start both successor chains at once, with constant extra state and the same output allocation. The full worker is frozen at `c0e0e657e1e9628a7b80c33f4584819d1b169893`. [Complete stage results and controls](STAGE.md) · [Source](writer.rs) · [Authoritative gate](official/candidate/summary.json)

The second start is the primary rank plus half the number of retained ranks, modulo that number. On the 233 actual admissions this paired 6,496,920 of 26,567,581 successor reads with a second simultaneous read. Two forward labels stay in natural byte order in provisional left and right reservations of the original buffer. A reservation that fills provides an upper bound on the opposite stream's complete length. That bound is exact for separate cycles too, since first-return edge spellings partition distinct original rows. Resolving the unknown split cost 52,763,621 bytes of contiguous movement over the 209,697,662 emitted bytes. On the real full-return blocks each relocated prefix moved once. The measured and independently reconstructed work agreed on all 466 block/schedule rows. [Mathematical proof and exhaustive checks](proof/README.md) · [Complete geometry and workload](geometry/README.md) · [Paid counts](all.csv)

The earlier four-tape writer established that arbitrary seed fragments compose in the order of their outgoing seed pointers. The incremental result here is the bounded forward placement in one allocation, the reciprocal size proof on exhaustion, and its measured native cost on the independently verified edge worker. Earlier sparse landmarks discovered offsets through a complete preceding walk, and the bidirectional writer built a reverse edge table. [Historical four-tape implementation](../open/lanes/README.md) · [Sparse landmark study](../sparse-landmarks/README.md) · [Bidirectional writer](../bidirectional/README.md)

The historical attribution study put residual emission at 38.39% of inverse CPU and admitted inverse at 79.64% of measured worker CPU. Applying those two shares and the present 7.72% emitter change arithmetically gives about 2.36% of worker CPU. Different blocks, machines and compiler layouts limit this calculation to a scale estimate. The actual full worker includes the front end, construction, stream ordering, RLE1 and integrity work and is measured separately. [Published attribution](../narrow-short/cost/summary.txt) · [Whole-worker decision rule](official/PLAN.md)

## Reproduce the native test

Use a clean checkout carrying this record, Rust 1.99.1 and the scored Wikidata corpus. The native four-arm/eight-label panel uses exactly the original edge constructor at `badb2aec3624bd65606c429b2e6f79e9d9d0050f`. The independent Segmented oracle builds a full inverse table. Every paid complete output is compared outside the measured interval. The paired sample comprises ten original landmark ordinals and all five additional dense admissions, with common ordinal 41 counted once.

```sh
d=research/astra-e/two-seed
scratch=$(mktemp -d)
python3 research/astra-e/edge-displacements/build.py "$scratch/support"
python3 "$d/prepare.py" "$scratch/experiment"
EDGE_TABLE_SOURCE="$scratch/experiment/kernel.rs" rustc --edition=2021 -C opt-level=3 -C codegen-units=1 -C debuginfo=1 "$d/main.rs" --extern mibz2="$scratch/support/libmibz2.rlib" -o "$scratch/panel"
"$scratch/panel" synthetic
"$scratch/panel" gate corpora/wikidata.bz2 > "$scratch/geometry.csv"
cmp "$scratch/geometry.csv" "$d/all.csv"
taskset -c 3 nice -n 16 "$scratch/panel" price corpora/wikidata.bz2 > "$scratch/paid.csv"
python3 "$d/report.py" | cmp - "$d/summary.json"
```
