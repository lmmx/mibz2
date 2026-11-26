# Constructing the shifted induced BWT from original runs

A uniform three-byte shift of the named compound reduced original LF translation pieces by 8.1% on ten systematic Wikidata blocks. The constructor handles the 415–2,068 section components using one sparse global set of excluded ranks. Its fair comparison with the same single-interval storage on the unshifted control gave a local complete native inverse CPU ratio of **0.983 [0.947, 1.020]**. The exact reduction in discovery work establishes a mathematical and engineering milestone. The total inverse timing does not resolve the smaller performance difference. [Partition proof](proof.md) · [Paired native receipts](hull-paid.csv) · [Matched layout results](hull-summary.json)

## Input, construction and returned object

The input is the original L run table, marked row, original section U and desired shift h. Every stage through h must have a common outgoing byte. A histogram and a stable per-byte select-run directory build the successive Ψ images with their compressed positions. The named U is selected independently by the heaviest quote first-return search. This selector used three or four heap pops and 4,855–14,806 original run pieces per block. Both timing arms receive the same selected U. Shift construction itself is paid inside each shifted arm. [Exact inputs, primary hashes and selector prices](inputs.csv)

The LF scan precolors each translated piece by terminal membership in V_h. A separate cursor per byte follows the increasing translated positions and splits a run at crossed section boundaries. The clipped table covers every rank, costs at most r+2s records for s terminal components and supplies the compressed source base on a positive. During discovery a precolored original run tests a flag. Scanning an interval pays an LF page lookup and any increments through that 64-row page. The h known backward bytes at the root contract in one stage, conditional on where each destination first returns. Early words are published immediately. On the surviving ranks the residual search begins at U with the known spelling suffix. All ten Wikidata compounds support exactly the first three uniform shifts. All three contractions have survivors only. The small tests include overlapping early returns.

For each surviving spelling, scan its ranks in original order and collect continuing children by preceding byte. At a terminal, its V source ordinal is available from the precolored piece. All emitted fragments for a word tile its source bucket in increasing compressed rank order under the shift conjugacy. Publishing these bases and consuming each child stream in selector order produces the canonical incoming token BWT. The outer stream naturally admits the compact second-stage interchange. This reference reconstructs it using a charged m-entry destination permutation and an m-entry word directory, transfers the original mark and emits exactly n bytes. Disjoint cycles use original LF. All dictionaries, source bases, multiplicities and canonical incoming RLEs were compared to dense forward orbit truth. The canonical source-base token sequence, lengths and symbol frequencies are invariant across shifts, so a second index induced solely by those identifiers has the same unmarked input. Ambiguous ordering on a generic holed section is avoided by order conjugacy of these particular sections. [Constructor](induce.hpp) · [Dense object and marked checks](main.cpp)

## What the geometry costs

For any original live spelling interval J the fused state is J minus F_h, where F_h is the union of the first h section images. All original intervals are disjoint. If F_h has f components, the *entire job* uses at most f extra physical live intervals. A whole node needs only the bounds of its hull and a flag recording a gap. On a gapped hull, skip the intersected components of the shared F_h. The implementation constructs that union lazily. The initial packet implementation is preserved at `5d4edbd`, and the scalar-hull proof and constructor at `582f6e7`. [Full theorem, overlap and mark proofs](proof.md)

| Ten-block sum | Unshifted | One shift | Two shifts | Three shifts |
|---|---:|---:|---:|---:|
| Terminal-image rank components | 10 | 11 | 2,084 | 8,140 |
| Fused live spelling states | 1,296,362 | 1,257,715 | 1,219,068 | 1,181,129 |
| Extra physical intervals over surviving states | 0 | 0 | 0 | 435 |
| Old spelling classes partially cut | 0 | 1 | 2 | 711 |
| Precolored LF table records | 690,494 | 690,486 | 693,458 | 694,719 |
| Live rank occurrences | 8,999,915 | 8,754,470 | 8,509,025 | 8,263,580 |
| Translated LF pieces | 1,423,936 | 1,385,289 | 1,346,642 | 1,308,198 |
| Selector RLE entries | 1,423,936 | 1,385,289 | 1,346,642 | 1,308,130 |
| Non-aliasing merge appends | 424,200 | 424,200 | 423,492 | 421,882 |
| Materialized distinct dictionary bytes | 1,926,457 | 1,926,457 | 1,926,457 | 1,926,457 |
| Canonical incoming runs | 66,373 | 66,373 | 66,373 | 66,373 |

