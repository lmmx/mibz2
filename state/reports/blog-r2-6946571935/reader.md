# Fresh simulated-reader review — revision 55b26bd7

## Scope and method

I read only `post.md` and the article text, equations and SVG labels in the supplied downloaded `live.html` (reported HTTP 200). I did not inspect a live browser, rendered layout, screenshots, slider behaviour or mobile sizing. Visual comments below concern the information conveyed by labels and diagram structure, not verified visual rendering. I followed no links and read no research records, other reports or other agents' context.

Reader profile: a programmer comfortable with caches and pointers, new to bzip2 and this project's history. This is a simulated comprehension review, not a fact check of the underlying research. Scores measure how confidently I can explain the article from its own contents.

The six questions below are reviewer-reconstructed goals, not preregistered questions: the supplied PR #527 body did not actually state explicit goal or held-out questions. The final two transfer probes are newly reconstructed too, not genuinely held out. The article's claims that its research predictions were frozen before experiments are a separate matter.

## Opening takeaways

My initial understanding is: a worker decodes one whole block; workers compete in a shared cache; on one i9, fewer workers are faster and use less memory; the purpose of a model is to transfer a decision beyond that sweep; the model remains research, while a narrow measured policy is going into a release. That is a useful, coherent motivation for a programmer unfamiliar with bzip2.

The opening has one immediate contradiction of status. The dek says mibz2 “sizes its workers from a model,” while the second paragraph says the model is not the decoder's policy, and section 6 says no general selector has shipped. I resolve this in favour of the explicit caveats, but the dek gives me the wrong answer if I skim. Change it to an investigatory or future-tense claim.

The opening repeats the same result in the first paragraph, figure, following paragraph and first takeaway. The repetitions make the result memorable, but the takeaway list arrives after a substantial summary already. Keep the first result and first chart; shorten the repeated catalogue of ratios.

## Answers from the post

### Q1. Why model rather than sweep? — 5/5

A sweep finds good worker counts for the measured decoder, workload and machine. A change to worker memory use can change the choice, and another machine has different cache/core resources. The proposed model takes those resource inputs, so a changed footprint or detected machine topology can yield a new prediction without redoing every worker-count measurement. It still needs validation: this is an ambition for generalisation, not evidence that sweeps are already unnecessary. The opening makes that distinction clear after the dek.

### Q2. Why can fewer workers be faster; what is evidence versus attribution? — 5/5

Less parallel work is running, but each output byte becomes cheaper by more than the lost parallelism. At 10 versus 7 workers, effective active processors fall from 7.63 to 6.48, a ratio near 0.849; cycles per byte fall to 0.752. Their ratio is about 0.886, agreeing with measured wall time 0.885. That accounting is an identity under a flat clock, not a causal proof.

The useful evidence is the pattern: clock unchanged, instructions per byte almost unchanged, private L2 misses nearly unchanged, shared-cache misses much lower. It supports shared-cache interference contributing materially. It does not identify how much time is LLC hit latency/queueing versus main-memory latency, nor turn avoided misses into a counted cycle saving. The article's observed/supported/under-investigation paragraph is especially effective.

### Q3. Derive the choices at N=10 and N=8; distinguish measurement, fit and policy — 3/5

At N=10: CPU budget = (10−1.08−1.25)/0.93 ≈ 8.2. Cache bound ≈ 7.9, below 8.2 and 10 cores. The fitted continuous optimum is therefore around 7.9, and comparing the displayed integer costs gives 8 a small advantage over 7. The measured winner is 7. A separate conservative 0.7-of-cache convention also gives 7. Thus I cannot honestly derive measured 7 as the unique optimum of this fit; the article correctly says I should not.

At N=8: CPU budget = (8−0.85−1.0)/0.93 ≈ 6.6, below the same 7.9 cache bound and 8 available cores. Choosing the integer below that bound gives 6, which matches measurement. The post says the cost model nevertheless puts 6 and 7 nearly level, while measured 7 is roughly 5% worse. The continuous resource bound, integer-selection convention and prediction of relative wall times are different objects.

