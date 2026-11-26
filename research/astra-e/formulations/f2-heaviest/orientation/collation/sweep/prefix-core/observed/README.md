# Recording only observed holes during prefix-core discovery

The constructor can eliminate the historical rank ranges and the entire global mask sort. At h=10 it used **0.920× [0.879, 0.963]** the CPU of the depth-matched global-mask constructor with the same actual Compact second-stage writer. Its CPU ratio versus native Compact on the original named section was **5.303 [5.036, 5.584]**. The result preserves a faster implementation of this experimental construction. Its measured first-stage cost ends lookup tuning of the standalone path. [Paired receipt](paid.csv) · [Confidence intervals and phase totals](summary.json)

## Where each gap comes from

After clipping positive terminals, a parent enumerates its remaining input ranks in increasing order. Stable LF preserves that order within a byte. The segments contributed to one continuing child are therefore disjoint and increase even when pieces directed to other children intervene. Keep the first and last endpoints on the child. On an append [a,b) following previous end z, equality a=z extends the current segment. A strict inequality z<a supplies the exact missing interval [z,a) and appends one gap record. When the child runs, stream its hull in order and skip these recorded gaps. At the root the same rule processes survivors around zero-length cores. By induction each emitted segment sequence is exactly the global-mask implementation's live sequence. Recognition uses only two endpoints already produced by the constructor.

Each node stores a tail index in place of its former gap flag. The node's hull still occupies twelve bytes. One global array holds twelve-byte (lo,hi,next) records. Each node's records form a circular list with the oldest record following its tail. An append or pop costs constant work per encountered gap. Across the complete job the number of records is precisely the number of extra live components. The earlier shared-set theorem bounds this by the number of components of the hypothetical global excluded set. The observed layout never constructs that set. In particular it keeps zero intermediate image ranges after they have produced the next forward layer. Final prefix/owner annotations must still be ordered by physical rank to classify LF terminals. [Source](../core.hpp)

| Ten-block costs at h=10 | Global mask | Observed gaps |
|---|---:|---:|
| Historical input intervals retained to form the mask | 169,883 | 0 |
| Final prefix/owner tag sort comparisons | 428,155 | 428,155 |
| Additional global mask sort comparisons | 2,942,722 | 0 |
| Gapped live states / holes consumed | 1,324 / 20,418 | 1,324 / 20,418 |
| Posted gap records | 0 | 20,418 |
| Logical next-link reads / writes | 0 / 0 | 39,512 / 39,512 |
| Backward interval searches | 966,263 | 966,263 |
| Clipped LF pieces | 1,076,917 | 1,076,917 |
| Selector runs | 1,063,421 | 1,063,421 |
| Full spellings / incoming runs | 38,648 / 66,373 | 38,648 / 66,373 |

The maximum observed gap-array capacity on one block was 98,304 bytes. Final tags and prefix labels, the forward select-run directory and the backward LF table remain charged. The oracle checks that gap records posted equal gaps consumed, and that the cumulative global-mask input is empty in every observed run.

## Matched complete native comparison

All arms start with the same original named compound U and actual primary. The two C++ layouts share the same constructor source and word/selector representation. Their canonical incoming word handles carry original U source bases. They transfer the original mark and skip. The checked Rust bridge takes the root runs, bases and prefix/core slices, materializes the full dictionary, then executes the actual native Compact second induction and writer. The stage timing includes assembling the C-compatible word views and populating these Rust buffers. Every real case used second induction. The independent Compact control uses its native optimized single-symbol first-stage convention on the same U and primary, including its internal one-byte word rotation. Its run input is prepared in its normal 127-byte capped representation. The C++ constructors receive already combined original runs. Input conversion, selection of U and the dense verifier are outside every timed arm. [Bridge](bridge.rs) · [Full harness](observed.cpp)

| h | Observed / global at same h, CPU | Observed / observed h0, CPU | Observed / native Compact, CPU |
|---:|---:|---:|---:|
| 0 | 1.096 (0.998–1.204) | 1.000 | 6.112 (5.544–6.738) |
| 3 | 1.029 (0.952–1.112) | 0.913 (0.837–0.996) | 5.580 (5.102–6.102) |
| 8 | 0.926 (0.839–1.022) | 0.877 (0.795–0.968) | 5.363 (5.015–5.735) |
| 10 | 0.920 (0.879–0.963) | 0.868 (0.793–0.949) | 5.303 (5.036–5.584) |

Against the initial global h0 control, observed h10 was 0.951 [0.879, 1.029]. No gap work is required at h0, and the interval for the layout difference there is wide. The two depth-matched h10 layouts have identical forward ownership bands, selectors, words and root runs.

At h10 the observed arm averaged 31.17 ms of setup, 134.56 ms of discovery plus mark propagation, 2.32 ms of materialization at the Rust boundary and 8.25 ms of second induction and writing, summed over ten blocks. Total mean thread CPU including scratch destruction was 180.08 ms. Native Compact averaged 33.69 ms for its complete ten-block inverse. Arithmetic stage means and paired geometric ratios summarize distinct statistics.

Each of nine arms received one warm-up and 36 measured laps per block and occupied every position four times. Each paired observation sums the ten blocks before computing its log ratio. Confidence intervals are two-sided Student t on those 36 log ratios. Output comparison was byte-exact and outside the timer. Both generic layouts allocate their first-stage scratch per invocation. Each arm retains its own supplied native Compact buffers across repetitions of that block, as the native implementation normally can. The timings cover marked BWT reconstruction. They give no 0.5.6 or i9 block-worker throughput claim.

Fresh maximum VmHWM at h=10 was 20,048 KiB observed and 20,484 KiB global. Native Compact reached 9,640 KiB. The fresh measurement process includes input runs, the linked C++/Rust runtimes, the actual output and any intermediate handoff buffers. The Rust capacity field counts only the retained consumer and output allocations. [All fresh-process peaks](memory.csv)

## Correctness and replay

All 51,340 admitted exhaustive objects and 353,846 marked outputs passed the original dense native gate with observed gaps. An extra 29,860 admitted marked paths were passed through the Rust interchange or the required disjoint-cycle fallback, including 10,037 paths using second induction and 7,788 disjoint marks. The ten-block gate checks both layouts against independently built dictionaries, incoming full/core columns and marks at depths 0/3/4/6/8/10/11. It verifies the rejection of 12 at 11 on all ten blocks. There were 3,240 measured real inverses with exact output. [Small receipts](small.txt) · [All depth checks](real.csv)

The build uses the checked shared Compact generator and the two SHA-256 fingerprints of its Compact and Second sources. From the repository root, use the installed Rust toolchain, existing release dependencies and a C++20 compiler.

```bash
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/prefix-core/observed
out=$(mktemp -d)
bash "$d/build.sh" "$out" "$PWD/target/release/deps"
python3 -B "$d/../../shift-constructor/panel.py" research/astra-e/dense/target/wikidata "$out/panel" | diff - "$d/../../shift-constructor/inputs.csv"
"$out/observed" | diff - "$d/small.txt"
"$out/observed" check "$out/panel" 1 > "$out/real.csv"
taskset -c 3 "$out/observed" time "$out/panel" 36 > "$out/paid.csv"
python3 -B "$d/report.py" "$out"
python3 -B "$d/memory.py" "$out/memory" "$out/panel" > "$out/memory.csv"
```
