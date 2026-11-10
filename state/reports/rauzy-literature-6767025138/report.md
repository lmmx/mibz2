# Literature dossier: Rauzy–Veech inversion beyond the current +27.6%

Researcher · 2026-11-10 · assignment [6767025138](https://github.com/lmmx/mibz2/pull/162#issuecomment-6767025138).
Instruction pin: a2da49638ac54d7763de1c167bd054f67ffdc763.
Witness pin: [149307d6c20868a18edf9d835c30ca213a54ec6e](https://github.com/lmmx/mibz2/commit/149307d6c20868a18edf9d835c30ca213a54ec6e), [journal](https://github.com/lmmx/mibz2/blob/149307d6c20868a18edf9d835c30ca213a54ec6e/docs/journal/monodromy/2026-11-10-rauzy-veech.md), [example](https://github.com/lmmx/mibz2/blob/149307d6c20868a18edf9d835c30ca213a54ec6e/examples/rauzy.rs).

## Result

The failed linked-order construction does not exhaust the line. The strongest relevant material separates three possibilities: contract interval paths while preserving their words; replace induction by a compressed representation conversion; or avoid random LF accesses through copies. None of the sources below establishes a practical sub-1.228 ns/row induction for these blocks. MONODROMY chooses the direction; this is material, not a new registration or implementation instruction.

**A citation correction matters now.** The witness §10 groups Erickson–Nayyeri Theorem 4.8 under an O(k log N) SLP claim. That theorem actually gives **O(k² log N)** time and grammar size for reduced, possibly disconnected normal curves. The paper separately cites an O(k log N) connected-curve result by Štefankovič, with a warning about extending it to disconnected curves. Agol–Hass–Thurston's orbit counter does not itself supply the claimed word-producing SLP theorem. Preserve those distinctions.

The nearest sources GNP, Nishimoto–Tabei, AHT and Precopy already occur in the witness. Their presence here is a checked applicability analysis, not a novelty claim. Brown–Gagie–Rossi is a move-table engineering source, not an RLBWT-to-SLP conversion paper. Kempa–Kociumaka's conversion runs in the opposite direction to the requested decompressor.

## Budget re-derived from the witness

These are calculations from reported measurements, not fresh timings or independent corpus replay. With order 0.525, induce 1.826, write 0.419 and baseline 2.172 ns/row:

- Required saving: 2.770 − 2.172 = **0.598 ns/row**.
- Holding other stages fixed: induction ≤ **1.228**, a **32.75% reduction**, or **1.487× speedup**.
- Even deleting write entirely leaves **2.351**, above baseline. Grammar balancing as an expansion-only improvement cannot close this gap.
- Deleting order entirely leaves **2.245**, still 0.073 above baseline.
- S ≥ r−1 bounds this elimination mechanism's step count; it is not a lower bound on all inversion algorithms, nor on elapsed time per step.
- The scan 0.484 and upstream run-emitter figure 0.173 come from different paths. Do not silently subtract both, or treat a changed input interface as free.

The 93 median segment rounds are measured dependency structure, not a theorem of work-efficient parallel execution. A round that rescans all live letters can consume the benefit. The already-failed exact-chain prepass (ratio 1.668) rules out advertising “bulk join first” as an untried discovery.

## Checked primary literature

Notation: n text length; r BWT runs; z LZ77 phrases; g SLP size; w machine-word bits. In topology papers k denotes triangulation/pairing complexity and N or X the expanded crossing range; identifying k with r requires a reduction.

### 1. Gagie–Navarro–Prezza: run-boundary samples and the toehold

Travis Gagie, Gonzalo Navarro, Nicola Prezza, **Fully Functional Suffix Trees and Optimal Text Searching in BWT-Runs Bounded Space**, JACM 67(1), article 2 (2020), DOI [10.1145/3375890](https://doi.org/10.1145/3375890). [Author manuscript, §§3, 5](https://arxiv.org/html/1809.02792v2).

A toehold retains one known suffix-array value during backward search. Their φ maps a text position SA[p] to SA[p−1]; it is not an LF successor. Run-boundary samples support neighbouring SA values in O(log log_w(n/r)) time and O(r) words. The text-access structure uses O(r log(n/r)) words and extracts ℓ symbols in O(log(n/r)+ℓ log σ/w) time. Hidden constants are not a kernel budget.

**Map:** order: run starts and image translations; induce: sampling/phrase representations suggest an alternative construction; write: packed extraction avoids requiring one random LF access per symbol once the structure exists. Construction and samples must be charged. A differential-LCP grammar is not automatically a text grammar.

### 2. Nishimoto–Tabei: stateful constant-time moves

Takaaki Nishimoto, Yasuo Tabei, **Optimal-Time Queries on BWT-Runs Compressed Indexes**, ICALP 2021, LIPIcs 198, 101:1–101:15, DOI [10.4230/LIPIcs.ICALP.2021.101](https://doi.org/10.4230/LIPIcs.ICALP.2021.101). [Primary PDF](https://drops.dagstuhl.de/storage/00lipics/lipics-vol198-icalp2021/LIPIcs.ICALP.2021.101/LIPIcs.ICALP.2021.101.pdf).

Their move structures support LF and φ⁻¹ in O(1) time with O(r) words. A query carries its containing-interval identifier; this is not an arbitrary predecessor query for free. Within a BWT run, LF(l+δ)=LF(l)+δ.

**Map:** order: a compact piecewise-translation table; induce: an alternative to linked interval maintenance, but not an acceleration of the same induction; write: following this table conventionally still takes n dependent moves. Smaller state may improve locality without reducing logical LF steps. The theorem gives no nanosecond constant.

### 3. Brown–Gagie–Rossi: explicit space/search tradeoff

Nathaniel K. Brown, Travis Gagie, Massimiliano Rossi, **RLBWT Tricks**, SEA 2022, LIPIcs 233, 16:1–16:16, DOI [10.4230/LIPIcs.SEA.2022.16](https://doi.org/10.4230/LIPIcs.SEA.2022.16). [Primary PDF, Theorem 1](https://drops.dagstuhl.de/storage/00lipics/lipics-vol233-sea2022/LIPIcs.SEA.2022.16/LIPIcs.SEA.2022.16.pdf).

For integer d≥2, their refinement has at most d/(d−1) times as many intervals, with fewer than 2d partition starts in each image interval. Thus d=2 permits ≤2r rows and <4 starts; d=4 permits ≤4r/3 rows and <8. These are useful explicit structural constants, not measured instruction counts.

**Map:** order/move-table representation, not SLP construction. The tradeoff supplies a bounded local-search competitor to predecessor search or pointer-heavy state. It neither contracts an LF orbit nor proves fewer induction rounds. Calling this an RLBWT→SLP theorem would misattribute its result.

### 4. Kempa–Kociumaka: distinguish the conversion direction

Dominik Kempa, Tomasz Kociumaka, **Resolution of the Burrows–Wheeler Transform Conjecture**, FOCS 2020, 1002–1013, DOI [10.1109/FOCS46700.2020.00097](https://doi.org/10.1109/FOCS46700.2020.00097). [Author PDF](https://www3.cs.stonybrook.edu/~kempa/files/FOCS2020.pdf).

They establish r=O(z log² n), and give an LZ77→RLBWT conversion in O(z polylog n) time. The inequality and the algorithm are not a reverse conversion; neither yields a cheap LZ parse from the run list.

**Map:** induce: structural comparison of representations and a warning against reasoning from compressed size to conversion cost. It does not directly change order or write. Polylogarithmic asymptotics have no stated small machine constant suitable for this budget.

### 5. Kempa: the actual RLBWT→LZ77 theorem

Dominik Kempa, **Optimal Construction of Compressed Indexes for Highly Repetitive Texts**, SODA 2019, 1344–1357, DOI [10.1137/1.9781611975482.82](https://doi.org/10.1137/1.9781611975482.82). [Author manuscript v8, Theorem 7.4](https://arxiv.org/html/1712.04886v8).

Given RLBWT, it computes LZ77 in O(n/log n + r log⁹ n + z log⁹ n) time and O(n/log² n + z + r log⁸ n) working space. This verifies the witness's quoted theorem. It uses SA/ISA support, smaller-value queries and LCP machinery; the n term and high logarithmic powers cannot be dropped.

**Map:** a theoretically established replacement for induce, followed by LZ expansion or grammar conversion. It establishes possibility, not competitive pricing at these block sizes. It supplies no usable ns/row constant.

### 6. Rytter: LZ77→balanced grammar, after obtaining the parse

Wojciech Rytter, **Application of Lempel–Ziv factorization to the approximation of grammar-based compression**, TCS 302(1–3), 211–222 (2003), DOI [10.1016/S0304-3975(02)00777-6](https://doi.org/10.1016/S0304-3975(02)00777-6). [Author institution abstract](https://digitalcommons.njit.edu/fac_pubs/14088/).

The verified abstract states O(z log n) time to transform an LZ77 encoding into a grammar of O(z log n) size, using AVL grammars. No concrete runtime constant is supplied. The author's full PDF was inaccessible; this entry relies on the institutional abstract, not a checked implementation proof.

**Map:** induce: a replacement construction after LZ77; write: a balanced representation. Composing it with item 5 is a legitimate theoretical route, but does not remove item 5's costs or establish that its grammar is smaller than the current one on Wikidata.

### 7. Zorich acceleration: a block is larger than one repeated pair

Anton Zorich, **Finite Gauss measure on the space of interval exchange transformations. Lyapunov exponents**, Annales de l'Institut Fourier 46(2), 325–370 (1996), DOI [10.5802/aif.1517](https://doi.org/10.5802/aif.1517).
For the precise grouping definition, see S. Marmi, P. Moussa, J.-C. Yoccoz, **Affine interval exchange maps with a wandering interval**, arXiv:0805.4737v1, §1.4 ([author PDF](https://www.college-de-france.fr/media/jean-christophe-yoccoz/UPL26932_affine_iem_denjoy.pdf)).

Zorich groups successive Rauzy arrows of the same type, equivalently the same winner. This can encompass changing losers. The finite invariant measure is a dynamical result, not a finite-integer runtime bound. No worst-case bound on this decoder's rounds or numerical speedup follows.

**Map:** induce: encode repeated loser itineraries by concatenation and power rules, rather than only repeated equal pairs. The word order must survive acceleration. Rare pair repetitions do not alone measure the opportunity for the more general block.

### 8. Agol–Hass–Thurston: orbit reduction is not word recovery

Ian Agol, Joel Hass, William Thurston, **The computational complexity of knot genus and spanning area**, Transactions AMS 358(9), 3821–3850 (2006). [Author manuscript, Theorem 12 and §6](https://arxiv.org/abs/math/0205057).

For k interval pairings on an integer range of size N, their orbit-counting analysis bounds algorithm cycles by 5k²(2+log₂ N); each cycle has polynomial cost in k log N. This is not a total O(k log N) algorithm. Periodic merging uses gcd; the weighted extension carries additional information.

**Map:** induce: termination potentials and periodic reduction. An orbit count or commutative weight does not recover an ordered word. A transfer needs noncommutative labels, reversal conventions where applicable, and a proof for the LF exchange's equal-length/component cases. No direct write algorithm or small per-cycle constant follows.

### 9. Erickson–Nayyeri: the word-preserving spiral construction

Jeff Erickson, Amir Nayyeri, **Tracing Compressed Curves in Triangulated Surfaces**, Discrete & Computational Geometry 49(4), 823–863 (2013). [Author PDF, §§3.3, 4.3, Theorem 4.8](https://jeffe.cs.illinois.edu/pubs/pdf/tracing.pdf).

A spiral phase produces words of the form prefix · (street sequence)^d · suffix; converting it to binary productions costs O(m+log d) rules. Theorem 4.8 constructs all component crossing sequences of a reduced normal curve in **O(k² log X)** time and SLP size. The following paragraph cites Štefankovič's **connected** O(k log X) result and states the disconnected extension is not immediate.

**Map:** induce: the nearest explicit word-preserving acceleration template; write: ordinary SLP expansion afterward. Missing bridge: a size-controlled reduction from the discrete LF exchange, including periodic/multiple-cycle cases. This correction narrows an attribution, not the algorithmic opportunity.

### 10. Chor–Goldreich: parallel gcd has a different scale

Benny Chor, Oded Goldreich, **An Improved Parallel Algorithm for Integer GCD**, Algorithmica 5, 1–10 (1990). [Primary institutional record](https://cris.tau.ac.il/en/publications/improved-parallel-algorithm-for-integer-gcd/).

For b-bit integers, the stated CRCW bound is O(b/log b) time using b^(1+ε) processors; the CREW variant takes O(b log log b/log b). Extended gcd is also addressed. b is bit length, not the number of BWT rows or runs.

**Map:** induce: conceptual evidence for parallel arithmetic reduction. It is not work-efficient batching of many machine-word divisions and does not promise the complete ordered quotient/word itinerary required here. No justified decoder-speed estimate follows from importing its PRAM depth.

### 11. Edwards–Vishkin: list ranking can invert BWT in parallel

James A. Edwards, Uzi Vishkin, **Parallel Algorithms for Burrows–Wheeler Compression and Decompression**, University of Maryland technical report (2012), [hdl:1903/13299](http://hdl.handle.net/1903/13299), [institutional record](https://drum.lib.umd.edu/items/7b674b30-00f2-4a11-9509-ace1fc3aa3f7).

For a constant alphabet their decompression has O(n) work and O(log n) depth, using list ranking. This is an expanded-n representation, not an r-space theorem. No useful CPU constant is asserted.

**Map:** order: prefix work; induce/walk: replace a serial orbit with ranked positions; write: scatter symbols to known text positions. For the current segments, first prove the reducible structure is a collection of paths with stable disjoint labels. General dependency DAGs are not lists. Naive all-node pointer doubling has O(n log n) work; citing a work-optimal list-ranking theorem does not make that implementation work-optimal.

### 12. Ganardi–Jeż–Lohrey: balance an existing grammar

Moses Ganardi, Artur Jeż, Markus Lohrey, **Balancing Straight-Line Programs**, [arXiv:1902.03568, Theorem 1.2](https://arxiv.org/abs/1902.03568).

An SLP of size g generating length n can be transformed in O(g) time to an equivalent O(g)-size SLP of depth O(log n). The size factor is constant asymptotically; this is not a stated numerical allocation budget. The linear-time construction theorem is not a parallel-construction theorem.

**Map:** write: shorter expansion dependency depth. It does not rebalance the historical sequence of Rauzy updates that produced the grammar.

**Own derivation:** once a binary balanced grammar and rule lengths exist, expand occurrences into disjoint output ranges. The occurrence tree has O(n) nodes and O(log n) depth, hence ideal work O(n), span O(log n). That excludes balancing and length computation and assumes sufficient scheduling/memory. Task-per-character overhead can overwhelm a real implementation; copied/shared derivations require separate scheduling analysis.

### 13. Díaz-Domínguez: parallel construction from already available text

Diego Díaz-Domínguez, **Efficient Terabyte-Scale Text Compression via Stable Local Consistency and Parallel Grammar Processing**, SEA 2025, LIPIcs 338, 14, DOI [10.4230/LIPIcs.SEA.2025.14](https://doi.org/10.4230/LIPIcs.SEA.2025.14). [Primary full text](https://drops.dagstuhl.de/storage/00lipics/lipics-vol338-sea2025/html/LIPIcs.SEA.2025.14/LIPIcs.SEA.2025.14.html).

BuildGram is stated to run in O(n) time with high probability; MergeGrams in O(Ga+Gb) time with high probability. Stable local consistency makes separately constructed grammars merge consistently. These are RAM/hashing results, not fixed per-rule cycle counts.

**Map:** induce: a model for independent local construction and grammar reconciliation; write: resulting grammar. Its input is text, not RLBWT. Decoding to obtain that input defeats a one-shot replacement unless useful structure can be obtained without first paying the original walk. It supplies no O(log r) span guarantee for Rauzy induction.

### 14. Kärkkäinen–Puglisi and Kärkkäinen–Kempa–Puglisi: random accesses are avoidable

Juha Kärkkäinen, Simon J. Puglisi, **Cache-friendly Burrows–Wheeler inversion**, CCP 2011, 38–42 ([author PDF](https://www.cs.helsinki.fi/u/tpkarkka/publications/ccp2011.pdf)). Repeated output factors can be copied from already recovered text, bypassing LF accesses. Map: write/walk; construction and first occurrences still cost work.

Juha Kärkkäinen, Dominik Kempa, Simon J. Puglisi, **Slashing the Time for BWT Inversion**, DCC 2012, 99–108, DOI [10.1109/DCC.2012.18](https://doi.org/10.1109/DCC.2012.18) ([author PDF](https://www.cs.helsinki.fi/u/tpkarkka/publications/dcc2012.pdf)).
Theorem 9 gives O(n) time and O(n/s+rs+n/B) cache misses, subject to σ+σ_T B+O(B)≤M. Choosing s≈√(n/r) gives O(√(rn)+n/B). The tabulated Precopy implementation uses 10n bytes with four-byte integers. Their multi-start method assumes starting positions stored in the compressed file.

**Map:** alternative order/walk/write pipeline, not r-space induction. It disproves an inherent n-random-access requirement, but does not remove the output or preprocessing passes. Published hardware speedups are not transferable promises, and stored start positions are unavailable for free in the existing format.

## Consequences for the next algorithmic choice

These are deductions and discriminators, not instructions to MONODROMY or authorization to benchmark.

1. **The promising contraction question is discovery cost.** A ready-segment DAG with 93 levels may expose ample width. It does not reveal how to find each ready set without repeated global scans, nor how to perform reseating without conflicting writes. A useful certificate records each segment's predecessor/successor, touched order cells, word concatenation and remaining dependencies; sum touched cells, not just levels. The distinction from the failed prepass is fusion or amortization of that work, not naming a different scheduler.

2. **A broader acceleration needs its own counter.** In the witness, 84 self-reseating segments cover 1,901 steps out of 27,439,723, while the equal-pair repetition count is 46. These are different categories. Under equal cost per step, removing the former is only 0.00693% of steps; that assumption is a sensitivity calculation, not a timing bound. Zorich same-winner blocks can include changing losers. To overturn the present negative acceleration evidence, identify longer *word-preserving* blocks that the existing counters miss; do not infer their abundance from the theorem alone.

3. **Separate grammar construction, grammar depth, and emission.** Balancing can shorten the third stage's span while leaving 1.826 ns/row unchanged. To attack induction it must change the scheduling or representation of updates themselves. Bottom-up materialization of every word is especially dangerous: the witness reports 27.7n bytes. Its small-word arena already avoids most long-rule materialization.

4. **There is no universal one-random-access-per-byte lower bound.** Precopy, grammar expansion and prebuilt text extraction are counterexamples under their respective setup costs. Conversely, O(r)-space constant-time LF is not a sub-n-step inverse transform. Price the actual choice: construction + auxiliary tables + orbit/expansion + output + rotation, with the upstream interface stated.

5. **Conventions are part of the reduction.** Several string results assume a unique sentinel and one text orbit. The current example must handle periodic rotations and arbitrary columns with several LF cycles, using its drop rule. A connected-curve theorem cannot silently dispose of these cases. A power rule must preserve order, length, selected row/rotation and repetition count. A grammar CRC can validate its summarized word without validating a faulty emitter; byte-level emission remains an independent obligation.

6. **Output is still necessary.** Materializing n bytes costs Ω(n) byte writes, or Ω(n log σ/w) packed-word writes. This does not imply Ω(n) random reads. Removing the temporary expansion/rotation copy might lower traffic, but the budget calculation shows write-only improvements cannot alone reach parity for the reported decomposition.

## Evidence scope and handoff

One primary-literature round. Source statements above were checked against full author/publisher texts except the explicitly identified Rytter abstract and Chor–Goldreich institutional abstract. No exhaustive novelty search is claimed. No source benchmark was rerun; no native build, timings, new implementation, safety qualification or production acceptance was performed.

The useful advance is a corrected theorem chain and a phase-specific map: compressed word-preserving contraction remains open; existing linear work/short-span results do not automatically compose into a fast r-sized decoder. The pinned instruction permits only the Researcher state branch and one REVIEW on #145. This dossier therefore goes to #145 for the Coordinator to forward to MONODROMY, using its current routing/queue rules. No comment was posted to inbox #346 or #162.
