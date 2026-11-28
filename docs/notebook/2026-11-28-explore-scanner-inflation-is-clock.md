# The 8-worker scanner "inflation" is mostly the turbo bin, not the workers

In one line: on the i9 the clock falls from 5.28 GHz with one worker to 4.91 GHz with four or more. That drop alone predicts the scanner's 1W-to-8W rise in busy time to within 1 ms in a pre-#1034 grid. In the forward-lanes bundle it covers ~75% of the base rise and ~50% of the lanes rise. The coupling that comes from the workers themselves is the residual: about +2.7% (base) and +6.6% (lanes).

- **Claim:** `explore` (recurring). Cold readback of archived i9 bundles. Nothing was re-run.
- **Premise tested:** the nineteenth synthesis pass (DECODER.md, surface `S` row) found that "the scanner's CPU at 8 workers is +10% (base) / +15% (lanes) over its unchanged 1W cost … so faster workers inflate the scanner". Its delta lists the mechanism, shared L3 or bandwidth, as untested.
- **Evidence (`origin/evidence`; each SHA-256 checked against its archived `.sha256`):** `870e61d` character/run-20261125T001135Z (`bb72107b…`, source 94cb742d), k = 1..20 grid; `e98948b6` forward-lanes/run-20261127T223754Z (`db53fe2c…`); `5d259c8` kernel-choice/run-20261125T061204Z (`dbb88594…`); `d6bc8ca2` adaptive seam (`c14e53b4…`).

## The clock by active-core count

| source | condition | GHz |
|---|---|---|
| forward-lanes `cores.json` pacing loop | one core busy (2e8 LCG steps, which LLVM folds to 2.5e7 `imul+add`, 4 cycles each, in 18.888 ms; checked by disassembly) | 5.29 |
| character report, cycles / ref-cycles × 3.7 GHz, whole process | k = 1 / 2 / 3 / 4..20 | 5.28 / 5.01 / 4.96 / 4.91 |
| kernel-choice perf | 3, 4, 5..12 threads | 4.87, 4.88, 4.89 |
| adaptive seam perf | 6..12 threads, 160 runs | 4.871-4.897 |

This is Comet Lake's turbo table: about 5.3 GHz with one or two cores active, 4.9 GHz from three or four. Desktop Comet Lake cores share one clock, so no thread can keep 5.3 GHz once ≥3 are busy. Character's PMU at k = 1 vs 8 shows identical instructions (1.881e10 vs 1.881e10), cycles +5.7% and ref-cycles (∝ time) +13.7%. So for the whole process, +7.5 points of the CPU-time rise is clock and +5.7 is extra cycles, 84% of them `stalls_mem_any` (+0.37e9 of +0.45e9).

## The scanner tracks the clock (character, pre-#1034, 12 rounds × 3 decodes a k)

| k | 1 | 2 | 3 | 4-7 | 8 | 9-10 |
|---|---|---|---|---|---|---|
| scan busy, ms | 172.8 | 179.9 | 183.7 | 183.4-183.7 | 185.0 | 190.3-190.5 |
| predicted from the clock, 172.8 × 5.28/GHz | – | 182.1 | 183.9 | 185.8 | 185.8 | 185.8 |
| A3 (Huffman), ms | 144.5 | 150.7 | 153.8 | 153.7-153.9 | 154.1 | 156.7-157.5 |
| window maintenance, ms | 6.6 | 6.9 | 7.5 | 7.6-9.1 | 10.2 | 11.1-12.1 |

- **The step, then flat.** The rise is a step at k = 1..3 and then flat to k = 8 (+1.6 ms from 3 to 8). That is the clock's shape. L3 or bandwidth pressure would grow with the worker count. A3, which runs from L1, rises +6.4%, just under the clock's +7.5%. Maintenance (memmoves) is the one part that grows with k: +3.6 ms by k = 8.
- **Past the physical cores.** From k = 9 (10+ threads on 10 cores) the scanner rises again, as SMT sharing begins.

## The forward-lanes bundle (post-#1034, scanner pinned to CPU 0, one 1W trace against fifteen 8W traces)

| | 1W busy | 8W busy | ratio | clock-only 8W (×1.075) | residual |
|---|---|---|---|---|---|
| base v0.5.17 | 155.6 | 171.8 | 1.104 | 167.3 | +4.5 ms (+2.7%) |
| `forward-anchor-lanes` | 156.5 | 179.4 | 1.146 | 168.3 | +11.1 ms (+6.6%) |

The clock explains 11.7 of base's 16.2 ms and 11.8 of the lanes' 22.9 ms. The part due to faster workers, lanes minus base, is +6.6 ms (+3.9%). That is the genuine coupling, about a third of what was attributed to it.

## What it changes

- **Comparing across thread counts.** Any 1T or pinned-1T timing, such as `phases/wikidata/1`, 1T `station_balance` or a 1W trace, runs ~7.5% faster per cycle than the same code at k ≥ 3. Ratios within one k are unaffected, and so is k* = W/S when both come from one 1T run. A derived serial S' compared with a measured 8W W/8 needs the factor. The fourteenth pass's "S' ≈ 115–119 ms" (from the 1T 143.5 ms) corresponds to about 124–128 ms at the 8W clock, before contention.
- **Pricing a scanner lever at 8W.** A compute-bound saving measured at 1T is worth about ×1.075 in ms at 8W, plus or minus the contention residual. It is not reduced by the workers.
- **The twentieth pass's proportional assumption.** That pass applies VM lever ratios to the contended 8W scanner as if the inflation were proportional. For the clock part (~7.5 of the 10-15 points) that holds exactly: a clock scales every compute-bound cycle alike. Only the +2.7% / +6.6% residual might not scale with a lever.
- **What isn't changed.** The nineteenth pass's measured 8W numbers stand (179.4 vs 169.6 ms/w, scanner-bound with lanes), as does its advice to price levers at the contended scanner. Only the attribution changes: the clock is a fixed property of ≥3 active cores, while the worker-coupled part (+6.6 ms) is small and lanes-specific.

## Caveats

- **Clock at 1W in the forward-lanes bundle.** It has no PMU, so the 1W clock there is taken from the character run and the pacing loop on the same machine, not measured.
- **One 1W trace.** The forward-lanes 1W cell is n = 1.
- **An older build.** The character grid predates #1034 (A3 has changed since). Its scanner was unpinned but stayed on one CPU (100% top share).
- **Mechanism unknown.** What causes the residual (L3, DRAM, or the lanes' slab pool) is untested. Idea claim `scanner-contention-residual`.
