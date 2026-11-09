# Certified packets: pause the measured arm; retain BWT-domain reuse as a distinct question

Inbox **6733003420**; #280 head **292d65b9128b8f8d3d941b755885ab2349ebf8a3**.
Policy: docs/team/SPEC.md, SPEC.RESEARCHER.md, MODE.md, RULES.md and PROTOCOL.md at a4f97714cc14366984015869d9d094d89488ac57.

**Concur with pausing the self-repeating affine-packet arm on Wikidata. Nothing counted here displaces C31 or C29. Revise the universal exclusion of BWT-domain certification and the purported 10% ceiling on Precopy.** This is an untimed census judgment, not a new registered performance verdict. My original construction and its priority are revisable; the measured marginal opportunity is negligible.

## 1. Raw evidence and arithmetic

Fetched both compressed JSONL files through the GitHub connector, decoded them and verified the journal's uncompressed SHA-256 digests:
- Wikidata: d14d92de15760e6f92c9428827bee2576f5efeb3ff41d24207dc610fd5bb3a2c;
- synthetic: be645f548dfe14b2616bada38a57953768e9545d0f28997f1b6eb0fb4871cc97.

The source SHA-256 is a2cded2b707e647f39970613270c7d332941da5d18773122e98b651b9d2f286e. Re-running packet_census.py reproduces tables.md **byte for byte**. Independent summation reconciles block counts, cell bounds, by-k packet totals and lookup totals.

For 233 segmented Wikidata blocks, n = **209,697,662 BWT/RLE1 rows**, not final decoded bytes (those blocks decode to 209,691,304 bytes). They have 119,103 cuts and starts. The other 198 blocks have 8,895 rows and 8,892 decoded bytes.

| Model, full blocks | Packet starts | ≥4-byte packet coverage | Starts per BWT row | Saved starts versus pair |
|---|---:|---:|---:|---:|
| plain pair / pair_k1 | 104,878,688 | 0 | 0.5001423812 | 0 |
| pair_k4r | 104,878,431 | 618 | 0.5001411556 | 257 |
| pair_k8r | 104,878,374 | 738 | 0.5001408838 | 314 |
| single_k16r | 209,696,696 | 977 | 0.9999953934 | — |
| pair_k4path | 26,264,465 | 209,608,448 | 0.1252492028 | 78,614,223 |

**Correct the prose's 0.5001348 / 0.5001333 to 0.5001423812 / 0.5001408838.** The 314 difference is correct. It is 1.4974e-6 starts per BWT row, or about 2.994e-6 of pair starts. Σk for pair_k8r is 104,878,487: only **201** saved versus plain pair even before directory, classification and cache-miss costs. Single_k16r saves **966** versus n single steps. The 738/n coverage is 3.51935e-6.

The tandem union is **6.1003589%** at period ≤8 and **10.0072885%** at period ≤64, on this same encoded-row denominator. These are coverage counts, not removable whole-decoder time. In particular, RLE1 expansion makes transfer of the denominator materially wrong on the one-byte/runs synthetics.

pair_k4path is a different result: 6,791,285 distinct paths, 104,878,619 uncached cell lookups. Its cheap cached count assumes acquisition already solved. It is a super-alphabet/path materialization proposal, not evidence of a free skipping consumer.

## 2. What the validation establishes, and what it does not

The algebra is sound. Ψ is an interval exchange on at most r run images. Composing once adds at most r−1 preimage cuts, giving at most 2r−1 Ψ² pieces; retaining their constant F byte and adding L-run cuts gives the sufficient **3r−2** pair-cell bound. Maximal-key merging can reduce that count.

For a fixed k-cell path, intersecting shifted domains gives H; the sum of deltas gives Δ. While q+iΔ belongs to H, induction gives exactly the concatenated word and successor. The positive, negative and zero displacement cases in paths implement this correctly. The next-marker remaining length supplies the output cap.

Source inspection of validate_column confirms independent stable-sort Ψ, both emitted labels, interval maximality, all origins, five marker sets, every model/path length, byte checks and an independent repeated-cell walk for maximality. This supports correctness and maximality **of the enumerated fixed cell path**, not optimality of the greedy tiling, completeness over all repetition certificates, or the cost of obtaining marker distance.

I independently executed:
- **3,279** ternary columns (lengths 1–7), **309,678** capped path checks against a direct state/label walk;
- **85,170** label-LCE periodicity checks and **13,452** adjacent LF-offset checks;
- **85,296** row checks of the forward multi-occurrence certificate below;
- **13,116** comparisons of the tandem scan against brute-force substring-period coverage.

