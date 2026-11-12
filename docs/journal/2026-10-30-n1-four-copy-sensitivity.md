# N1 at four workers: what the four-copy regression is, and where N1 loses

**Status:** read-only analysis. Nothing was built, benchmarked or dispatched. Two existing
CI artifacts (3–5 KB each) and the bench comments already on lmmx/mibz2 were read. The
model is `fourcopy_model.py`, over measured stage figures.

**What this replaces:** the construction-economics note's four-copy column (§5.4, "pass if
c_e ≤ 31" on the 7763). That figure rested on one measured factor (the shipped walk ×2.08)
and three assumed ones (shipped build ×1.14, N1 build ×1.14, N1 walk ×1.3). Here the
assumed factors are replaced by measured analogues and a surface.

**Boundaries:** four concurrent workers, each decoding its own block, on CI's 4-vCPU
runners (2 physical cores with SMT). Figures are c/B of output per copy at the run's
reported clock, unless marked ns/B. Each figure is labelled:
- **measured:** from a recorded run, cited;
- **deduced:** follows arithmetically from measured figures;
- **model:** an estimate, with its assumptions listed (§7).

## 1. Summary

1. **The headline: N1 loses on CPU, and on wall, wherever c_e exceeds about 14–24 cycles
   at four workers, depending on runner and plan.** On the one runner where every stage
   was measured at one and at four copies (the 7763, run 40579732655):
   - P^{2|4} breaks even at c_e ≤ 17.5–21.2 with the conservative walk (24 instructions
     a lookup), and ≤ 21.5–24.0 with the optimistic one (16), for α_N between α_T and
     1.26;
   - P^{4|8} breaks even at c_e ≤ 13.6–14.8 (conservative) and ≤ 15.2–15.9 (optimistic);
   - the four-copy *pass* (N1's build + walk ≤ 0.75× the shipped one) needs c_e ≤ 10.3–16.0.

   That is close to the note's one-thread pass threshold (10.2), not to its 31.
2. **Wall and CPU fall on the same side everywhere in the model.** At `Plan::Workers` a
   worker's wall is its CPU, less a utilisation that is measured at 0.83–0.86 and barely
   moves. No region of the surface wins on wall and loses on CPU. The only one-sided
   regions are noise bands, 1–3% of the plausible range (§5.3).
3. **The note's 31 had three faults** (deduced):
   - **An inverted factor.** `model.py` divides c_e by the N1 build factor (`ce / 1.14`)
     instead of multiplying. Corrected alone, the threshold falls from 31.2 to 24.0.
   - **The shipped build's factor was measured, not 1.14:** ×1.26 (1.24–1.27) at four
     copies on the 7763. The 1.14 most likely came from P110 (+14% CPU a block with both
     cores busy, the whole decoder, already packed on siblings). That is a cross-core
     factor, not a four-copy one.
   - **α_N = 1.3 is below every measured analogue.** No stage on the 7763 inflates by
     less than ×1.26 at four copies, except the plain one-chain walk (§2.2).
4. **The regression is SMT sharing of a per-core resource, not shared L3 or DRAM**
   (measured and deduced, §3).
   - **Per core, the shipped walk is no faster with two copies than with one:** 0.955×
     one copy's throughput.
   - **The plain one-chain walk, a pure L3-latency probe, slows only ×1.045** with all
     four T-vectors live.
   - **Where four workers sit on four separate cores** (the local VM pilot), the walk's
     factor is ×1.08.
5. **So N1's smaller table escapes only the cross-core part,** about ×1.04–1.10. Its
   walk's four-copy factor depends on what bounds it per core.
   - **The model** (§4) gives α_N ≈ 1.36–1.47 if each lookup has at most one L2-missing
     access and 24 instructions.
   - With a second L2-missing access, or 16 instructions, α_N = α_T.
   - When α_N = α_T, four copies give N1 no advantage over one thread.
6. **Zen 5 has no four-copy stage measurement.** The whole decoder's CPU amplification
   there is the largest of the three runners: ×2.21, against ×1.74 on the 7763 and ×1.83
   on the 9V74. That implies α_T ≈ 2.2–3.2 (deduced, §2.4). The 9V45 rows below rest on
   that range.
7. **A unit problem the note inherits** (measured): stage "cycles" are wall × the reported
   clock, and the reported clock does not track the running clock.
   - **The 9V45** reports 2596–4548 MHz on runs whose compute stages take the same ns/B
     (build 0.32–0.35 ns/B).
   - **The note's Zen 5 row** (9.17 c/B, walk IPC 4.04) is from a run reporting 4548 MHz.
   - **So the c_e thresholds** are in each reference run's reported cycles, not core
     cycles. Gate B should report ns per element.

## 2. The historical four-copy evidence, run by run

### 2.1 What establishes "5.0 → 10.4 c/B" (measured)

The figure is from **CI run 40573190665** (`sweep.yml`, tool `sweep`, master `5431d90`,
5 rounds; `2026-10-27-walk-sweep.md`, §"The curve in decode order").
- **What was timed:** the walk alone, in decode order. First one copy pinned to CPU 1,
  then one copy on each of the 4 CPUs at once (four processes).
- **The runner model was not recorded.** The artifact's header has none, and the journal
  says so. Its reported clock, 2.8724 GHz, matches the EPYC 9V74's 2870.4–2871.5 MHz in
  three bench runs. No 7763 (15 runs) or 9V45 (4 runs) reading is near it. So it was
  **probably a 9V74** (inference). The note applied it to the 7763.
- **The room walk at 32 lanes, L1 prefetch, 512 segments:** 5.00 alone. Four copies gave
  9.67, 10.35, 10.85 and 10.62 (mean 10.37, ×2.07, range ×1.93–2.17).
- **Fewer lanes inflate less**, and every setting converges near 10–12 c/B at four copies:

  | Setting | One copy | Four copies (mean) | Factor |
  |---|--:|--:|--:|
  | room, 8 lanes, L1 | 8.01 | 11.97 | 1.50 |
  | room, 12 lanes, L1 | 6.36 | 10.60 | 1.67 |
  | room, 16 lanes, L2 | 5.89 | 9.70 | 1.65 |
  | room, 32 lanes, L1 (shipped) | 5.00 | 10.37 | 2.07 |
  | room, 32 lanes, no prefetch | 5.73 | 12.53 | 2.19 |
  | flagged, 32 lanes, L1 | 5.86 | 12.70 | 2.17 |

  This is the signature of a per-core cap. A walker that one copy leaves short of the cap
  loses less when a sibling arrives.

### 2.2 A second run, with every stage: the 7763 (measured)

**Caveat (added on review):** this run's commit, c8f9740 (27 October, branch
`perf/assembly-row`), predates #114 (6a341f6, 29 October), which made the one-core
`bwt::build` one pass: each entry written whole, with no copy pass and no
read-modify-write. So its build factor, ×1.26, is the two-pass build's. The one-pass
build's four-copy factor is unmeasured. Every row below that uses β_T = 1.26 carries that
dependency.