Three shifts produced 222 gapped hulls in total. Static skipping dropped 435 holes covering 3,399 interior positions. These counts exclude completely pruned old classes and gaps outside the retained hull endpoints. The ten masks contain 10,235 merged components. The geometry bound predicts at most 1,335,218 clipped LF translations versus the actual 1,308,198. The original node count reduction is paid for principally by fewer lookups and selectors. The dictionary, incoming-run count and output volume are still present. [All rankwise bounds and transferred marks](partition.csv) · [Per-block fused and unfused counters](real.csv) · [Independent scalar-hull objects](hull-real.csv)

## Paired price and memory

| Local thread CPU ratio (32 paired laps) | Estimate | 95% interval |
|---|---:|---:|
| Three shifts / named, both with packet lists | 0.934 | 0.907–0.962 |
| Scalar hull / packet list, both named | 0.762 | 0.736–0.789 |
| Scalar hull / packet list, both with three shifts | 0.802 | 0.772–0.833 |
| Three shifts / named, both with scalar hulls | 0.983 | 0.947–1.020 |
| Discovery and selector build only, three shifts / named, both scalar | 0.954 | 0.918–0.991 |

Each of the four arms occupies each position eight times, after one warm-up per block. Each observation sums all ten blocks in that lap. Intervals use Student t on the paired log ratios. The timer covers setup, marked reconstruction and destruction of internal scratch. Dense gold is compared outside it. The wall ratio with matched scalar storage is 0.980 [0.941, 1.020]. The earlier 24-lap depth panel stopped the timer after emission and before destruction. It priced shift depths separately with packet storage. Its h1/h2/h3 CPU ratios versus unshifted packet storage were 0.984 [0.924, 1.048], 0.940 [0.884, 0.999], and 0.941 [0.890, 0.994]. [All depth samples](paid.csv) · [Depth summary](summary.json) · [Complete late-scope samples](hull-paid.csv) · [Late-scope summary](hull-summary.json)

Maximum fresh-process VmHWM in KiB for packet storage was 27,616 / 25,832 / 25,308 / 24,776 at h=0/1/2/3. Scalar hulls used 22,444 at h=0 and 21,832 at h=3. These are one-process structural inverse peaks, including original run input and output. The memory process omits the dense verifier. The shipping worker's entropy/RLE/CRC stages, allocator workload and output concurrency require separate production-like measurement. The companion compact-kernel investigation can test whether projected ranks retain the measured lookup savings with lane scheduling, reusable buffers and second induction. [Packet memory](memory.csv) · [Scalar memory](hull-memory.csv)

## Tests and reproduction

Binary stable columns through eight rows and ternary through five, every nonempty contiguous section, every admissible depth through four and all marks gave 84,743 constructor objects and 576,684 marked outputs. There were 10,876 holed objects and 21,098 fusions with early returns. The partition verifier separately checked 301,054 small ranks and all 301,054 transferred marks, including direct LF and the h-step mark conjugacy. The scalar-hull verifier checked 40,846 objects, and the real verifier tested 70 block/depth/fusion combinations. The original tape inputs are ordinals 0 through 216 in steps of 24. C++ checks use a stable-sort oracle and compare complete bytes. The input adapter independently checks the marked SHA-256 against the archived dense Python inversion. [Small native gate](exhaustive.txt) · [Small masks and marks](partition-small.txt) · [Small hull gate](hull-small.txt)

```bash
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/shift-constructor
out=$(mktemp -d)
for src in main hull partition memory; do
    g++ -O3 -std=c++20 -Wall -Wextra -Wpedantic "$d/$src.cpp" -o "$out/$src"
done
"$out/main" | diff - "$d/exhaustive.txt"
"$out/hull" | diff - "$d/hull-small.txt"
"$out/partition" | diff - "$d/partition-small.txt"
python3 -B "$d/panel.py" research/astra-e/dense/target/wikidata "$out/panel" | diff - "$d/inputs.csv"
"$out/main" check "$out/panel" 1 > "$out/real.csv"
"$out/partition" "$out/panel" | tee "$out/partition.csv" | diff - "$d/partition.csv"
"$out/hull" check "$out/panel" 1 > "$out/hull-real.csv"
taskset -c 3 "$out/main" time "$out/panel" 24 > "$out/paid.csv"
taskset -c 3 "$out/hull" time "$out/panel" 32 > "$out/hull-paid.csv"
python3 -B "$d/memory.py" "$out/memory" "$out/panel" "$out"
python3 -B "$d/report.py" "$out"
python3 -B "$d/hull-report.py" "$out"
```
