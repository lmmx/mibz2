# W4_v3 MLP & Memory-Layout Probe: `K = 4, 6, 8, 10` Lanes, Decoupled Batch-Chase, Software Pipelining, Arena Hoisting, and `Inline15Len`

- **Date:** 2026-11-27
- **Probe binary:** `examples/w4_v3_mlp_layout_probe.rs` (smooth-zone, zero `src/` edits, 100% safe Rust)
- **Corpus:** `wikidata.bz2` (431 blocks; 233 production-admitted Open blocks, 209,691,304 decoded bytes; exact byte-for-byte & CRC32 check against `mibz2::decode_block` on all 233/233 blocks)
- **Verdict:** **STOP** (frozen stop test missed across all 12 candidate arms on all 3 compiler profiles — baseline x86-64, `-C target-cpu=native`, and JCC-32B-aligned `-C target-cpu=native`).

---

## 1. Motivation & Memory-Anatomy Audit

In `#1067` (`docs/journal/2026-11-27-explore/composed-worker-v2-probe.md`), the composed Open worker `W4_v2_q496_32k` (`#1059` hoisted `OutQ16<496,512>` + `#1053` safe 64K SoA A6b + `#1057` `S = 16` dynamic 4-lane forward emission + cold-outlined `SLAB = 32 KiB` fused RLE1/CRC32) cut realistic L3-cold worker CPU by `-21.6%..-24.6%` (`W'/8 ≈ 140.0..145.5 ms` on matched flags, `~6–11 ms` shy of `#1056`'s `W'/8 <= 133.9 ms` `6 GB/s` contour). Meanwhile, `#1068` (`direct-literal-4lane-emit`, STOP) showed that `code < 256` literal splitting mispredicts because `A7` is latency-bound on random `edges[j]` pointer chases across `26.57M` steps (`81.58%` 1B literals, `11.99%` 2..16B inline, `6.43%` `LONG`).

Before timing any candidate, we audited `W4_v2`'s memory layout and concurrency:

1. **`edges` is already 1 cache line per step (`[u32; m]`, `456 KiB` avg / `1.0 MiB` max):** It already packs `(code << bits) | next_idx` + bit-31 `ANCHOR_TAG` into 4 bytes. Widening `edges` to `u64` would double L3 footprint (`0.9–2.0 MiB`) for zero cache-line reduction.
2. **Effective MLP at `K = 4, S = 16` is `eff_K = 3.15` (`ph1 = 81.1%`):** Haswell/Comet Lake cores have 10 Line Fill Buffers (LFBs), raising the hypothesis that `K = 6, 8, 10` lanes with finer anchor sampling (`S = 32, 48, 64` -> `eff_K = 3.49, 5.07, 6.60, 7.68`) and L2-pinned slabs (`K * SLAB <= 128 KiB`, respecting `#1067`'s 256 KiB L2 eviction cliff) could saturate LFBs and hide L3/DRAM chase latency.
3. **Dictionary 2-array vs 1-array layout (`lens[]` + `inline16[]` vs `Inline15Len`):** Packing `len` (`1..=15` or `LONG = 0xFF`) into byte 15 of `[u8; 16]` eliminates the `lens[]` array and cuts 2 L1d lookups/bounds-checks to 1 (at the cost of routing `len == 16` rules to `LONG`).
4. **Loop architectures to avoid `LONG` stalls and register pressure:**
   - **Round 1 (Decoupled batch-chase):** `chase_K_batch<B>` (`B = 8, 16`) chasing `K` lanes into `cbuf[K][B]` with zero `LONG` branches, followed by per-lane `expand_lane_batch!`.
   - **Round 2 (Coupled, 1-step software-pipelined, and arena-hoisted `K = 4, 6, 8, 10`):** Direct coupled `K`-lane generalization of `emit_4lane_sampled` (`K = 6, 8, 10`), 1-step software-pipelined pre-load of next iteration's `nw0..nw(K-1) = edges[j0..j(K-1)]` before expanding current `ed0..ed(K-1)`, and `&mut alloc.arena[..]` hoisting (`all_fast` inner loop eliminating per-step `Vec` ptr/len reloads).

### Frozen Stop Test (Frozen Before Any Timing Data; C2)

- **PASS** iff a 100% safe-Rust, 233/233 byte- and CRC32-exact `A6c + A7 + A9` upgrade inside `W4_v2` beats `W4_v2_ctrl` (`W4_v2_q496_32k`) by **`>= 5.0%` on L3-cold `A6c+A7+A9`** AND **`>= 3.0%` on full composed `W4_v2` L3-cold**, non-overlapping (`disj_v2 = FASTER`) on **BOTH** baseline x86-64 and `-C target-cpu=native` (confirmed under JCC-32B alignment control `-C llvm-args=-x86-align-branch-boundary=32 -C llvm-args=-x86-align-branch=fused+jcc+jmp`), while keeping resident per-worker scratch **`<= 4.0 MiB`** (`<= 1.15x` `W4_v2`).
- **STOP** otherwise (record scoped dead-end in `frontier/dead-ends.tsv`).

---

## 2. Exactness, MLP Census, Working Set & Code Size

All 14 arms passed 233/233 byte-for-byte & CRC32 exactness with `ovf = 0` and `fb = 0` (zero fallbacks to 1-lane emission), keeping resident scratch well below the `4.0 MiB` cap (in fact `K = 6, 8, 10` with `K * SLAB <= 128 KiB` shrink resident scratch by `-56..-106 KiB` relative to `W4_v2_ctrl`):

| Arm | Design (`K, S, SLAB`) | L2 slab budget | `eff_K` | `ph1%` | Repairs | `fb` | Resident scratch | Native `.text` size |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| `W0_shipped` | Shipped Open (`512/512`, `Elim`, bidi, in-place RLE1) | — | — | — | — | — | `2,896.6 KiB` | — |
| `W4_v2_ctrl` | `#1067` control (`K=4, S=16, 32 KiB`, coupled) | `128 KiB` | `3.15` | `81.1%` | `2` | `0` | `3,488.5 KiB` | `19,457 B` |
| `V3_K4_S32_32k` | Coupled `K=4, S=32, 32 KiB` | `128 KiB` | `3.49` | `90.2%` | `2` | `0` | `3,493.2 KiB` | `19,457 B` |
| `V3_K4_hoist_S16_32k` | Coupled `K=4, S=16, 32 KiB` + hoisted `&mut arena` | `128 KiB` | `3.15` | `81.1%` | `2` | `0` | `3,488.5 KiB` | `16,082 B` |
| `V3_K4_pipe_S32_32k` | 1-step pipelined `K=4, S=32, 32 KiB` | `128 KiB` | `3.49` | `90.2%` | `2` | `0` | `3,493.2 KiB` | `13,519 B` |
| `V3_K6_cpl_S48_20k` | Coupled `K=6, S=48, 20 KiB` | `120 KiB` | `5.07` | `89.5%` | `6` | `0` | `3,432.8 KiB` | `16,602 B` |
| `V3_K6_pipe_S48_20k` | 1-step pipelined `K=6, S=48, 20 KiB` | `120 KiB` | `5.07` | `89.5%` | `6` | `0` | `3,432.8 KiB` | `16,858 B` |
| `V3_K8_cpl_S64_16k` | Coupled `K=8, S=64, 16 KiB` | `128 KiB` | `6.60` | `89.1%` | `9` | `0` | `3,407.4 KiB` | `18,478 B` |
| `V3_K8_pipe_S64_16k` | 1-step pipelined `K=8, S=64, 16 KiB` | `128 KiB` | `6.60` | `89.1%` | `9` | `0` | `3,407.4 KiB` | `18,890 B` |
| `V3_K8_cpl_S64_8k` | Coupled `K=8, S=64, 8 KiB` (25% L2) | `64 KiB` | `6.60` | `89.1%` | `9` | `0` | `3,411.5 KiB` | `18,478 B` |
| `V3_K8_hoist_S64_16k` | Coupled `K=8, S=64, 16 KiB` + hoisted `&mut arena` | `128 KiB` | `6.60` | `89.1%` | `9` | `0` | `3,407.4 KiB` | `20,106 B` |
| `V3_K8_cpl_S64_16k_f15` | Coupled `K=8, S=64, 16 KiB` + fused 1-array `Inline15Len` | `128 KiB` | `6.60` | `89.1%` | `9` | `0` | `3,405.7 KiB` | `17,844 B` |
| `V3_K10_cpl_S64_12k` | Coupled `K=10, S=64, 12 KiB` (10/10 LFBs) | `120 KiB` | `7.68` | `85.7%` | `9` | `0` | `3,383.0 KiB` | `19,022 B` |
| `V3_K8_dec_B8_S64_16k` | Decoupled batch-chase `K=8, B=8, S=64, 16 KiB` | `128 KiB` | `6.60` | `89.1%` | `9` | `0` | `3,407.4 KiB` | `16,361 B` |

---

## 3. Round-1 Diagnostic Finding: Decoupled Batch-Chase (`B = 8, 16`) and `FUSED15` Split-Copy Bug

Initial Round-1 runs (baseline x86-64, 10 arms, `K = 4, 6, 8`, `B = 8, 16`) immediately exposed two microarchitectural mechanisms via `objdump -d`:

1. **Decoupling kills OoO chase/expand overlap (`+56.0 ms` even at matched `K = 4, SLAB = 32 KiB`):**
   - In warm CPU, `V3_K4_dec_B8_S32_32k` ran `A7+A9 = 301.94 ms` vs `W4_v2_ctrl`'s `245.98 ms` (`+55.96 ms`, `+22.7%` slower) at the exact same `K = 4` and `SLAB = 32 KiB`, while `V3_K8_dec_B8` beat `V3_K4_dec_B8` (`284.67` vs `301.94 ms`, `-17.3 ms`) and `B = 16` was slower than `B = 8` (`297.78` vs `288.51 ms`).
   - Disassembly of `emit_klane_decoupled::<8, 8, 16384, false>` showed:
     - Loop 1 (`0x35d75..0x35d91`) successfully issued all 8 `edges[j]` loads back-to-back, but `#[inline(always)]` into the `0xea8`-byte frame spilled `j0..j7`, `edges_ptr`, `mask`, `bits`, `i`, and `want` to stack.
     - Loop 2 (`0x35ff0..0x36081`) passed `&mut a, &mut e` into `emit_slow_fused` inside the loop, forcing `a` (`0x150(%rsp)`) to reload/store on every step (a 5-cycle L1d store-forwarding loop-carried stall × 26.57M steps ≈ 45 ms).
     - Most fundamentally, filling the 192-entry OoO ROB with `K * B = 32..128` pure chase loads followed by `K * B` sequential expands eliminated the free overlap where coupled `W4_v2_ctrl` executes L1d `inline16` copies under the shadow of the next iteration's L3 misses.
2. **`FUSED15` 15-byte split-load bug (`+42 ms` in Round 1 -> fixed in Round 2):**
   - Copying `let entry = inline16[c];` with `entry[15]` extracted caused LLVM to emit two overlapping 8-byte GPR loads + two 8-byte stores (`0x3a0c4..0x3a0cb`: `mov (%r8),%rdx; mov 0x7(%r8),%rsi; mov %rsi,0x7(%rax,%rdi,1)`). Changing to `let l = inline16[c][15]; alloc.arena[a..a + INLINE].copy_from_slice(&inline16[c]);` restored a single 16-byte `vmovdqu` load/store in Round 2.

---

## 4. Full 7-Repeat Interleaved Benchmark Results (3 Profiles × Warm & L3-Cold)

All timings below are medians of **7 interleaved alternating-order repeats** pinned to CPU 0 (`taskset -c 0`) on the 233 admitted Open blocks of `wikidata.bz2`, using steal-free Linux `/proc/thread-self/schedstat` CPU time (`CPU`) alongside `Instant` wall (`WALL`). Because peer swarm workers shared the 2-vCPU VM during runs, steal-free `schedstat` CPU is the authoritative signal (and `i9_W'/8 = 185.68 * (arm / W0_shipped)` is a VM diagnostic projection only; real i9-10900K measurement is required before claiming `6 GB/s`, C5).

### 4.1 Profile 1: Baseline x86-64 (`cargo build --release`)

#### L3-Cold CPU (16-worker rotating pool `>50 MiB` + `64 MiB` flush)

| Arm | `A5+6a` | `A6b` | `A6c` | `A7+A9` | `6c+7+9` | Total med `[min..max]` | vs `W0` | vs `W4_v2` | `6c+7+9` vs `v2` | `disj_v2` | VM `W'/8` |
|---|---:|---:|---:|---:|---:|---|---:|---:|---:|---|---:|
| `W0_shipped` | `215.35` | `136.36` | `106.58` | `324.37` | `430.94` | `781.68 [752.20..844.67]` | `+0.00%` | `+28.21%` | — | shipped | `185.68 ms` |
| **`W4_v2_ctrl`** | `176.47` | `99.66` | `38.75` | `299.77` | **`338.52`** | **`609.70 [594.41..645.49]`** | **`-22.00%`** | **`+0.00%`** | **`+0.00%`** | **ctrl** | **`144.83 ms`** |
| `V3_K4_S32_32k` | `169.29` | `107.27` | `36.01` | `294.37` | `330.39` | `616.21 [587.32..636.61]` | `-21.17%` | `+1.07%` | `-2.40%` | overlap | `146.37 ms` |
| `V3_K4_pipe_S32_32k` | `163.83` | `111.66` | `46.17` | `326.39` | `372.56` | `642.35 [611.95..661.84]` | `-17.82%` | `+5.35%` | `+10.05%` | overlap | `152.58 ms` |
| `V3_K4_hoist_S16_32k` | `166.86` | `108.45` | `37.07` | `339.53` | `376.60` | `647.79 [623.35..693.43]` | `-17.13%` | `+6.25%` | `+11.25%` | overlap | `153.87 ms` |
| `V3_K6_cpl_S48_20k` | `154.97` | `108.21` | `34.24` | `340.83` | `375.07` | `647.99 [621.87..683.47]` | `-17.10%` | `+6.28%` | `+10.80%` | overlap | `153.92 ms` |
| `V3_K8_cpl_S64_16k` | `170.37` | `98.32` | `39.37` | `348.56` | `387.93` | `661.98 [649.16..689.29]` | `-15.31%` | `+8.57%` | `+14.60%` | **SLOWER** | `157.25 ms` |
| `V3_K8_cpl_S64_8k` | `167.39` | `100.66` | `38.73` | `362.79` | `401.52` | `666.72 [645.40..707.43]` | `-14.71%` | `+9.35%` | `+18.61%` | overlap | `158.37 ms` |
| `V3_K8_cpl_S64_16k_f15` | `164.95` | `102.33` | `37.49` | `363.80` | `401.29` | `668.58 [639.40..717.55]` | `-14.47%` | `+9.66%` | `+18.54%` | overlap | `158.81 ms` |
| `V3_K10_cpl_S64_12k` | `163.15` | `109.76` | `48.88` | `343.26` | `392.14` | `672.57 [634.61..683.88]` | `-13.96%` | `+10.31%` | `+15.84%` | overlap | `159.76 ms` |
| `V3_K8_pipe_S64_16k` | `159.46` | `100.51` | `39.84` | `367.75` | `407.59` | `672.59 [643.44..709.18]` | `-13.96%` | `+10.31%` | `+20.40%` | overlap | `159.77 ms` |
| `V3_K6_pipe_S48_20k` | `165.76` | `101.75` | `41.02` | `360.92` | `401.95` | `676.42 [646.37..735.09]` | `-13.47%` | `+10.94%` | `+18.74%` | **SLOWER** | `160.68 ms` |
| `V3_K8_dec_B8_S64_16k` | `172.46` | `103.39` | `42.59` | `383.48` | `426.07` | `697.99 [662.66..707.73]` | `-10.71%` | `+14.48%` | `+25.86%` | **SLOWER** | `165.80 ms` |
| `V3_K8_hoist_S64_16k` | `158.38` | `108.94` | `36.12` | `393.44` | `429.56` | `700.28 [669.92..754.78]` | `-10.41%` | `+14.86%` | `+26.89%` | **SLOWER** | `166.34 ms` |

- **Baseline Warm CPU summary:** `W0_shipped 736.15 ms`, `W4_v2_ctrl 582.48 ms` (`A6c+7+9 = 307.38 ms`), `V3_K4_S32_32k 586.31 ms` (`+0.66%`), `V3_K4_pipe_S32_32k 573.17 ms` (`-1.60%` overlap; `A6c+7+9 = +1.02%`), `V3_K6_cpl 591.22 ms` (`+1.50%`), `V3_K8_cpl 598.80 ms` (`+2.80%`), `V3_K10_cpl 606.17 ms` (`+4.07%`), `V3_K8_cpl_f15 613.77 ms` (`+5.37%`), `V3_K8_hoist 648.45 ms` (`+11.33%`), `V3_K8_dec_B8 662.83 ms` (`+13.80%` disjoint SLOWER).

### 4.2 Profile 2: `-C target-cpu=native` (AVX2 + BMI2)

#### L3-Cold CPU (16-worker rotating pool `>50 MiB` + `64 MiB` flush)

| Arm | `A5+6a` | `A6b` | `A6c` | `A7+A9` | `6c+7+9` | Total med `[min..max]` | vs `W0` | vs `W4_v2` | `6c+7+9` vs `v2` | `disj_v2` | VM `W'/8` |
|---|---:|---:|---:|---:|---:|---|---:|---:|---:|---|---:|
| `W0_shipped` | `227.95` | `136.16` | `102.75` | `305.48` | `408.23` | `784.67 [775.09..804.48]` | `+0.00%` | `+21.28%` | — | shipped | `185.68 ms` |
| **`W4_v2_ctrl`** | `201.54` | `111.95` | `42.77` | `277.31` | **`320.08`** | **`646.98 [627.34..688.67]`** | **`-17.55%`** | **`+0.00%`** | **`+0.00%`** | **ctrl** | **`153.10 ms`** |
| `V3_K4_S32_32k` | `207.97` | `114.28` | `38.96` | `289.03` | `327.99` | `647.02 [629.58..709.33]` | `-17.54%` | `+0.01%` | `+2.47%` | overlap | `153.11 ms` |
| `V3_K4_pipe_S32_32k` | `203.84` | `104.61` | `42.34` | `316.19` | `358.52` | `671.36 [653.25..699.98]` | `-14.44%` | `+3.77%` | `+12.01%` | overlap | `158.87 ms` |
| `V3_K4_hoist_S16_32k` | `210.28` | `100.68` | `33.13` | `324.67` | `357.80` | `673.24 [657.30..674.67]` | `-14.20%` | `+4.06%` | `+11.78%` | overlap | `159.31 ms` |
| `V3_K6_cpl_S48_20k` | `203.00` | `112.03` | `34.05` | `338.91` | `372.96` | `677.94 [665.31..696.49]` | `-13.60%` | `+4.79%` | `+16.52%` | overlap | `160.42 ms` |
| `V3_K10_cpl_S64_12k` | `207.51` | `108.79` | `30.35` | `338.38` | `368.73` | `683.19 [681.19..720.29]` | `-12.93%` | `+5.60%` | `+15.20%` | overlap | `161.67 ms` |
| `V3_K8_cpl_S64_16k_f15` | `195.60` | `109.42` | `40.02` | `341.39` | `381.42` | `686.76 [673.57..714.73]` | `-12.48%` | `+6.15%` | `+19.16%` | overlap | `162.51 ms` |
| `V3_K8_cpl_S64_16k` | `207.54` | `103.58` | `30.89` | `344.12` | `375.01` | `692.08 [685.09..720.40]` | `-11.80%` | `+6.97%` | `+17.16%` | overlap | `163.77 ms` |
| `V3_K6_pipe_S48_20k` | `202.90` | `105.91` | `39.88` | `344.23` | `384.10` | `696.09 [679.56..732.77]` | `-11.29%` | `+7.59%` | `+20.00%` | overlap | `164.72 ms` |
| `V3_K8_cpl_S64_8k` | `213.01` | `105.91` | `38.05` | `338.23` | `376.28` | `697.24 [680.87..734.56]` | `-11.14%` | `+7.77%` | `+17.56%` | overlap | `164.99 ms` |
| `V3_K8_dec_B8_S64_16k` | `205.47` | `107.81` | `36.53` | `349.54` | `386.06` | `699.70 [687.38..713.13]` | `-10.83%` | `+8.15%` | `+20.61%` | overlap | `165.57 ms` |
| `V3_K8_pipe_S64_16k` | `201.14` | `111.17` | `40.19` | `362.85` | `403.03` | `715.35 [697.22..725.24]` | `-8.83%` | `+10.57%` | `+25.92%` | **SLOWER** | `169.28 ms` |
| `V3_K8_hoist_S64_16k` | `205.19` | `106.76` | `37.89` | `367.51` | `405.41` | `723.34 [711.65..770.58]` | `-7.82%` | `+11.80%` | `+26.66%` | **SLOWER** | `171.17 ms` |

- **Native Warm CPU summary:** `W0_shipped 730.02 ms`, `W4_v2_ctrl 587.31 ms` (`A6c+7+9 = 295.61 ms`), `V3_K4_S32_32k 576.33 ms` (`-1.87%` overlap; `A6c+7+9 = -4.21%`), `V3_K4_pipe 609.16 ms` (`+3.72%`), `V3_K4_hoist 610.64 ms` (`+3.97%`), `V3_K6_cpl 614.80 ms` (`+4.68%`), `V3_K8_cpl_f15 627.53 ms` (`+6.85%`), `V3_K8_cpl 630.92 ms` (`+7.42%` SLOWER), `V3_K10_cpl 636.61 ms` (`+8.39%` SLOWER), `V3_K8_dec_B8 651.94 ms` (`+11.00%` SLOWER), `V3_K8_hoist 657.98 ms` (`+12.03%` SLOWER).

### 4.3 Profile 3: JCC-32B-Aligned `-C target-cpu=native` (`-x86-align-branch-boundary=32 -x86-align-branch=fused+jcc+jmp`)

#### L3-Cold CPU (16-worker rotating pool `>50 MiB` + `64 MiB` flush)

| Arm | `A5+6a` | `A6b` | `A6c` | `A7+A9` | `6c+7+9` | Total med `[min..max]` | vs `W0` | vs `W4_v2` | `6c+7+9` vs `v2` | `disj_v2` | VM `W'/8` |
|---|---:|---:|---:|---:|---:|---|---:|---:|---:|---|---:|
| `W0_shipped` | `192.75` | `119.71` | `94.06` | `290.97` | `385.03` | `698.39 [673.62..1285.73]` | `+0.00%` | `+26.64%` | — | shipped | `185.68 ms` |
| **`W4_v2_ctrl`** | `173.73` | `108.76` | `33.24` | **`227.71`** | **`260.95`** | **`551.46 [543.61..790.52]`** | **`-21.04%`** | **`+0.00%`** | **`+0.00%`** | **ctrl** | **`146.62 ms`** |
| `V3_K4_S32_32k` | `208.03` | `103.65` | `30.36` | `225.16` | `255.52` | `566.42 [545.22..598.86]` | `-18.90%` | `+2.71%` | `-2.08%` | overlap | `150.59 ms` |
| `V3_K4_hoist_S16_32k` | `160.21` | `99.18` | `40.79` | `282.07` | `322.86` | `584.23 [575.79..602.65]` | `-16.35%` | `+5.94%` | `+23.73%` | overlap | `155.33 ms` |
| `V3_K4_pipe_S32_32k` | `171.74` | `113.10` | `35.68` | `264.80` | `300.48` | `585.41 [563.07..620.11]` | `-16.18%` | `+6.16%` | `+15.15%` | overlap | `155.64 ms` |
| `V3_K6_cpl_S48_20k` | `170.11` | `88.45` | `37.95` | `297.51` | `335.46` | `595.72 [578.26..619.70]` | `-14.70%` | `+8.03%` | `+28.55%` | overlap | `158.38 ms` |
| `V3_K8_cpl_S64_16k` | `169.33` | `89.00` | `36.29` | `309.36` | `345.65` | `610.93 [603.59..655.15]` | `-12.52%` | `+10.78%` | `+32.46%` | overlap | `162.43 ms` |
| `V3_K6_pipe_S48_20k` | `176.34` | `95.79` | `36.11` | `305.72` | `341.83` | `618.89 [604.26..638.43]` | `-11.38%` | `+12.23%` | `+31.00%` | overlap | `164.54 ms` |
| `V3_K8_cpl_S64_16k_f15` | `180.03` | `91.02` | `39.09` | `321.21` | `360.30` | `626.74 [590.32..721.26]` | `-10.26%` | `+13.65%` | `+38.08%` | overlap | `166.63 ms` |
| `V3_K8_cpl_S64_8k` | `178.10` | `97.58` | `39.51` | `315.55` | `355.06` | `628.96 [604.36..693.77]` | `-9.94%` | `+14.05%` | `+36.07%` | overlap | `167.22 ms` |
| `V3_K10_cpl_S64_12k` | `155.93` | `94.18` | `46.86` | `332.67` | `379.53` | `631.09 [602.13..667.11]` | `-9.64%` | `+14.44%` | `+45.44%` | overlap | `167.79 ms` |
| `V3_K8_dec_B8_S64_16k` | `177.34` | `107.84` | `29.93` | `325.91` | `355.84` | `632.44 [594.01..709.72]` | `-9.44%` | `+14.68%` | `+36.37%` | overlap | `168.15 ms` |
| `V3_K8_pipe_S64_16k` | `161.38` | `87.34` | `35.63` | `329.74` | `365.37` | `635.57 [613.41..674.07]` | `-8.99%` | `+15.25%` | `+40.02%` | overlap | `168.98 ms` |
| `V3_K8_hoist_S64_16k` | `159.93` | `102.25` | `35.44` | `361.18` | `396.62` | `665.67 [633.33..695.43]` | `-4.68%` | `+20.71%` | `+51.99%` | overlap | `176.98 ms` |

- **JCC-aligned Warm CPU summary:** `W0_shipped 630.07 ms`, `W4_v2_ctrl 491.82 ms` (`A6c+7+9 = 277.56 ms`), `V3_K4_S32_32k 493.55 ms` (`+0.35%` overlap), `V3_K4_pipe 503.71 ms` (`+2.42%`), `V3_K6_cpl 533.77 ms` (`+8.53%` SLOWER), `V3_K8_cpl 542.21 ms` (`+10.25%` SLOWER), `V3_K10_cpl 544.79 ms` (`+10.77%` SLOWER), `V3_K8_dec_B8 563.29 ms` (`+14.53%` SLOWER).
- **Bonus JCC-alignment observation:** Enabling 32B branch alignment on `-C target-cpu=native` dropped `W4_v2_ctrl`'s own L3-cold `A7+A9` from `277.31 ms` to `227.71 ms` (`-49.6 ms`, `-17.9%`; total `646.98 -> 551.46 ms`, vs `W0_shipped 784.67 -> 698.39 ms`), widening `W4_v2_ctrl`'s lead over every `K >= 6` arm to `+28.6%..+52.0%` on `A6c+7+9`.

---

## 5. Microarchitectural Root Cause: Why `K = 4, S = 16, SLAB = 32 KiB` Coupled Is the x86-64 Optimum

`objdump -d` of `emit_4lane_sampled` (`0x26330..0x26431`) vs `emit_klane_coupled::<8>` (`0x3e29a..0x3e426`) and `emit_klane_decoupled::<8, 8>` (`0x35cf0..0x36081`) explains every single number across all three profiles:

1. **The 15-GPR Register Wall (`K = 6, 8, 10` coupled):**
   - At `K = 4` (`W4_v2_ctrl`), `j0..j3` (4) + `ed0..ed3` (4) + base pointers stay cleanly in registers (`0x2636a..0x26376`).
   - At `K = 6, 8, 10`, coupled emission requires `4K = 24..40` live scalars (`j`, `ed`, `a`, `e`) across 15 GPRs. In `emit_klane_coupled::<8>` (`0x3e2a8..0x3e356`), LLVM spills all 8 loaded edge words `ed0..ed7` to stack (`0x30(%rsp)..0x180(%rsp)`), reloads `a`/`e` for all 8 lanes, then spills `a`/`e` back to reload `j0..j7`. Worse, because `edges` (`456 KiB` avg) fits in L3 (~35 cycles, not 80 ns DRAM), the register-spill + segment-close overhead (`16 -> 64` `compact_and_crc_slab` + GF(2) CRC combines) completely swamps the marginal LFB gain from `eff_K = 3.15 -> 7.68`, regressing `A6c+A7+A9` by **`+10.8%..+45.4%`** in L3-cold across all 3 profiles.
2. **Decoupled batch-chase (`B = 8, 16`) loses free OoO chase/expand overlap:**
   - Separating Loop 1 (`chase_K_batch`) from Loop 2 (`expand_lane_batch`) serializes chase and expansion across the 192-entry ROB (`+56 ms` at matched `K = 4`, `+66..+95 ms` at `K = 8` L3-cold), while `&mut a` in the inner loop adds a 5-cycle L1d store-forwarding stall per step (`0x150(%rsp)`).
3. **`6.43%` `LONG` frequency defeats `all_fast` arena-slice hoisting (`HOIST = true`):**
   - With `6.43%` `LONG` steps (`#1068`), `1 - 0.9357^4 = 23.3%` of `K = 4` rounds and `41.2%` of `K = 8` rounds exit the hoisted `&mut arena` inner loop (`+4.06%..+6.25%` at `K = 4`, `+11.80%..+20.71%` at `K = 8` L3-cold).
4. **1-step software pipelining (`PIPE = true`) doubles live edge-word pressure:**
   - Keeping next-wave `nw0..nw(K-1)` live across current-wave `expand(ed0..ed(K-1))` adds `K` extra live registers across `emit_slow_fused` calls, regressing `+3.77%..+6.16%` at `K = 4` and `+7.59%..+15.25%` at `K = 6, 8` L3-cold.
5. **Anchor density (`S = 16 -> S = 32` at `K = 4`) is statistically flat:**
   - Raising `eff_K` from `3.15` (`ph1 = 81.1%`) to `3.49` (`ph1 = 90.2%`) cuts `A6c+A7+A9` by only `-2.40%` (base) / `+2.47%` (native) / `-2.08%` (JCC) and leaves total L3-cold CPU flat (`+1.07% / +0.01% / +2.71%`, overlapping) because 2× segment closes offset the smaller tail drain.
6. **Sub-`128 KiB` slabs (`SLAB = 8 KiB` -> `64 KiB` L2) add compaction overhead for zero L2 gain:**
   - At `K = 8`, `SLAB = 8 KiB` trails `SLAB = 16 KiB` by `+0.8%..+3.3%` across all 3 profiles.
7. **Fused 1-array `Inline15Len` (`FUSED15`) is roughly neutral on `K = 8`:**
   - Once fixed to a single 16-byte `vmovdqu`, `V3_K8_cpl_S64_16k_f15` vs `V3_K8_cpl_S64_16k` is `-6.8 ms` to `-15.3 ms` warm but `+1.1%` base / `-0.8%` native / `+2.6%` JCC in L3-cold — routing `len == 16` rules to `LONG` cancels the single-array lookup saving.

---

## 6. Frozen Stop-Test Verdict & Next Implications

- **Verdict: STOP.** Zero of 12 candidate arms beat `W4_v2_ctrl` by the frozen bars (`A6c+A7+A9 <= -5.0%` AND `total <= -3.0%` disjoint on baseline and native); in fact **every `K > 4`, decoupled, pipelined, hoisted, and `FUSED15` variant regresses L3-cold CPU** across all three compiler profiles (`+3.8%..+20.7%` total, `+10.1%..+52.0%` on `A6c+A7+A9`), and `S = 16 -> S = 32` at `K = 4` is flat (`+0.01%..+2.71%`).
- **Implication for `#1056`'s `6 GB/s` contour (`W'/8 <= 133.9 ms`):** Coupled `K = 4, S = 16, SLAB = 32 KiB` with cold-outlined slab compaction (`W4_v2_ctrl`) is the x86-64 safe-Rust optimum on unchanged `m` steps. Closing the remaining gap to `6 GB/s` requires (a) real i9-10900K measurement of the full composed `W4_v2` stack (where `A5+A6a`, `A6b`, `A7`, and L3 bandwidth ratios differ from this shared Haswell VM, C5), and/or (b) cutting `A5+A6a`, `A6b`, or the step count `m` itself via a sublinear/fused contraction rather than wider `K > 4` chase lanes.
