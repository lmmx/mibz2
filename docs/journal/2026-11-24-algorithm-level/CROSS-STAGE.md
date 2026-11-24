# The cross-stage track: one representation across the decoder's stages

*Opened 24 November 2026 (journal §§35–37), at Louis's go-ahead ("do whatever you think best").*

**Why.** In the supplied-codes test, Open's word codes were supplied in output order and their acquisition was left uncharged. The emitter halved, but the whole decoder fell only to 0.838 at 1 worker, and to 0.988 at 10, an interval that includes no change (journal §30). *Corrected 24 November (journal §42):* this is evidence about that one intervention. It is not a ceiling on every inverse-BWT improvement. An algorithm that also removes construction, changes word emission, or changes the split of work between scanner and workers is untested. Changing several stages together is one route to a large whole-decoder gain at 10 workers; the 2×2 combined oracle (§42) tests whether that route has potential.

**The question.** The stages each have their own representation: bits → symbols → MTF indices → the column L → LF tables → text. Are those boundaries mathematically necessary? Two framings, both open:
- **Shared object.** One composable object that serves more than one stage. Ordinary loop fusion of Huffman, MTF and construction was tried and failed; a common *representation* was not.
- **Enrichment.** Lift an early stage into a richer algebra, so that a later stage's information rides along at little cost. Here the early stage is the Huffman decode or inverse MTF, and the later information is what LF construction or emission needs. Cf. forward-mode AD, (a, a′).

**Known already, so don't present these as new:**
- **The front end already builds Open's run records and byte counts straight from inverse MTF/RLE2,** without materialising the full column (`src/runs.rs`).
- **Huffman lookups carrying composed run and MTF actions,** tested on 22 November: the scanner went from 74.5 to 107.9 ms and the whole decoder to about 1.185 (the Neon teamwork journal).
- **RLE1 and CRC composed through the word grammar:** the Rauzy–Veech record of 10 November, and carbon-kitchen-sink C4.
- **Gap-coded, sampled Ψ built from MTF,** as Neon's E1 does, is Sadakane's and Grossi–Vitter's compressed suffix array: a memory trade.
- **Inverse MTF as composed chunk permutations,** the (O, P) chunk decomposition and the S(R) condition: external reports 5–7, 30 October (95b9cf9b and cc6c5831).
- **Associative and parallel BWT decompression:** Edwards–Vishkin, in the 29 October expert review.
- **Occurrence counts accumulated alongside MTF:** already in the records.
- **Carbon's batched selector/delta scan and its column-MTF kill:** cycle 38, 300a2997.

**What a result must show.**
- What the rider or shared object is, exactly.
- That it is acquired cheaply, in the same currency as the referee: (Q, D, P, S), with acquisition included.
- That it doesn't merely restate the answer. Text positions themselves do not count.
- Which later stage it makes cheaper, with a whole-decoder figure against master, at 1 and 10 workers.

**Owner:** mathematics with MONODROMY (channel #22), as its second line after the senses of moving together. Measurement goes through NEEDS lines.