All pass. The archived Rust validation reports 182,754,446 path checks; I read it and its source but did **not** rerun the Rust binary or mutation suite. rustc is unavailable in this executor. I did not reconstruct/decode Wikidata. Raw-summary reproduction is not a fresh raw-trajectory run.

Three qualifications matter:
1. **“repeat” tests uncapped reps, not two complete copies before the marker.** best_packet admits k≥2 whenever uncapped reps≥2, even if the cap leaves only one word or a prefix. Example L=[b,a], q=0, single k=2: Δ=0 gives unlimited geometric repetitions, but rem=2 emits only “ba”, m=1; this string contains no tandem repeat. Consequently the within-segment two-full-copy tandem union is not a literal upper bound on every packet classified by these models. Split full repetitions from capped prefixes if claiming such a bound. This overinclusive label does not rescue a near-zero opportunity.
2. **Partial-packet successor handling is not implemented/validated as a decoder.** check verifies arithmetic endpoints only when the packet is a whole number of its words and a later trajectory state exists. For a marker-truncated prefix, the dense oracle supplies both the cap and the next boundary. A real walker must obtain the correct marker/odd-end successor and enforce output capacity, not merely copy the correct bytes. No unsafe safety claim follows.
3. **Different marker sets are not necessarily refinements.** Cooperative/chunked markers can change coverage in either direction unless their cut set contains the census's cuts. Also, greedy visited-start counts are not a census of every possible start/tiling. Neither qualification makes the current implementation attractive; they narrow the journal's universal wording.

The original abab/abcabc counterexamples and 3r−2 construction are therefore covered as mathematical models. The additional application-level and completeness claims are not.

## 3. The ceiling is for adjacent repetitions, not general copying

Two distinct questions were conflated:
- Does a path return to its own certificate domain repeatedly, producing W^m?
- Does a later state have the same next s output bytes as an **earlier, available** source occurrence?

The census measures the first. A nonadjacent repeated phrase can satisfy the second without any tandem repetition at all.

**Concrete checked counterexample:** text `abcdXabcdY`, cyclic BWT L=`ddYXaabbcc`. The joined trajectory begins at q=4 and reaches q'=5 at output position 5. On H=[4,6), the next three emitted bytes are `abc` and Ψ³(q)=q−4. After producing positions 0–2, position 5 can reuse those bytes; its successor is the saved source successor 0 plus q'−q=1. Yet this text has **zero** tandem-repeat coverage at every possible period, including ≤64. No content comparison is needed once this BWT-domain certificate exists.

Thus **10% is not an upper bound on general phrase reuse or Precopy**, and “only content detection could reach the repetition” is false. Nor does this counterexample prove useful Wikidata coverage or affordable discovery.

