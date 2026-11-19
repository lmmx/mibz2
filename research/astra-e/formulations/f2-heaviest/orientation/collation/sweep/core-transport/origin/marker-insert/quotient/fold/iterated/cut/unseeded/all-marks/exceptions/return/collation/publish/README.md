# Publishing the marked true-exception BWT from sparse bands

Thirty ordinary weighted columns were constructed from original-RLE context ranges and physical return bands. Every incoming symbol, forward partner and class spelling matched the independent literal-word oracle. Each reconstructed original marked byte buffer also matched the pinned Rust inverse reference. At depth eight, ten blocks with 409,432 true exceptions produced 163,280 interval pieces and 148,077 maximal incoming runs. The publisher needs neither a chronological clock nor a per-event permutation. Its reference implementation pays 2.924 million ordered-key comparisons and 1.542 million base-sort comparisons at that depth. [Exact native implementation at 0e176f2e](publish.cpp) · [All thirty complete columns](columns.tar.xz) · [Per-block measurements](blocks.csv)

| Depth | True exception ranks | True return bands B | Source ranges S | Target-byte ranges D | Refined pieces Q | Bound, summed per block | Distinct classes | Incoming runs |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 4 | 1,122,585 | 269,447 | 248,744 | 239,186 | 322,322 | 757,357 | 219,840 | 296,655 |
| 8 | 409,432 | 127,359 | 110,003 | 103,903 | 163,280 | 341,245 | 95,150 | 148,077 |
| 16 | 229,013 | 101,275 | 86,599 | 82,711 | 125,900 | 270,565 | 78,606 | 115,962 |

## Publishing the incoming column

Number true event rows in increasing original physical rank. A backward return band sends [i,i+w) to [j,j+w) after d positive LF steps. The shifted section consists of physical LF(x) for the true event rows x. Its order is a stable sort on L[x], so within a constant-byte domain the shifted ordinal is the byte's total lower-bucket count plus the number of prior events carrying that byte. The native constructs these prefix counts on compressed domains. Intersect the original physical event ranges with the k-context partition before discarding physical holes, then coalesce equal descriptors on the event ordinals. This yields S source domains of constant (bᵢ,contextᵢ), and D destination domains of constant bⱼ. The poststate after bᵢ is fixed by that pair.

Cut each band at internal source-domain boundaries and at preimages of internal destination-byte boundaries. Each source boundary can add at most one cut globally. Since the bands' images partition the event ordinals, each destination boundary can add at most one. Alignments reduce the count. Thus Q≤B+S+D−2 on every nonempty input. In this first implementation, source descriptors stream; the initial destination domain for a band takes one binary lookup, followed by a forward sweep within that band. Its full lookup counts are in the receipt.

On each resulting piece, the forward source is shifted j and the forward destination is shifted i. The annotated return class is keyed by (d,bⱼ,bᵢ,contextᵢ). The contiguous-cylinder theorem assigns a single source interval to this class. Its base is the minimum shifted j among its pieces, and its width is their total mass. Append each class label and width to the bᵢ queue in increasing original i. Concatenating queues in byte order gives the complete incoming column. Coalescing equal adjacent records yields its maximal runs. The inverse builds cumulative per-class select records from the incoming runs and sparse class bases. Given a shifted source, select finds its forward destination. A key spells its leading bⱼ, followed by the reversed sequence of d−1 modal predecessor bytes starting from the stored poststate after bᵢ. The emitted slice is written directly into the final buffer. The dense reference alone materializes individual words and per-event links. [Independent cylinder proof and literal audit](../README.md)

## Removing the artificial primary and recovering byte phase

The RLE-based section overlay temporarily includes the original primary p, so the first-return band constructor covers its cycle. A genuine event primary requires no splice. For an artificial primary, obtain its unique predecessor and successor in a scan of the bands. Isolate the rank p on both axes with at most four boundary cuts, discard its outgoing singleton, and redirect its incoming singleton to its old successor with the summed positive distances. Compress source and target ordinals above p by one. Adjacent bands merge when both their translation and their distance agree. A singleton self-return at p identifies an event-free marked cycle; omit that edge and retain its ordinary LF lap length. The cycle emits the reversed lap of defaults beginning at the original marked state, repeated or truncated to n. Foreign cycles may still have true events.

For a genuine primary let j=p and h=1. For an artificial primary whose backward return from p reaches true j at distance a, let h=a+1. In either case the shifted event LF(j) is h forward steps behind p. The next shifted boundary has not been crossed. Equality h=d means the mark coincides with that boundary and consumes an empty initial suffix, then advances. Send the ordinal of j through the same sparse stable-byte translation that publishes the pieces. Emit the class word suffix starting at h, then full or truncated succeeding words. No original per-row map is needed for the phase query. At each real depth nine primaries were genuine and one was artificial. [Finite marks and foreign cycles](small.csv)

## Cost and exact checks

At k=8 the preexisting physical band induction took 2,350,011 packet steps. Publication then required 1,721,250 binary-search comparisons to enter the destination-byte domain, 2,923,961 key comparisons and 1,542,131 class-base sort comparisons. These exclude the original context construction, subsequent select and default output. The raw compact column totals 3,278,196 bytes across ten cases at depth eight, covering class records, incoming runs and phase. Original context transitions and select scratch are separate. No target throughput or process peak memory has been measured for this implementation. [All construction charges](summary.json) · [Original context costs](../../../contexts/README.md) · [Physical-return costs](../../native/summary.json)

The finite check covered all ternary columns through seven rows, depths 1–3 and every primary, for 9,837 models and 63,972 marked outputs. Of these, 23,760 marks had an entirely empty exception section, and 10,546 had nonempty exceptions confined to other cycles. Across the thirty true models the implementation verified 26,999,745 marked bytes, every class base and its full spelling, every incoming rank, and every induced forward partner. The independent file audit checks maximal coalescing, complete class masses, unique keys, weight/mark headers and each SHA-256. The archived objects are ordinary weighted BWT columns with no select table. Their little-endian wire has an eight-byte EXCCOL01 magic and five u32 fields (class count, run count, phase source, skip, event-free lap). Each 22-byte class record holds base, width, distance, physical source contextᵢ, bⱼ, bᵢ and poststate. Each incoming run holds class base and width as two u32s. [Object and reference identities](inputs.json) · [Independent wire audit](audit.py)

Reproduce from this directory. The context suite supplies the original compressed run texts, and the first-return suite supplies authenticated masks used by the dense oracle. The native publisher takes its model and bands from the original run text. The separate capped Rust donor supplies marked reference bytes already checked against libbzip2 plaintext and block CRCs. [Source and ABI hashes](build.json) · [Context input regeneration](../../../contexts/README.md) · [Mask regeneration](../../panel.py)

```sh
work=$(mktemp -d)
python3 run.py "$work" build
python3 run.py "$work" small
python3 run.py "$work" real /tmp/astra-exception-contexts/raw /tmp/astra-clock-wires /workspace/work/astra-powers-original/capped
cmp "$work/small.csv" small.csv
cmp "$work/blocks.csv" blocks.csv
cmp "$work/inputs.json" inputs.json
python3 audit.py columns.tar.xz | cmp - summary.json
```
