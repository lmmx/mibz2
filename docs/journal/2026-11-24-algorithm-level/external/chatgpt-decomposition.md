# ChatGPT Pro on the decoder's decomposition and how to measure its full character (verbatim)

*24 November 2026, ~23:00, pasted by Louis. It answers `briefs/decomposition.md` at 9174b294. The inline links did not survive the paste (`⁠￼`). The Coordinator's comparison with MONODROMY's record 125 and Neon's `DECOMPOSITION.md`, and the resulting recipe brief, are in journal §51 and `briefs/character.md`.*

---

The scanner/workers split is a valid coarse execution topology, but it is not a complete performance decomposition. I would model mibz2 as a network of activities, buffers and shared resources, then use small interventions to test that model. The most consequential correction to the brief is that the control-coefficient summation theorem is not a general completeness test. It can fail at precisely the bottleneck transitions you want to locate, even when the model contains every component.

There is also a concrete implementation issue to resolve first: at this commit, requested threads are not necessarily actual workers, and changing the requested count can change the kernel and execution policy.

I read the pinned brief, journal §§44 and 47, and the relevant implementation. Below, proved means an algebraic result under stated assumptions, or a fact directly established by the pinned code; argued means an evidence-backed interpretation or design recommendation; conjectured means a proposition the proposed experiment should test. I have not run new workstation measurements.

1. Is the decomposition right?

First establish which decoder the experiment is varying

Proved from code. StreamDecoder::new can change the actual worker count, kernel, window and execution topology. On the admitted i9 configuration, its unpinned policy can turn requests for ten or more threads into seven lazy/Open workers. Forcing Plan::Workers(k) alone also does not hold the inversion algorithm constant: that path changes its kernel selection with the count.

For the experiment you describe, the existing mibz2-workers-lazy benchmark arm is the appropriate starting point. It explicitly requests k workers with the production OpenCells admission/fallback policy and a 4k window. The similarly named mibz2-workers-open uses ForcedOpenCells, which changes admission. These are materially different experiments. Sources: decoder construction and policy⁠￼, benchmark arms⁠￼.

Argued recommendation. Keep two separately labelled questions:

* Mechanism: how does one fixed worker implementation behave with k=1,\ldots,20 actual workers?
* Public API: what happens when a caller requests 1–20 threads and the shipped policy makes its choices?

The first is the primary study here. A separate, unperturbed sweep answers the second cheaply. Pooling them would make a "contention model" explain deliberate policy changes.

Use three connected descriptions

The useful decomposition has three layers:

1. Activities: what work transforms each block.
2. Flow and blocking: when that work can run, and when buffers or admission credits are released.
3. Resources: what concurrent activities compete for.

The brief mixes these layers. Queue waiting, for example, is an outcome of the network; it is not another service demand comparable to Huffman decoding.

For the activities, I would make these corrections:

Activity	Recommended boundary
Input acquisition and maintenance	Separate feeder reads/copies from scanner-side buffer maintenance and input waiting. Input-associated CPU work occurs on both threads.
Header, bitmap, selectors, tables	Retain as scanner work, separately from the symbol loop.
Huffman decoding	Retain; count input bits and produced symbols as well as time.
Dispatch and recycling	Account for active locking, handoff and pool operations; record blocking separately.
MTF/RLE2 and representation formation	One fused activity. On the run route, this already constructs the run records.
Inversion construction	Instrument natural sub-boundaries: first-pass classification/admission; setup plus induction/dictionary construction; retained-edge wiring.
Emission	Separate from construction where the implementation provides that boundary; retain route and emitter tags.
RLE1/CRC finalization	Preserve the existing tiled implementation. Do not introduce separate passes merely to obtain separate timings.
Ordered consumption and output	A distinct execution agent and service demand, outside worker finalization. Include the actual output contract.

Proved from code. decode_owned calls runs::front to perform MTF/RLE2 and form run records together. Counting "runs" again as a separate S5 pass would therefore overlap S4. Conversely, Open's construction has real internal boundaries worth observing. RLE1 and CRC are deliberately interleaved in tiles, so separating them experimentally could change the very cache traffic being studied. Sources: ⁠￼decode_owned⁠￼, Open construction⁠￼, RLE1/CRC implementation⁠￼.

