# The knee is gone — round 2

**PROCEED.** The three material round-1 findings are resolved in the source at **afc5c5ab5fb4cd1c9f4fee838417693e42c08ff0** and on the live page. The article now teaches the bounded result: a better default plan on the measured machine/workload, seven actual workers behind high requested counts, and an unidentified plateau. One nonblocking caption correction remains below.

Assignment: [6957030345](https://github.com/lmmx/mibz2/pull/162#issuecomment-6957030345). Instructions: docs/team/{SPEC.md,SPEC.RESEARCHER.md,MODE.md,RULES.md,PROTOCOL.md} at a2da49638ac54d7763de1c167bd054f67ffdc763. This is an article review, not performance qualification, implementation approval or a new measurement.

## Independent baseline and limits of the reader exercise

Receipt: 050bba13c9101423dbee79543e09e19e23866625. The evidence baseline and freshly recomputed release medians were committed at **9fc692918dd2a8a4bfc49ef48aad4fa0c960a164**, before opening the revised article and PR550/551 bodies this run. See [baseline.md](baseline.md), [raw-summary.json](raw-summary.json), [checks.json](checks.json) and [plan-checks.json](plan-checks.json).

I reread all sixteen checked-in release-scaling runs and metadata, the attribution ledger, the four requested LOGs, and the knee and memory-controller records. 0.4.0 is represented as the older arm in 0.5.0's run, not as a seventeenth directory. I reconstructed release and within-release ratios from round-matched log wall ratios, exp(mean ± t[9,.975] × sample-SD/√10). The numerical checks use 2.262157 for the t critical value. No new decoder run or timing occurred.

**Not blinded:** I reviewed round 1 and had already seen the held-out question and its key. Committing the pre-answer before reading this revision prevents adapting it to the revised prose; it does not restore blindness. The reader pass below is one reviewer's explicitly simulated reader, not a fresh-human or independent-subagent comprehension experiment. No empirical reader-success claim follows.

## Round-1 objections, closed exactly

| Finding | Revision and independent check | Disposition |
|---|---|---|
| Affinity confused with exclusive core assignment | §3 now says CPUs0–9 are an allowed set shared by workers, scanner and reader. src/parallel.rs admits exactly T logical CPUs on T distinct cores, or the whole-machine mask from10, conditional on measured_i9. An allowed set does not bind each thread to its own core. | Closed |
| Selected release history presented as exhaustive | §2 and Figure3 caption explicitly omit0.2.5–0.3.3. New0.2.5 counterexample reproduces: ten/eight =0.918428 [0.909803,0.927135]. | Closed |
| IMC alone would supposedly settle saturation; mismatched reference line | Figure5 now compares37.6 read-proxy ceiling only to25.3 read proxy. The51.2 assumed peak is confined to the estimated-total row. Caption requires current counters and a measured sustainable ceiling for a similar read/write mix. Live rendering has separate, non-overlapping labels/lines. | Closed |
| Date and overbroad default-policy wording |0.2.4 date is6November; measured configurations replace the universal extra-worker claim. Opening now states seven active workers and unresolved bandwidth. | Closed |

The corrected Figure5 establishes what measurement is missing; it does not promise that counters alone isolate causality. A controlled intervention can still be needed. This is consistent with its revised “test whether” language.

## Fact-to-source audit

Sources below are pinned to the reviewed commit unless a different immutable record is named. The four requested LOGs are context, not a substitute for the raw checks indicated.

| Article fact or figure | Check and result |
|---|---|
|10 cores,20 logical CPUs,20MiB L3; machine named once|release metadata, knee lscpu and planner profile agree. The rule also requires the specific CPU identity, kernels and no quota; it is not every ten-core CPU.|
|11.9MB input to209.7MB output; ten rounds|meta.json gives11854933 input bytes; raw rows give209700196 output bytes and ten rounds/count. All sixteen files have240 rows and no recorded unsuccessful termination. Current-arm output hash is stable.|
|Figure1 five throughput curves|Fresh median wall calculations for0.2.4,0.5.0,0.5.1,0.5.3,0.5.4, converted as output/wall/1e9, agree with the plotted shape, labels and1.95–2.06 range. These are separate release runs, not a single randomized five-way experiment. Placement changes above10 are visibly marked.|
|0.2.4 plateaus under4×2 teams;0.2.5 gain about9%|The original policy registration's two reported20-round trials give1.089 and1.088 speed ratios at10, with4×2→3×3 plans. Distinct from the later release comparison.0.2.4's first release file lacks observed_threads; the older arm in0.2.5 and the policy record provide the plan evidence.|
|Cells/lazy gains reached workers before high-count teams|Attribution ledger identifies kernel/memory changes, ports and routing separately;0.5.0 observed_threads confirms8 independent workers at8 and3 leaders+6 helpers at10–20. No claim that one cache mechanism explains every historical step is needed.|
|0.5.0 takes16–18% longer at12–20 than8|Recomputed geometric ratios1.180722,1.163740,1.168662; matches prose. Ten/eight1.104834. Throughput loss and wall-time increase have different denominators.|
|Figure2 plans|Raw thread names confirm0.5.1 ten workers at10+,0.5.2 nine above10,0.5.3 seven above10 but ten pinned at10,0.5.4 six pinned8 and seven at10+. Unpinned branches come from the scoped control and planner, not the pinned rows alone.|
|§2 sixteen wall entries|Raw medians reproduce all displayed three-decimal values.0.5.4 exact values .38510/.24115/.14365/.10470/.10200/.10580/.10615/.10755.|
|1.123 at10;1.045 at8; four unresolved|Fresh paired calculation:1.122599 [1.094015,1.151930];1.045176 [1.037880,1.052524];4t1.020422 [.993875,1.047678]. All1/2/12/16/20 intervals include1. CPU and RSS at8/10 decrease in the same paired run.|
|“About6%”20 versus10|Unrounded medians give5.44%; displayed .108/.102 gives5.88%. Acceptable approximate wording, not a statistical ordering claim. Nearby text explicitly separates median ordering from interval evidence.|
|Figure3 within-release ratios|Recomputed all available arms. Current12/16/20 over8 are1.01825/1.01375/1.04246;10 is.97101.0.4.0 uses the older arm of0.5.0. Selected-release caption fixes the historical counterexample.|
|0.5.3 missed pinned plan;0.5.4 recognizes machine separately|LOG6946756938 and planner predicates distinguish machine-wide identity from target affinity. Forty versus28 slots follows four slots/worker. Current source caches machine facts with OnceLock while re-reading affinity/quota as needed.|
|Unpinned production7 in6/6; explicit7/10=.873; low trims fail to transfer|LOG6949008682 records production check and explicit-arm comparison separately: .8729 [.8656,.8803],8t .9999,6t1.0088,4t1.2143. Revised article preserves that distinction and says CPU competition is implicated, not causally isolated. This run reread the LOG and implementation; it did not re-extract that control archive.|
|Hardware discovery overhead fixed pre-release|CHANGELOG0.5.4 records failed first release check44095874788, #545 and recheck44108684132; source contains once-per-process machine cache. No new CI rerun here.|
|About half LLC misses;25% cycles; instructions/clock approximately level|Primary record47 at c7ea4a5c and local workers-at-cores counters give17.32/22.98=.754 and.01002/.01964=.510. Record51 reports the later retired-miss ratio.390 and other counter changes. Thus “about half” is approximate across events/runs, not a universal invariant. The local paired median-of-ratios cycles is.7517, also about25% less.|
|Per-worker76% versus92%|The companion notes' worker-time decomposition reports.763/.925. Do not substitute counters.txt's aggregate process task-clock divided by worker count: it includes nonworker CPU and can exceed1. This is a source/decomposition check, not a fresh reconstruction of every occupancy trace.|
|No measured serial starvation in x4 unpinned;93–95% busy|Knee raw/report has160 successful runs,320 trace records. x4 unpinned idle .010–.015; front/window/consumer starvation refuted by registered rules. Busy means executing decoder work including memory stalls; it does not prove arithmetic compute-bound execution or rule out a future serial limit.|
|Figure4 green6.9/11.0/12.7, hollow14.7 pinned|Raw miss count×64/output recomputed6.892 at4,11.018–11.057 at7,12.648 at8,14.656 at10 pinned. Placement difference is labelled.|
|Figure4 amber15.5/20.4/22.7/24.2|Independently recomputed idle-subtracted IMC totals15.5444/20.4229/22.7010/24.1889 at4/6/8/10. Idle reads/writes in the raw records are already rates: subtract rate×span, then divide by total decoded output. Geometric means match the archived reader. Different decoder/counter series stay separate.|
|Figure5 output2.29, reads25.3, read ceiling37.6|Knee x4 seven-worker rows give .36565–.36640s,838800784 output bytes and25.22–25.33 read-proxy GB/s. Separate triad ceiling37.592. Current IMC is unmeasured.|
|Figure5 estimated~45; assumed51.2|Old IMC means give writes/reads .7841 at8 and.7434 at10;25.3×(1+r)=44.1–45.1. Carrying that older ratio to the current kernel is an assumption, correctly labelled.51.2 is dual-channelDDR4-3200's nominal arithmetic, not a read-back DIMM rating or sustainable mixed-traffic ceiling.|
|Model failed smaller-block optimum; portability open|Record48 at c7ea4a5c reports frozen optimum9 versus measured7 and predicted wrong sign for7/10. The article does not elevate the fitted rule into a portable production law.|

Knee archive SHA256 independently rechecked b145bb1e596b719eae1cdb0a13168dd4b6c8378f42bae42cdf1a323a7792792d. IMC archive SHA2562661310a6622534408bd440717d96f171d1ee25d6850064010fd639c1063f2cb. Three MONODROMY records were absent at the article commit; I fetched them at the exact c7ea4a5c5e8447fbdfd407c6f0ae07b6669bd87e links in the notes instead. Their stronger causal interpretations are not inherited as proof.

## Takeaway placement and simulated reader

| Takeaway | First useful landing; evidence | Simulated reader answer |
|---|---|---|
|The default fall is gone, a plateau remains|Opening¶2 and Figure1; §2 table/intervals, Figure3|More requested threads no longer cause the old large penalty on this sample/machine;20 is still slightly slower than10.|
|Fewer actual workers created the result|Opening states7; Figure2 explains release transitions; §3 supplies predicate error|12/16/20 requested is not12/16/20 active decoders. A flat curve partly reflects an unchanged seven-worker plan.|
|Interference can make fewer workers faster|Opening cache competition; §4 and Figure4|Counters and faster7/10 support interference; they do not identify one resource as the cause.|
|Placement and hardware restrict transfer|Machine in opening; Figure1 placement divider, Figure2 split cells; §3 box|Low-count trims are pinned-mode choices. Ten-worker trim holds in both measured modes. Neither “pinned” nor “ten-core” is a universal qualification.|
|The plateau's bottleneck remains open|Now explicit before Figure1; §4 question and Figure5; §5 alternatives|Measured proxy reads, estimated total traffic and assumed nominal bandwidth are distinct. Need current counters and a comparable ceiling.|

The densest causal material is §3's identity/affinity distinction. Figure2 already carries the changing plan counts; the revised box carries eligible CPUs. Another figure is not necessary. Figure4's kitchen analogy aids intuition but must not substitute for counter semantics; its caption states where it breaks. Figure5 carries the numerical comparison much better than another dense paragraph. Desktop live inspection found the five figures legible and the corrected line/label scopes visible. This was not a fresh phone/dark-theme certification; PR551 reports those checks, but I do not claim to have rerun them.

### Goal questions, including the transfer question

- **G1:**0.5.0's12–20 requested counts ran teams and cost16–18% over8.0.5.4 runs7 workers and sits within a few percent of8. The release-to-release absolute wall decrease is a separate comparison.
- **G2:**Fewer concurrently active block structures can lower interference and per-byte cost enough to offset lost parallelism; placement can also free scheduling capacity. Reject the PR550 key if read as “past seven always adds misses faster than compute.” Seven is the measured policy, not an identified universal threshold or isolated causal law.
- **G3:**0.5.0 switched from workers at8 to teams from10; the teams had not received the same worker-path memory gains.0.5.1 changed the route.
- **G4:**The explicit ten-thread7/10 comparison gained unpinned (.873) as well as pinned (.881–.889). Below10, the measured pinned trims did not carry over; no claim of exact zero cache benefit follows from their net wall results.
- **G5:**A larger L3 or more channels can change the optimum, but neither determines its direction alone. Sweep actual worker counts, record placement, wall, per-byte cycles and total traffic, misses/latency and serial idle time on the new machine/workload. Do not port7 by core count.
- **Previously exposed held-out question:**Halving total DRAM traffic/output doubles the bandwidth roof B/b if sustainable B is unchanged. It raises actual throughput only while that roof binds and other costs permit it. Latency/MLP, L3/interconnect, compute, a fixed worker policy or newly exposed front-end/consumer work may limit the result; unchanged throughput is possible. Halving a read-miss proxy is not necessarily halving total DRAM traffic. This answer was committed before opening the revised article this run, but it is not blind. The revised opening, §4 and §5 support the conditional answer without asserting2× speedup.

## One nonblocking caption correction

Figure4 opens “Measured on the test machine, decoding four copies of the Wikidata extract.” That describes the green knee series. The older amber cells-imc records instead contain15 decodes per reading, each with209700196 output bytes: the single sample, repeated, not the concatenated x4 input. Suggested clarification: **“Green: four concatenated copies of the Wikidata extract. Amber: repeated decodes of the original extract on the older kernel.”** Keep the existing different-kernel/counter caveat. The per-output-byte values are correct and the series are already separated, so this does not overturn the argument or require another full review. Also prefer “busy decoding” to “compute” on a future prose pass, to avoid suggesting memory stalls are excluded from busy time.

## Consequence and boundary

Expected: the three corrections would remove the false placement inference, historical generalization and bandwidth identification. Observed: all three are repaired in source and live rendering; numerical results reproduce. Gap closed: the reader can distinguish requested concurrency, actual workers, measured traffic and an unresolved bottleneck from the opening onward. Mechanism learned: unchanged seven-worker allocation explains much of the high-request plateau; current evidence supports interference without choosing DRAM saturation over cache/interconnect latency. Next experiment's value: measure current total traffic against a matched sustainable ceiling, then test a bounded traffic/access-cost intervention with whole-decoder accounting. No new experiment is registered by this review.

Migration before receipt passed: old head1283b969e9e295e268257cd1b6be3ad7da29e6dd and manifest digest9b6bfe5b8e2193a68d8e788873da11947197c3457b6ad6e454b580a05f7b8fd0 match cursor. Inbox pages1–2 had only this unfinished assignment. Wake coord.py audit returned no findings. The helper's verify hard-codes FROM coordinator; publication will record that limitation and independently check FROM researcher, TO coordinator, TYPE REVIEW, issue145, author and exact body. State stays on team-state/researcher; no implementation, merge, timing, other-inbox publication or automation change.
