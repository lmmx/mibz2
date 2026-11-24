# Brief: the decoder's full character. A finer decomposition, measured without an exponential grid

*Louis, 24 November 2026, ~22:15: "we are analysing it in a simple form, no? 'doormen vs assemblers'? is that the true decomposition here, is there any way we could do so further so as to analyse this in such a way we would thus be able to tell the properties better? either more than these 2 phases or splitting the phases somehow, etc. we want to avoid this blowing up into a huge exponential run on 1-20 threads but we also want to establish the full character. recall the 'systems biology' analogy made in the DM blog post. i think this is worth putting to monodromy, chatgpt pro, antigravity neon etc"*

*The analogy, from the conversation post of 22 November (note 18): systems biology is "the study of living systems as networks of interacting parts, through quantitative models with many parameters fitted to data, rather than one mechanism at a time. Louis wants the same kind of modelling for the many knobs of a real decompressor's performance."*

## Why the two-stage picture is too coarse

The four-arm oracle (journal §44) split the decoder in two: the serial scanner and the k workers. Inside the workers sit four quite different stages:
- MTF/RLE2;
- construction of the inversion (Open's runs, retained map and induction);
- emission;
- RLE1/CRC and output.

Beyond the stages, there are **shared resources** that couple them at high worker counts:
- memory bandwidth;
- the last-level cache;
- the allocator;
- the bounded job queue;
- hyperthread pairs on the 10-core, 20-thread i9.

A two-stage model cannot say which of these sets the curve at each k.

## A candidate decomposition, for the members to correct

| # | stage or resource | where | measured today? |
|---|---|---|---|
| S0 | input read (I/O, chunk recycling) | reader thread | partly (#838 input waits) |
| S1 | block boundary and header parse | scanner | #838 (header, bitmap, selectors, tables) |
| S2 | Huffman decode | scanner | #838 (decode phase) |
| S3 | hand-off and queue | scanner → workers | #838 (slot waits, hand-off, worker idle) |
| S4 | MTF/RLE2 | workers | no |
| S5 | construction (runs, retained map, induction) | workers | no |
| S6 | emission (the walk over the induced structure) | workers | no; its oracle ceiling is in §44 |
| S7 | RLE1, CRC, output assembly | workers | no |
| R1 | memory bandwidth and LLC | shared | no |
| R2 | allocator and page faults | shared | #838 adds RSS, faults and switches per decode |
| R3 | SMT pairing (k > 10 shares cores) | shared | implicit in the 1–20 grid |

## Avoiding the exponential: three linear methods

The grid is 20 worker counts. Crossing every stage on or off against every count is exponential in the number of stages. Each of the methods below costs a number of runs **linear** in the number of stages.

1. **Instrument every boundary, once per count.** Timers on S0–S7 and counters for R1–R2 (LLC misses and bytes moved, via perf or uncore counters where available) give each stage's busy time, waiting time and resource use at every k from one run per count. This extends #838 to the workers.
2. **Control coefficients: the systems-biology tool.** In metabolic control analysis, a stage's *flux control coefficient* is C_i = ∂ln J / ∂ln v_i. It measures how much the pipeline's throughput J changes when stage i alone is made slightly faster or slower. Make each stage slower by a small injected amount, about 5–10% of its own time, one stage at a time, at a handful of counts. The coefficients show which stage controls throughput at each k.
   - The **summation theorem** (the coefficients sum to 1 for a pathway's flux) is a built-in check that the decomposition is complete. A shortfall means a missing stage or resource.
   - Cost: stages × chosen counts × rounds. That is linear, and only *slowing* a stage is needed, which is always possible.
   - The four-arm oracles are large-perturbation versions of the same idea (stages set to zero cost). Small perturbations give the local slope instead, and are cheap at every stage.
3. **A fitted model, validated on held-out counts.** Fit a queueing-network model to the instrumented service times and the shared-resource terms: a serial stage feeding k parallel workers, with stage times that rise with contention. Train on some counts, predict the others. Report the residuals; where the model fails, a decomposition element is missing.

## What we want from each member (MONODROMY, ChatGPT Pro, Neon)

1. Is the table above the right decomposition? What is missing or wrongly merged? Consider, for example:
   - per-block heterogeneity: block size, h₁, alphabet;
   - NUMA and frequency scaling;
   - the output writer;
   - cold against warm caches.
2. The minimal experimental design that "establishes the full character" across k = 1–20, without an exponential grid:
   - which counts;
   - which perturbations and of what size;
   - how many rounds;
   - which counters.

   Say what precision it buys.
3. Which properties should be *reasoned about*, rather than measured? For example:
   - bounds from the summation theorem;
   - when a serial stage must dominate (Amdahl, Gustafson);
   - when memory bandwidth must saturate, from bytes per output byte and the machine's bandwidth.
4. Whether control coefficients are the right formalism here, or whether something else fits better: elasticities, sensitivity analysis, causal profiling (Coz), roofline models.

## Process

- MONODROMY answers in a record. ChatGPT Pro and Neon answer by paste or on their branch.
- The Coordinator compares the answers and turns them into a recipe brief for the Pathfinder, after #838's profile has run.
- Measurements go into `math-targets/data/` as TSV, with their scripts.