I can reproduce these explanations, but the bound-chart label “cache binds: 7 workers” collapses the distinction its preceding figure painstakingly establishes. It should read “binding bound: 7.9; measured choice: 7,” with a separate line for fit-preferred integer 8. The 8-thread panel should similarly show “binding bound: 6.6; convention: 6; measured: 6.”

A second stumbling point is arithmetic internal to the displayed model: the text says c(10)=1, but a≈0.42 and b≈0.0068 give c(10)≈1.10. The stated c(7)≈0.76 does agree with those coefficients. Is the regression approximate and its fitted value at 10 different from the normalised observation, or should the curve be explicitly divided by its fitted c(10)? The post does not tell me. This is a reader-level reproducibility obstacle, not a finding about the external data.

### Q4. Transfer to two cores/four logical CPUs or multiple cache domains — 3/5

I should not mechanically cap workers at two using the headline min(C, …) equation. That equation is scoped to one worker per physical core. The full SMT model assigns a two-worker core a yield of σ≈1.24–1.38 and scores counts accordingly. On the sampled runners the cache bound does not bind; the prediction is four workers, or three within 2%, with today's plan retained. The refuters are usefully explicit. I can explain the principle but cannot reconstruct that count score numerically because the full scoring expression is absent.

For several cache domains I sum min(C_d, 0.8 L_d/F) over accessible domains, provided workers actually spread across them. CPU budget remains process-wide; the plan needs cache-level discovery, shared_cpu_list and soft domain affinity. I understand why summing all nominal cache capacity without placement would be wrong. However, the per-domain formula again caps at physical C_d, so its combination with SMT is not explained. One sentence saying this too is the non-SMT form would prevent me applying it as a universal selector.

### Q5. What reaches users versus remaining research? — 5/5 after reading section 6

The merged change is seven lazy workers for the measured i9 profile from ten requested threads, due in the next release. The general model has not shipped. Choices of six at eight threads and five at six threads await review; four threads retain today's eager plan. Other machines retain existing behaviour pending transfer tests. Proposed future behaviour is detect once, decide once at startup, no midstream tuning. This is clear at the end and in the takeaways, but should also be true in the dek.

### Q6. What fails in the low-count and -3 tests, and what next? — 4/5

At low counts, the main model names 6, 5 and 4 workers for 8, 6 and 4 threads. Fixed 0.7-of-threads and a fixed idle-capacity rival fail. At four threads, the best lazy count is still four, but eager four is faster, so choosing a worker count does not imply switching the kernel. That distinction requires the long baseline paragraph; the figure caption helps.

For -3, smaller blocks reduce the footprint and measured contention, but the prediction that the optimum moves to 9 or 10 fails: 7 remains best, with only about a 2% gain. The three-limit model even predicts the wrong direction for 7 versus 10. A fourth scanner-throughput limit is a post-data hypothesis, supported by a throughput plateau and suggestive idle traces, with the trace's validity caveats made explicit. The old frozen scores must stand. Next: a fresh frozen test of the revised model, runner transfer tests, contention counters, a low-count intercept fit and reviews before further policy changes.

I understand the result without knowing bzip2 internals. The test summary table is helpful, but its many neighbouring variants make the exact genealogy of “simple rule,” three-limit model and later revision harder to retain.

## Additional reconstructed transfer probes

**Does the null time result for 12-byte rules identify the remaining cause? — 5/5.** No. Misses fell, instructions rose, and no time change was detected; overlap and offset are compatible with this. It neither proves LLC-hit queueing nor rules out a memory-latency contribution. Pending counter predictions distinguish alternatives. The post says this directly, and the table keeps measurements separate from predictions.

**Does a 44% model cost share promise 1.8× wall speed? — 5/5.** No. It is an extrapolated fitted cost decomposition, sensitive to exponent and intercept, not measured recoverable stall time. 0.76/0.42≈1.8 becomes a speedup only if that term can be removed without changing anything else that matters. The post explicitly says no such result has been shown; the 34–60% range is model-form sensitivity, not a confidence interval.

