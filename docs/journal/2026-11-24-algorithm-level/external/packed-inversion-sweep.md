# Sublinear (packed word-RAM) BWT inversion: literature sweep

Swept 24 November 2026, for mibz2.

## Direct answer

**No. I found no algorithm that inverts a general BWT in o(n) word operations on the packed word RAM, and none that takes substantially fewer than one dependent step per symbol asymptotically, for any alphabet size.** I also found no paper that poses this as an open problem or proves a word-RAM lower bound for it. The Kempa–Kociumaka sublinear line of work covers construction, SA and ISA indexes, LZ77, and RLBWT size. Inversion does not appear in it, nor in their conditional-hardness hierarchy.

**Best known bounds for the plain BWT:**
- **Integer alphabets:** Θ(n) time, the classical LF walk.
- **Comparison model:** Θ(n log n), with a matching lower bound (Cotumaccio 2026).
- **Repetitive inputs:** the only sub-n refinements are in cache misses, not operations. Kärkkäinen–Kempa–Puglisi's Precopy takes O(√(rn) + n/B) misses. Move structures take O(n) time in O(r) working space.

**No lower bound below Ω(n) is stated.** The only one implicit is the trivial Ω(n/log_σ n) for writing the output.

**The bzip2 regime makes the question moot even if an analogue existed.** The construction result's bound is O(n log σ/√log n). For n = 900,000, √log₂ n ≈ 4.45, while log₂ σ ≈ 4 (σ = 16) to 6.5 (σ = 90). A bound of that shape gives a ratio of roughly 0.9–1.5, which is no asymptotic saving at this block size.

## Evidence

**1. Kempa & Kociumaka, "String synchronizing sets: sublinear-time BWT construction and optimal LCE data structure", STOC 2019, arXiv 1904.04228.** Verified in paper. This is construction only. Inversion appears only as "counting inversions" (array inversions), in the conditional lower bound.
- Thm 6.3: "Given the packed representation of a text T ∈ {0,1}^n and its τ-synchronizing set S of size O(n/τ) for τ = ε log n … the Burrows–Wheeler transform of T can be constructed in O(n/√log n) time and O(n/log n) space."
- Thm 6.5: "Given the packed representation of a text T ∈ [0..σ)^n with log σ ≤ √log n … the Burrows–Wheeler transform of T can be constructed in O(n log σ/√log n) time and O(n/log_σ n) space."

**2. Later Kempa–Kociumaka papers do not cover inversion.** Verified by full-text search of each paper.
- *Breaking the O(n)-barrier…CSA/CST* (SODA 2023, arXiv 2106.12725): builds CSA/CST from the text in O(n min(1, log σ/√log n)) time. No "invert" or "inversion of BWT" occurs in it.
- *Constant-time ISA queries…* (arXiv 2608.19123, Aug 2026): builds SA and ISA structures in the same time. Its conditional-optimality discussion lists "BWT construction, inversion counting, and batched orthogonal range queries", not BWT inversion.
- *On the Hardness Hierarchy for the O(n√log n) Complexity in the Word RAM* (STOC 2025, arXiv 2503.21049): its O(n√log n)-in-words class includes "BWT Construction … Batched Inverse Suffix Array Queries" and RLBWT size. BWT inversion does not appear.

**3. Cotumaccio, "On Inverting the Burrows-Wheeler Transform", OASIcs vol. 132 (Grossi Festschrift), 2026, art. 17.** Verified in paper.
- Thm 1: "In the comparison model, to solve Problem 4 [BWT inversion] we need Ω(n log n) queries c(i,j)'s in the worst case."
- It concludes the problem's complexity is "Θ(n log n) in the comparison model and Θ(n) in the case of integers in a polynomial range".
- It notes that for sorting in the word RAM the complexity "is still open". It says nothing about packed or sublinear inversion.

