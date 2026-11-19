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

## Complete worker and independent status

Both independently built worker executables passed the refreshed scorer's full 338-case gate. In the official nine-round, repeat-two scorer at W1 and W4, the candidate's local wall ratios to 0.5.6 were 0.673 [0.340, 1.332] and 0.723 [0.271, 1.930], with median RSS ratios 0.889 and 0.734. The estimates at both worker counts have substantial host variance. The common scorer path revision was `75297a1d57b2bdf262c3a19f3064848dedf2c493`. The scorer printed `v1 at 81585172f5fb9e7a59f330124779ba084fd6bf04`. [Scorer output, including opend](official/score/summary.txt) · [54 raw official observations](official/score/runs.jsonl)

In 20 direct pairs per worker count, two-seed over the exact unchanged edge worker was 1.0202 [0.9088, 1.1454] at W1 and 0.9769 [0.8686, 1.0986] at W4. Direct RSS ratios were 1.0023 [0.9964, 1.0082] and 0.9961 [0.9826, 1.0098]. Clustering four consecutive rounds widened the wall intervals. The native emitter benefit remains measured, and the complete local comparison leaves the incremental direction unresolved. The Coordinator's i9 result would establish target throughput and peak RSS. [Direct comparisons and uncertainty](official/direct-summary.json) · [80 raw direct observations](official/direct.jsonl) · [Independent CI run 44465700927](https://github.com/lmmx/mibz2/actions/runs/44465700927)

For an exact receipt replay run `python3 research/astra-e/two-seed/official/verify.py`. Complete scorings use a fresh detached checkout at `c0e0e657e1e9628a7b80c33f4584819d1b169893`. Run the following with the canonical corpus path and a scratch directory. For a new round, refresh again immediately before scoring, save the scorer path and printed version, and compare arms only under the common projection. A separate archive holds both exact binaries, including their original compiler layouts.

```sh
git fetch origin master && git checkout origin/master -- bench/moonshot .just Justfile
git log -1 --format=%h origin/master -- bench/moonshot
d=research/astra-e/two-seed
python3 "$d/build_worker.py"
python3 bench/moonshot/moonshot.py score "$d/candidate.rs" --corpus "$wiki" --target-dir "$scratch/candidate-target" --rounds 9 --workers 1,4 --repeat 2 --settle 2 --with-opend --no-proxy --out "$scratch/score"
python3 bench/moonshot/moonshot.py score "$d/control.rs" --corpus "$wiki" --target-dir "$scratch/control-target" --no-proxy --no-timing --out "$scratch/control-gate"
python3 "$d/official/direct.py" run "$scratch/control-target/release/moonshot" "$scratch/candidate-target/release/moonshot" "$wiki" "$scratch/direct.jsonl"
```

Independent paired CI at 20 rotations then found a regression for **two-forward / edge** on the 4-worker Xeon runner, ratio 1.0302 [1.0093, 1.0516]. W1 was 1.0133 [1.0003, 1.0265] and W2 was 1.0105 [0.9970, 1.0243]. W4 median RSS was 0.9995 of the edge worker and 0.7249 of LazyCells. The new writer therefore fails its independent incremental screen. The native emitter saving remains a stage result. On the separate three-block diagnostic the complete candidate executed 12.4–13.8% more instructions with 7.8–11.5% fewer past-L2 dependent links. [Independent CI](https://github.com/lmmx/mibz2/actions/runs/44465700927) · [All six nominal proxy comparisons](proxy/README.md)

## Preserved descendants

The 3- and 4-seed bounded-reservation theorem terminates within k passes and one original output allocation, with an integer family proving the bound is tight. Four streams achieved 0.8324 [0.8071, 0.8585] of the original emitter's local CPU. The complete scalar-four/single-scalar comparison was 1.0462 at W1 and 1.0164 at W4, with both local 95% intervals including parity. Its frozen code, 338-case gates, all 254 complete W1/W4 observations and exact executables are available at `47218b5` and `1f2ec40`. [Bounded multiple streams](multi/README.md)

The latest segmented RLE1 variant uses those four positive fragments without permuting them into physical marked order. It eliminated every one of the 141,536,791 bytes in recorded rotation ranges on its 233 real admissions. Each of 28 uncertified joins needed only a few raw bytes from its adjacent fragments. The complete sliced/joined W1/W4 ratios were 1.1872/1.0641, both inconclusive. The implementation, independent proofs and complete receipts are pinned at `85025c9` and `8d43bd6`. [Sliced RLE1 and CRC](multi/seams/README.md)
