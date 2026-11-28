# Outside review of the main-theorem challenge (ChatGPT Pro, 24 November 2026)

Commissioned by Louis as an independent adversarial review of the formulation. Relayed verbatim. The Coordinator's notes are at the end.

## A. Adversarial review

[Argued] The challenge is not yet stated in its fullest honest sense. The central ambition survives scrutiny, but three parts of the formulation need substantial correction: c_1 is a statistic of a derivation calculus, not a machine-cost lower bound; hex singleton rows have been conflated with independent random content; and the proposed conditional theorem omits costs that its own discussion acknowledges. The claimed exhaustion of alternative algorithms is also stronger than the arguments support.

[Argued — scope] I take the strict reading to require exact decoding of valid standard bzip2 inputs, all preprocessing charged, no corpus-specific advice, explicit resource limits, and a substantial improvement through the real decoder. I read all seven pinned documents. Repository measurements below are reported measurements, not independently rerun benchmarks. Proved denotes a direct argument or a cited theorem under its stated assumptions; argued denotes a supported assessment; conjectured an open mathematical claim; guessed an empirical prospect without adequate evidence.

### 1. Is the challenge stated honestly enough?

**The ways it can be gamed, and their repairs**

| Issue | Finding and stricter restatement |
|---|---|
| A cheap margin | [Proved] c_1<h_1 admits a one-event saving. Fixing \delta>0 removes that particular loophole, but choosing a tiny \delta still does not establish a structural advance. [Argued] Keep the proposed margin explicit, add an independent whole-decoder utility threshold, and require a new cost law with charged acquisition. Lowering \delta after seeing results constitutes a changed challenge. |
| A finite family \mathcal F | [Proved] The draft's finite set of 232 blocks plus six admitted blocks admits a lookup-table solution unless program size, advice and stored answers are constrained. Asymptotic O(r) notation on a fixed finite set also imposes essentially no useful constant bound. [Argued] State the theorem over an infinite, uniformly described structural family; use the frozen corpus and held-out blocks for validation. Give explicit byte limits for actual bzip2 block sizes. |
| Choosing an easy family | [Argued] A family selected to exclude random-heavy blocks can make the theorem true while avoiding the expensive workload. Require prespecified, cost-weighted coverage of the primary workload and report results by difficulty stratum. Charge recognition and fallback. Correctness should hold generally even when the improved bound applies only to the promised family. |
| Moving cost outside c_1 | [Proved] Wider-subject operations cost zero under the draft's definition, irrespective of their computational expense. A global operation can therefore contain large amounts of work without increasing c_1. [Argued] Define costs independently of fact-language labels, over the complete executable computation; retain c_1 as a separate structural diagnostic. |
| An ambiguous pass budget | [Proved] If only input-stream passes count, an algorithm can read the input into O(n) RAM, compute or sort indefinitely there, and write the answer in a second pass. This matters whenever r=\Theta(n). [Argued] Specify accessible memories, read/write permissions, working-array scans, sorting, total traffic and work. A fixed K=3 needs operational meaning. Conversely, sorting that genuinely fits those budgets should not be disqualified by name. |
| "Beyond the output" workspace | [Proved] Exempting an unrestricted output buffer permits it to serve as substantial uncharged scratch space. [Argued] Exempt only committed final output storage. Charge temporary use, rereading, rewriting and scattering, with explicit peak live memory. |
| N/R as an imposed architecture | [Argued] An analytical split need not be an efficient algorithmic decomposition. A joint algorithm might exploit correlations across it or avoid constructing row positions altogether. Permit such algorithms. Require exact output bytes, not an obligatory N/R interface or an obligatory suffix array. |
| "Outside \mathcal R" as novelty | [Proved] Being outside a chosen formal language does not imply being a new algorithm. [Argued] Require comparison with prior art and an actual computational advantage. A different proof can produce the same computation; a new fact constructor can merely rename an old one. |

[Argued] The charter already contains useful safeguards: cost-weighted applicability, supplied-capability payoff and final native performance. The repairs above make those safeguards enforceable. They should not be replaced by the easier test "a new fact type appears." (Frontier, target; Method, §§3a, 4, 6a; conversation, §§12–24.)

**The current main theorem does not follow from its premises**

