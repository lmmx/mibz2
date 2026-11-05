# A co-resident walk shape: registration of the first check

Registered before any timing on the current kernel. The decision rule may be amended only
in a commit made before the first of the runs below completes, and each amendment says what
changed and why.

## The hypothesis

When two workers share a core's SMT siblings, a narrower walk with an L2-targeted prefetch
(room walk, 16 lanes, `prefetcht1`) is faster than the default (32 lanes, `prefetcht0`).

The only evidence is one cell of one sweep: CI run 40573190665 (walk sweep, head 5431d90b,
2026-10-27), on a runner whose model was not recorded.

| room/16/L2 against room/32/L1 | change |
|---|---|
| alone | **+23.1%** (+17.1, +29.4) |
| four copies, cpu0 / cpu1 / cpu2 / cpu3 | **−3.8%** / **−6.3%** / **−8.9%** / **−6.7%** |

16/L2 was the fastest four-copy row on every CPU (`2026-10-27-walk-sweep.md`).

The adversarial review of this branch (workflow wf_4bacfdbb-cdd) puts the prior at weak:

- **The kernel is gone.** The sweep ran on kernel A, which had no staging, per-lane segment
  stores and a rolled lane loop. Master's kernel has been staged since #171 and uses #181's
  start test.
- **It is one cell of one run.** About 232 four-copy intervals were read with no correction
  for multiplicity.
- **16/L2 was an outlier alone as well.** There, L2 cost 12.5% over L1, against 0.4% to 2.4%
  at every other lane count.
- **Co-runners were not controlled.** All four copies ran the same rotation, so a setting's
  co-runner was probably a neighbouring setting.
- **No model predicts it.** B2's interval model predicts 16 lanes 6-13% slower than 32 even
  on an SMT pair, and it has no prefetch-hint term.

## The build

`research/coresident-walk` 885823f2 (on d1fd4405). `bwt::Coresident = Shaped<16, PF_L2>` is
chosen once, when the parallel decoder is built, if the workers outnumber the physical cores.

- **The alone path is unchanged:** it is machine-code identical to master. Two independent
  builds give the same Ir (19,273,477 on wikidata block 1) and identical objdump listings
  of the per-block functions.
- **Output:** byte-identical at every shape. Miri passes at 16 lanes.
- **Known defect:** the rule also fires on plain oversubscription without SMT (8 workers on
  4 cores with no siblings). The fix is to go co-resident only when the allowed CPUs include
  siblings. It is not made yet, because the check below comes first.

## The first check, and its rule

Before any decoder A/B: two dispatches of the walk sweep (`.github/workflows/sweep.yml`,
`tool: sweep`, 5 rounds) at master 0769b52f. Each runs `examples/sweep.rs`, which times
`room_segments::<L, PF>` plus `assemble` in decode order, alone on CPU 1 and then as four
copies, one on each CPU.

- **The one hypothesis tested:** room/16/L2/512 against the default room/32/L1/512, with
  four copies. Every other cell is exploratory and decides nothing.
- **It reproduces** if, in each of the two runs:
  - the four-copy change is faster beyond noise on at least 2 of the 4 CPUs;
  - the mean of the four point estimates is faster;
  - no CPU is flagged slower.
- **If it reproduces:** fix the dispatch rule, then register a decoder A/B (bench.yml).
  The A/B decides; the sweep only shortlists.
- **If it does not:** the hypothesis is closed as not reproduced on the current kernel. The
  branch stays unmerged, and the result is written up here.
- **Power:** a four-copy cell's 95% half-width was about ±2-5% in the old sweep, so an
  effect of −4% to −9% would show. A smaller one may not, and a null is reported as "not
  reproduced", not as "absent".
- **Alone:** reported, not decided. A dispatch keyed on co-residency never runs the
  co-resident shape alone.
