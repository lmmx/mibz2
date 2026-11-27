# 2026-11-18 · A BWT of phrases that can be RLE1-decoded independently

The byte BWT contains a wavelet trie for a sequence of whole phrases. A single separator makes
that statement exact. Its routing decisions recover a smaller, ordinary BWT. Under a condition
testable in the compressed column, its dictionary can also hold the final RLE1-decoded bytes. A
decoder can therefore emit verified bzip2 output without constructing the full intermediate
inverse-BWT string.

The native prototype decoded the complete pinned Wikidata corpus. It reduced the allocation peak
of a fresh block decode and lost the wall-time comparison with Cells. A supplied phrase multiset
determines every nonbranching suffix bucket. Only the branching buckets carry occurrence order.
Discovering the multiset from compressed input still accounts for work in both constructors.

The accompanying Python program verifies both directions of the factorization independently of the
native code. The Rust patch, raw results and reproduction commands are kept alongside this entry.
[Experiment and reproduction](2026-11-18-phrase-bwt-rle1/README.md) · [Independent reference](2026-11-18-phrase-bwt-rle1/verify_phrase_bwt.py)

## The return words have their own BWT

Write L for the byte last column and F for the sorted first column. Use mibz2’s convention that
the marked row p starts the forward output. Pick a byte c. Cut each cyclic text immediately after
each c. A token is u followed by c, where u contains no c and may be empty. These tokens form a
prefix-free code. Comparing two rotations at boundaries compares their first tokens, in ordinary
byte lexicographic order, and continues to the next token on a tie. Equal periodic rotations
retain stable occurrence order.

The boundary rows are precisely those with L equal to c. One LF step takes them, in order, onto
the contiguous F bucket for c. Work on this bucket and number its rows 0 to m−1. From each row,
read L and follow LF until reading another c, including that last step. The interior bytes read
backwards give u. Record the token uc in source-rank order. For a single cyclic input the
resulting sequence U is the last column of the token BWT. For a general last column the same
construction gives the stable LF permutation on the cycles that visit c. The destination is its LF
image. For equal tokens, every byte step preserves occurrence order. Their destinations form the
contiguous interval of rotations beginning with c u c. Those intervals also give the lexical token
IDs by destination order.