[Proved] The draft's Pmargin bounds only a_N h_{1N} + a_R h_{1R}, while Pglue supplies correctness of interleaving without a corresponding event-cost bound. Its conclusion also promises O(r) state without hypotheses bounding the stream decoders' or glue's state. The pass inequality omits the split unless its work is already included elsewhere. These are missing hypotheses, not unfinished optimisations. (Draft, §§3–4.5.)

[Argued — stricter statement] Use a complete accounting:

C_total ≤ C_split + C_shared + C_N + C_R + C_glue,

and require this entire expression to meet the margin. Alternatively, prove that every term has already been charged exactly once inside C_N + C_R. Add a joint peak-memory bound and a pass bound for the actual combined schedule.

[Proved] Partitioning an existing trace into two sets of events proves an accounting identity. It does not construct two independently executable decoders, supply their inputs, or produce their interleaving. Similarly, the aggregate inequality c_1 ≥ h_1, together with additivity, does not imply c_{1,R} ≥ h_{1R}; that requires a stronger, row-preserving charging argument.

[Argued] The boundary bound also needs a precise domain. On a cyclic sequence partitioned into sets N, R, the elementary switch bound is 2 min(|N|,|R|). Replacing those cardinalities by singleton counts is justified only if the sequence being counted consists of singleton rows. For the full orbit, intervening thick-run rows can create additional boundaries. (Draft, §4.5.)

### 2. Is c_1 the right currency?

[Proved] No: c_1 < h_1 alone does not imply fewer dependent cache misses. The count records where certain mathematical operations occur. It does not record cache residency, access locality, physical transfers, dependency depth or instruction cost. Those distinctions can reverse an algorithm's ranking.

**A published counterexample to identifying compositions with misses**

[Proved — cited cache model] Kärkkäinen, Kempa and Puglisi's *Slashing the Time for BWT Inversion*, Theorem 2, analyses an algorithm that constructs and walks LF^2. Including preprocessing, it incurs at most ⌈n/2⌉ + O(σ_T + n/B) cache misses, provided max(σ, σ_T B) + O(B) ≤ M. Here M is cache capacity in words and B the transfer width. The construction is cache-efficient because LF values form σ_T interleaved increasing sequences. Singleton compositions therefore need not incur singleton cache misses.

[Argued] This is a calibration example, not a recommendation to repeat the project's failed T² experiment. T² is already in the records. Its significance is that an algorithm can remain within the rigid calculus, pay more c_1 events, and reduce memory misses. Conversely, a non-rigid procedure can reduce c_1 while creating expensive construction, gathers or output stores. A proof about c_1 cannot decide between them.

[Argued] There is also tension within the project's own economic account. Record 96 says the remaining per-row price is the emit loop's work, rather than latency; Method §6a expressly requires a latency-free ablation before transferring latency arguments to the real kernel. Dependent accesses are a valuable diagnostic, but their dominance must be demonstrated on the chosen machine and worker count. (Record 96, §§4–6; Method, §6a.)

**A replacement that remains yes/no**

[Argued — proposal] Fix a memory model 𝓗: word size, cache level and capacity, line size, initial-cache convention, worker count and allowed concurrency. Measure the entire algorithm, including acquisition and output placement, using:
- D_𝓗: total cache-missing accesses whose addresses depend on earlier loaded data;
- S_𝓗: longest chain of dependent misses;
- Q_𝓗: all cache-line transfers, including independent traffic and stores;
- W: computational work;
- peak memory and passes.

[Argued — stricter target] Against a pinned baseline A_0, require D_𝓗(A,c) ≤ (1−δ_D) D_𝓗(A_0,c) on the declared family, together with fixed bounds on S, Q, W, memory and passes. Count misses across all chains, rather than only the longest chain. Permit explicitly budgeted extra sequential work where it pays.

[Argued] This is a pass/fail conjunction, not a score. Its empirical counterpart should require a prespecified whole-decoder improvement η, beyond measurement uncertainty, with no primary-corpus regression. I would use a substantial hurdle such as 10% for the completed improvement, rather than letting "any detectable win" certify the major shot. The existing 10% supplied-capability headroom gate provides a starting economic standard. (Method, §4.)

[Argued] Keep two certificates separate: the new structural cost theorem, and the machine result. There is no sound reason to make h_1 the machine baseline until a theorem connects it to that machine currency. The published LF² analysis shows why that connection cannot simply be assumed.

