# Decoder system: the established research baseline

Start here before choosing or pricing a performance investigation. This is a short living read-back of the established architecture, not a new mathematical model, acceptance rule, experiment or review process. The original [hybrid-system analysis, 25 November](../docs/journal/2026-11-24-algorithm-level/reading/hybrid-dynamical-systems/relevance/decoder-as-hybrid-system.md) is the conceptual source; [surface.tsv](surface.tsv) maps local opportunities. Update this page when evidence materially changes the system picture, not for every probe.

## One coupled computation

```text
 serial Huffman scanner S
       |  admission guard: in-flight slots / ready bytes
       v
 job queue --> k parallel workers: M + C + E + R
       |        shared caches, memory, contention
       v
 ready blocks --> in-order reader D --> output
       ^               |
       +-- credit / recycled buffers --+
```

The original hybrid model H=(C,F,D,G) uses continuous service progress and discrete admit, issue, start, finish, take and consumed events. Admission is checked *before* scanning; issued advances after scan; taken advances before reader copy. Content-dependent routes, queue backpressure, ownership and ordering couple the stations. Contention affects service rates; remaining work is not directly observed. Do not confuse the hybrid model's sets C/D with decoder construction/delivery labels.

For fixed demands the operational lower bound is max(scanner service, total worker service/k, reader service). It is **not** an equality, predictor or upper completion certificate: dependencies, indivisible jobs, starvation, blocking and contention matter. The original analysis explicitly corrected this distinction. A replay fed observed service durations does not predict a new execution.

## Quantitative reference, not a new measurement

i9-10900K, Wikidata x4, about 839 MB output; reference runs are distinct and should not be spliced into a paired experiment.

| Evidence | 8-worker wall | Interpretation |
|---|---:|---|
| [Zero-copy floor](../docs/journal/2026-11-24-algorithm-level/ideal-parts/results/floor/README.md), cc8ba137 | 194.6 ms shipped; 179.1 ms scan alone | Even ideal workers cannot outrun unchanged serial scan; about 4.31 GB/s shipped |
| [Pooled-scan results](../docs/journal/2026-11-24-algorithm-level/pooled-scan/results/README.md), c71c4a56 | ~185 ms with scan ideal; ~135 ms with scan and construction ideal | Worker side also binds near the knee; joint relief has much larger room |
| [Warm paired-oracle protocol](../docs/journal/2026-11-24-algorithm-level/ideal-parts/warm-paired-oracle-protocol.md), same primary evidence | 184.9 ms S; 135.3 ms SC | Refines the measurement method: cold C store makes SC a biased reference, not a true warm ceiling |

The 6 GB/s research milestone corresponds to about 140 ms on this output size. Neither an isolated local gain nor an all-free oracle establishes that throughput. The eight-worker co-limitation is established prior knowledge, not a discovery to make again. A later synthesis may refine its numbers using a pinned trace.

## What this implies for research choices

