# Can the i9 edge writer fit its marked walk in private L2?

On the frozen 431-block Wikidata input, the measured edge worker admits 233 blocks. Across every admitted block and every positive whole-run cutoff from 1 through 16, the smallest sum of packed edge bytes, reachable spelling bytes and reachable offset bytes was **298,491 bytes**. The i9-10900K private L2 holds 262,144 bytes. Treating dictionary lines touched by only one expansion as freely streamed still left at least 296,640 bytes in the tested line layout. Fitting the edge vector alone helped its storage count, but its certified 256 KiB crossing took **1.4363 [1.3532, 1.5245]** times the thread CPU of the unchanged edge writer in a balanced 12-block local panel. [Per-block, per-width census](census.csv) · [Coverage and geometry](structural-summary.json) · [Paid measurements](paid-summary.csv)

## Which bytes the marked walk uses

The decoder is the independently measured i9 winner at `badb2aec3624bd65606c429b2e6f79e9d9d0050f`. This investigation keeps its admission, narrow node links when certified, immediate endpoint words and one-edge lookahead. The earlier generic width sweep and the equal-budget 16/17 strip exchanges studied different interventions. The present selector operates on complete true runs and uses a different cutoff where each block crosses a byte budget. [Previous sweep and exchange tests](../strip-selection/README.md) · [Target worker](../edge-budget/README.md)

Let a true run have width w and let t be the cutoff. Sum w over runs with w ≤ t to get S. Count runs with w > t to get H. If the marked run is short, the actual retained row and omitted-strip counts are S and H. If its width exceeds t, retain the marked row, remove its original strip, and add one strip for each nonempty fragment on either side. Thus m = S + 1 and h = H − 1 + f, where f is zero, one or two. One histogram of widths up to sixteen plus the marked run's boundaries gives all seventeen pairs exactly. Splitting a long run into capped input records leaves these formulas unchanged after coalescing. [Independent geometry calculation](geometry.rs)

The current residual row field takes b = max(1, ceil(log2(m))) bits. The observed code maximum is 255 + number of stored words. The prior bound of 511 + 3h certifies four-byte edges when it fits the remaining 32 − b bits. Otherwise the selector charges eight bytes. In 234 positive-width observations the preconstruction bound charged eight where construction produced four. The endpoint's separate 16-bit node-index guard depends on h alone. The stricter selector's changed positive choices all passed that guard. [Width guard](../edge-budget/gate.rs) · [Exact bounds and allocations](census.csv)

Every real admission visited all m rows and every stored spelling and offset from its actual primary. The census measures that property by traversing the constructed marked cycle and counting distinct spellings. Outputs for all seventeen widths match the independent Segmented inverse. A separate build repeated all 3,961 inverses, expanded RLE1, checked the stored block CRCs and compared 3,564,752,168 output bytes to an independently generated libbzip2 oracle. The scanner checked all twelve combined stream CRCs. [Independent oracle replay](libcheck.txt) The experiment also counts distinct dictionary expansion *events* for each 64-byte spelling and offset line, saturating at two. Both endpoints read from an offset line in one expansion count as one event. The smallest complete footprint was block 47 at t=15. The two recurrent-only footprints at the same block and cutoff were 296,640 and 296,704 bytes for the 0- and 32-byte table-base displacements. Edge lines are charged using the optimistic aligned count in both sensitivity calculations. Output write allocation, the current instruction stream and cache conflicts can take further space. Frequency alone cannot determine cache residency. [Marked walk and line counts](observe.inc.rs)

At t=0 only the primary is retained. On all 233 real admissions it gives a single edge and a single dictionary entry spelling the entire coded block. Across the corpus, constructing that spelling consumed 27,439,989 endpoint steps and generated 209,697,662 spelling bytes. A recurrence-only count at this cutoff therefore omits virtually the entire materialized output. The paid measurement keeps its constructor and output copy. This observed degenerate case concerns these marked input blocks; other marked permutations may have shorter primary cycles.

## Residual edge crossings and their price