### 3. Is G1 proved, and does 𝓡 contain everything claimed?

**The narrow mathematical idea is sound**

[Proved — conditional lemma] For forward rigid facts, there is a valid invariant: if every row of an interval emits the same word, each intermediate image of that interval lies inside a single maximal run. The first common character places the interval inside one run. LF translates that interval. Apply the same argument to the remaining common word. In particular, a fact passing through a singleton run must itself have width one.

[Argued] This supports the proposed injection proof: for each covered singleton row, identify either its output-chain link or a composition event whose join occurs there. Distinct rows cannot share a singleton event's unique join row. With an operational derivation model and the right orbit-coverage hypothesis, this should yield c_1 ≥ h_1. The join rule is not an obvious flaw: equal words on adjacent domains preserve the invariant.

[Argued] Thus I would not reject G1 merely because it lacks Lean verification. A correct prose proof is a proof. But the current record is a proof sketch for a partially specified calculus; it is not yet the claimed theorem about every executable decoder in the enlarged class. (Record 98, §1.)

**Four specification issues need settlement**

1. [Proved] Primitive charging is inconsistent. The definition charges width-one rule applications, and prim is a rule. Yet the claimed Open count treats primitive input facts as free. Decide which convention applies; otherwise the floor may be paid before any computation, or counted twice. (Record 98, §1; Draft, §2.)
2. [Proved] Event attribution is incomplete. Event.row explains composition and chain-link events, whereas c_1 also counts other width-one rules. Every charged event needs an attribution, or P1 must use a different complete definition. Inverse facts also need an oriented formulation: h = −1 is incompatible with the draft's h : ℕ+.
3. [Argued] Reuse needs operational semantics. Charge the actual construction DAG and execution, not an expanded proof tree. Otherwise a short shared computation can be assigned an artificially large cost merely by unfolding its proof.
4. [Proved — conditional domain counterexample] If Column permits arbitrary columns, "every column" is false under the intended free-input-fact convention. Take L = abcdefgh. Then LF = id, h_1 = 8, and primary row 0 emits eight copies of a. Three singleton doublings build a length-eight fact; one output link gives c_1 = 4 < 8. The other seven singleton rows are never visited. This is not a counterexample on a valid ordinary single-string BWT; it demonstrates why realizability and orbit coverage must be explicit.

**Membership is not established by recognising a familiar operation**

[Argued] Record 99 correctly retracts the earlier zero-cost claim for conventional pointer jumping: composing x → LF(x) → LF²(x) is a singleton composition. Conventional splicing has a similar interpretation. But that does not prove a cost-preserving simulation of every algorithm called list ranking, contraction or sorting. Those names denote broad families. (Record 99, §2.)

[Argued] The required bridge is: this executable algorithm lowers to these calculus events, with this relationship between its actual work and the charged events. Expanding a bulk computation into equivalent singleton facts is insufficient; the expansion may manufacture the cost the lower bound purports to establish.

[Argued] E1 is therefore not closed. A scattered-class fact is outside the literal interval-fact syntax. Showing that pointwise evaluation at k = 1 produces an LF successor does not show that every bulk use must perform separately charged pointwise evaluations. "It still needs output ordering" identifies an open obligation.

**Two other "closed" branches are overstated**

- [Proved] "Run labels determine the text" is a reduction, not a lower bound. It prevents treating supplied labels as an innocuous free oracle. It does not prove that a different algorithm cannot compute those labels cheaply.
- [Proved] Even accepting Gamma's claimed completeness result, L-completeness under uniform TC^0 reductions gives: coordinate problem ∈ TC^0 ⟹ L = TC^0. It does not unconditionally exclude such coordinates. Nor does worst-case completeness automatically apply to a restricted structural family.
- [Proved] Global sensitivity does not imply large sequential cost. A function can depend on the entire input and still be computed by a simple streaming reduction. An indistinguishability argument must establish an indistinguishable algorithm transcript under the actual access model.

[Argued] These should be labelled conditional barriers or acquisition obligations, not globally closed branches. (Draft, §5.)

**Known techniques that challenge the coverage claim**

[Proved — literature] Beller, Gog, Ohlebusch and Schnattinger compute LCP structure directly from the BWT through interval refinement and wavelet-tree operations. Those facts are not the calculus's translation facts. This is a concrete example of global suffix-structure computation not captured by simply classifying every rank operation as a singleton successor. It does not, by itself, provide text positions or a faster decoder.

