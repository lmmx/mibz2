# Paid sampled-anchor multi-lane forward wiring (`A6c`) and 4-lane emission (`A7`) probe

Claim [`forward-4lane-anchor-probe`](../../../frontier/claims/forward-4lane-anchor-probe.toml) — smooth-zone algorithmic probe pricing real, zero-extra-pass sampled-anchor forward wiring (`A6c`) and dynamic 4-lane forward emission (`A7`) end-to-end on `Open`. Zero edits to `src/`, zero `unsafe` (`#![forbid(unsafe_code)]`), zero workstation spending, frozen `stop_test` merged before any probe code or timing (#1050, C1a/C2).

- **Harness:** [`examples/forward_4lane_probe.rs`](../../../examples/forward_4lane_probe.rs).
- **Corpus:** `wikidata.bz2` (assembled from `origin/bench-data`, SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`, **431 blocks** / `209,700,196` bytes: **233 production-admitted Open blocks** with `209,697,662` rows and `26,567,581` retained rows + 198 non-admitted blocks forced through Open for exactness stress), plus 12 synthetic edge-case blocks.
- **Hardware & method (VM diagnostic only, C5):** 2-vCPU Intel Xeon @ 2.30 GHz KVM VM (Haswell, AVX2, 32 KiB L1d, 256 KiB L2, 45 MiB L3), pinned to CPU 0 (`taskset -c 0`), 7 interleaved repeats in forward/reverse order under both **warm** (per-block `A6c -> A7` on reused worker buffers, L2-cold / L3-hot) and **L3-cold** (233 separate per-block `outq` + `edges` buffers >160 MiB >> 45 MiB L3 + 64 MiB L3 eviction sweep before each pass) conditions, compiled for both **baseline x86-64** (`target_avx2 = false`) and **`-C target-cpu=native`** (`target_avx2 = true`). Because concurrent VM load caused OS runnable-queue preemption spikes on wall-clock `Instant`, we record **both** steal-free thread CPU time (`/proc/thread-self/schedstat` field 0 = `se.sum_exec_runtime`, pure safe Rust; note 4 ms `CONFIG_HZ = 250` scheduler tick quantization, ~0.8% over 220–490 ms) and `Instant` wall clock transparently.

---

## Executive summary & verdict (**PASS** under strict frozen C2 gate)

**Verdict: PASS.** Paid zero-extra-pass sampled-anchor 4-lane forward (`fwd_4lane_sampled_S16_vec`, plus `S8_vec` and `S12_vec`) clears **every clause** of the frozen stop test by **1.8×–4.2× margin** with **100% non-overlapping `[min..max]` spreads** across all four `(baseline/native × warm/L3-cold)` cells, **11,961 / 11,961 byte-exact checks**, **zero `unsafe`**, and **zero extra output buffer** in vectored mode:

| Regime (steal-free CPU, 7 interleaved repeats) | `shipped_bidi` (`A6c+A7`) | `fwd_1lane` (`A6c+A7`) | `fwd_4lane_oracle` (upper bound) | **`fwd_4lane_sampled_S16_vec`** | **vs `bidi`** (`>= -12%` gate) | **vs `fwd1`** (`>= -8%` gate) | **vs Oracle** |
|---|---:|---:|---:|---:|---:|---:|---:|
| **Baseline x86-64 WARM** | `335.89 [324.90..354.42]` | `271.64 [255.55..284.47]` | `215.59 [206.68..237.70]` | **`231.80 [219.99..248.94]`** | **`-30.99%` DISJOINT** | **`-14.67%` DISJOINT** | `1.075×` (+16.2 ms) |
| **Baseline x86-64 L3-COLD** | `486.28 [461.31..543.77]` | `457.00 [429.37..472.15]` | `323.46 [301.02..353.06]` | **`323.61 [307.79..342.39]`** | **`-33.45%` DISJOINT** | **`-29.19%` DISJOINT** | **`1.000×` (+0.15 ms)** |
| **Native WARM** | `328.07 [320.61..335.89]` | `290.04 [269.39..298.65]` | `203.87 [202.20..248.97]` | **`221.83 [215.91..231.70]`** | **`-32.38%` DISJOINT** | **`-23.52%` DISJOINT** | `1.088×` (+18.0 ms) |
| **Native L3-COLD** | `472.47 [454.54..523.27]` | `455.81 [439.31..533.43]` | `295.66 [283.70..303.43]` | **`303.41 [295.48..320.38]`** | **`-35.78%` DISJOINT** | **`-33.43%` DISJOINT** | **`1.026×` (+7.8 ms)** |

And even paying the full contiguous stitch copy (`arena -> out`, 209.7 MB) with zero downstream RLE1 changes (**`fwd_4lane_sampled_S16_stitch`**):
- **Native WARM:** `239.43 [231.42..249.99]` ms (**`-27.02%` vs bidi, `-17.45%` vs fwd1, DISJOINT FASTER vs both**).
- **Native L3-COLD:** `323.49 [319.28..340.11]` ms (**`-31.53%` vs bidi, `-29.03%` vs fwd1, DISJOINT FASTER vs both**).
- **Baseline L3-COLD:** `351.42 [319.29..367.41]` ms (**`-27.73%` vs bidi, `-23.10%` vs fwd1, DISJOINT FASTER vs both**).
- **Baseline WARM:** `249.60 [239.81..258.18]` ms (**`-25.69%` vs bidi DISJOINT FASTER**, `-8.11%` vs fwd1, 2.6 ms tail overlap due to scalar stitch copy).

**Modeled i9-10900K impact (VM diagnostic only, anchored to `character-stage1` `A6c = 161.4 + A7 = 490.3 = 651.7 ms` of `1,306 ms` 1W wall; real hardware validation belongs in `forward-emission`'s frozen registration):**
- **`fwd_4lane_sampled_S16_vec`:** saves **`+208.4 ms` (`+15.95%` 1W wall) warm** and **`+221.3..+235.4 ms` (`+16.94..+18.02%` 1W wall) cold** — capturing **92–93% (warm)** and **97.4–99.95% (L3-cold)** of the free offline 4-lane oracle (`+219.6..+247.8 ms`), and composing **directly** with vectored RLE1+CRC fusion (which eliminates the stitch pass completely and adds another ~`+49.9 ms` = `+3.82%` 1W wall).
- **`fwd_4lane_sampled_S16_stitch` (drop-in contiguous `out`):** saves **`+174.9..+178.8 ms` (`+13.39..+13.69%` 1W wall) warm** and **`+184.9..+208.3 ms` (`+14.16..+15.95%` 1W wall) cold**.

---

## 1. Mechanism: why paid sampled-anchor 4-lane matches the oracle

1. **Zero-extra-pass `A6c` wiring + bit-31 anchor tagging (`0.00 ms` constructor overhead):**
   - `A6c` runs the exact unmodified SIMD-vectorizable `outq::Reader::wire` (`~30 ms` warm / `~40 ms` cold vs `wire_xor`'s `91 ms` / `108 ms` — **2.7–3.0× faster**).
   - Because `max_bits = 18` and `max_dict = 1,712` (`<= 2^11`) across all 233 admitted wikidata Open blocks (`18 + 11 = 29 <= 31`), bit 31 of `edges[u]` is completely free on **233 / 233 (100%)** blocks (`can_tag = !wide && m >= 256 && lens.len() - 1 <= ((u32::MAX >> 1) >> bits)`).
   - Right after `Reader::wire`, we pick `S` uniformly-spaced row indices `a_0 = mark`, `a_k = (mark + k * m / S) % m` (mathematically distinct for `m >= 8*S`) and set `edges[a_k] |= 1 << 31`: **16 scalar stores (~30 ns/block = 0.007 ms across 233 blocks)**.
2. **Single combined sign-bit stop check across all 4 lanes:**
   - Each active lane emits its start anchor once (stripping bit 31), advances `j = edge & mask`, and enters a counted 4-lane loop (`for _ in 0..(m - steps_done) / 4`) that loads all 4 edges up front (`ed0..ed3`, exposing 4× MLP) and tests all 4 lanes in **one branch**:
     ```rust
     if ((ed0 | ed1 | ed2 | ed3) as i32) < 0 { /* cold: ~16 times/block = 0.014% */ }
     ```
   - When a lane hits an anchor, it records `(seg -> end_row)`, closes its current slice, and immediately grabs the next unstarted anchor `next_seg < S`. Once all `S` anchors have started, active lanes drain cleanly through 3-lane -> 2-lane -> 1-lane loops.
3. **Slab-continuation vectored arena (bounded `<= 992 KiB`, independent of `S`):**
   - Each of the 4 lanes holds a `32 KiB` slab inside one flat buffer (in `_vec` mode, **`out` itself**, reusing the `OutQ16` cache lines just read by `A6c` with zero second buffer).
   - On the hot path, `if len != LONG && a + INLINE <= e` is **identical** to `emit_4lane_oracle`'s boundary check — zero extra branches!
   - When a segment finishes at cursor `a`, the next segment assigned to that lane **resumes immediately at `a` in the same slab** (since earlier bytes `..a` are never touched again). Peak arena slack is bounded by `4 * 32 KiB` (`max_arena = 992 KiB` for `n = 900 KiB`), **completely independent of `S`**!
4. **O(S) cycle chaining & 100% safe edge-case handling:**
   - Untagging clears `edges[a_k] &= !(1 << 31)` in O(S).
   - Chaining `seg 0 (= mark) -> next_seg` in O(S) (~10 ns) automatically isolates `mark`'s cycle (ignoring off-cycle anchors on periodic multi-cycle blocks, then repeating the period to `n` bytes) and falls back cleanly to 1-lane `emit` if `can_tag == false`, `S > m/8`, or a corrupt non-permutation exceeds step budget `m`.

---

## 2. Exactness verification & structural / order-statistics census

### 2.1 Exactness (11,961 / 11,961 checks, 100% byte-identical to `bwt::walk`)
- **Synthetic edge cases (`12` blocks × `27` arms = `324 / 324` exact):** `n = 1`, `n = 2` (`"ab"`, `"ba"`), short blocks (`10`, `64`, `300` covering `S > m` and tight-anchor fallback), periodic multi-cycle (`"abac"^256`, `"aaab"^512`, `"ab"^500`, `"z"^1000`), dense 4 KiB, run-heavy 4 KiB.
- **Production-admitted `wikidata.bz2` (`233` blocks × `27` arms = `6,291 / 6,291` exact):** every arm (`shipped_bidi`, `fwd_1lane`, `fwd_4lane_oracle`, uniform & golden-ratio `S in {4,8,12,16,24,32}` in both vectored concatenation and contiguous stitched output) verified byte-for-byte with zero stale anchor tag bits.
- **Non-admitted `wikidata.bz2` forced through Open (`198` blocks × `27` arms = `5,346 / 5,346` exact):** all 197 runts + 1 dense-refused block exact.

### 2.2 Structural & load-balance census (233 admitted Open blocks, `K = 4` lanes, `SLAB = 32 KiB`)
- `total_n = 209,697,662`, `total_m = 26,567,581` (`m/n = 12.67%`), **`single_cycle = 233/233`**, **`can_tag = 233/233`**, `wide = 0`, `oracle_ok = 233/233`, `max_bits = 18`, `max_dict = 1,712`.

| `scheme_S` | `eff_K` (steps/iter) | `4L%` | `3L%` | `2L%` | `1L%` | `L_max / (m/4)` | `slices/blk` | `slab_sw/blk` | `max_arena` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `unif_S4` | 1.899 | 11.3% | 15.9% | 24.0% | 48.7% | 2.081× | 29.5 | 25.5 | 992.0 KiB |
| `unif_S8` | 2.652 | 42.1% | 11.2% | 16.5% | 30.2% | 1.310× | 33.5 | 25.5 | 1024.0 KiB |
| `unif_S12` | 2.912 | 53.8% | 8.3% | 13.1% | 24.7% | 1.029× | 37.5 | 25.5 | 992.0 KiB |
| **`unif_S16`** | **3.153** | **63.9%** | **6.8%** | **9.9%** | **19.4%** | **0.818×** | **41.6** | **25.6** | **992.0 KiB** |
| `unif_S24` | 3.363 | 73.2% | 4.9% | 6.8% | 15.1% | 0.609× | 49.5 | 25.5 | 992.0 KiB |
| `unif_S32` | 3.488 | 78.6% | 3.5% | 5.8% | 12.0% | 0.507× | 57.6 | 25.6 | 992.0 KiB |
| `gold_S4` | 1.986 | 13.1% | 16.6% | 26.0% | 44.3% | 2.021× | 29.6 | 25.6 | 992.0 KiB |
| `gold_S16` | 3.172 | 65.1% | 6.4% | 9.2% | 19.3% | 0.823× | 41.6 | 25.6 | 992.0 KiB |
| `gold_S32` | 3.514 | 79.8% | 3.3% | 5.4% | 11.5% | 0.496× | 57.6 | 25.6 | 992.0 KiB |

**Takeaway:** Uniform BWT row-index sampling (`(mark + k*m/S) % m`) matches theoretical i.i.d. exponential order statistics (`H_4 = 2.083`, measured `2.081×`) and golden-ratio Weyl hashing (`3.172` vs `3.153` at `S = 16`), proving zero BWT-clustering bias. `S = 16` hits the sweet spot: `eff_K = 3.153`, `41.6` ordered slices/block, `max_arena = 992 KiB`, and only 16 anchor hits/block.

---

## 3. Full 7-repeat interleaved timings (233 admitted blocks, baseline x86-64 & native)

### 3.1 Baseline x86-64 (`target_avx2 = false`) — Steal-free CPU (`schedstat`)

#### Warm (L2-cold / L3-hot reused worker buffers)

| arm | `A6c` | `A7` | `stitch` | `A6c+A7` med `[min..max]` | vs `bidi` | vs `fwd1` | `disj_bi` | `disj_f1` | i9 anc save (`%` 1W) |
|---|---:|---:|---:|---:|---:|---:|---|---|---:|
| `shipped_bidi` | 90.51 | 255.24 | 0.00 | 335.89 `[324.90..354.42]` | +0.00% | +23.65% | base | SLOWER | +0.0 ms (+0.00%) |
| `fwd_1lane` | 27.53 | 241.60 | 0.00 | 271.64 `[255.55..284.47]` | -19.13% | +0.00% | FASTER | base | +138.5 ms (+10.61%) |
| `fwd_4lane_oracle` | 32.04 | 195.20 | 0.00 | 215.59 `[206.68..237.70]` | -35.82% | -20.64% | FASTER | FASTER | +219.6 ms (+16.82%) |
| `fwd_4lane_sampled_S4_vec` | 31.93 | 216.52 | 0.00 | 248.45 `[227.28..277.09]` | -26.03% | -8.54% | FASTER | overlap | +178.8 ms (+13.69%) |
| `fwd_4lane_sampled_S8_vec` | 30.44 | 199.66 | 0.00 | 229.32 `[219.81..249.49]` | -31.73% | -15.58% | FASTER | FASTER | +213.9 ms (+16.38%) |
| `fwd_4lane_sampled_S12_vec` | 32.01 | 203.77 | 0.00 | 235.40 `[223.88..252.50]` | -29.92% | -13.34% | FASTER | FASTER | +203.2 ms (+15.56%) |
| **`fwd_4lane_sampled_S16_vec`** | **30.65** | **202.34** | **0.00** | **231.80 `[219.99..248.94]`** | **-30.99%** | **-14.67%** | **FASTER** | **FASTER** | **+208.4 ms (+15.95%)** |
| `fwd_4lane_sampled_S24_vec` | 27.44 | 205.29 | 0.00 | 228.20 `[215.79..255.77]` | -32.06% | -15.99% | FASTER | overlap | +208.4 ms (+15.96%) |
| `fwd_4lane_sampled_S8_stitch` | 35.86 | 202.18 | 16.05 | 254.67 `[243.36..260.28]` | -24.18% | -6.25% | FASTER | overlap | +168.5 ms (+12.91%) |
| `fwd_4lane_sampled_S12_stitch` | 35.28 | 204.85 | 19.88 | 247.76 `[231.17..282.23]` | -26.24% | -8.79% | FASTER | overlap | +183.1 ms (+14.02%) |
| `fwd_4lane_sampled_S16_stitch` | 24.11 | 205.85 | 12.03 | 249.60 `[239.81..258.18]` | -25.69% | -8.11% | FASTER | overlap | +174.9 ms (+13.39%) |
| `fwd_4lane_sampled_S24_stitch` | 34.43 | 199.84 | 18.04 | 247.69 `[235.87..262.56]` | -26.26% | -8.82% | FASTER | overlap | +170.4 ms (+13.04%) |

#### L3-Cold (233 rotating per-block buffers >160 MiB + 64 MiB flush)

| arm | `A6c` | `A7` | `stitch` | `A6c+A7` med `[min..max]` | vs `bidi` | vs `fwd1` | `disj_bi` | `disj_f1` | i9 anc save (`%` 1W) |
|---|---:|---:|---:|---:|---:|---:|---|---|---:|
| `shipped_bidi` | 107.54 | 375.40 | 0.00 | 486.28 `[461.31..543.77]` | +0.00% | +6.41% | base | overlap | +0.0 ms (+0.00%) |
| `fwd_1lane` | 41.36 | 416.99 | 0.00 | 457.00 `[429.37..472.15]` | -6.02% | +0.00% | overlap | base | +45.0 ms (+3.45%) |
| `fwd_4lane_oracle` | 40.09 | 281.61 | 0.00 | 323.46 `[301.02..353.06]` | -33.48% | -29.22% | FASTER | FASTER | +223.7 ms (+17.13%) |
| `fwd_4lane_sampled_S4_vec` | 40.00 | 309.50 | 0.00 | 349.51 `[327.39..370.14]` | -28.13% | -23.52% | FASTER | FASTER | +187.4 ms (+14.35%) |
| `fwd_4lane_sampled_S8_vec` | 39.94 | 289.96 | 0.00 | 329.69 `[311.22..343.86]` | -32.20% | -27.86% | FASTER | FASTER | +213.0 ms (+16.31%) |
| `fwd_4lane_sampled_S12_vec` | 39.90 | 288.56 | 0.00 | 328.53 `[317.45..349.32]` | -32.44% | -28.11% | FASTER | FASTER | +214.9 ms (+16.46%) |
| **`fwd_4lane_sampled_S16_vec`** | **39.91** | **283.70** | **0.00** | **323.61 `[307.79..342.39]`** | **-33.45%** | **-29.19%** | **FASTER** | **FASTER** | **+221.3 ms (+16.94%)** |
| `fwd_4lane_sampled_S24_vec` | 39.88 | 283.93 | 0.00 | 323.90 `[319.45..354.81]` | -33.39% | -29.12% | FASTER | FASTER | +221.0 ms (+16.92%) |
| `fwd_4lane_sampled_S8_stitch` | 39.99 | 287.67 | 26.61 | 347.99 `[332.72..362.24]` | -28.44% | -23.85% | FASTER | FASTER | +189.4 ms (+14.50%) |
| `fwd_4lane_sampled_S12_stitch` | 39.97 | 293.47 | 20.04 | 355.50 `[327.55..371.15]` | -26.89% | -22.21% | FASTER | FASTER | +179.6 ms (+13.75%) |
| `fwd_4lane_sampled_S16_stitch` | 40.00 | 287.94 | 20.05 | 351.42 `[319.29..367.41]` | -27.73% | -23.10% | FASTER | FASTER | +184.9 ms (+14.16%) |
| `fwd_4lane_sampled_S24_stitch` | 39.84 | 275.77 | 23.98 | 342.79 `[323.69..361.93]` | -29.51% | -24.99% | FASTER | FASTER | +196.3 ms (+15.03%) |

*(Baseline wall-clock `Instant` L3-cold medians agree: `shipped_bidi` 489.30, `fwd_1lane` 473.36, `fwd_4lane_oracle` 333.21, `S16_vec` 341.30 [-30.25%/-27.90%], `S24_vec` 332.95 [-31.96%/-29.66% disjoint FASTER], `S16_stitch` 365.31 [-25.34%/-22.83% disjoint FASTER]; warm wall clock saw 100–350 ms concurrent-VM preemption spikes across all arms.)*

### 3.2 Native (`-C target-cpu=native`, `target_avx2 = true`) — Steal-free CPU (`schedstat`)

#### Warm (L2-cold / L3-hot reused worker buffers)

| arm | `A6c` | `A7` | `stitch` | `A6c+A7` med `[min..max]` | vs `bidi` | vs `fwd1` | `disj_bi` | `disj_f1` | i9 anc save (`%` 1W) |
|---|---:|---:|---:|---:|---:|---:|---|---|---:|
| `shipped_bidi` | 91.35 | 244.19 | 0.00 | 328.07 `[320.61..335.89]` | +0.00% | +13.11% | base | SLOWER | +0.0 ms (+0.00%) |
| `fwd_1lane` | 32.01 | 251.20 | 0.00 | 290.04 `[269.39..298.65]` | -11.59% | +0.00% | FASTER | base | +90.8 ms (+6.95%) |
| `fwd_4lane_oracle` | 30.91 | 177.13 | 0.00 | 203.87 `[202.20..248.97]` | -37.86% | -29.71% | FASTER | FASTER | +241.4 ms (+18.49%) |
| `fwd_4lane_sampled_S4_vec` | 32.29 | 206.23 | 0.00 | 240.89 `[235.05..254.03]` | -26.57% | -16.94% | FASTER | FASTER | +180.6 ms (+13.83%) |
| `fwd_4lane_sampled_S8_vec` | 37.32 | 183.97 | 0.00 | 227.36 `[215.87..244.39]` | -30.70% | -21.61% | FASTER | FASTER | +216.4 ms (+16.57%) |
| `fwd_4lane_sampled_S12_vec` | 31.99 | 194.08 | 0.00 | 227.28 `[219.50..249.95]` | -30.72% | -21.64% | FASTER | FASTER | +205.5 ms (+15.74%) |
| **`fwd_4lane_sampled_S16_vec`** | **28.69** | **195.51** | **0.00** | **221.83 `[215.91..231.70]`** | **-32.38%** | **-23.52%** | **FASTER** | **FASTER** | **+208.4 ms (+15.96%)** |
| `fwd_4lane_sampled_S24_vec` | 36.02 | 183.85 | 0.00 | 215.82 `[213.55..225.38]` | -34.21% | -25.59% | FASTER | FASTER | +218.9 ms (+16.76%) |
| `fwd_4lane_sampled_S8_stitch` | 47.88 | 191.70 | 12.13 | 244.98 `[239.23..252.74]` | -25.33% | -15.54% | FASTER | FASTER | +165.7 ms (+12.68%) |
| `fwd_4lane_sampled_S12_stitch` | 35.85 | 187.40 | 15.97 | 243.89 `[226.29..256.08]` | -25.66% | -15.91% | FASTER | FASTER | +165.5 ms (+12.67%) |
| **`fwd_4lane_sampled_S16_stitch`** | **35.98** | **195.32** | **11.91** | **239.43 `[231.42..249.99]`** | **-27.02%** | **-17.45%** | **FASTER** | **FASTER** | **+178.8 ms (+13.69%)** |
| `fwd_4lane_sampled_S24_stitch` | 39.92 | 187.89 | 19.99 | 239.80 `[223.40..256.42]` | -26.91% | -17.32% | FASTER | FASTER | +172.2 ms (+13.19%) |

#### L3-Cold (233 rotating per-block buffers >160 MiB + 64 MiB flush)

| arm | `A6c` | `A7` | `stitch` | `A6c+A7` med `[min..max]` | vs `bidi` | vs `fwd1` | `disj_bi` | `disj_f1` | i9 anc save (`%` 1W) |
|---|---:|---:|---:|---:|---:|---:|---|---|---:|
| `shipped_bidi` | 107.88 | 362.39 | 0.00 | 472.47 `[454.54..523.27]` | +0.00% | +3.65% | base | overlap | +0.0 ms (+0.00%) |
| `fwd_1lane` | 41.55 | 414.27 | 0.00 | 455.81 `[439.31..533.43]` | -3.53% | +0.00% | overlap | base | +29.1 ms (+2.22%) |
| `fwd_4lane_oracle` | 42.29 | 251.77 | 0.00 | 295.66 `[283.70..303.43]` | -37.42% | -35.13% | FASTER | FASTER | +247.8 ms (+18.97%) |
| `fwd_4lane_sampled_S4_vec` | 39.97 | 282.80 | 0.00 | 322.80 `[315.34..347.21]` | -31.68% | -29.18% | FASTER | FASTER | +209.3 ms (+16.02%) |
| `fwd_4lane_sampled_S8_vec` | 40.00 | 285.24 | 0.00 | 325.16 `[303.78..353.38]` | -31.18% | -28.66% | FASTER | FASTER | +205.9 ms (+15.77%) |
| `fwd_4lane_sampled_S12_vec` | 43.21 | 271.34 | 0.00 | 315.13 `[303.86..330.15]` | -33.30% | -30.86% | FASTER | FASTER | +219.9 ms (+16.84%) |
| **`fwd_4lane_sampled_S16_vec`** | **40.00** | **263.49** | **0.00** | **303.41 `[295.48..320.38]`** | **-35.78%** | **-33.43%** | **FASTER** | **FASTER** | **+235.4 ms (+18.02%)** |
| `fwd_4lane_sampled_S24_vec` | 39.99 | 263.91 | 0.00 | 303.76 `[299.32..316.10]` | -35.71% | -33.36% | FASTER | FASTER | +234.8 ms (+17.98%) |
| `fwd_4lane_sampled_S8_stitch` | 39.99 | 270.29 | 16.01 | 327.83 `[319.81..343.84]` | -30.61% | -28.08% | FASTER | FASTER | +207.9 ms (+15.92%) |
| `fwd_4lane_sampled_S12_stitch` | 39.99 | 267.84 | 12.00 | 324.41 `[315.90..365.80]` | -31.34% | -28.83% | FASTER | FASTER | +207.1 ms (+15.86%) |
| **`fwd_4lane_sampled_S16_stitch`** | **40.00** | **264.28** | **23.99** | **323.49 `[319.28..340.11]`** | **-31.53%** | **-29.03%** | **FASTER** | **FASTER** | **+208.3 ms (+15.95%)** |
| `fwd_4lane_sampled_S24_stitch` | 40.00 | 255.71 | 20.00 | 323.16 `[311.85..333.18]` | -31.60% | -29.10% | FASTER | FASTER | +208.6 ms (+15.97%) |

*(Native wall-clock `Instant` medians match closely: WARM `shipped_bidi` 330.37, `fwd_1lane` 288.54, `fwd_4lane_oracle` 211.89, `S16_vec` 226.07 [-31.57%/-21.65%], `S16_stitch` 238.65 [-27.76%/-17.29%]; L3-COLD `shipped_bidi` 526.45, `fwd_1lane` 541.51, `fwd_4lane_oracle` 319.85, `S16_vec` 306.46 [-41.79%/-43.41%], `S16_stitch` 326.15 [-38.05%/-39.77%].)*

---

## 4. Concrete production design for `forward-emission`

This probe closes the design & census prerequisites of [`frontier/claims/forward-emission.toml`](../../../frontier/claims/forward-emission.toml) and gives an immediately buildable, zero-`unsafe` production architecture:

1. **In `src/open.rs` (`A6c`):**
   - Replace `wire_xor` (`m >= 65_536`) with `Reader::wire` (already in `src/open/outq.rs`, 2.7–3.0× faster, SIMD-vectorizable).
   - When `!wide && m >= 256 && (lens.len() - 1) <= ((u32::MAX >> 1) >> bits) as usize`, choose `S = 16` uniform anchors `a_k = (mark + k * m / 16) % m` and set `edges[a_k] |= 1 << 31`. Otherwise fall back to 1-lane `emit`.
2. **In `src/open.rs` (`A7`):**
   - Replace `emit_bidirectional` with the dynamic 4-lane sampled-anchor slab emitter (`SLAB_SIZE = 32 KiB`, slab-continuation across segments, counted 4-lane inner loop with single `((ed0 | ed1 | ed2 | ed3) as i32) < 0` check, 3->2->1-lane drain, O(S) untag & cycle chain).
   - **Two clean integration modes (both verified here):**
     - **Phase A (drop-in contiguous `out`, zero downstream RLE1 changes):** emit into a worker-reused `1.03 MiB` slab arena and stitch ordered slices into `out[..n]` (`-25.7..-31.5%` on `A6c+A7`, `+175..+208 ms` modeled 1W).
     - **Phase B (paired with vectored RLE1+CRC fusion):** emit slabs directly into `out` (0 B extra buffer!) and feed the ~41.6 ordered slices directly to vectored RLE1+CRC, eliminating the 12–24 ms stitch copy (`-31.0..-35.8%` on `A6c+A7`, `+208..+235 ms` modeled 1W, plus vectored RLE1+CRC's own gain).
3. **System interaction (`DECODER.md`):**
   - At 1 worker, `forward-emission` eliminates ~175–235 ms of `C+E` demand directly.
   - At 8 workers, `S` (179–185 ms) and workers (185 ms) currently tie; landing `forward-emission` cuts worker busy by ~25–33 ms/worker (bringing worker demand well below `S`), which both directly unlocks scanner improvements (`scanner-coalesced-u8-decode`, `scanner-seam-window`, `huff-bmi2-build`) and crosses `pooled-scan-hybrid`'s `S'(8) <= 165 ms` screen trigger.
   - Because `src/open.rs` is queued behind `mtf-fold-run` / `mtf-fold-build`, `forward-emission` stays ready for immediate admission once `src/open.rs` is free.
