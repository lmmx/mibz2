# Binding census: model-specific prices and whole-block phase pairing

Inbox: [6657883434](https://github.com/lmmx/mibz2/pull/160#issuecomment-6657883434).
Evidence pin: **decc71b866a1d55ae86251e6e227b9c11987f357** (docs/binding-census, #193).
Instructions: **0479b380f5d6309f3fd0f23e0ad6dfade4b7a5c0**.
Disposition: **proposal and conditional resource accounting only; zero new verified credit**.

## Decision

The largest conditional pool is AMD walk interference. On the 8370C, build interference ranks ahead of further walk tuning. Across the heterogeneous CI fleet, Huffman is the clearest small, source-local opportunity; handoff and MTF remain worth their already-running isolated investigations. There is no measured positive lower bound for any new construction here, and no defensible fleet-weighted expected gain without the runner distribution.

Placement is a way of recovering a pool, not a second pool to add to walk/build savings. My distinct proposal is **bounded whole-block phase pairing**: choose among already-ready blocks to reduce harmful sibling phase coincidence, retaining four ordinary end-to-end workers and private scratch. Its sign is unknown until the pair matrix and workload exposure establish a benefit.

## Evidence and scope

I audited the [census](https://github.com/lmmx/mibz2/blob/decc71b866a1d55ae86251e6e227b9c11987f357/docs/journal/2026-11-05-binding-census.md), its accounting, supply, walk, serial-stages and no-go fronts **including falsifier corrections**, and [structured results](https://github.com/lmmx/mibz2/blob/decc71b866a1d55ae86251e6e227b9c11987f357/docs/journal/2026-11-05-binding-census/results.json). I cross-checked the relevant paths in parallel.rs, bwt.rs, block.rs, bits.rs and examples/phases.rs at the evidence pin. The census identifies production source as d1fd4405; later documentation commits are not new performance measurements.

The stage runs are 42011789918 (7763), 42020095581 (9V74), and 42008210159 (8370C). Decoder accounting uses the census's 41988588303, 41986908473, 41980640481 and 41986885355/41979947814 observations; occupancy includes 42009406264 and 41976148952. I did **not** download/replay their raw timing artifacts, execute candidates, reproduce complete disassemblies, or dispatch workflows. Quoted measurements are pinned archive reports; arithmetic below is independently recomputed in [model.json](model.json).

Four phases processes run stages at uncontrolled relative phases. The harness selects each stage's fastest pass. Thus their factors are neither a measured W/W pair nor exact stage shares in the parallel decoder. On the 8370C, walk inflation is 1.46 timed split versus 1.61 timed as one; this materially limits attribution. The walk-only sweeps also contain surrounding work and do not establish that every active sibling has the same effect. Cross-instance absolute times, collector perturbation, and 10-ms thread accounting further limit precision.

## Price the denominator before the lever

Let C be total process CPU for fixed output and u=C/(4W). Then

    W_new/W_old = (C_new/C_old) * (u_old/u_new).

This is an accounting identity, not an independent proof that CPU savings become wall savings. At unchanged u, removing x% of total co-run CPU removes x% of wall. A candidate can alter u, its own inflation, its sibling's inflation and the stage mix.

For a conditional stage intervention with alone time A, current inflation f and reference r, the own-stage pool is A(f-r)/C. The reference is a comparison scenario, not an achievable floor. Full net cost also includes changed sibling service, new instructions/traffic, scheduling and idle. Assembly is already inside the walk total where reported that way; do not add it again.

| CPU | First large resource question | Conditional own-stage pool if the stated reference were reached | Practical ordering after that question |
|---|---|---|---|
| 7763 | Walk interference, 2.12 versus compute reference 1.33–1.43 | Approximately **16–18%** of wall at fixed utilisation; achievable fraction unknown | Huffman; MTF/handoff; build excess only roughly 0–1% |
| 9V74 | Walk interference, 2.22 versus 1.23–1.32 | **20–22% scenario**, not a decoder measurement; see denominator assumption below | Build excess roughly **4% scenario**; Huffman; MTF/handoff |
| 8370C | Build interference, 2.25 versus reference 1.55 | Approximately **5–6%** | Huffman; MTF/handoff; walk excess over compute reference not established |
| 9V45 | Walk interference is plausible from sweeps and #171, but no matched stage factor table | **Unpriced**; do not import 9V74's percentage | Huffman/handoff/MTF isolated results; verify the unusually large idle without sampler |
| 8573C | Walk versus build unresolved without per-stage co-run factors | **Unpriced**; the 8370C is not a substitute | Huffman/handoff/MTF; retain the actual #25 negative/control history |

These are rankings by available resource opportunity, not expected build returns. Large pools without a working lever do not justify replacing the smaller prototypes already under way.

**7763 derivation.** C≈4×0.953×0.324=1.2351 CPU-s. The alone walk is 0.2845 s. Taking inflation from 2.12 to 1.43 or 1.33 removes 0.1963–0.2248 CPU-s: 15.9–18.2%. This uses different machine instances and assumes unchanged rest and u. It is not a confidence interval or rigorous upper bound. Build's corresponding 0.1389×(1.43−1.33)/1.2351 is only 1.12%; choosing scanner's 1.43 reference makes it zero.

**9V74 derivation.** No matched decoder bench decomposition exists here. The falsifier reports walk as 59.7% of worker stages. With an explicitly transferred worker fraction q=0.81–0.83 of total CPU, q×0.597×(1−r/2.22) gives 19.6–22.1% for r=1.32–1.23. This is deliberately not the census's 18%: the denominator and reference choices are exposed, and the result should remain an approximate ~20% scenario until measured on the decoder. The 4.8% build excess of total phases time becomes about 4.4% of process CPU if worker stages are 90% of phases and 82% of process CPU; both conversion fractions are assumptions.

**8370C derivation.** 0.1174×(2.25−1.55)/(4×0.973×0.392)=5.39%. Four-copy-share conversion gives a nearby ~6%; the difference belongs in uncertainty. The walk's 1.46–1.61 overlaps the compute reference, so this method finds no robust walk-specific excess. It does not establish irreducibility or a zero opportunity from a changed representation.

## Smaller levers and what is actually recoverable

**Huffman: 0 to about 4.3% conditional direct pool; no guaranteed 0.5% floor.** With scanner share 9.1–10.7%, an assumed 15–40% stage reduction and retention anywhere from zero to one, the maximum product is 0.107×0.40=4.28%. The existing prototype must establish both reduction and retention. Scanner starvation might contribute up to the census's ~0.9% scenario, but it overlaps supply/handoff and must not be added as independent credit. Source confirms hot lookups call peek_live/skip_short on buf/live, with refill boundaries and slow decoding. The reported store/reload listing is useful evidence; source alone does not prove each compiler build spills those fields or that removing the spill improves the sibling's throughput.

**MTF: central working envelope 0–3%, broad optimistic stage-removal scenarios larger, all unverified.** The census's 1.3–9.4% range is a conditional mechanism model, not a supported positive lower bound. For example, a 30% alone cut with inflation changing from 1.33 to 1.71 leaves 0.70×1.71/1.33=0.90 of old co-run MTF time. At 13.2% total share that buys 1.32%, before sibling effects. A 55% cut with full retention can produce 7–9%, but no evidence here establishes that combination. The register-head construction's #25 history includes null wikidata 4t and harms on other inputs; the new MTF workflow must distinguish its construction and preserve those controls. I do not add another MTF proposal.

**Handoff: 0–2% working wall envelope; 2–3% remains a broad all-idle scenario, not the lock's price.** The source holds Mutex<Receiver> across blocking recv. Much of the 3.0–3.5% lock elapsed time can be workers waiting behind an empty receiver. It is not removable CPU work. Job-queued idle of 1.7–2.4%, multiplied by the census's empirical 0.59–0.72 conversion, suggests roughly 1.0–1.7% if entirely causal and removable. Collector uncertainty, time denominators and changed sibling occupancy prevent treating this as a bound. Distinguish this mutex from the closed split-condvar proposal.

**Supply: shared steady pool ~1.2–1.8% on the measured 9V74 scenario.** 2.0–2.5% idle capacity ×0.59–0.72 gives that range if recovered. It overlaps Huffman and handoff. Startup/ramp/tail and fresh-process faults matter to the short bench, approaching zero per byte on a long decode. Window widening is at most ~0.4% in the corrected occupancy interpretation; condvar CPU below ~0.4% is not itself a wall guarantee. Unattributed sampled CPU is not a new removable stage.

**Build instruction cuts, fusion and compact.** A modest build cut can be ~1–2% on 8370C/9V74 even without recovering the entire interference pool; Ir does not establish proportional cycles. Correlation with RLE1 weakens the unique many-store-stream story, but does not prove issue saturation or guarantee instruction cuts. Fusion remains closed under its registered rule; a heterogeneous three-row average is not a new fleet qualification, especially with the later 7763 +1.2% interval spanning both signs. Keep a 0–2% possible pool, not new credit. Staged compact remains the already-running discriminator: archaeology predicts +2–7% wall, the walk model −2 to +11% before all build accounting, central +3–5%. These are related hypotheses, not three independent measured replications. Do not reopen a duplicate port or promote it on reduced footprint alone.

## Open the mechanism, rather than name it prematurely

1. **Core-side issue/queues:** similar ~2.12–2.16 inflation for kernels with very different line rates argues against one fixed line-throughput ceiling. It does not isolate issue slots: front end, execution resources, load queues, scheduling and power can create similar factors. The fit's lane-independent term is not a PMU reading of issue cost.
2. **Miss slots/latency:** inferred MLP 9.8→5.5 or 9.3→5.5 is derived using plain-chain latency. That latency is sampled under a different sibling workload from the staged walk. It is not direct occupancy of a miss structure. Fewer effective requests in flight can be consequence as well as cause of core-side starvation. Neither resource hypothesis is closed.
3. **Sibling versus beyond-core interference:** same-core pair effects must be compared with the same partner on the other core, at matched active work/duty, not simply with idle. Shared cache/memory, frequency and support-thread contention remain possible. A4/A2 includes both cores busy and oversubscription, not only SMT.
4. **Placement:** requires a *partner-dependent* net cost, not merely an active-sibling penalty. Measure both directions, fixed units of useful work, plus startup/tail. A fast victim with a severely slowed partner can lose overall. ALU and stream loads are discriminating stresses but not pure instruments: ALU can compete for frontend/load ports; streams consume issue and memory resources. Multiple intensities/footprints and the cross-core control strengthen inference.

## C21: bounded whole-block phase pairing

This is a fourth design relative to the three rivals as named in the brief; it is not a claim that all unseen variants of those rivals lack this technique.

Keep four pinned end-to-end workers. A worker retains its own scratch through MTF, build, walk and post-walk. At an ordinary block boundary, select from the existing bounded window of scanned, ready jobs using a coarse prediction of each job's stage durations and the sibling's current phase. Prefer a job whose compute-heavy prefix overlaps the sibling's remaining walk (or the reciprocal profitable pairing identified on that CPU). Never move a live partially decoded block or add per-row queues.

Candidate metadata could be symbol count and inexpensive summaries already available from scanning, calibrated from completed blocks. Predictive power is unproven: if it requires a second scan, detailed per-symbol classification, frequent cross-core coordination or per-row instrumentation, charge that cost and reject the cheap formulation. Publish phase hints at stage boundaries only. Keep output indices unchanged, bound bypass of the oldest ready job, reserve progress for the earliest unfinished output, and fall back immediately to FIFO when no useful alternative is ready. Do not deliberately leave a core idle to manufacture pairing. Preserve the existing window/memory bound, cancellation, error order and scratch recovery.

This changes **whole-job order and phase coincidence**, not stage ownership. There is no dedicated walker/compute thread, no walk-exclusion token, and no software-interleaved single-thread walker. Handoff replacement is not claimed as part of its gain: compare against the same handoff implementation. Private scratch avoids the stage-pipeline's intermediate-state migration, but selection, affinity and ordered-output delay are new costs.

### Pair-matrix gate and price

For chosen fixed useful-work amounts, let T(i,j) be the measured core makespan for completing both i and j, including the tail after one finishes. Compare two schedules doing the same two W units and two C units:

    D = T(W,W) + T(C,C) - 2*T(W,C).

A positive D supports replacing a like-paired batch followed by another like-paired batch with two mixed batches. D≤0 falsifies that particular batching advantage. This is a controlled pair-level screen, not a decoder makespan prediction: block precedence, unequal phase amounts, concurrent use of the other core and ready-job limits must still be charged. If the instrument instead reports steady-state victim rates, obtain both directions and reconstruct fixed-work completion with tails; do not sum rates from incompatible duty cycles.

The strongest separating signature is: W/C improves aggregate fixed-work completion over like pairs **and** remains better than serial execution while both siblings are active. That supplies a reason to retain SMT instead of excluding a walk or replacing SMT. If every active partner slows W equally, this design has little reason to work even if idle helps greatly; a token or single-thread design may still have a case. If gains require passing intermediate stages between blocks, stage-specialisation may win and this design fails.

The matrix alone cannot separate this from a stage-specialised pipeline producing the same W/C pairs. Their distinction is exposure attainable without handoff versus pipeline transfer, imbalance and queue costs. The worker walk share is about 55–60%, so perfect complementary exposure is not generally possible; all blocks can also be nearly identical. Actual ready-job diversity and phase durations must supply enough alternate pairings.

Price the candidate as:

    net wall saved = baseline fixed-work makespan
                   - predicted makespan under selected whole-job order
                   - selection/coordination/order-delay cost.

Until the matrix exists, credit is zero. As a transparent sensitivity example, recovering one quarter of a 16% interference pool gives 4% gross, and 1% added cost leaves 3%; neither the quarter nor the overhead is measured. Do not add this to the same walk or build pool elsewhere.

### Falsifier and stop condition

Stop this formulation before a build if the matrix lacks a symmetric pairing advantage, if the window supplies no predictively distinguishable ready jobs, or if weighted completion with the real stage mix erases the benefit. A later authorised comparison would need FIFO versus selection with identical kernels, worker topology, window and handoff, followed by complete decoder wall/CPU and existing input controls. Reject on non-positive net benefit, ordered-output/memory regression, or an effect that disappears after including both siblings and fixed output. A proposed practical minimum is 1% whole-decoder wall on the targeted CPU with controls preserved; the Coordinator must register the rule before any implementation. This report grants no build or timing authority.

## Ledger consequence

C21 is CONDITIONAL, zero credit. C15 is an existing discriminator, not a new build recommendation. C17 stays closed; C18 remains closed; C19/C20 share the small supply opportunity. Historical A1–A6, A7 and D013 accounting are unchanged. The ongoing pair matrix, Huffman, MTF, handoff and compact workflows are the next evidence, not duplicated work.
