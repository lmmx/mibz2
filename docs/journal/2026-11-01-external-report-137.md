# External research report 137: Astra's memorandum on the five explanations: test buffer-lifetime reuse first (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01, with reports 136–139 together.
- **Source:** Astra (the Codex session of reports 130–131).
- **In reply to:** the maintainer's request for a causal-structure assessment of the four-worker penalty.

**Evidence cutoff:** 1 November 2026. By its own account source review and design only: nothing implemented or run. **The coordinator's note, not Astra's:** the intervention it proposes is P87, whose four-core i9 diagnostic was completed and closed on 1 November (lmmx/mibz2#122; `docs/journal/2026-11-01-p87-four-core/i9-10900K/analysis.md`). The memorandum does not cite that result.

It is kept exactly as received (rendered text, cards flattened, tracking parameters
included).

## The message, as received

`````text
**My preferred next experiment is a controlled buffer-lifetime intervention: assemble the inverse-BWT output into the now-dead last-column buffer, while keeping the T-vector, walker, prefetch policy and assembly copy unchanged.**

This is the existing, untested buffer-reuse direction recorded as P87—not a new representation or another copy-elimination experiment. Its value is diagnostic: it changes the set of cache lines dirtied and reused without adding navigation work.

It cannot uniquely separate cache occupancy from dirty-line coalescing. It can answer a smaller, consequential question:

> Can changing intermediate-buffer lifetimes materially reduce the four-worker CPU penalty while preserving the inexpensive four-byte traversal?

That is a better first engineering decision than trying to name one cause from the existing counters. I have not implemented or run anything.

**The five explanations form a feedback loop, rather than five independent bottlenecks.**

A plausible sequence is:

```text
construction/output traffic displaces useful lines
    → subsequent loads and ownership requests take longer
    → requests occupy hardware resources longer
    → prefetch lead time becomes inadequate
    → dependent traversal slows
    → phase overlap and cache reuse change again
```

Dirty evictions can initiate additional contention, but can also merely record displacement caused elsewhere. The [attribution archive](https://github.com/lmmx/mibz2/blob/33e0e4ebac554bae03795bfb76e86440b33bae6b/docs/journal/2026-11-01-decoder-attribution-i9-10900K/analysis.md) establishes the concurrency penalty much more firmly than it establishes this causal sequence.

| Explanation | Established evidence | Unsupported inference and competing explanation | Discriminating evidence |
|---|---|---|---|
| **1. T-vector residency and eviction** | Retired-load L3 misses increase; inversion becomes substantially more expensive. | Neither four peak tables nor 2.19 average active table phases measures LLC residency. Other buffers, old dirty lines, replacement behaviour and prefetch insertion can produce the same misses. | Change the competing cache-line footprint without changing T-vector access instructions. Attribute any latency/miss change specifically to T-vector addresses. |
| **2. Prefetch interference** | Software prefetch is present; fill-buffer blocking increases; mixed offcore request counts remain approximately stable. | Stable request counts do not establish useful prefetches, adequate lead time or unchanged queueing. Prefetches may become late because service slows, rather than cause that slowdown. | An intervention that reduces competing traffic while leaving prefetch unchanged can show whether prefetch-resource pressure is downstream of that traffic. Harm caused by prefetch itself ultimately requires a separate policy intervention. |
| **3. Write allocation and dirty writebacks** | Much more traffic reaches DRAM, particularly writes. | This does not identify the responsible buffers or show that writebacks delay the walk. Read-for-ownership traffic and eventual writebacks are different transactions. | Reduce distinct dirty intermediate lines at unchanged logical copy volume; determine whether unchanged walkers become cheaper, or only write traffic falls. |
| **4. Construction/traversal interference** | The shipped builder writes every T-vector entry; workers can build and walk concurrently. | Temporal overlap alone is not causation. Slow walks can themselves prolong overlap. Freshly built tables also have different cache states. | Eventually, a controlled phase-order intervention preserving work and accounting for cache age. Correlating walk cost with builder activity is only a way to select that intervention. |
| **5. Uncore/request scheduling** | Additional stalls and an increased occupancy/request proxy are consistent with slower service. Average bandwidth is below the historical sustained rate. | That does not exclude bank conflicts, read/write turnaround, burst saturation, ring pressure or uncore-frequency changes. A streaming bandwidth ceiling is not the ceiling for this request mix. | Compare service cost at matched request classes and clock conditions. A change in latency without a corresponding miss-rate change would favour service/queueing effects over a simple residency explanation. |

Two architectural details deserve emphasis.

First, stores can hurt traversal without producing a matching increase in retired demand-load L3 misses. Ownership requests and writebacks can compete for resources; blocked stores can also constrain retirement and the instruction window. That can reduce useful overlap among the 32 chains.

Second, the existing occupancy figures do not establish that MLP is unchanged. Intel’s `L1D_PEND_MISS.PENDING` includes prefetch-allocated fill buffers **after demand accesses them**; its ratio to pending cycles is a conditional occupancy measure. `FB_FULL` counts requests blocked by unavailable fill buffers, including stores and software prefetches. Neither measures all useful outstanding traversal work.

**Why I would test buffer reuse first**

The current [decoder](https://github.com/lmmx/mibz2/blob/45b366d1455a487f1655def689ebd8c87a31f135/src/lib.rs#L310) has this lifetime:

1. MTF produces `ll`.
2. T-vector construction reads `ll`.
3. Walking produces segment buffers.
4. Assembly copies segments into `rle`.
5. RLE1 reads `rle` and writes final output.

After construction, the segmented walker no longer needs `ll`. The [inverse-BWT implementation](https://github.com/lmmx/mibz2/blob/45b366d1455a487f1655def689ebd8c87a31f135/src/bwt.rs#L79) confirms that separation.

The proposed contrast is:

| Arm | Assembly destination | Everything else |
|---|---|---|
| **A: distinct storage** | Existing `rle` arena | Shipped algorithms |
| **B: reused storage** | Existing `ll` arena, after its last read | Same algorithms and logical copy volume |

Conceptually:

```text
build T from ll
walk T into the existing segment buffers
assemble segments into [rle in A / ll in B]
run the existing RLE1 and CRC path
```

The input borrow must end before `ll` becomes the destination; this is not permission to alias input and output during construction.

Both arms still:

- build the same four-byte table;
- execute the same dependent traversal and prefetches;
- write the same segment bytes;
- perform the same assembly copy;
- decode and check the same output.

At a full block, B avoids using a second approximately 0.9 MB arena for these two successive roles. It does **not** shrink the walker’s instantaneous T-vector-plus-segment working set. The repository’s [P87 assessment](https://github.com/lmmx/mibz2/blob/9723f06847b88a10ba93d07c3f35680e58766015/docs/journal/2026-10-30-research-portfolio-register.md) correctly makes that distinction.

The possible benefit is between phases and workers: overwriting already-dirty storage may avoid additional cache-line acquisition, occupancy and writeback. If `ll` has already been evicted before assembly, much of that benefit may disappear.

This is why the experiment is useful. It tests the lifetime mechanism instead of assuming that one fewer allocation means one fewer block’s worth of DRAM traffic.

**The hypothesis and its strongest alternative**

The hypothesis is:

> Distinct storage for the last column and assembled BWT output creates material avoidable memory interference at four workers. Reusing that storage reduces the concurrency-specific CPU penalty, including some cost outside assembly itself.

The strongest alternative is:

> T-vector servicing and walker-to-walker interference dominate. The extra intermediate writes are mostly consequences of eviction, or are sufficiently decoupled from traversal that reducing them does not materially improve CPU cost.

Expected signatures:

| Under the reuse hypothesis | Under the alternative |
|---|---|
| B has a larger CPU benefit at four workers than at one. | B is neutral, or saves only local assembly/RLE cost. |
| Logical bytes copied and hot-loop instructions remain unchanged. | Same requirement; otherwise the contrast is contaminated. |
| Ownership traffic and/or writebacks decline. | They may decline substantially without reducing walker cost. |
| Unchanged walks become cheaper, potentially with fewer expensive T-vector demand samples or less fill-buffer blocking. | T-vector service cost and walker CPU remain approximately unchanged. |

A reduction in writes **without** a meaningful CPU improvement is an informative negative result: it argues against treating those removed writes as an important latency cause.

It would not prove that every other write stream is harmless.

**The bounded design is a 2×2 comparison: A/B at one and four workers.**

Use an isolated diagnostic path for the ordinary segmented decoder. Preserve production dispatch, admission, output ordering and prefetch settings.

To isolate lifetime reuse from allocation effects:

- Allocate and prefault both arenas in both arms, with identical capacities.
- Retain the unused arena in B throughout measurement.
- Change which arena assembly touches, not the amount of allocated memory.
- Balance arena placement across paired repetitions so a favourable address/cache mapping is not mistaken for a lifetime effect.
- Verify identical generated hot functions for construction and walking. Check assembly copy volume and RLE implementation equivalence.

A later production implementation could remove the unused arena. That RSS benefit is deliberately excluded from this causal comparison.

Use the same fixed Wikidata corpus and complete decoder boundary in both arms. Record BWT-row bytes separately from final decoded bytes: RLE expansion makes them different denominators.

Let \(C_{X,w}\) denote CPU-s/GB for arm \(X\) at worker count \(w\). The main diagnostic quantity is:

\[
D=(C_{A,4}-C_{B,4})-(C_{A,1}-C_{B,1}).
\]

This asks whether reuse specifically reduces the concurrency penalty, rather than merely making assembly locally cheaper.

I would prespecify **0.10 CPU-s/GB** as the materiality threshold for this first decision. That is approximately 14% of the observed incremental inversion penalty and 2.8% of current total CPU cost. It is a decision threshold, not a forecast.

- An upper confidence bound below 0.10 rejects the claim of a material concurrency benefit from this intervention.
- A lower bound above 0.10 supports further engineering.
- An interval crossing the threshold remains inconclusive; do not tune layouts until one wins.

Report the ordinary four-worker CPU and throughput effects alongside \(D\). A useful local improvement can exist even if the concurrency hypothesis fails.

**Minimum instrumentation should answer where the benefit occurs.**

Add a boundary between walking and segment assembly; the existing broad walk interval includes both. Record block identity, row count, worker, and phase CPU time for:

- construction;
- traversal;
- assembly;
- remaining decode work.

Record allocation ranges when buffers are allocated or grown. That allows address samples to distinguish T-vector loads from segment, last-column and output accesses. No per-lane logging is needed.

For the i9, I would use these event groups in separate, nonmultiplexed passes:

| Measurement | Events or facility | Semantic limitation |
|---|---|---|
| CPU cost and comparable work | `cycles`, `ref-cycles`, `instructions`, task-clock | Instructions do not measure bytes copied; core frequency does not establish uncore frequency. |
| Ownership and dirty-line movement | `l2_rqsts.rfo_miss`, `l2_lines_out.non_silent` | RFOs are ownership requests, not DRAM writes. Modified L2 evictions go toward LLC, not necessarily DRAM. |
| Execution/resource pressure | `cycle_activity.stalls_mem_any`, `l1d_pend_miss.fb_full` | Coincidence with an outstanding load is not complete causal attribution. `FB_FULL` is not a count of distinct misses. |
| Actual memory-controller traffic | `uncore_imc/data_reads/`, `uncore_imc/data_writes/` | Machine-wide; background and measurement boundaries matter. No source-buffer attribution. |
| T-vector demand-service evidence | Supported PEBS load-latency sampling, e.g. `MEM_TRANS_RETIRED.LOAD_LATENCY_GT_8`, with IP, address and latency | Thresholded, selected loads—not an unbiased mean of all loads. Reported latency includes more than bare memory service. |

Intel’s [model event definitions](https://github.com/intel/perfmon/blob/main/SKL/events/skylake_core.json) should determine the exact encodings supported by the installed kernel.

`MEM_LOAD_RETIRED.FB_HIT` is a useful optional addition, but a fill-buffer hit can follow a demand miss as well as a prefetch. It does not identify software-prefetch effectiveness.

I would not make the mixed offcore occupancy/request ratio a decision metric here. It is less discriminating than observing the result of the intervention.

**The important controls and confounders are specific to this intervention.**

- **Allocation and first touch:** equalize them in the diagnostic. Otherwise fewer page faults can masquerade as lower cache interference.
- **Clock and power:** retain the same policy, record effective core frequency and available uncore-frequency telemetry. The earlier 5.9% clock attribution concerns core frequency; it does not rule out uncore changes.
- **Placement:** preserve the original four-physical-core allocation and affinity policy. Record migrations and context switches; do not introduce a new scheduling policy.
- **Phase overlap:** record it, but do not force it. Faster phases can change overlap as a consequence of the intervention. Conditional overlap plots are explanatory evidence, not a causal substitute.
- **Writeback boundaries:** counters must cover equivalent whole-work intervals. Repeated steady-state windows should also show whether lower write counts merely reflect deferred eviction beyond the window. Do not insert cache flushes into the measured workload.
- **Instrumentation:** use uninstrumented A/B runs for primary CPU, throughput and RSS results. Check phase tracing and PEBS perturbation separately.
- **Correctness:** verify full output and CRCs, segment boundaries, small-block fallback, mixed block sizes and buffer-capacity reuse. The destination changes; decoding semantics must not.
- **Revision:** use the newly measured A as the denominator. The historical 0.68–0.72 figure supplies a planning budget, not a guaranteed baseline for a later binary.

The extra retained arena keeps diagnostic RSS comparable. This experiment should not claim production readiness merely because its arithmetic suggests a smaller eventual scratch allocation.

**The engineering-decision matrix**

| Outcome | Interpretation | Engineering decision |
|---|---|---|
| Material positive \(D\); unchanged walkers become cheaper; ownership/writeback pressure declines | Buffer lifetimes causally affect the concurrency penalty. Capacity relief and dirty-line coalescing may both contribute. | **Retain four-byte T-vectors.** Engineer bounded arena reuse first; investigate other intermediate lifetimes only with a measured budget. |
| Writebacks decline, but \(D\) is small and walker cost unchanged | The removed traffic is largely noncritical under these conditions. | Reject this route as a major throughput mechanism. Do not infer a large gain from eliminating the assembly copy. |
| CPU improves only in assembly/RLE, similarly at one and four workers | Local cache reuse helps, but does not explain the inversion concurrency penalty. | Consider a small independent improvement; keep it out of the 1.5–2× architectural argument. |
| \(D\) improves, but measured DRAM writes do not | Benefit may come from cache occupancy, ownership latency, TLB effects or request timing rather than reduced DRAM write volume. | Retain the layout opportunity provisionally; reject a “fewer DRAM writes caused it” explanation. |
| Little traffic or CPU change | Reuse did not substantially perturb the suspected resource—possibly because `ll` was already evicted. | Reject this particular intervention; the broader dirty-traffic hypothesis remains unresolved. |
| Residual walk cost is concentrated in build-overlap periods in both arms | Construction interference remains a candidate; correlation alone is insufficient. | Next discriminate with controlled build/walk overlap before engineering a phase policy. |
| Residual cost persists during substantial walk-only overlap, with expensive T-vector service | Intermediate reuse is insufficient; navigation footprint/request behaviour deserves renewed attention. | Revisit compact/composed Ψ **only with a construction-plus-walk budget**. This result alone does not select either representation. |
| Apparent benefit follows clock, placement or code-generation differences | The intended causal contrast failed. | Repair the diagnostic; make no architectural inference. |

No single outcome from this experiment cleanly separates all five explanations. In particular, it does not prove harmful prefetching, and it cannot authorize a phase-overlap policy from observational phase labels.

That limitation is intentional: the first decision is whether a realistic buffer-lifetime change can improve four-core economics without changing traversal.

**The recoverable budget should remain conservative.**

There is no evidence-based point prediction for this experiment. The defensible range includes zero.

Recovering all **0.68–0.72 CPU-s/GB** attributed to incremental inversion cost would reduce 3.579 to approximately **2.86–2.90 CPU-s/GB**. Even then, the four-core utilization bound permits only about **1.59–1.61×** current throughput, assuming perfect utilization.

For 1.5×, approximately **0.51 CPU-s/GB** must be saved even under that optimistic utilization assumption: roughly **71–75% of the entire incremental inversion penalty**. A successful 0.10 result would be useful evidence, but a small portion of the required recovery.

Twofold throughput requires efficiency gains beyond removing this measured inversion penalty.

The prior negative results therefore remain important. N1 must pay for construction and retained storage; compact Ψ must avoid its previous byte-recovery overhead; prefetch changes must overcome earlier full-decoder losses. Buffer reuse has a smaller ceiling, but a cleaner initial contrast.

**Test whether reusing the dead last-column arena reduces concurrency-specific CPU cost while preserving all traversal and copy work.** If traffic falls but unchanged walkers do not become cheaper, stop treating that intermediate traffic as a major latency cause and move the investigation toward construction overlap or T-vector servicing.
`````
