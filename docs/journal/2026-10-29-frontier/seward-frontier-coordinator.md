<!--
Provenance: the coordinator chatbot's reading of the researcher's frontier proposal
(seward-frontier-research.md), relayed by the user into the working session on
29 October 2026, unedited. Its two charts arrived as captions only, and its link as a
title followed by the site's name. The instruction it drafts for the working session is
part of this text; the session acts on it as the user relayed it.
-->

I think this deserves to become a permanent part of mibz2’s benchmarking programme. The researcher has identified a compelling architectural question, and it connects directly to the cache-capacity effects we’ve already observed on your i9-10900K.

The distinction I’d preserve is:

We have evidence for the mechanism, but we have not yet demonstrated an outward shift of the throughput–memory frontier.

That’s an excellent reason to start tracking it now, before we know which representation wins.

I also agree with your preference for releases as the main historical record, with selected PRs shown as experimental candidates. That gives us a useful performance history rather than an ever-changing research dashboard.

How I’d structure the benchmarks

1. Throughput versus concurrent blocks

Shows whether compact representations overtake direct T-vectors as simultaneous table residency increases.

2. Throughput versus memory

Shows which configurations are nondominated when measured throughput and peak memory are evaluated together.

Both charts are conceptual illustrations only. They depict the hypothesized crossover, not measured mibz2 results.

The page should also allow readers to distinguish table footprint from actual peak RSS, since those measure different things.

My recommendation: make this a named benchmark objective

I’d call it Throughput–Memory Frontier, and make it a permanent part of the documentation site, linked from the existing benchmarks.

The historical framing is sound: Seward’s 2001 paper explicitly investigated inverse-BWT space–time trade-offs and identified cache-miss servicing as a fundamental speed constraint. researchgate.net

Our hypothesis adds a different optimisation objective: aggregate throughput under shared-cache pressure.

That is worth documenting independently of which compact representation ultimately wins.

How I’d make it maintainable

Rather than generate a new page manually for every release, I’d ask Claude to build a small reproducible benchmark pipeline:

* Save machine-readable observations with version, commit, CPU, workload, concurrency, elapsed time, navigation-table bytes, peak RSS and CPU usage.
* Render both plots from those observations, keeping each hardware/workload combination separate.
* Identify nondominated configurations automatically.
* Make release measurements the default public series, with selected experimental PRs appearing as clearly labelled overlays.
* Never plot modelled 16-bit designs as measured results.

One important refinement: the headline frontier should ultimately use full-decoder throughput, because that’s what users care about. Build-plus-walk should have its own component-level analysis to expose the cache-residency mechanism.

And a technical caveat: a faster, lower-memory configuration can demonstrate a Pareto improvement over another configuration, but we should reserve the claim that the frontier has shifted until it beats the relevant best alternatives under the same experimental conditions.

The instruction I’d give Claude

I’ve revisited Seward’s 2001 inverse-BWT space–time trade-off with the researcher, and we’d like to preserve a specific overarching research objective: shifting mibz2’s aggregate throughput–memory frontier through improved shared-cache residency.

We have measured the cache-capacity constraint and Ψ’s compressibility, but have not demonstrated a compressed representation that improves whole-machine throughput. Please preserve that distinction.

I’d like a permanent Throughput–Memory Frontier benchmark page on the documentation site, with two principal plots:

1. Throughput versus concurrent full blocks, to identify cache-residency crossovers.
2. Throughput versus measured memory, showing nondominated configurations and whether releases or selected experimental PRs improve the frontier.

Please use releases as the primary historical record, selected PRs as clearly labelled experimental overlays, and preserve the underlying machine-readable results. Separate comparisons by CPU and workload; distinguish navigation-table footprint from peak RSS, and full-decoder results from build-plus-walk measurements. Keep theoretical footprints and hypothetical formats out of measured plots.

The initial page can document the question, methodology and any existing comparable measurements. Don’t invent a compact-versus-direct performance result that we haven’t measured.

Please assess how to extend the current benchmark harness with minimal duplication. The cooperative-decoder integration remains the main engineering priority, so implement this documentation/benchmark infrastructure in small steps rather than letting it block the next performance release.

The long-term success condition is a measured configuration that achieves both greater aggregate throughput and lower memory usage than the best relevant direct-table alternatives on the same machine and workload.

I wouldn’t ask Claude to invent lots of new tests immediately. First, it should audit which existing benchmark results can populate the page without mixing unlike measurements.

Then the plateau census, marker audit and eventual compressed-successor experiments can supply new candidates.

This is the right long-term research question. We’re no longer merely asking whether we can save two bytes per row; we’re asking whether saving them changes the optimal way to use the entire machine.
