# Building the first weighted child directly from original byte runs

The construction found only mixed prediction contexts, formed their shifted minority-byte seeds, then published complete incoming return classes and original literal spellings directly from the original BWT. All thirty first-child columns and all 26,999,745 marked output bytes matched their archived references. This removed the complete k-context table, physical return map and chronological event list from the algorithm. Its local depth-eight time nevertheless reached **16.03 [14.89, 17.27] times** scalar-cap. Indexing and recognizing seeds alone took **11.88 [11.22, 12.59] times** scalar even when the final output was supplied. The forest and terminal took **4.58 [4.35, 4.82] times** scalar with correct seeds and navigation indexes supplied. Each favorable comparison lost on every rotation. [Complete source at 271aaa04](price/worker.cpp) · [Six-rotation results](price/summary.json) · [360 whole-call observations](price/raw/times.csv)

## Exact recognition at original L-run seams

Let I be the ordinary physical rank interval of a k-byte forward context u. It is mixed precisely when it contains both endpoints of at least one original incoming-byte run seam. Scan these seams in physical order. At each seam compare the future bytes of the adjacent ranks, taking Ψ steps until the first mismatch or k matches. A mismatch rules out the seam. On equality, backward-search u against the original RLE to recover the full interval I. Histogram its original-run intersections once and apply the reference tie rule of least byte at maximal frequency. Every later seam strictly inside I can now be skipped. Conversely the first seam inside any mixed interval survives the skip for earlier disjoint intervals and discovers it. Pure intervals need no tables.

For each minority byte a in I, the original LF images of those occurrences form one full physical prefix cylinder [au]. Its lower endpoint is the LF image of the first a seen in the histogram. Its width is that byte's count. For fixed a, different mixed intervals are processed in physical order, so the resulting seed cylinders are already sorted within their byte bucket. Concatenate the 256 buckets to produce the entire shifted exception section V. Preserve separately tagged seed cylinders even when adjacent. Repeat these arguments per rank with stable occurrence order on tied rotations. [Classifier at 6dbc48ae](seeds.hpp) · [Independent model checks](seed_driver.cpp)

| Depth | Original seams | Individually tested | Enclosed seams skipped | Compared byte pairs | Mixed k-contexts | Shifted seed cylinders | True event rows |
|---|---:|---:|---:|---:|---:|---:|---:|
| 4 | 690,474 | 495,187 | 195,287 | 1,848,118 | 97,685 | 162,428 | 1,122,585 |
| 8 | 690,474 | 570,168 | 120,306 | 2,448,733 | 23,496 | 49,972 | 409,432 |
| 16 | 690,474 | 596,436 | 94,038 | 2,922,675 | 18,466 | 40,206 | 229,013 |

At depth eight, 23,496 of the 815,844 original fixed-k contexts were mixed. The small number of output intervals left a substantial recognition cost. Each compared byte pair takes two Ψ location queries. The 4,897,466 such queries used 30,765,797 binary comparisons inside 256-row page brackets. Recovering each successfully recognized interval used 375,936 FM endpoint queries and another 3,753,516 comparisons. The final histograms visited 167,298 original-run pieces. The per-block receipt includes the number of first mismatches at every tested byte depth, even unsuccessful seams. It agrees with the independent full context model on every mixed interval, exact mode, minority origin, seed extent and event mass. [Thirty complete recognition ledgers](seed-blocks.csv) · [All depths and aggregate counts](structural.json)

## From seeds to the ordered weighted column

Start at each destination seed [P], a cylinder of length k+1. At a current original interval [vP], intersect L with its physical ranks. For every occurring predecessor byte b, its LF images are the single child [bvP], with a computable first image and width. Because this prefix fixes the next k+1 bytes, the child is either entirely inside one tagged seed or entirely outside V. In the first case it is a class cylinder of width equal to the child interval, with sparse F base given by the containing seed's prefix mass plus the distance from its physical beginning. Its literal is the reverse of the traversed predecessor bytes. In the second case continue. This constructs the literal dictionary using original input bytes directly, with no original default transitions.

Every source row in a processed interval has a unique first future seed, a positive distance if it is outside V, and a unique intervening spelling. Rows in V begin exactly one root. Induction stops on the positive return, so the processed original intervals are disjoint and cover exactly the cycles meeting V. This shows termination on the finite permutation even with tied rotations. A source run seam strictly interior to one interval cannot be interior to another. Accordingly the total number of original L-run intersections is at most the number of processed nodes plus R−1 for R original runs, and their total row widths are at most n. The number of *nodes* can still grow to the full original population. Long unary paths survive this intersection bound.

A leaf supplies a constant class stream in the order of its physical endpoints. For an internal node, stable LF means that occurrences of each byte b feed its child in child-rank order. Scan the actual node's local L-run schedule, pulling each width from the corresponding child's output, and coalesce equal adjacent class IDs. At a unary node the child's order passes through unchanged. Concatenate the root streams in physical seed order. The result is the complete incoming column in shifted destination order. Counts and child dictionaries alone do not establish that order. On the smallest ternary column exhibiting the branch-sorting failure for k=1 or k=2, L=011000 at k=1 has shifted section [4,5]. Its source literals are 100 and 10 and its true child incoming class bases are [1,0]. Sorting the predecessor branches gives [0,1] and changes the two-cycle into two self-cycles. [Witness search](order_witness.py) · [Direct forest at 9bbb7fba](forest.hpp)

