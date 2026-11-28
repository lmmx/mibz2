# Probe: `forward-lanes-mlp-and-prefetch` — `K = 6/8` Bounded-Slab Forward Lanes & `edges[next]` Prefetch

**Scope & honesty notice:** All timings and disassembly counts in this note were measured on a shared Linux KVM VM (`rustc 1.99.1`, pinned CPU 0, steal-free `CLOCK_THREAD_CPUTIME_ID`, `[profile.release]` `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`) on top of PR `#1312`'s bounded-slab `in-out` `4 KiB` chunk-permutation architecture (`4f3c9ac2`, `SLAB = 4 KiB`, `MAX_TAILS = 16 × 4 KiB = 64 KiB`, active scratch `80 / 88 / 92 KiB/worker <= 128 KiB`). **These are VM probe measurements, NOT physical i9-10900K adoption evidence.**

**Unrelaxed C2 verdict: `STOP` on the frozen `>= 12.0%` L3-cold `base` gate** (despite a real, disjoint cross-profile win: **`-18.49%` `jcc` / `-11.17%` `base` L3-warm `A7`**, **`-17.12%` `jcc` / `-6.19%` `base` L3-cold `A7`**, and **`-14.48%` `jcc` / `-9.81%` `base` L3-warm end-to-end `A7 + A8+A9`** at `92 KiB/worker <= 128 KiB`!).

## 1. Setup, Census & Exactness

Extracted exact production forward-emission inputs (`edges`, `spelling`, `inline16`, `lens`, `mark`, `bits`, `n`, `crc`) from PR `#1312` (`4f3c9ac2`, `--features forward-anchor-lanes`):
- **Corpus:** All **`233 / 233`** production-admitted Wikidata Open blocks (`avg edges = 445.4 KiB`, `avg inline16 = 16.5 KiB`) plus **`1 / 1`** forced-Open block from `runs.bz2` (`scale = 4`, `10,000,235 B` -> `37,995 B`).
- **Exactness:** Every variant (`M0..M9`) verified byte-exact on BWT output (`want_bwt`), RLE1-decoded bytes (`want_decoded`), and block CRC-32 against `mibz2::decode_block::<bwt::Plain>` across all 234 blocks on both `base` and `jcc`: **`4,680 / 4,680` checks PASS (100% exact)**.
- **Per-worker scratch bound (`<= 128 KiB`):** `M0..M2` (`K = 4`) = **`80 KiB`**, `M3..M5` (`K = 6`) = **`88 KiB`**, `M6..M9` (`K = 8`) = **`92 KiB`** (all strictly `<= 128 KiB`).

## 2. Complete Paired Results (`n = 7` Interleaved Repeats, CPU 0, Steal-Free CPU)

### Baseline x86-64 (`base`)

| Arm (`scratch`) | Warm `A7` (ms) | Warm `A7` vs `M0` [95% CI] | Warm `A7+A8+A9` vs `M0` | Cold `A7` (ms) | Cold `A7` vs `M0` [95% CI] | Cold `A7+A8+A9` vs `M0` |
|---|---:|---|---:|---:|---|---:|
| **`M0_k4_1312`** (`80 KiB`) | `306.23` | `+0.00%` | `+0.00%` (`357.69 ms`) | `301.01` | `+0.00%` | `+0.00%` (`362.50 ms`) |
| **`M1_k4_pf_t0`** (`80 KiB`) | `294.28` | **`-4.69%`** `[0.9390, 0.9675]` | `-3.34%` | `292.49` | `-2.82%` `[0.9403, 1.0045]` | `-2.25%` |
| **`M2_k4_pf_t1`** (`80 KiB`) | `295.03` | **`-4.37%`** `[0.9401, 0.9728]` | `-4.76%` | `299.29` | `-1.18%` `[0.9602, 1.0170]` | `-0.87%` |
| **`M3_k6_4k`** (`88 KiB`) | `323.77` | **`+5.09%`** `[1.0229, 1.0798]` | `+2.33%` | `317.95` | **`+5.61%`** `[1.0410, 1.0714]` | `+5.06%` |
| **`M4_k6_pf_t0`** (`88 KiB`) | `283.13` | **`-7.89%`** `[0.9017, 0.9409]` | `-6.52%` | `289.41` | **`-4.29%`** `[0.9363, 0.9783]` | `-3.55%` |
| **`M5_k6_pf_t1`** (`88 KiB`) | `284.91` | **`-7.51%`** `[0.9058, 0.9444]` | `-5.39%` | `293.59` | **`-2.99%`** `[0.9560, 0.9845]` | `-2.45%` |
| **`M6_k8_4k`** (`92 KiB`) | `315.89` | **`+3.70%`** `[1.0027, 1.0725]` | `+2.63%` | `318.76` | **`+5.01%`** `[1.0311, 1.0694]` | `+4.35%` |
| **`M7_k8_pf_t0`** (`92 KiB`) | **`270.90`** | **`-11.17%`** `[0.8647, 0.9125]` | **`-9.45%`** `[0.879, 0.933]` | `286.04` | **`-5.49%`** `[0.9218, 0.9689]` | **`-4.70%`** `[0.926, 0.981]` |
| **`M8_k8_pf_t1`** (`92 KiB`) | `272.26` | **`-11.03%`** `[0.8585, 0.9220]` | **`-9.81%`** `[0.879, 0.925]` | **`282.70`** | **`-6.19%`** `[0.9084, 0.9687]` | **`-5.25%`** `[0.914, 0.982]` |
| **`M9_k8_pf_ei`** (`92 KiB`) | `295.32` | `-3.74%` `[0.9429, 0.9827]` | `-3.60%` | `296.78` | `-2.12%` `[0.9464, 1.0123]` | `-1.88%` |

