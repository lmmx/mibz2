# Round-two baseline from primary records

The inverse BWT is a permutation of rows with letters on its arrows. Cut the arrows leaving a chosen set containing the primary. The paths that remain can be solved on their own, and a stable sort of the kept letters reconstructs the erased pairing in entrance rank. Combine the two maps and follow the marked return backwards to emit the text.

- An entrance has lost its predecessor. Since every row on a cycle already has a predecessor, an entrance cannot join a cycle. Uncut cycles can survive away from the marked orbit. A marked orbit may be shorter than the block; repeat the marked spelling to block length.
- Sorting only the kept bytes preserves their order in the full stable sort. The positions in the shortened list are entrance ranks, not original row IDs. Banana's a$a example gives stable LF [1,0,2], open path exits [0,2,1], composition [2,0,1], and inverse words bana, na, $.
- Many short runs account for fewer byte rows. Keeping them moves boundary-heavy work into counting sort and leaves wide pieces where adjacent paths may stay together. Band count has a proven bound; work also depends on height, and width-one repeated shifts can cost linear work.
- On 233 large Wikidata blocks at threshold 16, median wide area is 88.6689% of all rows and median representative work 5.4943%. Median per-block A/U is 15.9236. The frozen example is 810,057/50,709. The six prose blocks have median wide area about 32.3%; they vary.
- Buying less path discovery does not eliminate the retained table or output writes. The queued endpoint uses 6.287 MB vs 10.353 MB additional requested heap for one frozen decode. Whole-Wikidata paired wall ratios versus eager/lazy are 1.171/0.918 on the calling thread, 1.211/0.931 at two pool workers, and 1.091/0.928 at four. This is one quota-limited Xeon VM with no exposed SMT siblings. The target i9 result remains open.
- The first-return factorisation and band bound are proved. Structural counts, timings and heap are observations. A proposed cause for the whole-decoder time difference, and conclusions drawn from the threshold sweep, require their own scope.

Read before the revised article: #565 journal, proof, executable and raw evidence at `e6df6aee5055840b8b6ed780e29727e23da6d42f`; the two technical reviews (record 57 and REVIEW 6978935386).
