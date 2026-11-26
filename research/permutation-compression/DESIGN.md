# Adaptive permutation compression: a reusable discovery, construction and selection library

**Owner's revised investment criterion (26 November 2026):** no standalone implementation programme solely for general-purpose compression or memory savings. First identify a concrete computational advantage for mibz2, with a credible construction-plus-consumer budget and a measured relevant improvement over matched controls. The separate decoder investigations establish that case; the standalone package then generalises the useful mathematics while preserving its independence. Until that application case exists, keep this design as an unadmitted idea, not an implementation task. The scoped #998 STOP stands; do not reinterpret a storage-only result as a decoder win.

## 1. Origin and intent

The maintainer wants to generalise permutation compression itself: both finding useful mathematical structure and constructing an efficient representation. Incubate independent software within this repository, release it separately when well defined, and never allow it to become intermingled with mibz2. This is analogous to incubating the standalone mibz2 crate within reshape. It also follows Vidit Nanda's approach to effective mathematical software, not a request to port Perseus or Helmer into the decoder.

The earlier [software survey](../../docs/journal/2026-11-24-algorithm-level/briefs/antigravity-software-survey-prompt.md) sought existing mathematical packages whose fast native implementations would materially help the decoder. Its [ranking](../../docs/journal/2026-11-24-algorithm-level/leads/research-software.md) did not find a compelling direct port among the tested candidates; Nanda's work was the methodological inspiration. His [Perseus 6](https://people.maths.ox.ac.uk/nanda/perseus/index.html) exemplifies a rewrite of exact mathematical software. Apply that style to permutations in their own right: identify useful existing mathematical operations, verify them against the original software or independent references, make the important ones efficient natively, and compose them into a workload-aware system. This is not a commitment to port Permuta wholesale, and the maintainer has not yet used Permuta.

The [permutation-patterns discussion](../../docs/journal/2026-11-24-algorithm-level/permutation-patterns/chatgpt-response.md) contains the original detailed literature leads and the distinction between pattern structure and iteration. The new project is a generalisation, independent of the decoder-specific pattern and ideal-structure experiments.

The maintainer explicitly wants complexity metadata to be *input to the algorithm that selects representations*, and experiments on different kinds of permutations, e.g. LF of Wikidata versus ordinary text. Abstract entirely from decompression and from Rauzy–Veech induction. General usefulness is not sufficient reason for this project to fund implementation; the application must benefit first.

## 2. Mathematical object and contract

Input is a finite bijection p:[0,n) -> [0,n), initially a zero-based integer array. A BWT last column with repeated symbols is not itself a permutation; its stable-rank LF map (or inverse) is. Begin with explicit permutation vectors; later consider generic validated permutation sources with declared access/acquisition costs. Validate length, range and uniqueness; keep validation cost visible. Define n=0, n=1, arbitrary cycle structures, invalid input and integer-width limits. No assumption of primitive text, a marked row, BWT, alphabet, or interval exchange. Every representation preserves the original-coordinate bijection, not just its cycle type, a pattern or a conjugate. If new coordinates are used, retain and price the maps required to recover the requested operations.

Separate:
- Discovery: certify or estimate structure from the input, with a budget.
- Representation: exact encoding of the permutation.
- Construction: materialise that encoding from the input, charging all necessary work.
- Evaluation: forward/inverse queries, dependent iteration, batches, powers or serialization where supported.
- Selection: choose the analysis and construction worth paying for, given workload and resource constraints.

A storage-only compressed file requiring full inflation is a possible comparator, not automatically an efficiently navigable representation. Orbit-order coordinates are legitimate if all mapping/index costs are charged. A worst-case exact encoding of arbitrary permutations requires at least log2(n!) bits; no universal promise of compressibility or fast queries. Label costs and proof scope explicitly.

## 3. Absolute package boundary

Use research/permutation-compression/ as an independent Rust crate, with its own Cargo.toml, src/, tests/, generic benchmark executable or benches/, README, license/provenance and lockfile policy. Name is provisional; publish=false during incubation. No registry reservation, release or new repository until separately authorised. No commitment to a stable public API yet.

