# Two-modes discriminator: REVISE before freeze

Researcher review of inbox [6757368708](https://github.com/lmmx/mibz2/pull/162#issuecomment-6757368708), dispatching [6757313827](https://github.com/lmmx/mibz2/pull/145#issuecomment-6757313827). Exact PR318 head: `3588d35dc9b2f2f4d4709cf37c4a4bee2afa9dff`. Rules/spec pin: `a2da49638ac54d7763de1c167bd054f67ffdc763`. Research only; no freeze, run, implementation, integration or authority change.

**REVISE the instrument and the interpretation, retain the diagnostic.** Byte-identical archived executables, explicit nine-core intervention, all records retained, and a sustained observation preceding the short runs are useful choices. But the current judge can erase a twofold progress loss, call sleeping tasks busy siblings, and label constant throughput alternating. Missing evidence can create a unique hypothesis match. This is not ready to name a mechanism.

The following counterexamples use the exact Python source and its synthetic fixture builder, not hardware timing. `probes.py` reproduces them; `probe-results.json` preserves the output. The source's **44 tests pass**; they do not test these distinctions. No target timing was run.

## 1. Progress is not the median rate of fixed-work intervals

`two_modes.py:549–625` computes each interval's rate and takes an unweighted median. Each interval represents the same work, not the same elapsed time. For work K and elapsed ticks d_i, elapsed-time progress is `n*K*Hz/sum(d_i)`, not `median(K*Hz/d_i)`.

Construct 700 intervals, K=2^20, Hz=3.7e9, d_i=808277. Add 700*808277 ticks to one interval:

| Reading | Uninterrupted | One long pause |
|---|---:|---:|
| Span, seconds | 0.1529172703 | 0.3058345405 |
| Current median, Giter/s | 4.80000198 | 4.80000198 |
| Total work / elapsed time, Giter/s | 4.80000198 | 2.40000099 |

The central statistic remains full while average progress halves. A pause crossing a decode-window boundary can also be discarded altogether by `probe_rates`' whole-interval selection. This is an estimator counterexample, not evidence that the i9 actually pauses this way.

Before freeze: retain interval medians as active-loop summaries, but use elapsed-time progress for the duty-sensitive reading; define boundary/censoring handling, coverage and longest gaps. Fixed-time windows or summed work/ticks over a documented common interior suffice. Record the probe thread's CPU time if distinguishing descheduling from reduced execution progress matters. Invariant TSC supplies a timebase; it does not identify why work stopped. Neither ratio alone establishes H3.

## 2. Last CPU is not current sibling occupancy

`readings` counts all named decoder tasks at their last CPUs in the decisive `sibling_share`, including sleeping tasks. A separate R-only share exists but is not matched. In the supplied H2 fixture, change all tasks on CPUs >=10 to state S. The judge still returns **VALID MATCH H2**, with slow all-task share 0.90909 while fast and slow R-only shares are both zero.

Linux's [proc_pid_stat documentation](https://man7.org/linux/man-pages/man5/proc_pid_stat.5.html) defines field 39 as the CPU last executed on; S is sleeping, R running. The sampled files are also read sequentially, not as an atomic scheduler snapshot. Even two R observations are an occupancy proxy, not proof of simultaneous execution.

Before freeze: stop matching all-task last-CPU coincidence as a busy sibling. At minimum use the R-only proxy, disclose asynchronous sampling, and retain its valid sample count and uncertainty. Do not turn it into measured contention. Proving a placement cause would require a subsequent controlled placement intervention, not a more emphatic label.

The clock row has a related alignment defect: `seen` is the union of every CPU visited across the run, then used for every frequency sample. It is not the CPUs active at that sample as the prose suggests. Correct the alignment or describe the union statistic accurately. [intel_pstate documentation](https://docs.kernel.org/admin-guide/pm/intel_pstate.html) describes per-logical-CPU policies and, for the documented active-mode interface, `scaling_cur_freq` as an average over scheduler updates. That is not an instantaneous decoder-clock oracle; it does not establish the registration's speculative shared-clock premise.

## 3. Input chunking manufactures the sustained signature

The analysis, unlike the collector's 100 ms display windows, classifies 500 ms decoded-equivalent blocks. Using the existing sustained fixture, impose constant **1.00 GB/s** with 4 MiB input chunks. Result: **alternates**, fast share 13/18, **9 switches**. At constant 0.98 GB/s: **alternates**, 11/19, **14 switches**. Individual reconstructed blocks range from 0.89031 to 1.03869 GB/s. There was no switching throughput in either construction.

This already occurs under a constant compression ratio. Real variation in compressed bytes per decoded byte and read-ahead add uncertainty. [procfs documentation](https://docs.kernel.org/filesystems/proc.html) defines rchar in terms of bytes returned by read-like calls; it is not decoded output progress. Multiplication by the whole-corpus ratio does not make every local window an output counter. Discarding the first second and final tail does not remove internal chunk boundaries.

Before freeze: either report this as raw input-consumption trajectory without mechanism-specific steady/alternating labels, or pre-register a defensible error band/window rule and validate it on constant and genuinely switching synthetic trajectories across chunk phases and rates near the threshold. A deadband must derive from the read-ahead/chunk bound and local conversion uncertainty; an arbitrary longer window is not a proof. Keep end-to-end decoded bytes/wall separately. The 0.999 GB/s burst separator is not automatically a calibrated sustained-state separator, especially across nine-core pinned and ten-core unpinned configurations.

## 4. Missing readings cannot be positive identification

`match` admits every column for an unread row. Set all rows to their no-difference baseline, except make the clock unread: `outcome` returns **MATCH H1**. With the full clock present, no column matches. Losing the only discriminating observation manufactured a unique result. The supporting-reading annotation does not repair it.

These columns are predictions under substantial assumptions, not an exhaustive identifiable causal model. Joint mechanisms can violate every column; measurement error or an unmodelled magnitude can do so too. Hence **SIXTH MECHANISM** does not follow from no match. Four fast runs and 36 slow runs likewise do not imply **ONE MODE**. Missing values are removed separately for each row, so five runs per class do not ensure five observations of its decisive row.

Before freeze: label results **compatible with the registered signature(s)**, **no registered signature matches**, or **insufficient class/measurement support**. Explicitly require the decisive observations for a positive signature; preserve hypotheses that cannot be tested, rather than claiming identification by elimination. Report per-row coverage and keep raw values around every cutoff. This is a bounded diagnostic, so no need to build a general causal estimator before collecting useful observations. It does need honest outputs.

Without power-limit events, low clock plus reduced idle-probe progress can describe a low-progress state compatible with H5; it cannot identify power limiting. Conversely absence of a thermal counter increment does not by itself prove no transient throughput-limiting event. Global `thp_fault_alloc` is not attribution to this child. Its own faults and AnonHugePages are useful corroboration, not currently decisive inputs.

The interpretation also overreaches past the table. [Kernel THP documentation](https://docs.kernel.org/admin-guide/mm/transhuge.html) distinguishes reduced first-touch faults from enduring TLB benefits. An H4-compatible signature is not inherently only a short-process phenomenon. Long processes can allocate/fault again; a continuously loaded system need not retain one placement or clock state. Replace the categorical remedies and “long decodes live in the slow mode” with falsifiable follow-up predictions. The sustained observation can challenge a prediction; it cannot validate those implications by itself.

## 5. Evidence admission and bounds are incomplete

The normal recipe enforces useful preflight and source checks. The standalone judge does not reproduce all of them. Starting from the valid H3 fixture, set `inputs_match_frozen=false`, `frozen=null`, `preflight_exit=1`, `sustained_exit=124`, change the after-ab digest, and make probe/corpus before/after digests disagree. The judge returns **rc 0, VALID, problems=[]**.

This does **not** mean the normal non-DRY recipe would execute past its rejected preflight. It means the published re-judgement admits contradictory recorded facts, and recorded post-run binary/corpus digests are not enforced by the judge. Before freeze, define one structured admission contract shared by collection and replay: freeze/source provenance, successful gated stages, before/after identities, required measurement coverage, and complete ordered run/interval containment. Missing or contradictory required evidence must remain REFUSED/INVALID on replay. Test changes one field at a time, not only this combined witness.

The sustained stage has a 300 s TERM deadline and 10 s KILL grace; the passes' `taskset ... ab run` calls do not. The busy gate can wait indefinitely—as the partial dry check already demonstrated—and the wrapper polls child completion without its own deadline. A seven-minute estimate is not a bound. Pre-register pass/whole-attempt deadlines, kill/reap descendants, preserve partial records, and record timeout without retry. This is needed even if the next idle-host dry run succeeds.

The concatenation cache is keyed by copy count and accepted by size. The concatenation's actual digest is not in the identity; sustained admission checks output length, not the expected content hash. A same-size wrong cached input is not excluded. Keep 593 MB outside the bundle, but verify its exact repeated-corpus construction, record its digest and before/after stability, and archive the constituent corpus identity/copy count/reconstruction method. Content-address or validate the cache; length alone is inadequate. A trustworthy content check may use reconstruction and digest comparison without altering the archived decoder.

Finally, registration §5 says the recipe's pass 1 ran. The assignment explicitly says it did not: only three direct wrapper invocations ran outside it. Correct that record and require the already-promised full in-recipe DRY check before freeze. Such a check proves plumbing only, not target discrimination.

## Answers to the seven questions

| Question | Answer before freeze |
|---|---|
| 1. H1 “full or rising”; warm probe? | Keep the per-run cold start if preserving the archived busy gate is the objective. Prewarming is a new intervention that can change clocks, power and mode mix. “Full or rising” is only a conditional compatibility prediction; fix the progress statistic and preserve early/late coverage. An isolated idle core need not track a decoder core. No positive H1 claim with its decisive clock evidence unread. |
| 2. Thresholds .95, .40–.65, .85, .50/+ .15, sys + .05, MIN_CLASS 5? | Unvalidated descriptive cutoffs, not established correct discriminators. Freeze them prospectively only with that meaning, raw values/coverage, and missing-evidence handling. Keep .21 s as the requested descriptive class separator, without moving it after data. MIN_CLASS 5 is a display floor, not evidence of a stable median or absence of a mode. |
| 3. 40 probed/20 unprobed, PPN? | Enough for a small descriptive attempt, not to establish probe neutrality. At iid fast probability .21, even 40 probed runs have P(fewer than 5 fast)=.05695; persistence worsens the effective support. Worst-case iid SE of a 40-vs-20 proportion difference is .13693. Fixed PPN aliases a period-three nuisance perfectly: fast, fast, slow gives P=40/40 fast and N=0/20 with zero treatment effect by construction. Pre-register a balanced randomized order/seed if interpreting the contrast as a probe effect; retain order/persistence and carryover caveats. Otherwise label it a scheduled association. |
| 4. 60 s rest, 300 s deadline? | Reasonable prospective operational choices; no evidence that 60 s resets thermal/power state. Say fixed rest, not proof of cool equilibrium. Keep 300+10 s sustained bound and add pass/attempt bounds. Do not adapt rest or retry to obtain a desired mix. |
| 5. Input progress at .999 GB/s? | Not acceptable for the present alternates/steady mechanistic interpretation; constant-rate counterexamples falsify it. Acceptable as explicitly quantized input consumption with uncertainty, or after a prospective error-aware classifier. |
| 6. H5 without power counters? | Acceptable collection with an unread field, not identification of H5. Restrict to compatible low-progress observations and name alternative explanations. Supporting clock and probe rate alone do not distinguish power limiting from other causes of reduced progress. |
| 7. Concatenation outside bundle? | Yes, if reconstructible and content-verified as above. No requirement to duplicate 593 MB in the archive. |

Pass 2 follows pass 1, rather than interleaving the one-probe and two-probe treatment. Its P-idle ratio therefore mixes sibling loading with temporal state. Conditioning both passes on fast/slow outcome can further select different states under treatment. Call it a serial contrast, not a sibling's causal price. If causal calibration is essential, prospectively interleave/block the two-probe contrast; this need not be made a performance qualification experiment.

## Disposition and next information value

Expected: discriminate five candidate mechanisms. Actual: source/test review proves several measurement and decision rules can generate the proposed signatures without the associated state. **No empirical gap in the i9's two modes has closed**; no target mechanism established. The useful advance is isolating exactly which observations currently lack the claimed meaning.

The next worthwhile step is a revised, bounded, nonqualifying diagnostic with elapsed-time probe progress, honest occupancy/input proxies, replayable admission and truthful compatibility labels. Add the demonstrated negative controls, correct §5, then perform the full DRY plumbing check before seeking freeze. No demand for an exhaustive causal study first. Use the resulting raw evidence to choose one controlled intervention; do not prescribe warmup, placement or power policy from a table match alone. PR316's later freeze remains separate and is not approved here.

## Evidence and reproducibility

- Reviewed the five files named in `source-manifest.json` at exact PR318 head above. Original Pathfinder proposal read at PR313 head `2e67726b140c9766bce0273c4d70780194675d1b`, `docs/journal/2026-11-10-two-modes.md`; it is design evidence, not authority or a target observation.
- Source links: [judge](https://github.com/lmmx/mibz2/blob/3588d35dc9b2f2f4d4709cf37c4a4bee2afa9dff/bench/two_modes.py), [probe](https://github.com/lmmx/mibz2/blob/3588d35dc9b2f2f4d4709cf37c4a4bee2afa9dff/bench/two_modes_probe.c), [recipe](https://github.com/lmmx/mibz2/blob/3588d35dc9b2f2f4d4709cf37c4a4bee2afa9dff/.just/two-modes.just), [registration](https://github.com/lmmx/mibz2/blob/3588d35dc9b2f2f4d4709cf37c4a4bee2afa9dff/docs/journal/2026-11-10-two-modes-discriminator.md).
- Reproduce in a scratch directory: obtain `bench/two_modes.py` and `bench/test_two_modes.py` from that exact commit, place beside `probes.py`, run `python3 test_two_modes.py`, then `python3 probes.py`. No native build or target timing is required. All counterexamples use analytical or synthetic records; fixture dependence is disclosed.
- All five local source copies were checked against the Git blob IDs in `source-manifest.json`. 44 source tests passed locally on those exact bytes; output in `tests.txt`. Deterministic probe assertions passed; output in `probe-results.json`. CI check-run summaries at the reviewed head showed applicable checks successful (release/publish skipped); job logs were not independently audited.
- Archived executable and evidence-bundle digests were reviewed in source, not independently downloaded/rehashed for this design review. No new runtime bundle or physical-i9 measurement was assessed. Sampling overhead, actual clock coupling, mode prevalence and power-counter availability remain unmeasured here.
- Fresh channel audit at the start of this review returned no routing violations; migration guard passed before receipt. No implementation, branch beyond own state, or automation setting was changed.
