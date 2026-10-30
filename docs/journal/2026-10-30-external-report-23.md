# External research report 23: research burn-down rounds 23–24, P74's critical path and its fallback invariant (verbatim)

A research input, not a finding of this repository. The maintainer relayed it on
2026-10-30, after external report 22 (`2026-10-30-external-report-22.md`).

**What it contains:**
- **Round 23:** P74's critical path, split into rank, selected byte, updated head and next
  update. Three ways P74 could lose. Separate criteria against #25.
- **Round 24:** the fallback's exact state invariant for ranks ≥ 32:
  - H′ = [v, H[0…30]];
  - T′ = [H[31], T without its element t].

  With the obligation H′‖T′ = MTF(H‖T, q) for every q, and so for every sequence.

**The coordinator's checks** (`2026-10-30-research-portfolio.md` §3.2d). The partitioned
state, head register plus tail, matches plain MTF with 0 mismatches on:
- a Wikidata-like rank mix, uniform 0–255, all-high (≥ 32) streams, and repeated
  crossings of the 30–33 boundary (60 runs of 4,000 tokens each);
- two real bench blocks' explicit-rank streams with bzip2's own symbol mapping (89 and
  86 used symbols; 51,184 and 57,506 tokens; ranks ≥ 32 at 1.51% and 0.94%).

This is source material. Its claims are checked there, not taken as findings.

The text reached us as rendered text (some formula symbols lost) and is kept as received.

## Round 23, as received

````text
That’s a useful result for P74. The correctness question is now substantially resolved: a register-held 32-entry MTF update can reproduce the scalar transition across every rank from 0 to 31.

The remaining question is whether its dependency chain is shorter than the shipped AVX2 load–blend–store sequence.

One distinction I’d preserve in the report: correctness across all 32 ranks is not yet correctness for the complete MTF decoder. The high-rank fallback must preserve the register-held head, the tail and their ordering across arbitrary transitions.

Round 23 — P74’s critical path

I would separate the instruction chain into four parts:

\text{rank }q
\rightarrow
\text{selected byte }v
\rightarrow
\text{updated head }V’
\rightarrow
\text{next rank update}.

The important distinction is that the selected byte and the shifted head can potentially be calculated in parallel.

Proposed register-held dependency graph

The diagram describes dependencies, not measured instruction latency.

The critical path is approximately

\boxed{
L_{\mathrm{iteration}}
\ge
\max(L_{\mathrm{extract}},L_{\mathrm{shift+mask}})
+
L_{\mathrm{insert}}.
}

Actual latency also includes dependency-generating instructions, register selection, and possibly cross-lane permutations.

The literature agent should confirm instruction latencies for the actual proposed instruction sequence on the target microarchitecture, not isolated nominal latencies added indiscriminately.

There are three ways P74 can lose despite correct SIMD arithmetic:

* Cross-128-bit-lane operations lengthen the update chain.
* Variable rank extraction requires multiple serialized shuffles.
* High-rank fallback requires spilling and reconstructing register state frequently enough to erase the common-path gain.

Conversely, its strongest advantage is potentially removing the shipped implementation’s load-after-overlapping-store dependency from successive explicit tokens.

The decisive comparison with #25

I’d preserve these separate criteria:

Criterion	Current finding
Register-held update for ranks 0–31	Correctness check passed
Distinct from #25’s 16-entry register design	Yes, in coverage and proposed transition structure
Arbitrary sequences including ranks ≥32	Not yet reported as verified
Shorter critical path than shipped AVX2	Awaiting instruction-latency verification
Lower full-decoder CPU cost	Unmeasured
No random-data regression	Unmeasured

If the critical-path calculation is unfavorable, we can stop P74 before implementation. If favorable, P74 remains conditionally viable, not proven faster.

The broader burn-down remains on course. N2 and N2b have failed their registered structural tests, #74’s lane-coincidence question is answered, and P74 is approaching a similarly precise decision point.

I would not reopen N2/N2b or start an implementation campaign on the strength of this result. Finish P74’s latency and fallback analysis, then record the disposition against the existing inventory.
````

## Round 24, as received

````text
New burn-down round. Don’t wait.

The 8,160 successful doubling-level checks close the remaining correctness concern for layout C’s bitvector counting. Both count directions match the merge construction, and post-splitting handles segment caps. That’s a substantial result, but no reason to stop researching while Claude records it.

I’d now take round 24: the P74 high-rank fallback and its exact state invariant.

We already know that the register-held update works for ranks 0–31. The next step is to prove correctness across arbitrary sequences, including transitions from the register path to the high-rank path and back.

Let the full 256-entry MTF list be partitioned into a register-held head of length 32 and a memory-resident tail of length 224.

For rank , define , and let .

The exact update is:

\boxed{
H’=\[v,H\[0\],\ldots,H30]
}\boxed{
T’=\[H\[31\],T0,\ldots,Tt-1,Tt+1,\ldots,T223]
}

The important detail is that the selected tail element is removed, while the displaced head element becomes the first tail element. Merely inserting into the tail would produce an incorrect 257-element logical list.

This gives a straightforward proof obligation:

H’\mathbin\Vert T’

\operatorname{MTF}(H\mathbin\Vert T,q)

for every , and therefore by induction for every sequence of valid ranks.

What to verify

Have the checker compare the partitioned representation against ordinary scalar MTF across random rank sequences, including repeated crossings of the 31/32 boundary, rank 255, and the original bzip2 symbol mapping.

Then evaluate the high-rank fallback’s cost separately. The measured 1.6% high-rank frequency on Wikidata is promising, but the tail update can be expensive, and #25 already demonstrated that correctness and register residency don’t guarantee a win.

Round 24’s intended outcome: either establish P74 as a fully correct alternative MTF state machine, leaving only its instruction-latency and CPU economics unresolved, or expose a correctness flaw before implementation.

After that, the independent literature report and instruction tables can finish their checks while we keep closing the existing questions.

So yes: keep burning. The construction proof is finished; move directly to the next independent uncertainty.
````