| Depth | Processed intervals | Unary | Branching | Original-run intersections | Stable-merge claims | Maximal output runs | Literal bytes copied |
|---|---:|---:|---:|---:|---:|---:|---:|
| 4 | 1,098,197 | 1,072,700 | 25,497 | 1,239,962 | 195,561 | 296,655 | 1,274,961 |
| 8 | 1,742,761 | 1,723,640 | 19,121 | 1,849,316 | 157,932 | 148,077 | 2,206,359 |
| 16 | 2,275,153 | 2,258,706 | 16,447 | 2,357,789 | 130,153 | 115,962 | 2,886,547 |

At depth eight, the original covered mass was 8,999,915 bytes; only 157,932 width pulls were needed for all nontrivial stable interleavings. Maximum simultaneously live **intermediate** merged records was 1,433 on a single root. That count excludes the output RLE, class dictionary, original indexes, forest branch nodes, live path and allocator capacity. Membership classification of 1,787,939 child intervals took 3,475,363 comparisons in seed page brackets. The 1,723,640 unary intervals still needed predecessor and section queries one step at a time. [Per-block actual forest charges](forest-blocks.csv)

## Mark and verification

While visiting an interval, test once whether it contains the original primary. If the mark is in V, start at its direct shifted ordinal. Otherwise the forest gives its exact forward distance g to the first true seed. Follow Ψ for g steps from the mark to locate its row within the known seed. Its incoming child label and prior occurrence count recover the preceding class's source rank. Emit from offset d−g in that class of length d, then follow the ordinary weighted child. If the marked cycle never reaches any seed, no visited interval contains it. Follow original Ψ from the primary for its one exact lap and repeat or truncate its forward bytes to the requested length. No other cycle enters this fallback. On the ten depth-eight cases the residual phase cost was 226 original Ψ steps total. On one real case per depth a mark exactly at a shifted boundary was encoded as (next row,0) in place of the archived equivalent (prior row,d); the audit checked the partner and offset explicitly.

Recognition passed 49,200 independent ternary-column/depth cases. Full forest publication and marked expansion passed 85,296 cases including all marks on all ternary columns through seven rows at four depths. Across the ten real blocks at k=4/8/16 it matched each archived class base, mass, every literal byte and every maximal incoming class-base/width record. All 30 original buffers matched the pinned references with separate libbzip2/CRC lineage. [Finite recognition](seed-small.txt) · [Finite full publisher](forest-small.txt) · [Real canonical objects and independent hashes](../publish/inputs.json)

## Paid failure and precedent

The six-rotation complete arm started with original cap records, built both original navigation indexes, discovered its true seeds, published the literal dictionary and ordinary column, and emitted all marked bytes. The complete median was 429.93 ms per ten blocks versus 27.04 ms for scalar. In the fully measured arm indexing took 9.19 ms, seam recognition took 302.61 ms, direct forest construction took 110.65 ms and marked terminal output took 6.92 ms, as per-phase medians. The earlier registered terminal with the literal bank supplied established 16.95 [16.21, 17.69] ms of paired acquisition budget across ten. Both dominant stages individually exhaust that margin by several multiples. The implementation keeps worker buffers for the input navigation geometry and packed terminal and allocates new seed and forest results per block. The finite test uses unrestricted class/rank widths. The paid frozen corpus used the verified 16-bit companion fields for its first child. This local screen covers no process RSS, complete decoder release path, CI, or target i9 comparison. [Phase timestamps](price/raw/phases-2.csv) · [All rotation contrasts](price/summary.json) · [Previously measured terminal and budget](../recursive/native/README.md)

The conjugate multi-root F2 forest had already established stable interleaving from each root's incoming-run schedule, destination tagging, and direct construction on physically selected cylinders. The new elements here are exact selection of *all* true fixed-k exception cylinders from original run seams and direct emission of the complete modal residual and literal dictionary from that particular forest. The previous domain-based predictor forms arbitrary safe partitions; this experiment keeps each original fixed-k modal decision exactly. The prior interval-return and linear-radix publishers start after finding original contexts or affine return bands. This route eliminates those supplied objects but pays heavily for select navigation and unary paths. Cheaper selection would need to batch or avoid the unsuccessful adjacent-future tests. Cheaper publication needs to skip entire homogeneous first-return substrings while proving no seed boundary was crossed. Reducing the already small stable-merge count alone cannot address the measured losses. [Original F2 union at e3be3a0d](../../../../../../../../../../../../../../../../union/native/README.md) · [Interval-state formulation](../interval-states/README.md) · [Affine-band publisher](../publish/radix/README.md)

Reproduction starts in this directory and uses the previously authenticated original run tapes, context files, and Rust capped decoder. The context files and archive are oracle inputs only for the structural validator. Neither is loaded by the complete native price arm.

```sh
raw=/tmp/astra-exception-contexts/raw
ctx=/tmp/astra-exception-contexts/data
ref=/workspace/work/astra-powers-original/capped
work=$(mktemp -d)
python3 seed_check.py "$raw" "$ctx" "$work"
python3 forest_check.py "$raw" "$ctx" "$ref" "$work"
cmp "$work/seed-blocks.csv" seed-blocks.csv
cmp "$work/forest-blocks.csv" forest-blocks.csv
python3 report.py | cmp - structural.json
python3 order_witness.py | cmp - order_witness.txt
python3 price/run.py "$ref" "$work/price" build
taskset -c 1 nice -n 10 python3 price/run.py "$ref" "$work/price" gate
taskset -c 3 nice -n 10 python3 price/run.py "$ref" "$work/price" time
python3 price/run.py "$ref" "$work/price" report
```
