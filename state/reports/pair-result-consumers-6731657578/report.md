# Pair result: retain STOP; separate the residual from the consumer

Researcher response to [inbox 6731657578](https://github.com/lmmx/mibz2/pull/162#issuecomment-6731657578). Instructions: `docs/team/SPEC*.md`, MODE, RULES and PROTOCOL at a4f97714cc14366984015869d9d094d89488ac57. Result reviewed: [journal at 96b0794d9605614e691b4356032a9120f1bb6baa](https://github.com/lmmx/mibz2/blob/96b0794d9605614e691b4356032a9120f1bb6baa/docs/journal/2026-11-09-pair-from-runs-result.md). Frozen candidate: **1c4d0f0714183f83798a0b746463ae34535fbc50**.

**Verdicts stand: Q1 INCONCLUSIVE; Q2 NEGATIVE STOP; exploratory M1 NOT MET. Revise the causal and impossibility wording, not those verdicts.** Direct-F resumed cursors remains the first construction to pursue for the existing one-hop consumer. For consumers, the greatest structural prize is skipping navigation through certified repeated output; the nearest bounded change remains shared-offset pair rooms, then flag-compatible tags. Neither is qualified by this report.

## 1. Judgment and provenance

I read the frozen registration §§6–7 and its C1 amendment, the frozen builder/walker/harness/validator, the GitHub run identity, the complete job log, artifact metadata, the result journal, and the prior Researcher and Optimiser reports/comments. [Run 42556998342](https://github.com/lmmx/mibz2/actions/runs/42556998342), attempt 1, is successful at the stated frozen SHA. Its log records 15 rounds, AMD EPYC 7763, four available CPUs, sampled 3.3091 GHz, and planned copies `1`, `4-cpu0`…`4-cpu3`. The validator prints VALID and the journal's readings. The single copy, pinned to CPU1, is the judged copy; four concurrent copies remain exploratory.

The raw ZIP was **not** downloaded or independently replayed. This is an independent check of the decision against the frozen rules and logged results, not a fresh certification of every raw sample. Artifact 13647977266 is named for the full frozen SHA, digest `sha256:d5ae792b4cc8e5637d923e63202c8687d83037071dfac7261d69a80843c496a7`. [evidence.json](evidence.json) records scope and log excerpts. Successful workflow status alone is not the validity argument.

| Reading | Logged value | Applying the unchanged rule |
|---|---|---|
| Q1 | .6865 [.6845,.6886] | Not ≤.50 with upper<.60; lower not>.80. **INCONCLUSIVE**. A real 31.35% relative build saving does not change that label. |
| Q2 | 1.8950 [1.8850,1.9050] | Lower>1. **NEGATIVE STOP**. Worst synthetic 1.6484 independently misses the positive screen; it does not change this to STOP(synthetic), which applies when Wikidata passes. |
| M1 | 3.457 [3.452,3.463] nominal c/B | Lower>1. **NOT MET**, exploratory. Clock-scaled wall time, not counted cycles. |

Q1 alone isolates the builder. R/O=.8139 is a **hybrid policy contrast**, because R declines two blocks that O takes. “19% whole pair saving” needs this qualification. Random/records fallback giving R/P about1.00–1.03 demonstrates the sampled selector's behavior on those corpora, not universal robust admission; the earlier aliasing counterexample remains.

The C1 amendment resolves the prior pin objection. Commit 1c4d0f07 has parent69f97c80 and changes only the registration. It explicitly accepts the changed synthetic text, 3,294,910 bytes from a33-file seed of823,717, before the03:32:53Z dispatch. The commit timestamp is03:18:45Z. It preserves thresholds,15 rounds, pilot exclusion and admission disclosure. This is the amended input's result, not evidence on the earlier3,258,654-byte text. The registration's stale DRAFT opening is poor labeling; the explicit amendment, dispatch pin and Coordinator's judgment identify what actually ran.

## 2. The residual is useful arithmetic, not a measured walk

The harness divides build times by **209,697,662 paired rows**, whole-entry times by **209,700,196 decoded output bytes**. Let κ=0.9999879161. Correctly normalized median subtraction gives:

| Arithmetic residual, nominal c/output-byte | Value |
|---|---:|
| R − κ·run-build | 11.705 −5.66793 = **6.03707** |
| O − κ·row-build | 14.427 −8.24590 = **6.18110** |
| P − κ·one-hop-build | 6.184 −2.21297 = **3.97103** |

Reasons these are not consumer measurements: separate timing brackets and persistent buffers; differences of medians are not medians of paired differences; R's build-only arm visits all233 paired blocks but its whole arm accepts231; short/fallback blocks, admission, setup/flags, allocation and joins remain in the whole entry. The residual includes other work and subtracts work not executed in exactly that whole arm. There is no confidence interval on the hypothetical free-build intervention. RLE1 makes the normalization mismatch material on other corpora: one-byte has98,040 rows for5M output bytes. Its run-build contribution is about.03553 c/output-byte, not1.812.

**Correct the journal's “now measured” and “no builder can carry Q2” assertions.** The measured fact is the1.895 whole-entry loss. About6 c/B is a residual estimate compatible with a costly consumer. It does not identify that consumer's exact cost or prove that every builder-only redesign fails. Even this rough model puts a free build at6.037/6.184=**.97624**, slightly faster than P, though short of Q2's.95 point target. The original §7 correctly called the free-build statement a prior, not a necessity. C3 and §6 explicitly allow reopening through a materially cheaper builder or another justified mechanism; retain that scope.

More useful than “consumer first” is a **joint cost frontier**. Treating the residuals as an additive model only, the unchanged5% target is5.8748 c/B:

- With the current5.66793 build, only **.20687** remains for everything else. Making the consumer merely as cheap as the estimated production residual3.97103 would leave9.63896, about1.559×P.
- With consumer residual3.97103, the pair builder must fall to **≤1.90377**, about66.4% below the current one.
- With builder cost equal to the estimated one-hop2.21297, the remaining budget is **≤3.66183**.

These are sensitivity calculations from rounded medians, **not new gates or predicted intervals**. A pre-built consumer win is neither sufficient nor logically necessary for a whole-entry win: build savings can offset a slower walk, and a faster walk can fail to pay a dearer build. The journal's consumer gate is a useful diagnostic priority only. Match residency, markers, segment starts, output ownership and assembly; then retain whole entry and whole decoder as the decisions. Table warming by itself does not establish production behavior.

## 3. A translation interval is not a text run

Own derivation, checked below. For an L run J_i=[p_i,p_i+ℓ_i), character c_i, its F interval I_i=[a_i,a_i+ℓ_i) satisfies Ψ(q)=q+(p_i−a_i). That describes **adjacent source rows under one application**. The decoder follows q,Ψ(q),Ψ²(q),…; those states need not remain in I_i or be adjacent. Scanning an entire translated image would generally emit bytes in row order rather than text order.

For the current pair convention, a step emits **L(q),F(q)=L(Ψ(q))**, then advances to Ψ²(q). The registered pair construction partitions its q domain into at most2r−1 cells on which Ψ²(q)=q+δ. Every such cell is inside an I_i, so F(q) is constant there. Refine by L-run boundaries: at most another r−1 cuts. Therefore **at most3r−2 cells suffice to make both emitted bytes and δ constant**. This is a descriptor bound, not a claim that descriptors fit in the existing word or that every visit skips work.

On a refined cell H=[u,v), with fixed pair W=(a,b), starting q∈H:

    δ>0: m = floor((v−1−q)/δ)+1
    δ<0: m = floor((q−u)/(−δ))+1
    δ=0: m is limited by remaining output, not H's width.

Cap m by complete pairs remaining and every required segment/marker/buffer boundary. Then emit W repeated m times and set q←q+mδ. All m source states are in H; the final successor may leave it. This replaces m pair navigation loads with one descriptor acquisition and arithmetic. Output stores remain; initialization, directory access, integer division or reciprocal preparation, tails and descriptor construction must be charged. A δ=0 cell can describe period2, not necessarily a one-byte run.

**Counterexamples and extension.** For text `ababab`, L=`bbbaaa`, Ψ swaps the halves and Ψ² is identity: pair batching is ideal despite a one-hop interval exit every step. For `abcabcabc`, L=`cccaaabbb`: all three pair cells have length3, but δ=6 or−3, so every pair visit exits immediately and m=1. Increasing the repetitions makes the cells arbitrarily long without improving pair batching. Low r/n, entropy, aligned16 coverage and cell length do not certify a skip.

The second example suggests the stronger mechanism: **compose a short cell path into a repeated packet**. Suppose k successive pair cells have deltas δ_j and constant pairs W_j. Let s_0=0 and s_j=Σ_{i<j}δ_i. The source interval following that same path is

    H = intersection_j ([u_j,v_j) − s_j),
    W = W_0 W_1 … W_(k−1),   Δ = Σ_j δ_j.

For q∈H, k steps emit W and reach q+Δ. Apply the same m formula to H and Δ to emit W^m and jump q+mΔ. In the period3 example, three pair steps return to the same row (Δ=0) and emit a six-byte repeated packet; a one-cell-only design misses it. This is **not** permission to copy the L run. It is a certificate of an output sequence and its successor. Bound k, descriptor/cache population and first-visit costs; an unrestricted per-row path cache can recreate the dense-table expense.

For flagged lanes, a packet cannot hide a START or PAIR_LF event. If a marker is at row f and the jth internal source has offset s_j, its packet occurrence satisfies q+tΔ+s_j=f. Stop before the smallest nonnegative integral t in the batch, and use the original exact endpoint semantics. A START emits no ordinary pair; an LF endpoint emits one byte. Known output length caps δ=0/Δ=0 cycles and odd tails. Never infer safety from a long arithmetic cell. Output is BWT-domain/RLE1-encoded bytes; emitting a repeated byte directly to final decoded output is not licensed.

**Finite checks:** [oracle.py](oracle.py), [oracle.json](oracle.json). All9,840 ternary columns of lengths1–8;73,812 source states validate the3r−2 bound, pair output, arithmetic successor and single-cell marker cap. Another295,248 checks validate composed packets of1–4 pair steps, including Δ=0. These are untimed mathematical checks, not bzip2/unsafe/segmented decoder validation, a workload census, or speed evidence.

## 4. Prior art and rank by work removed

Primary sources freshly read:

- [Kärkkäinen–Kempa–Puglisi, DCC2012, pp5–7](https://www.cs.helsinki.fi/u/tpkarkka/publications/dcc2012.pdf): Precopy uses s-runs and previously recovered output to skip navigation; Lemma8 relates multiple LF steps by a row offset. This supports looking beyond constant-byte output. Its sentinel convention, metadata, initial discovery and copying obligations still need adaptation.
- [Kärkkäinen–Puglisi, CCP2011, §IV](https://www.cs.helsinki.fi/juha.karkkainen/publications/ccp2011.pdf): recording repetitions during inversion permits later sequential copies. Forward Ψ adaptation must preserve an available source region and overlap semantics; “memcpy” is not automatically correct for an overlapping repeated pattern.
- [Brown–Langmead, SEA2026, Theorem2/Corollary3](https://drops.dagstuhl.de/storage/00lipics/lipics-vol371-sea2026/LIPIcs.SEA.2026.9/LIPIcs.SEA.2026.9.pdf): a move structure can replace dense permutation storage with interval/offset state; length capping bounds average work across distinct queries. Full-traversal amortization assumes a single cycle. Cyclic BWT for `ababab` has multiple short cycles, so the theorem cannot be transferred unchanged. O(1) move queries also do not mean fewer queries per output byte.

No historical timing above is a performance prediction for this decoder. The packet derivation is an application to this pair convention, not a claim to have invented repetition-copy inversion.

| Structural rank | Removed work per BWT output byte | New costs and falsifier |
|---|---|---|
| 1. Certified packets / repetition copy | A k-pair packet repeated m times can replace km pair navigation loads by one packet acquisition: up to(1/2−1/(2km)) loads/byte. Can also avoid dense link materialization on that represented region. | Discovery, descriptor lookup, exits, flags, retained output, copy bandwidth and setup. First measure **actual trajectory coverage after marker caps**, not row-cell sizes. Low km or expensive first visits kills the gain. |
| 2. Shared-offset pair rooms (C37) | Remove per-lane output-pointer updates and capacity checks, currently incurred every2 bytes; regularize staging and lane state. | Tile-copy traffic, segment joins, odd endpoints and replacements. Current source already has32 lanes, flags and next-row prefetch; those are not new savings. A256-byte room is128 pair rounds. Whole consumer including assembly must win. |
| 3. Two-bit tags (C38) | Ordinary step can remove the dependent `f[c+1]` load and comparison; one smaller directory access still remains, per2 bytes. | Extraction/masking, exceptional blocks, tag writes, markers. The .002 correction iterations/step do not price the initial dependent load. Census actual segmented visits. |
| 4. Compressed move state | Reduces navigation footprint from4n bytes toward O(r) words, possibly improving residency. | Does not automatically reduce one transition per step; interval repair and representation decode may cost more than the dense load. Need a bound valid for periodic cyclic inputs. Combine with packets only if their joint setup pays. |
| 5. LF orientation / hybrid literals / admission | LF removes constructor destination-symbol recovery; hybrid input avoids some emitter records; admission avoids losing blocks. None alone deletes the current pair consumer's per-byte navigation. | Reverse output/RLE1 and assembly for LF; random-access indexes for hybrid literals; decision/instrumentation cost and false admissions. Rank separately as construction or representation work. |

**Tag census correction:** the frozen `spare_bits` uses `F>>3`, a **three-bit tag** model. Its .343% dynamic exceptional rate and bound31 blocks do not certify two-bit C38. C38 needs `F>>2`, with at most63 boundary-crossing256-row blocks; its dynamic rate must be recomputed. Bits30/31 are PAIR_LF/START. Bits28/29 are spare only with links below2^20; the current safe API accepts n≤2^22, so a two-bit representation must check/narrow its domain or retain an explicit wide fallback. Every successor extraction must mask the tags. The Optimiser already identified the flag conflict in6730372937; this run confirms it against source and adds the census-transfer boundary.

Aligned16-window admission remains useful for the one-hop builder, not a proxy for consumer packet coverage. Consumer admission needs a prospective model of saved navigations per decoded byte, packet acquisition/repair cost, and actual marker-truncated visits. Even r and cells/r cannot recover those trajectories. Keep the rule fixed before evaluation; an after-the-fact best-block chooser is only an upper bound.

## 5. One-hop construction and the next consequential check

**Direct F with resumed cursors remains first among constructions preserving today's consumer.** For each byte c, destination cursor a[c] advances monotonically within its disjoint F bucket. Seed the L predecessor once per bucket; resume it across later runs. All bucket cursors together cross at most r−1 L boundaries after seeding. This removes an F-record materialization and repeated predecessor search while retaining T[q]=L[q]|Ψ(q)<<8. It must still pay exact/bucket-confined tails, multiple write streams, input/emitter handling and fallback. The5.668 pair-build result does not price this one-hop construction: it constructs Ψ² with additional run ordering, intersections and state.

Its prize is narrower but nearer: the current2.213 one-hop build is about35.8% of the6.184 entry under the same rough accounting, not35.8% of the whole decoder. A same-consumer build+walk comparison can test the smallest unresolved production-relevant question. For a run-fed route include MTF/emission and helper critical-path costs, not only the fill. LF-oriented sequential construction remains the next larger alternative, conditional on paying reverse reconstruction/RLE1 costs.

Packet/repetition consumers displace direct F only if a bounded census shows enough **real skipped transitions**, with setup, markers, source availability and output handling charged, to offer greater recoverable whole-decoder time. For pair work, C37 and then C38 are nearer implementation questions, but the joint frontier shows that a modest consumer improvement alone is not enough at the current pair-build cost. Do not require all consumer ideas to inherit the expensive dense pair builder.

Expected: Q1 INCONCLUSIVE and Q2 STOP. Observed: both;31.35% relative row-builder saving, no production qualification. Mechanism learned: representation costs are separable; current source exposes pointer/capacity bookkeeping and dependent first-byte lookup, while the interval algebra permits certified multi-step output. Binding hardware cost remains unmeasured. Next value: matched one-hop direct-F construction, and a bounded trajectory census comparing one-cell batches with composed packets/repetition reuse. No timing, implementation, merge, new experiment admission or authority change here. I proposed the original run-pair construction; this is an involved scientific review, not an uninvolved B5 safety sign-off.

Routing audit: five resolved historical items only, destination#145. Migration branch and manifest digest verified. The pinned publication verifier's Coordinator-only sender check will be reported literally and supplemented with exact body/destination and Researcher header checks.
