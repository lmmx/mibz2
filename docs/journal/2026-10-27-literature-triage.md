# The 42-paper research inventory, triaged for mibz2

The triage of the bibliography in
[2026-10-27-bwt-inversion-literature](2026-10-27-bwt-inversion-literature.md)
(`works.csv`: 42 works), and the research brief for the phase after issues #6–#10. Kept
as written, for reference.

The useful distinction isn't simply read versus unread. It's whether a work can plausibly
help decode existing bzip2 streams faster, rather than improve compression, build a
different index, or introduce a new file format.

Every entry in works.csv is assigned to one of four groups:

| Group | Works |
|---|--:|
| Already represented in mibz2's research agenda | 3 |
| Priority reading after the current issues | 14 |
| Background or targeted reference reading | 6 |
| Low direct applicability / false positive | 19 |

These are provisional relevance classifications, not claims that all 42 papers have been
read in full. The literature entry records that most were captured through Google Scholar
listings, and its 47 citation edges are largely unverified.

## 1. Already represented in the existing research agenda

| Work | What mibz2 already covers |
|---|---|
| Kärkkäinen, Kempa & Puglisi (2012) — Slashing the Time for BWT Inversion | Independent traversal lanes (#4/#5), super-alphabet LF² (#6), Precopy (#9). The central algorithmic source. |
| Kärkkäinen & Puglisi (2011) — Cache Friendly Burrows-Wheeler Inversion | Detecting repeated substrings and copying previously reconstructed output instead of following LF pointers. Directly relevant to #9. |
| Burrows — Block-Sorting Compression | Foundational description of BWT. Relevant to correctness and terminology, not a fresh optimisation hypothesis. |

The first two papers and their techniques have been confirmed independently from their
published texts.

Covered by an issue does not mean experimentally exhausted. In particular, LF² and
Precopy still deserve their proper experiments before moving on.

## 2. Priority reading after the backlog

The order reflects the likelihood of finding something useful for a conventional bzip2
decoder, not academic citation impact.

| Priority | Work | What to investigate |
|--:|---|---|
| 1 | Gagie, Navarro & Prezza (2017) — Fast Locating with the RLBWT | Whether run boundaries can expose predictable LF transitions or cheap shortcuts |
| 2 | Gagie (2026) — Reportable Shortcuts with RLBWTs | Whether shortcut structures have an analogue useful during ordinary decompression |
| 3 | Baier (2018) — On Undetected Redundancy in the Burrows-Wheeler Transform | Whether redundancy can be recognised cheaply enough to predict profitable copying |
| 4 | Baier & Dede (2019) — BWT Tunnel Planning Is Hard but Manageable | The cost and structure of detecting repeated LF paths |
| 5 | Petri (2024) — Scalable Succinct Indexing for Large Text Collections | Cache/layout trade-offs in compact data structures, subject to checking the actual text |
| 6 | Alanko (2020) — Space-Efficient Algorithms for Strings and Prefix-Sortable Graphs | Potential engineering details and related references for practical LF traversal |
| 7 | Navarro (2021) — Indexing Highly Repetitive String Collections, Part II | A survey route to algorithms that exploit repeated LF structure |
| 8 | Navarro (2020) — Indexing Highly Repetitive String Collections | Check whether this is separate from, or overlaps, the 2021 survey |
| 9 | Kärkkäinen, Kempa & Puglisi (2016) — Lazy Lempel-Ziv Factorization Algorithms | Whether any lazy parsing or reuse idea transfers to inversion |
| 10 | Kempa (2017) — Optimal Construction of Compressed Indexes for Highly Repetitive Texts | Potential construction/layout techniques; relevance uncertain |
| 11 | Kempa (2017) — Closing in on Time and Space Optimal Construction of Compressed Indexes | Compare with the preceding Kempa entry and look for practical LF techniques |
| 12 | Bannai et al. (2019) — Indexing the Bijective BWT | Whether traversal techniques transfer from a different BWT variant |
| 13 | Alanko et al. (2019) — Tunneling on Wheeler Graphs | Generalisations of tunnel structure; likely indirect |
| 14 | Bonizzoni et al. (2026) — Patching Leaky Tunnels in BWTs | Recent developments in identifying or repairing tunnel structure; speculative for decoding |

The top four form the most coherent cluster. They take different approaches to
exploiting structure in the BWT, rather than merely making random memory accesses faster.

One hard filter applies when reading them: can the idea be exploited without first
constructing a costly index or changing the compressed format? Baier's original
tunnelling paper, for example, aims to make the transformed representation smaller. That
is interesting mathematics, but not automatically a faster decoder for existing .bz2
files.

## 3. Background and targeted reference reading

These six are kept, but read cover-to-cover only if a specific experiment points toward
them.

| Work | Assessment |
|---|---|
| Daykin et al. (2018) — A Survey of String Orderings and Their Application to the Burrows–Wheeler Transform | General survey; useful for understanding transform variants and finding references |
| Kärkkäinen, Mikkola & Kempa (2012) — Grammar Precompression Speeds Up Burrows–Wheeler Compression | Changes the compression pipeline; unlikely to accelerate existing bzip2 streams directly |
| Ferragina & Manzini (2014) — Burrows-Wheeler Transform | Foundational encyclopedia treatment; useful for terminology |
| Prezza (2021) — Subpath Queries on Compressed Graphs: A Survey | A route into graph-indexing techniques if those become relevant |
| Langmead (2025) — We Are What We Index; a Primer for the Wheeler Graph Era | Contextual overview; likely more useful for indexes than decoders |
| Deng et al. (2022) — FM-indexing Grammars Induced by Suffix Sorting for Long Patterns | Grammar/indexing connection; low immediate transferability |

## 4. Low direct applicability

These 19 entries stay in the bibliography without joining the initial reading queue.

| Work | Why deprioritised |
|---|---|
| Daykin & Watson (2017) — Indeterminate String Factorizations and Degenerate Text Transformations | Different transform/string problem |
| Daykin et al. (2016) — Binary Block Order Rouen Transform | Different transform |
| Sadakane & Navarro (2017) — Computation over Compressed Structured Data | Broad meeting report |
| Fan et al. (2023) — Spectrum Preserving Tilings Enable Sparse and Modular Reference Indexing | Genomic reference indexing |
| Brown et al. (2026) — Col-bwt: Pangenomic Seed Chaining… | Genomic classification |
| Brown et al. (2025) — Improved Pangenomic Classification Accuracy with Chain Statistics | Genomic classification |
| Cotumaccio et al. (2025) — Wheeler Graphs and Wheeler Languages | Graph theory rather than decoding |
| Goga & Baláž (2022) — Prefix-free Parsing for Building Large Tunnelled Wheeler Graphs | Index construction |
| Samofalov & Polak (2026) — Ternary Data Categorization… | Image-compression transforms |
| Alanko et al. (2025) — Graph Indexing Beyond Wheeler Graphs | Different indexing structures |
| Baier et al. (2022) — Edge Minimization in de Bruijn Graphs | Graph optimisation |
| Mustafa (2022) — Algorithms for Efficient Sensitive Search… | Genomic search |
| Gagie et al. (2018) — Relaxing Wheeler Graphs for Indexing Reads | Indexing rather than inversion |
| Jeż (2015) — Faster Fully Compressed Pattern Matching by Recompression | Pattern matching |
| Jeż (2015) — Approximation of Grammar-Based Compression via Recompression | Compression construction |
| Jeż (2020) — Recompression: Technique for Word Equations and Compressed Data | General technique/talk |
| Cankaya & Darwish (2013) — Improving Compression Performance with a Star Encoding Front End | Changes encoding, not decoding existing bzip2 |
| Kempa & Kociumaka (2023) — Collapsing the Hierarchy of Compressed Data Structures… | Succinct index construction |
| J. E. Baier (2021) — Charakterisierung der inhibitorischen Wirkung von Ambroxol… | Confirmed irrelevant Scholar false positive |

This does not say those papers contain nothing interesting. It says their titles and
recorded descriptions give insufficient reason to spend mibz2's research budget on them
yet.

That accounts for all 42 entries.

## 5. Candidate directions from the literature

These are hypotheses, not verified findings from full-paper readings.

- **A. Run-aware LF traversal.** Consecutive rows in a BWT run of the same character have
  consecutive LF targets. Could those regularities be encoded or exploited to do less
  random access? Sources: Gagie–Navarro–Prezza, the RLBWT literature.
- **B. Repetition-aware copying.** Detect whether the output being reconstructed matches
  an earlier substring, and substitute sequential copying for repeated LF lookups.
  Sources: the 2011/2012 inversion papers, Baier's redundancy work. Already substantially
  within #9.
- **C. Memory-efficient lookup structures.** Whether a different representation of LF or
  the T-vector offers a worthwhile time–memory trade-off. Sources: the succinct-index
  literature. Relevant because mibz2's RSS exceeds pbz2's, but compact representations
  often make random access slower.
- **D. Cheap predictors of the best inversion algorithm.** Could BWT run counts,
  repetition estimates, block size or observed traversal behaviour predict whether plain
  walking, segmented walking, LF² or Precopy will win? Sources: structural BWT research.
  Extends #9's adaptive-selection question.

A is the most interesting potentially fresh algorithmic direction. B and D overlap
existing #9 work substantially, and C deserves attention only with careful memory and
throughput measurements.

The caveat to A: adjacent LF entries may be predictable, but inverse-BWT traversal does
not necessarily visit adjacent rows consecutively. That is the gap between a useful
mathematical identity and an actual fast decoder, and whether it yields enough practical
shortcuts to matter is the question.

## 6. Research sequence

1. Finish the current experiments, especially LF² (#6) and adaptive inversion/Precopy (#9).
2. Read the top-ranked papers in full, taking short technical notes on each one's
   algorithm, data structures, memory cost, construction overhead and applicability to
   existing bzip2 blocks.
3. Compare every candidate with what has already been tried, and remove duplicates.
4. Turn only the genuinely new, testable ideas into the next issue queue.

Most of these 42 works were found by following citation chains into compressed indexing,
which explains why so many are tangential to decoder performance. It does not mean the
relevant literature is exhausted. The next discovery pass broadens the search beyond
this citation graph, into practical decompressor engineering, cache-miss avoidance, SIMD
entropy decoding and memory-level-parallelism research.

The outcome of this first triage: 42 works accounted for, 14 retained for prioritised
reading, four provisional technical themes, and no new issues yet.

## Research brief: BWT inversion beyond the current mibz2 backlog

**Objective:** read the most relevant papers from the 42-work bibliography and extract
new, testable optimisation hypotheses for decoding ordinary .bz2 streams. Prioritise
ideas that reduce dependent memory accesses, improve locality, exploit repetition or
reduce memory consumption.

**Timing:** begin after issues #6–#10 have been resolved experimentally. Do not duplicate
completed work.

### Priority research queries

- [ ] 1. Gagie, Navarro & Prezza (2017) — Fast Locating with the RLBWT. Can consecutive
  LF targets within BWT runs be used to batch traversal, predict addresses or avoid
  dependent loads without building a full compressed index?
- [ ] 2. Gagie (2026) — Reportable Shortcuts with RLBWTs. What shortcuts are constructed,
  how expensive are they, and could any be generated cheaply enough during bzip2 block
  decoding to amortise their cost?
- [ ] 3. Baier (2018) — On Undetected Redundancy in the Burrows-Wheeler Transform. Can
  repeated LF paths or tunnel-like structures be recognised in a standard bzip2 block
  without changing its representation? What is the detection overhead?
- [ ] 4. Baier & Dede (2019) — BWT Tunnel Planning Is Hard but Manageable. Are there
  lightweight heuristics for locating repeated traversal structure that could guide
  copying or skipping, without performing expensive tunnel planning?
- [ ] 5. Petri (2024) — Scalable Succinct Indexing for Large Text Collections. Are there
  cache-efficient lookup layouts or compact representations transferable to mibz2's
  T-vector? Quantify memory saved against additional lookup instructions.
- [ ] 6. Alanko (2020) — Space-Efficient Algorithms for Strings and Prefix-Sortable
  Graphs. Does the thesis describe practical LF-access optimisations, cache layouts or
  parallel traversal techniques not covered by the 2012 inversion paper?
- [ ] 7. Navarro (2020/2021) — Indexing Highly Repetitive String Collections, Parts I/II.
  Identify relevant algorithms and references involving LF shortcutting, sequential
  reconstruction, run-aware traversal or direct decompression. Avoid following
  indexing-only branches unnecessarily.
- [ ] 8. Kärkkäinen, Kempa & Puglisi (2016) — Lazy Lempel-Ziv Factorization Algorithms.
  Can its lazy evaluation or repeated-substring reuse techniques transfer to inverse-BWT
  output reconstruction without requiring a separate LZ parse?
- [ ] 9. Kempa (2017) — Optimal Construction… / Closing in on Time and Space Optimal
  Construction…. Check whether these are distinct publications. Look specifically for
  compact LF structures, cache-conscious construction and reusable traversal techniques.
- [ ] 10. Bannai et al. (2019) — Indexing the Bijective BWT. Do any inversion or
  permutation-traversal methods transfer to ordinary BWT blocks despite the different
  transform?

### Cross-paper synthesis

- [ ] Compare run-aware LF traversal, repetition-based copying and shortcut techniques.
  Identify what genuinely differs from #6 (LF²) and #9 (adaptive walking/Precopy).
- [ ] Determine whether any method can be constructed in one pass over an existing bzip2
  block and pays back its setup cost within that same block.
- [ ] Look for practical memory/time trade-offs relevant to mibz2's higher peak RSS
  against pbz2's.
- [ ] Identify overlooked references on cache-aware BWT inversion, pointer chasing,
  software prefetching and SIMD decompression, including work outside the existing
  citation graph.

### Deliverable

A concise research note containing:

1. **Paper-by-paper findings:** the relevant technique, its assumptions, construction cost
   and applicability to existing .bz2 streams.
2. **Novel hypotheses:** ranked by expected benefit, implementation complexity and
   likelihood of success.
3. **Overlap check:** which ideas have already been tested or are covered by existing
   mibz2 issues.
4. **Cheap experiment:** for each genuinely new hypothesis, the smallest measurement that
   could quickly validate or reject it.
5. **References:** the paper sections, algorithms or page numbers supporting each proposed
   technique.

Do not open GitHub issues, implement code or start extensive benchmarks during this
phase. The output is a shortlist for discussion and prioritisation before the next
optimisation campaign.