The library must never depend on mibz2, import its modules, assume BWT properties, or use decoder-specific traits. Conversely, the root crate must not depend on the incubator, even behind a feature, within this programme. Do not alter the root Cargo/workspace/build/CI/bench/release files. All data extraction and application adapters live OUTSIDE the package, e.g. research/permutation-compression-data/, and export a generic permutation dataset. No decoding inside the library. No shared global state, include! of parent paths, symlinks to parent code, parent-relative path dependencies or mandatory access to the host checkout.

Build with cargo --manifest-path research/permutation-compression/Cargo.toml. Test by copying the directory to an unrelated temporary location and building/testing independently. Verify the root dependency graph has not changed and inspect root cargo package --list for accidental inclusion of the nested package; if exclusions need changing, record a separate reviewed change before release, not a silent production manifest edit. The eventual extraction should amount to moving a directory. The interchange format, not shared source code, is the boundary. Python bindings may come later after the Rust interface and useful primitives settle. A future mibz2 adapter or dependency is a separate adoption decision; it is not part of this programme.

## 4. Literature and software: Nanda-style operation-first approach

Do not assume there are no existing competitors or that a port will be faster. Survey existing implementations, precise API contracts, mathematical preconditions, complexity, observed costs and licensing. Use stock tools first as comparators or exactness oracles where useful. Identify which operations support discovery, construction and navigation and which merely enumerate a class. Port/reimplement only useful operations for which the native version has a justified purpose. Prefer safe Rust, a small dependency surface, provenance and citations. Respect licenses and clean-room requirements; the license of a paper does not necessarily license implementation code. No new paid compute/infrastructure.