## Stuck points and diagnostic categories

| Location | Category | What stopped or slowed me |
|---|---|---|
| Dek versus paragraphs 2 and section 6 | Statement | Present implementation and proposed research are described differently. |
| c(10)=1 versus a≈0.42, b≈0.0068 | Symbol/statement | I cannot reproduce the declared normalisation from the printed coefficients. |
| “cache binds: 7 workers” | Step | A continuous 7.9 bound silently becomes 7 despite the explicit integer cost comparison preferring 8. |
| “simple rule” after 0.7 of L3, then “simple rule” = 0.7 of threads | Chain | Two different mappings share a name. I need to keep cache fraction and thread fraction separate. |
| N = logical CPUs in affinity set versus requested threads | Symbol | In the examples they agree by experimental setup; I am not told how they relate generally when the caller requests fewer threads than affinity allows. |
| “full model” SMT yield and domain sum | Beyond-post | I can explain the transfer intention, but cannot independently score a two-core case or combine domain arithmetic with SMT. |
| Low-thread model success versus today's eager kernel | Chain | A correct worker-count prediction is not the same as a beneficial kernel/policy change. The later paragraph resolves this. |
| “three thread counts it hadn't seen” after an 8-thread worked fit | Chain | Presentation order obscures which observations were calibration and which came later. I accept the stated chronology but cannot readily reconstruct it. |

The main fragmentation is repeated result summaries separated from model versions and baseline definitions. The main collapse is counting fit, integer convention and measured winner as a single “model choice” in a figure label. The largest missing steps are full SMT scoring and the relationship between requested thread count and affinity capacity. None requires teaching the entire decoder.

## Dense passages and exact visual/editorial suggestions

1. **“The CPU budget” paragraph:** replace the two five-number utilisation sequences with a small table, columns workers / predicted utilisation / measured utilisation. Keep the arithmetic and the caveat in prose. This frees the paragraph to explain why idle and non-worker capacity are subtracted.
2. **“At 10 threads … At 8 threads … Past the cores”:** use the existing bound chart, but change its result labels as described above. Add a compact adjacent table with columns binding continuous bound / fit's preferred integer / conservative convention / measured best. Do not add another decorative diagram.
3. **Low-thread paragraph beginning “The baselines differ by count”:** put N, baseline kernel/count, best lazy count, lazy ratio, and retained/review-pending policy in one table. It should visibly show eager-4 and lazy-4 as separate plans. The current count-only table cannot carry that distinction alone.
4. **“Across the 10-thread arms … about 589”:** separate the observed slope, assumed memory latency and inference into three short sentences. An optional two-row comparison is enough: 589 cycles per avoided miss across arms; assumed 390–490 cycles per uncontended memory trip. Label the second as an assumption and explicitly forbid treating the first as latency. The existing hypothesis table is already the appropriate visual for the next step.
5. **Rule genealogy:** a compact table would help more than a new curve: fitted 0.8-of-cache optimum; conservative 0.7-of-cache rule; rival 0.7-of-threads rule; three-limit model; post--3 four-limit revision. Give each a stable name and indicate fitted, frozen-tested or post-data. This addresses fragmentation without a chronology digression.
6. **44% discussion:** no new chart needed. The prose answers the dangerous inference clearly. A pie chart would misleadingly solidify an extrapolated model share.

## Length and overall judgement

Keep this as a substantial research post; do not cut the evidence/attribution distinctions or the failed predictions to meet an arbitrary short format. A reduction of roughly 10–15% is plausible through repeated introductory numbers, duplicate verdict summaries and converting numeric sequences to tables. The current 24-minute depth is defensible for its intended technical reader, but section 3 could use the largest improvement in pacing.

Overall, I can explain the main contribution and its limits well (about 4/5 comprehension). The article is strongest when it distinguishes observation, fitted model and hypothesis. The most important fixes are local consistency fixes: the dek's shipped status, the 7.9→7 chart label, the normalisation explanation and distinct names for the two 0.7 rules. These would make the argument easier to reproduce without requiring more research exposition.
