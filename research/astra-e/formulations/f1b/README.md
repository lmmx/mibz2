# F1b. Sparse bitmaps for direct construction of the anchor return

F1 at `9f477db` verified that the open endpoint bands and thin-run select intervals compose to a residual exchange but found that explicit 12-byte pieces and the select queues cost more than the dense retained table. This implementation builds the same labeled map T from the capped input with a bitmap and a packed entry per interval start. It stores neither the retained-row successor array nor the queues of thin-run selects. The research candidate retains opend's density gate, fallback and width-16 complementary endpoint elimination.

## Direct construction

Traverse the final endpoint nodes once in bottom order. Record each top band's first bottom coordinate and its nonliteral bottom intervals as compact anchor ranges. Include the primary. The top band endpoints remain sorted by top coordinate. Scan the original run records in L order, skipping unretained rows and following one monotone top-band cursor per character. A thin-run/top-band intersection yields a consecutive bottom domain, a consecutive retained L image and a constant label. The intersections partition the m residual rows because stable sorting and the open return are both bijections.

One scan sets a bit at the first bottom row of each intersection. It additionally cuts around the primary when its label is literal. One forward/reverse pass over pages supplies the number of preceding start bits and the next nonempty page's first start. A second run-record scan emits one affine payload per intersection and writes it to the ordinal given by the bitmap rank. This implementation counts naturally adjacent but separately constructed intersections as separate starts. The full cost includes the original endpoint's first scan of records, two further run scans, 256 top-cursor lookups per additional scan, one bitmap-rank computation per output intersection, the page passes and every macro-tracing query.

A 64-row page contains its 64-bit cut mask and two u32 values. A query uses its row's page, POPCNT of the low mask and a payload lookup. TZCNT on the remaining mask, or the cached next start, gives the right slack. Page construction guarantees one initial boundary. The packed word stores the displacement modulo the next power of two in its low bits and the word code in its high bits. Each translation lands inside the residual rank interval, so modular addition recovers that target. Wide codes use a separate entry per cut. The mark check supplies a distinguished literal singleton as needed.

Macro tracing uses the same minimum-right-slack proof as F1. It stores the initial word, exit displacement and tail end in each 12-byte macro record. A second cut bitmap over compact anchor ranks retrieves this record on the final marked walk. Both indexes use two dependent loads in the packed case. These are real construction and output dependencies. Periodic output is repeated and truncated to the original length. Reusable vectors include endpoint scratch, top bands, both indexes, both payloads on wide blocks, anchor ranges, macro records and literal-tail bytes.

## Exact screen

```sh
d=research/astra-e/formulations/f1b
cargo run --release --locked --manifest-path "$d/Cargo.toml" -- small
sha256sum corpora/wikidata.bz2
cargo run --release --locked --manifest-path "$d/Cargo.toml" -- corpora/wikidata.bz2
```

On the canonical corpus (SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`), all ten systematic samples match Plain, including frozen. The 55,302 exhaustive small binary column/mark/cutoff combinations also pass. The raw outputs are `blocks.csv` and `small.txt`.

| Count or bytes | Frozen block | Nine admitted large samples combined |
|---|---:|---:|
| Retained rows | 89,874 | 956,546 |
| Packed interval payloads | 47,908 | 531,128 |
| Macro bands | 8,079 | 90,292 |
| Compiling queries including stops | 69,235 | 761,768 |
| Marked anchor visits | 15,278 | 175,410 |
| Sparse residual index bytes | 214,112 | 2,363,728 |
| Macro program bytes including bitmap/tails | 153,849 | 1,708,624 |
| Old dense residual table bytes | 359,496 | 3,826,184 |

Relative to the old dense edge table, the residual index consumes 0.618 times the logical bytes on the admitted samples. During macro compilation the sparse index and the program coexist. Their combined logical size exceeds the old edge table alone, and the other buffers differ across algorithms. These counts omit allocator-capacity growth and the other decoder stages. The exact 81.7% reduction in marked program steps across the nine blocks accompanies 761,768 construction queries and the two record scans. That reduction cannot establish a wall-time gain or a whole-process memory win.

The scorer entry is `research/astra-e/formulations/f1b/candidate.rs`. This kernel runs its production form with all structural counters compiled out. The ordinary testing driver selects those counters at compile time. The standalone source is ready on checkout.
