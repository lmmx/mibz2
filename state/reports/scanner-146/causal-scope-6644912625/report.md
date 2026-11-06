# Four-worker penalty: supported effects, unidentified mechanisms

Trigger: [6644912625](https://github.com/lmmx/mibz2/pull/145#issuecomment-6644912625), 2026-11-04.

**The record identifies where much of the historical i9 cost accumulates and shows that changing output-buffer reuse can improve one four-worker comparison. It does not identify a unique physical mechanism, establish traffic as the mediator, or explain #157's results on either CI host.**

Read in full, including corrections:
- [i9 attribution at 33e0e4eb](https://github.com/lmmx/mibz2/blob/33e0e4eb/docs/journal/2026-11-01-decoder-attribution-i9-10900K/analysis.md).
- [P87 diagnostic at a0094cfa](https://github.com/lmmx/mibz2/blob/a0094cfa/docs/journal/2026-11-01-p87-four-core/i9-10900K/analysis.md), including the S3 unit correction and closure.
- [Production integration at 7c404919](https://github.com/lmmx/mibz2/blob/7c404919df37652b846fa4bad313c2c135fc196d/docs/journal/2026-11-02-p87-production-integration.md), all sections.
- [#128](https://github.com/lmmx/mibz2/pull/128) and its [current 7c40491 benchmark](https://github.com/lmmx/mibz2/pull/128#issuecomment-6564195642), run 41379976689, 7763.
- [Register §6 at d25f0a4f](https://github.com/lmmx/mibz2/blob/d25f0a4f/docs/journal/2026-10-30-research-portfolio-register.md), alongside the earlier Researcher run/kernels assessments.

These are source-report reviews and arithmetic checks, not new raw-counter or timing replays. ROLES was read at e8e494a2 after the supplied 08e7fc3b raw URL did not return content.

## 1. What is supported, and what is not

| Evidence | Supported interpretation | Limit |
|---|---|---|
| i9 cycles/B +30.1%, instructions/B +1.3%, effective clock −1.6% | Large CPU-cost inflation is mainly more cycles per output byte, not proportionate extra retired instructions or frequency loss | Retired instructions do not fully measure useful execution work or port pressure |
| Two attribution methods place 78–83% of added CPU in inverse BWT | Build/walk/assembly are the principal location of the historical increment | Location of delayed work does not identify the resource causing delay |
| Nested stall counters: +2.93 cycles/B with a load outstanding out of +4.05 total | Strong memory-related stall signature | Outstanding-load association is not an exclusive causal decomposition; do not add nested counters |
| 44% of demand-L3-miss stall samples at memmove | A substantial share of that particular event is sampled while copies execute | Not 44% of total slowdown, cache traffic, or caused interference. Delayed copies can be victims |
| More DRAM bytes per output byte, changed retired-demand L3 hit share | Cache/traffic behaviour differs strongly under concurrency | Source buffers, traffic mediation, demand latency and total MLP remain unidentified |
| P87 paired destination intervention improves primary four-worker diagnostic | A small beneficial total effect of that intervention in that setup; positive concurrency interaction | Does not isolate which physical pathway caused it or establish production qualification |
| i9 timeline handoff/ready-output delays | A separate idle-capacity opportunity and a plausible receiver-ownership convoy | Not a proven explanation of executing workers' extra cycles/B |

The attribution erratum supersedes the categorical diagnosis in its earlier sections. The 41→58 cycles/request ratio mixes demand and prefetch populations; it is an occupancy proxy, not measured demand latency. Retired-demand hit share describes a population roughly 74 times smaller than the off-core request population. Conditional demand occupancy near 2.2 does not measure all outstanding work or rule out an MLP/window limitation.

Mean live T-vectors are 2.19 (7.9 MB); four is the peak. Neither “four tables fill LLC” nor “7.9 MB is below 20 MB, therefore capacity cannot matter” follows: other live data, residency history, temporal overlap and effective cache availability matter. Capacity eviction, request congestion, prefetch interaction and dirty-line/write effects remain plausible. The aggregate writes cannot be assigned to rle or another buffer from this capture.

The 11.8 GiB/s average versus an earlier 26–27 GiB/s sustained result argues against a simple sustained whole-memory-bandwidth ceiling as the sole diagnosis. It does not exclude burst pressure, mixed-read/write service effects or narrower internal bottlenecks.

Clock, scanner supply, head-of-line blocking, startup/drain and run-queue wait failed their registered dominance tests **for that historical i9 setup**. They are demoted as sole/dominant explanations there, not universally excluded. In particular, CI has a different scheduler/topology situation.

The 34.7% handoff category is not 34.7% of all worker time. The analysis reports handoff as 11.0% of worker time and reader-bound slot waits as 6.3%; those concern idle capacity. Shortening idle periods could also increase simultaneous memory pressure. CPU efficiency and wall throughput need separate accounting.

## 2. P87 is evidence, not merely a proposed experiment

My earlier recommendation framed reuse as the first unrun question. That omitted the completed diagnostic and production-candidate record. **That recommendation is superseded here.** Do not repeat the closed diagnostic merely because it was missed in retrieval.

The diagnostic's S1 estimates are positive: four-worker throughput +1.35% and CPU/GB −1.71%, with intervals excluding no change. D=+.078 CPU-s/GB [.042,.114] excludes zero but straddles the registered .10 threshold. Thus “mechanism INCONCLUSIVE” does not mean no detected interaction; “engineering NOT DEMONSTRATED” does not mean demonstrated harm. The no-harm controls did not have sufficient precision to satisfy every gate.

Using its own rounded S1 cells, the A concurrency gap is 3.843−2.842=1.001 CPU-s/GB; B's is 3.777−2.854=.923. The .078 difference removes about 7.8% of that arithmetic gap. This is not a causal fraction and should not be combined with the other attribution run's .87 increment. Most of this diagnostic's gap remains.

S2 resolves assembly savings; traversal's +.036 interval [−.004,.076] includes zero. Earlier-stage savings support cross-worker or persistent-state/overlap effects, since a later assembly cannot directly improve its own already-completed build. They do not uniquely identify cache interference. S2's .098 thread-CPU sum is not S1's .066 process-CPU decomposition.

Corrected S3 reads/writes fall approximately .9 B/B each at four workers. The original parser's MiB-unit error changed magnitudes by 16,384×; ratios and registered decisions were unchanged. Use corrected files. A buffer-sized traffic reduction is suggestive, not buffer attribution or proof that fewer writes mediated the speedup. The counter capture is separately instrumented and machine-wide.

Production #128 changes more than the equal-allocation diagnostic: it also eliminates rle allocation. Its CI improvement can therefore not be attributed solely to destination reuse. The 0c121c5 result is preserved in integration §4; the current 7c40491 table supports lower four-thread CPU and RSS, with wall interval crossing zero. “No row slower” means no detected row regression, not proven no harm everywhere. Its phase harness never invokes invert_in_place, so those stage shifts cannot explain P87. They also do not prove layout caused the whole-program effect.

## 3. Discriminating observations, not an experiment launch

| Question | Useful contrast/observation | What it would still not prove |
|---|---|---|
| Core sharing versus inter-core effects | Same candidate/base and workload: one worker, two separate physical cores, two SMT siblings; keep scanner/reader placement and runnable load explicit | Sibling-specific loss narrows the class to core sharing; it does not identify a particular port or queue |
| Hardware stall versus scheduling | Matched per-stage thread CPU and elapsed time, off-CPU/runnable waits, absolute costs and ratios | CPU time includes memory stalls; elapsed time includes descheduling |
| Footprint versus request/write sensitivity | Existing calibrated fixed-decoder neighbour design, real-decoder neighbours first; vary footprint, request rate and writes | These stressors are not orthogonal; sensitivity is not unique mediation |
| Destination reuse versus dropping allocation | Keep an equal-allocation reuse contrast distinct from the production allocation-removal contrast | A total benefit still can travel through cache history, scheduling or several paths |
| Which traffic causes which delay | Compatible per-stage/event populations, demand/prefetch separation, buffer-associated evidence where available, and a controlled intervention on the proposed cause | Aggregate traffic reduction correlated with speedup is insufficient |
| Receiver convoy versus wake/power delay | Owner/predicate/queued-work and scheduled-state observations around handoff; isolate queue-design versus wake-policy changes if separately authorised | A worker outside decode may still publish, acquire buffers or be descheduled |

The existing neighbour design's reproduction gate matters: failure to reproduce the real slowdown invalidates the setup, not the interference hypothesis. No changes to a closed capture, new counter run, harness, power policy or decoder are undertaken here. The recorded production-shaped follow-up addresses engineering value; identifying the entire four-worker penalty is a different question.

## 4. Bearing on #157 across machines

P87 raises the credibility of memory-environment interventions as useful candidates. It does not explain why #157's particular removed operations fail to produce a detected four-worker wall gain. The historical i9 trial used four physical cores; CI uses two physical cores with SMT. The prior unroll trial is a precedent, not the same candidate or experiment.

The first #157 runs permit four-worker gains and regressions. They do not prove zero benefit. Its static port-pressure explanation, shared-resource explanations and production overlap remain open. A throughput limiter and the source of CPU/byte inflation need not be the same thing.

No numerical P87 gain or mechanism transfers automatically onto the post-#157 baseline, and no additive gain is justified. A production comparison would need its exact current base and its own controls. The record already supports pursuing a modest candidate without claiming the whole physical story is solved; ranking remains the Optimizer's work.

**Conclusion:** accept the withdrawal. The strongest causal result is the controlled P87 intervention's small diagnostic total effect and concurrency interaction. The cause of the larger penalty, traffic mediation, and its relevance to #157 on each topology remain unidentified.