[Proved — literature] Kempa's RLBWT-based constructions obtain SA/ISA support and LZ77 representations without first requiring the fully decoded text as input. These deserve a specific acquisition and provenance audit. Their published bounds do not establish the present O(r)-space, fixed-pass practical target.

[Argued] Kempa is already mentioned in the frontier, so the missed point is not necessarily the citation. It is that known positional and compressed-string constructions prevent "no construction is known" from being used without specifying which resource bound is missing. I found no verified existing algorithm satisfying every strict requirement here. I did find that the claimed exhaustive classification is unsupported.

### 4. Stream R: how plausible is the floor, and would it kill the target?

**The conjecture needs a different foundation**

[Argued] I would not endorse "no O(1)-pass decoder beats the floor on random digits" as presently stated. In c_1, it needs a definition covering non-rigid algorithms. In cache-miss currency, a coefficient-one singleton floor conflicts with the LF² example under suitable cache parameters and high singleton density. That example establishes constant-pass behaviour, not automatically the exact proposed K = 3 contract.

[Conjectured] A defensible question is whether random-field inputs impose a strong tradeoff among preprocessing, cache capacity, dependent misses, total traffic and passes, relative to the best admissible baseline. That is substantially more precise than declaring each random singleton irreducible.

**What lower-bound methods can actually provide**

[Proved] m independent uniform hex digits contain 4m bits of entropy. An information argument with w-bit probes yields a scan-scale information requirement, not one dependent probe per digit. If the input has already been scanned into accessible memory, its entropy has already arrived. The remaining difficulty is organising and accessing it.

[Proved] The induced BWT permutation of a constant-alphabet text has only O(n) bits of entropy; a uniform arbitrary permutation has Θ(n log n). Therefore arbitrary-permutation or pointer-chasing bounds require a reduction that preserves valid BWT inputs, alphabet, access model and cost.

[Proved — literature] Sinha and Weinstein give a directly relevant BWT-specific starting point. Their symmetric positional-query lower bound requires n + Ω(n/(tq)) bits when forward and inverse queries take t, q probes in the stated range. Their entropy-polarization argument identifies a relatively small set of coordinates carrying most of the permutation's information. This is a succinct data-structure theorem, not a whole-decoding dependent-miss theorem.

[Argued] That distinction matters sharply here: on random-like inputs with r = Θ(n), O(r) words permits Θ(n log n) bits, far beyond a near-n-bit succinct representation. The additional preprocessing and whole-decoding arguments remain research obligations.

[Proved — literature] Ferragina, Gagie and Manzini show that single-disk sequential BWT computation or inversion cannot have memory-in-bits times passes equal to o(n). This does not obstruct constant passes with linear-word RAM. Their paper explicitly notes that sufficient internal memory permits reading the input once and computing there.

[Argued] Communication or external-memory arguments are still promising, but their restrictions must survive translation to this problem. Jacob, Lieber and Sitchinava's list-ranking bounds illustrate how much depends on atomicity and permitted operations. Such a model cannot silently forbid packing or global arithmetic and then claim to cover those algorithms.

**The 75–85% premise is not established**

[Proved — reading of the pinned record] That percentage is the estimated share of hex-character singleton runs. It is not an estimate of independent random content. The proxy includes repeated SHA1 references, ordinary numbers and Q-id digits; its reported precision for GUID membership is only 0.31–0.73. Record 96's actual GUID singleton shares, including all GUID characters, are 0.666, 0.615, 0.574, 0.269, 0.597, 0.487. The fully random-nibble subset is smaller. (Record 96, §§2–3; Record 99, §4.)

[Argued] Consequently, a theorem about independent GUID nibbles cannot be applied to the entire hex proxy. Doing so would classify some of the best repeated-string opportunities as proved hard.

[Proved] Model I also needs repairs before supporting a lower bound:
- Agreement with neighbour-equality and singleton frequencies does not establish joint independence or independence from suffix sorting.
- The key-collision calculation gives expectations or probability bounds under its model, not deterministic limits for every real block.
- For an iid column with p = 1/16, a run sampled at its start has a geometric length distribution. The run containing a uniformly sampled row instead has Pr(W = k) = k(1−p)² p^{k−1}. Record 96 mixes these sampling conventions.