**CI run 40579732655** (`sweep.yml`, tool `phases`, master `c8f9740`): an AMD EPYC 7763,
4 CPUs, 3.2407 GHz reported, 3 passes, each stage's fastest kept. One copy ran pinned to
CPU 1, then four copies, one a CPU.
- **Cited before:** only for the assembly share (`2026-10-27-output-side.md` §7, "1.9–2.0%
  with four copies on an EPYC 7763").
- **Not recorded in any journal:** its full stage table, below.

| Stage | One copy | Four copies (mean of 4 CPUs) | Factor (range over CPUs) |
|---|--:|--:|--:|
| scanner (headers, Huffman) | 1.22 | 1.86 | 1.52 (1.47–1.58) |
| MTF + RLE2 | 2.27 | 3.08 | 1.36 (1.34–1.37) |
| T-vector build | 2.27 | 2.85 | **1.26** (1.24–1.27) |
| walk, room (used) | 4.49 | 9.40 | **2.09** (2.00–2.20) |
| RLE1 | 0.28 | 0.41 | 1.46 |
| CRC | 0.26 | 0.36 | 1.38 |
| segment assembly (within the walk) | 0.16 | 0.36 | 2.23 |
| walk, flagged (not used) | 6.10 | 13.03 | 2.14 |
| walk, bitmap, 12 lanes (not used) | 13.25 | 25.15 | 1.90 |
| **walk, plain: one dependent chain (not used)** | 38.55 | 40.27 | **1.045** (1.04–1.05) |
| total | 10.78 | 17.96 | 1.67 (1.63–1.71) |

- **The phases align:** `phases` runs its used-walk passes first, then each other walk's
  passes in turn. The four copies start together and run at equal speed, so each row is
  mostly measured against the other copies running the same stage. The plain walk's row is
  plain walks against plain walks.
- **Four processes against four threads:** the total's ×1.67 sits inside the bench's
  whole-decoder 4-thread CPU amplification on the 7763: 1.74 median, 1.58–1.82 over 15
  runs. The caution "four processes are not four threads" doesn't change the factor much
  here (deduced).

### 2.3 Whole-decoder amplification on every runner (measured, bench 3.x)

These are the base arms of the bench comments on lmmx/mibz2 PRs #69–#118, 10 paired rounds
each (`extract_bench.py` → `bench_runs.json`). CPU per byte is at N threads over the same
build's at 1.

| Runner | Runs | 2 threads, one a core | 4 threads, on SMT siblings | Their ratio (≈ the SMT part) | Utilisation u at 4 threads |
|---|--:|--:|--:|--:|--:|
| EPYC 7763 | 15 | 1.12 (1.01–1.17) | 1.74 (1.58–1.82) | 1.55 | 0.856 |
| EPYC 9V74 | 8 | 1.13 (1.07–1.19) | 1.83 (1.76–1.88) | 1.62 | 0.852 |
| EPYC 9V45 | 4 | 1.20 (1.16–1.23) | 2.21 (2.16–2.27) | 1.85 | 0.826 |

u is (CPU/4)/wall at 4 threads. The other separations already recorded:
- **#54 (placement):** per-block CPU is 1.43–1.46× on siblings, and spreading gives
  +39.6% / +40.5% throughput (7763, 9V74).
- **#71 (contention fingerprint, 9V74, competitor on the other physical core):**
  - a twin decoder: 1.10× over compute;
  - a cache stressor at ½ the measured effective LLC: 1.07×;
  - a DRAM streamer: 1.64×.
  - The effective LLC is 12.8–16.8 MB, from a pointer chase
    (`2026-10-28-early-warning.md`).

### 2.4 What each runner has (measured or not)

| Runner | One-copy stages | Four-copy walk | Four-copy other stages | Whole-decoder 4-thread amplification |
|---|---|---|---|---|
| 7763 | yes (note §5.1; run 40579732655) | **×2.09, measured** | **measured** | 1.74 |
| 9V74 | yes (note §5.1) | ×2.07, measured on a runner inferred to be a 9V74 | not measured: the 7763's factors assumed | 1.83 |
| 9V45 | yes (note §5.1, at 4548 MHz reported) | **not measured** | not measured | 2.21 |

**The 9V45's α_T (deduced, under an assumption).** Write the amplification as the
CPU-weighted sum of stage factors: 2.21 = w·α_T + (1 − w)·β_rest, with the walk share
w = 4.18/9.17 = 0.456 (note §5.1).
- For β_rest of 1.36 (the 7763's MTF), 1.6 or 1.8, α_T is 3.2, 2.9 or 2.7.
- At the uniform bound (β_rest = 2.21), α_T is 2.2.

The model uses 2.2–3.2. The same identity reproduces the 7763's 2.09 to within 0.2
(1.74 gives 2.27 at its measured β_rest of 1.36).

### 2.5 The local VM and the i9 (measured; other machines, kept apart)

- **The local VM** (an Intel Xeon at 2.8 GHz under KVM; 4 vCPUs reported as 4 cores with
  1 thread each; a 33 MB L3):
  - **The compact-Ψ pilot** (`scratchpad/compact-vm.jsonl`, 8 rounds, the T-vector phase
    only, 4 threads, each pinned to its own vCPU): per worker, the factor at four
    workers was:
    - the T-vector walk ×1.08 (1.02–1.11), build ×1.02;
    - the 32-bit Ψ walk ×1.16;
    - the packed 20-bit Ψ walk ×1.04;
    - the split 16 + 4-bit walk ×1.00 (`vm_pilot.py` → `vm_pilot.json`).
  - **PR #21's earlier four-process runs on the same VM type** (`walk-exp/sweep3-top-1t.txt`,
    `sweep4-cpu*.txt`, flagged walk) gave larger factors:
    - 32 lanes with prefetch ×1.66 (12.28 → 20.35);
    - 12 lanes without ×2.41;
    - the plain chain ×1.7–2.0.

    Those four-copy files report 78 MB walked against 209 MB alone, so the two arms did
    not walk the same blocks. The two sessions disagree, and the VM's host placement is
    unknown. **Unresolved.**
- **The i9** (10 cores, 20 MB L3; `2026-10-28-workstation-scaling.md`):
  - capacity binds past about 4 workers: the demand L3 hit share is 0.76 at 4 and falls
    to 0.38 at 10;
  - at 4 blocks the hit share is about 0.93 (compact-Ψ feasibility §5);
  - teams on SMT siblings lose, "as the walk's memory parallelism is per core";
  - the N-copy stage proxy is quoted only at 10 copies (walk ×9.9, build ×9.0). No
    4-copy figure is recorded.

## 3. Which mechanism the evidence supports

| Mechanism | Evidence for | Evidence against | Verdict on CI's 4 vCPUs |
|---|---|---|---|
| **SMT sibling sharing of a per-core resource** (miss buffers, load queue, L1/L2) | Per core, the room walk's throughput at four copies is 2 / 2.09 = 0.955× one copy's (deduced). Low-MLP walk settings inflate less, and all converge near one level (§2.1). #54: CPU a block ×1.43–1.46 on siblings. The ratio 4 threads / 2 spread is 1.55–1.85. The VM, with no SMT exposed, gives ×1.08. | No counters on CI, so the resource (MABs, L2, load queue) isn't identified | **Supported: most of the ×2.09** |
| Shared L3 capacity | The effective LLC is 12.8–16.8 MB on the 9V74, against 14.4 MB of T-vectors plus buffers. On the i9 it is the knee's first mechanism past 4 workers. | The plain one-chain walk slows ×1.045 at four copies with all four T-vectors live. At 11.9 ns a step that allows about 0.6% of its loads to move to DRAM (deduced; the DRAM penalty of about 80 ns is an assumption). #71's ½-LLC stressor: 1.07×. | **At most ×1.04–1.10 on CI**; the dominant factor past 4 workers on the i9 |
| DRAM bandwidth | #71's streamer costs 1.64× | A twin decoder costs only 1.10×, and the plain walk doesn't slow | **Not supported at four copies** |
| Frequency drop under all-core load | — | The plain walk's ×1.045 bounds a core-clock drop to about 4.5%, if its L3 latency scales with the core clock (assumed for Zen 3's L3). The VM's 2 cores are 2 of 64 on the socket. | **Not supported**; ≤ about 4.5% |
| TLB | 4 × 3.6 MB at 4 KiB pages is about 3,500 pages | The i9: page walks 1–2% of cycles. No CI counters. | **Untested on CI**; small on the i9 |
| 4K aliasing / address aliasing | — | Nothing in any journal; no test | **No evidence either way** |
| Cross-core sharing (L3 bandwidth, host, scanner handoffs) | 2-thread spread amplification 1.12–1.20; #71 twin 1.10×; P110's +14% | — | **About ×1.04–1.20 of the whole decoder** |