### JCC-32B-Aligned (`jcc`, `-C llvm-args=-x86-branches-within-32B-boundaries`)

| Arm (`scratch`) | Warm `A7` (ms) | Warm `A7` vs `M0` [95% CI] | Warm `A7+A8+A9` vs `M0` | Cold `A7` (ms) | Cold `A7` vs `M0` [95% CI] | Cold `A7+A8+A9` vs `M0` |
|---|---:|---|---:|---:|---|---:|
| **`M0_k4_1312`** (`80 KiB`) | `321.26` | `+0.00%` | `+0.00%` (`404.62 ms`) | `340.95` | `+0.00%` | `+0.00%` (`409.50 ms`) |
| **`M1_k4_pf_t0`** (`80 KiB`) | `302.47` | `-5.70%` `[0.8585, 1.0358]` | **`-8.29%`** `[0.866, 0.971]` | `309.07` | **`-11.63%`** `[0.7814, 0.9993]` | `-10.12%` |
| **`M2_k4_pf_t1`** (`80 KiB`) | `318.31` | `-3.78%` `[0.8869, 1.0440]` | `-6.07%` | `317.83` | `-8.81%` `[0.8276, 1.0047]` | `-7.59%` |
| **`M3_k6_4k`** (`88 KiB`) | `310.66` | `-5.72%` `[0.8834, 1.0062]` | `-4.17%` | `342.66` | `-2.35%` `[0.8809, 1.0824]` | `-2.28%` |
| **`M4_k6_pf_t0`** (`88 KiB`) | `302.77` | `-6.45%` `[0.8333, 1.0502]` | **`-8.35%`** `[0.885, 0.949]` | `323.05` | `-8.17%` `[0.8266, 1.0202]` | `-7.15%` |
| **`M5_k6_pf_t1`** (`88 KiB`) | `288.98` | **`-12.24%`** `[0.8116, 0.9490]` | **`-9.66%`** `[0.840, 0.972]` | `316.69` | **`-8.77%`** `[0.8343, 0.9976]` | **`-7.48%`** |
| **`M6_k8_4k`** (`92 KiB`) | `312.96` | `-4.62%` `[0.8861, 1.0268]` | `-0.84%` | `323.76` | `-5.63%` `[0.8709, 1.0225]` | `-5.13%` |
| **`M7_k8_pf_t0`** (`92 KiB`) | **`266.34`** | **`-18.49%`** `[0.7461, 0.8906]` | **`-14.40%`** `[0.806, 0.909]` | `286.84` | **`-16.57%`** `[0.7334, 0.9492]` | **`-14.59%`** `[0.763, 0.956]` |
| **`M8_k8_pf_t1`** (`92 KiB`) | `277.04` | **`-15.69%`** `[0.7718, 0.9210]` | **`-14.48%`** `[0.810, 0.903]` | **`280.58`** | **`-17.12%`** `[0.7242, 0.9485]` | **`-15.03%`** `[0.755, 0.956]` |
| **`M9_k8_pf_ei`** (`92 KiB`) | `292.19` | `-9.56%` `[0.7988, 1.0240]` | **`-12.15%`** `[0.840, 0.919]` | `299.03` | `-12.58%` `[0.7574, 1.0091]` | `-10.84%` |

## 3. Disassembly Audit (`objdump -dC`, `base` & `jcc`)

