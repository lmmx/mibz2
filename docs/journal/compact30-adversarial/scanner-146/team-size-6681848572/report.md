# Team size: a useful policy screen, not yet a mechanism experiment

Researcher · 2026-11-06 · response to [inbox 6681848572](https://github.com/lmmx/mibz2/pull/162#issuecomment-6681848572)

Instruction pin: `0479b380f5d6309f3fd0f23e0ad6dfade4b7a5c0`.
Registration/source reviewed: `bb9291abc6ec353fd07c0cba00a6fa8eaba61c15`, PR #219.
This is a source and literature review. No benchmark, implementation, merge, or new authorization.

## Decision

The fixed-team comparison is worth doing. It addresses an important gap left by the October experiments: four tables with more than two threads each. But the four proposed outcome readings are neither exclusive nor identified by the registered fields. Preserve the throughput/resource decision; describe causal readings as hypotheses requiring discriminators.

**First correct a concrete confound: the reference has 16 in-flight slots, while every four-team fixed arm has 8.** The supposed same-plan noise control is not configuration-identical. **The most valuable diagnostic addition is a per-block phase/wait timeline with verified placement, ahead of system-wide memory-controller counters.** Add shared request/service capacity to the competing explanations: unchanged miss counts do not rule it out.

## 1. The reference changes more than team size

The following follows directly from the registered source:

1. [examples/ab.rs, decode](https://github.com/lmmx/mibz2/blob/bb9291abc6ec353fd07c0cba00a6fa8eaba61c15/examples/ab.rs#L431) invokes `BzReader::parallel` for `mibz2`.
2. [src/read.rs, parallel](https://github.com/lmmx/mibz2/blob/bb9291abc6ec353fd07c0cba00a6fa8eaba61c15/src/read.rs#L87) supplies `SLOTS_PER_THREAD * threads`; [src/parallel.rs](https://github.com/lmmx/mibz2/blob/bb9291abc6ec353fd07c0cba00a6fa8eaba61c15/src/parallel.rs) sets that constant to **4** and `window` scales slots by teams/requested threads.
3. At default `mibz2@8`, four teams therefore receive **16 slots**.
4. [ab.rs, fixed arms](https://github.com/lmmx/mibz2/blob/bb9291abc6ec353fd07c0cba00a6fa8eaba61c15/examples/ab.rs#L446) supplies `2 * plan.workers()` to `parallel_planned`. Here `workers()` means teams. Thus `coop2@8`, `coop3@12`, and `coop4@16` each receive **8 slots**.

Consequences:

- `coop2@8 / mibz2@8` is a window intervention as well as an entry-point comparison. A reproducible difference is not necessarily noise.
- Candidate/reference throughput, CPU and RSS include this window difference. Reduced buffering can lower RSS or change scanner backpressure independently of k.
- Ratios between the fixed four-team arms do hold the slot count fixed, and remain useful.
- Before the run, prefer matching the window to the production reference for all fixed arms. Alternatively retain the original cells as bundle comparisons and add a matched-window reference; preserve the registration/amendment chronology. If results already exist, do not silently relabel them.

This is separate from table capacity. Four allocated T-vectors do not establish four simultaneous active walks; phase overlap, short-block fallbacks and scanner starvation require observation.

## 2. What the fields can and cannot discriminate

| Proposed reading | What the registered evidence could establish | What is missing for causal attribution |
|---|---|---|
| More threads on the same tables is useful | A reproducible whole-decoder advantage of a particular configuration, subject to matched settings, hashes and resource gates | Pure k attribution requires matching slots; actual concurrent phases and k-dependent work must be acknowledged |
| SMT siblings are the cost | Larger requested teams perform differently under the OS scheduler | Actual topology/placement and a matched intervention on sibling sharing |
| Leader-only work binds | Leaders consume more aggregate CPU than individual helpers | Critical-path phase durations, per-block work and waits; CPU totals alone do not locate a throughput bottleneck |
| Helpers add cache pressure | Retired-load hit/miss composition changes with the configuration | Which phase/address population changed, and whether capacity, prefetch timeliness, placement or request contention caused it |
| Shared request/service capacity binds (missing alternative) | Little speedup despite more runnable work, potentially with little change in miss volume | Phase-scoped latency/outstanding-request evidence and controlled placement/concurrency |

**SMT.** Twelve team threads on ten cores require some sharing or scheduling if all run simultaneously. That does not specify which threads share, when, or how much. Even ten unpinned team threads are not a no-sibling control; scanner, feeder, hashing/reader and sampler also exist. `plan_with` counts physical cores; it does not bind threads to them. Record the allowed CPU topology and placement/migrations. For a later discriminator, compare an identical feasible G×k configuration spread over physical cores versus deliberately sharing siblings, controlling support-thread placement. Three teams of three versus three teams of two fits within ten cores but still needs verified placement. Such a diagnostic complements the unpinned user-facing result; it does not replace it.

**Leader versus helper.** [Pool::run and helper](https://github.com/lmmx/mibz2/blob/bb9291abc6ec353fd07c0cba00a6fa8eaba61c15/src/coop.rs) spin before parking, synchronize through task/done state, and wait for the slowest participant. A helper's CPU includes useful build/walk work and synchronization; parking consumes wall time without comparable CPU. A leader's CPU combines its shared inversion contribution, serial stages, and waits/spins. More helper CPU with unchanged leader CPU therefore does not prove that the leader-only phases bind. Compare total helper CPU per team as well as per-helper averages: adding helpers changes the denominator.

The minimal informative diagnostic records block ID and rows, team and phase, phase wall duration, participant active CPU, and completion skew/wait. Separate:
- waiting for a block or scanner output;
- leader-only pre/post-inversion stages, including assembly;
- shared build and shared walk;
- useful traversal, synchronization spinning and parked time;
- time the leader waits for the final helper, and helper idle intervals while the leader finishes a block.

A generic “helper idle inside Cooperative” total is better than nothing but still merges several causes. Use a separate diagnostic pass and disclose instrumentation overhead; no per-row clocks are needed. Count cooperative/fallback blocks and actual plan rather than inferring all phase activity from names.

**Cache metrics.** Retired-load L3 hit/(hit+miss) is a fraction of that event population, not the residence fraction of T or a complete traffic account. Prefetching can alter where a later demand load hits. The whole-process counters mix build, walk, output, queues and synchronization. More threads can contend for shared service/request capacity without increasing DRAM-miss volume. A stall count taken while a miss is outstanding is not an exclusive causal attribution to that miss, and summed thread cycles are not elapsed decoder time.

Keep misses/output-byte and cycles/output-byte (already present), retain each reading and its coverage, and show instructions/output-byte and uncertainty rather than only pooled ratios. To choose the next mechanism test, phase scoping and waits come first. If the evidence then points to off-chip pressure, IMC read/write traffic can help: retain interval duration, baseline/background traffic and comparable workload conditions. It is system-wide, not uniquely this decoder's traffic; bandwidth below a streaming benchmark does not exclude latency or request-capacity limits. No sudo/system-wide collection is required merely to decide whether a configuration wins.

## 3. Prior work changes the explanation, not the policy gate

**Direct BWT prior art.** Kärkkäinen, Kempa and Puglisi, *Slashing the Time for BWT Inversion*, DCC 2012, §3, explicitly interleave independent reconstructions on **one execution thread**, exploiting out-of-order overlap; their split positions are assumed available in the compressed file. This supports separating cache-miss cost from miss count. It is not evidence for linear scaling of OS threads on one bzip2 table. The registered [bwt.rs](https://github.com/lmmx/mibz2/blob/bb9291abc6ec353fd07c0cba00a6fa8eaba61c15/src/bwt.rs) already cites it and uses 32 flagged lanes per participant. Recommending multiple chains as a new construction would rediscover existing work.

Source: [author PDF](https://www.cs.helsinki.fi/u/tpkarkka/publications/dcc2012.pdf), §3, especially p.5.

**Closest useful counterexample: AMAC.** Kocberber, Falsafi and Grot, *Asynchronous Memory Access Chaining*, PVLDB 9(4), 2015, §5.1, study prefetched pointer walks. On their Xeon, throughput levels off before all physical cores are used; off-chip access counts change little. A two-socket placement intervention supports their shared-LLC request-resource explanation. Their SPARC results differ, including some SMT benefit. Thus “miss count unchanged, so serial code or SMT must explain the plateau” is not a valid elimination. Their hardware limits and speedups are not transferable measurements of the i9.

Source: [paper](https://www.vldb.org/pvldb/vol9/p252-kocberber.pdf), §5.1.1 and Table 4.

**libsais as a comparator, with a boundary.** Its official README documents parallel inverse BWT and recent improvements, but the familiar memory-channel/thread-saturation guidance explicitly discusses suffix-array construction. Do not use its “near eight threads” guidance as a measured inverse-BWT ceiling for this decoder. It motivates implementation comparison, not a numerical prediction.

Source: [official repository](https://github.com/IlyaGrebnov/libsais), README “Multi-threading” and changes, consulted 2026-11-06.

**Project history matters more for this prediction.** The [October 29 decoder record](https://github.com/lmmx/mibz2/blob/bb9291abc6ec353fd07c0cba00a6fa8eaba61c15/docs/journal/2026-10-29-cooperative-decoder.md) reports 4×2 versus 3×3 and losses from increasing team count; it does not measure 4×3 or 4×4. It also records unpinned-placement and wake-up risks. This makes the present fixed-table question new evidence, not a rejected experiment repeated unchanged. Its claim that the L3 cap explains the losses remains broader than those throughput comparisons alone identify.

A further source-level confound is intentional: `Cooperative<K>` calls `walk_room::<32>` with **512×K segments**, then assembles on the leader. Increasing k changes aggregate lanes, segmentation, handover/bookkeeping and scratch organization, as well as OS concurrency. It holds T size fixed, not all work and auxiliary footprint fixed. If balance/tail observations later justify it, a fixed-segment-count comparison would separate scheduling benefit from the changed decomposition; this review does not implement one.

## 4. Prediction and next measurement

The registered 1.00–1.12 for 4×3 and 0.95–1.08 for 4×4 are reasonable subjective scenarios, not calibrated confidence intervals. The cited leader/helper CPU totals do not identify the serial wall fraction, and there is no principled zero-regression floor for 4×3.

For a deliberately idealized fixed-work, fixed-placement team with serial wall S and perfectly divisible work P,
`T(k)=S+P/k`. If f is the parallel fraction of **baseline team wall time**, ideal 2→k speedup is
`1 / (1-f+2f/k)`. f is not the fraction of CPU samples in inversion, nor inferred by subtracting aggregate helper CPU from leader CPU. Real synchronization, shared resource contention and scanner/reader bottlenecks can invalidate this ideal gain.

Recommended order:
1. Resolve the window mismatch and record expected versus observed configuration. Keep the existing whole-decoder decision threshold.
2. Run/read the registered throughput screen; conclusions concern configurations and this corpus/host.
3. If a mechanism decision is needed, take a bounded phase/wait and placement diagnostic on 4×2 and the relevant candidate, plus a feasible physical-core comparison.
4. Add IMC or further request/latency diagnostics only if those observations make their answer decision-relevant.

No failure pattern in the current four-way table alone warrants declaring SMT, serial stages, or cache capacity the unique cause. A gain still warrants replication and resource evaluation without waiting for a complete microarchitectural explanation.

## 5. Measurement/reporting checks before interpreting results

These are source-review findings in [team_size.py](https://github.com/lmmx/mibz2/blob/bb9291abc6ec353fd07c0cba00a6fa8eaba61c15/bench/team_size.py), [ab.rs](https://github.com/lmmx/mibz2/blob/bb9291abc6ec353fd07c0cba00a6fa8eaba61c15/examples/ab.rs), and [.just/bench.just](https://github.com/lmmx/mibz2/blob/bb9291abc6ec353fd07c0cba00a6fa8eaba61c15/.just/bench.just), not observed run failures:

- `plan_of` rounds the helper/leader ratio; the report flags varying plans but does not enforce the expected plan or divisibility. An identically wrong plan can pass that presentation. Validate exact leader/helper counts per run and all expected rounds/cells; dictionaries currently overwrite duplicate cell/round rows and pairing uses whatever common rounds remain.
- Counter collection discards child JSON and checks exit status. `ab child` serializes decode errors into `err` and normally returns success; the shell's `|| exit 1` does not catch those errors. Validate the counter-pass output/hash/error itself before normalizing counts by timed-pass bytes.
- Counters cover shell/startup/whole children including the sampler, whereas timed CPU subtracts sampler CPU. Do not equate the populations. Three requested general events is not proof of complete hardware coverage: inspect unsupported/not-counted results and running fractions on the actual CPU. The script preserves non-100% coverage in JSON but does not display it in the Markdown table. Preserve raw perf output, tool version and event resolution.
- The just recipe runs counters **after the entire timing run**, not after each timed round. Three counter readings per cell are not twenty paired timing rounds, nor five independent within-reading replications. Do not correlate their pooled values as if synchronous with each timed round.
- “Two 95% comparisons imply about a one-in-ten chance one passes” overstates this directional rule under ideal null calibration: requiring a two-sided 95% interval's lower bound above one is a 2.5% upper-tail event per comparison. Bonferroni gives at most 5% for two; independence would give 4.94%, before the additional ≥3% effect floor. The shared reference induces dependence. This does not solve selection bias or dependence between rounds; independent replication remains valuable.

For counter semantics, consult the [Intel event guide](https://cdrdv2-public.intel.com/671378/335279-performance-monitoring-events-guide.pdf) and the installed CPU's event definitions; this review does not certify a Comet Lake PMU encoding or actual counter availability.

## Evidence and scope

Read the pinned registration, benchmark collector/report, command recipe, decoder entry points, scheduling policy, cooperative pool/walk, BWT dispatch, and the October 29 measured report. External primary sources above were read for methodological relevance. Historical measurements are quoted from the project record, not independently rerun. The window arithmetic and collector behavior were reconstructed from source; no native tests or timing were performed. Advice only: no implementation or policy acceptance is granted.
