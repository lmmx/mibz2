# Native marked terminal on the first ordinary exception column

Starting with the actual depth-eight ordinary child, its weighted class descriptors, original default transitions, and marked phase, the terminal rebuilt every distinct class spelling, recovered marked class order and emitted the original coded bytes in **0.639 [0.582, 0.702]** of complete scalar-cap inverse CPU. Its highest observed fresh-process HWM was 5,852 KiB against scalar's 5,984 KiB. As a paired arithmetic difference, the remaining local CPU budget for forming all supplied inputs was **8.53 [6.84, 10.22] ms per ten blocks**. The original cap-to-context and physical-return stages have already exceeded that budget in their separate measurements. This result supports the weighted terminal and leaves raw-BWT-to-child construction as the limiting step. [Relative costs](summary.json) · [Paired absolute budgets](budget.json) · [All 1,440 native and Rust calls](raw/times.csv)

| Charged from the supplied first child | Median CPU per ten blocks | Paired CPU / scalar | Highest process HWM |
|---|---:|---:|---:|
| Stable scatter, class order, copies from a supplied literal dictionary | 6.714 ms | 0.285 [0.276, 0.294] | 5,484 KiB |
| Stable scatter, class order, rebuild every literal, copy output | 14.856 ms | 0.639 [0.582, 0.702] | 5,852 KiB |
| Stable scatter, class order, expand occurrences from supplied eight-byte jumps | 19.268 ms | 0.802 [0.719, 0.895] | 7,176 KiB |
| Complete scalar-cap from original cap records | 23.971 ms | 1 | 5,984 KiB |
| Complete LazyCells from original cap records | 25.295 ms | — | 9,184 KiB |
| Complete OpenGated from original cap records | 22.680 ms | — | 9,120 KiB |

The middle arm took 2.920 ms median to rebuild the stable successor table and recover the class cycle, 8.704 ms to produce each literal spelling once, and 3.342 ms to copy the marked output. Their paired fractions of scalar were 0.126 [0.112, 0.142], 0.367 [0.336, 0.402], and 0.143 [0.132, 0.154]. Across these ten blocks there are 409,432 child rows, 148,077 incoming runs, 95,150 weighted classes, 2,206,359 distinct spelling bytes and 8,999,915 final marked bytes. Further recursion can displace some of the 2.920 ms order stage. A faster first-child constructor has a larger opportunity. The arm with a supplied literal dictionary bounds the favorable case where the raw constructor emits the dictionary as a byproduct. [Phase samples, paid spelling dictionary](raw/phases-1.csv) · [Original model construction](../../../../contexts/README.md) · [Physical return construction](../../../native/summary.json)

## Exact terminal and inputs

The first child is the published incoming RLE with its class-base labels. The class dictionary provides its F base, weight d, first byte and final-boundary poststate. Load a cursor at each base. Scan each incoming (base,width) run in destination order, assigning consecutive (destination,class) entries at the cursor and advancing it by width. Stability of LF makes the resulting source-indexed table exactly the child's forward permutation. Both fields fit in 16 bits on all ten cases. A 64-bit pair handles populations or alphabets above 65,536. Follow the supplied source once around its marked cycle, producing a compact class list in order. Scatter, cursor initialization, list write and all required scratch growth occur in every priced treatment. The list contains only that cycle.

For each class, write its first byte, then its d−1 defaults in reverse position order, starting at the stored poststate. The paid dictionary arm performs these unit transitions once per class and then copies occurrences. If the first class has offset h, emit its suffix [h,d), each remaining full class and finally its prefix [0,h). This produces a forward lap starting at the true mark for both h<d and h=d. Repeat or truncate that lap to original n. For a mark on a cycle with no true exception, use its separately supplied original default lap and starting state. Arbitrary tests compare with an independently expanded and rotated cyclic tape. [Source at 246dfe88](terminal.cpp) · [Finite weighted oracle](small.cpp)

Input parsing and authentication occur in an untimed preflight. Each real conditional arm starts with a first reduced column and its original model's default transitions derived once from the authenticated CTXMOD01. Physical context ranges and ordered event times are absent from the model used by the priced terminal. The prepared-bank arm constructs its bank in preflight and releases unused default transitions when the mark has a section. The direct-rule arm builds its jump8 table in preflight. The paid-bank arm has neither side table at the start of a call. Original model formation, affine return construction, true-column publication, Huffman/MTF, RLE1 and CRC are outside all three terminal measurements. Complete controls start with the original capped input. Warm timing retains worker scratch capacity across blocks. The paid-bank worker's maximum retained scratch was 3,326,868 bytes and its maximum per-block input capacity was 1,171,692 bytes. The maxima need not occur on the same block. [Input identities](inputs.json) · [Per-arm acquisition budgets and capacities](budget.json)

## Validation and reproduction

All 1,440 timed outputs and all 180 fresh-process RSS outputs matched the complete marked-byte reference. The prior donor's plaintext SHA-256 and block CRC are recorded beside each cap, default model, first-column and marked-byte hash. The artificial/true primary belongs to the original sparse producer. This terminal verified its supplied phase by reconstructing every original marked byte. The finite gate included 62,192 weighted marked cases, 51,435 cases with foreign rows, 45,797 nondivisor outputs, 6,021 offsets at the right boundary, default-only marks and tests at 65,535/65,536/65,537 rows/classes. The fresh-process HWM rows include the Rust donor's cap extraction, C++ input and side preparation, terminal allocations, linked code and allocator behavior. Reusing these HWM values as full decoder memory would omit the upstream child's construction peak. [Full finite receipt](small.csv) · [180 RSS rows](raw/rss.csv)

The exact executable and source hashes are recorded in the build manifest. Results use 24 six-arm rotations, all ten blocks per round and a two-sided t interval on four consecutive groups of six paired round-log ratios. Every measured C++ arm was checked against that Rust process's scalar-cap, Lazy and OpenGated arms. The local data provide no CI or i9 production-speed verification. [Preregistration](PLAN.md) · [Build manifest](build.json)

```sh
ref=/workspace/work/astra-powers-original/capped
contexts=/tmp/astra-exception-contexts/data
work=$(mktemp -d)
python3 run.py "$ref" "$contexts" "$work" build
taskset -c 1 nice -n 10 python3 run.py "$ref" "$contexts" "$work" gate
taskset -c 3 nice -n 10 python3 run.py "$ref" "$contexts" "$work" time
taskset -c 1 nice -n 10 python3 run.py "$ref" "$contexts" "$work" rss
python3 run.py "$ref" "$contexts" "$work" report
python3 budget.py | cmp - budget.json
```
