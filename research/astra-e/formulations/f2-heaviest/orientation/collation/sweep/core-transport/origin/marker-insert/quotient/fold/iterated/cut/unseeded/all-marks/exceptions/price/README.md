# Emitting marked bytes from a supplied predecessor clock

Given an authenticated context model and the ordered exception times, an eight-byte default jump emitted the marked bytes using **0.628 [0.565, 0.698]** of scalar-cap's complete inverse CPU at context depth 8 and **0.702 [0.658, 0.750]** at depth 16. Parsing the supplied clock and constructing the jump table on every call raised the ratios to **0.800 [0.761, 0.840]** and **0.928 [0.876, 0.982]**. Scalar-cap starts with the real capped BWT input and reconstructs the marked bytes. The clock terminal receives the context model and exception order for free in both comparisons. The results establish a conditional terminal speedup and leave the cost of deriving the clock from the capped BWT open. [Registered comparisons](PLAN.md) · [C++ terminal](clock.cpp) · [Frozen sources and executable hashes](manifest.json)

| Context depth | Prepared eight-byte jump / scalar | Build jump on a supplied parsed model / scalar | Parse supplied wire, build jump and emit / scalar | Parse supplied wire and emit one transition at a time / scalar |
|---|---|---|---|---|
| 8 | 0.628 [0.565, 0.698] | 0.683 [0.600, 0.777] | 0.800 [0.761, 0.840] | 1.010 [0.996, 1.024] |
| 16 | 0.702 [0.658, 0.750] | 0.799 [0.745, 0.857] | 0.928 [0.876, 0.982] | 1.253 [1.195, 1.313] |

At depth 8 the prepared jump issued 915,165 eight-byte copies, 1,269,163 unit default transitions and 409,432 explicit events across 8,999,915 bytes. The 815,844 states and their ordered events occupied 7,764,348 wire bytes across the ten blocks. At depth 16 the 1,021,546 copies, 598,534 unit defaults and 229,013 exceptions covered the same output, using 1,170,188 states and 7,912,297 wire bytes. The second emission immediately after a checked first pass was 0.405 of scalar at depth 8 and 0.420 at depth 16. Including the first emission, byte comparison and second emission in that same arm gave 1.022 and 1.079 respectively. Warm replay is separately identified in the raw phase timestamps. [Depth 8 prepared](k8-prepared/summary.json) · [Depth 16 prepared](k16-prepared/summary.json)

Maximum fresh-process HWM of the prepared ordinary jump was 6,716 KiB at depth 8 and 7,920 KiB at depth 16, against scalar's 6,040 and 6,044 KiB in the respective panels. With parsing and jump setup included, the maxima were 7,432 and 8,584 KiB against 5,980 and 5,984 KiB. Prepared jumps retain one 12-byte jump per state, a 4-byte default transition per state, an 8-byte record per exception and the output buffer. Provisioned jumps also hold the supplied wire. The RSS rows include their process, data and allocator high-water effects. [Depth 8 provisioned](k8-provision/summary.json) · [Depth 16 provisioned](k16-provision/summary.json)

Each panel included six arms on the same ten real blocks over 24 balanced rotations. The three controls used the original Rust scalar-cap at `d0a42980e896211906234c7679b38760ae44a46e`, LazyCells and OpenGated. The clock treatments used measured implementation `6ccbea90`. The driver checked every emitted marked buffer against the reference and preserved hashes of the capped input, RLE1-coded marked output and libbzip2 plaintext. It identifies a supplied case by its row count, primary and cap count. This finite-input experiment does not supply an inverse for arbitrary inputs. Intervals use four ordered six-round groups of paired logs, summing the ten blocks' CPU per round. The panels sampled separate executions, so their close cross-panel differences carry no direct paired interpretation. Timings cover inverse work, excluding Huffman/MTF, RLE1 expansion and CRC. No whole-worker, CI or i9 throughput follows from these measurements.

Each panel directory contains the unchanged driver output in `times.csv`, its 750 phase rows in `stamps.csv` including 30 preflight calls, the merged 1,440 calls in `paired.csv`, 180 fresh-process samples in `rss.csv`, and the ten-block gate manifest. Recovery checked all 5,760 timings against the original process rows and stamps, all 720 HWM rows, each native library and input wire hash, and all 40 pairs of archived cap and marked-output hashes. All saved summaries reproduced from these data. [Raw depth 8 prepared](k8-prepared/paired.csv) · [Raw depth 8 provisioned](k8-provision/paired.csv) · [Raw depth 16 prepared](k16-prepared/paired.csv) · [Raw depth 16 provisioned](k16-provision/paired.csv)

Recompute from this directory. The `run.py` driver uses the immutable Rust donor and the clock wires exported by the adjacent census. The active workspace holds the donor at `/workspace/work/astra-powers-original/capped` and the wires at `/tmp/astra-clock-wires`. The wire and binary hashes in the gate and build manifests allow a rebuilt input to be checked before timing. The next decisive experiment is a complete construction that produces actual physical LF destinations directly from capped runs, including its memory and marked-cycle handling. [Scheduled physical-jump experiment](../jumps/PLAN.md)

```sh
for cell in k8-prepared k8-provision k16-prepared k16-provision; do
    python3 report.py "$cell" | cmp - "$cell/summary.json"
done
ref=/workspace/work/astra-powers-original/capped
wires=/tmp/astra-clock-wires
work=$(mktemp -d)
python3 run.py "$ref" "$wires" "$work" build
for k in 8 16; do
    for mode in prepared provision; do
        for step in gate price rss; do
            nice -n 10 taskset -c 0 python3 run.py "$ref" "$wires" "$work" "$step" "$k" "$mode"
        done
        python3 report.py "$work/k$k-$mode" > "$work/k$k-$mode/summary.json"
    done
done
```