Proved from code. This is also a routed graph, not one universal sequence through Open. Blocks can take the run or column route; Open can be admitted, skipped, or attempted under a cap and then followed by fallback. Log the actual route and retain the cost of rejected Open work. A model of successful Open blocks alone is not a model of the decoder. Source: Open admission and fallback⁠￼.

The important missing connection is the output-to-scanner feedback

Proved from code. The outstanding-work limit is based on issued - taken, and taken advances when the ordered reader takes the next required result. A block waiting in the job queue, executing, or completed out of order can therefore continue occupying admission capacity.

The worker path also has a 12 MiB ready-output threshold, with a floor of k+2 outstanding blocks. Thus the effective admission rule is more than "the job queue has capacity W." Source: ⁠￼Shared::full⁠￼, output cap and ordered ⁠￼advance⁠￼.

The relevant topology is:

flowchart TB
    I["Input feeder"] --> S["Scanner"]
    S --> W["Jobs and k workers"]
    W --> R["Completed blocks"]
    R --> O["Ordered reader and output"]
    O -->|"Releases admission credits"| S
    H["Shared cores, caches and memory"] -.-> S
    H -.-> W
    H -.-> O

Argued implication. A slow early block can hold up ordered consumption, accumulate later results, retain memory and stop the scanner. A scanner waiting for admission does not, by itself, prove insufficient worker compute capacity. It might reflect output ordering, the consumer, the ready-byte cap, or a long-tail block.

Split the shared-resource description more carefully

"Memory bandwidth and LLC" merges mechanisms that need different remedies:

* DRAM traffic and sustainable bandwidth: bytes crossing the memory-controller interface.
* Cache capacity, locality and cache bandwidth: which accesses reach which level.
* Latency and memory-level parallelism: how many independent misses can overlap.
* Core execution and SMT sharing: competition for execution resources, including helper threads.
* Allocation and virtual memory: allocation/reuse, zeroing, page faults and retained buffers.
* Synchronization: lock contention, wakeups, scheduling and migrations.

Frequency and power policy affect the service rates of several activities simultaneously. They should be controlled or observed, rather than silently absorbed into an "SMT penalty."

Also, k>10 is not a universal SMT boundary. The scanner, feeder and consumer compete alongside the k workers. Placement and runnable activity determine when cores are shared. At twenty workers, there are already additional software threads beyond the machine's twenty hardware threads.

NUMA does not deserve a full experimental axis on this single-socket target unless the reported topology reveals an unexpected arrangement. Record the topology once. Frequency, cache state and helper-thread placement are much more immediate concerns.

What the four-arm result establishes

Argued from the recorded experiment. The result strongly supports count-dependent control and bottleneck migration: an emission capability can become valuable after front-end work is removed. It does not uniquely identify the physical reason.

Supplying scanner output changes more than Huffman instruction count: it can change memory traffic, allocation, runnable-thread pressure and scheduling. Supplying codes also leaves substantial emission-related work in place. The results describe those supplied-interface interventions; they are not isolated derivatives, nor universal ceilings on algorithms that change the representations and work together. Source: recorded four-arm results⁠￼.

2. A minimal design, and the precision it buys

Argued recommendation. Use a staged campaign: measure the complete count curve, estimate a small set of local responses, then test predictions. There is no universally sufficient number of rounds before seeing the variance.

A. Freeze the experimental contract

For the primary campaign, specify:

* The explicit actual-worker path and production admission/fallback rules.
* k=1,\ldots,20, with the declared window policy—initially the existing W=4k.
* One placement policy throughout. An unpinned run on all twenty logical CPUs is a reasonable deployment-oriented primary condition; targeted pinned runs can investigate interference.
* The input corpus and order, input residency, decoder/buffer lifetime, output sink, and what the clock includes.
* Compiler/build, kernel selection, allocator, frequency/power policy and machine state.
* Whole-decode time through the declared output-consumption endpoint, plus steady-state and startup/drain measurements separately.

Changing k while using W=4k changes both workers and permitted in-flight work. That is a legitimate operating policy, but its model must include both variables.

Keep normal correctness checks. Do not turn the baseline into a different memory experiment by unnecessarily loading oracle stores.

B. Instrument every count, with repeated runs

Run twelve randomized complete rounds. Each round contains one baseline observation at every count. This gives 240 baseline observations.

Randomize count order; do not repeatedly sweep upward from one to twenty. At perturbation counts, keep the baseline and treatments close in time and randomize their order.

