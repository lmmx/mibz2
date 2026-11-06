# Topology projection: W2 is unidentified; distinguish break-even from the target

Trigger: [6640324039](https://github.com/lmmx/mibz2/pull/145#issuecomment-6640324039). Researcher, 2026-11-04.

## Decision

Do not infer W2 or select compact30 for every multi-worker plan from W1/W4 endpoints. The cited validated capture has W1 and four **physical-core**, lockstep W4 modes only. Its analysis supplies no W2 estimator. Both W2=0.95 and W2=1.10 fit monotonically decreasing W1=1.14845, W4=0.84047 endpoints, yet give opposite break-even decisions at b=0.865. Cache-capacity transitions and SMT make interpolation unjustified.

D014 at Guardian pin `89fff90379278bf100a0a8cd8c3e43c68543e588` closes the specific D013 attribution objection after its raw replay. Honour that closure. It grants neither performance PASS nor a fresh safety certification. This assessment does not repeat the raw replay.

## Correct model and useful thresholds

The checkpoint's equation is a **build-plus-traversal CPU** ratio, conditional on approximate §9 phase costs applying to the tested topology. It is not the whole-decoder CPU ratio. Walk's 75.73% share is of those two components, not established as 76% of production decoder CPU.

For unchanged other CPU cost H, whole-decoder ratio is
`(H + 0.592 b + 1.847 w)/(H + 2.439)`.
H dilutes the percentage change; if other stages change, add their signed cost too. In general use topology-specific measured baseline phase costs B_t,W_t,H_t. Importing i9 W4 ratios into the same constants for every topology does not establish those weights.

Recomputed using the archived, unrounded walk ratios:

| b (assumed) | W1 two-phase ratio | W4 two-phase ratio | w for break-even | w for 0.25 CPU-s/GB saving |
|---:|---:|---:|---:|---:|
| 0.865 | 1.07965 | 0.84642 | 1.04327 | **0.90792** |
| 0.940 | 1.09786 | 0.86462 | 1.01923 | **0.88388** |
| 1.000 | 1.11242 | 0.87919 | 1.00000 | **0.86465** |
| 1.086 | 1.13329 | 0.90006 | 0.97244 | **0.83708** |

Thus “four workers win for every b” is true for these listed assumptions and zero-saving threshold, not all b and not the existing 0.25 target. At b=1.086 even W4 central w=0.84047 misses that target. Under these fixed weights W4 breaks even only for b<1.49774; W1 could win if b<0.53684. None of those b values is a measured combined-build result. The 0.865 sensitivity descends from INVALID historical construction captures and constant-IPC assumptions; it is not a topology-specific prediction.

### What W2 must accomplish

Let A_P(2)=P2/P1 and A_C(2)=C2/C1 for comparable per-row CPU costs on the **same machine, corpus and preparation**. Then
`w2 = w1 × A_C(2)/A_P(2)`.
At b=0.865, using i9 w1 purely as a conditional reference, W2 needs:
- A_P(2)/A_C(2) > **1.10082** to break even;
- A_P(2)/A_C(2) ≥ **1.26493** to achieve 0.25 CPU-s/GB.

At b=1 those thresholds become 1.14845 and 1.32824. This identifies the required differential contention relief; it does not fabricate a W2 value. Two physical workers may fit both tables well enough that the compact lookup penalty survives. Four logical CPUs on two cores also share execution resources unlike the i9's four physical cores. Runner topology must come from the run's env.json, not a permanent assumption about ubuntu-latest.

## Completion and the existing CI

Decoder elapsed time/throughput is the user-facing speed outcome; aggregate CPU remains a separate resource outcome. Neither substitutes for the other. Archived W4 completion is 0.85601 versus CPU 0.84047, already demonstrating the distinction. Summed worker CPU is not the critical path; max/scheduling, overlapped stages and stragglers matter.

At master `c4b69de363dba4485567142a0d9cec4034fba6e2`, existing bench.yml runs whole decodes at 1, 2 spread, and nproc, records environment, and runs phase diagnostics **on one CPU only**. The two-thread whole-decoder result can decide the production plan's value; it does **not** identify isolated w2, because build and other stages also contribute. Do not relabel it as w2. Do not change benchmark infrastructure or authorise extra timing here. Have the Coordinator use the already-authorised existing CI path for the qualified integration candidate; reserve dispatch policy until its actual per-topology CPU/completion results exist. “Workers > 1” is a hypothesis, not a demonstrated policy boundary.

## Is the one-core loss removable?

The archived ordinary compact step is 21 instructions/four data loads versus conventional 16/three; its successor prefetch depends on header shift plus offset addition. Field and header loads are independently addressed, not serial dependent table loads. Their being in one cache line reduces footprint, not their load-instruction count. Extra load/issue work and successor preparation are plausible causes of W1 loss; cache counters and causal timing were not recorded.

A concrete bounded code-shape candidate is the listed redundant `mov ecx,ecx` after `add ecx,edi`, introduced at the ordinary/resumption join. Investigate whether factoring that join removes the re-extension without duplicating more hot-path work, increasing pressure, or weakening RAW/PARK semantics. This is a research lead, not an implementation instruction or a promised cycle saving. Register copies can have microarchitecture-dependent cost; five extra instructions do not prove five extra cycles.

At b=0.865, the current W1 traversal cost must fall **9.16%** merely to break even on the two-phase model, **20.94%** to meet 0.25 CPU-s/GB. Removing one of 21 instructions cannot be credited with either result without a resource/dependency argument and evidence. Prioritise the measured production-topology decision over assuming a small syntactic cut erases the whole W1 penalty.

## Evidence and limits

- Space-time post §§7–9 at master pin above: approximate costs, measurement boundaries and preparation caveat.
- Traversal protocol and `i9-attempt-4/c30-analysis.json` at `95977867fdf318b3f66420c383a73d9c48b36740`: W1/W4-only validated design; ratios and intervals.
- `docs/journal/2026-11-02-compact30-walker.md` at that pin: ordinary listing and dependency audit.
- Guardian decisions.md at pin above: D014 scope.
- audit.json contains recomputed equations and thresholds. Source/archive inspection and arithmetic only; no new build, timing, counters, implementation, merge, or authority change. Existing prior involvement disclosed; this is not a blind §6.2 certification.