| Budget | Positive actual edge crossings | Positive certified edge crossings | Certified changed blocks | Output on changed blocks | Share of historical decode-block CPU | Retained rows | Endpoint steps | Stored spelling bytes |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 192 KiB | 184 | 181 | 181 | 162,893,305 | 68.79% | 0.4520× | 3.9479× | 8.2559× |
| 256 KiB | 202 | 194 | 193 | 173,691,608 | 75.00% | 0.6210× | 2.2708× | 3.3874× |

A positive crossing allows any t from 1 through 16, including an already-fitting width-16 block. Geometry ratios compare changed admissions with the same blocks at their original width 16. The admitted 233 account for 209,691,304 of 209,700,196 final bytes. Historical CPU weights use median complete `decode_block` thread CPU for each admitted block from the prior edge/shortcut stage and its single cold comparison on the remaining tiny blocks. Those weights exclude the entropy scanner, worker overlap and delivery. The dataset provides zero positive crossings of the complete or recurrent marked footprint under either budget. [Contextual CPU source](../front-fusion/cap-batch/phase.csv) · [Exact geometry by policy](structural-summary.json)

The narrower test selects solely on certified edge bytes with a paid histogram pass from the capped input. It uses the first eligible block in each of twelve streams with a nondegenerate 256 KiB crossing below 16. Every arm in each block occupied each position exactly three times in 24 rounds. Each call includes the live admission, selection when applicable, the complete current constructor, and all output generation. The local Xeon reports a 2 MiB L2. An independent low-priority process ran on CPU0 while the panel ran on CPU3 with four CPU-equivalents of quota. The ratios and two-sided paired log intervals describe whole-lap thread CPU on this host. They do not measure target-machine cache transfers or complete decoder wall time. [Frozen plan](plan.md) · [Exact per-block selections](panel-selection.csv) · [Manifest](manifest.txt) · [All 2,304 calls](paid.csv)

| Intervention | CPU over unchanged width 16, geometric mean [95% interval] |
|---|---:|
| Identical binary path and width | 1.0002 [0.9715, 1.0298] |
| Equal histogram charge, width 16 | 1.1007 [1.0539, 1.1497] |
| Certified 256 KiB edge crossing | 1.4363 [1.3532, 1.5245] |
| One width above that crossing | 1.2974 [1.2403, 1.3571] |
| One width below that crossing | 1.7847 [1.6527, 1.9273] |
| Certified 192 KiB edge crossing | 2.1037 [1.9058, 2.3221] |
| Sole primary, t=0 | 3.0163 [2.9112, 3.1253] |

Against the equal-histogram control the 256 KiB crossing still cost 1.3049 [1.2432, 1.3696]. The observed growth in omitted strips, endpoint elimination and word payload accompanies the smaller edge table. A different representation could change the footprint bound. Changing t in this complete, whole-run representation cannot give the proposed private fit on any positive-threshold Wikidata block in this experiment.

## Reproduction

Use the repository corpus and Rust 1.99.1. The standalone build by default draws both the decoding library and the measured constructor from the frozen edge commit, with modifications confined to the temporary build. The paid binary used the campaign's separately generated support rlib identified in the manifest. Its compiler recipe is in `budget/portfolio/prepare.py` at commit `3978fad70ecd829604ddf5fed4618bc9116aa1aa`. Passing its generated `libmibz2.rlib` as the second argument to `build.py` reproduces that selection. Support-library scanning, front processing and the oracle are outside the timed interval. The captured counts and checksum are independent of the library path. [Native driver](panel.rs) · [Receipt calculations](summarize.py)

```bash
d=research/astra-e/private-cache
python3 "$d/build.py" /tmp/astra-private
python3 -c 'import bz2,shutil; z=bz2.open("corpora/wikidata.bz2"); f=open("/tmp/astra-private/wikidata.raw","wb"); shutil.copyfileobj(z,f)'
/tmp/astra-private/libcheck corpora/wikidata.bz2 /tmp/astra-private/wikidata.raw
/tmp/astra-private/replay corpora/wikidata.bz2 > /tmp/astra-private/census.csv
cmp /tmp/astra-private/census.csv "$d/census.csv"
nice -n 16 taskset -c 3 /tmp/astra-private/panel corpora/wikidata.bz2 24 > /tmp/astra-private/paid.csv
python3 "$d/summarize.py"
python3 "$d/paid.py" /tmp/astra-private/paid.csv
```