If individual fast-count decodes are still around a fraction of a second, aggregate several complete decodes into one observation. Preserve the creation/destruction and cache contract; concatenating them into a longer stream would change startup and drain costs. Inner repetitions reduce measurement noise but do not become independent experimental rounds.

Use lightweight, per-thread buffered events:

* Block identity, route, stage start/end and queue transitions.
* Input waiting, admission waiting, worker starvation and ordered-output waiting.
* Queued, executing and completed-but-unconsumed work.
* Occupancy integrals and high-water marks, not merely an occasional queue-length sample.
* Compressed bits, symbols, BWT rows and final output bytes.

Maintain separate ledgers for per-thread wall states and on-CPU work. An elapsed stage interval may contain descheduling; CPU time includes time executing while stalled on memory. Neither is interchangeable with whole-decoder wall time.

Perform six paired instrumentation-on/off checks at k=1,10,20. If instrumentation changes throughput or queue behavior appreciably relative to the intended precision—say, around 0.5–1%—reduce its cost or separate detailed tracing from the timing campaign. Do not assume an overhead subtraction repairs changed scheduling.

C. Collect a small counter set consistently

Collection level	Measurements
Every timing observation	Wall time, output bytes, CPU time, correctness, actual workers/route/window, faults, context switches, migrations, peak memory
Lightweight trace	Exclusive phase durations, queue transitions, ready bytes, pool reuse, startup and drain
Focused PMU runs	Cycles, instructions, reference cycles/frequency, branches/misses; then cache/TLB and memory-stall events supported on this CPU
Shared-resource runs	DRAM read/write traffic through available memory-controller counters; memory occupancy over time

Use time-weighted mean memory or byte-seconds alongside peak RSS. A process-lifetime high-water mark is not automatically a fresh per-decode peak.

For richer PMU data, use two small counter groups, three repeats at the six anchor counts below: 36 additional observations. Record counter running/enabled coverage and avoid heavy multiplexing. Linux's perf output exposes event runtime and running percentage for this purpose. 

Two interpretation rules matter:

1. LLC misses × 64 is not exact DRAM traffic. Prefetches, writebacks, cache-to-cache transfers and event definitions matter.
2. Shared uncore counters do not automatically attribute bytes to individual concurrent stages. They constrain the resource model; they are not an additive per-stage ledger.

Compute expensive structural features outside timed runs and join them by block identity. The existing census performs an additional O(n) pass; enabling it inside every measured decode would alter the workload. Also preserve identity grouping: the existing Open census has 233 distinct blocks repeated four times, not 932 independent input samples. Source: block census and split⁠￼.

D. Perturb at six anchor counts

Use:

\boxed{k\in\{1,6,8,10,12,20\}.}

These retain the existing one/six/ten conditions, add a point inside the suspected transition, and cover higher concurrency. Add one further anchor only if the baseline reveals a transition inadequately covered by these.

Start with +10% nominal service-demand interventions. For the internal compute graph, a concrete initial set is:

1. Scanner header/selectors/tables.
2. Huffman symbol decoding.
3. MTF/RLE2 plus representation formation.
4. Open first-pass classification/admission.
5. Open setup/induction/dictionary work.
6. Open retained-edge wiring.
7. Open emission.
8. Non-Open inversion work, initially as an aggregate.
9. RLE1/CRC finalization.

These are disjoint scopes: a rejected Open attempt is charged before subsequent fallback work. Add input-maintenance, dispatch or consumer interventions when the baseline indicates material demand or control there. All remain in the accounting even if they are not initially perturbed.

For p perturbation coordinates and twelve rounds, the initial campaign costs

\boxed{12(20+6p)=240+72p}

observations, sharing one baseline per count and round. Thus p=9 gives 888 observations, before calibration and selective follow-ups. That is linear in the number of activities.

The shared baseline induces correlation between coefficient estimates. Preserve complete rounds in the bootstrap or other inference procedure.

Specify what "10% slower" means

An intervention needs an operational definition:

Intervention	What it changes
Busy-loop delay	Core occupancy and potentially SMT/frequency interference
Repeated work	Instruction demand, memory traffic and possibly cache state
Parked delay while retaining the job	Worker availability and buffer lifetime, while releasing CPU
Delay before handoff	Arrival timing and upstream occupancy

