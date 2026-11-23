# Launch record: Performance Landscape, 23 November 2026 (prepared)

- **Model:** Gemini Neon, in a fresh Antigravity conversation using `/teamwork`, in its own
  checkout.
- **Branch:** `research/performance-landscape`. Label WIP commits `wip(performance-landscape)`.
- **Role:** theory and experimental programme, not implementation.
- **Relation to Contraction Budget** (`docs/team/research-charter/launches/2026-11-23-contraction-budget.md`):
  that line tests one falsifiable rule, a per-block construction budget. This line builds the
  theory that such a rule would be one case of. Read that line's branch if it exists, cite it, and
  don't duplicate its experiments.
- **Origin:** the charter comes from Louis's reading of his conversation with Thomas Murphy
  (blog, 22 November 2026, "Decompression is the harder problem"), refined with ChatGPT on 23
  November. From that conversation: "real world problems are not proof shaped in simple ways …
  they are cache contention shaped", the data "is heterogeneous … so this needs a diversity of
  tactics", and the systems-biology approach of "modelling all these little parameter knobs",
  then compressing "that realism to something 'nicer' mathematically, yet preserve the
  properties we care about that cannot be hacked (real world perf)".
- **Prepared** by the Coordinator at Louis's request.
  - **Two adaptations from the charter as drafted:**
    - The team's usual research rule replaces its blanket "do not modify the repository": write
      only to your own branch.
    - Bibliographic details are explicitly to be verified. The Coordinator has not checked the
      DOIs.
  - **Paste everything inside the fence.**

````text
# Research charter: a systems biology of inverse-BWT performance

## Mission

Investigate whether the measured performance of exact inverse-BWT decoders can be explained by a
low-dimensional family of coupled structural and hardware parameters, such that the optimal
algorithmic strategy and its transition boundaries can be predicted for previously unseen blocks
and execution regimes.

The goal is not another decoder optimisation. The goal is a mathematical theory of the
performance landscape that can predict which optimisation will work, which will fail, and why.

Use lmmx/mibz2 as the experimental organism.
- Repository: https://github.com/lmmx/mibz2
- Benchmark history: https://lmmx.github.io/mibz2/benchmarks/history/
- Research journals and experimental evidence are in `docs/journal/`.
- Workstation evidence bundles are on the `evidence` branch, each identified by a SHA-256 posted
  on PR #145.
- Earlier Gemini research lines, by branch:
  - `research/carbon-word-transport`
  - `research/carbon-wormhole`
  - `research/carbon-kitchen-sink`
  - `research/neon-teamwork-paradigm`

Read the current repository rather than assuming its architecture or results are unchanged.

This investigation draws from four MSC2020 subject areas:
- **62K20, response surface designs:** how to design experiments that reveal interactions.
- **93B30, system identification:** how to infer mechanisms from observations, including
  identifiability.
- **92C42, systems biology and networks:** how coupled mechanisms produce robustness, fragility
  and emergent behaviour.
- **90C31, sensitivity, stability and parametric optimisation:** how optimal solutions change
  across parameter space.

Also consider:
- 93B11, system structure simplification;
- 62K25, robust parameter designs;
- model reduction.

Don't treat these classifications as boundaries. Import mathematics wherever it materially
strengthens the investigation.

## Part I: establish the empirical object

Study mibz2's actual benchmark methodology and decoding architecture. Pay particular attention to:
1. The Open inverse-BWT kernel, and its construction of runs, edges, anchors and convoys.
2. The relationship between construction cost and the number of dependent steps retained for
   walking.
3. The scanner, the block reader, the parallel front end and worker scheduling.
4. Shared hardware constraints: core execution, L1/L2/L3, memory bandwidth, memory-level
   parallelism and peak memory.
5. Performance differences between repetitive or structured text and identifier-heavy Wikidata
   blocks.
6. The historical sequence of accepted *and rejected* optimisations.

The benchmark history chains paired measurements across merged changes, propagates uncertainty,
separates CPU models, and distinguishes CI results from release-scaling measurements. Respect those
distinctions:
- A chronological chain of successful merges is not a factorial experiment.
- Failed experiments are essential evidence, even when they don't appear in master's history.
- Measurements from distinct CPU models must not be treated as interchangeable.
- CI at 1–4 threads and i9 runs at higher concurrency probe different resource regimes.
- Relative speedups cannot be assumed to compose independently.
- A modelled full-dump estimate is not an observed full-dump measurement.

Find existing evidence before proposing new experiments. Never invent benchmark outcomes.

**The working hypothesis.** The Open kernel can eliminate dependent LF steps by performing
additional construction.
- **The figures:** the teamwork programme reports construction at about 60% of block time and
  walk-plus-emit at about 40%.
- **Verify them first:** check them independently against their claimed source, the i9's
  `bench-emit-cost`. The Coordinator couldn't confirm that run.
- **What follows:** the interesting object isn't the cost of a pointer chase. It is the
  *exchange rate* between construction work, eliminated dependencies and contention for shared
  resources. Investigate whether that rate varies systematically with block structure and
  machine state.
