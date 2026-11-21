# Reviewer's baseline, written before reading the post (from #351's journal at 2dde542d and CORRECTION 6770249707)

## The simplest correct explanation
bzip2's last decoding stage undoes the Burrows–Wheeler transform by following a chain of n
table lookups, one a byte, each to an unpredictable place in a table of several megabytes.
The permutation being followed moves every run of equal bytes of the transformed block as
one rigid piece, so it is fully described by r pieces and their lengths (r is about 7% of n
on Wikidata). The new method never follows the chain: it repeatedly simplifies that
description (compare the last two pieces, subtract the shorter length from the longer, and
record "this piece's text is that one's followed by the other's"), and after about 1.8 r
such steps the whole block is one tree of rules, which is written out by copying.

## Key takeaways
1. WHAT: an exact inversion of the BWT with no walk. On Wikidata's 233 blocks the grammar has
   1.81 r = 0.13 n rules; every block's bytes equal the walk's.
2. HOW FAST, AND OF WHAT: an experimental kernel takes 0.82 of the time of production's
   inversion stage (0.2.8's) on Wikidata: one thread, one container, indicative. Faster on
   192 to 197 of 233 blocks. It is the inversion stage, not the whole decode.
3. WHERE IT LOSES: data without long runs. Prose 4.0x, random bytes 8.9x, JSON of many
   records 1.7x; a periodic worst case 11x. It needs a selector and a step budget.
4. STATUS: not in the decoder. An example, unsafe and unreviewed. Not measured on the i9, on
   many threads, or inside a decoder. A defect in its writer (bytes left unwritten, the check
   blind to it) was found by review and fixed; the 0.756 figure predates the fix and is against
   the older baseline.
5. WHY IT IS FASTER WHERE IT IS: the work is a step a run, not a lookup a byte; the bytes come
   from copies. 184 instructions a run against 260; 2.1 simulated last-level misses against 8.7.
6. WHAT IS NEW AND WHAT IS NOT: the piecewise description and the induction step are known
   mathematics; using the induction to invert a BWT, the counts, and the grammar's identity
   (the Lyndon tree of the least rotation) are this work's. The first construction lost (1.276x).