- At one worker, eliminate construction/emission/MTF work; at eight, unlock the value by reducing scan and worker demands together. Continue promising local research even when masked in shipped wall, but name the complementary lever and quantify the conditional opportunity. Do not add isolated percentages or pretend projections are measurements.
- Pooling the existing scan redistributes roughly the same CPU work; the tested pooled build failed its frozen gate. Slack-only hybrid scanning has a separate work-bound argument (`wall_8W ≈ (8·W'/8 + S')/9 + 4.85 ms` derived from `character-stage1` traces, `+3.50 ms` with small chunk 0, zero `k = 6` regression via feeder-offloaded async `OnceLock` `Finder`, zero-copy `Arc<Chunk>` skipping seam blocks, `full_only` filter, and Policy A jump-ahead); the warm protocol screens its next workstation run at `S'(8) <= 165 ms` measured after worker improvements. This is an existing trigger, not a new gate.
- Current complementary paths: scanner coalescing/u8 LUT/BMI2/recycled symbols and seam plumbing; L1d-batched MTF-fold + hoisted single-branch `OutQ16` (`f + w <= C`, `OVF_BIT`) and safe 64K `A6b` redesign on workers; and one coupled **4-lane vectored pipeline** uniting paid `S = 16` bit-31 sampled-anchor dynamic slab emission (`#1057`), slab-level zero-scratch in-place RLE1+CRC fusion (`#1045`, avoiding `74.9%` stitch `memmove` and keeping RSS `<= 1.05×`), and zero-stitch vectored `ReadyBlock` delivery (`#1058`, `>= 64` slices + scatter-fill `Read`/Python). Consult live claims, not this page, for their status. Do not duplicate BMI2 implementation inside a combined scanner build.
- **Knee update (synthesis ninth pass, 27 Nov, derived from VM probes):** the first composed worker stack (#1062/#1067: L1-batched MTF + hoisted `OutQ16`, safe 64K `A6b`, paid 4-lane anchors, 32 KiB cold-outlined slab RLE1+CRC; 233/233 exact) measures `W'/8 ≈ 134.7 ms` native warm and `140.0–145.5 ms` L3-cold: at the top of the eighth pass's derived `126.5–135.8 ms` warm, and 4–10 ms above it cold. The A2 scanner redesign (#1069) projects post-`#1034` `S' ≈ 134–139 ms`. The k = 8 knee is then a tie or worker-bound, so further scanner cuts and the slack-only hybrid (which needs `S' − W'/8 > ~5.5 ms`) pay mainly at k = 9–10, while worker cuts (`A7+A9` is 46% of the composed worker) pay at k = 8. Branch alignment moved shipped worker code by −6..−12% in #1067/#1068: a control, and a possible release lever (`jcc-aligned-release-build`).
- **The knee is a wikidata property (explore, 27 Nov, VM, one thread):** wikidata (17.7x) is the worker-heavy extreme. Off it, the demand ratio k* = W/S is 0.58-0.85x wikidata's on seven text/binary corpora (median 0.64): the scan cost a symbol is flat while symbols a byte rise faster than worker cost a byte. Scaled to the i9 (derived, not measured), typical text balances near k* ≈ 4.6-6.8 and is scanner-bound at 8 workers, where scanner levers and the slack-only hybrid pay and worker levers are masked ([note](../docs/journal/2026-11-27-explore/station-balance-off-wikidata.md); claim `text-knee-i9`). State which corpus a knee statement is about.
- Cache pressure is coupled, not automatically a third serial station. [Worker live-set census](../docs/journal/2026-11-26-synthesis/worker-l3-footprint.md) distinguishes worker footprint at k=6..8 from queue/SMT bursts at 9..10. Cold construction-oracle traffic exaggerates some R/D costs; use the corrected warm protocol, not the early uncorrected synthesis.
- **Intel JCC-erratum 32-byte alignment control:** both Haswell VMs and the Comet Lake i9-10900K carry Intel's JCC-erratum mitigation, which can swing tight `A5`/`A6a`/`OutQ16` loops by `20–40 ms` (`~1.5–3.1%` of 1W wall, the width of an adoption gate; `#1059`). Include a JCC-aligned control (`-C llvm-args=-x86-align-branch-boundary=32 -C llvm-args=-x86-align-branch=fused+jcc+jmp`) alongside default builds during local benchmarking so alignment noise is never mistaken for algorithmic gain or loss.
- A diagnostic that finds strong component value but loses paid total is a prompt for constructive rescue: identify costs, try distinct refinements, seek independent technical input. Preserve frozen verdicts. Finish a successful candidate through implementation and applicable evidence; prepare and queue a frozen, reviewed, green hardware recipe under standing authority when target-machine validation is needed. The owner is not the scheduler.

## Use this in existing claims and deltas

For a performance investigation, explain in ordinary prose which station/work is changed, its measured or projected cost including construction, what masks or complements it, and the next smallest informative implementation or measurement. For a new result distinguish genuinely new evidence from confirmation/refinement of this baseline. For a combined path, compare the combined real execution and relevant isolated/paired diagnostics. Use [performance policy](../docs/team/PERFORMANCE-POLICY.md) and existing frozen gates; this page introduces no additional checklist, approval or requirement to re-register old experiments.

Primary sources: [hybrid model](../docs/journal/2026-11-24-algorithm-level/reading/hybrid-dynamical-systems/relevance/decoder-as-hybrid-system.md), [floor](../docs/journal/2026-11-24-algorithm-level/ideal-parts/results/floor/README.md), [pooled-scan](../docs/journal/2026-11-24-algorithm-level/pooled-scan/results/README.md), [warm protocol](../docs/journal/2026-11-24-algorithm-level/ideal-parts/warm-paired-oracle-protocol.md), [synthesis fourth pass](../docs/journal/2026-11-26-synthesis/fourth-pass.md), [synthesis eighth pass](../docs/journal/2026-11-27-synthesis/eighth-pass.md), [ninth-pass delta](deltas/2026-11-27-synthesise-9.md). Keep original dated evidence intact.