Argued recommendation. For the first experiment, use a calibrated boundary-pacing intervention with a budget derived from unperturbed block/phase measurements. Record achieved delay and its placement relative to resource release. Never recalculate the budget from a phase's already slowed duration.

Call the result a response to that intervention. A boundary delay is not automatically equivalent to making the original code slower.

For influential or surprising results, test a second implementation of the intervention or a real small optimization. This is particularly valuable for negative responses and SMT-sensitive counts.

Use +5% follow-ups for selected coefficients and likely transitions. Comparing the 5% and 10% responses checks local linearity. Do not use 5% universally: it approximately halves the signal.

Also reserve a small combined test: for example, Huffman and emission together at k=8,10. The model should predict this without being fitted to it.

E. State precision mathematically

For a demand increase \delta, estimate

\widehat C_i
=
\frac{\overline{\log T_{i,\delta}-\log T_0}}
{\log(1+\delta)}.

Use achieved exposure, or a regression against it, when the realized dose differs materially from the nominal dose.

With n independent paired rounds and paired log-ratio standard deviation s, an approximate pointwise 95% half-width is

h_C
=
\frac{t_{n-1,0.975}s}
{\sqrt n\,\log(1+\delta)}.

For +10%:

Paired log-ratio SD	12 rounds	20 rounds
1%	C_i\pm0.067	C_i\pm0.049
3%	C_i\pm0.200	C_i\pm0.147
5%	C_i\pm0.333	C_i\pm0.246

Proved by calculation, conditional on those variances. These are not promises about the workstation.

For baseline runtime, twelve observations give approximately ±0.64% precision at 1% log-runtime SD, or ±1.91% at 3% SD. Simultaneous intervals across twenty counts will be wider.

A coefficient C=0.1 produces only about:

* 0.96% more runtime under a 10% demand increase;
* 0.49% more runtime under a 5% increase.

The existing twenty-round ten-worker interval for C/A, 0.806[0.782,0.831], has roughly ±3% relative width. Although that does not determine the variance of the new experiment, it warns against assuming these sub-percent effects will be resolved. Source: four-arm report⁠￼.

At +10%, achieving a coefficient half-width near ±0.05 takes approximately twenty pairs if paired SD is 1%, but about 160 if it is 3%. A 5% dose needs about 3.8 times as many observations for comparable precision.

Therefore:

* Twelve rounds are a defensible first identification pass.
* They can identify large controls under reasonable noise.
* Small effects should be reported as unresolved intervals, not zeros.
* Use the first pass to size an independent confirmation, or predeclare sequential inference. Do not extend ordinary 95% intervals repeatedly until a preferred conclusion appears.

For the optimum, report a near-optimal count set, such as counts consistent with being within 2% of the best, with simultaneous uncertainty. A plateau is a useful result.

F. Fit and validate the actual network

Argued recommendation. Start with a discrete-event model of the finite admission window, routed workers, out-of-order completion and ordered consumption. An M/M/k model is unnecessarily restrictive; exponential service times and independent arrivals are not credible defaults for this fixed block sequence.

Use complete per-block service vectors in original order. Independently resampling construction and emission times would destroy correlations that can cause long-tail blocks and output stalls.

Predeclare, for example:

* Fit counts: \{1,2,4,6,8,10,12,16,20\}.
* Held-out counts: \{3,5,7,9,11,13,14,15,17,18,19\}.

Start with fixed demands. Add a small number of contention terms when residuals demand them, shared across relevant stage families. Candidate explanatory quantities are simultaneous working set, core sharing, traffic pressure and frequency—not an unrestricted parameter for every stage at every count.

Do not insert measured service times from a held-out count and call its resulting throughput a held-out prediction. Fit the service-demand/contention law on training counts, freeze it, then predict the held-out counts.

Validate:

* Whole-decode time and startup/drain.
* Scanner admission/input waiting.
* Worker starvation.
* Outstanding and ready-output occupancy.
* Ordered-output stalls.
* The held-out +5% and combined interventions.

Conjectured acceptance target: a useful first model might achieve ≤3% median and ≤5% maximum relative error on held-out mean runtime, with major wait fractions within about five percentage points. These are proposed acceptance thresholds, not expected precision.

A model that fits throughput but gets the queues wrong has likely compensated one error with another.

Finally, avoid multiplying every nuisance factor into the main grid. Use targeted checks:

