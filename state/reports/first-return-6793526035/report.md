# First-return package: one bounded reproduction and assessment

Inbox **6793526035**. Package **af3327d8aee9fffe26d2d6de76818b6ac69cc145**, directory research/chatgpt-first-return-20261111. Authenticated role/spec pin **a2da49638ac54d7763de1c167bd054f67ffdc763**.

**Decision: recommend one untimed real-block falsifier of delimiter induction.** No correctness defect was demonstrated in the supplied transformations. Do not promote the full-array batch rounds as a cheaper constructor from these results. No production B5, speed, adoption, authority or programme-choice approval is given. MONODROMY and Pathfinder retain their respective roles; #365/#367/#366 and peer cursors were untouched.

## Reproduction, not author-verdict adoption

Read all three Python programs before execution: stdlib computation and local result writes; no network/subprocess/production mutation in them. Retrieved MANIFEST and all nine payloads, including bundled GPL-3 fixture, at the exact package commit. Every payload byte length and SHA-256 matches. The original ZIP was not fetched and is not certified.

Ran each supplied program once, phrase first, then batch, then copy; no extra sweep or real-data run. All completed within the 30-minute total computation bound, without timeouts. See reproduction.json for exact commands, Python 3.12.14/Linux environment and comparisons; logs and result JSON are preserved beside this report. The batch log's incidental elapsed seconds are not performance evidence.

| Program | Reproduced correctness checks | Archived comparison |
|---|---|---|
| phrase | 6,168 successful binary-column/origin/delimiter cases; 1,004 no-anchor fallbacks | all census fields and result bytes identical |
| batch | 9,194 cases, four checked rounds each; seven fixtures, twelve checked rounds each | all census fields and result bytes identical |
| copy | 7,696 seed/length cases | all result fields and bytes identical |

No checks were unfinished. Original-result digests and reproduced-result digests coincide, stronger than the requested structural comparison excluding elapsed-time fields (none occurs in these JSON files). No failure was hidden: optional README lookup at the cells-gap journal path returned 404; its HANDOFF was accessible and sufficient for proposed corpus provenance. No retry of that path. No execution failure occurred. Prior source inspection and Python-model involvement in cells/F01 is disclosed; this pass did not implement these supplied prototypes.

Synthetic JSON illustrates the tradeoff rather than settling it: quote gives E=4,587 and 7,887 token rows, 928 return pieces, 425 phrase words and 2,661 dictionary bytes; colon E=5,776 with 2,535 tokens; newline E=31,043 with 281 tokens but 32,768 dictionary bytes. These are differing costs, not interchangeable operations. Twelve batch rounds leave 1,121 JSON rows but pay 25,673 overlaps, versus 5,918 unmarked reference cells; a smaller residual is not a cheaper constructor.

## Independent invariant argument and limits

Let f be the stable LF permutation of the nonempty column. Each initial run maps by translation. For delimiter d, A={x:L[x]=d}. Every cycle intersecting A has a unique previous anchor for each row on that cycle. Thus first-return segments starting at anchors partition those cycles; cycles disjoint from A contribute nothing.

This gives a useful exact bound missing from a mere successful census: sum of return lengths over anchor occurrences is at most n. In this implementation, weighted_rows counts that sum. Each queued piece has positive width, so E=steps <= weighted_rows <= n. At any instant, completed and queued pieces partition anchor source ranks, so their combined piece count <= |A|; final Q<=|A|. A new grammar concatenation occurs at most once per processed piece, hence H<=E. These are bounds on logical records, not a Python heap/RSS bound. Images/starts/anchors/accumulators, queue/completed/merged lists, hash-cons table, grammar integers and strings coexist.

The byte dictionary is also not unbounded by a fantastically long cyclic phrase: each represented first-return phrase occurs in that partition, and total distinct expanded phrase bytes <= weighted_rows. Yet a linear-size intermediate grammar and E near n can still lose the hoped-for compressed construction. The source's completed list, sorting and per-step binary searches cost more than just peak_frontier. The counters do not count all touched boundaries, allocations or peak live bytes.

For each completed piece (s,t,width,w), every anchor source rank in its interval follows the same LF emission word and returns at target rank plus the same offset. Splitting at every crossed run boundary preserves that statement. Coalescing requires simultaneous source adjacency, target adjacency and word-ID equality. Merely identifying equal words does not identify occurrences or their successor dynamics. Keep s/t/width; replacing the return map by a dictionary alone is incorrect in general.

Reversing each LF return word gives a phrase ending in d with no internal d, hence a prefix-free set. Stable LF composition orders images by these reversed strings, preserving source order within an equal string. This supplies the token counting-sort identity without assuming a single cycle. For arbitrary columns, it is an induced LF exchange over a collection of cycles, not necessarily one original token text. The program explicitly checks a.t against cumulative token ranks. Its left-associated, hash-consed append construction gives consistent IDs for identical return strings; arbitrary future grammar rewrites must not assume different tree IDs imply different strings.

Origin matters independently of token order. The initial LF walk finds an anchor on the selected cycle or returns None on returning to p0. Reversing the anchor cycle and rotating by the saved shift recovers the chosen origin; repeating/truncating, rather than assuming the cycle length divides n, matches row_oracle. Tests include arbitrary binary columns, multiple cycles, short cycles and all origins up to length eight. This is finite evidence plus the argument above, not exhaustive validation of all byte alphabets and sizes. The initial origin search is up to n and is outside step_limit; future resource accounting must include it. No-anchor is a mathematical decline here, not a tested production fallback/error transaction.

