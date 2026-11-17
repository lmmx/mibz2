# Round 3: Modelling the cache to rightsize workers

**REVISE, targeted.** Assignment [6953475922](https://github.com/lmmx/mibz2/pull/162#issuecomment-6953475922); article and notes at **745a7381ab72ca07675ddfcb000c4156f926c0eb**. This reviews the article's evidence and exposition, not production admission. The measured wins stand. No implementation, timing, merge or authority change.

The [baseline](baseline.md) was committed at **83a523b13e45340f7075eb9e29160b44344bc956 before fetching this revision**. It was reconstructed from the primary research records and new release/unpinned LOGs. This is independent of the revised narrative, not blind to the research or earlier reviews. The reader exercise below is my explicit simulation, not evidence from a new human or an independent agent.

## What is repaired

The source's dek and second paragraph now distinguish the shipped measured-machine policy from the model under test. The split gives the main argument a sensible path. The logical-CPU/domain caps, fitted smaller-within-1% tie-break, normalised cost, c(10)=1.102 residual, approximate footprint, failed small-block count, and conditional cycles-per-miss calculation address the previous mathematical objections. The main post and notes now preserve the failed sizes of counter predictions and the unmeasured total DRAM traffic. W12's null result is appropriately qualified. None needs another rewrite of the model exposition.

## F1. The unpinned experiment does not identify a unique cause

Locations: takeaway bullets 1, 3 and 5; section 5's second paragraph and final paragraph; notes section 3 after the ledger. The categorical claims that the low-count gains were not cache contention, that an unpinned worker removed only loses compute, and that the surviving ten-thread gain therefore is the cache effect exceed the experiment.

[LOG 6949008682](https://github.com/lmmx/mibz2/pull/145#issuecomment-6949008682) reports net plan ratios under different affinity/conditioning: 0.873 at ten threads, 1.000 at eight, 1.009 for eager five at six, and 1.214 for eager three at four. That establishes transport failure of the low-count pinned gains and strongly implicates competition with other decoder threads. It does not measure a zero cache contribution. Removing a worker may reduce cache cost while sacrificing enough parallelism to cancel it. For a concrete countermodel, a cache-cost factor 0.96 multiplied by a parallelism penalty 1.0416667 produces net 1.000. The same 0.96 with penalty 1.2645833 produces 1.214. These are logical counterexamples, not fitted explanations. Multiple decompositions fit the same observations.

Conversely, persistence at ten supports a shared-resource explanation alongside the counter evidence, but does not prove which cache, queueing, overlap or scheduling contribution caused the gain. The earlier thread-busy accounting was correctly labelled close to an identity; do not now replace it with a causal proof from a plan ratio. This objection applies to the wording in the Coordinator's LOG too: a source can contain the same unsupported inference.

**Replacement:** “Below ten threads, the gains measured with the process pinned did not carry to full affinity. Competition with the decoder's other threads is implicated, so those points cannot calibrate cache contention alone. At ten threads the gain survives full affinity; the counters support shared-resource interference, while its exact contribution remains unresolved.” Retain exclusion of confounded points from a cache-only fit. No new experiment is required to correct this prose.

## F2. The claimed complete prediction ledger omits a failed kernel prediction

Notes section 3 records the successful record-49 eager-five and eager-three predictions but omits its eager-seven prediction: **eager7/lazy7 at ten threads = 0.98 [0.95, 1.02]**. The observed ratio was **1.0906 [1.0821, 1.0992]**. This is outside the frozen interval, not merely an imprecise point prediction. Source: record 49 at c7ea4a5c; [independently replayed eager-trim report](https://github.com/lmmx/mibz2/blob/56034daf7439de030f8957e3bfbc8a4abce9f4f9/state/reports/eager-trim-6947285652/report.md), [REVIEW 6947387421](https://github.com/lmmx/mibz2/pull/145#issuecomment-6947387421). The current session also inspected that replay's retained calculations. Add the failed row and say that the kernel model had mixed results. Otherwise change the assertion “every frozen prediction” to an explicitly selected ledger; retaining this particular failure is still necessary to give the kernel model a fair account.

There is a second transcription error: “4 of 11 intervals missed” inherits record 49's incorrect heading. Its table contains **five** outside-band values: 8-thread 5/8, 8-thread 5/6, 6-thread lazy5/lazy6, 4-thread lazy2/lazy4, and 4-thread lazy2/production. Count the rows, not the heading. Correct to five unless a separately documented exclusion changes the denominator. Preserve the original record with a correction beside it, per B4; do not rewrite a frozen result.

Eager-three's 0.959 is inside [0.95,1.03]; its successful ledger status is right. The low-count performance choices need not be withdrawn because the kernel model fails at seven.

## F3. “From 10 threads in any mode” is broader than the shipped predicate

Takeaway 5 is false for arbitrary affinity masks. At release d27b6e7f, `affinity_admits` is exactly `pinned_one_a_core(threads,m) || (whole_machine(m) && unpinned_trims(threads))`. The first requires exactly T distinct physical cores and T logical CPUs; the second requires the complete machine affinity and T >= 10. A ten-logical-CPU mask spanning only five SMT cores satisfies neither. The source explicitly excludes other masks, unreadable topology and quota cases. “4 to 8” also reads as including five and seven, but the low trim cells are exactly **4, 6 and 8**.

Use “at ten threads in the measured pinned and full-affinity modes, and above ten with full affinity; at four, six and eight only with the measured pinning.” Section 5's detailed pinned description is already good. Keep the measured-machine and no-general-selector qualification. The observed 0.873 is specifically the ten-thread full-affinity comparison, not a result measured at every count above ten. No need to enumerate every feature check in the opening, but do not claim every mode.

## F4. The live article served a different revision

The source was reviewed at 745a7381. Two HTTP fetches of the public article, including a cache-busting query and Cache-Control:no-cache, served the older 24-minute article with the old model-as-current-policy dek; the new notes URL returned HTTP 404. The second response is dated **2026-11-17 16:08:11 UTC**, body SHA-256 **afd14f90e40a970191661e6f9b3942219373fc843ec725a3042e43ce14ea6761**, 151270 bytes. See [live-check.json](live-check.json). The new dek and 0.5.4 were absent. This establishes what the endpoint served to this client at that time, not the cause of deployment/cache behaviour. Verify the corrected source actually reaches the public article and notes before declaring the live round complete. No visual/interactive-browser certification is claimed.

## F5. Two small source corrections

The opening's “timing every worker count at each thread count” overstates the sparse sweep. The calibration record lists selected arms (e.g. only eight workers in the first past-core run, then seven/six); write “a sweep of candidate worker counts at selected thread counts.” “Up to 11.5%” describes the initial pinned sweep, whereas the later full-affinity result is 12.7%; label the initial result rather than imply a current maximum.

The notes' MLP sentence cites Qureshi and Patt's *Utility-Based Cache Partitioning* (MICRO 2006). That is relevant to the earlier cache-utility discussion. For the specific failure of raw miss counts to predict stalls under changed MLP, cite **Qureshi, Lynch, Mutlu and Patt, A Case for MLP-Aware Cache Replacement (ISCA 2006), section 1.1** instead: [author-hosted primary paper](https://users.ece.cmu.edu/~omutlu/pub/qureshi_isca06.pdf). Its counterexample directly separates miss count from stall cost. No claim of novelty for this general principle is warranted.

## Takeaway placement and simulated reader

| Intended takeaway | First landing and reinforcement | Reader outcome |
|---|---|---|
| Measured workers fewer than requested threads can win | Dek and opening; first chart; takeaway 1 | Immediate and quantitative; qualify initial sweep and affinity |
| Give up parallelism to reduce per-byte shared-resource cost | “Simplest correct picture”; section 1 analogy; section 2 counters and bookkeeping | Mechanism accessible; reader may overlearn a proved cache-only cause because section 5 is categorical |
| Measured policy shipped; general model has not | Paragraph 2; takeaway 5; release box | Clear in the revised source; false “any mode” needs F3 |
| Count rule is a fitted heuristic with a calibrated tie-break | Takeaway 3; section 3 worked examples and curve | Correct: eight minimises the simple curve; seven is calibrated choice |
| Tests can refute the model | Takeaway 4; section 4 small-block test; notes ledger | Clear for footprint/count, incomplete for kernel choice without F2 |
| Transfer requires topology, affinity and new evidence | Paragraph 2; section 5; notes section 5 | Goal visible; exact runner falsifiers are available only after following notes |

**Simulated first-pass retelling:** “The decoder makes fewer workers on its tested desktop because sharing hardware can make each worker slower. Measurements pick the current plans. A proposed model combines cache, CPU and serial-stage bounds. It needed a fitted tie-break and failed the smaller-block prediction. The experiment without pinning changed which low-thread plans helped.” This is a successful narrative. The source then tempts the reader to add “that experiment proved exactly which gains were cache and which CPU”; F1 prevents that incorrect extension.

**Held-out goal question**, fixed in the baseline before the article was fetched: “How do I pick workers on an unmeasured machine, what is deployed today, and what evidence would justify replacing the measured profile?”

Article-only answer: “Today there is a scoped measured-machine policy, not a universal model. The proposed count reads usable CPUs, LLC domains and block footprint, applies CPU/cache/serial bounds and a fitted near-tie rule; new-machine tests are still pending. I should not treat its number as a validated optimum.” Pass on goal and status. Notes add the four-logical-CPU runner's three frozen refuters: three beating four by >2%, two within 3% of four, or a lazy plan beating the reference by >2%. These are prospective tests, not evidence already obtained. Recommend one sentence in the main transfer paragraph linking explicitly to this acceptance/refutation test. No extra derivation is needed there.

Second baseline probe: “Do disappearing low-thread gains prove a cache law?” A careful baseline answer is no. The current source steers the simulated reader towards a causal exclusion it cannot support: fail, F1. A reader should also distinguish selecting a count from selecting the kernel/window; F2 supplies the missing counterexample.

This is a structured self-simulation, not an empirical comprehension trial. Existing topic knowledge limits independence; no fresh-reader outcome is fabricated.

## Decision and next value

Expected from round 2: arithmetic, deployment status and causal qualifications repaired. Achieved: those mathematical/status changes substantially land; the new affinity evidence creates a fresh causal overclaim, the ledger omits an important failure, and the public endpoint did not match the revision. The gap is now editorial and publication verification, not another benchmark. Correct F1–F5, retain the selected dek, recheck the two live URLs, then a focused closure review can suffice. The actual decoder measurements are not invalidated. Further mechanism experiments remain the registered research programme's decision; no new run is requested by this review.

See [source-audit.md](source-audit.md) for claim-family provenance, arithmetic and verification limits.