This builds directly on mibz2’s delimiter first-return experiment. It had already verified the
token LF on its frozen block. The new uses here are the factorization into stable partitions,
recursion with the marked phase, and the independent RLE1 evaluation. Prefix-free parsing supplies
the order correspondence, and the wavelet trie supplies the stable routing interpretation.
[mibz2 technical companion](https://github.com/lmmx/mibz2/blob/f396ccc7b3d4290468b12e4ccdfcfcfe7708f3b2/docs/site/blog/2026-11-12-rauzy-technical-companion.md) · [Earlier frozen experiment](https://github.com/lmmx/mibz2/blob/bf88365ba80388e17488a31c80d2f3f250b19fb2/docs/journal/2026-11-11-first-return-block/README.md) · [Boucher et al. on prefix-free parsing](https://pmc.ncbi.nlm.nih.gov/articles/PMC6534911/) · [Grossi and Ottaviano on wavelet tries](https://arxiv.org/abs/1204.3581)

### The mark belongs to a boundary and an offset

Follow LF from p until L is c. Let t be the number of steps taken before that row and z its
zero-based rank among the c entries of L. The token primary is z. Its first token starts t old
symbols before the required output. On the selected cycle, expand from that boundary, skip t old
symbols once, and take the requested length. For recursion, extract the selected primitive cycle
of row visits at the innermost level. Expand one cycle at a time and correct its phase at that
level. Repeat or truncate at the outer boundary. If the selected cycle misses c, this
factorization has no marked root and the prototype uses a conventional inverse.

For example, the text `b#z#ab#z#y#a` has L equal to `yzzbb##aa###` and primary 7. With marker `#`,
the sorted dictionary is `ab#`, `y#`, `z#`. Its token BWT is `1 2 2 0 0`, with primary 0 and
offset 1. Its forward token cycle spells `ab#z#ab#z#y#`. Starting one byte into that cycle
recovers the original. The offset matters even without periodicity. For arbitrary columns, other
LF cycles can carry different words and the requested block length can end partway through the
selected cycle.
[mibz2’s marked-cycle convention](https://github.com/lmmx/mibz2/blob/f396ccc7b3d4290468b12e4ccdfcfcfe7708f3b2/docs/site/blog/2026-11-16-mathematical-dictionary.md)

## The routing information in the original column

Let I(w) be the row interval of text rotations starting with w. All strings w = s c with no c
inside s are prefix-free as a family. Their nonempty intervals partition the rows of cycles that
contain c, according to the first delimiter in the forward direction. At I(w), the bytes of L say
how to route its occurrences. A preceding byte a other than c sends its occurrences, in order, to
I(a w). A preceding c terminates the phrase w. Starting at I(c) gives a multiway wavelet trie of
the reversed phrase interiors with explicit termination. A unary node spells a byte and makes no
choice of occurrence order.

There is an exact information count. Let Nᵤ be the number of occurrences reaching node u and tᵤ
the number terminating there. For fixed phrase types and multiplicities, the possible local stable
shuffles telescope.

```math
\prod_{u} \frac{N_u!}{t_u!\prod_{v\ {\rm child\ of}\ u} N_v!}
  = \frac{m!}{\prod_{u}t_u!}.
```

The right side counts all orders of the multiset of preceding phrase IDs. The inverse construction
takes such an order and stably partitions it at each node, appending the routing bytes to that
suffix’s bucket. Concatenating the buckets in suffix order reconstructs the byte column when all
cycles contain the marker. Requiring a single LF cycle imposes an additional restriction on
allowed orders. The formula counts the full multiset ordering before imposing that restriction.

On the frozen block, supplying just the 2,716 phrase types and their multiplicities lets an
independent Python program recover the entire phrase BWT by reading 1,058 mixed buckets. They
contain 363,378 byte positions in 12,361 run fragments. Its only input reads are in those buckets.
Running the stable routing in the other direction reproduced all 899,931 original column bytes.
The native bzip2 decoder discovers its own dictionary and counts. A supplied multiset is a
controlled experiment in which information the order requires.

### Two constructors and a bound

The grouped constructor carries increasing spans of original delimiter ranks through each distinct
suffix interval. It spells a unary path in a loop and distributes spans only at a mixed node. All
nodes together encounter at most r + D − 1 run fragments when D intervals partition the block and
L has r maximal runs. Cutting disjoint interval boundaries into an ordered run list proves the
bound. The prototype still pays for locating each interval and for distributing spans. The frozen
block has 70,560 distinct suffixes and 81,863 run intersections. The earlier experiment’s 70,559
new grammar nodes were already capturing essentially the same distinct spellings.

The second constructor enumerates the runs of the token BWT directly. Start at the leftmost
as-yet-unassigned delimiter rank. Follow its left context to the previous delimiter, taking the
minimum of the remaining bucket width and the number of rows remaining in each visited byte run.
Call this minimum w. A translation preserves the row offset at each LF step, so all w source rows
carry the same phrase and adjacent destinations. At a limiting run end the next row differs at
that step. This makes the width maximal. Emit one token run, advance by w, and repeat. Its code
needs one path under construction. If the emitted runs have widths wⱼ and return heights hⱼ, their
weighted heights sum to the covered rows. The representative LF work is the unweighted sum of the
heights. The weighted identity follows because each row between consecutive visits to the marker
bucket belongs to exactly one return tower.

```text
s = 0
while s < number_of_markers:
    x = start_of_F_marker_bucket + s
    width = number_of_markers - s
    backwards = []
    loop:
        width = min(width, exclusive_end_of_L_run(x) - x)
        a = L[x]
        x = LF(x)
        if a equals marker: break
        append a to backwards
    emit (reverse(backwards) followed by marker, width, x - bucket_start)
    s = s + width
```

If the byte BWT has r runs, the phrase BWT has at most r runs. At each change between consecutive
phrase labels, follow the gap between those adjacent root rows backwards to their first different
preceding byte. Equal bytes transport adjacent rows to adjacent rows, giving a partial injection
on gaps. A first mismatch is an original L run boundary. Two differing root gaps cannot reach the
same first mismatch. The partial inverse is unique and before mismatch each common extension is a
nonmarker, so the path cannot revisit the delimiter bucket. This injects the q − 1 internal phrase
changes into the r − 1 raw changes. With one occurrence of c in a primitive block, the one
representative can still take the full n steps. A small residual offers no guarantee of a cheap
constructor. If each level selects a marker with frequency at most half the rows, the sum of all
representative heights over the recursion is less than 2n, by the geometric series. This counts LF
steps. Run lookup, dictionary construction and copying have their own costs. The Rust
implementations also use page samples and temporary lists.
[mibz2’s itinerary certificates](https://github.com/lmmx/mibz2/blob/a2ddc1ef3dd26e8400cef0323e300baf76b26e7d/docs/journal/monodromy/2026-11-16-certificates.md)

## A boundary at which RLE1 becomes independent

Suppose cccc is absent from the cyclic pre-RLE text. At a cut just after c the RLE1 decoder is
either fresh or has counted one to three copies of c. It might be fresh because c was a count byte
for four copies of a different byte. If the next byte differs from c, it starts a new literal run
in either case. If more c bytes follow, the forbidden quartet ensures all of them are literals
until the first change. Consequently splitting at any such cut and decoding both pieces from fresh
gives the same bytes, or the same terminal unfinished-run error, as decoding the joined input.
This also applies to the partial first and last phrases in the required linear output.

Three FM backward extensions from I(c) decide whether I(cccc) is empty. This check is sufficient
and can conservatively fail on another cycle or across the cyclic block boundary. On success,
decode each distinct phrase once, keep its decoded payload, and expand the token program directly
to final bytes. The grouped Rust implementation composes two dictionary levels and checks the
usual block CRC over the emitted output. The scanner retains the stream CRC check.

For a positive edge case, `aaaa"` spells 38 copies of the byte `a`. The delimiter itself supplied
the count of 34. For a counterexample, with marker `!`, the complete input `!!!!A` spells 69
exclamation marks. Independent decoding of singleton exclamation marks would lose the count. For
full streams the prototype also requires mibz2’s run-based front end and a marker on the selected
cycle. It falls back to the production decoder if these conditions or the RLE1 test fail. The
project already has a general finite-state RLE1 and CRC algebra on a binary grammar. The delimiter
condition removes entry-state bookkeeping for these phrases.
[Existing transducer experiment](https://github.com/lmmx/mibz2/blob/f396ccc7b3d4290468b12e4ccdfcfcfe7708f3b2/docs/journal/monodromy/2026-11-10-rauzy-veech/rle1_crc_on_grammar.py.txt) · [Production RLE1](https://github.com/lmmx/mibz2/blob/f396ccc7b3d4290468b12e4ccdfcfcfe7708f3b2/src/rle.rs)

## Results on the pinned Wikidata corpus

The stream contains 431 blocks across 12 independent Wikimedia windows. Its SHA-256 is
`82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`. Both fused implementations
matched libbzip2 byte for byte on all 209,700,196 output bytes. They used the fused route on 367
blocks, including every one of the 233 blocks of at least 65,536 pre-RLE rows. The quote marker
came from the previous delimiter experiment. Later levels chose the most frequent eligible token
with at most half the rows.
[Pinned input specification](https://github.com/lmmx/mibz2/blob/f396ccc7b3d4290468b12e4ccdfcfcfe7708f3b2/bench/wikidata.json)

| Alphabet | Rows on the 233 large blocks | BWT runs | Rows on frozen block |
| --- | ---: | ---: | ---: |
| Bytes | 209,697,662 | 15,141,602 | 899,931 |
| First phrase level | 31,566,556 | 1,862,456 | 135,716 |
| Second | 5,698,486 | 1,489,653 | 24,165 |
| Third | 720,879 | 461,746 | 3,232 |
| Fourth | 72,385 | 72,341 | 241 |

The four-level representative constructor performed 52,472,423 representative LF steps across
these blocks, in addition to phase finding, dictionary building and output. At level four almost
every residual row is already its own run. The grouped first pass saw 20,584,810 suffix contexts,
296,943 of which branched. These are counts of different operations.

On the frozen block, RLE1 changes 18 of 2,716 distinct phrase payloads, affecting 33 of 135,716
occurrences. Across the large blocks it changes 4,681 of 763,665 block-local dictionary entries
and 11,757 of 31,566,556 occurrences. Those occurrences cover 548,794 pre-RLE bytes. This counts
unequal payloads directly, including equal-length replacements that change byte values.

The four-level kernel and grouped constructor were checked against mibz2’s plain inverse on the
qualifying blocks. The exhaustive native check covers 100,840 arbitrary column, primary and marker
combinations, including short cycles, off-cycle markers and partial final repetitions. In 42,382
cases both fused writers ran, matching the scalar RLE1 result or its error, including 568
unfinished runs. A separate Python implementation compared the quotient with direct rotation
sorting in 9,075 small text and marker cases, including periodic texts, and reconstructed the
original column by stable routing. Forty damaged or truncated variants of the frozen stream
produced the same errors as mibz2, including error location. Additional controls cover reference
prose, random bytes, no marker, a marker as count, four markers together and synthetic phrase
dictionaries.

## Throughput, memory and remaining cost

On a Xeon Platinum 8573C VM, the table gives whole-decoder medians across seven rotated, paired
rounds on one pinned CPU. Every timed call decodes the entire 209.7 MB of Wikidata output. Heap
figures are medians from separate measurements of all 233 large blocks rewrapped singly, each call
starting with a fresh decoder. They count peak live requested Rust allocation above entry,
including the returned buffer. MB is decimal.

| Decoder | One CPU seconds | Output MB/s | Fresh block heap peak MB |
| --- | ---: | ---: | ---: |
| Cells | 0.823 | 255 | 9.59 |
| Lazy Cells | 0.898 | 234 | 10.01 |
| Grouped phrases and fused RLE1 | 1.736 | 121 | 6.13 |
| Representative phrases and fused RLE1 | 2.074 | 101 | 4.84 |

The two fused rows use two token levels. The grouped version constructs decoded second-level
payloads and copies those directly. The representative version expands the token IDs and writes
decoded first-level payloads. The four-level count above is a separate structural experiment. A
warm worker’s LLC footprint needs a separate measurement.

In a second experiment the same scheduler assigned the twelve original streams to 1, 2 or 4
workers. Five rotated rounds at four workers gave 0.317 s for Cells, 0.384 s for lazy Cells and
0.633 s for grouped phrases. The VM exposed five virtual CPUs, a four-CPU quota and no sibling
pairing. Rust 1.90.0 was available for this experiment. The repository pins 1.99.1 for release
measurements.

Cachegrind 3.24.0 on the frozen block counted 32.17 million instructions for the grouped decoder
and 18.86 million for Cells. With a simulated 20 MiB last-level cache and 64-byte lines, grouped
had 13,600 data read misses and 51,959 write misses. Cells had 2,517 and 62,372. Lazy Cells had
2,516 and 51,446. The run includes process startup. The grouped route eliminated the pre-RLE
output buffer on admitted blocks yet performed more simulated reads that missed the last cache.
Cachegrind models line misses. DRAM traffic on the i9 was measured separately.

On the repository’s prose stream the grouped path took about 1.8 times Cells’ elapsed time by
paired medians. Tests with random sequences over fixed dictionaries of 64 words and word lengths
16, 128 and 1,024 also favored Cells. The timings and generator are in the companion directory.

A writer-only experiment supplied the same linear phrase parse to each arm and timed dictionary
finishing, copying and CRC. Across 30 large blocks, decoding all dictionary entries took 1.099
times the materializing writer’s time. Decoding only entries with a potential four-byte run took
1.092 times its time. Each ratio is the median of the 30 ratios of per-block medians from 15
rotated rounds. This isolates the finishing cost even if another constructor could supply the
parse for free.
[Writer timings](2026-11-18-phrase-bwt-rle1/evidence/writer.csv)

The i9 measurements belong to a different machine and experiment. On the 10900K the seven-worker
plan moved 11.0 DRAM bytes per output byte, 7.0 read and 4.0 written, at 24.8 GB/s against a
measured streaming ceiling of 33.1 GB/s. At eight workers the traffic rose to 12.4 bytes per
output byte and at ten pinned workers to 13.9. The higher requested thread counts in release 0.5.4
still selected seven workers. The new experiments ran on a Xeon VM with no exposed sibling threads
or memory-controller events. The effect of this decoder on i9 DRAM traffic and SMT throughput
remains unmeasured.
[Raising the knee](https://github.com/lmmx/mibz2/blob/f396ccc7b3d4290468b12e4ccdfcfcfe7708f3b2/docs/site/blog/2026-11-17-raising-the-knee.md)

## What a shared dictionary would buy

A spelling learned in the first six Wikimedia windows occurred at 96.52% of token positions in the
remaining six, accounting for 79.80% of their pre-RLE bytes. Yet those learned words covered only
12.03% of the bytes in the held-out block-local dictionaries. Including all known suffixes raised
coverage to 17.82% of the held-out suffix nodes. These are 115 held-out large blocks. Training
used only the earlier six windows. The descriptive census used the plain inverse to establish the
actual spellings and supplied nothing to the timed decoder.

This is the distinction that matters for a shared dictionary. It could serve most emitted
references while leaving most of the work of discovering distinct literal spellings. A candidate
for further investigation is an escape for rare spellings in the compressed constructor. The
relevant success measure is the cost of discovering and placing those escapes plus the cost of the
common-word route. Token hit rate alone estimates neither. The present whole-decoder results favor
Cells on the tested machine. A cheap falsification for an escape representation is to count the
original run fragments and bytes needed to identify and emit the misses in the six held-out
windows. A subsequent decoder needs to include that discovery cost, verify every block CRC, and
beat Cells on paired whole-stream runs before its SMT scaling would answer a useful performance
question.

A follow-up takes the separator away and reconstructs the marked output from unweighted two-sided
extensions. It also measures the first counted substring length that fixes the frozen necklace.
[What the BWT needs to know at a branch](2026-11-18-bispecial-factors.md)