**4. Kärkkäinen, Kempa & Puglisi, "Slashing the Time for BWT Inversion", DCC 2012.** Verified in paper. This is the multi-symbol step the brief asked about.
- The super-alphabet variant (Mtl-sa) precomputes LL[2i] = L[i], LL[2i+1] = L[LF[i]] and LF2[i] = LF[LF[i]], and decodes 2 symbols per dependent load.
- Thm 2: "Algorithm Mtl-sa runs in O(n) time and causes at most ⌈n/2⌉ + O(σ_T + n/B) cache misses provided that max(σ, σ_T B) + O(B) ≤ M." The ComputeLF2 pass needs pair counts CC of size σ².
- Separately, they interleave p = 8 independent sub-walks from stored start positions. This "reduce[s] the cost of cache misses" through out-of-order execution. Together the two techniques give "2.3–4 times faster than Seward's algorithm".
- Precopy, Corollary 10: "Precopy with s = √(n/r) causes O(√(rn) + n/B) cache misses". Its gain is asymptotic only when r is sublinear, which does not hold for bzip2 blocks in general.
- All variants are still O(n) operations. I found no academic follow-up that generalises to k > 2 symbols per step with an analysis.

**5. libsais (I. Grebnov), README and CHANGES on GitHub.** Engineering, verified in the source docs, not peer-reviewed.
- The inverse BWT uses "a bi-gram LF-mapping technique, which allows for the decoding of two symbols simultaneously".
- 2.12.0 (18 Oct 2026) added "cyclic … inverse BWT construction (e.g. as used by bzip2)" and "significantly improved performance of inverse BWT, both single- and multi-threaded".

**6. Move structures: Brown & Langmead, "Bounding the Average Move Structure Query…", arXiv 2602.11029 (v4, Apr 2026).** Verified in paper.
- Thm 9: "Given RLBWT[0..r−1] corresponding to BWT[0..n−1], the original text T[0..n−1] can be inverted in optimal O(n)-time and O(r) working space."
- "Optimal" here means per output character, not packed words.

**7. Kempa, "Optimal Construction of Compressed Indexes for Highly Repetitive Texts", SODA 2019, arXiv 1712.04886.** Verified in paper.
- This is construction in O(n/log_σ n + r polylog n), not inversion.
- It does use "LF-shortcuts" over SA blocks, where LF^k of a block is computed in O(τ₂ log n). That is the nearest packed or skip-ahead LF machinery I found, and it depends on r.

## Brief checks

**(1) Offline vs online separation, the analogue of Komlós and Pettie.** None found for BWT, LF or inversion. The nearest result is batching that beats per-query rank lower bounds. Munro, Navarro & Nekrich, arXiv 1607.04346, verified in paper: "the lower bounds for rank queries can be circumvented in the batched scenario: we can answer the batch of queries in O(n/Δ) time, i.e., in constant time per query."
- This still gives O(1) per query, so inversion stays Θ(n) operations. LF steps also cannot be batched across one chain, because each query depends on the previous answer.
- Sinha & Weinstein, arXiv 1808.03978, abstract verified in paper, prove an Ω(n/t²) cell-probe lower bound for *local* decoding of BWT with "symmetric" data structures, and note that decoding "requires Ω(n) time even to retrieve a single character" without an index. This concerns random access, not full inversion.

**(2) Decoding k symbols per step.** KKP 2012 (k = 2) and libsais (bigram) are the documented instances. I found no theory for general k, or for LF^k tables, on a plain BWT. The LF-shortcuts in item 7 and Precopy's s-runs (item 4) are the only skip-ahead mechanisms, and both pay off only on repetitive input.

**(3) Results from 2023–2026.**
- **In the papers:** Cotumaccio (2026, comparison lower bound), Brown–Langmead (2026, O(r)-space move-structure inversion), and the KK hardness and ISA papers (2025–26), none of which covers inversion.
- **In engineering:** libsais 2.12 (cyclic, multi-threaded unbwt).
- **Not found:** no 2023–2026 paper on SIMD, GPU or parallel inversion of a plain BWT, and no work-efficient parallel inversion tailored to BWT.

## What I could not find or check

- I found no sublinear or packed BWT inversion algorithm, no stated open problem for it, and no word-RAM lower bound beyond the trivial one.
- I could not retrieve Seward's DCC 2001 paper, "Space-time tradeoffs in the inverse B-W transform", or the UC Davis GPU reverse-BWT report, so I make no claims about them.
- **Inference, not a citation:** for a single 900 kB block with σ ≈ 16–90, the asymptotic literature offers nothing beyond constant factors. The practical levers on record are fewer dependent loads per symbol (bigram LF2) and more independent walks to overlap misses, which needs stored start points or multiple blocks.
