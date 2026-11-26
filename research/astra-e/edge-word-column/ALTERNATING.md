# An ordinary weighted column using the two existing edge phases

Each existing band identifies a translation P from physical source x to consecutive queue ordinals. Q takes an ordinal to its actual retained row, stably within each original L byte. The old edge successor is T = Q ∘ P. Represent these two maps on 2m distinct rows, first all physical rows X and then all ordinal rows K. The outgoing weight on X is W with the last byte removed. The outgoing weight on K is its retained queue byte. Source intervals within X split whenever the truncated spelling changes or P has a descent. Within K, give each byte one source interval. Thus both transitions are strictly increasing on their assigned intervals. These two phases have disjoint ID namespaces even when two byte spellings agree.

The incoming on X is exactly the original retained byte tape, interpreted by queue-byte ID. The incoming on K is the physical X-piece ID of each band in the constructor’s existing queue order. The two streams concatenate. Constructing IDs requires physical band order, obtainable with the band-boundary rank bitvector in PROOF.md, and word-prefix comparison or safe numeric aliases. No dense old successor or partner acquisition appears in either stream. A marked X emits the concatenation W by two successive arcs. An empty X prefix on a literal is safe because the subsequent queue arc always emits one byte. Every original nonempty cycle lifts to an even cycle, including a fixed point, and every augmented cycle contains queue rows.

Across 233 admissions the constructor creates 384,646 X classes and 20,024 K classes. Its incoming has 14,291,364 low-byte and 584,712 high-ID runs. There are 21,672,913 zero-weight X visits. Mass-weighted zero-order class entropy is 7.629 bits per augmented row. It cannot be compared directly with 8.373 bits per original refined row without charging the doubled number of rows. These entropy figures omit boundary indices, dictionaries, frequency histograms and any navigation support.

| Forced section | Queries | Incoming-RLE intersections | Distinct second source-word bytes |
|---|---:|---:|---:|
| Heaviest X class | 42,163,080 | 46,370,800 | 206,531,512 |
| Heaviest K byte | 36,218,416 | 38,554,509 | 182,439,596 |
| Quote K byte | 42,858,009 | 43,742,605 | 188,868,148 |

All 233 original primary cycles reached each listed section. On an arbitrary input there can be disjoint cycles. A caller falling outside its chosen section needs an exact original marked traversal or a different section on that cycle. Distinct second dictionary bytes above count sums of expanded typed return spellings. A shared grammar might use less memory and must pay for its own construction, ordering and expansion. [Every augmented root](alternating.csv) · [Each second-section traversal](alternate-sections.csv) · [All marked checks](extra-check.txt)
