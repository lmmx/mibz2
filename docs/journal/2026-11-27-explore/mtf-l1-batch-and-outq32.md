# L1d-Batched Packed Maximal MTF Fold, Hoisted Single-Branch `OutQ16`, `OutQ32`, and Intel JCC Alignment on Wikidata

*Date:* 27 November 2026
*Zone:* Smooth (`examples/outq_hoist_and_rle_probe.rs`, zero `src/` edits)
*Primary corpus:* `corpora/wikidata.bz2` (11,854,933 B compressed; 209,700,196 B decoded; SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`; 431 blocks → 233 Open-admitted blocks, 0 non-injective active `seq_to_unseq`, avg `in_use = 88.1`, `14,386,581` thin runs, `max = 179,404` thin runs/block, `26,567,581` thin rows, `avg_w = 1.847`)
*Machine:* 2-vCPU Haswell Xeon @ 2.30 GHz (32 KiB L1d / 256 KiB L2 / 45 MiB L3, no vPMU), Linux `/proc/thread-self/schedstat` on-CPU thread time, 1 warmup + 7 interleaved repeats (alternating arm order per repeat).

## 1. Questions Tested

Following `docs/journal/2026-11-27-explore/mtf-fold-and-outq-dissection.md` (`#1048`) and `frontier/claims/mtf-fold-l1-batch-redesign.toml` (`idea`), we tested four concrete questions before any striated `src/` build spends an implementation slot or an i9 run:

1. **Does `mtf-fold-l1-batch-redesign`'s `[u32; 256]` L1d batch + `OutQ16<496, 512>` guard zone clear its `>= 2.5%` local 1W gate on its own?**
2. **What is the true 2×3 factorial decomposition** between (a) the L1d-batched packed maximal MTF fold, (b) the `OutQ16` power-of-2 guard zone (`496/512` and `1008/1024`), and (c) **batch-hoisted `p`/epoch checks + single-branch `OVF_BIT` `OutQ16::push_hoisted` + register-local `m`** (100% safe Rust)?
3. **Can run-encoded `OutQ32` (`(m << 4) | (w - 1)`, 4 B/run, 8× less write traffic than `OutQ16`'s 32 B iota, zero epochs/cuts) beat `OutQ16` when `A6c` (`wire_xor` bidi and `wire_fwd` forward) is paid in full?**
4. **How much of the observed variance across `OutQ16` loop variants is real algorithmic gain vs Intel JCC-erratum 32-byte branch alignment on Haswell/Comet Lake?**

## 2. Exactness & Safety Verification

Before every timing run, all 17 arms passed 100% exact verification on:
- **All 233 Open blocks of `corpora/wikidata.bz2`**: exact `First` (`thick_rows`, `thin`, `thick`, `m`, `mark`, `runs`), exact `ns` and `edges[..3*ns]` thick strips, exact drained `OutQ` rank sequence per byte `0..256`, exact `A6c` `wire_xor` (`edges[0..m]` + `predecessor`), and exact `A6c` forward `wire_fwd` (`edges[0..m]`) on real Wikidata `bands` (constructed via `src/open.rs::Kernel::trace`). Every `OutQ16` (`496/512`, `1008/1024`) and `OutQ32` (`256`, `512`) arm produced **`0` overflow chunks** across all 233 blocks.
- **27 synthetic stress cases (`synthetic_stress_tests`)**: non-injective `seq_to_unseq` detection, tiny `out` (`1024` / `4096` B) forcing `>100` `OVF_BIT` overflow chunks through `push_hoisted` / `push_slow_tagged`, exact `EPOCH = 65536` straddling inside a 256-run batch, and marked-row `p` sweeps across thin/thick/batch-boundary positions (`0, 1, 255, 256, 511, 512, 65535, 65536, n-1`).
- **Guard-zone invariant catch (`f + w <= C`, not `f < C`)**: With usable capacity `C = 496` and stride `S = 512`, `push` must test `f + w <= C` (not `f < C`), so `self.fill[a]` never exceeds `C` and `Reader16` reads a constant `C = 496` ranks/chunk without a per-chunk fill table. Because `w >= 1`, `f + w <= 496` guarantees `f <= 495` and `f + 16 <= 511 < 512`, keeping the 32 B iota write strictly inside the chunk's 1024 B stride. Encoding `ovf` chunks in `fill[a]` bit 31 (`OVF_BIT`) makes `let nf = f.wrapping_add(w); if nf <= C as u32` check uninitialized (`fill[a] = C`), full/straddling (`f + w > C`), and overflow (`OVF_BIT`) in **one single add + compare**.

## 3. Crucial Methodological Discovery: 20–40 ms Intel JCC-Erratum Alignment Swings

Both this Haswell Xeon VM and Louis's Comet Lake i9-10900K carry the Intel JCC-erratum microcode mitigation (conditional jumps crossing or ending on a 32-byte boundary drop out of the DSB uop cache). On `Pass16` / `OutQ16` (14.39M tight loop iterations):

- On default `-C target-cpu=native`, `flush_batch_hoisted_rows` (`0x3480e..0x348b5`) and `flush_batch_avx2` (`0x338be..0x33965`) emitted **instruction-for-instruction identical machine code**, yet differed by **27.9 ms** (`135.92 ms` vs `163.82 ms`) in Run 1, and flipped by **20.9 ms** (`163.87 ms` vs `143.02 ms`) in Run 2 when adding `run_shipped_q496` shifted code addresses. `objdump -d` traced the slow instance directly to the loop backedge `cmp %rbx,%r11; jne` at `0x3395c..0x33965` crossing the `0x33960` 32-byte boundary.
- Recompiling with **`RUSTFLAGS="-C target-cpu=native -C llvm-args=-x86-align-branch-boundary=32 -C llvm-args=-x86-align-branch=fused+jcc+jmp"`** collapsed the difference between the two byte-identical functions from **20–28 ms down to `0.04 ms` (`127.58 ms` vs `127.62 ms`, `0.03%`)** and cut `shipped_split` itself from `195.76 ms` to `155.81 ms` (`-40.0 ms`).

All design conclusions below are anchored in the **JCC-aligned profile** (true alignment-invariant algorithmic cost), with unaligned baseline x86-64 and native profiles reported beside it.

## 4. Results

### 4.1 Part 1 — `A5 + A6a` (233 Open blocks, 7 interleaved repeats, `schedstat` CPU ms)

| Arm | Baseline x86-64 (med / paired ratio) | Native (med / paired ratio) | **JCC-Aligned Native (med [min..max] / paired)** | JCC Delta |
|---|---:|---:|---:|---:|
| `shipped_split` (`512/512`) | `199.69 ms` (`1.000x`) | `195.76 ms` (`1.000x`) | **`155.81 ms`** `[147.74..162.06]` (`1.0000x`) | `+0.00 ms` |
| `shipped_q496` (`496/512`, no fold) | `191.69 ms` (`0.960x`) | `187.87 ms` (`0.923x`) | **`151.42 ms`** `[147.82..159.82]` (`0.9732x`, `[0.967..1.027]`) | `-4.39 ms` (`-2.68%`) |
| `shipped_q496_hoist` (no fold, hoisted) | `167.91 ms` (`0.842x`) | `155.89 ms` (`0.796x`) | **`139.96 ms`** `[139.18..144.70]` (`0.9046x`, `[0.888..0.973]`) | **`-15.85 ms` (`-9.54%`)** |
| `direct_fold_1010` (`#1010` inline) | `199.31 ms` (`1.001x`) | `245.04 ms` (`1.225x`) | **`183.89 ms`** `[179.68..198.68]` (`1.2036x`, `[1.150..1.263]`) | **`+28.08 ms` (`+20.36%`)** |
| `batch256_q512` (batch only) | `179.82 ms` (`0.902x`) | `155.93 ms` (`0.781x`) | **`147.85 ms`** `[143.83..151.70]` (`0.9480x`, `[0.911..1.001]`) | `-7.96 ms` (`-5.20%`) |
| `batch128_q496` | `175.94 ms` (`0.896x`) | `159.86 ms` (`0.817x`) | `139.97 ms` `[139.58..149.85]` (`0.9246x`, `[0.875..0.945]`) | `-15.84 ms` (`-7.54%`) |
| `batch256_q496` (`#1048` proposal) | `154.94 ms` (`0.775x`) | `159.87 ms` (`0.792x`) | **`138.63 ms`** `[135.86..147.85]` (`0.8946x`, `[0.859..0.924]`) | `-17.18 ms` (`-10.54%`) |
| `batch512_q496` | `159.83 ms` (`0.801x`) | `163.78 ms` (`0.834x`) | `142.59 ms` `[135.87..145.06]` (`0.9069x`, `[0.872..0.945]`) | `-13.22 ms` (`-9.31%`) |
| `batch1024_q496` | `158.89 ms` (`0.796x`) | `155.76 ms` (`0.792x`) | `139.76 ms` `[131.86..148.66]` (`0.8949x`, `[0.871..0.954]`) | `-16.05 ms` (`-10.51%`) |
| `batch256_q1008` (`1008/1024`) | `159.80 ms` (`0.801x`) | `155.84 ms` (`0.792x`) | `139.94 ms` `[135.92..152.75]` (`0.9041x`, `[0.861..0.980]`) | `-15.86 ms` (`-9.59%`) |
| **`batch256_q496_hoist`** | `163.87 ms` (`0.819x`) | `163.87 ms` (`0.832x`) | **`127.58 ms`** `[123.54..131.82]` (`0.8191x`, `[0.762..0.892]`) | **`-28.22 ms` (`-18.09%`)** |
| **`batch256_q1008_hoist`** | `127.54 ms` (`0.634x`) | `139.84 ms` (`0.719x`) | **`123.83 ms`** `[122.86..127.90]` (`0.7920x`, `[0.762..0.866]`) | **`-31.98 ms` (`-20.80%`)** |
| `batch256_q496_avx2` | `123.91 ms` (`0.621x`) | `143.02 ms` (`0.715x`) | **`127.62 ms`** `[123.88..131.56]` (`0.8158x`, `[0.764..0.866]`) | `-28.19 ms` (`-18.42%`) |
| `shipped_outq32` (A5+A6a only) | `154.84 ms` (`0.776x`) | `163.50 ms` (`0.835x`) | `138.65 ms` `[135.55..147.92]` (`0.8894x`, `[0.836..0.974]`) | `-17.16 ms` (`-11.06%`) |
| `batch256_outq32_hoist` (A5+A6a only) | `154.50 ms` (`0.761x`) | `147.95 ms` (`0.761x`) | `123.87 ms` `[119.87..130.11]` (`0.7933x`, `[0.764..0.839]`) | `-31.94 ms` (`-20.67%`) |

### 4.2 Part 2 — Joint `A5 + A6a + A6c` on Real Wikidata Bands (7 interleaved repeats)

| Arm | Baseline x86-64 (med / ratio) | Native (med / ratio) | **JCC-Aligned Native (med [min..max] / paired)** | JCC Delta |
|---|---:|---:|---:|---:|
| `bidi_shipped_q512` | `293.43 ms` (`1.000x`) | `268.15 ms` (`1.000x`) | **`223.70 ms`** `[218.92..233.59]` (`1.0000x`) | `+0.00 ms` |
| `bidi_shipped_q496_h` (no fold) | `251.33 ms` (`0.855x`) | `235.81 ms` (`0.868x`) | `211.18 ms` `[207.76..211.91]` (`0.9315x`, `[0.897..0.968]`) | `-12.52 ms` (`-6.85%`) |
| `bidi_batch256_q496` (no hoist) | `243.43 ms` (`0.823x`) | `235.84 ms` (`0.881x`) | `210.95 ms` `[203.88..213.14]` (`0.9357x`, `[0.905..0.965]`) | `-12.75 ms` (`-6.43%`) |
| **`bidi_batch256_q496_h`** | `251.85 ms` (`0.866x`) | `240.94 ms` (`0.909x`) | **`195.43 ms`** `[191.78..208.32]` (`0.8580x`, `[0.836..0.949]`) | **`-28.27 ms` (`-14.20%`)** |
| `bidi_batch256_outq32` | `347.79 ms` (`1.185x`) | `335.66 ms` (`1.258x`) | `310.09 ms` `[303.77..320.16]` (`1.4057x`, `[1.300..1.431]`) | **`+86.39 ms` (`+40.57%`)** |
| `fwd_shipped_q512` | `227.51 ms` (`1.000x`) | `227.81 ms` (`1.000x`) | **`179.82 ms`** `[175.52..183.63]` (`1.0000x`) | `+0.00 ms` |
| `fwd_shipped_q496_h` (no fold) | `194.92 ms` (`0.858x`) | `183.92 ms` (`0.807x`) | `167.21 ms` `[163.85..170.32]` (`0.9320x`, `[0.892..0.970]`) | `-12.61 ms` (`-6.80%`) |
| `fwd_batch256_q496` (no hoist) | `183.89 ms` (`0.822x`) | `183.86 ms` (`0.807x`) | `163.82 ms` `[163.25..170.16]` (`0.9123x`, `[0.894..0.946]`) | `-16.00 ms` (`-8.77%`) |
| **`fwd_batch256_q496_h`** | `195.83 ms` (`0.860x`) | `187.91 ms` (`0.825x`) | **`154.79 ms`** `[150.92..155.54]` (`0.8466x`, `[0.824..0.886]`) | **`-25.03 ms` (`-15.34%`)** |
| `fwd_batch256_outq32` | `307.71 ms` (`1.357x`) | `303.76 ms` (`1.316x`) | `241.99 ms` `[231.84..248.70]` (`1.3458x`, `[1.293..1.417]`) | **`+62.18 ms` (`+34.58%`)** |

## 5. Disassembly Audit (`objdump -d`)

- **Zero `f0`/`f1` spills in `front_avx2_batched` hot loop (both baseline and native)**: `f0`/`f1` stay pinned in `%xmm10`/`%xmm9` (baseline `0x239ab..0x239c4`) and `%xmm9`/`%xmm10` (native `0x2261b..0x22634`) across all 256 literals in a batch; they are saved/reloaded only once every 256 literals around `call flush_batch_hoisted_rows` (`~59k` calls across 233 blocks, vs `15.14M` per-literal spills in `#1010`).
- **Fused Pass-1 `lits` popcount** eliminates both the separate `f.lit` popcount pass and per-literal `recs += 1` inside Pass 2 (`recs = lits + extra_recs`).
- **Register-clean `flush_batch_hoisted_rows`**: keeping `m` in a local register (`%r8d`), `iota` constants in `%xmm2/%xmm3` (baseline) or `%ymm1` (native), hoisting `p` (`cur > p || cur + batch_rows <= p`) and `EPOCH` (`lo + max_m_add < EPOCH`) to once per 256 runs, and collapsing `OutQ16::push` to one `nf <= C` check (`OVF_BIT`) cuts the thin-run fast path to 15 instructions (of which 5 are the safe `self.out[at..at+32]` slice bounds check).

## 6. Conclusions & Concrete Implications

1. **Refinement required for `mtf-fold-l1-batch-redesign`:** On JCC-aligned native, `[u32; 256]` + `496/512` *without* hoisting (`batch256_q496`) saves `-17.18 ms` on `A5+A6a` (`-12.75 ms` bidi / `-16.00 ms` fwd) — only `~1.85–2.49%` of 1W `wkr_cpu_ms` (`~690 ms`), right on or below `mtf-fold/registration.md`'s `2.5%` K1 bar. **Adding batch-hoisted `p`/`EPOCH` + `OVF_BIT` single-branch `push_hoisted` + local `m` (`batch256_q496_hoist` / `batch256_q1008_hoist`) roughly doubles the alignment-invariant gain to `-28.22..-31.98 ms` (`-18.1%..-20.8%` of `A5+A6a`, `-14.2%..-15.3%` of `A5+A6a+A6c`, `~-4.1%..-4.6%` of 1W worker CPU; derived i9 `wikidata x4` projection: `~-60..-69 ms` at 1W [`-4.6%..-5.3%` wall], `~-8.6..-9.9 ms/w` at 8W)** — clearing K1 with nearly 2× headroom while remaining 100% safe Rust in `Pass`/`OutQ16`.
2. **Standalone safe-Rust opportunity (`shipped_q496_hoist`):** Even without `mtf-fold`, `OutQ16<496, 512>` + `OVF_BIT` single-branch `push_hoisted` + `epoch_end` + local `m` delivers **`-15.85 ms` (`-9.54%`)** on `A5+A6a` and **`-12.5..-12.6 ms` (`-6.8%`)** on `A5+A6a+A6c` (`-31.8..-39.9 ms` unaligned) with zero `unsafe`, zero feature flags, and zero `text` fallback cost.
3. **Scoped STOP for run-encoded `OutQ32` (`(m << 4) | (w - 1)`):** Despite cutting `A5+A6a` by `-20.7%` (`-31.9 ms`) and eliminating 8× write traffic and `EPOCH`/`cuts`, `OutQ32` blows up `A6c` by `+94..+118 ms` (net **`+40.6%` bidi / `+34.6%` forward**) by destroying sequential `chunks_exact(2)` / SIMD `vpmovzxwd` wiring. Do not pursue run-encoded `OutQ` formats.
4. **Benchmarking hygiene:** Whenever comparing tight `OutQ16` / `A5` / `A6a` loops on Haswell or Comet Lake, include a JCC-aligned control (`-C llvm-args=-x86-align-branch-boundary=32 -C llvm-args=-x86-align-branch=fused+jcc+jmp`) alongside default builds so 20–40 ms 32-byte alignment swings are never mistaken for algorithmic gains or regressions.
