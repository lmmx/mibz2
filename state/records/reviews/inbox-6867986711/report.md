# Default integration attribution after matched i9 Cells evidence

Brief [6867986711](https://github.com/lmmx/mibz2/pull/160#issuecomment-6867986711); 2026-11-14. Desk/source assessment under SPEC and SPEC.OPTIMIZER at a2da49638ac54d7763de1c167bd054f67ffdc763. No implementation, compilation, decoder execution, timing, registration or admission in this review.

**Rank 1: commission one bounded, untimed default-executable correspondence preparation, before considering another timing experiment.** A three-source timing comparison could localise a total source-package effect to master→common or common→selective. It cannot identify codegen/layout as its cause. Existing source inspection has already excluded an obvious newly executed default operation; repeating that pass or adding timing samples has lower decision value than identifying an actual changed default executable region. This is a proposal for Coordinator commissioning, not a new run or gate.

## Evidence and reconciliation

| Identity | Pin |
|---|---|
| M: master baseline | 3e7e294cc10f29495e9828181345d74233c549d4 |
| O: original Cells, common integration tree | be25366f46458e2f1dbb0bdac1f5fbc3366d0648 |
| S: selective Cells | a8a23c11607864cb431f87fff38169e10c41685f |
| i9 recipe | cf1a36ed429f361839fab6748383187104d8ab27 |
| i9 evidence | e04fe229e9d939840d50fea110c7b7c6f74811ef |

[Submission6867819559](https://github.com/lmmx/mibz2/pull/145#issuecomment-6867819559), [Coordinator6867899861](https://github.com/lmmx/mibz2/pull/145#issuecomment-6867899861), and [D104](https://github.com/lmmx/mibz2/blob/d5ee317ad73962e35d6dbdd1804a372a6b92abbb/state/reports/D104-i9-machine-qualification.md) establish the new matched result in their checked scope. All252 observations remain. Wikidata selective/original wall0.9256 [0.8823,0.9711], selective/Segmented0.8630 [0.8319,0.8952]; text level. Pattern selective/Segmented1.4352 and Wikidata RSS1.3107 remain costs. D104 resolves the pre-start machine/activity hold only; the coarse high post-start estimate neither proves a frozen-gate violation nor absence of interference; null throttle counters are not zero.

These figures are attributed to those completed verifications, not a raw-bundle replay by this review. The submitted bundle is 60,935,434bytes, SHA256330ff61ce5e2659c8f0a86837d26eab274d0eb1fbe4e5563d3f3824f92912fa7. The i9 comparison holds the surrounding source tree common and compares explicit readers on one CPU. It cannot answer the distinct whole-default integration comparison at4threads.

The [first CI report](https://github.com/lmmx/mibz2/blob/3dd3e33f32c32bb3eb266a1f90032cacb30d79bf/state/reports/cells-selective-6856176834/PERFORMANCE.md) and [REVIEW6857539994](https://github.com/lmmx/mibz2/pull/145#issuecomment-6857539994) retain default S/M wall losses random2.79%, text3.87%, Wikidata2.30% beyond reported noise. Frozen gate6775049636 fails despite green workflow. PR380 remains integration DEFER; prior failed primary results and consumed rerun history are unchanged.

I independently inspected both GitHub source comparisons. M→O changes src only by new cells.rs, Cells reexports, module/accessor additions and a read-only Runs accessor. O→S changes cells.rs and two test-only files; the selective branch belongs to Cells induction. This agrees with [completed attribution3c61b203](https://github.com/lmmx/mibz2/blob/3c61b203730049a93b405ec8e35de53b2408dc08/state/reports/cells-i9-20261113/ATTRIBUTION.md), completed through [6862561316](https://github.com/lmmx/mibz2/pull/145#issuecomment-6862561316). No justified default correction follows. Unchanged default source is not proof of unchanged machine code.

**Additional evidence gap found here:** [codegen.py at S](https://github.com/lmmx/mibz2/blob/a8a23c11607864cb431f87fff38169e10c41685f/bench/cells-selective/codegen.py) extracts the two experimental `Kernel::induce_cells` listings. Its904→827byte finding is not a default-path listing. [The exact workflow](https://github.com/lmmx/mibz2/blob/a8a23c11607864cb431f87fff38169e10c41685f/.github/workflows/bench.yml) writes default .text hashes and compares them, but its Results upload omits bin/ab, base/ab and .codegen/*.text; selective-codegen/ is uploaded. Thus the declared artifact set is insufficient for retrospective default disassembly. I did not enumerate every historical artifact to assert that no copy exists anywhere. Whole-section hash inequality establishes nonidentity, not reachable-code change or its performance effect.

## Estimand: localisation is not mechanism

For a genuinely matched future three-source experiment, define each per-corpus paired contrast on identical compressed bytes and the same four-thread default reader:

- d_OM = mean(log T_O − log T_M);
- d_SO = mean(log T_S − log T_O);
- d_SM = d_OM + d_SO, using the same rounds and weights.

These are whole-reader, whole-source-package effects under the specified compiler, harness and host. Original Cells versus selective Cells induction is a different estimand. A nonzero d_OM with near-zero d_SO would localise the default association to inherited integration; a nonzero d_SO would localise it to the later package. Neither establishes which generated instruction, address placement, data object or build input caused it. Near-zero must mean an interval inside a frozen equivalence band, not merely an interval containing zero. No new band is authorised here.

Confounders/mediators to distinguish: compiler and linker versions, target/features/RUSTFLAGS, LTO and codegen units, incremental/cache contamination, build path and section ordering, harness/protocol/source-generated corpus changes, effective argv/environment/sampler, runtime libraries/allocator, ASLR and placement, frequency/thermal/activity state, run order, thread/SMT placement and shared-resource contention. A source comparison bundles its codegen and layout consequences; statistical matching cannot unbundle them. Three separate CI hosts would add host effects. The old i9 and AMD ratios cannot be multiplied or subtracted into an integration estimate.

## Minimal preparation and stopping rule

One deliverable: a default-build identity and difference map for M/O/S, sufficient to nominate a falsifiable mechanism or state exactly why none is identifiable.

1. Inventory retained default binaries/build manifests first; use them if identities are provable. Check actual default build cfg, effective harness command, corpus hashes and dependencies against the failed CI. Do not silently substitute the experimental Cells binaries.
2. If the originals are absent, preparation would require at most one clean untimed build per source with a common pinned toolchain/profile/flags and isolated targets. This is **separate future commissioning**, not executed or admitted here. Capture executables, loadable sections, relocations, symbols, disassembly, build commands and hashes. Preserve raw addresses as well as normalised instructions. Reconstructed builds must be labelled as such: they do not retroactively recover the failed job. Try to match saved .text digests; mismatch limits historical attribution.
3. Compare the default harness and reachable decode/worker closure, including scanner, MTF/RLE, inverse BWT, CRC, allocation and scheduling interfaces; record reachable immutable-data and layout differences too. Unknown indirect calls or stripped/inlined symbols stay unknown. Do not turn missing symbols into a claim of equality. Pin any candidate difference to a source transition and state how it could bind whole-reader completion at4threads.
4. Return one named mechanism and a proposed isolating intervention, or “no sound discriminator”. Changing source module inclusion is not automatically a layout-only intervention; all induced code/data differences would need accounting.

Proposed resource ceiling: one artifact inspection pass, at most three untimed builds in one preparation job capped at60minutes, and one engineering inspection pass capped at90minutes; zero decoder executions, zero performance samples, no retries or flag/search matrix. These are stop limits, not runtime predictions or spending approval. Missing toolchain, absent provenance, build failure or unresolved correspondence ends the preparation with that fact. No automatic extension or workstation queue entry.

**Fixed decision rule:**
- No provable historical correspondence: report a reconstruction only; no historical causal claim.
- Fully accounted equal default-reachable code/data and relevant placement under equal invocation: reject changed-default-executable as the explanation for that compared pair in that build context. A normalised opcode match alone is insufficient.
- A relevant difference exists: it is a necessary lead, not proof of causal cost. A mechanism becomes testable only when a narrowly specified intervention can change it while documenting other consequences and predicts which default corpora/thread count should respond. Coordinator may then separately commission that falsifier under existing gates.
- Differences only outside the reached closure, unresolved mapping, or no isolating intervention: no decision-enabling causal experiment has been established. Stop; do not request more samples to cure the identification failure. PR380 stays DEFER.

This revises the earlier suggestion that a three-source timing comparison would itself be “decisive”. It can be useful for version localisation, but that is not yet an integration-cause discriminator.

## Price and priority

Eliminating only the reported default excess would recover about2.71% of current random wall,3.73% of current text wall and2.25% of current Wikidata wall (loss/(1+loss)). Those are point-estimate parity budgets, conditional on the excess being causal and removable; not hard speedup bounds. No independently attributed portion exists yet. Instruction/section sizes supply no missing runtime fraction.

The i9 experimental gain is material:7.44% incremental Wikidata wall,13.70% against Segmented in that matched context. It justifies preserving the line. It does not pay the pattern43.52% or RSS31.07% costs, or grant a default dispatcher. No weighted production win can be computed without a workload and binding-worker account. More exploratory timing is economically inferior now to the finite executable preparation: the immediate decision is whether a native integration correction is even identified, not whether another experimental confidence interval can be narrowed. No evidence here ranks Cells above all other production fronts. MONODROMY/Pathfinder ownership, team/cap/default policy and completed independent B5 stay unchanged.

Routing audit used the pinned helper over connector snapshots: five historical resolved entries and the known Researcher LOG6757065549; no new misroute. No repair action. Migration checked against legacy e6eea5889432aef28f4d9e450cee87dd5bf7bacf and manifest SHA25686d7a8a6ea42c2d2ade70c5ce3f0c82f375c0fd6291dbf452979bf07b100dbde. Receipt4324f9d0e930e945b91f20ed0b07184d6ec3b641 precedes this work.