Starting points (not a mandated port list):
- [Permuta](https://github.com/PermutaTriangle/Permuta): Python library for patterns, avoidance classes, statistics, symmetries, sums and mesh-pattern discovery. Public README documents BSD-3-Clause licensing. Examine relevant exact pattern tests, structural operations and controlled generation; determine what scales and whether efficient native equivalents already exist.
- [Kozma–Opler, Compact Representations of Pattern-Avoiding Permutations](https://doi.org/10.4230/LIPIcs.ESA.2026.92): fixed-pattern avoidance gives compact exact representations and constant-time forward/inverse navigation in the stated model. Check construction, auxiliary tables, constants and hypotheses before implementing.
- [Brignall, A Survey of Simple Permutations](https://oro.open.ac.uk/23475/1/simples-survey.pdf): common/strong intervals, substitution decomposition, irreducible pieces.
- [Barbay–Navarro, On Compressing Permutations and Adaptive Sorting](https://arxiv.org/abs/1108.4408): run/subsequence-sensitive storage and navigation.
- [Chakraborty et al., ISAAC 2024](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ISAAC.2024.17): separable permutation representations.
- [Even-Zohar–Leng, counting small patterns](https://arxiv.org/abs/1911.01414) and [Combinatorial Exploration](https://arxiv.org/abs/2202.07715): potential discovery or controlled-generation techniques.
- [SDSL](https://github.com/simongog/sdsl-lite), existing rank/select, succinct permutations, bitpacking and adaptive compression implementations: compare actual available tools and whether any already solve representation selection. Avoid unnecessary reimplementation.

The previous survey's direct decoder-port stop results stand. This is a different, general mathematical-software problem.

## 5. Candidate representations and discovery

Start with an explicit array baseline (inverse array separately priced), bitpacked baseline (ceil(log2 n) bits per entry with small-n cases), and one justified structural encoding. Potential families:
- runs of translations (p(i+1)=p(i)+1), affine/reverse intervals and compact offsets;
- monotone runs or interleaved monotone subsequences, rank/select navigation;
- strong-interval/substitution trees, direct/skew sums, separable pieces;
- repeated small permutation shapes, grid/block decompositions, exception dictionaries;
- cycle-coordinate/power structures and bounded-class succinct methods.

These are alternatives for a representation registry, not an MVP requirement to implement everything. Each candidate declares an applicability predicate/certificate, analyser, constructor, operations and cost metadata. Possible features: run-length entropy, LIS/LDS, pattern counts, decomposition depth/size, displacement, repetition. Each feature costs time; a statistic is useful only if it supports a decision, certificate or representation. Sampling may find a forbidden-pattern witness but cannot establish exact avoidance. If applicability is unknown, use exact recognition, a proved property, explicitly encoded exceptions or fallback.

Use progressive analysis: cheap scans/estimates first; expensive recognition only when possible savings justify the budget; build a selected candidate or account for trial builds and discarded work. The search may be expensive offline, but online analysis is not free. Do not enumerate all permutation classes on every input. Prefer simple selection rules until evidence justifies a more complex learned policy. Keep a universally valid fallback.

## 6. Complexity metadata is executable input to selection

The distinctive software hypothesis is a composition of certified applicability, complexity descriptors and measured calibrated costs, not just a collection of encodings. Propose machine-readable metadata in Rust and a portable benchmark/report schema. Exact field names are provisional:
- family ID, version, source/citation, license and implementation provenance;
- structural preconditions, recognition algorithm, certificates, parameters (n, r, k, forbidden pattern, class growth, depth, etc.), known/unknown applicability;
- symbolic time/space expressions for discovery, construction, forward/inverse, iteration/power, batches and serialization, with model assumptions, worst-case/expected/amortised distinctions, unsupported operations explicitly tagged;
- persistent bytes including indexes, labels if applicable, metadata and coordinate maps, peak temporary bytes, validation/construction cost, calibrated query latency and throughput, machine/compiler/version, cache regime, uncertainty and validity domain;
- discovery budget consumed, estimates versus actuals, extrapolation status, exactness evidence and fallback.

Big-O is not a wall-time estimate. Preserve theorem-derived bounds separately from calibration; an upper bound exceeding a memory limit does not prove infeasibility. Unknown costs are not zero. Account for shared tables and simultaneous instance footprints. Report full allocated/persistent bytes, not just an ideal bit count.

The caller supplies a workload: number of forward/inverse queries, sequential/random pattern, dependent chain versus independent batches, orbit length or power distribution, build once/use once versus reuse, memory cap, construction cap and required operations. Return chosen representation with applicability evidence, predicted and measured costs when available, reasons for rejecting alternatives, manual override and a Pareto report.

Illustrative first model for a representation R:
    T_R = D_R + C_R + sum_j N_j * t_R,j
where D is analysis/selection (including losing trials), C is actual construction, N_j the operation count, t_R,j calibrated cost. Batches, dependent access, cache state and overlapping work need actual workload-specific measurements; the expression is not a proof of elapsed time. Minimise paid time subject to resource limits or return the nondominated frontier of build time, query time, persistent and peak memory. For one query type, a build/discovery premium B breaks even after B/(t_baseline - t_candidate) queries when the denominator is positive. Separate use-once, many-query and memory-constrained policies.

## 7. Data and benchmark independent of decompression

Only the permutation is input to the library and timed benchmarks. No bzip2 decode, Rauzy induction, return-word spelling, CRC, dispatch or decoder throughput in these measurements. Use a documented generic interchange (e.g. versioned little-endian integer vector plus manifest: n, element width, orientation, SHA-256, source/license, generator version, family, seed). Validate or explicitly tag prevalidated inputs. Keep large data out of Git and record reproducible acquisition instructions. An external adapter may export real LF/FL from existing authorised corpora. The generic harness never knows how a map was produced.

Compare different permutation families, not just one dataset:
- raw LF and inverse maps from Wikidata blocks versus ordinary text/prose; where available random bytes or other corpora, size-matched. Interpret the maintainer's "Wikidata vs text split" as separate data cohorts, not a new decoder benchmark;
- identity, reversal, rotations, blockwise translations, random uniform permutations and random single cycles (distinct distributions);
- known separable/pattern-avoiding constructions, monotone interleavings, repeated local shapes, sparse exceptions, irregular/simple pieces, adversarial near-misses;
- at least one non-BWT-derived family such as suffix-array permutations, graph reorderings or stable ranks of generic keys. Label synthetic data honestly.

Cover exact small exhaustive inputs and sizes up to cache-relevant regimes (roughly 900k elements is relevant to the host project, not a package limit). Distinct train/held-out instances, grouped by source identity, repeated blocks and derived inverse/symmetry variants to avoid leakage. Where feasible hold out whole families. Deterministic seeds, comparable sizes, interleaved repetitions, identical query schedules. Measure persistent bits/element, compared with log2(n!) and packed baseline, peak live memory, discovery, validation and construction separately, independent random throughput, dependent latency, inverse and sequential evaluation, batches and power queries if supported. Consume query results (checksum/black_box), verify exactness against array; exclude acquisition/I/O from timed operations and report it separately. Separate warm/cold and prebuilt/paid measurements. No extrapolation from VM to other hardware without validation.

Compare (1) fixed array/packed baselines, (2) fixed structural representation, (3) free offline selection of best feasible candidate in the implemented finite family, charging actual materialisation/query costs but not selection, (4) actual adaptive selection paying all discovery, validation, rejected trials, construction and query costs. Also compare prebuilt representations to isolate product quality. Report held-out selection regret versus finite-family oracle, not universal optimum. Include cases when selector picks fallback; do not bury family regressions in an average.

## 8. Bounded initial implementation and decision

One claim, one session: preregister a feasible first slice; implement small standalone crate with explicit validated array, packed baseline, one justified structural candidate (e.g. translation runs), a real query consumer, metadata and simple workload-aware selection. Reference Permuta or an independent simple oracle for structural checks as useful. Provide generic reader/generator, exactness tests and reproducible comparisons on at least structured and random families, plus raw LF from Wikidata and text when permitted corpus data is available. If absent, document the export contract and real-data gap; use deterministic fixtures without pretending they are real corpus results. No full research-library port in the first slice.

Freeze candidate families, sizes, operation profiles, train/test split, repeat design, local compute budget and criteria before timing. First viability: at least one held-out regime with 20% lower persistent bytes at no more than 10% paid build+query time regression, or 10% paid time improvement at no higher persistent/peak memory, relative to strongest matched supported baseline. Report selector regret, including overhead, separately. Target at most 5% paid-time overhead on designated random fallback profile beyond common validation. Report failure scoped to the tested representation and profile; keep reusable harness where useful. Follow-on claims require evidence and a new bounded mechanism. No bzip2-specific 39.18 ms gate. No claimed novelty or speed until measured.

Tests: exhaustive small permutations (e.g. n<=7), deterministic large random and structured inputs, malformed input, boundary sizes, multicycle cases, forward/inverse identity, every supported query, serialization if implemented. No silent full materialisation for unsupported queries. Safe Rust initial slice. Test copied crate outside repository; verify root graph and package boundary. Report any needed root packaging exclusion as a separate reviewed change. Deliver the scaffold, metadata design, source comparison, generic benchmark recipe/results, selection explanation and delta.

Potential follow-ons, each separate claim as evidence warrants: additional representations, exact recognisers/constructors, metadata and selector calibration, larger corpus study, Python bindings, extraction and release planning. Publishing, new repo and application adoption remain separate maintainer decisions.

## 9. Relation to existing research

- ideal-permutation-structures (#985): exact labelled return-map/grammar products for a decoder consumer; keeps its frozen decoder-specific gate.
- permutation-patterns (#989): pattern-derived structures in retained maps and labelled decoder evaluation; keeps its own gate.
- permutation-compression-incubation: general mathematical software on explicit permutations, arbitrary operation workloads. Neither other result is a prerequisite. Share mathematical insights and generic fixtures, not decoder harnesses, source dependencies or assumptions. Discoverable artifacts and compatible contracts, not session routing.

The package must be justified by concrete mibz2 computational benefit first, even though its eventual interface and benchmarks remain application-independent.
