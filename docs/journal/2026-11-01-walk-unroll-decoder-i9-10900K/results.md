# The unrolled room walk's full-decoder trial: the registered run on the i9-10900K

**The run (measured):**
- `just bench-walk-decoder i9-10900K`, run by the maintainer at head
  `f5c492d0005d2f1e6bbc15762c5beb742ce433f0` (pin `df90ba8`; amendments 1 and 2).
- rustc 1.97.0 (`environment.json`), the toolchain amendment 1 accepts. `frozen.json`'s
  "rustc 1.90.0" string is the erratum recorded before the run
  (`../2026-11-01-walk-unroll-decoder-preauthorisation-inspection.md` §5).
- The binary's SHA-256 is `982bf1d3…79f1f5` (kept as `walk_decoder.gz`).
- The timed trial took 937 s, with 2,772 timed children.
- **CPUs:** 1–4 (four physical cores) allocated; their siblings 11–14 were 0.6–0.8% busy
  over the trial; steal 0.
- **Clock:** governor `powersave` (intel_pstate), turbo enabled.
- **Before timing, every check passed:**
  - the 20 harness sources and 11 corpus hashes;
  - `bzip2 -t` and `-dc`;
  - `--check` at w = 1–4;
  - the arm's tests;
  - the structural audit W1–W4 (b), with W4 (c) matching the accepted signature;
  - the synthetic self-test.
- **During the trial:** `stderr.txt` was empty, and end snapshots held 0 exited records.
- **Re-check:** the frozen summariser (`bench/walk_decoder.py summary` at `f5c492d`),
  rerun on `raw.tsv`, reproduces `verdict.json` exactly.
- **The environment snapshot,** taken before the self-test, lists one process at 26% CPU
  on CPU 4, an allocated CPU.
  - Over the trial, `/proc/stat` shows CPU 4 no busier than the other allocated CPUs:
    33.6%, against 32.7–36.1%.
  - So there is no sign that it shared the allocation while timing ran.
  - `environment.json` keeps PID, PSR and %CPU only. Home paths are written as `<repo>`.

## Registered verdict (§8, mechanical): **NEGATIVE**

| Gate | Outcome | Deciding rows |
|---|---|---|
| 0 Correctness | pass | every output hash, every plan `Workers(w)`, every thread within the allocation |
| 1 Primary: wikidata-20261026, w = 4 | **NEGATIVE** | B/A **1.000 [0.987, 1.012]**: upper < 1.02. Not harmful (upper > 1.00). |
| 2 Secondary flags | none raised | fitted 1.008 [0.995, 1.021]; held-out 1.012 [0.996, 1.030] at w = 4 |
| 3 Controls, every w | INCONCLUSIVE | `records` w1 0.981 [0.968, 0.994], w2 0.979 [0.968, 0.991], w4 0.979 [0.935, 1.024]. Wide intervals at w4 for `text-l1`, `text-l5`, `zeros` and `period-3`, and `zeros` w2. No row NEGATIVE. |
| 4 CPU-s/GB, w = 4 | **NEGATIVE** | `records` 1.018 [1.010, 1.026]: lower > 1.01. INCONCLUSIVE: `mixed-independent`, `text-l5`, `zeros`. |
| 5 Peak RSS | INCONCLUSIVE | the max rule only: `random` w4 1.042, `text` w3 1.016, `text-l5` w4 1.011, `period-3` w1 1.011 and w2 1.096. Every paired lower bound ≤ 1.00. |
| 6 A/A | pass | 2 of 88 rows exclude 1, against a threshold of 9 |

**Under the stop rule:** the unrolled walk (`SegmentedUnrolled`) is archived at decoder
level. There is no rerun, no tuning, no threshold change and no default change.

## Figures (measured)

**Wikidata B/A throughput by w:**

| | w1 | w2 | w3 | w4 |
|---|--:|--:|--:|--:|
| wikidata-20261026 | 1.017 [1.006, 1.028] | 1.012 [0.982, 1.043] | 1.005 [0.988, 1.022] | 1.000 [0.987, 1.012] |
| wikidata-fitted | 1.018 [1.005, 1.032] | 1.020 [0.985, 1.055] | 1.004 [0.995, 1.012] | 1.008 [0.995, 1.021] |
| wikidata-heldout | 1.008 [0.994, 1.021] | 1.001 [0.982, 1.020] | 1.008 [0.992, 1.025] | 1.012 [0.996, 1.030] |

**Throughput and CPU on the primary and `records`:**
- **Primary, A's median throughput:** 369 / 684 / 775 / 808 MB/s at w = 1–4. B is
  808 MB/s at w = 4. The fourth worker adds 4%.
- **Primary CPU-s/GB B/A:** 0.985–0.994 at every w. The arm does slightly less work
  without the wall time following.
- **`records`:** B/A 0.979–0.989 at every w; CPU-s/GB B/A 1.013–1.018 at every w.
- **`period-3`,** the short-cycle walk control: +6.5% at w = 1 and +4.6% at w = 2.

**Across hosts, primary w = 1** (the CI rows are exploratory):

| Host | wikidata-20261026 w1 | `records` w1 |
|---|--:|--:|
| EPYC 9V45 (Zen 5), CI | +17.9% | +0.3% |
| EPYC 7763 (Zen 3), CI | +8.1% | −5.2% |
| EPYC 9V74 (Zen 4), CI | +1.7% | −8.3% |
| Xeon 6975P-C, CI | +0.2% | −7.2% |
| **i9-10900K (Comet Lake), registered** | **+1.7%** | **−1.9%** |

## Reading (deduced; not part of the verdict)

**The stage gain doesn't reach the decoder on this core:**
- The stage PASS was a 12% walk saving, measured on Zen 3.
- On the i9 it is under 2% of the decoder at w = 1 and nothing at w = 4, where the
  four-core decoder is near saturation.

**Consistent with the static floor model** in
`../2026-11-01-walk-unroll-astra-fidelity/README.md`, though it doesn't demonstrate it:
- On Skylake-family cores the unrolled step is bounded by the two load-address ports
  (loads, the prefetch and the indexed byte store's address), at ≈ 2.5 c/step, not by
  dispatch.
- So removing bookkeeping buys less there than on Zen.
- No counters were collected to separate this from other causes.

**`records`:** it costs 1–2% more CPU and throughput on this host, smaller than on three
CI hosts. Its cause is still uninvestigated.

**Every registered full-decoder trial on the i9 has now come out NEGATIVE:** P74-C, and
this one.

**Read-only analysis of these results:** `analysis.md`.
