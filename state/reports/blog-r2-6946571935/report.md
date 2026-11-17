# Round 2: Modelling the cache to rightsize workers

**REVISE.** The measured worker-plan gains stand. Round 1's most serious explanations are substantially repaired, but the article still combines incompatible model versions and overstates what has shipped. Correct the factual issues first, then shorten the main argument; this is an in-flight editorial review, not a withdrawal of the published post or an implementation/adoption gate.

Inbox [6946571935](https://github.com/lmmx/mibz2/pull/162#issuecomment-6946571935), dispatching QUEUED6946459981. Article reviewed at **55b26bd746fb346743c41cb70665ea719a37f437**. Independent records45–51 read at **c7ea4a5c5e8447fbdfd407c6f0ae07b6669bd87e**, before reading the revision. The pre-reading baseline is [baseline.md](baseline.md), checkpointed separately. I remember round 1; this is independence from the revised narrative, not a claim of an empty prior.

The [live page](https://lmmx.github.io/mibz2/blog/2026-11-17-modelling-the-cache-to-rightsize-workers/) returned HTTP200 and its browser text contains the reviewed revision. I inspected the desktop hero, model section/figures and cache slider. All ten slider states changed and supplied narration. This was not a complete publish-page rerun: phone/dark layouts and every figure were not visually certified. Source equations, SVG labels and captions were read throughout. The fresh-reader report is separate.

## The baseline and where it lands

Simplest correct explanation: on this i9, reducing workers sacrifices parallel processing capacity but reduces cycles per decoded byte by more. The measured whole plans therefore finish sooner. Cache interference is supported; its precise causal cost, a transferable worker selector, and how much overhead another intervention can recover remain unsettled.

| Baseline takeaway | First location in revised article | Assessment |
|---|---|---|
| Fewer workers can finish sooner; scope is i9 and measured plans | Opening paragraph; first graph | Lands early. Define the historic baseline rather than “today's” ten/nine workers. |
| Measured policy differs from proposed model | Opening paragraph2; takeaway5 | Clear in body, contradicted by dek and old baseline/status wording. |
| Cheaper bytes outweigh lost parallelism | Takeaway2; §2 bookkeeping | Sound numerical explanation; distinguish the identity from causal evidence, as revision now does. |
| Fit, discrete choice and conservative policy differ | Takeaway3; §3 cost curve | Strong repair in prose, undermined by the next figure's label and alt text. |
| Low-budget plan includes kernel and global window | Global window in §1; kernel contrast only §4 | Window fixed. Move lazy5/eager6 and eager4 exception beside first low-count takeaway. |
| Predictions can fail; transfer not established | Opening paragraph2; takeaway4 | Failure arrives early, but “three thread counts it hadn't seen” overstates novelty; eight-thread data already existed. |
| Working footprint differs from total buffers/residency | §1 definition and callout | Good repair. Subsequent exponent discussion must use the corrected proxy and report its failed homogeneity test. |

The first paragraph is about 260 words and the takeaway list repeats several result sequences. Put the three-sentence explanation above the detailed sweep numbers; retain one small outcome panel. The result is understandable from the top, but its scope currently needs the reader to reconcile separate claims.

## Required factual corrections

### R1. Present tense and baseline identity

The dek says the decoder sizes workers from a model. It does not. The opening explicitly calls that model a proposal; §6 says no general selector shipped. A scope paragraph cannot repair a false standalone title-card claim. Change the dek, description and copies together. A possible factual direction, not a tested final house-style draft:

> Fewer workers give mibz2 more speed on the i9 as a cache model begins to explain where extra parallelism stops paying.

The actual release is 0.5.3, carrying the scoped seven-lazy-worker/window28 policy at ten or more requested threads. [LOG6946439743](https://github.com/lmmx/mibz2/pull/145#issuecomment-6946439743) establishes this; the live header also displays v0.5.3. Replace “goes out in the next release.” Replace opening “today's mibz2 runs 10 workers” and “already runs … nine” with **the reference plan used in these experiments**. This preserves the denominators after release.

During this review, [LOG6946575966](https://github.com/lmmx/mibz2/pull/145#issuecomment-6946575966) records admission of lazy6/24 at eight and lazy5/20 at six following [REVIEW6946555118](https://github.com/lmmx/mibz2/pull/145#issuecomment-6946555118). The plan PR was being built, not thereby proved shipped. Update “await review” to the actual admission/implementation state in one dated status paragraph. Do not invent a subsequent release result.

### R2. One model version, including SMT and integer selection

The new cost curve correctly shows that eight beats seven under the quadratic fit. But the next figure still says **“cache binds: 7 workers”**, and its accessible description says **“the model gives 7.”** Its eight-thread counterpart similarly turns6.6 into6. A footnote about flooring does not undo those main labels. Label continuous bounds7.9/6.6 and measured choices7/6 separately. If presenting the current selector, show the fitted within1%-choose-smaller policy explicitly, with its calibration status.

§3's SMT explanation uses effective-core yield, while §5 still defines a domain limit capped at physical cores C_d. On a single domain with C=2,N=4 and abundant L3, that printed sum is at most2. It cannot predict four workers. A yield of1.24–1.38 per core does not make a physical cap disappear. Record49's correction replaces both outer and domain physical caps with logical N,N_d and puts SMT effects in per-worker cost. Use that corrected version consistently, or clearly label the older formulas as historical and supply the actual transfer procedure.

Do not then claim the current composed rule reproduces every low-count result. Record50 calculates three at N=4 under its own ν≈.16W assumptions; measured eager4 remains the safe policy. Separate the earlier frozen record46 count predictions from the later heuristic and its exception.

### R3. Normalised observation is not fitted c(10)

The units repair is welcome, but “so that c(10)=1” is false for the printed fit: .422+.0068·100=**1.102**. Normalisation makes the observed ten-worker reference one; it does not force an unconstrained regression through that observation. Write that distinction. Record46 explicitly identifies this quadratic residual and proposes c(W)=a+bW·W_run, obtaining roughly.983 at ten. Show the quadratic as the simpler illustrative fit, not the complete calibrated cost model.

The arithmetic otherwise checks: t7=.107885714; t8=.10715; t8/t7=.993180614; optimum7.877742; overhead share at seven44.1208%; conditional ratio1.78957. General p gives share1/p at the continuous optimum. [arithmetic.json](arithmetic.json) records recalculation.

### R4. Update the actual failed tests, without rewriting history

The article calls the counters run pending. Record51 has scored it:
- occupancy-derived L3-hit latency91.0→65.2cycles: direction agrees with Q, magnitude−28% is outside its10–25% prediction;
- L3-miss latency594→552: about−7%, not D's≥25%;
- L3-miss stall reduction2.36/5.52≈43%, but L2-miss stall reduction2.56/5.52≈46% misses Q's≥70%;
- retired hit/miss ratios.908/.390 miss their predictions;
- total IMC traffic was **not measured**. Demand L3-miss reads1.8GB/s are not total DRAM bandwidth. D is not formally excluded by its conjunctive refuter.

Replace the pending table with frozen prediction/outcome/status. Do not write “the counters decide” as though this run identifies a unique mechanism.

Record49 also scores the sharper footprint-homogeneity test: W=3's predicted misses.00225 fall below measured.0034 [.0027,.0042]; the simple equal-occupancy transfer fails. Record50's corrected hot-footprint ratio is26046/69048=.377216, not the total-buffer ratio5.97/16.81=.355146. With the four quoted slope ratios, k recomputes to2.03,2.23,1.85,2.05. A rough local near-quadratic association remains; **“the contention term's scaling … held”** is too broad, especially in the closing chain.

Preserve historical frozen scores separately. Eight-thread six-worker superiority was already observed in #515 before record46; the low sweep is fresh data, not three previously unseen thread budgets. Say it confirmed the earlier eight-thread result and tested lower budgets/new arms. Record49's quantitative misses for record44 also prevent “here they all are” from describing an exhaustive prediction ledger. Either include them in notes or call the main table selected tests.

### R5. The assumed miss-cost comparison still is not an upper bound

Calling589 an across-arm slope is a real improvement. The next sentence still says fewer misses explain **“at most”** two-thirds to four-fifths using an assumed390–490cycle DRAM latency. An assumption about uncontended latency is not an upper bound on loaded cost. Record51 now derives roughly595cycles, already above the assumed range. Different miss populations, overlap and other changed work defeat that partition.

Keep this only as the conditional calculation: **if** each removed miss carried390–490 unhidden cycles **and** other costs remained fixed, its direct contribution would be66–83%. Those assumptions are unestablished; the remainder cannot be assigned by subtraction.

Do not replace it with record51's unqualified “at most half.” Dividing mean latency595 by mean outstanding2.2 is not a proof of critical-path exposed latency. A counter-associated43% stall reduction is not a causal upper bound on recoverable wall time. This challenges the source, not just the article. Qureshi/Patt explicitly give a case where large miss reductions do not substantially reduce CPI because MLP changes; Yasin explicitly warns about overlapping stalls and workload-dependent penalties.

### R6. Smaller precision fixes

- Name the two different “simple rules”: the conservative cache-budget policy and the registered rounded/floored 0.7-of-threads candidates. Their status differs; a shared label obscures which result falsifies which rule.
- “One footprint … nothing competes” in slider state1 ignores scanner, reader, OS and unrelated workloads. Say no other decoding worker competes.
- Independent decoded blocks do not prove no coherence/false-sharing cost: queue, handoff and allocator/control state remain shared. The evidence does not identify coherence as the explanation; it also does not rule it out categorically.
- Reuse distance/MRC discussion is useful prior art. Qualify the cache/replacement model: one trace's stack curve is not an exact miss curve of arbitrary hardware, replacement and concurrent scheduling.
- Opening “each change … changes how much memory” is too absolute. Algorithm changes **can** alter footprint or CPU demand; some affect neither.

## Fact ledger and evidence limits

| Claim family | Primary check | Disposition |
|---|---|---|
| #51510/8-thread walls, counter ratios,95% bars, fixed clock | LOG6943194948 | Values match; .885 means11.5% less time and12.994% higher throughput. Six-at-ten is context, not judged. |
| Past-core sweeps and direct confirmation | LOG6931719814,6942020517,6944271342 | Values match. Direct reference9/36 must be named; held-out no-SLOWER is not no-harm proof. |
| W12 at7 null, instructions+1.6%,miss ratio.926 | LOG6944610173 | Intervals and no-detected-change wording match. Cause not identified. |
| Low-count plans and controls | LOG6945639925; prior independent archive replay REVIEW6946555118 | Counts/windows and comparison classes match. Log rounds six-thread ratio.963; exact replay.962492 rounds.962 at three decimals. Do not manufacture a discrepancy in significance. |
| -3walls,.355buffer ratio, frozen failed optimum | LOG6945320222; records45,46,48 | Values match; invalid/unpinned trace caveat now present. Hot-set exponent/scaling conclusion superseded by49–50. |
| a,b,p,busy shares,cache/CPU equations | records45–46, corrected49–50; arithmetic.json | Formula identities and numbers checked; R2/R3 distinguish model generations and observation normalisation. |
| c(W) causal interpretation/miss-cost predictions | records47,51 | Conditional model; R4/R5. New counter results read from record51, not independently replayed raw archive this turn. |
| Pipeline ownership/global window/cache-file path | parallel.rs at33e7225ba6147fb480168dc54fe30727f0b93aea, opening andl3_bytes | Matches source. Four-times-worker slots form one global window, not four concurrently decoded blocks per worker. |
| CI topology/SMT values and future refuters | records45–46 | Reported runner configuration; no new CI transfer result independently established here. Physical-domain cap contradicts proposed SMT use. |
| Release/current admission | LOG6946439743 and6946575966 | Scoped7/28 released0.5.3; low counts admitted, implementation then in progress. |
| Prior art | Mattson1970 scan; Qureshi/Patt2006; Xu etal2010 CAMP; Gunther2008; Yasin2014 | Relevant comparison, not validation of mibz2 coefficients. CAMP uses reuse distances, access frequencies and throughput/miss relationships. USL shape analogy must not assert coherence as this mechanism. |

Primary comments and files were opened. This is an article/source audit, not a new replay of every historical benchmark archive. The prior low-plan replay is explicitly separate. Hardware details are the experiment records' reported configuration; no new hardware measurement was performed.

Prior-art links read: [Mattson scan](https://www.bitsavers.org/pdf/ibm/IBM_Systems_Journal/092/ibmsj0902B.pdf), [Qureshi/Patt](https://users.ece.northwestern.edu/~rjoseph/eecs453/papers/quereshi-micro2006.pdf), [CAMP](https://web.eecs.umich.edu/~zmao/Papers/xu10mar.pdf), [Gunther](https://arxiv.org/abs/0808.1431), [Yasin paper](https://github.com/mrdrivingduck/paper-outline/files/14018926/A_Top-Down_method_for_performance_analysis_and_counters_architecture.pdf).

## Open questions: length and dek

**Yes, move detailed §3 hypotheses and §4 scoring to a -notes appendix.** At about6,300 reported words/24minutes, the article exceeds its7–15minute target. Aim at roughly2,500–3,500words in the main text, not a mechanical cutoff. Retain the core cost-vs-worker figure, one CPU-budget example, one compact frozen-success/failure panel including the failed-3 prediction, the global-window/kernel distinction and current scope. Move the general-p derivation, sensitivity table, full Q/D scoring, all low-arm ratios and topology algebra to notes. Keep uncertainty in the main text; moving it out would make the short version less true.

A useful main chain is: measured fewer-worker gain → lost parallelism versus cheaper bytes → proposed cache/CPU heuristic → failed small-block prediction → scoped release and transfer requirement. The opening and outcome panel should carry the low-count kernel exception without explaining every arm.

**No, the opening's scope statement is not enough to repair the dek.** Cards and search previews circulate without the paragraph. Make the finding true in the dek itself; keep changing release/admission status in one dated body paragraph.

## Reader answers graded against the baseline

The fetched PR #527 body lists takeaways and fact checks but contains no explicit goal-question set or held-out question. That requested provenance is missing. We reconstructed six questions from the takeaways and supplied two transfer probes; they must not be represented as preregistered or genuinely held out. The fresh reader received only the article, profile and these questions, no baseline or findings.

| Goal | Required answer | Parent assessment of reader answer |
|---|---|---|
| Motivation | A model aims to transfer beyond one measured sweep; validation remains necessary | Pass: all three distinctions recovered. |
| Mechanism | Lost parallelism, cheaper bytes, accounting identity distinct from causal evidence | Pass: recovered numbers and limitations. |
| Integer choice | Continuous bound, fit optimum, policy convention, measured winner | Partial: four objects recognised, but article cannot supply one consistent selector. Reader independently found c(10) inconsistency and figure conflict. |
| Transfer | Affinity, domains, placement, logical CPUs, SMT cost | Partial: domain/placement principle recovered; numerical two-core/four-logical score unavailable. Formula still caps at two. |
| User status | Scoped release versus proposed model; admitted versus shipped low counts | Article-comprehension pass, factual failure: reader faithfully repeated the stale next-release and awaiting-review claims. This is direct evidence of their effect. |
| Failed tests | Preserve frozen outcomes and label revised serial limit as post-data | Partial: the reader recovered the -3 failure and controls, but source omits records49–51 outcomes. |
| Null W12 probe | No unique cause follows from no detected net gain | Pass. |
| 44% probe | Fitted cost share is not measured recoverable wall time | Pass. |

Stuck levels are concrete: symbol (N versus requested threads; c units); statement (shipped/proposed); step (7.9→7); chain (two simple rules, model generations and baseline kernels); beyond (SMT/domain transfer). Fragmentation comes from spreading these qualifications across sections; collapse from figure labels merging fit and choice. A missing step is how requested thread budget limits an affinity mask that may contain more CPUs: define effective N, rather than silently equating the two.

Dense-prose repairs: replace the five-number CPU-utilisation sequences with predicted/measured rows; add a small kernel/count/window baseline panel to the low-thread result; use the existing bound figure to separate continuous bound, fitted integer and measured choice; and move full hypothesis/refutation tables into notes. No pie chart of the44%: it would make an extrapolated attribution appear measured. Keep the kitchen analogy labelled conceptual; it explains competition, not the measured eviction or queueing law.

The fresh reader favours retaining a24-minute technical post with10–15% cuts. I disagree on final structure: even15% leaves about20minutes and retains two partially independent arguments (choosing counts and attributing every cycle). The stated audience and7–15minute target favour a main post plus notes. This is editorial judgement, not a claimed experimental result.

## Convergence and next pass

Closed or substantially improved: wall versus throughput percent; global window; extrapolated normalised coefficients' units; quadratic exponent as fit; integer7vs8 comparison; general1/p cost share; sensitivity rather than confidence interval; W12 null interpretation; specific flat-miss rival rather than all ring mechanisms; held-out uncertainty; flagged-3trace; kernel controls.

Still open: SMT transfer arithmetic, caption/alt-text agreement, source-version reconciliation, and causal “at most” attribution. New factual correction: fittedc(10) is1.102. The detailed simulated-reader answers and grading are in reader.md; these remaining chain/transfer failures justify another review after restructuring.

No article or decoder implementation was changed. No run, merge, subscription change or replacement automation was requested or performed. Research only on team-state/researcher; one REVIEW on #145.


## Pinned primary model records

- [Record 45: workers-machine-model](https://github.com/lmmx/mibz2/blob/c7ea4a5c5e8447fbdfd407c6f0ae07b6669bd87e/docs/journal/monodromy/2026-11-17-workers-machine-model.md)
- [Record 46: workers-cpu-budget](https://github.com/lmmx/mibz2/blob/c7ea4a5c5e8447fbdfd407c6f0ae07b6669bd87e/docs/journal/monodromy/2026-11-17-workers-cpu-budget.md)
- [Record 47: contention-counters](https://github.com/lmmx/mibz2/blob/c7ea4a5c5e8447fbdfd407c6f0ae07b6669bd87e/docs/journal/monodromy/2026-11-17-contention-counters.md)
- [Record 48: level3-scoring](https://github.com/lmmx/mibz2/blob/c7ea4a5c5e8447fbdfd407c6f0ae07b6669bd87e/docs/journal/monodromy/2026-11-17-level3-scoring.md)
- [Record 49: worker-rule](https://github.com/lmmx/mibz2/blob/c7ea4a5c5e8447fbdfd407c6f0ae07b6669bd87e/docs/journal/monodromy/2026-11-17-worker-rule.md)
- [Record 50: rule-scope](https://github.com/lmmx/mibz2/blob/c7ea4a5c5e8447fbdfd407c6f0ae07b6669bd87e/docs/journal/monodromy/2026-11-17-rule-scope.md)
- [Record 51: miss-cost-scoring](https://github.com/lmmx/mibz2/blob/c7ea4a5c5e8447fbdfd407c6f0ae07b6669bd87e/docs/journal/monodromy/2026-11-17-miss-cost-scoring.md)