Batch contraction's 0-to-1 selection prevents adjacent removed vertices, so every surviving edge uses f or f²; excluding the marked destination retains the selected cycle. Coordinate rank compression and concatenated emissions preserve its output. However, monochromatic word IDs may stall and global overlay/sort/refinement costs recur. No constant shrink guarantee follows.

The copy example is a valid **demonstrated counterexample** to “a whole-phrase SCC necessarily prevents expansion”: byte dependencies decrease their indices and terminate at a seed, with 15 doubling copies and 32,767 copied bytes for n=32,768. It proves neither cheap source discovery nor a solution for arbitrary copy SCCs. Cyclic unseeded equations remain underdetermined; bounded initialized subranges and overlap validity must be established separately.

## Seam and untested semantics

The tested input is an already expanded BWT column. from_column scans n bytes; it is not a measured/direct adapter from the decoder's compressed run records. bwt/plaintext fixture generation, n-entry row oracle, dictionary expansion/sort, assertions and output reconstruction are validation costs, not free constructor primitives.

Actual inverse BWT emits the RLE1-coded stream. A count byte equal to d is a legitimate byte delimiter for the invariant but not a JSON boundary. Raw JSON phrase statistics therefore cannot substitute for the real seam. Final bytes N can differ from BWT rows n. None of these programs exercises compressed-stream parsing, Huffman/MTF/RLE2 limits, invalid primary/empty-input API validation, truncation, RLE1 state across pieces, block/combined CRC, partial output/error location, streaming reader/backpressure, allocation failure/reuse, concurrent ownership or Rust pointer safety. Those omissions do not invalidate the mathematical probe; they prohibit production claims.

## Draft next falsifier — not executed or registered by this report

**Question:** on one actual inverse-BWT seam, does fixed quote-delimiter induction remove enough row-level traversal to justify a compiled constructor experiment, once construction and occurrence routing are counted separately from oracle expansion?

**Exact provenance:** use the 11,854,933-byte compressed corpus SHA-256 **82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915**, reconstructed from bench-data **a0692452b67ec3a8833e1fc4adf58a4f09cd9e6a**, described in cells-gap HANDOFF at **a08277e1c36e276cfea7b3116d99ea5fd93f0e84**. Deterministically select the first stream-order block with n>=65,536, regardless of its phrase outcome or cells admission. Use unchanged Segmented at **57f51bbc0ca62ac3ee1cc148a1010842eea6222a** as byte/error baseline and the same revision's Cells only as an explicitly separate comparator. Existing corpus claims are provenance from that HANDOFF, not a fresh download verified in this pass.

Before execution, freeze an adapter commit, its digest, the selected block index/bit offset/n/primary, ordered run records and their digest, and baseline RLE1-coded/output digests. Those export identities are **not yet available**; this draft is not run-ready. No “best” block or delimiter selected after results. Fix d=34. A no-anchor selected cycle is a recorded decline, with prefix-search cost, and ends the probe.

**Separate accounts:** (1) offline oracle phrase/grammar generation and reference expansion, explicitly including expanded-column/oracle storage; (2) actual constructor from run records, charging conversion/coalescing and ordering; (3) occurrence traversal and program expansion; (4) RLE1/CRC/final output. Record E, weighted rows, H (created/live), Q, origin steps, boundary touches/splits, searches/sorts, cumulative and peak live metadata bytes, phrase-length distribution, residual token visits, copy calls/lengths/bytes and duplicated writes. Hash-cons dictionary and retained old objects count. Neither dictionary bytes nor frontier alone is peak metadata.

**Exit bound:** one block, one delimiter, one attempt; 10 minutes total local computation and 256 MiB process-memory ceiling, with bound exit preserved, no retry. These are prospective experiment bounds, not decoder caps. Abort on byte mismatch or invalid dependency; never call an unfinished check PASS. Verify seam bytes and final output/CRC against the frozen baseline, but report separately which stream-error tests remain absent.

**Pre-data decision:** mismatch stops this concrete implementation; resource exit or no-anchor declines this probe. If E plus origin-search steps plus residual token visits is >= n, reject the claim that this concrete schedule reduces traversal count on this block; no timing loss is inferred. If it is < n, and explicit peak candidate metadata (excluding separately charged oracle storage) is <4n bytes, retain it as a candidate for one compiled-cost experiment; otherwise revise fragmentation/routing first. The count sum is a prospective screening heuristic, not equal-cost accounting or an acceptance gate for performance. Even passing it leaves hashes, sorting, copies, RLE1/CRC, refuses and decoder context to price. No historical ns/row or 70% share is promoted to a threshold.

The expected gap was reproducibility and an invariant/seam check. It is closed for the supplied probes; the real RLBWT constructor-cost gap remains wholly open. This single seam falsifier has value because it can reject raw-text optimism cheaply without launching another decoder programme. The peers decide whether to take it.

Migration check matched before receipt; pinned routing audit found no misroutes. Publication is one REVIEW on #145. Pinned verify's Coordinator-only FROM check remains a known tooling limitation; exact body/author/issue and role-aware routing are checked without falsifying the Researcher header.

