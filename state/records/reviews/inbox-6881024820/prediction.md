# Pre-data prediction: Cells 24 → 20 B slot

Inbox [6881024820](https://github.com/lmmx/mibz2/pull/160#issuecomment-6881024820). The Coordinator's [frozen step-1 rule](https://github.com/lmmx/mibz2/pull/145#issuecomment-6881022819) asks for Q and two wall ratios before the paired recipe is frozen or run. This is a forecast, **not** a measured interval or an adoption verdict. No build, decoder, timing, or candidate execution was performed.

## Three preregistered forecasts

| Statistic, 20 B / 24 B | Point | Plausible range | Mechanism |
|---|---:|---:|---|
| Q = [CPU₍₂₀₎(10)/CPU₍₂₀₎(1)] / [CPU₍₂₄₎(10)/CPU₍₂₄₎(1)] | **0.96** | **0.91–1.01** | Expect roughly 0.95 ten-thread CPU ratio versus 0.99 one-thread CPU ratio. At ten threads streamed traffic is a demonstrated shared constraint; smaller slots can relieve it. The 11% line estimate is a footprint model, not a 11% CPU promise. |
| Four-thread Wikidata wall ratio | **0.97** | **0.94–1.02** | Four threads are traffic-sensitive in the dose, but P0 still scales to six threads, so expect a smaller gain than at ten. |
| One-thread Wikidata wall ratio | **0.99** | **0.97–1.02** | Less slot traffic may help slightly; at one thread the cache and dependency chain can hide it. Changed u32 offset arithmetic or alignment could instead cost a little. No one-thread loss is expected at the point estimate. |

For scale, the [dose record](https://github.com/lmmx/mibz2/pull/145#issuecomment-6880764544) has P0 medians 0.405, 0.195, 0.169, 0.170, 0.179 s at 1, 4, 6, 8, 10 threads. The wall point forecasts imply about **0.401 s** at one and **0.189 s** at four; the implied ten-thread wall illustration is **0.170 s** if aggregate CPU scales with wall at the same ten-thread occupancy. That last conversion is an assumption, not the registered Q statistic.

## Basis and limits

The [pinned memory census](https://github.com/lmmx/mibz2/blob/95b98eda/docs/journal/2026-11-14-cells-memory/README.md) finds 0.1025 distinct 64 B lines per output byte; the two slot orders account for 65% of them and 70% of modelled 2 MiB misses. Its 24→20 B estimate is ~0.011 fewer lines per output byte (~11% of all distinct lines), ~12% fewer modelled 2 MiB misses. These are address-level, single-core model results; they do not establish DRAM bytes saved at ten workers. The same census counts 13.79 heap bytes written and 11.27 read per output byte, with induction consuming most of the repeated slot passes. The [phase census](https://github.com/lmmx/mibz2/blob/b5617a8/docs/journal/2026-11-14-cells-census/summary-wikidata.json) has 790,102,938 slots Ir and 1,155,202,206 induction Ir on 196 admitted blocks; the 8-byte to 4-byte offset targets the hot slot representation, not the writer.

The [dose](https://github.com/lmmx/mibz2/pull/145#issuecomment-6880764544) establishes that added streaming traffic hurts proportionally more at ten threads: c₁₆(10)/c₁₆(1)=1.583 [1.516,1.654]; P16 plateaus at four, P0 at six. Its ~24 B/output-byte DRAM premise is an estimate, not a counter reading. Slot compaction may save fewer fetched bytes than the cold-line estimate suggests, or more traffic if repeated passes and write allocation also shrink. It might change codegen/alignment and the exact ten-thread CPU occupancy. That is why the Q range crosses both the 0.95 threshold and 1; a paired log-t interval and valid complete roster must decide the frozen rule. A nominal Q=0.96 is in the rule's gap: neither MOVES THE PLATEAU nor NO SCALING EFFECT follows from the point alone.

The proposal changes only bytes 16..24 to a u32 other-order offset. The hard >u32::MAX layout refusal, B5 checks and Guardian review remain requirements. This forecast grants no gate, merge, next-step, or adoption authority. Step 2 waits for step 1's measured rule. The census's strong induction share keeps induction restructuring as a separate higher-ceiling whole-decoder possibility, but the 20 B cut is the smaller and directly testable traffic intervention.