| Symbol | `base` insns (`vzu/ymm`, `t0/t1`, `rld/rst`) | `jcc` insns (`vzu/ymm`, `t0/t1`, `rld/rst`) |
|---|---|---|
| `Walk::run::<4, 0>` (`M0`) | `491` (`0/0`, `0/0`, `81/33`) | `536` (`0/0`, `0/0`, `81/33`) |
| `Walk::run::<4, 1>` (`M1`, T0) | `533` (`0/0`, `4/0`, `92/29`) | `562` (`0/0`, `4/0`, `92/29`) |
| `Walk::run::<6, 0>` (`M3`) | `345` (`0/0`, `0/0`, `56/31`) | `359` (`0/0`, `0/0`, `56/31`) |
| `Walk::run::<6, 1>` (`M4`, T0) | `354` (`0/0`, `6/0`, `54/31`) | `374` (`0/0`, `6/0`, `54/31`) |
| `Walk::run::<8, 0>` (`M6`) | `391` (`0/0`, `0/0`, `70/38`) | `417` (`0/0`, `0/0`, `70/38`) |
| `Walk::run::<8, 1>` (`M7`, T0) | **`412`** (`0/0`, **`8/0`**, **`57/38`**) | **`411`** (`0/0`, **`8/0`**, **`57/38`**) |
| `Walk::run::<8, 2>` (`M8`, T1) | **`412`** (`0/0`, **`0/8`**, **`57/38`**) | **`411`** (`0/0`, **`0/8`**, **`57/38`**) |
| `Walk::run::<8, 3>` (`M9`, E+I) | `430` (`0/0`, `16/0`, `61/42`) | `462` (`0/0`, `16/0`, `61/42`) |

## 4. Mechanism Analysis & Strict C2 Verdict

1. **Why `K = 6/8` without prefetch (`M3`, `M6`) loses (`+3.7%..+5.6%` `base`):** LLVM fully unrolls `run::<4, 0>` (`491` insns) but keeps `for i in 0..K` rolled for `K = 6` (`345` insns) and `K = 8` (`391` insns). Without explicit `edges[next]` prefetch, the extra rolled-loop overhead and register pressure prevent the OoO window from discovering next-iteration `edges` loads earlier.
2. **Why `K = 8 + edges[next]` prefetch (`M7`/`M8`) wins sharply across every cell (yet stops at the `>= 12.0%` `base` L3-cold gate):**
   - Issuing 8 `prefetcht0`/`prefetcht1` instructions immediately after the `OR < 0` anchor check (and before the 8-lane `lens`/`inline16`/slab copy loop) hides L2/L3 miss latency on `edges` (`avg 445.4 KiB > 256 KiB L2`), even reducing stack reloads vs `run::<8, 0>` (`rld 70 -> 57`), and bringing absolute `A7` time down to **`266–277 ms` warm / `281–287 ms` cold** on **both** `base` and `jcc`.
   - Adding `inline16` prefetch (`M9`) hurts vs `M7`/`M8` because `avg inline16` is only **`16.5 KiB`** (already L1d-resident), wasting 8 load-port µops per step.
   - Because each Wikidata block's `edges` averages **`445.4 KiB`** (`< L3`, `> 256 KiB L2`), a single-thread `24 MiB` LLC flush warms `edges` into L3 early in the block; combined with `#1312`'s invariant in-place `4 KiB` chunk permutation + backward tail stitch (~20% of `A7`), `base` L3-cold `A7` lands at **`-6.19%` (`M8`) / `-5.49%` (`M7`)**, which is **disjoint faster** (`[0.9084, 0.9687]`) yet below the frozen `>= 12.0%` `base` L3-cold threshold (whereas `jcc` L3-cold clears it at **`-17.12%` / `-16.57%`** and warm `A7` reaches **`-11.17%` `base` / `-18.49%` `jcc`**).
3. **Unrelaxed C2 disposition:** Under C2 (`CLAUDE.md`), the frozen `>= 12.0%` L3-cold `base` gate is NOT relaxed -> **`STOP`** on `forward-lanes-mlp-and-prefetch` (`status = "done"`, scoped row appended to `frontier/dead-ends.tsv`). Crucially, the real, disjoint cross-profile **`-6.2%..-18.5%` `A7` / `-4.7%..-15.0%` `A7+A8+A9`** win from `K = 8 + edges[next]` prefetch (`92 KiB/w`, `0` `vzeroupper`) is **NOT closed**: a calibrated follow-on (e.g., paired with `L6a` zero-stitch streaming where chunk-permutation overhead is removed, or targeting multi-worker 8W L3 contention on the i9 where `8 × 445 KiB + 8 × out` genuinely spills L2/L3) remains eligible.