* Change window size—or the ready-byte threshold if that is what binds—at a few anchors.
* Compare scanner isolation with core sharing where interference is suspected.
* Check a fresh-process/cache regime and real output sink at a few counts.
* Test other existing corpus classes, especially dense/fallback and high-expansion blocks.

These establish the limits of the primary result without a full Cartesian product.

3. What should be reasoned about rather than measured?

Capacity bounds and bottleneck migration

Proved under the ideal model. Let a be scanner service seconds per block and b=\sum_i b_i worker service seconds per block, with independent workers, sufficient buffering and no other bottleneck:

J_{\rm blocks}(k)\leq\min\left(\frac1a,\frac{k}{b}\right).

In its steady-state limiting case:

* Scanner-limited: scanner control is one; worker-stage controls are zero.
* Worker-limited: scanner control is zero; worker-stage i's control is b_i/b.
* At the crossing, the ordinary derivative need not exist.

This already explains strong nonadditivity: improving a hidden component has little effect until the controlling component changes. You do not need an arbitrary pairwise interaction term to produce the four-arm pattern.

The ideal crossover is approximately k=b/a. With contention, a and b become functions of concurrent activity, so the equation becomes implicit.

Amdahl-style bounds remain useful when their fixed-cost assumptions hold. An additive serial-plus-parallel wall-time model is inappropriate for stages that overlap. Gustafson addresses changing workload scale; it does not resolve this fixed-corpus overlap problem.

Finite-run dependencies and accounting identities

Proved conditional on a fixed execution/resource-order graph. Completion time has a max-plus form:

T=\max_{\text{paths }p}\sum_{e\in p}d_e.

This supplies critical-path bounds and explains why a large amount of CPU work can have little local wall-time control. Changing scheduling or cache behavior can change the graph or its durations, so it is a model theorem rather than a complete hardware model.

The stronger completeness checks are accounting identities:

* Enqueues minus dequeues equal queue occupancy.
* Issued minus taken equals outstanding admitted work, with transient states accounted for.
* Every block's successful, rejected and fallback work remains charged.
* Integrating queue occupancy equals the sum of residence intervals, with boundary corrections.
* Each thread's timeline is partitioned without gaps or double-counted intervals.

These can establish whether the instrumentation accounts for the execution. A control-coefficient sum cannot.

Memory and output bounds

Proved conditional on the resource limit. If q bytes must cross a resource interface per output byte and its available bandwidth is B,

J_{\rm output}\leq \frac{B}{q}.

The useful engineering version uses measured sustainable bandwidth under an appropriate traffic mix and actual memory-controller traffic. Roofline explicitly bases operational intensity on traffic after cache filtering. 

You can reason about compulsory representation sizes, passes, copies and output writes. You must measure or validate which of them reach DRAM, and how that changes with count and live working set.

Also, a bandwidth demand exceeding capacity proves a proposed throughput infeasible; it does not prove the real program will reach bandwidth saturation. Dependency latency, limited memory-level parallelism, branching or scanner supply can limit it earlier.

For a specified stream of necessary misses, an additional conditional ceiling is

J_{\rm output}\leq
\frac{M}{L\,m},

where M is sustainable concurrent misses, L their mean latency and m misses per output byte. Establishing those quantities is the difficult part. An algorithmic addressed-read count alone does not establish a machine-time lower bound.

What cannot be inferred from a linear experiment

Proved. One-at-a-time experiments do not determine arbitrary interactions. Two response functions can agree at baseline and on every single-coordinate perturbation while differing by

\gamma x_i x_j

away from those axes.

The way to avoid an exponential grid is to assume and test a constrained mechanism: known flow topology, finite buffers, a small number of resource couplings and limited workload dependence. Neither linear sampling nor systems-biology terminology removes this requirement.

4. Are control coefficients the right formalism?

They are useful global elasticities

A flux control coefficient is already an elasticity: a global throughput response to an independent local parameter.

The independent variable must be a service capability or demand parameter, not the stage's observed completed blocks per second. In a stable pipeline, flow conservation couples those observed rates.

For a demand multiplier d_i, fixed output Q, and J=Q/T, a convenient convention is

C_i
=
-\frac{\partial\log J}{\partial\log d_i}
=
\frac{\partial\log T}{\partial\log d_i}.

It says how strongly increasing that demand increases total runtime. It is a useful local quantity at a specified worker count, workload, placement and operating point.

