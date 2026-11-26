# The cost of recovering chronological order from the physical return ring

The chain walk and compact event writes alone cost **0.447 [0.410, 0.487]** of the complete scalar-cap inverse with reusable storage. Emitting bytes from the recovered chronological list then cost **0.623 [0.570, 0.681]** of scalar. Including both phases in the timed Rust call cost **1.076 [0.995, 1.163]** of scalar. Fresh per-block event storage gave **1.009 [0.964, 1.055]**. Both panels gave the split arm an already constructed physical ring, context model and eight-byte spelling table. They establish no remaining time budget for building those inputs with this representation on the measured blocks. [Reusable results](reuse/summary.json) · [Fresh results](fresh/summary.json)

| Same-panel paired CPU ratio | Reusable storage | Fresh storage |
|---|---:|---:|
| Split terminal / complete scalar | 1.076 [0.995, 1.163] | 1.009 [0.964, 1.055] |
| Order acquisition / complete scalar | 0.447 [0.410, 0.487] | 0.411 [0.395, 0.429] |
| Acquisition plus storage release / complete scalar | 0.447 [0.410, 0.488] | 0.413 [0.397, 0.429] |
| Recovered-clock emission / complete scalar | 0.623 [0.570, 0.681] | 0.593 [0.561, 0.627] |
| Split terminal / fused physical terminal | 1.046 [0.954, 1.146] | 1.080 [1.031, 1.132] |
| Split terminal / supplied chronological terminal | 1.621 [1.429, 1.839] | 1.682 [1.598, 1.769] |
| Supplied chronological terminal / complete scalar | 0.664 [0.556, 0.792] | 0.600 [0.558, 0.644] |

The median chain walk over ten blocks took 11.396 ms in the reuse panel and 10.482 ms in the fresh panel. Median recovered-clock emission took 16.587 and 14.878 ms respectively. The bare emitter on the just-produced list divided by the identical supplied-list evaluator gave 0.938 [0.842, 1.046] and 0.989 [0.940, 1.040]. Phase and total ratios have individual paired denominators and their geometric means need not add. The two storage policies ran in separate panels; the fresh-versus-reuse difference includes host variation. Both are closed here without tuning. [Reusable phase timestamps](reuse/stamps.csv) · [Fresh phase timestamps](fresh/stamps.csv)

## Input, output and storage

Each ordered pair is two 32-bit words holding the absolute backward clock and the packed exceptional byte and poststate. Starting at the physical primary's anchor, the walk appends the current pair before advancing by its positive return distance. An artificial primary advances elapsed distance exactly once and contributes no pair. Stop at the next visit to the primary, pass that lap length and the ordinary compact clock to the unchanged chronological jump8 evaluator, and repeat the forward lap to n if necessary. Consequently events on other physical cycles supply no output, and an event at time zero is included exactly when the primary is a genuine exception. [Implementation](split.cpp)

The ten cases require 409,433 ring reads and 409,432 pair writes. Reuse retains one worker-wide vector at 413,176 bytes after preflight. Fresh storage allocates 286,256–413,176 bytes per call and releases it before the outer stopwatch ends. Both reserve the known number of physical anchors and write only genuine marked-lap events. The artificial primary accounts for the one spare pair on the largest block. The fresh allocator runs in the usual process and may reuse previously freed pages. The totals report direct vector capacities in addition to the physical ring, model, jumps and output already priced in the previous screen. Process high-water memory was not sampled in this panel. [Physical ring and base storage](../README.md) · [Per-block inputs](../inputs.json)

Before timing, 190,426 marked finite cases checked every emitted pair against a direct LF walk and every reconstructed output against forward Ψ, including 96,240 nondivisor horizons and 134,148 cases with exceptions on foreign cycles. Both genuine/artificial primaries and both ring layouts were checked. Every pair on every real case matched its authenticated original wire. The unchanged Rust driver checked all 4,320 timed output buffers against the original marked RLE1-coded bytes. Each panel includes 36 balanced rotations with a same-process physical fused arm, the supplied-clock terminal, and complete capped-input scalar-cap, LazyCells and OpenGated controls. Ratios use intervals from six chronological groups of six paired round log ratios. The physical fixture, model, and eight-byte table are supplied outside timing. The separately measured depth-eight physical-return constructor plus its clock traversal took 84.193 ms per ten-block median in its own panel. Neither study establishes i9 throughput. [Preregistration](PLAN.md) · [Finite gate](small.csv) · [Physical-return construction price](../../native/summary.json)

## Replay

The implementation and checked inputs were frozen at `27f475bc`. The panel links the original capped Rust binary and uses the input wires pinned by the earlier physical terminal. Raw Rust rows retain the donor arm labels. The report maps stream to fused, streamgaps to chronological, and dense to split, and joins the ordered timestamps against only the split samples. The first ten phase rows are preflight calls. [Source and executable hashes](build.json) · [Reusable raw rows](reuse/times.csv) · [Fresh raw rows](fresh/times.csv)

```sh
ref=/workspace/work/astra-powers-original/capped
wires=/tmp/astra-clock-wires
work=$(mktemp -d)
python3 run.py "$ref" "$wires" "$work" build
for mode in reuse fresh; do
    taskset -c 2 nice -n 10 python3 run.py "$ref" "$wires" "$work" gate "$mode"
done
for mode in reuse fresh; do
    taskset -c 3 nice -n 10 python3 run.py "$ref" "$wires" "$work" price "$mode"
    python3 run.py "$ref" "$wires" "$work" report "$mode"
done
```