[Argued] The exact singleton census remains useful. The probabilistic conclusions must distinguish iid original texts, iid last-column labels, random fields embedded in templates, and the six fixed measured blocks.

**The conditional ceiling**

[Proved] Suppose a valid lower bound applies to the R part of the joint decoder, let q = h_{1R}/h_1, and let additional split/shared/glue cost be g·h_1. If R costs at least h_{1R}, even free N decoding gives δ ≤ 1 − q − g.

| Hypothetical genuinely hard share q | Maximum whole-h_1 saving, before overhead | Required a_R for δ = 1/2, even with free N and glue |
|---|---|---|
| 75% | 25% | ≤ 0.667 |
| 80% | 20% | ≤ 0.625 |
| 85% | 15% | ≤ 0.588 |

[Proved] Thus a true 75–85% coefficient-one hard share would kill δ = 1/2 unless R itself improved substantially. It would not kill every useful margin. Neither the required hard-share identification nor that lower bound is currently established.

[Argued] A further missing premise is a conditional or direct-sum theorem. Hardness of an isolated random block does not automatically imply hardness of its random fields inside a mixed block: the scaffold, boundaries and jointly computed state can supply useful information. Event attribution alone does not address this.

[Argued — honest reformulation] If an R floor is eventually established, retain a substantial residual challenge: replace the N work by a new parameter q_N, with acquisition and glue included, and demonstrate that q_N is much smaller than the displaced N cost. Continue to require a substantial whole-decoder improvement. Label this a revised residual target, not achievement of the original half-h_1 target.

[Proved] For utility, the relevant ceiling is the measured fraction f_N of baseline time that can actually disappear: maximum speedup ≤ 1/(1 − f_N). An event share cannot substitute for f_N.

### 5. Stream N: the mathematical frameworks most worth pursuing

[Argued] I would prioritise two concrete algorithmic frameworks and one speculative mathematical framework. None presently has a demonstrated small parameter on the expensive rows and a native acquisition procedure meeting the budget.

**5.1 Positional macro schemes and string attractors**

[Proved — literature] Kempa–Prezza show that BWT run boundaries induce an attractor of size r, and derive compact macro schemes and grammars from attractors. These are representations of text substrings and their occurrences, rather than rigid transport of contiguous LF-row intervals.

[Argued] The useful primitive would be an acquired positional equality T[a..a+ℓ) = T[b..b+ℓ), with source, destination and a legal evaluation schedule. One such copy could cover many expensive rows scattered in suffix order.

[Argued — parameter] Measure q_copy = copy/literal descriptors + unresolved anchors + cycle-breaking obligations, together with source-read traffic and scheduling cost. Total copied bytes alone is inadequate.

[Guessed] Repeated reference hashes and repeated substantive fields may make this parameter small on some expensive content. Generic JSON structure is less persuasive: the frontier reports that keys occupy many bytes but few thin rows. The prospective gain must be demonstrated on the rows whose work remains expensive. (Frontier, working-backwards lead.)

[Argued] The acquisition trap is exact: a run boundary's BWT row is known; its text coordinate is not. An attractor existence theorem does not supply that coordinate. Existing RLBWT-to-LZ77 conversion demonstrates constructibility, but the O(r)-workspace construction of Nishimoto–Tabei still has n-scale time.

[Argued — next discriminating step] Supply an acyclic positional copy plan on real blocks and measure its complete payoff, descriptor count and anchor count. If it wins, the new theorem to seek is cheap acquisition of that plan. Re-encoding the already observed cyclic, short Φ fragments in a grammar does not solve the obstruction.

**5.2 Compressed coordinate differences and RLBWT suffix sampling**

[Proved — literature] The r-index literature exploits repeated blocks in differential SA/ISA arrays and supports positional access in run-bounded space. Under the usual sentinel convention, adjacent rows inside a BWT run satisfy the differential relation DSA[LF(i)] = DSA[i], where DSA[i] = SA[i] − SA[i−1]. This suggests representing coordinate relations collectively.

[Argued — parameter] Measure the size g_{Δ,N} of the coordinate-difference representation needed for expensive N rows, the number s_N of absolute samples, and the cost of acquiring and evaluating both. The desired inequality is D_acquire + D_evaluate + D_place < D_baseline,N.