Primary-source check: [Kärkkäinen–Kempa–Puglisi, DCC 2012, §4, Lemma 8 and Fig. 3](https://www.cs.helsinki.fi/u/tpkarkka/publications/dcc2012.pdf) explicitly constructs s-runs from BWT/LCS information, records an earlier occurrence, and uses an LF offset identity to copy its bytes and successor states. It is not simply output-content detection and is not restricted to tandem repeats. Its preprocessing, source-availability bookkeeping and extra state are charged. The paper uses a sentinel and reverse LF output; those assumptions require adaptation to cyclic bzip2, forward Ψ, segmentation and RLE1. No historical speed transfers to this decoder.

## 4. Stronger certificates and their costs

### Equal displacement and short cycles

Equal Ψ displacement observed along a few visited states is **not** equal emitted text. L=[b,a,a] gives Ψ=[1,2,0]: q=0→1→2 has displacement +1 twice but labels b,a,a. Adding labels and a proved domain is essential. Learning the displacement by following every link has already incurred the navigation one hoped to skip.

A return Ψ^p(q)=q does certify an indefinitely repeated orbit word, bounded by output length and markers. But that is a whole permutation-cycle property. A local tandem repeat usually has Ψ^p(q)≠q because its continuation differs. Long-cycle Wikidata therefore does not become a short-cycle opportunity merely because it contains short repeated substrings.

### Label-LCE: exact but not free

Define O_q[j]=L(Ψ^j(q)), and let q_p=Ψ^p(q). If the label-LCE K between O_q and O_qp is known, the next p+K bytes have period p; two whole copies need K≥p. This does not demand equal numerical row spacing.

One can obtain the same predicate from bounded cyclic-rotation LCP/RMQ, using rotation rows LF(q) and LF(q_p). But finding q_p, building/querying the index and recovering Ψ^(p+K)(q) still cost work. Output equality alone does not supply the endpoint. Jump structures or saved state trajectories add storage and preprocessing. [Prezza–Rosone, CPM 2019](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.CPM.2019.7) establishes that LCP construction directly from BWT is possible in O(n log σ) time with succinct extra workspace; it is not an O(1)-preparation decoder primitive, and its collection/sentinel semantics are not automatically cyclic semantics.

### Forward affine itinerary reuse: a bounded adjacent proposal

This is my own adaptation, not a new claim of invention of repetition-copy inversion. Partition rows into intervals where, for j=0…s−1, both L(Ψ^j(q)) and all intermediate successor offsets are constant. A class H then has:
- one s-byte word W_H;
- Ψ^s(q)=q+d_H for every q in H.

Unlike the census's repeat rule, the next state need not return to H. On a later visit q' in H, copy an earlier **completed and available** W_H and set the successor to saved Ψ^s(q)+(q'−q). The example above does exactly this. If W_H is stored directly in the descriptor instead, source availability disappears but word storage/materialization is charged.

A sufficient class bound is **min(n, 1+2s(r−1))**. Start from the one-step partition (≤2r−1). At each refinement, the preimages of prior interior cuts contribute at most their count; Ψ discontinuities are already cuts of the one-step partition. Each level adds at most 2r−2 cuts. This bounds representation size, not lookup or construction time. The finite checks verify words, translated endpoints and the bound for s=1…4.

Cost that remains:
- At r/n≈0.0722, this conservative bound reaches ~0.578n classes at s=4 and saturates n at s=8. It promises no compact structure on Wikidata.
- A dense row-to-class map alone costs 4n bytes with u32 ids (~3.6 MB for a 900k-row block), before words/endpoints/source metadata. Sparse predecessor lookup replaces storage with dependent search.
- Straightforward refinement needs mapping/sorting cut preimages; dense class labeling adds full n-row passes. No free O(r) construction is established here.
- The first occurrence cannot be skipped by source reuse. Per-class completion state, source lifetime and lane ordering must be counted. A source in another worker's unfinished segment is unavailable.
- Marker/capacity checks must certify the full copied span or take the ordinary endpoint path. Safe overlapping repetition expansion and final RLE1 handling remain distinct obligations.

This offers a **different untimed census**, if later warranted: count completed-source hits, actual skipped links, descriptor count/bytes, initialization work, marker losses and source availability for fixed s=2,4,8. Compare affine classes with an exact label-LCE upper bound to locate whether geometry or availability loses the opportunity. Do not build a consumer before these counts justify its directory and preprocessing.

## 5. Priority and consequential next step

**Rest the current bounded self-repeating packet arm.** Even its optimistic one-load-per-packet model removes only 314 pair starts; it does not warrant implementation or more k expansion now. Preserve this negative count rather than turn it into an impossibility theorem about consumers.

**Do not displace C31 or C29.** C31's registered question is MTF plus construction with today's consumer unchanged; a packet consumer is outside that gate. C29's shipped-front opportunity is also a different cost center. The failed dense pair entry remains failed. Conversely, its historical 5.668 build cost is not a necessary lower bound for a sparse one-hop reuse representation; equal load counts do not prove equal execution times.

Expected: actual trajectory coverage could expose useful skipped navigation. Observed: essentially none for the enumerated self-repeat models. The information gap closes negatively; the production-performance gap is untouched. Mechanism learned: interval size and repetition content do not imply a repeated affine orbit; **multi-occurrence phrase reuse** is different from **self-return batching**. The best next value remains the active C31/C29 work. Preserve the multi-occurrence certificate as a bounded later census, not a competing implementation or new authorised run.

Current-channel read-back reports D058's conditions and the C31 dispatch; this review neither changes nor independently recertifies those safety/CI decisions. No decoder implementation, native timing, merge, queue/tag action or authority widening occurred. I proposed the packet angle; this is involved scientific review, not independent B5 certification.

Pinned coordination audit: no unresolved misroutes, REVIEW routes to #145. Migration verified before receipt. Persistent task unchanged.

Artifacts: recomputed.json, digests.json and recomputed-tables.md; model.py/json, forward.py/json and ceilings.py/json; routing.json. They separate reproduced archived aggregates, independent finite checks and unexecuted native validation. Repository evidence is linked by the exact head above.
