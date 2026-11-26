# The depth-eight return bands already exceed the scalar inverse budget

On the ten systematic Wikidata blocks, construction of 127,360 canonical weighted return bands took **2.347 [1.862, 2.958] times the CPU** of the complete frozen scalar inverse. The paired wall ratio was **2.366 [1.852, 3.024]**. Both arms ran at one worker on the same CPU. The median ten-block total was 58.603 ms thread CPU for the bands and 23.787 ms for the scalar bytes. The prespecified lower-bound rule stops native pricing of this implementation at k=8. [Frozen rule](PLAN.md) · [All 480 timed calls](pairs.csv) · [Aggregate and per-block intervals](summary.json)

The range arm started with the maximal width/byte RLE, original mark and prepared k8 context starts/modal bytes. Timed work included LF edge extraction, smooth-state contraction, complete affine interval routing, coalescence and publication of all retained source/target/elapsed triples. The independent membership-packet constructor's full canonical wire matched after every call. The scalar arm started with the Rust scanner/frontend's actual capped Runs and produced all 8,999,915 marked pre-RLE1 bytes across the ten blocks. Every output matched its previously libbzip2-bound reference. The production frontend's cap bytes, the candidate's supplied maximal runs and the independent checker’s original incoming bytes were bound to each other before timing. The physical exception masks were independently reconstructed from the supplied starts/modal bytes and the original incoming column. Across the ten blocks 409,432 modal exceptions are present. Supplying the original RLE and the k8 model avoids charging their acquisition to the range arm. The measured range stage finishes before marked event ordering and output. [Input and reference digests](inputs.json) · [Warm exact gate](gate.csv)

The CPU intervals use the 24 within-round ratios of the ten-block summed times and four disjoint six-round clusters of their logarithms. The mean log and two-sided Student interval use all four clusters. Each physical block had twelve samples in each arm order. The individual estimated ratios range from 1.667 on block 313 to 3.085 on block 269. Every individual CPU lower bound exceeded one. Scalar on round 4, block 313, took 133.999 ms thread CPU and 133.994 ms wall. This conspicuous observation is retained. The median across all round ratios is 2.423. As a descriptive sensitivity check, omitting the entire fourth round would raise the geometric mean to 2.529. Its cause is unknown.

| Fresh-process VmHWM | Prepared range construction | Capped scalar inverse |
|---|---:|---:|
| Median before kernel, KiB | 3,006 | 2,360 |
| Median after kernel, KiB | 6,482 | 4,680 |
| Maximum before kernel, KiB | 4,368 | 2,840 |
| Maximum after kernel, KiB | 12,844 | 6,084 |

The 60 fresh processes cover three calls per physical block and arm. A range process holds its supplied original runs, model starts/modal bytes and a new solver. A scalar process uses a recovered block to build real cap records, discards the scanner input and starts with a new scalar scratch. Peak memory is sampled before loading the independent verification output. This assessment includes the selected input's local provisioning high-water. During paired timing the gates keep all ten inputs and expected outputs resident, with one reused solver per arm. Neither the fresh stage RSS nor timing is a multiworker measurement of a bzip2 decoder. [Fresh observations](rss.csv) · [Runtime](runtime.json)

## Reproduction

Build the stage driver from the repo root with the prepared d0a42980 scalar checkout, its matching release rlib and its original capped-worktree output bytes/caps. The native source and independent reference are pinned at 990450a2 and dd836e65 respectively. The Rust executable references the exact frozen candidate, kernel, gate and adapter and verifies all their hashes and that of the rlib before compiling. The range ABI inputs are prepared by the direct context constructor and the separately gated physical mask exporter. The independent reference reads only its mask and original L. GNU C++ 14.2.0, rustc 1.99.1, Linux 6.18.57 and Intel Xeon Platinum 8573C were used. The panel ran pinned on CPU3 at nice +12 with concurrent research on other CPUs permitted. [Build identifiers](build.json) · [RLE and context reconstruction](../../../../contexts/README.md) · [Clock mask exporter](../../../../return/README.md)

```bash
d=$(dirname "$(rg --files | rg '/acquire/ranges/native/price/driver.rs$')")
scratch=$(mktemp -d)
prepared=/workspace/work/astra-scalar-route-build/scalar
rlib=/workspace/work/astra-scalar-route-build/scalar-build/release/deps/libmibz2.rlib
original=/workspace/work/astra-powers-original/capped
models=/tmp/astra-exception-contexts
masks=/tmp/astra-clock-wires
python3 -B "$d/build.py" "$scratch" "$rlib" "$prepared"
for action in prep gate price rss; do
    taskset -c 3 nice -n 12 python3 -B "$d/run.py" "$action" "$scratch" "$models" "$masks" "$original"
done
python3 -B "$d/report.py" "$scratch"
python3 -B "$d/report.py" "$d" | diff - "$d/summary.json"
```

The compile flags and binary fingerprints record the actual native run. The decision applies to this implementation and endpoint. The range construction's exactness, including width-independent routing, remains a separate mathematical result. [Native source](../ranges.hpp) · [Bound and proof](../../PROOF.md)