[Proved — literature] Kempa's Theorem 5.1 constructs SA/ISA support from the RLBWT with parameter τ, using O(n/τ + rτ) workspace and O((n/τ)[log(rτ) + log(n/τ)] + τ² r log(rτ)) construction time. These explicit terms expose the acquisition tradeoff; they do not establish O(r) workspace and a few scans for the desired use.

[Guessed] Repeated strings may produce a compact coordinate representation. The reported small Φ-overlap transition count is a reason to inspect it, not evidence that g_{Δ,N} + s_N ≪ h_{1N}. Since h_1 ≤ r, another generic O(r) descriptor bound is not sufficient.

[Argued — next discriminating step] Oracle-compress the required coordinate arrays, including seeds, then evaluate an executable schedule. Charge random output writes and any sorting needed to restore output order. Adapt sentinel-based results to the actual cyclic BWT and origPtr contract explicitly.

**5.3 Ordered return-word hierarchies: substitutions, S-adic systems and Bratteli–Vershik coordinates**

[Proved — literature] Durand's return-word characterisation connects primitive substitutive sequences with finiteness of their derived sequences. It provides a principled source of reusable hierarchical descriptions.

[Proved — conditional construction] Given an ordered substitution hierarchy and expansion lengths, a position can be computed by summing preceding-sibling lengths along its derivation path. Conversely, a target position can be found by descending cumulative lengths. This is a genuine positional mechanism.

[Argued — parameter] Its economics must include the total ordered directive description s, hierarchy depth d, exception description e, and actual evaluation visits. Small matrix dimension alone is irrelevant: incidence matrices retain counts while discarding order.

[Guessed] Reused templates with literal holes might admit small finite hierarchies. There is presently no evidence that s, e, and evaluation cost are small on the expensive portion of these blocks. Every finite text admits a trivial hierarchy, so existence alone has no value.

[Argued] Ostrowski coordinates are an excellent exact toy. Rauzy fractals are a more demanding transfer route: geometric results require substantial hypotheses, and almost-everywhere dynamical coding is not yet an exact finite decoder with cheap tile decisions and boundary handling. Berthé–Steiner–Thuswaldner's S-adic results make those hypotheses explicit.

[Argued — next discriminating step] Seek a finite ordered hierarchy with explicitly charged exceptions and collective coordinate evaluation. Replaying its successor map one row at a time would restore the original obligation.

**Frameworks I would not promote as standalone answers**

[Argued] Wavelet trees and run-length FM-index machinery can support acquisition; ordinary rank-based successor evaluation is still a walk. Conversely, the frontier's observed four-load implementation cost should not be treated as a theorem about all such structures: modern move structures support amortised constant-time consecutive LF queries in O(r) workspace, while remaining walks.

[Argued] Lyndon/Gessel–Reutenauer theory is less promising as the leading route under this input contract. Bijective BWT constructions concern a different transform, and a small number of Lyndon factors does not bound the internal complexity of those factors or supply their positions. Any use here needs a paid bridge from ordinary bzip2 BWT data.

### 6. What would a clean "no" look like, and is it publishable?

**The draft's "no" does not address the open case**

[Proved] Its quantified language is 𝓡 ∪ E1 ∪ E3 ∪ E2_local ∪ E2_O(1). That omits global E2. A lower bound for this union cannot establish that global E2 is empty. (Draft, §6.)

[Proved] There is a second logical issue. From PN ∧ PR ∧ Pglue ∧ ⋯ ⟹ Goal, disproving PN does not imply ¬Goal. It defeats that sufficient construction. To exclude every construction, the premises must be shown necessary or the quantified algorithm class must independently cover every admissible decoder.

**Three legitimate negative results**

[Argued] A calculus-level no would state: every decoder in a fully specified rigid derivation calculus, on valid inputs satisfying the orbit-coverage condition, incurs at least one charged singleton event per singleton run. That is a useful scoped theorem. It closes that calculus, not inversion as a whole.

[Argued] A computational no would fix a uniform algorithm class 𝔄, an infinite valid-input family, a cost model and explicit resource budgets, then prove, for example, ∀A ∈ 𝔄, ∃c ∈ 𝓕: D_𝓗(A,c) > (1−δ_D) D_𝓗(A_0,c), for algorithms satisfying the correctness and budget requirements. The class must admit global positional computation, packing and writable workspace to the extent the target admits them. The proof cannot exclude them by definition.