**The topology** (measured, `2026-10-27-four-thread-ceiling.md` §3, and every bench comment's
"Conditions"):
- 4 CPUs on 2 physical cores, siblings 0-1 | 2-3, on both the 7763 and the 9V74;
- the L3 is shared with other tenants, and its size is unknown;
- the 9V45 too: its four bench comments record 4 CPUs on 2 physical cores (0,1 | 2,3).

**What this means for N1 (deduced):** N1's 1.0–2.0 MB table (against 3.6 MB) addresses
the minor term, capacity. It does nothing by itself for the major term, two walkers
sharing one core's miss handling and issue.

## 4. What sets α_N, and what evidence exists for 1.3 and 1.14

**No direct evidence exists for N1's ×1.3 walk factor or ×1.14 build factor.** No N1 walker
or builder exists. The proxies, all measured:
- **The shipped build:** ×1.26 at four copies (7763).
- **MTF:** ×1.36.
- **The scanner:** ×1.52.
- **RLE1 and CRC:** ×1.38–1.46.

  These are the compute-bound stages under SMT, and the lowest factor among them is the
  build's. So α_N = 1.3 and β = 1.14 are optimistic.
- **The walks:** ×1.90–2.25 for every 12–32-lane walk. The plain chain, ×1.045, is the
  only factor below 1.26, and it is latency-bound with one load in flight. N1's walk is
  designed not to be that.
