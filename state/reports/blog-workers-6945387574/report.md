# Round 1: Modelling the cache to rightsize workers

Trigger: [6945387574](https://github.com/lmmx/mibz2/pull/162#issuecomment-6945387574). Researcher spec pin: `a2da49638ac54d7763de1c167bd054f67ffdc763`. Reviewed article source: [3c957d936dcdcd4a691925256491d8f4992a5187](https://github.com/lmmx/mibz2/blob/3c957d936dcdcd4a691925256491d8f4992a5187/docs/site/blog/2026-11-17-modelling-the-cache-to-rightsize-workers.md), merged PR527. Research only; no article, implementation, acceptance rule or frozen verdict changed.

**REVISE.** The measured result is useful and the opening gives readers its practical significance. The article overstates what the fitted model explains, rounds a continuous optimum incorrectly, drops SMT from its transfer formula, and turns counter-based conjectures into causal findings. The requested 44% / 1.8× / 34–60% arithmetic is correct, with important units and assumptions missing. Keep the measured gains; repair the explanation.

**Access limitation:** the supplied live directory URL returned HTTP404; the `.html` alternative also returned HTTP404. This is a source-text and SVG-content review, not a verified live rendering or interaction test. No conclusion about visual clipping, responsive layout, JavaScript execution or deployed/source equality is claimed. The Coordinator should verify the published route and render in the follow-up.

## Independent baseline, then comparison

`baseline.txt` was recorded after reading the primary records and before opening the article. In plain language: independent blocks still compete for shared cache and CPU service. On the measured ten-core i9, seven lazy workers with 28 global slots beat the previous plans at ten requested threads and past the cores; six/24 also won at eight, with admission deferred. A fitted model helps choose the next experiment. It has not established a transferable worker-selection law or identified all of the cost it calls contention.

| Baseline item | Where the article puts it | Required repair |
|---|---|---|
| Fewer workers can lower elapsed time and memory on this i9 | Opening, first figure, first takeaway | Keep; change “11.5% faster” to “11.5% less time”; name the coupled worker/window plan |
| Worker count, requested threads, allowed CPUs, kernel and baseline all matter | Opening gives some; §1 gives roles; §3 introduces CPU budget | Add a small terminology key before the first figure; distinguish global slots from per-worker ownership |
| Held-out tests are loss screens, not universal equivalence | Opening says neither file decoded slower anywhere | Say no held-out cell was classified SLOWER; retain inconclusive results |
| Counters support cache-related interference, not a uniquely identified cause | Second takeaway and §2 assert the cause | State measured counter pattern first and separate candidate mechanisms |
| The CPU model is calibrated, not independently confirmed | Third takeaway, §3 | Make its missed 8-thread magnitude and unvalidated transfer visible in the takeaways |
| W12's old benefit cannot be added to the seven-worker plan | Late, dense paragraph in §3 | Add an early takeaway: smaller rules reduced misses without a detected time gain |
| 44% and 1.8× are extrapolated model costs | Late §3 | Introduce as a model diagnostic, never recoverable measured wall time |
| Smaller blocks reduced the gain but did not move the best tested count to 9/10 | §4 still pending | Requested follow-up: prediction beside result; update opening/model claims too |
| Domain placement and other pipeline limits constrain transfer | §§5–6 | Include one early sentence saying other machines and block sizes remain tests, not supported automatic selection |

The opening is not missing the result; it is missing the boundary between the result and the proposed explanation. Its long first paragraph moves from a measured win to “update inputs instead of re-running the sweep” without establishing that transfer. Split at that transition. A suitable central claim is: **On the tested i9, seven workers use less time and memory; modelling the shared resources gives testable explanations, but smaller blocks already expose a missing limit.** This is a suggested direction, not an article edit.

## Findings that change the argument

### 1. A stationary point is not a hard capacity boundary

The §3 caption says to take “the whole number below the binding limit”; the text then obtains seven from 7.9. For the very objective it prints,

`t(W) ∝ .422/W + .0068 W`,

the continuous optimum is 7.877742. But `t(7)=.107885714`, `t(8)=.10715`, so **eight is 0.682% cheaper than seven in this model**. Both fit below the stated 8.2-worker CPU budget at ten threads. Rounding down is not minimisation. The measured preference for seven is real; it is evidence of model residual, not proof of the rounding step.

Check adjacent integer candidates under the full objective. Keep the separate frozen candidate rule `floor(.7 L3/F)` distinct: its conservative coefficient produces seven, but that is a different rule. Record46 §5 itself says to check floor and ceiling of the CPU term. Do not silently repair the old frozen model or rewrite its scores.

The quadratic's “half at the optimum” is also model-specific. For `c=a+bW^p`, stationarity gives `a=(p−1)bW^p`, and the excess fraction is `1/p`. At p=1.5 it is two thirds; at p=2.5 it is 40%. The robust-looking location of the fitted peak does not make a half-cost decomposition robust.

### 2. The model's units and SMT case are missing

Record45's a=.422 and b=.0068 are in **normalised cost units**, relative to production's ten-worker cycles per output byte; .422 is not .422 literal cycles/byte. The post calls them cycles/byte without declaring the normalisation. The same counters elsewhere report about 23 cycles/byte. It also calls a the cost of “a worker alone”: a is an extrapolated intercept, while the printed law at W=1 gives a+b. Neither has been directly measured here.

The headline rule `min(C,W_L3,W_CPU)` calls C physical cores. That caps the CI example at two workers, while §4 predicts four on two cores. Record45 includes an SMT yield function; record46 uses CPU equivalents. The post omits both while using twenty logical CPUs as about seventeen worker-equivalents past the cores. Restrict the simple formula to the non-SMT case, or show the yield conversion and explain why a fitted optimum is not a resource capacity limit. Do not conflate physical cores, logical allowed CPUs and equivalent active CPUs.

The explanation of the square also needs restraint: multiplying “W workers” by “W footprints” does not derive a quadratic **per-worker cost**. The exponent is fitted, with several exponents fitting similarly. Present it as a useful approximation; a mechanistic derivation requires additional assumptions.

### 3. Correct arithmetic, conditional interpretation

Recomputed in `arithmetic.py` / `arithmetic.json`, using record45's printed fit table:

| p | a | b | c(7) | Excess share at 7 | Cost ratio c(7)/a |
|---:|---:|---:|---:|---:|---:|
| 1.5 | .300 | .0248 | .759302 | 60.490% | 2.5310 |
| 1.75 | .370 | .0129 | .758608 | 51.226% | 2.0503 |
| 2 | .422 | .0068 | .755200 | 44.121% | 1.7896 |
| 2.5 | .494 | .0020 | .753284 | 34.420% | 1.5249 |

Thus 44%, 1.8× and 34–60% are faithful rounding. The range is **sensitivity to the fitted form**, not a confidence interval. The 1.8× becomes a whole-decoder speedup only if the fitted excess can actually be removed, worker service remains the bottleneck, clock and useful parallelism remain comparable, and the intervention introduces no offsetting cost. Those conditions are not observed. Other plausible fits give conditional cost ratios of 1.52–2.53.

“A large share ... is not a sign of waste” is too broad: at a throughput optimum, a marginal tradeoff balances; that does not prove the cost unavoidable under another algorithm. The low-worker sweep can constrain the intercept and reject forms; saying it “settles” the decomposition overpromises when pipeline supply, occupancy and block-size effects can also change.

### 4. Counter correlations do not prove the residual mechanism

The W12-at-seven registered outcome is no detected gain: wall ratios 1.012 [0.998,1.028], .996 [.987,1.005], 1.002 [.985,1.018]. The counters at twelve threads show LLC-load-miss ratio .926 and instruction ratio 1.016. This is consistent with an overlapped miss saving offset by extra work; it does not prove that residual contention is unrelated to footprint or misses, nor that L3-hit queuing caused the null. The Coordinator's LOG6944610173 makes the same stronger inference; it needs qualification at the source as well as in the article.

From record47's printed counts, `(22.98−17.32)/(.01964−.01002)=588.36` cycles per avoided miss; ~589 is sound. But 80–100 ns, converted to 390–490 cycles, is an assumed uncontended access cost, not a measured upper bound under contention. The slope also includes any change in the cost of remaining misses, overlap, hit latency, stores and other work. Therefore it does not bound the fraction caused by misses, nor uniquely allocate the remainder to L3 hits.

Likewise the 4–9 GB/s and 10–20%-of-peak figures in record47 are **predictions**, not measured DRAM utilisation. That record explicitly leaves hardware prefetch traffic unknown and provides a DRAM rival to test. “Main memory is not the bottleneck” jumps ahead of the test. Label the 4–8 extra cycles per L3 access as conjecture. L2 misses are a proxy for downstream accesses, not a direct count of retired L3-hit loads; record47 separately proposes measuring the latter.

The article later labels the paragraph a conjecture, but that does not repair its earlier categorical sentences. In §4, the flat-miss bandwidth rival was refuted; an entire class of bandwidth/interconnect mechanisms was not. Otherwise §4 appears to refute the very ring-queuing account §3 proposes.

### 5. The transfer result requires restructuring, not a pending-note append

[LOG6945320222](https://github.com/lmmx/mibz2/pull/145#issuecomment-6945320222) arrived after the article froze, as the assignment acknowledges. At -3, seven/28 gives .978 [.973,.983] against ten/40 and remains the best tested arm; nine gives .992 and eight .987. Buffers fall to 5.97/16.81=.3551; those are **total buffers**, not an independently measured hot set.

Keep the registered scores: nine/ten optimum refuted; “seven no better than .98” borderline because its interval includes .98; unchanged .885 bandwidth-rival gain refuted. Do not claim the later CPU budget retrospectively explains seven at both block sizes. [Record48](https://github.com/lmmx/mibz2/blob/be614017696167f2f84ed7dafee600afc1afa8b6/docs/journal/monodromy/2026-11-17-level3-scoring.md) explicitly scores frozen record46 as predicting nine and ~1.18 for seven/ten, wrong in direction.

The serial-supply limit is a plausible missing term, not yet a clean causal measurement: record48's occupancy trace is flagged with 63 ledger violations and is unpinned, unlike the timed arms. Its admitted plateau supports investigating supply; do not turn the 68%/28% trace into unqualified proof. Preserve both the failed prediction and any later revision separately. The low-thread sweep remains pending in this review; no result is invented or backfilled.

## Fact audit by passage

This groups repeated values and claims; it covers the substantive prose, equations and figure labels. It is not a new replay of the confirmation/W12/-3 raw archives. The prior three-run replay is separately linked below.

| Passage / fact | Source and check | Disposition |
|---|---|---|
| Ten-core i9, twenty hardware threads, 20 MiB L3; Wikidata ×4; ten paired rounds; lazy kernel | Three sweep LOGs and prior independently replayed REVIEW6943617731 | Supported within measured setup; “one worker a core” should describe the process affinity mask, not a guaranteed permanent worker binding |
| .885 at ten; .929 at eight; ~quarter less RSS | LOG6943194948 / prior review | Supported ratios; .885 means 11.5% less elapsed time or 13.0% more throughput, not exactly 11.5% faster |
| Direct seven/nine .921/.930/.923; confirmation .881/.933; RSS17–27% reduction | LOG6944271342 | Supported; make direct confirmation vs earlier chained context explicit |
| “Neither held-out file decoded slower anywhere” | LOG6944271342: none classified SLOWER, small-block text inconclusive | Overstated; no detected slowdown is not equivalence or every point ≤1 |
| Scanner, worker MTF/inverse BWT/RLE/CRC, ordered reader | Pinned parallel.rs lines1–22 | Faithful to this architecture. “Can't be split” should be scoped to this scanner, not a universal compression impossibility |
| Four blocks per worker, seven holds28 | parallel.rs SLOTS_PER_THREAD and global slots invariant | Arithmetic right; ownership wrong. Global in-flight window is four times workers; each worker finishes one block at a time |
| 256 KiB L2, ~2.1MB hot footprint, 20MiB shared L3, ~eight L2s | Calibration and records45–47 | Supported as reported machine/estimated hot set. Distinguish MiB/MB, hot set and allocated buffers; fits do not directly measure residency |
| Cache slider .7 policy / .8 fitted peak; overlap caveat | record45 plus calibration | Useful capacity illustration, not a miss predictor. .7 and .8 are different rules; do not silently round one into the other |
| Clock1.326, <2% instructions change, ~48% misses /25% cycles reduction | LOG6943194948 and counters table47 | Broad pattern supported. Rounded table47 implies ~49% misses and L2 +3.34% for10→7, so prefer “about3%” to “within3%”; preserve aggregation/rounding provenance |
| .763/.925 busy shares; 7.63/6.48 CPUs; .752/.849≈.886 | Prior REVIEW6943617731 and record46 | Arithmetic/bookkeeping sound. Worker utilisation and aggregate cycles are different counters; retain the explicit near-identity limitation |
| a,b,p, x≈.77–.84, seven from7.9 | record45 fit and independent arithmetic | Fits/peak range supported; units/intercept/rounding and mechanistic derivation need fixes above |
| .93 busy ceiling, ν≈1.08, ι≈1.25, CPU8.2/6.6 | record46 | Calibrated values supported. Scheduler cause is conjecture. CPU/wall normalisation still uses elapsed time; “only processor time” does not mean independent causal validation |
| At eight, model6/7 near-tie vs measured~5% difference | record46 | Correctly disclosed; connect this miss to the opening's causal confidence |
| W12 ratios, miss reduction7.4%, ~.98–.99 atnine | LOG6944610173 | Numbers supported; .926 is at twelve threads, not all contexts. Add intervals; no detected change, not no effect |
| 589,390–490,0.69,35×,4–8,4–9GB/s | record47 | Derived slope/rough count ratio supported; hard latency bound, exclusion of DRAM and assignment of residual cost not established |
| Frozen low-count predictions6/5/4; rival6/5/3; simple6/4/3 | record46 and LOG6943635048 | Copied correctly. Busy shares themselves require elapsed time; say no additional throughput experiment, not “without timing anything” |
| ≤2% penalty and within-one-worker criterion | calibration/model plus LOG6943635048 | Keep scopes distinct: candidate R's LOG scores ≤2% penalty; transfer/model criterion is separate and neither is adoption |
| -3 optimum9/10 and ≥.98 prediction; results pending | record45 vs newer LOG6945320222 | Historically frozen prediction; update with failed/borderline outcomes, not erase |
| Four-vCPU/two-core/32MiB free runners | record45's sampled AMD runner profiles and SMT table | Not a universal GitHub runner specification. Label specific profile; retain CPU/affinity/cache per run. Formula currently contradicts two physical cores |
| Per-L3 sum, one L3 per socket, soft placement | record46 and pinned l3_bytes source | Sum conditional on placement and workload; “one per socket” is an example, not universal topology; workers may still contend for shared memory/service resources across domains |
| l3_bytes reads cpu0/cache/index3/size | Opened pinned parallel.rs lines2831–2846 | Verified; proposed discovery by cache level/shared_cpu_list is future design, not current implementation |
| Startup detection, no mid-stream adaptation; limited admission | calibration / #524 | A proposed design and scoped production patch, not a generally validated model selector |
| #524 not merged | GitHub PR read-back during review | Now stale: merged 2026-11-17T10:31:18Z as 521ef934edb4c5f763aaf542ee70d043a1735dfc. Update status, but do not claim this merged patch implements the general model |

## Reader pass and figures

Reader profile: programmer familiar with caches and pointers, new to bzip2 internals and this research history. Goal questions: why fewer workers can help; which result is measured; how the model picks ten/eight-thread counts; what 44% permits; how to transfer; what changes now. Held-out questions: whether W12's null identifies a cause, and whether the formula selects seven rather than eight. A fresh subagent receives only article source and that profile/questions; its separate answers are in `reader.md`. Source-only access limits also apply there.

| Level | Stuck point | Repair |
|---|---|---|
| symbol | Thread count means requested workers in one place, allowed CPUs in another; c has no normalisation; “teams policy” appears without an explanation | Define requested T, actual W, allowed logical N and physical C once; state c's reference; link/briefly describe teams |
| statement | Present-tense dek promises automatic model sizing; later adoption is future and specific | Say fitted model/proposed transfer in opening; state the measured scoped production status separately |
| step | 7.9→7; quadratic per-worker cost; 589→proof of L3-hit cost | Show adjacent integer objective values; mark fitted exponent; separate observed slope from causal assumptions |
| chain | Null W12 result leads to exclusion of misses, then ring queue mechanism, while §4 says ring contention refuted | Name each distinct hypothesis and its falsifier; keep evidence and conjecture side by side |
| beyond | A two-core runner should use four workers after a physical-core minimum formula; two sockets assumed two caches | Give one SMT-aware worked case and a topology sketch with explicitly discovered cache domains |

Three figure changes earn their space. First, replace the dense “half the cycles” derivation with a cost-vs-worker plot: a/W falls, bW rises, their sum bottoms between 7 and 8; mark the measured seven-worker choice separately and annotate model sensitivity. Second, replace the long W12/miss-cost/ring paragraph with an evidence-to-hypothesis panel: observed LLC/instruction/wall changes on one side, Q/D predictions and missing measurements on the other. Do not draw conjectural causal arrows as facts. Third, give frozen prediction/result pairs for -3 and the pending low sweep; make the failed optimum and borderline .98 threshold visible. Existing pipeline/cache sketches and the capacity slider are useful; they need clearer status/ownership labels, not more decorative figures.

## Fresh-reader answers checked

The independent reader recovered the measured result, the cheaper-bytes-versus-lost-parallelism accounting, and the current-versus-proposed distinction as presented. Its ten/eight count answer follows the article's flooring procedure but independently rejects that procedure as a derivation. Its transfer answer identifies the missing inputs and SMT contradiction. Its user-status answer repeats the article's unmerged status, demonstrating why the now-merged #524 needs a status update. Both held-out answers pass: the reader rejects causal identification from the W12 null and computes that eight beats seven under the printed objective. It also independently distinguishes model cost attribution from recoverable time. These are answer-level observations, not a claim that the page “clicked.” No revised text was produced or second-pass convergence claimed in this review.

## Follow-up and evidence boundary

Expected: a cache/CPU model would reproduce selected counts and transfer when block size shrank. Observed: scoped worker-plan gains repeat, but the printed minimisation selects a different integer and the frozen -3 optimum fails. The measured seven-worker win remains; no portion of the model's 44% has thereby been recovered. Counter changes narrow plausible explanations but do not identify a unique residual cost.

Next value: publish a factual follow-up correcting units, integer selection, SMT, causal status and prediction scores; show the pending low-sweep results only after they are available under their frozen rule. Direct latency/bandwidth evidence can distinguish the proposed mechanisms; an amended serial-limit model must be labelled a new model and independently tested. No extra workstation run is authorised by this review.

Primary records opened: LOGs [6931719814](https://github.com/lmmx/mibz2/pull/145#issuecomment-6931719814), [6942020517](https://github.com/lmmx/mibz2/pull/145#issuecomment-6942020517), [6943194948](https://github.com/lmmx/mibz2/pull/145#issuecomment-6943194948), [6944271342](https://github.com/lmmx/mibz2/pull/145#issuecomment-6944271342), [6944610173](https://github.com/lmmx/mibz2/pull/145#issuecomment-6944610173), [6945320222](https://github.com/lmmx/mibz2/pull/145#issuecomment-6945320222); prior independent [REVIEW6943617731](https://github.com/lmmx/mibz2/pull/145#issuecomment-6943617731); calibration at 33e7225ba6147fb480168dc54fe30727f0b93aea; records45–48 at PR497 head be614017696167f2f84ed7dafee600afc1afa8b6. The newer record48 was consulted as a correction, not substituted for frozen45–47. No novel prior-art claim is needed to establish these errors; elementary discrete optimisation and accounting suffice.