- **Don't assume:**
  - that the current convoy representation is optimal;
  - that identifier-heavy blocks with few runs and high entropy are fundamentally irreducible.

  Distinguish limits specific to a representation from general lower bounds.

**Recent evidence to include:**
- **The worker-count recalibration** (#145 LOG 7085615681, evidence `472ce552`): adding
  workers helps Wikidata and costs text by similar amounts. That's a clear corpus-by-concurrency
  interaction.
- **The 0.5.13 release scaling and crossover** (#145 LOGs 7086295745 and 7086402673).
- **This week's ports that were not adopted** (#762, #765 and #778) and their reasons. Each is a
  local speed-up on a path that barely runs, or that doesn't bind.

## Part II: follow the mathematical lineage

Read these in the given order. Verify each bibliographic detail, as the DOIs below were not
checked. Distinguish what each paper proves from what might transfer to decoder performance.

**Stage A: experimental design (62K20)**
1. Box & Wilson (1951), *On the Experimental Attainment of Optimum Conditions*.
   doi:10.1111/j.2517-6161.1951.tb00067.x
2. Morris (1991), *Factorial Sampling Plans for Preliminary Computational Experiments*.
   doi:10.1080/00401706.1991.10484804

*Question:* how do we design a few experiments that reveal the important interactions in a large
parameter space? Decide which mibz2 measurements are responses, which are explanatory
variables, and which are controlled interventions.

**Stage B: identifiability (93B30)**

3. Bellman & Åström (1970), *On Structural Identifiability*. doi:10.1016/0025-5564(70)90132-X
4. Ljung (1982), *Aspects on the System Identification Problem*. doi:10.1016/0165-1684(82)90058-5

*Question:* which internal decoder mechanisms can actually be inferred from observable
benchmarks?
- Distinguish structural identifiability, practical identifiability, parameter uncertainty and
  predictive uncertainty.
- Find what additional measurement or intervention would make indistinguishable mechanisms
  distinguishable.
- One candidate intervention is titration: adding calibrated work to one stage and measuring the
  throughput response. It gives control coefficients in the sense of Metabolic Control Analysis.

**Stage C: systems biology (92C42)**

5. Barkai & Leibler (1997), *Robustness in Simple Biochemical Networks*. doi:10.1038/43199
6. Carlson & Doyle (1999), *Highly Optimized Tolerance: A Mechanism for Power Laws in Designed
   Systems*. doi:10.1103/PhysRevE.60.1412
7. Kitano (2004), *Biological Robustness*. doi:10.1038/nrg1471
8. Gutenkunst et al. (2007), *Universally Sloppy Parameter Sensitivities in Systems Biology
   Models*. doi:10.1371/journal.pcbi.0030189. **This is the conceptual centre.**

Also consider Metabolic Control Analysis: Kacser & Burns (1973) and Heinrich & Rapoport (1974).
Look at its control coefficients and summation theorems as a model for how throughput control is
shared between pipeline stages and shifts with conditions.

*Question:* can dozens of implementation and hardware parameters produce behaviour governed by a
few strongly constrained combinations? Explore:
- robustness against sensitivity;
- highly optimised tolerance and fragility;
- compensating mechanisms;
- sloppy and stiff directions;
- whether stable aggregate throughput hides large shifts in internal bottlenecks.

Don't assume biological results transfer to computer programs. Establish where the transfer is
mathematically justified, and where it's only suggestive.

**Stage D: parametric optimisation (90C31)**

9. Shapiro (1994), on sensitivity analysis of parametrized programs via generalized equations.
   Verify the exact citation.
10. Jones, Schonlau & Welch (1998), *Efficient Global Optimization of Expensive Black-Box
    Functions*. doi:10.1023/A:1008306431147

*Question:* can we identify stable regions of parameter space where different strategies are
optimal, including their boundaries and any discontinuous transitions? Account for measurement
uncertainty and model error when claiming that a strategy wins.

**Supplementary:**
- Saltelli et al. (2010), *Variance Based Sensitivity Analysis of Model Output*.
- Kennedy & O'Hagan (2001), *Bayesian Calibration of Computer Models*.
- Seek further primary literature:
  - mechanistic performance models;
  - bottleneck transitions;
  - queueing networks;
  - roofline models;
  - resource contention;
  - experimental design under expensive evaluations.

Don't summarise the papers one by one. Derive a coherent methodological lineage, and say where
traditions genuinely connect and where they must be combined.

## Part III: construct a mathematical performance model

Build a hierarchy of models, from the simplest identifiable baseline to a coupled mechanistic
model. Start from

    T = F(T_scan, T_build, T_walk, T_emit, T_contention, N),

where F must account for overlap, scheduling, pipeline bottlenecks and shared resources. It must
not simply add the stage times. Let:
- x be measurable block structure;
- h be the hardware and execution regime;
- θ be the decoder strategy and its tunable parameters;
- T(x, h, θ) be the predicted total time.

Investigate whether a reduced coordinate system z = φ(x, h, θ), of much lower dimension, gives
T ≈ G(z). More importantly, find whether θ\*(x, h) = argmin_θ T can be predicted reliably.

Candidate dimensionless coordinates, which are hypotheses only:
- construction cost over walking cost;
- work spent per dependent step eliminated;
- the scanner's production rate over the workers' consumption rate;
- aggregate memory demand over effective bandwidth;
- working set over cache capacity;
- available parallelism over resource saturation.

Answer:
- **A. Identifiability:** can the available measurements identify the mechanisms? If not, what
  is the minimum additional instrumentation or intervention?
- **B. Interactions:** are the effects of optimisations additive, multiplicative, or strongly
  conditional? Can a local kernel gain worsen the whole decoder through contention?
- **C. Dimensionality:** does the sensitivity matrix show strongly separated singular values?
  Which combinations are stiff and which are sloppy? Don't confuse poor parameter
  identifiability with good predictive accuracy.
- **D. Regime boundaries:** can parameter space be partitioned by which strategy wins? Are the
  regions smooth, piecewise, discontinuous, or too noisy to classify?
- **E. Robustness:** is there a strategy that gives up a little peak performance for reliability
  across heterogeneous blocks and hardware?
- **F. Adaptation:** would block-local strategy selection beat the best fixed strategy after its
  overhead is counted? If selecting requires the very construction it would avoid, name that
  circularity explicitly.
- **G. Falsification:** what empirical result would show the low-dimensional theory wrong, or
  not useful?

## Part IV: the minimum decisive experiments

Don't propose an indiscriminate sweep. Design a staged plan that maximises information per unit of
benchmarking cost. Prefer archived measurements where they suffice. Include:
1. real and synthetic block families with measurable structural differences;
2. Wikidata's identifier-heavy blocks against structured ones;
3. several contraction configurations, where the code supports them;
4. construction and walk stage measurements;
5. low and high concurrency;
6. resource-pressure measurements, where available and reliable;
7. hold-out blocks, and at least one hardware regime not used for fitting.

For each experiment, give:
- the hypothesis;
- the variables held constant;
- the intervention;
- the observations expected under each competing hypothesis;
- the measurement method;
- the precision needed;
- the decision rule;
- what a negative result would mean.

Separate experiments possible with existing evidence from those needing new resources. The i9 is
the maintainer's workstation: runs on it are frozen recipes he runs himself. Never time on it,
and don't assume access to it or to hardware counters.

## Part V: deliverables

1. **The mathematical lineage:** what each field contributes that algorithm analysis and
   profiling don't.
2. **A mechanistic model:** its assumptions, observables, unknown parameters and identifiable
   predictions.
3. **Reduced performance coordinates,** and how to test whether they keep predictive power
   across workloads and hardware.
4. **A strategy phase diagram:** where each decoding strategy is optimal. If the evidence can't
   support one yet, give the smallest experimental plan that would.
5. **Falsifiable predictions and priorities:** the three most informative experiments, each
   with a concrete prediction and a competing hypothesis that would give a distinguishable
   outcome. End with a go/no-go assessment of whether this programme is justified by the evidence
   available now.

## Standards of reasoning

Be mathematically adventurous but empirically disciplined. Label every claim as one of:
- proved;
- measured (with its machine);
- published (cited);
- model-dependent prediction;
- speculative transfer between disciplines.

Don't manufacture a unified theory because the analogy is appealing. Treat rejected optimisations
as evidence. A simpler model that predicts reliably beats an elaborate one that explains everything
in retrospect. Don't claim a low-dimensional manifold unless the data support it. Don't confuse
correlation with causal identification, or resource saturation with a universal lower bound.
You may reject the systems-biology analogy altogether, if the evidence warrants it.

The most valuable result would be a theory that predicts the failure of an apparently promising
optimisation before it is implemented.

## The deeper objective

A proof may admit a compact route to correctness. A high-performance implementation lives in a
heterogeneous environment of data, memory hierarchies, contention, scheduling and resource limits.
Its optimum may not be one algorithm but a collection of strategies, each winning in some states.

Can that complexity itself be compressed? Not by ignoring the physical constraints, but by finding
a smaller representation that keeps exactly the constraints responsible for performance. Can a
mathematically simpler theory of the decoder give better predictions, better experiments, and in
the end better executable algorithms?

## Working rules for this repository

- **Where to write:** commit only to your branch, `research/performance-landscape`. Write under
  `docs/journal/`, and put scripts and data under `docs/journal/<your topic>/`.
- **What not to touch:** `src/`, master and other branches. Open no PRs and run no
  benchmarks on shared machines.
- **Naming:** name things by what they do, never after a team, model or round.
- **Handoffs:** give interim handoffs as you go, and end with a final handoff: a substantial
  technical report, not a generic survey. If the evidence can't answer the central question,
  say exactly what's known, what remains unidentifiable, and which smallest decisive experiment
  comes next.
````