- **The compact-Ψ tables** (2.3 MB, more instructions a step): ×1.00–1.16 per worker on
  the local VM, with no SMT exposed. **No four-copy run of them exists on CI.** The
  move-structure four-copy test (E8, pre-registered in #42) has never been run.

**The per-core cap model** (`alpha_n_model.py`; model). Suppose:
- the shipped walk sits at the per-core cap on L2-missing accesses (about one a byte);
- N1's lookup makes m such accesses at the same per-core rate;
- otherwise N1's walk is instruction-bound and shares the core like the compute stages
  (s = 1.36).

Then α_N = max(s·c_L, m·α_T) / max(c_L, m), with c_L in units of one shipped walk step
(16–24 instructions over 16.9):

| α_T | m | 16 instructions | 20 | 24 |
|--:|--:|--:|--:|--:|
| 2.09 | 0.5 | 1.36 | 1.36 | 1.36 |
| 2.09 | 1.0 | 2.09 | 1.77 | 1.47 |
| 2.09 | ≥ 1.5 | 2.09 | 2.09 | 2.09 |
| 2.7 | 1.0 | 2.70 | 2.28 | 1.90 |

The reading (model):
- **With one L2-missing access per lookup and the 24-instruction lookup,** α_N ≈ 1.5.
  N1 stays instruction-bound at one copy.
- **With a second access** (the rank directory and the record both missing L2: N1's
  tables are 2–4× Zen 3's 512 KB L2, and two siblings share it), N1's walk sits at the
  same cap as the shipped one. Then α_N = α_T.
- **So the plausible range is 1.3–α_T, centred near 1.5–2.0.** The note's 1.3 is its lower
  edge.

## 5. The surfaces (model over the measured stage figures)

### 5.1 The model

**Per copy at four copies**, with the non-inversion stages at their measured four-copy
values (assumed unchanged by N1):
- CPU_T = other₄ + build_T·β_T + walk_T·α_T;
- CPU_N = other₄ + β_N·(e·c_e + scan_L) + α_N·L·c_L.

**The N1 inputs:**
- **e** is the elements per byte, with upgrades weighted 1.5 (the note's u). The census
  medians are P² 0.201, P^{2|4} 0.315, P⁴ 0.369, P^{4|8} 0.509 and P⁸ 0.586. The corpus
  aggregate is 16% higher for every plan (0.233, 0.365, 0.429, 0.593, 0.680).
- **L** is the lookups per byte: 0.500, 0.296, 0.250, 0.150 and 0.125.
- **c_L** is 16 or 24 instructions at the base's one-copy walk IPC (16.9/walk_T₁), the
  note's convention. That gives 4.3–6.4 cycles on the measured 7763 base and 6.3–9.5 on
  the note's.
- **scan_L** is 0.3 c/B (optimistic) or 0.6 (conservative).

**Wall** at 4 workers: wall/B = (CPU/4)/u. The idle (1 − u) splits into:
- a share f = 0.5 that is fixed in absolute time: the scanner's reads, which are 10–11%
  of the 18–19% idle (#53);
- the rest, which scales with the work.

So wall_N/wall_T = [r·(1 + (1 − f)k) + f·k] / (1 + k), where r = CPU_N/CPU_T and
k = 1/u − 1. With u = 0.83–0.86, a wall change is about 0.93 of the CPU change.

**Noise:** a win or a loss on CPU must clear ±1% and on wall ±2%. These are the bench's
4-thread interval half-widths on Wikidata.

**The four bases:**
- **(A)** the measured 7763 run 40579732655, every stage at 1 and 4 copies;
- **(B)** the note's §5.1 7763 one-copy figures (walk 6.69), times (A)'s factors;
- **(C)** the 9V74's §5.1 figures, with the walk ×2.07 (§2.1) and the other stages at
  (A)'s factors;
- **(D)** the 9V45's §5.1 figures, with α_T at 2.2–3.2 and the other stages at (A)'s
  factors.

### 5.2 The loss boundary: c_e at CPU break-even (median block, β_N = 1.36)

The c_e above which N1 loses on CPU per decoded byte at four workers. The wall boundary is
the same to 0.1 cycle in every cell (§5.3). α_T is at its range mid: (A), (B) 2.10;
(C) 2.05; (D) 2.70.

| Base | Plan | Walk | α_N = 1.26 | 1.5 | 2.0 | α_N = α_T |
|---|---|---|--:|--:|--:|--:|
| (A) 7763, measured | P² | 24 / 16 instructions | 27.2 / 33.6 | 24.4 / 31.7 | 18.6 / 27.8 | 17.4 / 27.1 |
| (A) | **P^{2\|4}** | 24 / 16 | **21.2 / 24.0** | 20.1 / 23.3 | 17.9 / 21.8 | **17.5 / 21.5** |
| (A) | P^{4\|8} | 24 / 16 | 14.8 / 15.9 | 14.4 / 15.7 | 13.7 / 15.3 | 13.6 / 15.2 |
| (A) | P⁸ | 24 / 16 | 13.1 / 14.0 | 12.8 / 13.9 | 12.3 / 13.5 | 12.2 / 13.5 |
| (B) 7763, §5.1 walk | P^{2\|4} | 24 / 16 | 28.9 / 32.6 | 27.4 / 31.6 | 24.1 / 29.4 | 23.4 / 29.0 |
| (B) | P^{4\|8} | 24 / 16 | 20.4 / 21.8 | 19.9 / 21.5 | 18.9 / 20.8 | 18.7 / 20.7 |
| (C) 9V74 | P^{2\|4} | 24 / 16 | 28.5 / 32.3 | 26.8 / 31.2 | 23.4 / 28.9 | 23.0 / 28.7 |
| (C) | P^{4\|8} | 24 / 16 | 20.2 / 21.7 | 19.7 / 21.4 | 18.6 / 20.7 | 18.5 / 20.6 |
| (D) 9V45 | P^{2\|4} | 24 / 16 | 23.7 / 26.4 | 22.7 / 25.7 | 20.7 / 24.4 | 17.8 / 22.4 |
| (D) | P^{4\|8} | 24 / 16 | 16.2 / 17.3 | 15.9 / 17.1 | 15.3 / 16.7 | 14.4 / 16.1 |

**The boundary's sensitivities** (deduced from the model's linearity, base (A), P^{2|4},
conservative walk):
- **α_T:** +1.05 cycles of c_e for each +0.1 (that is walk_T₁/(β_N·e) = 4.49/0.43). On
  (D), +0.98 a 0.1.
- **α_N:** −0.44 cycles for each +0.1 (L·c_L/(β_N·e)).
- **β_N:** 1.52 in place of 1.36 lowers it by about 10% (21.2 → 18.8 at α_N = 1.26).
- **The corpus aggregate** in place of the median block lowers it by 2–4 cycles (P^{2|4}:
  20.1 → 17.4 at α_N = 1.5).
- **The one-copy shipped walk** is the largest single lever. Between bases (A) and (B), the
  same runner model at walk 4.49 against 6.69 c/B, the boundary moves by 6–8 cycles. On
  the 7763 the one-copy walk is 1.49–2.33 ns/B over 15 bench runs (§6).

**Inverted: the largest affordable α_N** (break-even, base (A), β_N = 1.36, conservative
walk):

| c_e | 6 | 10 | 16 | 20 | 24 | 27 |
|---|--:|--:|--:|--:|--:|--:|
| P² | 3.08 | 2.74 | 2.22 | 1.88 | 1.54 | 1.28 |
| P^{2\|4} | 4.72 | 3.81 | 2.45 | 1.54 | 0.63 | none |
| P^{4\|8} | 7.64 | 4.75 | 0.41 | none | none | none |

The same table for base (D), Zen 5:
- P^{2|4}: 5.59, 4.61, 3.15, 2.17, 1.20, 0.47;
- P^{4|8}: 9.23, 6.12, 1.46, none.

"None" means N1 loses even with no four-copy inflation of its walk at all.

**The four-copy pass and kill thresholds** (best plan at each c_e, as the note's §5.4
defines them):

| Base | α_N = 1.26 | 1.5 | 2.0 | α_N = α_T |
|---|---|---|---|---|
| (A) 7763, measured | pass ≤ 16.0 (P²), kill ≥ 29.2 | ≤ 13.2, ≥ 27.3 | ≤ 10.7 (P^{2\|4}), ≥ 23.4 | ≤ 10.3, ≥ 22.7 |
| (B) 7763, §5.1 walk | ≤ 21.1, ≥ 39.2 | ≤ 17.6, ≥ 36.4 | ≤ 14.3, ≥ 30.6 | ≤ 13.6, ≥ 29.4 |
| (C) 9V74 | ≤ 19.9, ≥ 38.4 | ≤ 17.0, ≥ 35.4 | ≤ 13.6, ≥ 29.4 | ≤ 13.3, ≥ 28.8 |
| (D) 9V45 | ≤ 19.5, ≥ 32.9 | ≤ 16.9, ≥ 31.1 | ≤ 13.0, ≥ 27.5 | ≤ 10.1, ≥ 22.5 |

**Against the note:**
- the note gave pass ≤ 31 (7763 and Zen 4) and ≤ 19 (Zen 5);
- its own formula with the factor applied the right way gives 24.0;
- with α_N = α_T, the four-copy pass threshold is the one-thread one, 10–14.

### 5.3 Regions over the plausible range: CPU against wall

**The plausible range, as a uniform grid:**
- c_e from 6 to 27 cycles: the note's model 6–16, and #42's CI-scaled anchor 16–27;
- α_N from 1.26 (the lowest measured stage factor) to α_T;
- α_T over its measured or deduced range: (A), (B) 2.00–2.20; (C) 1.93–2.17; (D) 2.2–3.2;
- β_N = 1.36.

Each cell gives the share of the grid where N1 loses on CPU per byte, the share where it
loses on wall, and the share where it wins on both. Each is for the conservative walk,
then the optimistic one.

| Base | P² | P^{2\|4} | P⁴ | P^{4\|8} | P⁸ |
|---|---|---|---|---|---|
| (A) 7763, measured | 20 / 17 / 71; 0 / 0 / 99 | **35 / 32 / 59; 19 / 16 / 75** | 44 / 42 / 51; 32 / 30 / 63 | **59 / 58 / 37; 53 / 52 / 43** | 66 / 65 / 30; 62 / 60 / 35 |
| (B) 7763, §5.1 walk | 4 / 3 / 91; 0 / 0 / 100 | 5 / 4 / 89; 0 / 0 / 100 | 14 / 12 / 80; 1 / 1 / 95 | 34 / 32 / 61; 26 / 24 / 69 | 44 / 42 / 52; 38 / 36 / 58 |
| (C) 9V74 | 6 / 4 / 89; 0 / 0 / 100 | 6 / 5 / 88; 0 / 0 / 100 | 15 / 13 / 78; 2 / 1 / 94 | 35 / 33 / 60; 27 / 25 / 69 | 44 / 43 / 51; 38 / 37 / 58 |
| (D) 9V45 | 20 / 17 / 73; 1 / 1 / 97 | 28 / 26 / 66; 13 / 12 / 82 | 38 / 36 / 57; 25 / 23 / 70 | 54 / 53 / 42; 48 / 46 / 49 | 62 / 61 / 35; 57 / 55 / 40 |

The figures are percentages: CPU loss / wall loss / both win; first conservative, then
optimistic.

**How to read it:**
- **Wall and CPU never disagree in sign.** The wall-loss share is 1–3 points below the
  CPU-loss share in every cell. The difference is the band where CPU loses beyond its ±1%
  but wall stays inside its ±2%. A matching 1–2% band wins on CPU while wall ties.
- **No cell wins on wall and loses on CPU** (or the reverse) beyond the noise. That
  follows from the model: at `Plan::Workers` wall is CPU/(4u), and N1 doesn't change u.
- **The shares are mostly the c_e prior.** Along c_e the boundary sits at 14–29 cycles. α_N
  and α_T together move it by only 2–6 cycles. On the measured 7763 base, P^{2|4} with
  the conservative walk loses wherever c_e ≳ 17.5–21, a third of the grid. A uniform prior
  over 6–16 alone (the instruction model) would put it at 0%. Over 16–27 (#42's untuned
  anchor) it would be about 70%.
- **Where the regions could diverge** (model): only through a change in utilisation u. A
  change Δu/u moves the wall boundary against the CPU one by about Δu/u × CPU_T/(β_N·e)
  cycles of c_e. For P^{2|4} on (A) that is 0.42 cycles a percent. A 3% change in N1's
  utilisation would open a band about 1.3 cycles wide, winning on one and losing on the
  other. There's no evidence it would move: the scanner's handoffs and the slot waits set
  u, not the inversion. Cooperative builds on otherwise idle siblings (teams, ≥ 6 cores)
  are the one mechanism that buys wall with CPU, and they are outside `Plan::Workers`.

### 5.4 CPU and wall at representative points (model; base (A), β_N = 1.36, conservative walk)

The shipped build + walk at four copies is 12.26 c/B a copy, and the whole decoder 17.96.
In wall terms that is about 1.62 ns/B at 4 workers ((17.96/3.24)/4/0.856). The bench
measures 1.91 ns/B for the whole 4-thread decoder on the 7763 (524 MB/s), which includes
work the stage table doesn't.

| Plan | c_e | α_N | N1 build | N1 walk | N1 build + walk | CPU, whole decoder | Wall |
|---|--:|--:|--:|--:|--:|--:|--:|
| P^{2\|4} | 10 | 1.26 | 5.10 | 2.38 | 7.47 | 0.73× | 0.75× |
| P^{2\|4} | 10 | 2.00 | 5.10 | 3.77 | 8.87 | 0.81× | 0.82× |
| P^{2\|4} | 22 | 1.26 | 10.23 | 2.38 | 12.61 | 1.02× | 1.02× (tie) |
| P^{2\|4} | 22 | 2.00 | 10.23 | 3.77 | 14.01 | 1.10× | 1.09× |
| P^{4\|8} | 10 | 1.26 | 7.74 | 1.21 | 8.95 | 0.81× | 0.83× |
| P^{4\|8} | 22 | 2.00 | 16.05 | 1.92 | 17.96 | 1.32× | 1.29× |

## 6. Assumptions whose failure would reverse N1's projected advantage

1. **That N1's walk escapes the SMT term (α_N well below α_T).**
   - The evidence says the regression is per-core sharing, which a smaller table doesn't
     avoid (§3).
   - If each lookup has two L2-missing accesses, or the walk is at the per-core cap at
     one copy, α_N = α_T. The four-copy advantage then vanishes, and the threshold
     returns to the one-thread 10–14.
2. **That the build doesn't contend more than the shipped build.**
   - β_N = 1.36 is MTF's factor. A builder tuned to saturate the core, as the walk is,
     could reach ×1.5–2.
   - At β_N = 1.52 the boundary falls by about 10%.
3. **That the one-copy shipped walk of the reference run stands for the release.**
   - On the 7763 it ranges 1.49–2.33 ns/B over 15 bench runs. The compute stages stay at
     0.69–0.71 ns/B, so the walk's spread is the memory system's (other tenants), not the
     clock.
   - N1's cost is tied to it only through the c_L convention.
   - A fast runner instance moves the boundary down by 6–8 cycles. Base (A) is such an
     instance.
4. **That lookups stay at about 2 dependent L2/L3 accesses and 16–24 instructions.** Under
   SMT the 512 KB (Zen 3) or 1 MB (Zen 4, Zen 5) L2 is shared by two copies' directories
   and records.
5. **That the non-inversion stages are unchanged by N1.**
   - If N1's walk loads the sibling less, MTF and the scanner beside it may speed up, in
     N1's favour.
   - If its build runs beside a sibling's walk, the reverse.
   - Neither is measured.
6. **That "c/B" means the same on both sides.** It is wall × the reported clock. c_e
   measured with core-cycle counters would be in other units. On the 9V45 the reported
   clock varies 1.75× at a fixed speed.
7. **That four processes stand for four threads.** For the shipped decoder they agree
   (×1.67 against ×1.74, §2.2), but this is untested for N1.
8. **That Zen 5's α_T is 2.2–3.2** (deduced from whole-decoder amplification, under the
   assumption about the other stages).
   - If its compute stages carry more of the 2.21, α_T is near 2.2. The Zen 5 boundary
     then drops by about 5 cycles.
   - If its walk carries more, N1 gains on Zen 5 at four workers.

## 7. Negative findings

1. **The note's four-copy threshold of 31 is not supported.** It has an inverted factor
   (24 with that fixed), an underestimated shipped-build factor, and an α_N below every
   measured analogue. The corrected range is 10–21 cycles (pass) across runners.
2. **Shared L3 capacity is not the main cause of CI's four-copy doubling.** The plain
   chain's ×1.045 and the VM's ×1.08 with four separate cores both say so. The note's
   framing, "N1's table is 1.0–2.0 MB against 3.6", is therefore not the lever on CI's
   runners. It is the lever past 4 workers on the i9 (P112–P113).
3. **No frequency effect is visible** (≤ about 4.5%).
4. **The 5.0 → 10.4 figure is probably from a 9V74, not the 7763** (inferred from the
   reported clock). The 7763's own measurement, ×2.09, agrees with it.
5. **No N1, compact-Ψ or move-structure four-copy run exists on an SMT runner.** The only
   smaller-table multi-worker data (the local VM) has no SMT exposed, so it can't inform
   α_N under SMT.
6. **4K aliasing and TLB:** no evidence for or against on CI. Nothing in the journals
   tests them.

## 8. What would pin α_N (not authorised now)

**Required:** each of these at one copy, two copies on siblings (CPUs 0,1), two copies on
separate cores (0,2), and four copies:
- the per-copy walk time, in ns/B as well as reported cycles, paired by round;
- the shipped room walk in the same run as the control.

The sibling/separate split gives SMT and cross-core apart. Without it, α_N can't be
transferred to a machine with another topology (a laptop with 4–8 real cores).

**The cheapest, with existing code:**
1. `sweep.yml` already runs a tool at one copy and at one copy a CPU. Adding the two
   2-copy placements is a few lines.
2. **The cheapest proxy for "a smaller table with a heavier step":** #112's compact-Ψ arms
   (`compact.rs`, 2.3 MB tables), or E8's move structure (`scratch/move-structure`).
   - Either run in that workflow gives a measured α for a sub-T-vector table under SMT,
     in about 10 minutes of CI.
   - It bounds α_N from the side of "instruction-heavier walker". It doesn't measure N1.
3. **The deciding measurement** is Gate C's N1 walker over stored P^{2|4} records, at the
   same four placements. It needs the walker, not the builder: records can be built
   offline, untimed.
4. Gate B's builder at the same placements gives β_N. A single four-copy figure is enough
   there, since the build's analogue factors span only 1.26–1.52.

## 9. Files

Read-only scripts and data, kept in the scratchpad's `n1-fourcopy/` (not committed):
- **`fourcopy_model.py`:** the model of §5. Its output is `fourcopy_model.json` (surfaces,
  regions, thresholds and the reproduction of the note's 31).
- **`alpha_n_model.py`:** the per-core-cap α_N model of §4. Its output is
  `alpha_n_model.json`.
- **`parse_fourcopy.py`:** parses the two CI artifacts (`art-40579732655/`,
  `art-40573190665/`). Its output is `measured_fourcopy.json`.
- **`extract_bench.py`:** extracts per-runner amplification and stages from
  `comments-recent.json`, the lmmx/mibz2 issue-comments listing. Its output is
  `bench_runs.json`.
- **`vm_pilot.py`:** the local VM's four-worker factors from `../compact-vm.jsonl`. Its
  output is `vm_pilot.json`.
- **`log-40573190665.txt`, `log-40579732655.txt`:** the two runs' job logs (runner lines).

Inputs: `../data/census.json`, `../n1-analysis/model.json`. The journals are
`2026-10-27-walk-sweep.md`, `2026-10-27-output-side.md`, `2026-10-27-four-thread-ceiling.md`,
`2026-10-28-workstation-scaling.md` and `2026-10-28-early-warning.md`; the portfolio's
`2026-10-30-research-portfolio-register.md` (P110–P113) and
`2026-10-30-compact-psi-feasibility.md`.
