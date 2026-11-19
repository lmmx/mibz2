# Output ownership with zero added cache capacity

The terminal grammar can reuse the dense event scheduler’s predecessor array for its output owners. The resulting complete inverse took **0.8841 [0.8254, 0.9469]** of unowned dense event CPU time and used exactly the same maximum retained vector capacity, 10,920,006 bytes. All 205,197,794 marked output bytes on 228 retained inputs matched the independent dense LF. [Complete paired observations](phase.csv) · [Calculated results](summary.txt) · [Byte and per-block capacity checks](gate.txt)

Each internal grammar production corresponds to one event that removes at least one distinct original token row. Therefore there are at most m−1 internal productions for m original rows. Successful terminal construction allocated at least m predecessor slots, which are dead as soon as the marked row becomes a fixed point. The writer lends this existing Vec to the unchanged first-complete-occurrence emitter, clears its length, initializes just the required prefix to the invalid sentinel, and restores the Vec afterward. The grammar remains immutable. The next constructor clears and repopulates the predecessor links. No new owner or validity storage is allocated. A mark outside the named section or a natural saturation takes the inherited fallback before borrowing. [Storage-only transformation](build.py) · [Ownership proof and precursor](../PROVENANCE.md)

At most 66,910 slots (267,640 live bytes) were repurposed in a block. The predecessor Vec’s retained capacity reached 1,085,728 bytes, all already charged to the dense constructor. The earlier separate cache needed 304,352 additional retained bytes. Across the full corpus sequence the borrowed-storage worker’s total retained capacity was no greater on **each** block than the dense predecessor’s. Fresh native peak readings were 9,444 KiB borrowed and 9,440 KiB dense, a difference of four KiB. The separate-table twins used 9,696 and 9,688 KiB. No precision about complete-worker or target-machine process peaks follows from this native comparison. [Borrowed memory](rss-3.csv) · [Unowned dense memory](rss-0.csv)

The same 32-round, ten-block, four-position native panel measured separate-table/dense at 0.8787 [0.8420, 0.9170], borrowed/first separate at 1.0061 [0.9329, 1.0850], and borrowed/second separate at 0.9964 [0.9124, 1.0881]. The same-code separate twin ratio was 1.0097 [0.9294, 1.0969]. All 1,280 outputs matched the oracle. The benefit established here is retained memory with the emission improvement over unowned dense; these intervals leave the relative speed of the two storage choices unresolved. The unchanged original named writer led the preceding full comparison, so the production throughput objective remains open. [Original named and indexed comparison](../README.md)

## Reproduction

Use the cap inputs and Rust 1.99.1 release library from the parent study. The builder verifies the hash of the complete generated parent sources before applying the lifetime-only change.

```sh
source /workspace/work/env.sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/origin/marker-insert/quotient/fold/iterated/native/event/ownership/reuse
dst=/tmp/astra-reused-owner-results
out=/tmp/astra-terminal-reuse
taskset -c 0 nice -n16 bash "$d/run-gate.sh" "$out" "$PWD/target/release/deps" "$dst"
taskset -c 0 nice -n16 "$out/reuse" price /tmp/astra-clipped/ten.bin > "$dst/phase.csv"
python3 "$d/report.py" "$dst"
```
