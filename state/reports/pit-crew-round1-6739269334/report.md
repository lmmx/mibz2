# Pit-crew blog: round 1

**REVISE the published post through follow-up PRs. No publication HOLD.**

Assignment [6739269334](https://github.com/lmmx/mibz2/pull/162#issuecomment-6739269334), dispatching QUEUED 6736621741. Reviewed `docs/site/blog/2026-11-09-pit-crew-service-threads.md` at **a773853f05fa9e12bb65fb93bf5c07e1bc7fea11**. Instructions pinned at a4f97714cc14366984015869d9d094d89488ac57. Research only; no post, production code, policy, or automation changed.

## Independent baseline and opening

The [baseline](baseline.md) was derived from the primary records and committed at **558ea6952d288c3a5692e55d8a7a4451643dd865 before fetching the post**. Its simplest account: two shared workers take preparation and finishing away from two three-person inversion teams. Against the three-team 0.2.6 default, this whole arrangement achieved registered speed 1.045 [1.017,1.073] and 12.7% less CPU time on one i9/file, with eight versus nine decoding workers. That establishes neither the isolated cause of the gain nor a production default.

| Baseline takeaway | First useful arrival | Finding |
|---|---|---|
| What moved; 2×3+2 versus 3×3 | Opening paragraph 1 | Prompt and intelligible. Define “ten threads” as caller setting here, rather than making readers reconcile eight/nine with ten through a later caption. |
| Registered gain, CPU reduction, one machine/file | Opening paragraph 2; exact file in §3 | Main result arrives promptly. Put prototype status and explicit single-file scope here too. |
| Two teams alone lose; bundle is not a causal isolation | Opening plot, explanation §4 | The negative control is visible early but its meaning and confounding caveat arrive late. One sentence before/within §1 would suffice. |
| One service worker loses; extra-budget 3×3+2 is a trade | Opening; twelve-thread trade §3 | Single-worker result is early. A short mention that the extra-worker alternative buys speed with more CPU prevents an apparent universal two-service-worker prescription. |
| Larger admission threshold inconclusive; trace labels do not identify a bottleneck | Opening; warning §6 | Outcome early, mechanism qualification late. Say the remaining waits are not yet causally identified before presenting the diagnostic narrative. |
| Opt-in prototype; scoped safety closure is not production approval | Explicit in §7 | Material adoption caveat too late for an opening read. Do not move the entire safety history forward; move its consequence. |

The fresh reader independently recovered all five supplied goal answers. This is not a wholesale comprehension failure. The intervention is to shorten the route from headline to correct scope, then repair specific statements and missing temporal steps.

## Ranked corrections: factual follow-up first

**1. §2: performing serial stages is not the same as waiting for them.** “None of them waits for MTF or the finish” is false as an absolute; §6 itself documents waits. Replace with: *No team member leaves the shared build and walk to perform MTF or finishing; teams can still wait for those stages or for free buffers.* This preserves the useful pit-crew metaphor without promising uninterrupted inversion.

**2. §6: the threshold inference is too strong in two places.** “The threshold alone is not the change that helps” exceeds INCONCLUSIVE. Use *Raising the threshold alone did not demonstrate a gain in this run.* “Raising the threshold helps with neither” also does not follow from the two surviving mechanisms. A larger threshold does not change finish-first priority, preempt finishing, or release ordered-output credits. It can nevertheless leave more prepared stock before a finite finish interval.

Counterexample to impossibility, not a prediction: suppose both workers enter long finishes with two versus four prepared blocks available. Two teams consume two blocks per inversion interval. The smaller stock runs out sooner; the larger stock masks another interval if the finish ends within it. Both histories respect nonpreemptive finishing; no priority change is required. This is a conditional scheduling construction, not an executed trace or a claim that the observed workload benefits. Retain the proposed eligibility/credit diagnostic: the existing result does not select a cause.

**3. Figure 1 caption, §3 and §5: distinguish confidence interval from round-to-round spread.** The registration specifies a paired geometric-mean ratio with a 95% t-interval over 20 rounds. “Spread across the 20 rounds” suggests a distribution/prediction interval for individual round ratios. Say *uncertainty in the estimated paired geometric-mean ratio within that run*. The warning that it does not quantify between-run variability is good and should remain. The later run also changes prototype commit and experimental context, so a difference between estimates is not a clean estimate of repeatability at fixed code.

**4. §6: #289 is already merged.** Remove “not yet merged.” GitHub records merge at 2026-11-09T09:25:49Z, commit **2745e0c999cfa01a73515fde8b9cf68a088ac63f**. The substance of the trace-label withdrawal is correctly represented.

**5. §7: CPU time does not prove less machine work.** Replace “does less work in total” with *uses less aggregate processor time*. The causal saving is explicitly unmeasured; neither instructions, energy, nor amount of algorithmic work follows from CPU seconds alone.

**6. §7 scope sentence needs its twelve-thread exception.** “Everything here … at ten threads” conflicts with §3's twelve-thread comparison. Say *The headline comparison uses the ten-thread setting on this ten-core processor; §3 also reports a twelve-thread configuration.*

## Reader pass and restructuring

A fresh isolated reader received only the post text (including SVG labels, alt text, captions and tables), a programmer profile, and the five goal questions. It did not receive these findings, the primary records, or the held-out transfer question. Its detailed response is in [reader.md](reader.md).

- **Symbol:** no serious blockage. Stage expansions are sufficient; explaining MTF/BWT algorithms would add bulk.
- **Statement:** eight/nine decoding workers versus a ten-thread setting; the no-wait contradiction; the overly strong threshold conclusion.
- **Step:** service scheduling is a rule at a worker's next job selection, not a guarantee that an empty prepared queue is immediately replenished. Qualify the “never leave [it] empty” formulation accordingly.
- **Chain:** “teams could run dry while a service thread sat idle” elides time. Earlier idle preparation capacity can be followed by later starvation while both workers finish. This need not mean a worker remains idle with eligible preparation available.
- **Chain:** explain prepared-queue admission versus all-in-flight credits before introducing threshold four/window twelve. The former regulates when preparation starts, not a strict occupancy cap (with two workers, occupancy may reach L+1); the latter spans admission through ordered consumption.
- **Beyond:** the unrelated run-fed MTF result near the end fragments the argument. Move to a short research-note aside or link.
- **Collapse:** “service threads caused the saving” would collapse the tested package into one isolated treatment. The post mostly avoids it; retain §4's explicit extra-workers/feeder/queues/handoffs caveat and bring its consequence forward.

The reported prose checker result (30% overloaded sentences against 15%) was supplied in the assignment, not independently rerun here. The reader identifies where density actually blocks an inference: §6's two gates and time sequence, then the opening's full experiment inventory. Fix those structures rather than mechanically splitting every long sentence.

### What the four figures carry

1. **Opening interval plot:** accurately preserves registered versus reported status and the change of comparator in the lower panel. It asks a newcomer to understand seven estimates before the stage model. Keep the headline comparison prominent; move the threshold panel to §6 if simplifying the opening. Fix interval semantics.
2. **Worker allocation:** carries ownership and lost-team cost. It does not show the temporal opportunity cost of a leader alternating serial work with shared work.
3. **Block race circuit:** useful stage order, not a duration or scheduling explanation. Preserve its non-timing qualification. It cannot resolve finish-first and credit blocking on its own.
4. **Cost plot:** good direct comparison, but duplicates the §4 table after the argument has moved on. Put it beside the control interpretation; retain a compact numerical table only if needed for exact values.

Two replacement visual briefs, not additions to an already long post:
- §2: schematic lanes for old leader/helpers and new team/service workers, showing the leader's absence during shared build. Label as schematic; widths are not measured durations.
- §6: two panels. A shows earlier preparation opportunity, both workers finishing, stock depletion and subsequent refill. B shows later output-ready blocks retaining in-flight credits behind an earlier unfinished block. Label both **possible mechanisms, not measured attribution**. Replace the corresponding dense causal paragraphs.

Keep growth within the drafting workflow's 25% ceiling; relocation and deletion should make net growth unnecessary. Do not burden the opening with every interval or safety checkpoint.

## Source audit and limits

Primary records were read before the post: [registered 2×3 result](https://github.com/lmmx/mibz2/blob/a773853f05fa9e12bb65fb93bf5c07e1bc7fea11/docs/journal/2026-11-09-service-2x3-result.md), [lookahead result with correction](https://github.com/lmmx/mibz2/blob/a773853f05fa9e12bb65fb93bf5c07e1bc7fea11/docs/journal/2026-11-09-service-lookahead-result.md), [first service result](https://github.com/lmmx/mibz2/blob/e4a41080/docs/journal/2026-11-08-service-core-result.md), Researcher [6736071231](https://github.com/lmmx/mibz2/pull/145#issuecomment-6736071231), and Guardian channel records D045/D053/D057. Also checked the service registration, predecessor blog posts, workload manifest and run-fed result.

| Claim group | Check |
|---|---|
| P1 1.045 [1.017,1.073], CPU 1.146 [1.123,1.170] | Matches registered record. Independently transformed CPU ratio: 1−1/1.146 ≈12.74%; interval about 10.95–14.53%. |
| C1 1.402 and P2 0.866; their CPU figures; five-arm medians; model predictions | Match 2×3 journal. Median ratios are correctly distinguished from paired estimates. Package interpretation is warranted; isolated causal attribution is not. |
| #251 invalid first attempt, registered unchanged rerun, N1 1.003; single-worker 0.799, 96% marked busy | Matches first-result record. Retain marked-busy versus actual processor-time distinction. |
| #251 twelve-thread 1.058, CPU/memory cost; ten-thread unjudged 1.045 | Correct trade and judgment distinction. |
| #271 1.073, CPU 1.169; threshold 0.992/0.979; selected window arm 1.049 | Matches lookahead record. Correctly unjudged/selected lead; cannot replace registered P1. CPU conversion ≈14.46%. |
| 20 measured rounds; six additional trace rounds; file sizes and i9/cache scope | Consistent with registration/result/workload records. Three run histories are distinguished from the discarded first attempt. |
| 16.5% wait-entry labels; 70%/67% marked busy; withdrawn moved-bottleneck reading | Correction accurately explained, apart from the categorical threshold claim and stale merge status above. |
| Cache explanation and prior pipelining result | Earlier records support their historical context. Current service experiment does not isolate cache effects; the post correctly says so. |
| D045 → D053 → D057 | Channel records support the scoped evidence hold and closure, not production qualification. Guard/integration changes cannot inherit measured performance merely from that closure. |

This is source-level factual review and arithmetic reconciliation, not fresh timing, raw-bundle statistical replay, safety requalification, or an exhaustive source-code audit. I did not independently establish every cross-run RSS detail from raw captures; the third-run “64 versus 74 MB” should retain a direct summary/bundle citation. A live-page fetch failed; both parent and fresh-reader reviews used the pinned page source and visible figure text, not a rendered browser viewport. Typography, clipping and responsive layout remain unchecked.

Round 2 should assess the restructuring and the held-out transfer question. That question was not administered or scored in this round. No second review round is claimed.