The summation theorem needs explicit assumptions

Proved conditional result. Suppose independent capacity parameters \mathbf v satisfy

J(\lambda\mathbf v)=\lambda J(\mathbf v),

and J is differentiable at the operating point. Euler's homogeneous-function theorem then gives

\sum_i
\frac{\partial\log J}{\partial\log v_i}=1.

This homogeneity condition is the substantive basis of the metabolic summation result. Giersch's analysis explicitly connects the theorem to it. 

Here is a complete counterexample to using the sum as an unconditional completeness test:

J(v_1,v_2)=\min(v_1,v_2),\qquad v_1=v_2.

Direction tested	Coefficient for server 1	Coefficient for server 2	Sum
Infinitesimal slowdown	1	1	2
Infinitesimal speedup	0	0	0

Proved. Nothing is missing. The operating point is a bottleneck switch and has no ordinary gradient.

Further difficulties in this decoder are:

* Scaling code rates while leaving memory, I/O and scheduling times fixed need not uniformly rescale time.
* Treating a stage's memory-dependent speed and the same memory service as independent additive controls can double-count a mechanism.
* A missing component with zero local control need not disturb a sum of one.
* Slowing one activity can reduce interference and improve total throughput, producing negative responses.

Therefore, use the sum only as a conditional model-consistency diagnostic. Do not normalize measurements to make it one, interpret the deficit as missing time, or impose [0,1] bounds without the necessary monotonicity assumptions.

A response matrix better captures the systems-biology objective

Argued recommendation. Retain the throughput coefficients, but measure the complete response to each intervention:

\text{intervention }i
\longrightarrow
\begin{bmatrix}
\text{throughput}\\
\text{other stages' service times}\\
\text{scanner waiting}\\
\text{worker starvation}\\
\text{ready backlog}\\
\text{DRAM traffic}\\
\text{memory occupancy}
\end{bmatrix}.

The same runs provide all these outputs.

A stage can have nearly zero throughput control while substantially changing queue occupancy, another stage's service time or available headroom. Those responses expose coupling that a scalar ranking misses.

This gives the biological analogy precise content:

* Activities correspond to transformations.
* Buffers and outstanding work are state variables.
* Resource sharing determines local service laws.
* Interventions test the network's global response.

It still does not uniquely identify every physical cause. If cache pressure, SMT and frequency remain observationally confounded, report a combined contention effect until a discriminating intervention separates them.

Coz tests optimization direction

Coz virtually speeds selected code by delaying other concurrently executing work and correcting elapsed time. That is different from slowing the selected stage. Its progress points measure completed useful work. 

Argued recommendation. Use Coz, or an equivalent carefully controlled virtual-speedup experiment, for one or two leading targets—especially near a crossover or where slowdown responses are surprising.

For mibz2, progress should correspond to ordered useful output, preferably fixed output-byte increments when block sizes vary. Keep the whole-file stopwatch as the ultimate outcome.

A virtual speedup still needs validation against a real change where possible. The cost of removing instructions, removing memory traffic, and retaining a job while releasing a CPU are different causal operations.

Roofline constrains the model; it does not replace the network

Use resource ceilings to reject impossible predictions and distinguish traffic reduction from faster execution. They cannot explain admission blocking, output ordering or bottleneck migration alone. Likewise, being well below the DRAM roof does not establish that a phase is compute-bound.

The division of labour I would adopt is:

Tool	Role in this study
Routed activity graph and finite-window simulation	Explain overlap, ordering, blocking and startup/drain
Resource-demand measurements and ceilings	Constrain contention and physically attainable rates
Local control coefficients	Rank small, specified changes at a fixed operating point
Multivariate intervention responses	Reveal coupling and test the model's internal mechanisms
Coz and small real optimizations	Check whether inferred sensitivity transfers to improvement
Structural oracles	Evaluate large changes in information or representation, with acquisition costs stated

Conjectured: a compact network with a few shared-resource terms will explain most of the 1–20 curve. The proposed design makes that conjecture falsifiable.

The result to aim for is a model that predicts where control moves, which intervention helps at each count, how much it helps, and what it does to memory and waiting, with explicit uncertainty. That would give the mathematical programme a quantitative target: the demand, dependency or representation change a new construction must achieve to improve the whole decoder in the operating regimes that matter.
