# External research report 160: the prompt sent to Astra, part 1: an architectural review of the 30-entry compact navigation design (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02, with reports 159 and 161. All three are exact
  slices of one relayed message.
- **Source:** the maintainer's prompt to Astra, written with the guardian.
- **Sent to:** the existing Astra conversation, at Max reasoning, with no local commands.
- **Part 2:** report 161, which replaced an earlier addition to this prompt.

**Evidence cutoff:** 2 November 2026. The prompt cites the census at `1fbc0a0`. A later
commit, `1441b49`, changes only the census journal's wording on RAW rows, as report 159
asked.

It is kept exactly as received (rendered text, cards flattened).

## The prompt, as relayed

`````text
mibz2 — Architectural review of the new 30-entry compact navigation design

We have substantial new evidence since your original P40/P38/P84 architectural memorandum and FOR-14 proposal.

I want you to conduct a focused, high-effort architectural review of a new compact inverse-BWT navigation layout developed by the independent researcher and validated in a reference implementation by Claude Code.

The objective is to decide whether this design deserves one bounded optimized prototype, not to reopen the entire optimization portfolio.

Please exercise independent architectural judgment. Inspect the actual repository code and experimental records rather than relying solely on this message.

1. Why we are reconsidering compact navigation

Our goal remains a substantial improvement in mibz2’s four-physical-core decoding throughput on the Intel i9-10900K, ultimately aspiring toward 1.5–2× without sacrificing CPU efficiency or reasonable memory use.

The shipped decoder uses:

* A 32-bit inverse-BWT T-vector, approximately 3.6 MB per full 900,000-row block.
* A fused table constructor.
* A segmented 32-lane dependent traversal with software prefetch.
* Segment assembly, RLE1 expansion and CRC validation.

Historical experiments established that smaller navigation tables can change memory behaviour and concurrency scaling, but expensive lookup and byte recovery can erase those benefits.

The first FOR-14/raw candidate reduced the table to approximately 1.87 MB per full block, but its frozen i9 traversal results were poor:

Walker	One-core speed relative to conventional control	Four-core speed
FOR-14 h	0.475×	0.688×

That experiment is complete. Its registered STOP verdict remains unchanged.

Subsequent compile-only work improved the common path:

Arm	Instructions per ordinary step	Loads	Stores
Shipped reference	16	3	2
FOR-14 BMI2 b	37	5	3
Packed-state k	31	4	2
Packed-state plus IMUL j	27	4	2

The k and j variants passed correctness checks, but neither has been timed on the i9.

Their important improvement was packing the successor row and carried byte into one lane token, removing an actual lane-state load/store pair.

However, both still pay for extracting arbitrary 14-bit offsets.

The independent researcher has now proposed a way to eliminate that extraction operation almost entirely.

2. New candidate: thirty 16-bit offsets per cache line

The proposed representation uses one aligned 64-byte record:

* Thirty directly addressable u16 successor offsets, occupying bytes 0–59.
* One four-byte header in bytes 60–63.
* Two reserved offset values for exceptional cases.
* A transformed row-coordinate system that eliminates division by thirty during navigation.

The field values are:

Field	Meaning
0..=65533	Ordinary successor offset
0xFFFE	Marked segment start
0xFFFF	RAW row or parking record

The two exceptional values allow the design to remove FOR-14’s separate start bitmap.

The header stores the encoded successor base and the reconstructed byte for ordinary groups. RAW groups use an external area, as in the existing design.

The coordinate transformation

Original row r becomes:

\[
\phi(r)=32\left\lfloor\frac r{30}\right\rfloor+(r\bmod30).
\]

Two positions in each 32-position group are unused as row labels because they contain the header.

For encoded row x=\phi(r):

\[
\begin{aligned}
\text{offset address}&=\text{records}+2x,\\
\text{header address}&=\text{records}+2(x\mathbin{\&}\sim31)+60.
\end{aligned}
\]

Every successor is stored in encoded coordinates. Navigation therefore does not need to convert back to original row numbers on every step.

The packed lane token retains the previous optimization:

q=x+(\text{carried byte}\ll20).

For an ordinary group, with header H=B+(F[r]\ll20) and offset o,

q_{\mathrm{next}}=H+o.

This simultaneously recovers the encoded successor and the new carried byte without an additional byte-state load/store pair.

The proposed ordinary step is conceptually:

x = token & ROW_MASK
offset = load_u16(records + 2*x)
if offset < 0xFFFE:
    header = load_u32(records + 2*(x & ~31) + 60)
    next_token = header + offset
    emit the old token's carried byte
    token = next_token
    prefetch the next record
else:
    handle segment start, RAW row or parked lane

This is not yet a compiled optimized walker.

The key architectural claim is that an unpredictable 14-bit packed-field extraction becomes a direct halfword load, while the lookup remains within one 64-byte cache line for ordinary records.

It still requires two table loads, rather than the shipped T-vector’s one load. Those two loads should be independently addressable from the current token, but this must be established in realistic generated machine code.

3. The new census is complete

Claude has created a reference implementation and full corpus census.

PR #132: https://github.com/lmmx/mibz2/pull/132

Commit: 1fbc0a0b609ed9584fdafcaf9668308ccbd0358d

Journal: docs/journal/2026-11-02-compact30-census.md

Implementation: examples/census30.rs

Please read those files directly.

Across the established eleven corpora:

* 1,513 blocks were examined.
* Approximately 690 million BWT rows were covered.
* Every specification check passed.
* Both scalar reference round trips were byte-exact on every block, including marked starts on ordinary and RAW rows.

The primary Wikidata result is:

Quantity	Result
Logical bytes per row	2.164
Relative to shipped four-byte table	0.541×
Relative to FOR-14’s 2.076 bytes/row	1.042×
Largest full block	1,953,600 bytes
RAW rows	0.72%
RAW groups in full blocks	189–262
RAW-group causes	62% span too wide; 38% character-bucket crossing

The reference implementation checks:

* Monotonic Ψ successors within each first-column character bucket.
* Encoded coordinate and parking limits.
* No successor pointing into header positions.
* Offset and sentinel separation.
* Correct packed-token arithmetic without byte-field carry.
* The conservative RAW-group bound.
* Byte-exact reconstruction, both normally and with starts marked.

The storage bound

The independent researcher derived:

R\leq\min\left(
G,\;
\sigma\left\lfloor\frac{M}{65534}\right\rfloor+\sigma-1
\right),

where G=\lceil n/30\rceil, M=\phi(n-1), and \sigma is the number of used byte values.

With a 128-byte RAW area per exceptional group:

\text{logical bytes}=64(G+1)+128R.

At 900,000 rows:

* At most 98 used symbols: 2,108,096 bytes under the bound.
* Any 256-symbol alphabet: 2,411,456 bytes under the bound.

Please independently scrutinize the proof, especially the claim that disjoint monotone successor spans bound the number of wide RAW groups.

The corpus census supports a substantially smaller observed representation than either upper bound.

Do not mistake these logical byte counts for measured RSS, LLC residency or performance.

4. New evidence from the hardware-ceiling experiment

We also completed the independent researcher’s minimal-walker diagnostic.

PR #131: https://github.com/lmmx/mibz2/pull/131

The capture was frozen at 29a0215, and its subsequently amended interpretation is recorded on the same research branch.

The capture passed its registered validity gates.

The shipped walker has little demonstrated bookkeeping headroom

At 32 lanes, one core:

* Reduced shipped-style walker M: 6.293 cycles/row.
* Shipped walker P: 6.897 cycles/row.
* Ratio: 1.096×.

The registered D1 stopping rule therefore rejects a generic walker-scheduling rewrite.

M omits required output work, so its performance is not an attainable decoder target.

Concurrency remains expensive even in simplified walks

With four concurrent walker copies, matched execution-counter groups showed:

Kernel	Change in retired instructions/row	Change in cycles/row
A, without software prefetch or decoded-output stores	+0.026%	+38.3%
B, with software prefetch but without decoded-output stores	+0.014%	+36.6%
P, shipped walker	+0.024%	+43.3%

Thus a substantial concurrency penalty appears without the production job queue, decoded-output stores, assembly or even software prefetch being necessary.

It does not prove which shared memory resource is responsible.

The existing walker benefits substantially from prefetch. The collected issued-µop share was about 57% of a four-per-cycle reference, but that does not establish spare execution capacity.

Demand-MLP and fill-buffer observations likewise did not establish an exclusive physical bottleneck.

The rare L3 events had control-window contamination large enough to prevent strong L3-specific claims.

The test repeatedly traversed one warmed block; it was not a full-decoder throughput experiment.

The architectural implication: a smaller representation might reduce concurrent traversal cost, but it must pay for its own additional extraction, state management and construction. Neither its success nor its failure follows from the current counters.

5. Important historical comparison

The earlier compact-Ψ experiment found that a smaller navigation table improved cache behaviour but could still be slower overall.

At four blocks, the 24-bit u representation reportedly:

* Improved the L3 hit share from approximately 0.929 to 0.971.
* Reduced the wait-per-read proxy from about 67 to 52 cycles.
* Reduced DRAM reads substantially.
* Still achieved only about 0.89× the reference throughput.

That is important negative evidence.

But the new 30-entry layout differs materially:

* No separate F-directory lookup on its ordinary path.
* No variable-width packed-field extraction.
* Direct halfword addressing.
* Packed lane-state update.
* A one-cache-line ordinary record.

Do not transfer the old format’s performance costs or cache benefits directly to the new one.

A smaller logical table does not establish a cache-hit rate. A different number of instructions per step does not establish a speedup.

The central question is whether the new layout changes the access economics enough to avoid repeating the earlier failure.

6. Your four technical review obligations

A. Encoding and semantics

Independently inspect the implementation and examine:

* The conjugated mapping \Psi'=\phi\circ\Psi\circ\phi^{-1}.
* Successors remaining in encoded coordinates.
* Reconstructed-byte orientation.
* Initialization from the original last column.
* Segment starts, especially on RAW rows.
* Temporary sentinel insertion and restoration.
* RAW parking, wait/resume behaviour.
* One-byte and short segments.
* Tail handling and partial records.
* Correctness for periodic and multi-cycle structures.
* Whether any original-row conversion is needed in the hot path.

Identify any correctness gap that the scalar reference round trip would not reveal in a 32-lane segmented implementation.

Do not assume that successful scalar checks prove the optimized walker.

B. Storage mathematics and construction

Verify the RAW-count bound and every byte in the record layout.

Check for:

* Hidden metadata requirements.
* Alignment and padding.
* Parking-record overhead.
* Partial groups.
* Exceptional-path storage and indexing.
* Whether encoded labels remain inside twenty bits at every valid block size.
* Dynamic RAW visitation frequency versus storage frequency.

Assess whether a bounded-memory direct constructor could be engineered from the last column without constructing a full temporary T-vector.

Estimate its likely work: scans, counting-scatter operations, staging, encoded-coordinate conversion, random writes and cache traffic.

Do not assume a smaller final table implies faster construction.

C. Execution cost on the i9-10900K

This is the central engineering question.

Determine whether the theoretical ordinary path could plausibly compile into substantially fewer instructions, µops, dependent operations and memory requests than FOR-14 k and j.

Pay particular attention to:

* Direct indexed u16 loads.
* Independent header and field addresses.
* Address-generation cost for encoded coordinates.
* The packed-token update.
* RAW sentinel branching.
* Prefetch address formation.
* Lane-state loads and stores.
* Register pressure at 32 lanes.
* Effects of the actual Rust/LLVM compilation target.

Build a realistic dependency model for the common path, rather than just counting static instructions.

The researcher’s approximately 20-instruction estimate is a hand sketch, not a measured or compiled result.

Compare with the shipped 16-instruction step and the existing k/j 31/27-instruction steps.

Explain what portion of the previous 14-bit extraction penalty is actually eliminated, and what unavoidable costs remain.

D. Investment choice and falsification

Make a decisive recommendation between these options:

Option 1: Measure k and j first.

Explain what their performance would tell us that materially affects the 30-entry investment decision, and why that information is necessary.

Option 2: Build one bounded optimized 30-entry walker prototype.

Specify the smallest implementation that can establish correctness and compiled hot-path cost without requiring a full direct constructor or production integration.

Include the precise conditions under which its VM-only compilation and correctness results would earn a separate performance experiment.

Option 3: Stop or defer the design.

Identify the specific proof, instruction dependency, exception-handling or construction-cost problem that makes further effort unjustified.

Do not recommend a large matrix of layouts, prefetch policies, lane counts or exception strategies.

7. The performance economics

Our retained approximate phase-cost model is:

\text{CPU saving}\approx0.592(1-b)+1.847(1-w),

where b and w are new/old cost ratios for construction and traversal-plus-assembly.

This is a planning approximation, not a measured prediction for the new format.

To recover 0.25 CPU-s/GB:

* Unchanged constructor cost requires about 13.5% less traversal-plus-assembly time.
* A 10% slower constructor requires about 16.7% less traversal time.
* A 20% slower constructor requires about 19.9% less traversal time.

The production goal is larger than 0.25 CPU-s/GB, but this gives a minimum useful economic screen.

Please use this model cautiously and explain where the assumptions break.

The key is full-decoder economics, not an impressive standalone compact-table benchmark.

8. Research standard and required answer

Please produce a concise but technically substantial review with:

1. Executive recommendation: one of the three investment options, clearly chosen.
2. Correctness and proof audit: any defects or unproven optimized-walker obligations.
3. Execution-path cost model: plausible ordinary-path assembly structure, dependencies, loads/stores and likely bottlenecks.
4. Construction feasibility: bounded-memory approach and a realistic construction-cost risk assessment.
5. Cheap falsification plan: the smallest useful implementation or measurement, and clear stopping rules.
6. Questions for Claude: at most three precise implementation or audit questions.
7. Assessment of upside: whether a substantial four-core throughput gain remains credible, clearly distinguishing evidence from speculation.

Use the actual repository evidence, CPU documentation and prior negative trials.

Be willing to reject the design if its economics do not work.

Equally, do not reject it because the previously tested 14-bit or 24-bit representations failed for costs that this format genuinely avoids.

Scope: This is a research-architecture review for a handoff to Claude Code. Please inspect and reason; do not change repository files or implement an experimental walker in this response.

The desired output is one decisive recommendation and the reasoning needed to act on it, not another broad portfolio or another large research journal.

We want to find a decoder architecture capable of meaningfully larger gains. The question is whether this particular design deserves the next bounded investment of engineering effort.

I’d send that as-is. It incorporates the completed census, the important findings from the walker-ladder experiment, and the historical negative evidence, without forcing Astra to choose the new design.

Existing Astra conversation, Max reasoning, no local commands.

Replace the earlier addition with this:
`````
