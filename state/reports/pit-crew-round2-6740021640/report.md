# Pit-crew blog: round 2

**REVISE two local factual statements. The structure converged; no publication HOLD.**

Assignment [6740021640](https://github.com/lmmx/mibz2/pull/162#issuecomment-6740021640). Reviewed `docs/site/blog/2026-11-09-pit-crew-service-threads.md` at **4840f1e9f83d91e80e686b2d14dae81a4ca701d1**. Operating specs: a4f97714cc14366984015869d9d094d89488ac57. Research only. No article, production, policy, measurement or automation change.

## Baseline and convergence

The round-one independent baseline remains the comparison standard: [baseline committed before the original post was read](https://github.com/lmmx/mibz2/blob/558ea6952d288c3a5692e55d8a7a4451643dd865/state/reports/pit-crew-round1-6739269334/baseline.md). I reopened the primary registrations/results and safety reviews to check it. This is a revision review with prior knowledge, not a newly blinded baseline reconstruction; the revised post was retrieved before that recheck.

Simplest account: two shared service workers perform preparation and finishing for two three-thread inversion teams. This whole package achieved registered speed 1.045 [1.017,1.073] with 12.7% less summed processor time on the i9/Wikidata sample, using eight decoding workers against nine. It establishes neither an isolated cause nor production qualification.

All six baseline takeaways now arrive in the opening: worker roles and lost-team cost; headline result and scope; two teams alone lose; the single-service failure and twelve-thread trade; admission changes inconclusive with remaining waits unidentified; prototype and scoped safety status. The new §2 timeline supplies the opportunity cost of the leader's serial work. §6 separates admission from in-flight credits and earlier idle time from later starvation. The cost plot now supports the controls in §4. The unrelated result has moved out of the argument.

Expected: fewer missing steps, early scope, successful transfer. Observed: the isolated reader recovered all five ordinary answers and the central held-out inference; no unresolved chain/beyond-level blockage. The gap is now two statement-level inaccuracies, not the architecture of the article. Another wholesale rewrite has low value; correct the two statements and verify the exact revised wording and render.

## Required factual corrections

### 1. The thread setting is not partitioned into scanner, bookkeeping and decoding threads

The opening says the caller tells mibz2 how many threads it may use, then subtracts the scanner and bookkeeping to leave decoding threads. The registered accounting contradicts that explanation. The [2×3 registration](https://github.com/lmmx/mibz2/blob/3e201246cf1266d4b2a984cb19eb9fdba5b9835d/docs/journal/2026-11-08-service-2x3-registration.md), “The cells”, explicitly excludes support threads from the budget:

| Benchmark arm at setting 10 | Decoding threads | Other threads, including benchmark consumer | Total in this accounting |
|---|---:|---:|---:|
| Default | 9 | 3 | 12 |
| Service 2×3+2 | 8 | 4 | 12 |

The [first registration](https://github.com/lmmx/mibz2/blob/e4a41080eaeca974feffa50540eb557131f6d170/docs/journal/2026-11-07-service-core-registration.md), “Budgets, and the threads every arm runs”, names the same convention. Teams of three leave one decoding-budget slot unused in the default. The service plan uses eight; support threads are additional. The consumer is the harness/calling thread, so do not present twelve as a universal number of library-created threads.

Suggested replacement for the opening's first four accounting sentences:

> The caller's thread setting determines the decoding plan. It budgets the decoding workers; the scanner and support threads are additional. At a setting of ten, 0.2.6 uses nine decoding threads in three teams of three. The service layout uses eight: two teams of three plus two service threads.

Keep the existing feeder caption. A new figure is unnecessary; an optional two-row accounting table would suffice. This correction preserves the important eight-versus-nine comparison without implying a hard process-wide cap or subtraction of support threads from ten.

### 2. Qualify “one service thread is too few” by the tested layout

§3's table correctly identifies the failed arm as **3×3+1**, speed 0.799 [0.786,0.812] against default. The headline successful arm is **2×3+2**. The opening and final lesson can be read as a matched one-versus-two-service test at two teams, which these records do not contain.

The [first result](https://github.com/lmmx/mibz2/blob/e4a41080eaeca974feffa50540eb557131f6d170/docs/journal/2026-11-08-service-core-result.md) also has a matched three-team K=2/K=1 contrast, D1 = 1.318 [1.297,1.339]; that supports the three-team saturation account. It does not supply a two-team K=1 result. Reducing teams changes offered work and contention; failure with three teams cannot alone prove failure with two.

Use “One service thread could not keep up with the tested three-team layout” in the takeaway. Qualify the nearby “Two service threads did; one did not” and the closing “one was too few and two were enough” similarly. No new experiment is requested, and no registered result is reopened.

## Fresh-reader answers and grading

An isolated subagent read only the pinned article source, including visible SVG text/captions, a programmer profile and the questions. It received no prior review, sources or expected answers. Its full response is [reader.md](reader.md).

| Goal | Assessment |
|---|---|
| Summary | Recovered stage ownership, either service doing either job, 2×3+2, 1.045 interval, CPU/RSS, eight/nine workers, prototype and single-machine/file scope. |
| Motivation | Recovered the absent leader during build, reclaimed shared work, fewer teams, service capacity and hand-off costs. |
| Status | Distinguished first reported 1.045, registered 1.045 and later unjudged 1.073; no replacement of the registered result or clean repeatability inference. |
| Controls | Rejected “two teams alone”; retained the whole-package and unisolated-cause limitation. |
| One-service failure | Recovered 0.799, marked-busy versus CPU, backlog and waits; independently identified the three-team scope restriction above. |
| Held-out 16-core transfer | Correctly refused to predict the same plan wins; named the unknown default, useful parallelism, cache and service/team balance, and paired elapsed/CPU/memory measurements with validity controls. |

Transfer grading is not perfect verbatim recovery: the reader did not explicitly repeat that the proposed production selector fires only on the measured ten-core shape, nor explicitly say “freeze a registration before the new run.” Those requirements are present in the post's §7 and earlier definitions. The scientifically central transfer succeeds: no extrapolated win and no chosen sixteen-core plan. Its suggested broader experiment is a hypothetical reader answer, not authorization to vary the reserved grid, repeat policy or workload.

The reader starts at the opening. §1's vocabulary is sufficient; §2 now explains the decisive step; §3 introduces the evidence conventions at a useful point. No blocking symbol, step, chain or beyond finding. Opening repetition and the model-prediction table are optional compression opportunities, not reasons for another structural pass. Seven existing figures cover the comparisons, roles, sequence, opportunity cost and two possible waits; adding more would dilute the improved line of argument.

## Factual reconciliation and verification

- The §2 no-serial-work/possible-wait distinction survives.
- The conditional stock counterexample is now sound: preparation earlier may mask a later finite finish, without changing finish-first priority, preempting a finish or releasing credits. It is expressly schematic, not causal attribution from the trace.
- §6 retains INCONCLUSIVE for thresholds three/four; queue admission is not a strict occupancy cap. The corrected [lookahead record](https://github.com/lmmx/mibz2/blob/f9284a4b96bb258454525599a447173e9b201df6/docs/journal/2026-11-09-service-lookahead-result.md) supports withdrawal of the trace-based bottleneck claim and the two surviving hypotheses.
- Interval wording now describes uncertainty of the paired geometric-mean estimate, not the range of rounds. “A few per cent” is acceptable as a scoped summary; the text explicitly disclaims a clean repeatability estimate.
- P1/C1/P2 values, medians and predictions match the [2×3 result](https://github.com/lmmx/mibz2/blob/94a205c34d35205ab1e4d72c92deb15945265cb4/docs/journal/2026-11-09-service-2x3-result.md). Recomputed 1−1/1.146 = 12.739965%; transformed bounds 10.952805–14.529915%.
- First-run C2/C4/S4 and their CPU intervals match the first result; later D-a2w8/LA3/LA4/J4 match the lookahead record. Reported status and the selected J4 comparator remain explicit.
- D045/D053/D057 channel records were reread. The safety narrative retains exact research-snapshot scope, no production qualification and no automatic transfer of old performance to guarded code.
- “Less processor time” and the twelve-thread exception survive. The stale #289 merge wording is gone.
- Independently ran the exact head's prose checker: **23/256 sentences flagged (9%); dek PASS, 25 words, one break, two figures, reading ease 56**. All seven inline SVGs parse as XML. These checks establish neither layout nor factual truth.

Limits: no new timing, raw-bundle replay or Rust/safety qualification. The third run's 64/74 MB cross-run RSS statement remains supported by the author's #301 bundle check, not a fresh raw-bundle extraction here. Live-page retrieval failed; Playwright was attempted but its browser binary is absent. Both parent and reader used pinned source/visible figure text, not a rendered viewport. #301 reports its own render checks at 1338e463; those are author evidence, not my inspection. The requested rendered-text review is therefore only partially fulfilled. Retain the normal exact-head render check when applying these corrections.

Routing audit: five historically resolved misroutes, zero unresolved. Migration check passed before receipt; previous branch and manifest digest unchanged. Task remains enabled.