[Argued] A distributional or practical no would address the intended workload more directly: either prove an appropriate expected-cost bound for a specified random-field distribution, including scaffold information and preprocessing, or establish that even a precisely supplied capability has insufficient measured whole-path headroom. The latter is a scoped experimental obstruction, not a mathematical impossibility theorem.

**Publication value**

[Argued] A new, realistic lower bound for BWT inversion with charged preprocessing, valid-BWT structure and non-rigid positional computation would be a serious result. A matching upper/lower tradeoff, or a separation between rigid derivation cost and memory cost, could also be valuable.

[Argued] G1 alone is more likely a useful lemma or technical note than the main result of a substantial paper, unless its calculus captures a broad independently interesting algorithm class and the simulation theorems are substantive. A reproducible account combining scoped lower bounds, failed constructions and calibrated native costs could support a strong systems contribution.

[Argued] The publication claim should therefore match the outcome: "this class cannot remove these operations," "this capability cannot pay on this workload," or "this realistic computational model imposes this tradeoff." Failure of the present three-premise construction would not justify "no new inversion algorithm is possible."

## B. The message for the agents (as relayed)

FROM: louis via chatgpt
TO: monodromy, neon
TYPE: PROPOSAL

Revise the specification before formalising the current main theorem.

Keep c₁ as a rigid-calculus diagnostic. Make the practical target a fixed reduction in total dependent cache misses against a pinned baseline, with explicit limits on dependency depth, traffic, work, passes and peak bytes. Charge acquisition, classification, shared state, glue and output placement. Use the published Kärkkäinen–Kempa–Puglisi LF² analysis as a calibration counterexample: singleton compositions need not cause singleton misses.

Replace the finite corpus as theorem domain with a uniform structural family; keep the corpus for validation. Prohibit corpus-specific advice and charge recognition and preprocessing. Define what K counts, including working-array scans and sorting.

Repair Pmargin: include split/shared/glue cost and add actual space hypotheses. Trace additivity does not construct independently decodable streams. Prove row-preserving attribution before deriving streamwise floors. Do not require a new decoder to materialise row positions or adopt the N/R split.

Treat G1 as a scoped provenance theorem until the operational trace, primitive-fact charging, inverse direction, reuse and valid-column/orbit assumptions are settled. Require cost-preserving simulations for claimed class membership. E1's bulk uses remain unclosed; labels determining text is no cost lower bound; L-completeness excludes TC⁰ only conditional on separation.

Do not identify σ_hex with randomness. Its 75–85% includes repeated hashes and numbers. Establish a valid-BWT random-field model and a conditional lower bound allowing the nonrandom scaffold and charged preprocessing. Read Sinha–Weinstein's entropy-polarization argument; its succinct-query theorem does not establish PR.

If a true hard share q has coefficient-one cost, δ≤1−q before overhead. Keep a residual structural challenge explicit rather than silently relaxing the whole-block target.

Screen three supplied capabilities: acyclic positional copy plans; compressed SA/ISA differences with paid anchors; ordered return-word hierarchies with paid exceptions. Measure descriptor/anchor/exception counts on expensive rows and full-path payoff before acquisition work. Audit Kempa's RLBWT constructions; do not rediscover failed Φ replay or ordinary rank-based walking.

Fix the "no": its quantified class must include global E2. Refuting one sufficient premise does not refute every decoder.

## Coordinator's notes for the swarm

- **Treat this as a peer's claims, not orders.** Verify each against the records before adopting it. Rebut what you can show wrong, with evidence, in CLAIMS.md or a REVIEW.
- **It overlaps Neon's cycle 1:**
  - its Pmargin/Pglue finding matches Neon's break 2;
  - its arbitrary-Column counterexample (L = abcdefgh) matches Neon's break 1 (L = aab).
  - Neon's N-2 measured that 85–89% of h₁ rows are unique at window 8, and that repeated non-GUID width-1 rows are only 10.7–15.2% of h₁. That bears directly on its "σ_hex isn't randomness" point. Novelty, not hex, is the right split, and on the six blocks the novel share is still large.
- **Changing the target's currency** (dependent misses alongside or instead of c₁) is a change to the frozen §1. Under C2 and Louis's rule, add the stricter conjunct; never drop a frozen clause. Record the decision, with its reason, before any data bears on it.
