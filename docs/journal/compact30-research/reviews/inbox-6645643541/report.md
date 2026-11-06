# Four-worker opportunity ranking at master 08e7fc3b

Trigger: [inbox 6645643541](https://github.com/lmmx/mibz2/pull/160#issuecomment-6645643541), resuming [6645304470](https://github.com/lmmx/mibz2/pull/160#issuecomment-6645304470). This replaces the requested ranking from the six superseded #145 checkpoints; it does not process them as separate assignments.

Baseline: **08e7fc3b46c7a6adb61ce0e70b64e9973b95cbb7**. Date: 2026-11-05.
Scope: read-only source/listing inspection, source-report review and arithmetic. No implementation, compilation, timing or new experiment. No fresh raw-profile replay or independent qualification is claimed.

## Recommendation

**First: the production-shaped P87 rebase, with its own current-baseline validation and unchanged gates.** Its existing positive diagnostic and production CPU/RSS observations make it the strongest near-term engineering candidate, even though its historical diagnostic did not qualify. This is a recommendation, not permission to time, promote or reopen a closed diagnostic. Do not rebuild the closed equal-allocation experiment.

Then **true tiled B**, **handoff diagnosis**, **gather C**, **smaller table at production cost**, and **Precopy deferred**. This is risk-adjusted research priority, not a descending list of guaranteed percentage gains. None earns new verified instruction credit.

The four-worker limiter is **not identified uniquely on either topology**. On the historical i9 four physical cores, inverse BWT accounts for 78–83% of the CPU-cost increment and memory-related stalls accompany it. On CI, four logical CPUs share two cores; contention within each core is an additional hypothesis. Neither transfers a unique cause to the post-#157 decoder. No detected four-worker gain is not proof of zero benefit. The November 1 unroll is a close precedent, not #157's identical experiment.

## Pins and evidence

Read the decisive reports including corrections and closure:
- [Researcher causal assessment, 3843e7da](https://github.com/lmmx/mibz2/blob/3843e7da/docs/journal/compact30-adversarial/scanner-146/causal-scope-6644912625/report.md).
- [Attribution, 33e0e4eb, including erratum](https://github.com/lmmx/mibz2/blob/33e0e4eb/docs/journal/2026-11-01-decoder-attribution-i9-10900K/analysis.md).
- [P87 diagnostic, a0094cfa, §§1–6](https://github.com/lmmx/mibz2/blob/a0094cfa/docs/journal/2026-11-01-p87-four-core/i9-10900K/analysis.md).
- [Production integration, 7c404919](https://github.com/lmmx/mibz2/blob/7c404919df37652b846fa4bad313c2c135fc196d/docs/journal/2026-11-02-p87-production-integration.md).
- [Register, d25f0a4f, §6 and P04](https://github.com/lmmx/mibz2/blob/d25f0a4f/docs/journal/2026-10-30-research-portfolio-register.md). Its older P87 “not run” entry is superseded by the diagnostic above.
- [B2 model and kernels, 9723f068](https://github.com/lmmx/mibz2/blob/9723f068/docs/journal/2026-10-30-b2-gathered-walk.md).
- [#157 block accounting and listing index, b0f6e89d](https://github.com/lmmx/mibz2/blob/b0f6e89d/docs/journal/compact30-channel/evidence/walk-75b13f00/README.md).
- [#155 accounting, c493a45d](https://github.com/lmmx/mibz2/blob/c493a45d/docs/journal/compact30-channel/evidence/port-b4786f66/README.md).
- Current source: [bwt.rs](https://github.com/lmmx/mibz2/blob/08e7fc3b/src/bwt.rs), [parallel.rs](https://github.com/lmmx/mibz2/blob/08e7fc3b/src/parallel.rs); [published MLP synthesis](https://github.com/lmmx/mibz2/blob/08e7fc3b/docs/site/blog/2026-11-02-memory-level-parallelism.md). Read the latter's “nothing” language through D021's narrower no-detected-gain interpretation.

The current commit's diff is test instrumentation and tests. The archived #157 release listing is evidence for that production construction, not a newly compiled 08e7fc3b binary. I inspected its setup and loop layout alongside current source: table loads, lane row state, byte stores and prefetch remain; source advances eight lanes and writes each byte immediately. B2 B instead buffers eight successive steps per lane and replays failed tiles. These are materially different.

## Whole-decoder accounting convention

For a block b, saving is I_base(b) - [I_build(b)+I_walk(b)+I_rest(b)]_candidate. Include allocation, zeroing, copies, replay, tails, dispatch and fallback; no removed hot-loop operation is credited twice. Corpus totals require summing actual block frequencies. Historical 434-block counts and the independent 473-block evidence corpus must not be mixed.

For elapsed time, a conditional unchanged-rest model is ΔT/T = f_walk × (r_walk - 1), with candidate/base ratio r and *current, matching* walk fraction f. It is a sensitivity equation, not an instruction-to-cycle conversion. For example f=.4 and r=.9 would yield -4%; neither input is established here for current four-worker execution.

## 1. P87: reuse ll for output

**Expected effect:** a modest beneficial engineering effect is plausible; there is no defensible current-master confidence interval or guaranteed instruction saving. Historical equal-allocation i9 diagnostic: throughput +1.35% [+0.29,+2.41], CPU/GB -1.71% [-2.37,-1.06]. Production CI: 0c121c5 w4 wall -1.9% [-3.2,-0.7]; 7c40491 w4 wall -0.7% [-2.1,+0.7], CPU -1.4% [-2.5,-0.3], RSS -6.2% (latter table as checked in the pinned Researcher assessment). These are priors, not additive or transferable gains.

**Frequency/accounting:** applies to each Segmented inversion, including its plain short-block path; ll remains available for output after build. Removing rle allocation primarily removes worker allocation/growth events, not one fresh allocation per block. Copy destinations change; byte copies are not thereby eliminated. Whole-program instruction ratio was near unity in the diagnostic. Do not book a large instruction saving.

**Prior verdict:** already tested positively but mechanism INCONCLUSIVE and engineering NOT DEMONSTRATED; all five no-harm cells inconclusive. D=.078 CPU-s/GB [.042,.114] excludes zero but crosses .10. This is not “unrun” and not “proven harmless.” Production drops allocation too, so it tests a different intervention.

**Next evidence/falsifier:** pinned current-master production comparison, byte-exact controls and adequately powered no-harm criteria registered before measurement. Refute its engineering priority with reproducible control harm or failure to meet the prospective net benefit criterion; an imprecise interval means inconclusive. CI can test two-core SMT engineering value, not the i9 four-physical-core effect. Keep the closed diagnostic and its gates unchanged.

**Safety/equivalence:** build is ll's final read and must copy all required symbols/links into T before overwriting ll. In-place success preserves pointer/capacity/length and produces exactly separate-output bytes. Unsupported kernels must return false without changing input or scratch. Include periodic/degenerate blocks, short blocks, errors followed by reuse, malformed input and allocation accounting. Borrow checking alone does not prove algorithmic last use.

## 2. True tiled B

**Expected effect:** unquantified on current master, with both gain and regression plausible. Old B2's 3.30–3.42 versus 4.49 c/B and -10–11% decoder-time model used the old 14.8-macro-op baseline. It cannot be advertised against #157's 9 instructions/step plus four/eight-lane turn. Macro-ops and instructions are different units.

**Frequency/model:** only successful full tiles obtain bulk commit savings. For E eligible rows and measured candidate cost u_B, price E×(u_current-u_B) minus added setup, failed-tile replay, tail and changed rest. Count E and failures; the old mean segment length 1,758 and 0.05–0.17 c/B replay are assumptions, not current frequencies. Both kernels still make one random T load per row. Any claim of fewer misses needs evidence.

**Prior/new evidence:** not #157, not a rejected B2 experiment. It shares instruction-reduction rationale with the negative four-worker unroll precedent. Distinct bulk stores and amortised tests could change that verdict, but only generated code and whole-block costs establish it.

**Falsifier:** current master versus true B at one copy, two separate cores and an SMT sibling pair, plus full decoder/control results. The optional old A reference permits the original B2 model's thresholds to be tested at its own boundary. B/current ≥.95 would reject a proposed ≥5% improvement in that cell; it does not by itself prove a miss cap or close every fewer-op construction. CI can expose those placements if affinity/topology are verified, but has no PMU guarantee and cannot emulate four physical cores.

**Safety:** mask START before any speculative successor access; links remain in range. Reserve full tile space before a wide store. Keep speculative bytes private until acceptance. Replay only valid prefix before the first flag from saved row; preserve ordered segment ownership, next links, queue exhaustion and tail pointers. One/two-row cycles, several simultaneous flags, exactly-full buffers and a flag at each of eight offsets are essential counterexamples. Spills and unaligned wide-store legality are explicit assembly obligations.

## 3. Steady handoff: investigate before choosing a queue rewrite

**Expected effect:** no numeric current throughput prediction. It changes work outside decode_block; direct decoder instruction saving is zero by construction, with queue/copy overhead charged to whole-process totals. Historical 34.7% is a share of Q2 idle, not all time. Handoff is about 11% of worker time. Converting all that to useful work at unchanged cost gives only an illustrative capacity ceiling 11/70≈15.7%, not a predicted throughput gain. Do not add reader's 6.3% or CPU inflation savings.

**Frequency/prior:** once per job plus waiting/wake transitions; benefit only where queued work waits while a usable worker cannot claim it. Current worker source still holds Mutex<Receiver> over blocking recv. The 980µs versus 62µs conditional delays fit a convoy but do not prove one.

**New evidence/falsifier:** observe owner, queued work, predicate and scheduled state. A separately authorised bounded multi-consumer queue or explicit dispatcher proposal must preserve one job claim, cancellation/drain, ordered publication and bounded memory. If delays are mostly off-CPU or publishing/buffer work, or queue change removes claim delay without improving wall throughput, the claimed bottleneck is refuted. CI can test its own queue behaviour; power-state conclusions and i9 effects require the appropriate host and separate authority. This ranks above gathers for diagnostic value, not as an implementation order to rewrite concurrency immediately.

## 4. Gather C, only after B

**Expected effect:** unknown current whole-decoder delta. Old model C-B saved 0.13–0.89 c/B on one 7763 calibration; not a transferable four-worker estimate. Apply the unchanged-rest equation only after B and C are measured at matching boundaries. Count vector packing, mask resets, extracts, commits, spills, replay and longer tail.

**Prior/new evidence:** C is unbuilt. It is not LF² (#29), which added an n-random-read construction pass and doubled table size; C adds neither. It remains vulnerable to gather serialisation and slowest-element latency, with the gather unit shared under SMT.

**Falsifier/topology:** compare C to true B, not just old A; reject the extra complexity if it cannot meet a predeclared incremental gain or regresses controls. The historical C≤B-.05A target belongs to its historical baseline. CI can compare AVX2 variants and sibling placements if feature support is verified, but timing ratios cannot identify a hardware queue. Four physical cores need the other topology. Reuse B's proofs, with valid indices/masks for every gather element and feature-gated scalar fallback; an inactive lane must not cause an invalid access.

## 5. Smaller table at production walk cost

**Expected effect:** the existing #155 port remains negative. Smaller storage by itself has no credited speedup. On archived full block 1 the new comparison is:

| Term | Instructions |
|---|---:|
| Post-unroll decode | 20,184,697 |
| Compact build | 4,369,204 |
| Compact rest | 4,953,555 |
| Maximum compact walk at whole-decoder parity | **10,861,938** |
| Existing compact walk | 20,207,700 |
| Required walk reduction | **9,345,762 (46.25%)** |

At 899,971 rows this is **12.0692 instructions/row**, not the stale 18.26. With the archived compact build/rest unchanged, its 29,530,459 total is about 46.3% above the post-unroll 20,184,697. This is a cross-archive conditional budget, not a new current build or corpus result. The archives' old baseline totals differ by two allocator instructions.

**Frequency/prior:** row work on full blocks, plus all tiny-block/setup costs. #155 trades one packed entry load for offset and group-header work and already lost on CI. A genuinely new representation must remove enough of that dependent reconstruction, not merely trim constructor instructions. At walk parity with current 9,859,793, the archived net saving would be 1,002,145 (~4.96%) on this block; this is a conditional target, not a reachable result.

**Falsifier/topology:** reject on whole decode instruction growth or failure of the prospective production performance gate; count full corpus and controls. CI can test engineering, but cannot establish the four-core memory hypothesis. A viable proposal needs exact byte/permutation/header/PARK/RAW/reuse proofs and new listings; none is supplied by the existing rejected port. A6's constructor credit is not added to production savings.

## 6. Precopy

**Expected effect:** negative prior at four workers, no new positive forecast. Forward chains copied 31.5% yet increased CI wall about 18%/6% at one/four threads; bookkeeping touches walked rows and removes relatively cheap cached steps. New post-unroll work reductions make that overhead harder to repay.

**Frequency/model:** ΔI = copied_steps×avoided_step_cost - per_walked_row_bookkeeping - chain_build - copy/setup costs, weighted by actual copyable chains, not the offline 65.1% census. Preserve overlap-safe copies, dependency ordering and malformed-input handling.

**Prior/falsifier/topology:** rejected; reopening is recorded at 8–10 workstation workers after the prior conditions, not ordinary four-thread CI. Only a changed cost construction or that separately authorised condition could alter the verdict. CI cannot test 8–10 physical-worker scaling. Do not reopen it for this brief.

## Causal and accounting safeguards

41→58 cycles/request mixes demand and prefetch populations and is not demand latency. Mean live tables are 2.19, not four; aggregate writes have no identified source buffer. Nested stalls cannot be summed. P87's corrected S3 traffic falls ~.9 B/B each way, but traffic mediation is unproved. S2's .098 thread-CPU sum is not S1's .066 process-CPU decomposition; production phase tables do not call invert_in_place.

Broader alternatives remain subordinate to evidence: scanner/MTF/CRC work and scheduling must be priced, but historical scanner supply failed its dominance test and handoff is the strongest separate idle lead. An LF² pass or another compact constructor is not justified by eliminating operations in isolation. No novel high-confidence construction outranks production P87 in the available record.

This ranking changes no historical verdict, no gate and no reserved authority. It is not independent qualification of any future implementation shaped by these proposals. Current-master net instruction and four-worker performance evidence remain required before any CONDITIONAL row becomes ACHIEVED.
