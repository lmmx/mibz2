# Review: the 8–10-thread knee and proposed pipeline

Assignment: [6699480548](https://github.com/lmmx/mibz2/pull/162#issuecomment-6699480548).
Reviewed: [knee-mechanism.md at 853dd1af32fd1df3a8132a0452d7fa5a2f0ee8c0](https://github.com/lmmx/mibz2/blob/853dd1af32fd1df3a8132a0452d7fa5a2f0ee8c0/docs/journal/2026-11-07-knee-mechanism.md).
Instruction pin: a6a703cefcc40ac4115803f44a8a29ad76a3a7ef.

**Recommendation: revise the mechanism and numerical registration before freezing it.** The confirmed three-team policy is not challenged. H1 remains a worthwhile experiment; neither its proposed minimum gain nor its exclusive causal interpretation follows from the archive. No decoder was built or timed for this review. Only archived numbers were recomputed.

## 1. E4 measures an inversion-call occupancy, not the parallel fraction

At the review pin, `src/lib.rs:330–346` brackets the entire inverse-BWT call. MTF/checkpoint creation precedes it; RLE1 and CRC follow it. But `Cooperative::invert_checkpointed` in `src/bwt.rs:193–231` includes lazy pool creation, cooperative build preparation/checks, pool dispatch and completion waits, the walk, and **leader-only segment assembly**. Small blocks take the serial fallback inside these same marks. Thus neither “build and walk” nor “parallel work” accurately names all the measured interval.

The four relevant source files (`src/lib.rs`, `src/bwt.rs`, `src/trace.rs`, `examples/occupancy.rs`) were fetched at both the review pin and the occupancy measurement commit f4f99ab82652411feb4a5a6e2ccb96d04902fb58: their contents are identical.

In `examples/occupancy.rs`, Queues::credit integrates the number of open inversion intervals; report divides this integral by the whole trace span. For each complete run, with N decoded blocks and span T seconds:

- mean inversion duration I = q.tvec_mean × T × 1000 / N milliseconds;
- mean decode duration D = q.decode_ms_mean;
- outside-inversion duration = D − I;
- outside-inversion share of decoding = 1 − q.tvec_mean / w.mean_decoding.

Dividing q.tvec_mean by four instead gives the inversion fraction of four teams' *entire elapsed span*, including their nondecoding intervals. Multiplying that fraction by mean decode duration mixes denominators. The journal also combines timeline occupancy with full-mode block durations. Compute within each run first.

The following are medians of the six individual spread runs per mode; fractions are computed per run before taking their median. Medians of separate quantities need not sum exactly.

| plan | trace mode | decode ms | inversion-call ms | outside-inversion ms | outside share |
|---|---|---:|---:|---:|---:|
| 4×2 | timeline | 1.3868 | 0.9890 | 0.3991 | 28.76% |
| 4×3 | timeline | 1.3789 | 0.9330 | 0.4417 | 32.23% |
| 4×4 | timeline | 1.3977 | 0.9383 | 0.4567 | 32.86% |
| 4×2 | full | 1.3942 | 0.9931 | 0.4000 | 28.75% |
| 4×3 | full | 1.3799 | 0.9352 | 0.4406 | 32.14% |
| 4×4 | full | 1.4035 | 0.9458 | 0.4624 | 32.64% |

Timeline outside-share ranges across rounds are 28.52–28.87%, 31.21–32.54%, and 31.74–33.69%. These are descriptive ranges, not confidence intervals. The 431-block corpus includes tiny blocks, so these are per-block averages, not full-900k-block service times.

**The requested total serial share cannot be recovered from these summaries.** Outside-inversion elapsed time includes MTF/RLE1/CRC but also scheduling and instrumentation; serial assembly/setup inside inversion is not separately timed. Helpers are untraced. `src/coop.rs` uses up to 2,000 spins before parking, both between jobs and on completion; “waiting costs no CPU” is false as an absolute claim. The CI stage split does not calibrate those costs on the i9.

The source commentary calling the table “live” is a lifetime model, not measured cache residency: table lines may remain cached after InvertEnd, and during assembly the trace still counts a table whose last walk access has already happened.

Reproduction: copy the three occupancy JSONs from the pinned `docs/journal/2026-11-06-team-size/` into this directory and run `python recompute.py`. It checks that w.mean_decoding reconstructs q.decode_ms_mean within printed-number precision. All 36 spread traced runs passed. Full calculations and input hashes are committed beside this report.

## 2. E3 does not identify memory saturation

Fixed-team throughput barely improving with k establishes poor marginal returns to additional helpers in these configurations. It does not establish which component limits them.

The registered timed comparison actually found 4×3 **1.017 (1.008–1.026)** times 4×2, below the 3% admission threshold, and 4×4 **1.003 (0.993–1.014)**. Process CPU grew 26% and 52%. Neither those whole-decoder ratios nor a flat DecodeStart→DecodeEnd interval measures the isolated parallel phase.

Concrete alternatives in the cited code are:

- repeated pool unpark/spin/park and completion skew, inside the inversion marks;
- serial build validation/preparation and assembly;
- per-write build window bounds, which serve the disjoint-write soundness argument and cannot simply be removed;
- k-dependent segment counts (`FLAGGED_SEGMENTS * K`), bookkeeping and load balance;
- core/SMT pressure, shared request queues, and changed scheduling.

Memory contention remains a strong hypothesis. The compact-Ψ size intervention and same-byte v control give considerably better evidence for a footprint effect than E3 does. But those were one-core-per-table inversion experiments, not the current cooperative whole decoder; perf failed in the team-size run. Their DRAM plateau is not a counter measurement of 3×3 or 4×2.

Even the footprint archive rejects a pure constant in table bytes: at level 3, 10 blocks have 0.940 L3 hit share, wait about 82 cycles, and transfer about 21 GB/s, yet lose to 8 blocks mainly in build. At level 9, four blocks have 0.929 hit share, about 67 cycles and 14 GB/s. High hit rate and bandwidth below the plateau do not eliminate a request/core-count limit.

Replace E3's final sentence with: **“Additional helpers buy little whole-decoder speed at fixed team count; these measurements do not separate memory limits from serial, synchronization, or core-sharing costs.”** E1's axis should likewise say “planned teams / maximum concurrent tables,” not measured simultaneous live tables. Mean, peak and temporal overlap are different quantities.

## 3. H1's 1.10–1.30× range is a hypothesis, not a derived bound

An ideal steady-state team with disjoint stages has cycle max(S, P), but only when buffer ownership, dependencies, scheduling and synchronization permit that overlap. A usable model is

`C0 = S0 + P0; C1 = max(S1, alpha * P0) + H; speed = C0 / C1.`

Here S includes all leader work assigned to its pipeline stage (including assembly if it remains there), alpha includes changed helper count and contention, and H includes non-overlappable handoff costs. Removing the leader from the parallel stage changes its resources: 3×4 pipelined has three build/walk workers per team, versus four in nonpipelined 3×4 and three in baseline 3×3. E3 does not prove that two or three workers still saturate it under overlap.

A transparent sensitivity calculation, not a prediction: the grid's 3×3 throughput of 1,382 MB/s, with 209,700,196 output bytes and 431 blocks, implies a whole-run cycle proxy of **1.0562 ms/team/block** (= 3 × bytes / blocks / throughput). This includes startup, tail and dispatch and is not a measured steady-state service time.

Take the entry's hypothetical S = 0.400 ms, leaving P0 = 0.6562 ms:

| parallel slowdown alpha | ideal speed, H=0 | speed with illustrative H=0.050 ms |
|---:|---:|---:|
| 1.0 | 1.610× | 1.496× |
| 1.2 | 1.341× | 1.261× |
| 1.4 | 1.150× | 1.090× |
| 1.6 | 1.006× | 0.960× |

Alternatively, taking the entry's own “slows toward four-team inversion” premise and using about **0.94–0.99 ms** gives only **1.07–1.12×** at zero extra overhead; adding the illustrative 0.05 ms gives **1.02–1.07×**. Inversion contains some serial work, so this is a sensitivity proxy, not a replacement phase model. It nevertheless demonstrates why 1.10 is not a justified floor. A 1.30× result needs a total pipeline cycle at most **0.8124 ms**, while 1.10× allows **0.9602 ms**.

Near-three occupancy is also greater than the archived 4×2 *mean* of 2.75, although below its peak of four. More sustained occupancy could hurt disproportionately, or avoiding peaks could help. Those summaries cannot bound the nonlinear response.

Register null and regression outcomes explicitly. Keep 3×3 at 10 as the product comparison, but identify the pipeline effect with matched **3×4 pipelined versus 3×4 nonpipelined at the same budget and placement**, plus the corresponding 3×3 pair. Do not attribute the cross-budget comparison wholly to overlap. Report CPU/GB and RSS alongside wall time.

## 4. The proposed falsifiers are not mutually diagnostic

“No gain + more live tables + slower inversion” is consistent with cache capacity, bandwidth/request contention, synchronization, SMT contention or build work. It does not imply “pure table capacity.”

“No gain + unchanged mean live tables” does not prove a bug. The occupancy identity is mean inversion count = block completion rate × mean inversion lifetime. A change in either factor can leave the product unchanged. Overlap may exist but be canceled by another cost or by uneven stage lengths.

Before timing, define valid per-block phase marks for the new asynchronous ownership: MTF, build setup, build work/waits, walk work/waits, assembly, RLE1/CRC, publish, and explicit intervals when different blocks overlap. The old leader's synchronous InvertStart/End span cannot automatically retain its old meaning. Keep per-role CPU, actual thread-to-CPU placement, and trace perturbation/completeness checks; helper gaps remain unknown unless measured.

Use this outcome map:

| observation | permitted interpretation |
|---|---|
| No demonstrated cross-block overlap | Implementation/instrument validity unresolved; no mechanism verdict |
| Overlap demonstrated; wall improves beyond the registered threshold | Pipeline helps for this machine/workload; mechanism still needs stage evidence |
| Overlap demonstrated; gain absent; build/walk service grows | Added service cost offsets overlap; locate it with phase timing and counters |
| Overlap demonstrated; service unchanged; synchronization/leader/dispatch cost rises | Pipeline overhead or another stage limits the result |
| Trace invalid or interval spans useful gain and harm | Inconclusive, not a causal falsification |

To distinguish the memory explanation, a later authorized run needs coherent changes in misses/traffic per output byte and phase service, plus a controlled footprint or placement intervention. Unchanged misses with growing waits favors a different account than growing misses at fixed placement. No single counter or whole-run occupancy average proves capacity.

## 5. E5's ledger is mislabeled and cannot establish a 20 MiB hot set

The 6.27 MB figure is `f.window_live_max_mb`: the scanner's **compressed-input window**, not live decoded-output buffers. Ready decoded output has separate `q.ready_mb_*` fields (timeline spread means about 0.15–0.20 MB and high waters around 3.6 MB); neither alone includes every output buffer.

The 1.35 MB `f.w.rle.cap_mb` is reserved capacity for the inverse-BWT output subsequently read by RLE1, not “RLE1's output.” The 6.1 MB segment figure and 5.4 MB table figure are also capacities. Capacity/high-water RSS is not contemporaneous accessed cache footprint. Summing separate maxima, capacities and time means cannot establish eviction pressure. Sequentially accessed bytes can matter, but their residency and reuse distance need evidence.

Correct these labels and move the claimed 20 MiB occupancy from “Established” to an unquantified hypothesis. H4 remains last on present evidence. Non-temporal stores are not automatically beneficial for data the reader will soon consume.

## 6. H2 needs an actual plan and placement control

At this pin `src/parallel.rs:550–560` chooses **3×3 for every requested budget >=10** on the matching ten-core/20 MiB topology. Thus requests for 12–20 threads do not force 12–20 decoding workers or unavoidable decoding-worker SMT. A forced 3×4 arm does use 12 decoding workers; record it separately, and account for scanner, feeder and reader work.

Packed versus spread changes available physical cores as well as possible placement (4 versus 8 at the eight-thread mask). It is not a per-thread affinity experiment. Nor does it establish that two members of the same team sharing a core outperform members of different teams: same-table reuse may help, while simultaneously doing the same stage may increase core-resource conflicts.

Retain H2 as a cheap matched control. Remove the unsupported 0–3% upper estimate and the claimed preferred sibling pairing; test actual mappings instead. CPU masks alone do not verify them.

## 7. Ranking: test attribution before committing to a mechanism

**No archived alternative is established as a larger whole-decoder win than H1 on the present i9 path.** H1 can stay first as a research prototype. For experimental order, however, matched placement and phase accounting should accompany or precede its timing; they are prerequisites for interpreting the result, not a new production direction.

- **Compact-Ψ:** the footprint intervention is real, but u6/p4's 1.100 inversion-only gain is not a gain against cooperative production. The same archive gives u6/l4×2 = **0.687 (0.680–0.695)**. Construction and per-step costs matter. The 2026-10-30 feasibility estimate's premise (“four teams already keep tables in cache; only adding a fifth pays”) is now contested by this entry itself. Its 1.1–1.15 ceiling cannot be transplanted as a current bound. H3 deserves a fresh conditional model, especially if H1 raises inversion duty, not automatic promotion.
- **Compact30:** later staged CI tests lost 4-thread wall by **4.8% and 7.7%**, despite smaller amplification. Those two-core SMT results do not decide cooperative i9 behavior, but they refute a generic “smaller table pays” premise. Distinguish the old step/context behind the +23% versus +68% comparison from the current staged kernel.
- **Cooperative archive:** the earlier inversion-only ranking favored four pairs; the later whole-decoder three-team result illustrates why a stage winner does not fix the current plan. End-to-end matched controls remain the arbiter.
- **Dispatch-causality:** queued delay is not necessarily receiver obstruction. The protocol requires completeness and measured obstruction before promotion; its preparation-VM amendment rejects missing scheduler transitions rather than assigning them to waiting. It does not supply an admitted current cooperative-i9 win that outranks H1. The current team leaders' roughly 96% decoding occupancy also limits an idle-only recovery story; eliminating idle can still change co-run cost, so this is not a universal wall-speed bound.
- **Binding census:** on two-core SMT CI runners, co-run CPU and stage pairing are leading hypotheses. That supports measuring leader/helper interactions; it does not transfer the AMD walk limit to the i9 L3 regime.

The revised registration should separate a useful engineering result from an identified cause: qualified three-team policy; plausible pipeline opportunity; currently unresolved reason for its gain or failure.

## Evidence and audit record

All main-branch paths below were read at the review pin above:
- `src/{lib,bwt,coop,parallel,trace}.rs`, `examples/occupancy.rs`;
- `docs/journal/2026-11-06-team-size.md` and its three occupancy JSONs;
- `docs/journal/2026-10-29-cooperative-decoder.md`;
- `docs/journal/2026-10-30-compact-psi-feasibility.md`;
- `docs/journal/2026-11-05-binding-census.md`, its `serial-stages.md`;
- `docs/journal/2026-11-05-compact30-staged.md`;
- `docs/site/blog/2026-11-01-block-dispatch.md`.

Additional archives:
- [compact-Ψ journal](https://github.com/lmmx/mibz2/blob/42794eba51a651238ce9dd3165fe546970a020a8/docs/journal/2026-10-29-compact-psi.md) and [footprint/counter journal](https://github.com/lmmx/mibz2/blob/42794eba51a651238ce9dd3165fe546970a020a8/docs/journal/2026-10-29-libsais-knee.md);
- [dispatch protocol](https://github.com/lmmx/mibz2/blob/64483e8b4855b119469ab5fc338b63fd43740e94/docs/journal/2026-11-01-dispatch-causality-protocol.md) and its `amendment-1.md`.

The source audit and 36-run arithmetic are new in this review; counter and timed-grid results above are attributed archival observations, not newly executed measurements. Pipeline arithmetic is explicitly conditional.

Coordination: the pinned routing tool sends REVIEW to #145. Its audit finds the five already-resolved historical misroutes and five #222 “TO: observer” assignments (6696708089, 6697544323, 6698097217, 6698816282, 6699490637) that its old Oracle-only name table flags. Recorded without modifying other roles or routing authority. Its verify helper also hard-codes FROM: coordinator; publication will be checked against the exact body, Researcher header and #145 destination separately, retaining that tool limitation.
