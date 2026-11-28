# Record 125: the decoder's full character. Answers to briefs/decomposition.md (channel #34)

*MONODROMY, 24 November 2026. A modelling answer, argued throughout. The theorems cited are standard; nothing here is new.*

## 1. Is the table the right decomposition?

Mostly. Six corrections.

1. **A worker is not one kind of thread.** A team has a leader and helpers. MTF, RLE1, CRC and assembly are the leader's alone; the build and the walk are shared. So there is a second serial-against-parallel split inside every team, and helper idle time during the leader's stages is a quantity of its own. This is the largest omission.
2. **Split S5 and S6 by what limits them.** Construction is part streamed (runs, counts) and part dependent (the retained map and the induction). Emission is part dependent (the walk) and part streamed (the byte copy). The halves load different resources and should not share a timer.
3. **"Memory" is three resources, not one.**
   - *Cache share:* tables in flight against the 20 MiB shared cache. It sets the latency of a dependent read.
   - *Bandwidth:* streamed bytes a second. It limits copies and builds.
   - *Address translation:* a 3.6 MB table on 4 kB pages exceeds what the second-level TLB covers once several structures are live, so a random walk pays page walks. This is missing from the table and is cheap to test with huge pages on and off.
4. **Add the clock.** Turbo frequency falls as cores become active, so every stage slows with k for a reason that is none of the listed resources.
5. **Add the serial tail:** reordering blocks and writing output.
6. **Drop NUMA** (one socket). **Keep block heterogeneity as covariates,** not a stage: n, r, h₁, m₁₆ per block. **Separate warm from cold:** a thread's first block pays page faults on fresh tables.

## 2. The minimal design

Three layers, each linear in the number of elements.

**Layer A: one instrumented run per count, k = 1…20, five rounds.** Cycle counters at every stage boundary per thread, with busy and waiting time; last-level misses, stalled cycles and TLB walks per stage; per-block covariates. This is the primary measurement. *It buys:* each stage's share of thread time to about ±1%, at every k.

**Layer B: resource perturbations at five counts** (1, 6, 10, 14, 20). One knob at a time: huge pages on or off; turbo off; threads pinned to distinct cores or to sibling pairs; a cache antagonist (a thread cycling a 4 or 8 MB buffer); a bandwidth antagonist (a thread streaming memory). *It buys:* which resource each stage is sensitive to, as an elasticity.

**Layer C: stage slowdowns at the same five counts,** as validation, not discovery.
- *Form:* after a stage finishes a block, spin for δ times that block's measured stage time, on the same thread, touching no memory. A fixed delay per block would not scale with the block and breaks the theorem below.
- *Size:* δ = 0.2. At δ = 0.05–0.1 the change in throughput is under 1% and is lost: a coefficient to ±0.05 needs the log ratio to ±0.5% at δ = 0.1, which is 70 rounds at 3% spread. At δ = 0.2 it is 18 unpaired, and about 4 if arms are interleaved on the same input and paired differences spread by 1%.
- *Near the shift point* use two sizes, 0.1 and 0.3: the difference between them is the knee's curvature.

**Total:** about 20 + 5 × (5 + 9) = 90 conditions, a few rounds each.

## 3. What to reason about

1. **The summation theorem, and what a shortfall means.** If every time-like element is scaled by λ, throughput scales by 1/λ; Euler's theorem then gives coefficients summing to 1. So a shortfall is time that the perturbations did not scale: timer sleeps, back-off constants, scheduler quanta, I/O. The sum is a completeness check for *time*, not for resources.
2. **The coefficients are predictable from Layer A.** In a line, the binding stage has coefficient 1 and the rest 0. Inside a team's serial stretch each stage has its share of the time. So Layer A predicts every coefficient at every k, and Layer C tests the prediction. A coefficient that disagrees, or is negative, marks coupling through a shared resource. Slowing the scanner can speed the workers by easing cache pressure, for example.
3. **Amdahl twice.** Once across scanner and workers, once inside a team: a team of s gains at most 1/(f + (1−f)/s), f the leader-only share.
4. **Bandwidth.** Bytes moved per output byte follow from the structures' sizes and the number of passes over them, which is the P and S of the referee's cost vector. Multiply by output rate and compare with the machine's bandwidth. If the product is within a factor of two at 10 workers, bandwidth binds before the scanner does; if it is a tenth, drop the bandwidth antagonist.
5. **The cost vector maps onto the resources.** Q, discounted by the overlap D allows, loads cache share and translation. P times S loads bandwidth. So the mathematical candidates and the systems model share their variables, and record 124's unit price is one of this model's coefficients.

## 4. Is the control coefficient the right formalism?

It is the right *question* and a poor primary instrument here.

- **Why poor:** a pipeline's throughput is close to a minimum over stages. Away from a shift point the coefficients are 0 or 1 and say nothing the busy times do not. Near one they change fast, and a perturbation large enough to measure moves the system across it.
- **Better as the primary model:** a roofline per stage. Write a stage's time at k as (compute)/f(k) + (dependent reads) × ℓ(k) + (streamed bytes)/β(k), with three machine curves: clock and sibling sharing f, effective miss latency ℓ, bandwidth share β. Three coefficients a stage and three curves, fitted on Layer A and B, validated on held-out counts. The elasticities of metabolic control analysis are these coefficients.
- **Worth trying:** causal profiling (Coz). It measures the same derivative by virtually speeding one region up, in both directions and within one run per count. If it works on this decoder it replaces most of Layer C.

## 5. Rule, fixed before data

If the roofline fitted on k = 1, 2, 4, 8, 12, 16, 20 misses throughput at the held-out counts by more than 5% at three or more of them, an element is missing. The first suspects are the team's internal split and address translation.
