# Cost of emitting from a prepared physical return table

On the ten real depth-eight blocks, traversing the prepared physical table took **1.006 [0.966, 1.049]** of the complete scalar-cap inverse CPU. Giving the terminal its events in chronological order took **0.674 [0.617, 0.737]** of scalar in the same panel. Renumbering identical 12-byte records into marked traversal order and using the same emitter recovered the chronological terminal's performance. This prices the physical event dependency before charging for discovering any defaults, exceptions, return destinations or distances. It leaves little measured room for their construction at these densities. [Full paired results](summary.json) · [2,160 original process observations](times.csv)

| Prepared terminal | Median for ten blocks | Paired time / scalar | Paired time / chronological |
|---|---:|---:|---:|
| Physical exception rank | 26.025 ms | 1.006 [0.966, 1.049] | 1.493 [1.376, 1.619] |
| Chronological clock and explicit payload | 16.971 ms | 0.674 [0.617, 0.737] | 1 |
| Return records in marked traversal order | 16.756 ms | — | 0.980 [0.920, 1.044] |
| Complete scalar-cap inverse | 26.091 ms | 1 | — |

Physical rank cost 1.523 [1.352, 1.716] of the lap-ordered layout on the identical emitter. Both layouts follow a data-dependent next index, use the same event byte, poststate, distance and eight-byte jump generator, and produce the same output. Locality and its interaction with the dependency matter in this panel. No counters were collected to apportion that effect between cache levels and prefetch. The historical 0.628 chronological estimate used an earlier panel. Each estimate here uses same-process paired comparisons clustered into six ordered groups of six rounds. Complete scalar-cap starts with capped BWT records; the three terminals start with prepared in-memory data. These are local inverse-stage CPU results. [Preregistration at b702c95e](PLAN.md) · [Historical prepared clock](../../price/README.md)

## Mark and phase

Define U as the physical exception rows plus the primary. At anchor x, record the true L[x] and poststate at LF(x), the first positive return distance d, and the dense U rank of LF^d(x). An artificial primary uses the default edge at the marked starting state. Emit the first byte explicitly, then d−1 model defaults. Each intermediate row lies outside U and therefore follows its exact default. Store these bytes backwards, ending at the current upper bound of the n-byte buffer. The next-anchor index and its payload restart the model at each return. Arrival at the original primary completes one lap. If its length is ℓ<n, the last ℓ buffer bytes now give the forward marked lap starting with the row reached by Ψ from the primary. Move that lap to the start and copy whole or partial repeats to fill n. This preserves phase even when n is not a multiple of ℓ. Other cycles, including cycles carrying physical exceptions, never enter this walk. [Native emitter and independent forward-Ψ check](terminal.cpp)

The finite test checked both layouts on **190,426** binary-through-seven and ternary-through-five physical column, primary and exception-mask combinations. This includes 160,576 short marked cycles, 96,240 nondivisor lengths, 95,213 artificial primaries and 134,148 cases with exceptions on foreign cycles. Defaults deliberately give a wrong byte at exceptional rows and, on n>1, a wrong successor. Forward Ψ produces the comparison bytes independently. On the ten real cases all marked laps had n rows. The unchanged Rust donor checked every arm's entire RLE1-coded marked buffer on every call against LazyCells. The SHA-256 receipts cover caps, predictor wires, physical masks, marked output and the previously libbzip2-verified plaintext. [Finite receipt](small.csv) · [Real inputs and reference hashes](inputs.json)

## What the terminal pays

Across 8,999,915 bytes, the physical table has 409,433 records, including one artificial primary on block 41. It issues 409,433 dependent anchor reads, 915,165 eight-byte default copies and 1,269,162 single-byte defaults. The chronological arm issues 409,432 explicit events, 915,165 copies and 1,269,163 single defaults. All arms consume the actual marked bytes once. Each ring has 12 bytes per physical anchor, totaling 4,913,196 bytes across ten blocks (maximum 619,764 per block). The clock uses 8 bytes per event, totaling 3,275,456 (maximum 413,168). Both need four bytes of default transitions and twelve bytes of jumps per state, totaling 3,263,376 and 9,790,128 bytes across the cases, plus the output. The benchmark caches the ten prepared tables per arm, reserving exact record counts. These are data sizes. Process high-water memory and whole-worker throughput remain unmeasured here.

The validation fixture constructs stable LF from the authentic physical L, combines the authenticated chronological predictor and physical mask, checks every real row's byte and final state, then discards dense LF, masks, row ordinals, chronological input and wire before the physical terminal is priced. The lap-ordered control receives the permutation for free. Obtaining a permutation with equivalent locality remains an algorithmic cost in an end-to-end decoder. The independently priced physical return constructor also requires the exception ranges as input. [Neighboring constructor and proof](../PROOF.md)

## Reproduce

Source commit `39a1ac33` preserves the emitter, build flags, signatures and finite gate. The immutable capped Rust binary starts with scalar-cap `d0a42980e896211906234c7679b38760ae44a46e`. Use the ten gated caps and predictor wires described in the original panel. The predictor and mask digests for this run are pinned in the input receipt. The stored Rust donor verifies the marked bytes in each call. [Build hashes](build.json) · [Driver](run.py) · [Wire regeneration](../panel.py)

```sh
ref=/workspace/work/astra-powers-original/capped
wires=/tmp/astra-clock-wires
work=$(mktemp -d)
python3 run.py "$ref" "$wires" "$work" build
taskset -c 2 nice -n 10 python3 run.py "$ref" "$wires" "$work" gate
taskset -c 3 nice -n 10 python3 run.py "$ref" "$wires" "$work" price
python3 run.py "$ref" "$wires" "$work" report
```
