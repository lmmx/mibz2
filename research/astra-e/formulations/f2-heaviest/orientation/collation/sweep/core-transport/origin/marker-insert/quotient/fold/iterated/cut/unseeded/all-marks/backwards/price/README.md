# Price of reversing a supplied numerical position tree

On ten original Wikidata blocks the reverse offset expansion and final marked F-byte scatter took **1.665 [1.599, 1.734] times the complete scalar-cap inverse**, even with every contraction and parsed descriptor supplied for free. An immediately repeated reverse and scatter of the same block took 1.657 [1.578, 1.740]. The ratio of the two inner timings was 0.995 [0.946, 1.046]. Across individual blocks, all ten ordinary free-tree ratios and all ten immediate-warm ratios had lower endpoints above one. These figures bound the measured terminal implementation optimistically under the conditions of this local inverse-stage comparison. They do not establish a complexity lower bound for the mathematical representation. [Same-cap paired observations](paired.csv) · [Ten-block and per-block statistics](summary.json) · [Preregistration](PLAN.md)

| Ten-block terminal quantity | Median thread CPU (ms) | Paired ratio to complete scalar-cap, 95% interval |
|---|---:|---:|
| Reusable parsed descriptors, reverse only | 18.065 | 0.745 [0.703, 0.789] |
| Reusable parsed descriptors, marked F scatter only | 22.421 | 0.918 [0.877, 0.961] |
| Reusable parsed descriptors, reverse plus scatter | 40.428 | 1.665 [1.599, 1.734] |
| Same-block verified rehearsal, then timed reverse plus scatter | 39.706 | 1.657 [1.578, 1.740] |
| Resident wire, fresh parse, scratch allocation, reverse and scatter, entire call | 47.901 | 2.016 [1.875, 2.169] |
| Frozen scalar-cap, complete input-caps-to-marked-bytes call | 24.541 | 1 |

The real Rust harness used identical capped records and the true primary for all six arms. A treatment lookup by (n,primary,record-count) supplies one previously authenticated numerical wire for that exact test block. Its initial frontend and all 25 cut constructions are outside every reported terminal comparison. Tree construction is essential to a real decoder and is deliberately free for the numerical lower-price experiment. Every Rust result was checked against the reference marked bytes. The on-disk cap and output hashes were recomputed; frozen input receipts connect the pre-RLE1 hashes to independent libbzip2 block output and stored block CRC. The numerical constructor at 6a80e8a9 regenerated all ten wire SHAs exactly as recorded at 3a1d3819. That earlier check independently walked and audited every one of the 8,999,915 original source positions. [Regenerated tree identities](wires.jsonl) · [Numerical all-position gate](../blocks.jsonl) · [New ABI and input gate](gate.json) · [Frozen compiler and Rust source hashes](manifest.json)

The free-tree arm has a separate parsed tree per block, reusable offsets and reusable output. Only offset[0] is initialized on every call. Every other active offset and every output byte are written by the verified reversal. The inner sum omits provisioning, instrumentation outside those boundaries, input lookup and release; its full Rust-visible call ratio was 1.669 [1.603, 1.738]. The immediate-warm arm rehearses the same block into the same scratch and verifies that entire rehearsal before its inner timing. Its external call includes both reconstructions and the extra byte comparison, giving 3.499 [3.346, 3.659]. Each primary panel cycles ten blocks, so the same address reuse seen by its free-tree arm need not reproduce any particular cache state on a target worker. The parsed-wire arm paid a median 4.114 ms for in-memory parsing and 1.865 ms for provisioning, over ten blocks. All wire file reads occur in each arm's unreported warmup. [Source and clocks](terminal.cpp) · [All C++ call-level observations including warmups](stamps.csv)

In 180 separate one-block processes, the maximum frontend-plus-inverse VmHWM was 8,992 KiB for parsed trees, 9,876 for warmed trees, 12,040 for fresh parsing, 5,976 for scalar-cap, 9,112 for LazyCells and 9,116 for OpenGated. Warmed trees additionally retain the reference bytes to verify the same-block rehearsal. The single-block free process reads a wire and parses its tree before returning its first output, so its peak includes acquisition of the supposedly free tree. In the timed ten-block process reusable C++ vectors grew beyond their largest live sizes when moving between blocks; the largest retained offset capacity was 4,809,776 bytes (maximum active offsets 2,418,548 bytes). Peak output capacity was 1,799,862 bytes for at most 900,000 active output bytes. The largest single parsed descriptor tree occupied 3,292,188 allocated bytes including its arrays, sentinels and stored Level records. These process peaks are stage data and do not establish the peak for a concurrent decoder. [Fresh memory rows](rss.csv) · [Fresh benchmark binary](rss-manifest.json)

The earlier bounded byte-matrix cut left a median 12.49% of scalar CPU for its continuation in its own ten-block panel. Here the reverse alone and the scatter alone both exceed that share by a large margin. The numerical descriptor history is unseeded and uses its own sequence of orientations and ranks, whereas that byte matrix built a protected singleton and a different weighted root. The two panels also have separately measured scalar denominators. The measured costs can be added inside this experiment when they are disjoint intervals of the same call; the numeric reverse and F scatter, for example, are summed within block and round before forming ratios. Adding the matrix prefix and this terminal would require an explicit common rank map and a matching tree for that prefix plus a single paired implementation, including any translation work. The reported ratios do not justify such a sum. A promising alternative would have to remove substantial offset expansion or output scattering work as well as constructing its forward maps. [Earlier seeded residual budget](../../../../fused-publication/byte-view/budget.json)

## Reproduce

Use the ten public original source tapes and the Rust 1.99.1 scalar/Lazy/Open capped harness built by the preceding calibration. Its static Rust executable, scalar source and source checks are verified. The C++ DSO must resolve to the generated library. The build uses the frozen emitter's integer loop order, drops optional audit-position stores and per-element asserts after the complete differential audit, and retains byte-exact output checks outside the measured kernels. The src-tree extraction in wires.py includes the original Python dependencies and project-root marker. The active timing run pinned thread CPU to CPU0 with nice +16 on a shared 4-CPU-equivalent host; there is no independent CI or i9 verification for this phase. [Stable Rust reference build](../../../../powers/xor/README.md) · [Frozen gate and compiler](manifest.json)

```bash
d=$(dirname "$(rg --files | rg '/all-marks/backwards/price/PLAN.md$')")
w=$(mktemp -d)
ref=/workspace/work/astra-powers-original/capped
taskset -c 0 nice -n 16 python3 "$d/wires.py" "$w/wires" > "$w/wires.jsonl"
taskset -c 0 nice -n 16 python3 "$d/run.py" "$ref" "$w/wires" "$w/price" build
taskset -c 0 nice -n 16 python3 "$d/run.py" "$ref" "$w/wires" "$w/price" gate
taskset -c 0 nice -n 16 python3 "$d/run.py" "$ref" "$w/wires" "$w/price" price
taskset -c 0 nice -n 16 python3 "$d/run.py" "$ref" "$w/wires" "$w/price" rss
python3 "$d/report.py" "$w/price"
python3 "$d/report.py" "$d" | diff - "$d/summary.json"
```
